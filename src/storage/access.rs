//! Cooperative directory leases. A held store is deliberately not a lease.
use super::{blobs, io_error, RsStoreError};
#[cfg(unix)]
use std::os::unix::{
    fs::{MetadataExt, OpenOptionsExt},
    io::AsRawFd,
};
use std::{
    cell::Cell,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

thread_local! { static COMMAND_DEPTH: Cell<u32> = const { Cell::new(0) }; }

pub(crate) fn diagnostic(reason: &str, detail: impl std::fmt::Display) -> RsStoreError {
    let message = format!("location_reason={reason}; {detail}");
    match reason {
        "invalid_input" => RsStoreError::InvalidInput(message),
        "binding_missing" | "not_found" => RsStoreError::NotFound(message),
        "unsafe_path" => RsStoreError::UnsafePath(message),
        "migration_busy" | "coordination_unavailable" => RsStoreError::Io {
            operation_id: None,
            message,
        },
        _ => RsStoreError::Integrity(message),
    }
}

pub(crate) fn reason(error: &RsStoreError) -> Option<&str> {
    let message = match error {
        RsStoreError::InvalidInput(s)
        | RsStoreError::Integrity(s)
        | RsStoreError::UnsafePath(s)
        | RsStoreError::NotFound(s)
        | RsStoreError::UnsupportedFormat(s) => s,
        RsStoreError::Io { message, .. } | RsStoreError::Database { message, .. } => message,
        _ => return None,
    };
    let code = message.strip_prefix("location_reason=")?.split_once(';')?.0;
    (!code.is_empty() && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')).then_some(code)
}

pub(crate) fn absolute_root(root: &Path) -> Result<PathBuf, RsStoreError> {
    if root.as_os_str().is_empty() {
        return Err(diagnostic("invalid_input", "empty store root"));
    }
    let root = if root.is_absolute() {
        root.to_owned()
    } else {
        std::env::current_dir()?.join(root)
    };
    if fs::symlink_metadata(&root).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(diagnostic(
            "unsafe_path",
            "final store root is a symbolic link",
        ));
    }
    let parent = root
        .parent()
        .ok_or_else(|| diagnostic("unsafe_path", "root has no parent"))?;
    let parent = fs::canonicalize(parent).map_err(io_error)?;
    let child = root
        .file_name()
        .ok_or_else(|| diagnostic("unsafe_path", "root has no child component"))?;
    if root.exists() {
        fs::canonicalize(root).map_err(io_error)
    } else {
        Ok(parent.join(child))
    }
}

#[derive(Debug)]
struct PinnedDirectory {
    path: PathBuf,
    identity: (u64, u64),
    file: File,
}

impl PinnedDirectory {
    #[cfg(unix)]
    fn open(path: &Path) -> Result<Self, RsStoreError> {
        let path = fs::canonicalize(path).map_err(io_error)?;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_DIRECTORY)
            .open(&path)
            .map_err(io_error)?;
        let meta = file.metadata()?;
        if !meta.is_dir() || meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o022 != 0 {
            return Err(diagnostic(
                "unsafe_path",
                format!("unsafe lease directory: {}", path.display()),
            ));
        }
        blobs::check_acl(&path)?;
        Ok(Self {
            path,
            identity: blobs::identity(&meta),
            file,
        })
    }
    #[cfg(not(unix))]
    fn open(_: &Path) -> Result<Self, RsStoreError> {
        Err(diagnostic(
            "coordination_unavailable",
            "directory leases require a supported Unix filesystem",
        ))
    }
    fn verify(&self) -> Result<(), RsStoreError> {
        let meta =
            fs::symlink_metadata(&self.path).map_err(|e| diagnostic("identity_unprovable", e))?;
        if meta.file_type().is_symlink() || blobs::identity(&meta) != self.identity {
            return Err(diagnostic(
                "identity_unprovable",
                format!("lease directory replaced: {}", self.path.display()),
            ));
        }
        Ok(())
    }
}

/// Owns the whole ordered set; no descriptor is acquired after construction.
#[derive(Debug)]
pub(crate) struct AccessLease {
    directories: Vec<PinnedDirectory>,
}
impl AccessLease {
    pub(crate) fn acquire(paths: &[PathBuf]) -> Result<Arc<Self>, RsStoreError> {
        Self::acquire_attempts(paths, 8)
    }
    fn acquire_attempts(paths: &[PathBuf], attempts: usize) -> Result<Arc<Self>, RsStoreError> {
        if COMMAND_DEPTH.with(|n| n.get() != 0) {
            return Err(diagnostic(
                "migration_busy",
                "reentrant public access during a command lease",
            ));
        }
        let start = Instant::now();
        for attempt in 0..attempts {
            let mut dirs = paths
                .iter()
                .map(|p| PinnedDirectory::open(p))
                .collect::<Result<Vec<_>, _>>()?;
            dirs.sort_by(|a, b| (a.identity, &a.path).cmp(&(b.identity, &b.path)));
            for pair in dirs.windows(2) {
                if pair[0].identity == pair[1].identity && pair[0].path != pair[1].path {
                    return Err(diagnostic(
                        "identity_unprovable",
                        "multiple mount aliases for lease directory",
                    ));
                }
            }
            dirs.dedup_by(|a, b| a.identity == b.identity);
            let mut acquired = 0;
            let mut busy = false;
            for dir in &dirs {
                #[cfg(unix)]
                if unsafe { libc::flock(dir.file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0
                {
                    let e = std::io::Error::last_os_error();
                    busy = e.raw_os_error() == Some(libc::EWOULDBLOCK);
                    if !busy {
                        return Err(diagnostic("coordination_unavailable", e));
                    }
                    break;
                }
                acquired += 1;
            }
            if !busy {
                let lease = Arc::new(Self { directories: dirs });
                lease.verify()?;
                return Ok(lease);
            }
            for dir in dirs[..acquired].iter().rev() {
                unlock(&dir.file);
            }
            drop(dirs);
            if start.elapsed() >= Duration::from_millis(250) {
                break;
            }
            std::thread::sleep(Duration::from_millis(
                1 + ((attempt * 7 + std::process::id() as usize) % 16) as u64,
            ));
        }
        Err(diagnostic(
            "migration_busy",
            "ordered directory lease acquisition exhausted",
        ))
    }
    pub(crate) fn verify(&self) -> Result<(), RsStoreError> {
        for dir in &self.directories {
            dir.verify()?;
        }
        Ok(())
    }
    pub(crate) fn contains(&self, path: &Path) -> bool {
        self.directories.iter().any(|d| d.path == path)
    }
}
fn unlock(file: &File) {
    #[cfg(unix)]
    unsafe {
        libc::flock(file.as_raw_fd(), libc::LOCK_UN);
    }
}
impl Drop for AccessLease {
    fn drop(&mut self) {
        for dir in self.directories.iter().rev() {
            unlock(&dir.file);
        }
    }
}

pub(crate) struct CommandScope;
impl CommandScope {
    pub(crate) fn enter() -> Self {
        COMMAND_DEPTH.with(|n| n.set(n.get() + 1));
        Self
    }
}
impl Drop for CommandScope {
    fn drop(&mut self) {
        COMMAND_DEPTH.with(|n| n.set(n.get().saturating_sub(1)));
    }
}

pub(crate) fn acquire(
    root: &Path,
    project: Option<&Path>,
) -> Result<Arc<AccessLease>, RsStoreError> {
    let root = absolute_root(root)?;
    let started = Instant::now();
    for parent in root.parent().unwrap().ancestors() {
        if parent.join("format.json").is_file() && parent.join("store.db").is_file() {
            return Err(diagnostic(
                "unsafe_path",
                "a store cannot be nested inside another store",
            ));
        }
    }
    for attempt in 0..8 {
        let before = discover(&root, project)?;
        match AccessLease::acquire_attempts(&before.paths, 1) {
            Ok(lease) => {
                let after = discover(&root, project)?;
                if before == after && after.paths.iter().all(|p| lease.contains(p)) {
                    return Ok(lease);
                }
                drop(lease);
            }
            Err(error) if reason(&error) == Some("migration_busy") => {}
            Err(error) => return Err(error),
        }
        if started.elapsed() >= Duration::from_millis(250) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1 + (attempt * 7 % 16) as u64));
    }
    Err(diagnostic(
        "migration_busy",
        "coordination changed or remained busy during bounded discovery",
    ))
}

#[derive(PartialEq, Eq)]
struct Discovery {
    paths: Vec<PathBuf>,
    observations: Vec<(PathBuf, String)>,
}
fn discover(root: &Path, project: Option<&Path>) -> Result<Discovery, RsStoreError> {
    use std::collections::{BTreeSet, VecDeque};
    let start = Instant::now();
    let mut roots = VecDeque::from([root.to_owned()]);
    let mut projects = VecDeque::new();
    if let Some(project) = project {
        projects.push_back(fs::canonicalize(project)?);
    }
    let mut seen_roots = BTreeSet::new();
    let mut seen_projects = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut observations = vec![];
    while !roots.is_empty() || !projects.is_empty() {
        if seen_roots.len() + seen_projects.len() > 256
            || start.elapsed() > Duration::from_millis(250)
        {
            return Err(diagnostic(
                "migration_busy",
                "coordination discovery budget exhausted",
            ));
        }
        if let Some(project) = projects.pop_front() {
            if !seen_projects.insert(project.clone()) {
                continue;
            }
            paths.insert(project.clone());
            roots.extend(
                crate::project_backend::candidates(&project, None)
                    .map_err(|e| diagnostic("backend_ambiguous", e))?,
            );
        } else if let Some(root) = roots.pop_front() {
            let root = absolute_root(&root)?;
            if !seen_roots.insert(root.clone()) {
                continue;
            }
            paths.insert(root.parent().unwrap().to_owned());
            observations.push((root.clone(), super::migration::observation(&root)?));
            paths.extend(super::migration::peers(&root)?);
            let (related, project) = super::migration::related(&root)?;
            roots.extend(related);
            if let Some(project) = project {
                projects.push_back(project);
            }
        }
    }
    observations.sort();
    Ok(Discovery {
        paths: paths.into_iter().collect(),
        observations,
    })
}

pub(crate) fn validate(root: &Path, mutation: bool) -> Result<(), RsStoreError> {
    super::migration::guard(root, mutation)
}
