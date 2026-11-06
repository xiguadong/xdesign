//! Shared, read-only backend classification. No default-home enrollment occurs here.
use serde::Deserialize;
use serde_json::Value;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Registry {
    pub schema_version: u32,
    pub wish_id: String,
    pub flow_id: String,
    pub run_id: String,
    pub wish_model_uri: String,
    pub plan_uri: String,
    pub plan_sha256: String,
    pub work_uri: String,
    pub work_sha256: String,
    pub run_record_uri: String,
}
#[derive(Clone)]
pub(crate) enum Cache {
    Absent,
    Legacy(Registry),
    Persistent(Registry),
}
fn ambiguous(s: impl std::fmt::Display) -> String {
    format!("location_reason=backend_ambiguous; {s}")
}
fn read(project: &Path, path: &Path) -> Result<Vec<u8>, String> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let project = fs::canonicalize(project).map_err(ambiguous)?;
    let canonical = fs::canonicalize(path).map_err(ambiguous)?;
    if !canonical.starts_with(&project) {
        return Err(ambiguous("backend file escapes the selected project"));
    }
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(ambiguous)?;
    let metadata = file.metadata().map_err(ambiguous)?;
    if !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o022 != 0
        || metadata.len() > 16 * 1024 * 1024
    {
        return Err(ambiguous("unsafe or oversized backend file"));
    }
    let mut bytes = vec![];
    file.take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(ambiguous)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(ambiguous("backend file exceeds size limit"));
    }
    Ok(bytes)
}
pub(crate) fn classify(project: &Path, id: &str) -> Result<Cache, String> {
    let directory = project.join("cache/xdesign").join(id);
    let path = directory.join("registry.json");
    if fs::symlink_metadata(&path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
        if fs::symlink_metadata(&directory).is_ok() {
            return Err(ambiguous("unmatched cache directory"));
        }
        return Ok(Cache::Absent);
    }
    let raw = read(project, &path)?;
    let r: Registry = serde_json::from_slice(&raw).map_err(ambiguous)?;
    let prefix = format!("cache/xdesign/{id}");
    if r.schema_version != 1
        || r.wish_id != id
        || r.run_id.is_empty()
        || !["simple-flow", "wish-verify"].contains(&r.flow_id.as_str())
        || r.plan_uri != format!("{prefix}/plan.json")
        || r.work_uri != format!("{prefix}/work.json")
        || r.run_record_uri != format!("{prefix}/run.json")
    {
        return Err(ambiguous("registry identity and linkage differ"));
    }
    if r.wish_model_uri == format!("mbse/wishes/{id}.sysml") {
        for (uri, digest) in [
            (&r.plan_uri, Some(&r.plan_sha256)),
            (&r.work_uri, Some(&r.work_sha256)),
            (&r.run_record_uri, None),
            (&r.wish_model_uri, None),
        ] {
            let bytes = read(project, &project.join(uri))?;
            if let Some(digest) = digest {
                use sha2::{Digest, Sha256};
                if format!("{:x}", Sha256::digest(&bytes)) != *digest {
                    return Err(ambiguous("frozen legacy bytes changed"));
                }
            }
        }
        let run: Value = serde_json::from_slice(&read(project, &project.join(&r.run_record_uri))?)
            .map_err(ambiguous)?;
        if run["run_id"] != r.run_id {
            return Err(ambiguous("legacy run identity differs"));
        }
        Ok(Cache::Legacy(r))
    } else if r.wish_model_uri == format!("cache/wishes/{id}.sysml") {
        Ok(Cache::Persistent(r))
    } else {
        Err(ambiguous("unrecognized registry backend"))
    }
}
/// Candidates are hints, never authority. RsStore validates each route and root.
pub(crate) fn candidates(project: &Path, explicit: Option<&Path>) -> Result<Vec<PathBuf>, String> {
    let mut roots = vec![];
    if let Some(root) = explicit {
        roots.push(if root.is_absolute() {
            root.to_owned()
        } else {
            project.join(root)
        });
    }
    let local = project.join(".xdesign");
    if fs::symlink_metadata(&local).is_ok() {
        roots.push(local);
    }
    let routes = project.join(".xdesign-access-v1/routes");
    if routes.exists() {
        for bucket in fs::read_dir(&routes).map_err(ambiguous)? {
            for entry in fs::read_dir(bucket.map_err(ambiguous)?.path()).map_err(ambiguous)? {
                let v: Value =
                    serde_json::from_slice(&read(project, &entry.map_err(ambiguous)?.path())?)
                        .map_err(ambiguous)?;
                if v["payload"]["source_root"] != serde_json::json!(project.join(".xdesign")) {
                    return Err(ambiguous("route project mismatch"));
                }
                roots.push(PathBuf::from(
                    v["payload"]["destination_root"]
                        .as_str()
                        .ok_or_else(|| ambiguous("route lacks destination"))?,
                ));
            }
        }
    }
    roots.sort();
    roots.dedup();
    Ok(roots)
}

/// Pure legacy commands hold the same physical directory lock domain. They do
/// not require a store, namespace, account lookup, or a persistent cache.
pub(crate) struct LegacyLease(Vec<fs::File>);
impl LegacyLease {
    pub(crate) fn acquire(project: &Path) -> Result<Self, String> {
        use std::os::unix::{
            fs::{MetadataExt, OpenOptionsExt},
            io::AsRawFd,
        };
        let project = fs::canonicalize(project).map_err(ambiguous)?;
        let f = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_DIRECTORY)
            .open(&project)
            .map_err(ambiguous)?;
        let m = f.metadata().map_err(ambiguous)?;
        if m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o022 != 0 {
            return Err(ambiguous("unsafe project lease directory"));
        }
        if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err("location_reason=migration_busy; project backend lease is occupied".into());
        }
        Ok(Self(vec![f]))
    }
}
impl Drop for LegacyLease {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        for f in self.0.iter().rev() {
            unsafe {
                libc::flock(f.as_raw_fd(), libc::LOCK_UN);
            }
        }
    }
}
