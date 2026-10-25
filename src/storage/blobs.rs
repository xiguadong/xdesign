use super::{
    io_error, sha256, valid_digest, valid_id, valid_uri, RsStoreError, MAX_MATERIAL_BYTES,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, DirBuilder, File, Metadata, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};

#[cfg(target_os = "macos")]
const O_NOFOLLOW: i32 = 0x100;
#[cfg(any(target_os = "linux", target_os = "android"))]
const O_NOFOLLOW: i32 = 0x20000;
#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
const O_NOFOLLOW: i32 = 0x100;
#[cfg(not(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)))]
const O_NOFOLLOW: i32 = 0;

#[cfg(unix)]
extern "C" {
    fn geteuid() -> u32;
}

// On macOS a plain fsync does not force the data out of the device cache;
// F_FULLFSYNC does. SQLite gets the same guarantee through the fullfsync
// pragma configured with the database connection. The command value comes
// from libc because it has changed between macOS versions.
#[cfg(target_os = "macos")]
pub(crate) fn sync_durable(file: &File) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) } == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn sync_durable(file: &File) -> std::io::Result<()> {
    file.sync_all()
}

fn path_label(path: &Path) -> String {
    path.display().to_string()
}

pub(crate) fn canonical_existing(path: &Path) -> Result<PathBuf, RsStoreError> {
    if path.as_os_str().is_empty() {
        return Err(RsStoreError::InvalidInput("path must not be empty".into()));
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            RsStoreError::NotFound(format!("path does not exist: {}", path_label(path)))
        }
        _ => io_error(error),
    })?;
    if metadata.file_type().is_symlink() {
        return Err(RsStoreError::UnsafePath(format!(
            "symbolic links are not accepted: {}",
            path_label(path)
        )));
    }
    fs::canonicalize(path).map_err(io_error)
}

pub(crate) fn canonical_for_creation(path: &Path) -> Result<PathBuf, RsStoreError> {
    if path.as_os_str().is_empty() {
        return Err(RsStoreError::InvalidInput("path must not be empty".into()));
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io_error)?.join(path)
    };
    let name = absolute.file_name().ok_or_else(|| {
        RsStoreError::InvalidInput(format!("path has no final component: {}", path_label(path)))
    })?;
    if name == "." || name == ".." {
        return Err(RsStoreError::InvalidInput(
            "invalid final path component".into(),
        ));
    }
    let parent = absolute.parent().ok_or_else(|| {
        RsStoreError::InvalidInput(format!("path has no parent: {}", path_label(path)))
    })?;
    // Every existing ancestor of the final component must be a real
    // directory: a symlink anywhere in the chain would let the write land
    // outside the validated destination. Walking must therefore continue
    // past the first existing ancestor up to the filesystem root. Stopping at
    // the first existing directory would miss an earlier symlink whenever
    // that directory is reached through it (for example `alias -> actual`
    // with `actual/nested` already present, so `alias/nested` stats as a
    // directory).
    let mut ancestor = parent.to_path_buf();
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(&ancestor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(RsStoreError::UnsafePath(format!(
                        "symbolic links are not accepted in the destination path: {}",
                        path_label(&ancestor)
                    )));
                }
                if !metadata.is_dir() {
                    return Err(RsStoreError::UnsafePath(format!(
                        "destination ancestor is not a directory: {}",
                        path_label(&ancestor)
                    )));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| {
                            RsStoreError::InvalidInput(format!(
                                "invalid destination: {}",
                                path_label(path)
                            ))
                        })?
                        .to_os_string(),
                );
            }
            Err(error) => return Err(io_error(error)),
        }
        match ancestor.parent() {
            Some(higher) => ancestor = higher.to_path_buf(),
            None => break,
        }
    }
    if !missing.is_empty() {
        return Err(RsStoreError::NotFound(format!(
            "destination parent does not exist: {}",
            path_label(parent)
        )));
    }
    let parent = fs::canonicalize(parent).map_err(io_error)?;
    Ok(parent.join(name))
}

#[cfg(unix)]
fn check_owner(metadata: &Metadata, path: &Path) -> Result<(), RsStoreError> {
    let effective = unsafe { geteuid() };
    if metadata.uid() != effective {
        return Err(RsStoreError::UnsafePath(format!(
            "object is not owned by the effective user: {}",
            path_label(path)
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_owner(_metadata: &Metadata, _path: &Path) -> Result<(), RsStoreError> {
    Ok(())
}

#[cfg(unix)]
fn check_mode(metadata: &Metadata, mode: u32, path: &Path) -> Result<(), RsStoreError> {
    if metadata.permissions().mode() & 0o7777 != mode {
        return Err(RsStoreError::UnsafePath(format!(
            "object does not have required mode {mode:04o}: {}",
            path_label(path)
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_mode(_metadata: &Metadata, _mode: u32, _path: &Path) -> Result<(), RsStoreError> {
    Ok(())
}

pub(crate) fn store_dir(path: &Path) -> Result<Metadata, RsStoreError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RsStoreError::UnsafePath(format!(
            "expected a non-symlink directory: {}",
            path_label(path)
        )));
    }
    check_owner(&metadata, path)?;
    check_mode(&metadata, 0o700, path)?;
    Ok(metadata)
}

pub(crate) fn store_file(path: &Path, mode: u32) -> Result<Metadata, RsStoreError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RsStoreError::UnsafePath(format!(
            "expected a non-symlink regular file: {}",
            path_label(path)
        )));
    }
    check_owner(&metadata, path)?;
    check_mode(&metadata, mode, path)?;
    Ok(metadata)
}

pub(crate) fn create_store_dir(path: &Path) -> Result<(), RsStoreError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            store_dir(path)?;
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(error)),
    }
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(path).map_err(io_error)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    store_dir(path)?;
    if let Some(parent) = path.parent() {
        fsync_dir(parent)?;
    }
    Ok(())
}

fn open_no_follow(path: &Path, mode: Option<u32>, owned: bool) -> Result<File, RsStoreError> {
    let before = fs::symlink_metadata(path).map_err(io_error)?;
    if before.file_type().is_symlink() || !before.is_file() {
        return Err(RsStoreError::UnsafePath(format!(
            "expected a regular non-symlink file: {}",
            path_label(path)
        )));
    }
    if owned {
        check_owner(&before, path)?;
    }
    if let Some(mode) = mode {
        check_mode(&before, mode, path)?;
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(O_NOFOLLOW);
    let file = options.open(path).map_err(io_error)?;
    let after = file.metadata().map_err(io_error)?;
    if !after.is_file() {
        return Err(RsStoreError::UnsafePath(format!(
            "opened object is not a regular file: {}",
            path_label(path)
        )));
    }
    #[cfg(unix)]
    if before.dev() != after.dev() || before.ino() != after.ino() {
        return Err(RsStoreError::Integrity(format!(
            "file changed while it was opened: {}",
            path_label(path)
        )));
    }
    Ok(file)
}

pub(crate) fn read_store_file(path: &Path, mode: u32) -> Result<Vec<u8>, RsStoreError> {
    let file = open_no_follow(path, Some(mode), true)?;
    let mut bytes = Vec::new();
    file.take(super::MAX_JSON_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() > super::MAX_JSON_BYTES {
        return Err(RsStoreError::UnsupportedFormat(format!(
            "JSON file exceeds 16 MiB: {}",
            path_label(path)
        )));
    }
    Ok(bytes)
}

/// Read one project-relative source through end of file into `buffer` and
/// return its length. The caller reuses one buffer across a save manifest, so
/// source reading holds one material-sized allocation instead of one allocation
/// per material; this keeps the save working set bounded where repeated
/// multi-megabyte allocations would otherwise accumulate allocator retention.
pub(crate) fn read_source(
    root: &Path,
    source: &str,
    buffer: &mut Vec<u8>,
) -> Result<u64, RsStoreError> {
    if !valid_uri(source) {
        return Err(RsStoreError::UnsafePath(format!(
            "invalid project-relative source path: {source}"
        )));
    }
    let mut path = root.to_path_buf();
    let segments: Vec<_> = source.split('/').collect();
    for (index, segment) in segments.iter().enumerate() {
        path.push(segment);
        let metadata = fs::symlink_metadata(&path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => {
                RsStoreError::NotFound(format!("source does not exist: {source}"))
            }
            _ => io_error(error),
        })?;
        if metadata.file_type().is_symlink() {
            return Err(RsStoreError::UnsafePath(format!(
                "source contains a symbolic link: {source}"
            )));
        }
        if index + 1 < segments.len() && !metadata.is_dir() {
            return Err(RsStoreError::UnsafePath(format!(
                "source parent is not a directory: {source}"
            )));
        }
        if index + 1 == segments.len() && !metadata.is_file() {
            return Err(RsStoreError::UnsafePath(format!(
                "source is not a regular file: {source}"
            )));
        }
    }
    let file = open_no_follow(&path, None, false)?;
    buffer.clear();
    file.take(MAX_MATERIAL_BYTES + 1)
        .read_to_end(buffer)
        .map_err(io_error)?;
    if buffer.len() as u64 > MAX_MATERIAL_BYTES {
        return Err(RsStoreError::InvalidInput(format!(
            "source exceeds 256 MiB: {source}"
        )));
    }
    Ok(buffer.len() as u64)
}

pub(crate) fn blob_path(root: &Path, digest: &str) -> Result<PathBuf, RsStoreError> {
    if !valid_digest(digest) {
        return Err(RsStoreError::Integrity(
            "invalid stored SHA-256 digest".into(),
        ));
    }
    Ok(root
        .join("blobs/sha256")
        .join(&digest[..2])
        .join(&digest[2..]))
}

pub(crate) fn read_published(
    root: &Path,
    digest: &str,
    expected_length: u64,
) -> Result<Vec<u8>, RsStoreError> {
    if expected_length > MAX_MATERIAL_BYTES {
        return Err(RsStoreError::Integrity(
            "stored material length exceeds 256 MiB".into(),
        ));
    }
    let file = open_published(root, digest)?;
    let mut bytes = Vec::with_capacity(expected_length.min(1 << 20) as usize);
    stream_published(file, digest, Some(expected_length), |chunk| {
        bytes.extend_from_slice(chunk);
        Ok(())
    })?;
    Ok(bytes)
}

/// Opens a published blob after the strict store path, ownership, type, and
/// permission checks shared by every reader. The file is opened read-only
/// without following a final symlink and checked against its pre-open
/// identity.
fn open_published(root: &Path, digest: &str) -> Result<File, RsStoreError> {
    let path = blob_path(root, digest)?;
    if let Some(parent) = path.parent() {
        store_dir(parent)?;
    }
    store_dir(root.join("blobs/sha256").as_path())?;
    open_no_follow(&path, Some(0o400), true).map_err(|error| {
        if matches!(error, RsStoreError::Io { .. }) && !path.exists() {
            RsStoreError::Integrity(format!("published blob is missing: {digest}"))
        } else {
            error
        }
    })
}

/// Streams an opened published blob through one bounded 64 KiB buffer,
/// hashing and counting it as it is read. `consume` sees each chunk exactly
/// once, so callers can discard the bytes (`verify_published`), copy them
/// (`materialize`), or append them to a bounded in-memory result
/// (`read_published`). No allocation proportional to the material size is
/// performed here.
fn stream_published(
    file: File,
    digest: &str,
    expected_length: Option<u64>,
    mut consume: impl FnMut(&[u8]) -> Result<(), RsStoreError>,
) -> Result<(), RsStoreError> {
    let mut reader = file.take(MAX_MATERIAL_BYTES + 1);
    let mut buffer = [0_u8; 64 * 1024];
    let mut length = 0_u64;
    let mut hasher = Sha256::new();
    loop {
        let count = reader.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        length += count as u64;
        hasher.update(&buffer[..count]);
        consume(&buffer[..count])?;
        if length > MAX_MATERIAL_BYTES {
            return Err(RsStoreError::Integrity(format!(
                "published blob exceeds 256 MiB: {digest}"
            )));
        }
    }
    if let Some(expected_length) = expected_length {
        if length != expected_length {
            return Err(RsStoreError::Integrity(format!(
                "published blob length differs for {digest}: expected {expected_length}, read {length}"
            )));
        }
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual != digest {
        return Err(RsStoreError::Integrity(format!(
            "published blob content differs from its SHA-256 identity: {digest}"
        )));
    }
    Ok(())
}

/// Verifies a published blob's recorded length and SHA-256 through end of
/// file without ever holding a material-sized byte vector in memory. This is
/// the bounded counterpart of `read_published` for callers that only need the
/// verification, not the bytes.
pub(crate) fn verify_published(
    root: &Path,
    digest: &str,
    expected_length: u64,
) -> Result<(), RsStoreError> {
    if expected_length > MAX_MATERIAL_BYTES {
        return Err(RsStoreError::Integrity(
            "stored material length exceeds 256 MiB".into(),
        ));
    }
    let file = open_published(root, digest)?;
    stream_published(file, digest, Some(expected_length), |_| Ok(()))
}

/// Verifies a published blob whose committed length is not yet known — for
/// example a blob published by an earlier attempt of a pending operation —
/// and returns the verified length. Like [`verify_published`] this streams
/// through one bounded buffer, so the caller learns the digest-verified
/// length without buffering the bytes.
pub(crate) fn verify_published_length(root: &Path, digest: &str) -> Result<u64, RsStoreError> {
    let file = open_published(root, digest)?;
    let mut length = 0_u64;
    stream_published(file, digest, None, |chunk| {
        length += chunk.len() as u64;
        Ok(())
    })?;
    Ok(length)
}

/// Resolves the store-controlled staging directory for one operation, or
/// `None` when it does not exist. The staging directory and the operation
/// directory must be real, effective-user-owned directories; a symbolic link
/// or non-directory there is tampering and is refused rather than followed.
/// Modes are not required here because staging is transient residue that the
/// doctor reports, and these helpers must work for manually reconstructed
/// crash states as well as for directories created by `publish_blobs`.
fn staging_operation_directory(
    root: &Path,
    operation_id: &str,
) -> Result<Option<PathBuf>, RsStoreError> {
    if !valid_id(operation_id) {
        return Err(RsStoreError::InvalidInput("invalid operation_id".into()));
    }
    let staging = root.join("staging");
    let operation = staging.join(operation_id);
    for path in [&staging, &operation] {
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(RsStoreError::UnsafePath(format!(
                        "symbolic links are not accepted in the staging path: {}",
                        path_label(path)
                    )));
                }
                if !metadata.is_dir() {
                    return Err(RsStoreError::UnsafePath(format!(
                        "staging path is not a directory: {}",
                        path_label(path)
                    )));
                }
                check_owner(&metadata, path)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error(error)),
        }
    }
    Ok(Some(operation))
}

/// Returns true when this operation's staging area holds a staged regular,
/// non-symlink file named exactly `digest` inside one attempt directory. This
/// proves that an earlier attempt of the same operation published those bytes,
/// which is the attribution rule recovery uses (S9). Absent staging is false;
/// every staging component traversed here must be effective-user-owned and
/// must not be a symlink.
pub(crate) fn published_by_attempt(
    root: &Path,
    operation_id: &str,
    digest: &str,
) -> Result<bool, RsStoreError> {
    if !valid_digest(digest) {
        return Err(RsStoreError::Integrity(
            "invalid stored SHA-256 digest".into(),
        ));
    }
    let Some(operation) = staging_operation_directory(root, operation_id)? else {
        return Ok(false);
    };
    let entries = match fs::read_dir(&operation) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(io_error(error)),
    };
    for entry in entries {
        let entry = entry.map_err(io_error)?;
        let attempt = entry.path();
        let metadata = match fs::symlink_metadata(&attempt) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_error(error)),
        };
        if metadata.file_type().is_symlink() {
            return Err(RsStoreError::UnsafePath(format!(
                "symbolic links are not accepted in the staging path: {}",
                path_label(&attempt)
            )));
        }
        if !metadata.is_dir() {
            continue;
        }
        check_owner(&metadata, &attempt)?;
        let staged = attempt.join(digest);
        match fs::symlink_metadata(&staged) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(RsStoreError::UnsafePath(format!(
                        "staged material is not a regular file: {}",
                        path_label(&staged)
                    )));
                }
                check_owner(&metadata, &staged)?;
                return Ok(true);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_error(error)),
        }
    }
    Ok(false)
}

/// Best-effort removal of one operation's transient staging residue. Only
/// digest-named regular files inside effective-user-owned attempt directories
/// are removed: symbolic links and foreign content are never followed or
/// deleted, directories are removed only when they are empty, and absent
/// staging is `Ok`. Published blobs are hard links outside `staging/`, so they
/// are unaffected.
pub(crate) fn cleanup_operation_staging(
    root: &Path,
    operation_id: &str,
) -> Result<(), RsStoreError> {
    let Some(operation) = staging_operation_directory(root, operation_id)? else {
        return Ok(());
    };
    let entries = match fs::read_dir(&operation) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error(error)),
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let attempt = entry.path();
        let metadata = match fs::symlink_metadata(&attempt) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        if check_owner(&metadata, &attempt).is_err() {
            continue;
        }
        if let Ok(children) = fs::read_dir(&attempt) {
            for child in children.flatten() {
                let staged = child.path();
                let staged_metadata = match fs::symlink_metadata(&staged) {
                    Ok(metadata) => metadata,
                    Err(_) => continue,
                };
                if staged_metadata.file_type().is_symlink() || !staged_metadata.is_file() {
                    continue;
                }
                let Some(name) = staged.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !valid_digest(name) || check_owner(&staged_metadata, &staged).is_err() {
                    continue;
                }
                let _ = fs::remove_file(&staged);
            }
        }
        let _ = fs::remove_dir(&attempt);
    }
    let _ = fsync_dir(&operation);
    if fs::remove_dir(&operation).is_ok() {
        if let Some(staging) = operation.parent() {
            let _ = fsync_dir(staging);
        }
    }
    Ok(())
}

/// Recovery evidence for the blobs one save attempt published: the staged
/// digest files live under `staging/<operation-id>/<attempt>/`, which is the
/// attribution layout [`published_by_attempt`] recovers after a crash.
pub(crate) struct Publication {
    operation_directory: PathBuf,
    attempt_directory: PathBuf,
    files: Vec<PathBuf>,
}

/// Stages and publishes the bytes of one material for `operation_id`.
///
/// Publication is incremental so a save never holds more than the material it
/// is currently publishing: the per-operation staging directory and one fresh
/// attempt directory are created lazily on the first published blob (a save
/// whose materials are all carried from committed blobs leaves no staging at
/// all), and each call stages `bytes` under
/// `staging/<operation-id>/<attempt>/<digest>`, flushes them durably, and
/// links them at their digest-derived path with atomic no-replace semantics.
/// A crash between calls therefore leaves exactly the attribution evidence
/// `published_by_attempt` uses; the caller drops each material's bytes before
/// reading the next one.
pub(crate) fn publish_blob(
    root: &Path,
    operation_id: &str,
    publication: &mut Option<Publication>,
    digest: &str,
    bytes: &[u8],
) -> Result<(), RsStoreError> {
    if sha256(bytes) != *digest || bytes.len() as u64 > MAX_MATERIAL_BYTES {
        return Err(RsStoreError::Integrity(
            "attempted to publish bytes under a different digest".into(),
        ));
    }
    if publication.is_none() {
        let staging = root.join("staging");
        create_store_dir(&staging)?;
        let operation_directory = staging.join(operation_id);
        create_store_dir(&operation_directory)?;
        let attempt_directory = operation_directory.join(uuid::Uuid::new_v4().to_string());
        create_store_dir(&attempt_directory)?;
        *publication = Some(Publication {
            operation_directory,
            attempt_directory: attempt_directory.clone(),
            files: Vec::new(),
        });
    }
    let publication = match publication {
        Some(publication) => publication,
        None => {
            return Err(RsStoreError::Integrity(
                "publication state was not initialized".into(),
            ))
        }
    };
    let stage = publication.attempt_directory.join(digest);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600).custom_flags(O_NOFOLLOW);
    let mut file = options.open(&stage).map_err(io_error)?;
    #[cfg(unix)]
    fs::set_permissions(&stage, fs::Permissions::from_mode(0o600)).map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    sync_durable(&file).map_err(io_error)?;
    drop(file);
    #[cfg(unix)]
    fs::set_permissions(&stage, fs::Permissions::from_mode(0o400)).map_err(io_error)?;
    publication.files.push(stage.clone());
    fsync_dir(&publication.attempt_directory)?;

    let final_path = blob_path(root, digest)?;
    let sha256_directory = root.join("blobs/sha256");
    create_store_dir(&sha256_directory)?;
    let prefix = final_path
        .parent()
        .ok_or_else(|| RsStoreError::Integrity("blob path has no parent".into()))?;
    create_store_dir(prefix)?;
    match fs::hard_link(&stage, &final_path) {
        Ok(()) => {}
        // An existing final file is deduplicated only after it verifies
        // through end of file; streaming keeps this check bounded too.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            verify_published(root, digest, bytes.len() as u64)?;
        }
        Err(error) => return Err(io_error(error)),
    }
    store_file(&final_path, 0o400)?;
    fsync_dir(prefix)?;
    Ok(())
}

pub(crate) fn cleanup_publication(publication: Option<Publication>) -> Result<(), RsStoreError> {
    let Some(publication) = publication else {
        return Ok(());
    };
    for path in publication.files {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_error(error)),
        }
    }
    fsync_dir(&publication.attempt_directory)?;
    match fs::remove_dir(&publication.attempt_directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(error)),
    }
    fsync_dir(&publication.operation_directory)?;
    match fs::remove_dir(&publication.operation_directory) {
        Ok(()) => {
            if let Some(staging) = publication.operation_directory.parent() {
                fsync_dir(staging)?;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => {}
    }
    Ok(())
}

pub(crate) fn fsync_dir(path: &Path) -> Result<(), RsStoreError> {
    let file = File::open(path).map_err(io_error)?;
    sync_durable(&file).map_err(io_error)
}

#[cfg(unix)]
pub(crate) fn identity(metadata: &Metadata) -> (u64, u64) {
    (metadata.dev(), metadata.ino())
}

#[cfg(not(unix))]
pub(crate) fn identity(_metadata: &Metadata) -> (u64, u64) {
    (0, 0)
}

pub(crate) fn write_new_external(path: &Path, bytes: &[u8]) -> Result<PathBuf, RsStoreError> {
    let path = canonical_for_creation(path)?;
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            return Err(RsStoreError::UnsafePath(format!(
                "output already exists: {}",
                path_label(&path)
            )))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(error)),
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600).custom_flags(O_NOFOLLOW);
    let mut file = options.open(&path).map_err(io_error)?;
    #[cfg(unix)]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    sync_durable(&file).map_err(io_error)?;
    drop(file);
    if let Some(parent) = path.parent() {
        fsync_dir(parent)?;
    }
    Ok(path)
}

fn create_external_dir(path: &Path) -> Result<(), RsStoreError> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(path).map_err(io_error)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    if let Some(parent) = path.parent() {
        fsync_dir(parent)?;
    }
    Ok(())
}

/// Materializes a revision's complete manifest into a previously absent
/// destination tree using no-replace semantics. Each published blob is
/// streamed into its destination file through a bounded 64 KiB buffer while
/// its recorded byte length and SHA-256 are verified, so checkout never holds
/// a material-sized byte vector in memory. A failed materialization removes
/// the partial destination tree instead of exposing it.
pub(crate) fn materialize(
    root: &Path,
    destination: &Path,
    materials: &[super::RsMaterial],
) -> Result<PathBuf, RsStoreError> {
    let destination = canonical_for_creation(destination)?;
    match fs::symlink_metadata(&destination) {
        Ok(_) => {
            return Err(RsStoreError::UnsafePath(format!(
                "checkout destination already exists: {}",
                path_label(&destination)
            )))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(error)),
    }
    let mut uris = BTreeSet::new();
    for material in materials {
        if !valid_uri(&material.uri) || !uris.insert(material.uri.clone()) {
            return Err(RsStoreError::Integrity(
                "manifest has an invalid or duplicate URI".into(),
            ));
        }
    }
    if let Some((existing, entry)) = super::manifest_entry_collision(&uris) {
        return Err(RsStoreError::Integrity(format!(
            "manifest names '{existing}' and '{entry}' cannot coexist on a filesystem"
        )));
    }
    create_external_dir(&destination)?;
    match materialize_blobs(root, &destination, materials) {
        Ok(()) => Ok(destination),
        Err(error) => {
            // The destination was absent before this call and was created
            // here, so removing the partial tree cannot destroy user content.
            // This removes the *whole* destination, including files already
            // materialized for earlier manifest URIs: a checkout either
            // publishes its complete verified tree or leaves no destination at
            // all, never a partial success. `remove_dir_all` does not follow
            // symbolic links.
            let _ = fs::remove_dir_all(&destination);
            Err(error)
        }
    }
}

fn materialize_blobs(
    root: &Path,
    destination: &Path,
    materials: &[super::RsMaterial],
) -> Result<(), RsStoreError> {
    for material in materials {
        if material.byte_length > MAX_MATERIAL_BYTES {
            return Err(RsStoreError::Integrity(
                "stored material length exceeds 256 MiB".into(),
            ));
        }
        let segments: Vec<_> = material.uri.split('/').collect();
        let mut parent = destination.to_path_buf();
        for segment in &segments[..segments.len() - 1] {
            parent.push(segment);
            match fs::symlink_metadata(&parent) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() || !metadata.is_dir() {
                        return Err(RsStoreError::UnsafePath(format!(
                            "checkout parent is not a private directory: {}",
                            material.uri
                        )));
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    create_external_dir(&parent)?
                }
                Err(error) => return Err(io_error(error)),
            }
        }
        let path = parent.join(segments[segments.len() - 1]);
        let published = open_published(root, &material.sha256)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600).custom_flags(O_NOFOLLOW);
        let mut file = options.open(&path).map_err(io_error)?;
        #[cfg(unix)]
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(io_error)?;
        stream_published(
            published,
            &material.sha256,
            Some(material.byte_length),
            |chunk| file.write_all(chunk).map_err(io_error),
        )?;
        sync_durable(&file).map_err(io_error)?;
        drop(file);
        fsync_dir(&parent)?;
    }
    fsync_dir(destination)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::RsMaterial;
    use super::*;

    struct Fixture {
        base: PathBuf,
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let base = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("cache/implementation/storage-blob-primitive-tests")
                .join(uuid::Uuid::new_v4().to_string());
            fs::create_dir_all(&base).unwrap();
            let fixture = Self {
                root: base.join("store"),
                base,
            };
            create_store_dir(&fixture.root).unwrap();
            fixture
        }

        /// Publishes one blob exactly where `publish_blobs` links it, with the
        /// strict `0400` published mode.
        fn publish(&self, bytes: &[u8]) -> String {
            let digest = sha256(bytes);
            let path = blob_path(&self.root, &digest).unwrap();
            create_store_dir(&self.root.join("blobs")).unwrap();
            create_store_dir(&self.root.join("blobs/sha256")).unwrap();
            create_store_dir(path.parent().unwrap()).unwrap();
            fs::write(&path, bytes).unwrap();
            set_mode(&path, 0o400);
            digest
        }

        fn material(&self, uri: &str, digest: &str, byte_length: u64) -> RsMaterial {
            RsMaterial {
                uri: uri.into(),
                role: "artifact".into(),
                sha256: digest.into(),
                byte_length,
            }
        }

        fn staging_attempt(&self, operation_id: &str, attempt: &str) -> PathBuf {
            let path = self.root.join("staging").join(operation_id).join(attempt);
            fs::create_dir_all(&path).unwrap();
            path
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }

    #[cfg(unix)]
    fn set_mode(path: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    #[cfg(not(unix))]
    fn set_mode(_path: &Path, _mode: u32) {}

    #[test]
    fn verify_published_streams_and_rejects_tampering() {
        let fixture = Fixture::new();
        let bytes = b"bounded verification bytes";
        let digest = fixture.publish(bytes);
        let length = bytes.len() as u64;
        verify_published(&fixture.root, &digest, length).unwrap();

        assert!(matches!(
            verify_published(&fixture.root, &digest, length + 1),
            Err(RsStoreError::Integrity(message)) if message.contains("length differs")
        ));

        // Same length, different content: the through-EOF digest check catches it.
        let blob = blob_path(&fixture.root, &digest).unwrap();
        set_mode(&blob, 0o600);
        fs::write(&blob, vec![b'x'; bytes.len()]).unwrap();
        set_mode(&blob, 0o400);
        assert!(matches!(
            verify_published(&fixture.root, &digest, length),
            Err(RsStoreError::Integrity(message))
                if message.contains("differs from its SHA-256 identity")
        ));

        fs::remove_file(&blob).unwrap();
        assert!(matches!(
            verify_published(&fixture.root, &digest, length),
            Err(RsStoreError::Integrity(message)) if message.contains("missing")
        ));
        assert!(matches!(
            verify_published(&fixture.root, "not-a-digest", 0),
            Err(RsStoreError::Integrity(_))
        ));
    }

    #[test]
    fn materialize_streams_manifest_without_partial_destination() {
        let fixture = Fixture::new();
        // Larger than one 64 KiB buffer so the copy path must stream.
        let payload: Vec<u8> = (0..150_000usize).map(|index| (index % 251) as u8).collect();
        let digest = fixture.publish(&payload);
        let length = payload.len() as u64;
        let materials = vec![
            fixture.material("tree/a.bin", &digest, length),
            fixture.material("tree/b.bin", &digest, length),
            fixture.material("deep/dir/c.bin", &digest, length),
        ];

        let destination = fixture.base.join("checkout");
        let materialized = materialize(&fixture.root, &destination, &materials).unwrap();
        assert_eq!(materialized, fs::canonicalize(&destination).unwrap());
        for uri in ["tree/a.bin", "tree/b.bin", "deep/dir/c.bin"] {
            assert_eq!(fs::read(destination.join(uri)).unwrap(), payload);
        }
        assert!(matches!(
            materialize(&fixture.root, &destination, &materials),
            Err(RsStoreError::UnsafePath(_))
        ));

        // Invalid, duplicate, and length-mismatched entries write nothing.
        let duplicate = fixture.base.join("duplicate");
        let aliased = vec![
            fixture.material("same.txt", &digest, length),
            fixture.material("same.txt", &digest, length),
        ];
        assert!(matches!(
            materialize(&fixture.root, &duplicate, &aliased),
            Err(RsStoreError::Integrity(_))
        ));
        assert!(!duplicate.exists());
        let invalid = fixture.base.join("invalid");
        assert!(matches!(
            materialize(
                &fixture.root,
                &invalid,
                &[fixture.material("../escape.txt", &digest, length)],
            ),
            Err(RsStoreError::Integrity(_))
        ));
        assert!(!invalid.exists());
        let wrong_length = fixture.base.join("wrong-length");
        assert!(matches!(
            materialize(
                &fixture.root,
                &wrong_length,
                &[fixture.material("wrong.bin", &digest, length + 1)],
            ),
            Err(RsStoreError::Integrity(message)) if message.contains("length differs")
        ));
        assert!(!wrong_length.exists());

        // Altered then missing published bytes are refused with no residue.
        let altered_destination = fixture.base.join("altered-checkout");
        let blob = blob_path(&fixture.root, &digest).unwrap();
        set_mode(&blob, 0o600);
        fs::write(&blob, vec![0_u8; payload.len()]).unwrap();
        set_mode(&blob, 0o400);
        assert!(matches!(
            materialize(&fixture.root, &altered_destination, &materials),
            Err(RsStoreError::Integrity(message)) if message.contains("differs")
        ));
        assert!(!altered_destination.exists());

        fs::remove_file(&blob).unwrap();
        let missing_destination = fixture.base.join("missing-checkout");
        assert!(matches!(
            materialize(&fixture.root, &missing_destination, &materials),
            Err(RsStoreError::Integrity(message)) if message.contains("missing")
        ));
        assert!(!missing_destination.exists());
    }

    #[test]
    fn published_by_attempt_attributes_only_this_operations_residue() {
        let fixture = Fixture::new();
        let bytes = b"staged publication bytes";
        let digest = fixture.publish(bytes);
        let other = sha256(b"some other bytes");

        assert!(!published_by_attempt(&fixture.root, "operation-a", &digest).unwrap());

        let attempt = fixture.staging_attempt("operation-a", "attempt-1");
        fs::write(attempt.join(&digest), bytes).unwrap();
        assert!(published_by_attempt(&fixture.root, "operation-a", &digest).unwrap());
        assert!(!published_by_attempt(&fixture.root, "operation-a", &other).unwrap());
        assert!(!published_by_attempt(&fixture.root, "operation-b", &digest).unwrap());
        assert!(matches!(
            published_by_attempt(&fixture.root, "operation-a", "not-a-digest"),
            Err(RsStoreError::Integrity(_))
        ));

        // A file directly under staging/<operation-id>/ is not an attempt
        // subdirectory entry and does not attribute the bytes.
        let shallow = fixture.root.join("staging/operation-c");
        fs::create_dir_all(&shallow).unwrap();
        fs::write(shallow.join(&digest), bytes).unwrap();
        assert!(!published_by_attempt(&fixture.root, "operation-c", &digest).unwrap());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let elsewhere = fixture.base.join("elsewhere");
            fs::create_dir_all(&elsewhere).unwrap();
            fs::write(elsewhere.join(&digest), bytes).unwrap();
            let operation = fixture.root.join("staging/operation-d");
            fs::create_dir_all(&operation).unwrap();
            symlink(&elsewhere, operation.join("attempt-link")).unwrap();
            assert!(matches!(
                published_by_attempt(&fixture.root, "operation-d", &digest),
                Err(RsStoreError::UnsafePath(_))
            ));
        }
    }

    #[test]
    fn cleanup_operation_staging_removes_residue_but_keeps_published_blobs() {
        let fixture = Fixture::new();
        let bytes = b"published bytes kept by cleanup";
        let digest = fixture.publish(bytes);
        let length = bytes.len() as u64;
        let attempt = fixture.staging_attempt("cleanup-operation", "attempt-1");
        // `publish_blobs` keeps a staging hard link to the published blob;
        // reproducing that layout proves cleanup cannot damage the blob.
        fs::hard_link(
            blob_path(&fixture.root, &digest).unwrap(),
            attempt.join(&digest),
        )
        .unwrap();
        let operation = attempt.parent().unwrap().to_path_buf();
        fs::write(operation.join("foreign-direct-file"), b"foreign").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(fixture.base.join("link-target"), attempt.join("link")).unwrap();
        }
        assert!(published_by_attempt(&fixture.root, "cleanup-operation", &digest).unwrap());

        cleanup_operation_staging(&fixture.root, "cleanup-operation").unwrap();
        assert!(!attempt.join(&digest).exists());
        // Foreign entries and symlinks are never followed or removed.
        assert!(operation.join("foreign-direct-file").exists());
        #[cfg(unix)]
        assert!(attempt.join("link").symlink_metadata().is_ok());
        // The published blob survives the hard-link removal and still verifies.
        verify_published(&fixture.root, &digest, length).unwrap();

        // Residue-only staging is removed completely, and absent staging is Ok.
        let clean_attempt = fixture.staging_attempt("cleanup-clean", "attempt-2");
        fs::write(clean_attempt.join(&other_digest(bytes)), bytes).unwrap();
        let clean_operation = clean_attempt.parent().unwrap().to_path_buf();
        cleanup_operation_staging(&fixture.root, "cleanup-clean").unwrap();
        assert!(!clean_operation.exists());
        cleanup_operation_staging(&fixture.root, "no-such-operation").unwrap();
    }

    fn other_digest(bytes: &[u8]) -> String {
        // A digest-named staged file does not have to be published for
        // cleanup to recognize it as this store's staging residue.
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.update(b"staging residue");
        format!("{:x}", hasher.finalize())
    }
}
