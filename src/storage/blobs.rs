use super::{io_error, sha256, valid_digest, valid_uri, RsStoreError, MAX_MATERIAL_BYTES};
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
    // Every ancestor of the final component must be a real directory: a
    // symlink anywhere in the chain would let the write land outside the
    // validated destination, so the whole chain is checked, not just the
    // immediate parent.
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
                break;
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
                match ancestor.parent() {
                    Some(higher) => ancestor = higher.to_path_buf(),
                    None => {
                        return Err(RsStoreError::UnsafePath(format!(
                            "destination root is not a directory: {}",
                            path_label(&ancestor)
                        )))
                    }
                }
            }
            Err(error) => return Err(io_error(error)),
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

pub(crate) fn read_source(root: &Path, source: &str) -> Result<Vec<u8>, RsStoreError> {
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
    let mut bytes = Vec::new();
    file.take(MAX_MATERIAL_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > MAX_MATERIAL_BYTES {
        return Err(RsStoreError::InvalidInput(format!(
            "source exceeds 256 MiB: {source}"
        )));
    }
    Ok(bytes)
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
    read_published_checked(root, digest, Some(expected_length))
}

/// Reads and digest-verifies a published blob whose committed length is not
/// yet known, for example a blob published by a pending attempt.
pub(crate) fn read_published_any(root: &Path, digest: &str) -> Result<Vec<u8>, RsStoreError> {
    read_published_checked(root, digest, None)
}

fn read_published_checked(
    root: &Path,
    digest: &str,
    expected_length: Option<u64>,
) -> Result<Vec<u8>, RsStoreError> {
    if expected_length.is_some_and(|length| length > MAX_MATERIAL_BYTES) {
        return Err(RsStoreError::Integrity(
            "stored material length exceeds 256 MiB".into(),
        ));
    }
    let path = blob_path(root, digest)?;
    if let Some(parent) = path.parent() {
        store_dir(parent)?;
    }
    store_dir(root.join("blobs/sha256").as_path())?;
    let file = open_no_follow(&path, Some(0o400), true).map_err(|error| {
        if matches!(error, RsStoreError::Io { .. }) && !path.exists() {
            RsStoreError::Integrity(format!("published blob is missing: {digest}"))
        } else {
            error
        }
    })?;
    let mut reader = file.take(MAX_MATERIAL_BYTES + 1);
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes = Vec::with_capacity(expected_length.unwrap_or(0).min(1 << 20) as usize);
    let mut hasher = Sha256::new();
    loop {
        let count = reader.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() as u64 > MAX_MATERIAL_BYTES {
            return Err(RsStoreError::Integrity(format!(
                "published blob exceeds 256 MiB: {digest}"
            )));
        }
    }
    if let Some(expected_length) = expected_length {
        if bytes.len() as u64 != expected_length {
            return Err(RsStoreError::Integrity(format!(
                "published blob length differs for {digest}: expected {expected_length}, read {}",
                bytes.len()
            )));
        }
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual != digest {
        return Err(RsStoreError::Integrity(format!(
            "published blob content differs from its SHA-256 identity: {digest}"
        )));
    }
    Ok(bytes)
}

pub(crate) fn published_exists(root: &Path, digest: &str) -> Result<bool, RsStoreError> {
    let path = blob_path(root, digest)?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(RsStoreError::UnsafePath(format!(
                    "published blob is not a regular file: {digest}"
                )));
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io_error(error)),
    }
}

pub(crate) struct Publication {
    operation_directory: PathBuf,
    attempt_directory: PathBuf,
    files: Vec<PathBuf>,
}

pub(crate) fn publish_blobs(
    root: &Path,
    operation_id: &str,
    blobs: &[(String, Vec<u8>)],
) -> Result<Option<Publication>, RsStoreError> {
    if blobs.is_empty() {
        return Ok(None);
    }
    let staging = root.join("staging");
    create_store_dir(&staging)?;
    let operation_directory = staging.join(operation_id);
    create_store_dir(&operation_directory)?;
    let attempt_directory = operation_directory.join(uuid::Uuid::new_v4().to_string());
    create_store_dir(&attempt_directory)?;
    let mut publication = Publication {
        operation_directory,
        attempt_directory: attempt_directory.clone(),
        files: Vec::new(),
    };
    for (digest, bytes) in blobs {
        if sha256(bytes) != *digest || bytes.len() as u64 > MAX_MATERIAL_BYTES {
            return Err(RsStoreError::Integrity(
                "attempted to publish bytes under a different digest".into(),
            ));
        }
        let stage = attempt_directory.join(digest);
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
        fsync_dir(&attempt_directory)?;

        let final_path = blob_path(root, digest)?;
        let sha256_directory = root.join("blobs/sha256");
        create_store_dir(&sha256_directory)?;
        let prefix = final_path
            .parent()
            .ok_or_else(|| RsStoreError::Integrity("blob path has no parent".into()))?;
        create_store_dir(prefix)?;
        match fs::hard_link(&stage, &final_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                read_published(root, digest, bytes.len() as u64)?;
            }
            Err(error) => return Err(io_error(error)),
        }
        store_file(&final_path, 0o400)?;
        fsync_dir(prefix)?;
    }
    Ok(Some(publication))
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

pub(crate) fn materialize(
    destination: &Path,
    materials: &[(String, Vec<u8>)],
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
    let uris: BTreeSet<String> = materials
        .iter()
        .map(|(uri, _)| uri.as_str())
        .filter(|uri| valid_uri(uri))
        .map(str::to_owned)
        .collect();
    if uris.len() != materials.len() {
        return Err(RsStoreError::Integrity(
            "manifest has an invalid or duplicate URI".into(),
        ));
    }
    if let Some((existing, entry)) = super::manifest_entry_collision(&uris) {
        return Err(RsStoreError::Integrity(format!(
            "manifest names '{existing}' and '{entry}' cannot coexist on a filesystem"
        )));
    }
    create_external_dir(&destination)?;
    for (uri, bytes) in materials {
        let segments: Vec<_> = uri.split('/').collect();
        let mut parent = destination.clone();
        for segment in &segments[..segments.len() - 1] {
            parent.push(segment);
            match fs::symlink_metadata(&parent) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() || !metadata.is_dir() {
                        return Err(RsStoreError::UnsafePath(format!(
                            "checkout parent is not a private directory: {uri}"
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
        fsync_dir(&parent)?;
    }
    fsync_dir(&destination)?;
    Ok(destination)
}
