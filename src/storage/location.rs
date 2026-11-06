//! Contextual store selection and the versioned identity grammar from I21.
use super::{
    access::{self, diagnostic},
    blobs, io_error, sha256, strict_json, RsStoreError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[cfg(unix)]
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt as MacMetadataExt;
#[cfg(unix)]
use std::os::unix::{
    ffi::OsStrExt,
    fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    io::AsRawFd,
};
use std::{
    ffi::{CStr, CString},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::{observe, rename_exclusive};

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub(crate) fn unhex(s: &str) -> Result<Vec<u8>, RsStoreError> {
    if s.len() % 2 != 0
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(diagnostic("coordination_corrupt", "noncanonical hex"));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| diagnostic("coordination_corrupt", e))
        })
        .collect()
}
pub(crate) fn random() -> Result<String, RsStoreError> {
    let mut b = [0; 32];
    File::open("/dev/urandom")?.read_exact(&mut b)?;
    if b == [0; 32] {
        return Err(diagnostic("identity_unprovable", "zero random value"));
    }
    Ok(hex(&b))
}
pub(crate) fn domain(label: &str, bytes: &[u8]) -> Vec<u8> {
    [
        b"xdesign.worktree.v1/".as_slice(),
        label.as_bytes(),
        &[0],
        bytes,
    ]
    .concat()
}
pub(crate) fn hmac(key: &[u8], bytes: &[u8]) -> String {
    let mut inner = [0x36; 64];
    let mut outer = [0x5c; 64];
    let long;
    let key = if key.len() > 64 {
        long = Sha256::digest(key).to_vec();
        &long
    } else {
        key
    };
    for (i, b) in key.iter().enumerate() {
        inner[i] ^= b;
        outer[i] ^= b;
    }
    let a = Sha256::digest([inner.as_slice(), bytes].concat());
    hex(&Sha256::digest([outer.as_slice(), a.as_slice()].concat()))
}
fn corrupt(s: impl std::fmt::Display) -> RsStoreError {
    diagnostic("coordination_corrupt", s)
}
fn fields(kind: u8) -> Vec<(&'static str, u8)> {
    match kind {
        1 => vec![
            ("path", 3),
            ("device", 1),
            ("inode", 1),
            ("volume_id", 4),
            ("birth_seconds", 2),
            ("birth_nanoseconds", 1),
            ("nonce", 4),
        ],
        2 | 3 => {
            let mut v = vec![
                ("schema_version", 1),
                ("namespace_epoch", 4),
                ("owner_uid", 1),
                ("home", 5),
                ("common", 5),
                ("admin", 5),
                ("worktree", 5),
                ("project", 5),
                ("project_relative", 3),
            ];
            if kind == 3 {
                v.extend([("key", 4), ("store_root", 3)]);
            }
            v
        }
        4 => vec![
            ("schema_version", 1),
            ("namespace_epoch", 4),
            ("owner_uid", 1),
            ("object", 5),
            ("roles", 6),
        ],
        5 => vec![
            ("schema_version", 1),
            ("namespace_epoch", 4),
            ("owner_uid", 1),
            ("path", 3),
        ],
        6 => vec![
            ("schema_version", 1),
            ("owner_uid", 1),
            ("parent", 8),
            ("child", 8),
        ],
        7 => vec![
            ("schema_version", 1),
            ("namespace_epoch", 4),
            ("owner_uid", 1),
            ("binding_key", 4),
            ("binding_sha256", 4),
            ("source_root", 3),
            ("destination_root", 3),
            ("source_parent", 5),
            ("destination_parent", 5),
            ("journal_base", 3),
            ("namespace_path", 3),
        ],
        8 => vec![
            ("schema_version", 1),
            ("namespace_epoch", 4),
            ("owner_uid", 1),
            ("migration_id", 3),
            ("attempt_nonce", 4),
            ("binding_key", 4),
            ("binding_sha256", 4),
            ("request_bytes", 4),
            ("request_sha256", 4),
            ("route_sha256", 4),
            ("journal_identity", 5),
            ("source_root_identity", 5),
            ("source_database_identity", 5),
        ],
        9 => vec![
            ("schema_version", 1),
            ("namespace_epoch", 4),
            ("owner_uid", 1),
            ("object", 5),
        ],
        _ => vec![],
    }
}
pub(crate) fn encode(kind: u8, value: &Value) -> Result<Vec<u8>, RsStoreError> {
    let fields = fields(kind);
    let obj = value
        .as_object()
        .ok_or_else(|| corrupt("expected object"))?;
    if fields.is_empty() || obj.len() != fields.len() {
        return Err(corrupt("unknown or missing canonical field"));
    }
    let mut out = vec![b'X', b'D', b'W', b'S', 1, kind];
    out.extend((fields.len() as u16).to_be_bytes());
    for (i, (name, mut ty)) in fields.into_iter().enumerate() {
        let v = obj
            .get(name)
            .ok_or_else(|| corrupt(format!("missing {name}")))?;
        let bytes = if name == "nonce" && v.is_null() {
            ty = 0;
            vec![]
        } else {
            match ty {
                1 | 2 => {
                    let text = if name == "schema_version" {
                        if v != &json!(1) {
                            return Err(corrupt("schema version"));
                        }
                        "1".into()
                    } else {
                        v.as_str()
                            .ok_or_else(|| corrupt("integer must be decimal string"))?
                            .to_owned()
                    };
                    if ty == 1 {
                        let n = text.parse::<u64>().map_err(corrupt)?;
                        if n.to_string() != text {
                            return Err(corrupt("noncanonical integer"));
                        }
                        n.to_be_bytes().to_vec()
                    } else {
                        let n = text.parse::<i64>().map_err(corrupt)?;
                        if n.to_string() != text {
                            return Err(corrupt("noncanonical integer"));
                        }
                        n.to_be_bytes().to_vec()
                    }
                }
                3 => v
                    .as_str()
                    .ok_or_else(|| corrupt("expected UTF-8 text"))?
                    .as_bytes()
                    .to_vec(),
                4 => {
                    let b = unhex(v.as_str().ok_or_else(|| corrupt("expected hex"))?)?;
                    let length = if name == "volume_id" { 17 } else { 32 };
                    if name != "request_bytes" && b.len() != length {
                        return Err(corrupt("hex length"));
                    }
                    b
                }
                5 => encode(1, v)?,
                6 => {
                    let a = v.as_array().ok_or_else(|| corrupt("expected list"))?;
                    let mut b = (a.len() as u32).to_be_bytes().to_vec();
                    let mut last = "";
                    for s in a {
                        let s = s.as_str().ok_or_else(|| corrupt("role text"))?;
                        if s <= last
                            || !["admin", "common", "home", "project", "worktree"].contains(&s)
                        {
                            return Err(corrupt("invalid role order"));
                        }
                        last = s;
                        b.push(3);
                        b.extend((s.len() as u32).to_be_bytes());
                        b.extend(s.as_bytes());
                    }
                    if a.is_empty() {
                        return Err(corrupt("empty roles"));
                    }
                    b
                }
                8 => unhex(v.as_str().ok_or_else(|| corrupt("native path hex"))?)?,
                _ => return Err(corrupt("unknown type")),
            }
        };
        out.extend(((i + 1) as u16).to_be_bytes());
        out.push(ty);
        out.extend(u32::try_from(bytes.len()).map_err(corrupt)?.to_be_bytes());
        out.extend(bytes);
    }
    if out.len() > super::MAX_JSON_BYTES {
        return Err(corrupt("encoding exceeds limit"));
    }
    Ok(out)
}
pub(crate) fn sign(
    key: &[u8],
    label: &str,
    kind: u8,
    payload: Value,
) -> Result<Value, RsStoreError> {
    let mac = hmac(key, &domain(label, &encode(kind, &payload)?));
    Ok(json!({"payload":payload,"hmac_sha256":mac}))
}
pub(crate) fn authenticate(
    key: &[u8],
    label: &str,
    kind: u8,
    record: &Value,
) -> Result<Value, RsStoreError> {
    let m = record
        .as_object()
        .ok_or_else(|| corrupt("authenticated envelope"))?;
    if m.len() != 2 || !m.contains_key("payload") || !m.contains_key("hmac_sha256") {
        return Err(corrupt("authenticated envelope fields"));
    }
    if sign(key, label, kind, m["payload"].clone())? != *record {
        return Err(corrupt("authentication mismatch"));
    }
    Ok(m["payload"].clone())
}
pub(crate) fn json_file(path: &Path) -> Result<Value, RsStoreError> {
    if fs::symlink_metadata(path)?.nlink() != 1 {
        return Err(diagnostic("unsafe_path", "hard-linked coordination record"));
    }
    let raw = blobs::read_store_file(path, 0o600)?;
    strict_json(&raw).map_err(corrupt)
}
pub(crate) fn safe_dir(path: &Path) -> Result<(), RsStoreError> {
    let m = fs::symlink_metadata(path)?;
    if !m.is_dir()
        || m.file_type().is_symlink()
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o022 != 0
    {
        return Err(diagnostic("unsafe_path", path.display()));
    }
    blobs::check_acl(path)
}
pub(crate) fn mkdir(path: &Path) -> Result<(), RsStoreError> {
    if path.exists() {
        return safe_dir(path);
    }
    let parent = path
        .parent()
        .ok_or_else(|| diagnostic("unsafe_path", "missing parent"))?;
    safe_dir(parent)?;
    fs::create_dir(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    blobs::fsync_dir(path)?;
    blobs::fsync_dir(parent)
}
/// Complete draft, then exclusive publication; never open the final for writing.
pub(crate) fn publish(path: &Path, bytes: &[u8]) -> Result<(), RsStoreError> {
    let parent = path.parent().ok_or_else(|| corrupt("missing parent"))?;
    safe_dir(parent)?;
    if path.exists() {
        if blobs::read_store_file(path, 0o600)? == bytes {
            return blobs::fsync_dir(parent);
        }
        return Err(corrupt(format!(
            "immutable record conflict: {}",
            path.display()
        )));
    }
    let pool = publication_pool(parent);
    safe_dir(&pool)?;
    let draft = pool.join(format!(".draft-{}", random()?));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&draft)?;
    f.write_all(bytes)?;
    let route = parent
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|n| n == "routes");
    if route {
        super::migration::fault("route-draft-written");
    }
    blobs::sync_durable(&f)?;
    drop(f);
    if route {
        super::migration::fault("route-draft-synced");
    }
    rename_exclusive(&draft, path)?;
    if route {
        super::migration::fault("route-published");
    }
    if path.file_name().is_some_and(|n| n == "decision.json") {
        super::migration::fault("decision-published");
    }
    blobs::fsync_dir(&pool)?;
    blobs::fsync_dir(parent)
}
pub(crate) fn publish_json(path: &Path, value: &Value) -> Result<(), RsStoreError> {
    publish(path, &serde_json::to_vec(value).map_err(corrupt)?)
}
#[cfg(target_os = "macos")]
pub(crate) fn rename_exclusive(from: &Path, to: &Path) -> Result<(), RsStoreError> {
    let a = CString::new(from.as_os_str().as_bytes()).map_err(corrupt)?;
    let b = CString::new(to.as_os_str().as_bytes()).map_err(corrupt)?;
    if unsafe { libc::renamex_np(a.as_ptr(), b.as_ptr(), libc::RENAME_EXCL) } != 0 {
        return Err(io_error(std::io::Error::last_os_error()));
    }
    Ok(())
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) fn rename_exclusive(_: &Path, _: &Path) -> Result<(), RsStoreError> {
    Err(diagnostic(
        "coordination_unavailable",
        "this build has no validated atomic publication profile for this platform",
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Object {
    pub path: String,
    pub device: String,
    pub inode: String,
    pub volume_id: String,
    pub birth_seconds: String,
    pub birth_nanoseconds: String,
    pub nonce: Option<String>,
}
impl Object {
    pub(crate) fn value(&self) -> Value {
        serde_json::to_value(self).expect("object serialization")
    }
}
#[cfg(target_os = "macos")]
pub(crate) fn observe(path: &Path, nonce: Option<String>) -> Result<Object, RsStoreError> {
    let path = fs::canonicalize(path)?;
    let text = path
        .to_str()
        .ok_or_else(|| diagnostic("identity_unprovable", "non-UTF-8 identity"))?
        .to_owned();
    let m = fs::symlink_metadata(&path)?;
    // Do not open/close a second descriptor to a live SQLite database: doing
    // that drops process-scoped POSIX locks. The metadata syscall itself does
    // not need a descriptor, and filesystem observations use its directory.
    let descriptor_path = if m.is_dir() {
        path.as_path()
    } else {
        path.parent()
            .ok_or_else(|| diagnostic("unsafe_path", "object has no parent"))?
    };
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(descriptor_path)?;
    blobs::check_acl(&path)?;
    if m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o022 != 0 {
        return Err(diagnostic("unsafe_path", path.display()));
    }
    let mut attrs: libc::attrlist = unsafe { std::mem::zeroed() };
    attrs.bitmapcount = 5;
    attrs.commonattr =
        libc::ATTR_CMN_RETURNED_ATTRS | libc::ATTR_CMN_CRTIME | libc::ATTR_CMN_FILEID;
    let mut b = [0u8; 128];
    let native = CString::new(path.as_os_str().as_bytes()).map_err(corrupt)?;
    if unsafe {
        libc::getattrlist(
            native.as_ptr(),
            (&mut attrs as *mut libc::attrlist).cast(),
            b.as_mut_ptr().cast(),
            b.len(),
            libc::FSOPT_NOFOLLOW as _,
        )
    } != 0
    {
        return Err(diagnostic(
            "identity_unprovable",
            std::io::Error::last_os_error(),
        ));
    }
    let word = |n| u32::from_ne_bytes(b[n..n + 4].try_into().unwrap());
    if word(0) != 48 || word(4) & attrs.commonattr != attrs.commonattr {
        return Err(diagnostic(
            "identity_unprovable",
            "missing birth/file attributes",
        ));
    }
    let birth = i64::from_ne_bytes(b[24..32].try_into().unwrap());
    let nano = i64::from_ne_bytes(b[32..40].try_into().unwrap());
    let ino = u64::from_ne_bytes(b[40..48].try_into().unwrap());
    if birth != m.st_birthtime()
        || nano != m.st_birthtime_nsec()
        || ino != m.ino()
        || birth == 0
        || !(0..1_000_000_000).contains(&nano)
    {
        return Err(diagnostic(
            "identity_unprovable",
            "inconsistent birth/file attributes",
        ));
    }
    let mut st: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstatfs(f.as_raw_fd(), &mut st) } != 0 {
        return Err(diagnostic(
            "identity_unprovable",
            "volume observation failed",
        ));
    }
    let fs_type = unsafe { CStr::from_ptr(st.f_fstypename.as_ptr()) }.to_bytes();
    if fs_type != b"apfs" || st.f_flags & libc::MNT_LOCAL as u32 == 0 {
        return Err(diagnostic(
            "identity_unprovable",
            "only local APFS has a validated host profile",
        ));
    }
    let mount = PathBuf::from(
        unsafe { CStr::from_ptr(st.f_mntonname.as_ptr()) }
            .to_str()
            .map_err(corrupt)?,
    );
    let volume = File::open(&mount)?;
    let mut vst: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstatfs(volume.as_raw_fd(), &mut vst) } != 0
        || volume.metadata()?.dev() != m.dev()
        || unsafe {
            libc::memcmp(
                (&vst.f_fsid as *const libc::fsid_t).cast(),
                (&st.f_fsid as *const libc::fsid_t).cast(),
                std::mem::size_of::<libc::fsid_t>(),
            )
        } != 0
    {
        return Err(diagnostic(
            "identity_unprovable",
            "volume membership mismatch",
        ));
    }
    attrs.commonattr = libc::ATTR_CMN_RETURNED_ATTRS;
    attrs.volattr = libc::ATTR_VOL_INFO | libc::ATTR_VOL_UUID;
    b.fill(0);
    if unsafe {
        libc::fgetattrlist(
            volume.as_raw_fd(),
            (&mut attrs as *mut libc::attrlist).cast(),
            b.as_mut_ptr().cast(),
            b.len(),
            0,
        )
    } != 0
    {
        return Err(diagnostic("identity_unprovable", "volume UUID unavailable"));
    }
    if u32::from_ne_bytes(b[..4].try_into().unwrap()) != 40
        || u32::from_ne_bytes(b[8..12].try_into().unwrap()) & libc::ATTR_VOL_UUID
            != libc::ATTR_VOL_UUID
        || b[24..40] == [0; 16]
    {
        return Err(diagnostic(
            "identity_unprovable",
            "invalid volume UUID attributes",
        ));
    }
    let volume_id = hex(&[&[1], &b[24..40]].concat());
    if blobs::identity(&fs::metadata(&path)?) != blobs::identity(&m) {
        return Err(diagnostic("identity_unprovable", "object replaced"));
    }
    Ok(Object {
        path: text,
        device: m.dev().to_string(),
        inode: m.ino().to_string(),
        volume_id,
        birth_seconds: birth.to_string(),
        birth_nanoseconds: nano.to_string(),
        nonce,
    })
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) fn observe(_: &Path, _: Option<String>) -> Result<Object, RsStoreError> {
    Err(diagnostic(
        "identity_unprovable",
        "this platform has no validated incarnation profile",
    ))
}

/// Pin every canonical component before any home namespace mutation. Trusted
/// system ancestors may be root-owned; the home itself must belong to the user.
/// Open relative to the pinned parent so a swapped symlink is never followed.
#[derive(Debug)]
struct HomeAncestry {
    components: Vec<(PathBuf, File, (u64, u64))>,
}
impl HomeAncestry {
    fn pin(home: &Path) -> Result<Self, RsStoreError> {
        use std::os::fd::FromRawFd;
        if !home.is_absolute() || fs::canonicalize(home)? != home {
            return Err(diagnostic(
                "unsafe_path",
                "account home must be absolute and canonical",
            ));
        }
        let mut chain = Self {
            components: Vec::new(),
        };
        for path in home.ancestors().collect::<Vec<_>>().into_iter().rev() {
            let file = if let Some((_, parent, _)) = chain.components.last() {
                let name = CString::new(path.file_name().unwrap().as_bytes()).map_err(corrupt)?;
                let fd = unsafe {
                    libc::openat(
                        parent.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                    )
                };
                if fd < 0 {
                    return Err(diagnostic("unsafe_path", std::io::Error::last_os_error()));
                }
                unsafe { File::from_raw_fd(fd) }
            } else {
                OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                    .open(path)?
            };
            let m = file.metadata()?;
            chain
                .components
                .push((path.to_owned(), file, (m.dev(), m.ino())));
        }
        chain.verify()?;
        // The identity profile proves local mount membership before creating a child.
        observe(home, None)?;
        Ok(chain)
    }
    fn verify(&self) -> Result<(), RsStoreError> {
        for (index, (path, file, expected)) in self.components.iter().enumerate() {
            let m = file.metadata()?;
            let live = fs::symlink_metadata(path)?;
            let is_home = index + 1 == self.components.len();
            if !m.is_dir()
                || live.file_type().is_symlink()
                || !live.is_dir()
                || (m.dev(), m.ino()) != *expected
                || (live.dev(), live.ino()) != *expected
                || (m.uid() != unsafe { libc::geteuid() } && (is_home || m.uid() != 0))
                || m.mode() & 0o022 != 0
                || live.mode() != m.mode()
                || live.uid() != m.uid()
            {
                return Err(diagnostic(
                    "unsafe_path",
                    format!("unsafe or replaced home ancestor: {}", path.display()),
                ));
            }
            blobs::check_acl(path)?;
            #[cfg(target_os = "linux")]
            linux::check_ancestor(file, path)?;
            #[cfg(target_os = "macos")]
            {
                let mut st: libc::statfs = unsafe { std::mem::zeroed() };
                if unsafe { libc::fstatfs(file.as_raw_fd(), &mut st) } != 0
                    || st.f_flags & libc::MNT_LOCAL as u32 == 0
                {
                    return Err(diagnostic("identity_unprovable", "nonlocal home ancestor"));
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn account_home() -> Result<PathBuf, RsStoreError> {
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result = std::ptr::null_mut();
    let mut buffer = vec![0u8; 65536];
    let rc = unsafe {
        libc::getpwuid_r(
            libc::geteuid(),
            &mut entry,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if rc != 0 || result.is_null() || entry.pw_dir.is_null() {
        return Err(diagnostic(
            "home_unavailable",
            "effective UID account lookup failed",
        ));
    }
    let p = unsafe { CStr::from_ptr(entry.pw_dir) }
        .to_str()
        .map_err(|_| diagnostic("home_unavailable", "non-UTF-8 account home"))?;
    if !Path::new(p).is_absolute() {
        return Err(diagnostic(
            "home_unavailable",
            "relative effective account home",
        ));
    }
    let p = fs::canonicalize(p).map_err(|e| diagnostic("home_unavailable", e))?;
    HomeAncestry::pin(&p)?.verify()?;
    Ok(p)
}
#[derive(Clone, Debug)]
pub(crate) struct GitContext {
    pub project: PathBuf,
    pub common: PathBuf,
    pub admin: PathBuf,
    pub worktree: PathBuf,
    pub relative: String,
}
fn git(project: &Path, args: &[&str]) -> Result<Vec<u8>, RsStoreError> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(project).args(args);
    for (key, _) in std::env::vars_os() {
        if key.as_bytes().starts_with(b"GIT_") {
            cmd.env_remove(key);
        }
    }
    cmd.env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_OPTIONAL_LOCKS", "0");
    let out = cmd
        .output()
        .map_err(|e| diagnostic("identity_unprovable", e))?;
    if !out.status.success() {
        return Err(diagnostic(
            "identity_unprovable",
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    Ok(out.stdout)
}
pub(crate) fn context(project: &Path) -> Result<Option<GitContext>, RsStoreError> {
    let project = fs::canonicalize(project)?;
    if !project
        .ancestors()
        .any(|p| fs::symlink_metadata(p.join(".git")).is_ok())
    {
        if project
            .ancestors()
            .any(|p| p.join("HEAD").exists() && p.join("objects").is_dir())
        {
            return Err(diagnostic(
                "identity_unprovable",
                "bare repository is not a project checkout",
            ));
        }
        return Ok(None);
    }
    let before = fs::metadata(&project)?;
    let marker = project
        .ancestors()
        .map(|p| p.join(".git"))
        .find(|p| fs::symlink_metadata(p).is_ok())
        .ok_or_else(|| diagnostic("identity_unprovable", "Git marker disappeared"))?;
    if fs::symlink_metadata(&marker)?.file_type().is_symlink() {
        return Err(diagnostic("identity_unprovable", "symbolic Git marker"));
    }
    let path = |args: &[&str]| -> Result<PathBuf, RsStoreError> {
        let out = git(&project, args)?;
        let s = std::str::from_utf8(&out)
            .map_err(|_| diagnostic("identity_unprovable", "non-UTF-8 Git identity"))?
            .trim_end_matches('\n');
        fs::canonicalize(s).map_err(io_error)
    };
    let admin = path(&["rev-parse", "--absolute-git-dir"])?;
    let common = path(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
    if admin == common {
        let bare = git(&project, &["rev-parse", "--is-bare-repository"])?;
        if bare == b"true\n" {
            return Err(diagnostic(
                "identity_unprovable",
                "bare repository is not a project checkout",
            ));
        }
        return Ok(None);
    }
    let worktree = path(&["rev-parse", "--show-toplevel"])?;
    if admin.parent() != Some(common.join("worktrees").as_path()) {
        return Err(diagnostic(
            "identity_unprovable",
            "unexpected linked admin parent",
        ));
    }
    let common_link = fs::read_to_string(admin.join("commondir"))?;
    if fs::canonicalize(admin.join(common_link.trim_end_matches('\n')))? != common {
        return Err(diagnostic("identity_unprovable", "Git commondir mismatch"));
    }
    let forward = fs::read_to_string(worktree.join(".git"))?;
    let forward = forward
        .strip_prefix("gitdir: ")
        .ok_or_else(|| diagnostic("identity_unprovable", "invalid worktree .git link"))?
        .trim_end_matches('\n');
    if fs::canonicalize(worktree.join(forward))? != admin {
        return Err(diagnostic(
            "identity_unprovable",
            "Git forward link mismatch",
        ));
    }
    let backlink = fs::read_to_string(admin.join("gitdir"))?;
    if fs::canonicalize(backlink.trim_end_matches('\n'))? != worktree.join(".git") {
        return Err(diagnostic(
            "identity_unprovable",
            "Git admin backlink mismatch",
        ));
    }
    let list = git(&project, &["worktree", "list", "--porcelain", "-z"])?;
    let key = format!(
        "worktree {}",
        worktree
            .to_str()
            .ok_or_else(|| diagnostic("identity_unprovable", "non-UTF-8 worktree"))?
    );
    let mut matches = 0;
    let mut in_record = false;
    for line in list.split(|b| *b == 0) {
        if line.is_empty() {
            in_record = false;
            continue;
        }
        if line == key.as_bytes() {
            matches += 1;
            in_record = true;
        }
        if in_record && (line == b"bare" || line.starts_with(b"prunable")) {
            return Err(diagnostic(
                "identity_unprovable",
                "invalid or prunable worktree registration",
            ));
        }
    }
    if matches != 1 {
        return Err(diagnostic(
            "identity_unprovable",
            "worktree registration mismatch",
        ));
    }
    if blobs::identity(&before) != blobs::identity(&fs::metadata(&project)?) {
        return Err(diagnostic(
            "identity_unprovable",
            "project changed during Git probing",
        ));
    }
    let relative = project
        .strip_prefix(&worktree)
        .map_err(corrupt)?
        .to_str()
        .ok_or_else(|| diagnostic("identity_unprovable", "non-UTF-8 project"))?
        .to_owned();
    Ok(Some(GitContext {
        project,
        common,
        admin,
        worktree,
        relative,
    }))
}

pub(crate) fn resolve(
    project: &Path,
    requested: Option<&Path>,
    enroll: bool,
) -> Result<PathBuf, RsStoreError> {
    let project = fs::canonicalize(project)?;
    let environment = std::env::var_os("XDESIGN_STORE_ROOT").map(PathBuf::from);
    if let Some(root) = requested.or(environment.as_deref()) {
        if root.as_os_str().is_empty() {
            return Err(diagnostic("invalid_input", "empty explicit store root"));
        }
        return Ok(if root.is_absolute() {
            root.to_owned()
        } else {
            project.join(root)
        });
    }
    let Some(ctx) = context(&project)? else {
        return Ok(project.join(".xdesign"));
    };
    resolve_linked(&ctx, &account_home()?, enroll)
}
/// Look up an authenticated binding without selecting authority or creating state.
pub(crate) fn binding_for_project(
    ctx: &GitContext,
    home: &Path,
) -> Result<Option<Binding>, RsStoreError> {
    HomeAncestry::pin(home)?;
    let ns = home.join(".xdesign/worktree-stores/v1");
    if fs::symlink_metadata(&ns).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
        return Ok(None);
    }
    namespace_valid(&ns)?;
    let index = ns
        .join("paths")
        .join(format!("{}.json", path_key(&ns, &ctx.project)?));
    if fs::symlink_metadata(&index).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
        return Ok(None);
    }
    let index = read_path_index(&ns, &ctx.project, &index)?;
    let binding = load_binding(
        &ns,
        index["active_key"]
            .as_str()
            .ok_or_else(|| corrupt("no active binding"))?,
    )?;
    binding.verify()?;
    Ok(Some(binding))
}

fn resolve_linked(
    ctx: &GitContext,
    home: &Path,
    enroll_new: bool,
) -> Result<PathBuf, RsStoreError> {
    let ns = home.join(".xdesign/worktree-stores/v1");
    let local = ctx.project.join(".xdesign");
    let local_exists = fs::symlink_metadata(&local).is_ok();
    if ns.exists() {
        namespace_valid(&ns)?;
        let index = ns
            .join("paths")
            .join(format!("{}.json", path_key(&ns, &ctx.project)?));
        if index.exists() {
            let index = read_path_index(&ns, &ctx.project, &index)?;
            let key = index["active_key"]
                .as_str()
                .ok_or_else(|| corrupt("no active binding"))?;
            let binding = load_binding(&ns, key)?;
            binding.verify()?;
            if enroll_new {
                super::migration::select_initialization(&binding, local_exists)?;
            } else {
                super::migration::select(&binding, local_exists)?;
            }
            return Ok(binding.root());
        }
    }
    if local_exists {
        return Err(diagnostic(
            "migration_required",
            format!(
                "local candidate {}; explicitly enroll and migrate",
                local.display()
            ),
        ));
    }
    if !enroll_new {
        return Err(diagnostic(
            "binding_missing",
            "linked project has no enrolled binding",
        ));
    }
    Ok(enroll(ctx, home, &format!("auto-{}", uuid::Uuid::new_v4()), None)?.root())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn home_ancestry_refuses_writable_and_replaced_parents_before_mutation() {
        let (home, _, _) = fixture();
        let parent = home.parent().unwrap();
        let pinned = HomeAncestry::pin(&home).unwrap();
        // The chain contains ordinary root-owned system ancestors as well as
        // the effective-user-owned fixture. Requiring all ancestors to be ours
        // would reject normal /Users or /home layouts.
        assert!(pinned
            .components
            .iter()
            .any(|(_, f, _)| f.metadata().unwrap().uid() == 0));
        fs::set_permissions(parent, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(ensure_namespace(&home)
            .unwrap_err()
            .to_string()
            .contains("unsafe_path"));
        assert!(!home.join(".xdesign").exists());
        fs::set_permissions(parent, fs::Permissions::from_mode(0o755)).unwrap();
        pinned.verify().unwrap();
        let moved = parent.join("home-moved");
        fs::rename(&home, &moved).unwrap();
        fs::create_dir(&home).unwrap();
        assert!(pinned.verify().is_err());
        fs::remove_dir(&home).unwrap();
        std::os::unix::fs::symlink(&moved, &home).unwrap();
        assert!(pinned.verify().is_err());
        assert!(ensure_namespace(&home).is_err());
        assert!(!moved.join(".xdesign").exists());
    }
    #[test]
    fn safe_home_ancestry_creates_only_the_namespace() {
        let (home, _, _) = fixture();
        let namespace = ensure_namespace(&home).unwrap();
        assert_eq!(namespace, home.join(".xdesign/worktree-stores/v1"));
        HomeAncestry::pin(&home).unwrap().verify().unwrap();
        namespace_valid(&namespace).unwrap();
    }

    #[test]
    fn known_answers() {
        let v: Value = serde_json::from_str(include_str!(
            "../../docs/interfaces/assets/worktree-store-vectors.json"
        ))
        .unwrap();
        for row in v["vectors"].as_array().unwrap() {
            let bytes = encode(row["kind"].as_u64().unwrap() as u8, &row["payload"]).unwrap();
            assert_eq!(hex(&bytes), row["encoding_hex"], "{}", row["name"]);
            assert_eq!(sha256(&bytes), row["encoding_sha256"]);
            if let Some(label) = row["label"].as_str() {
                let input = domain(label, &bytes);
                assert_eq!(hex(&input), row["domain_input_hex"]);
                if row.get("sha256").is_some() {
                    assert_eq!(sha256(&input), row["sha256"]);
                }
                if row.get("hmac_sha256").is_some() {
                    assert_eq!(
                        hmac(&unhex(v["hmac_key_hex"].as_str().unwrap()).unwrap(), &input),
                        row["hmac_sha256"]
                    );
                }
                if row.get("xattr_hex").is_some() {
                    let object: Object =
                        serde_json::from_value(row["payload"]["object"].clone()).unwrap();
                    let payload = &row["payload"];
                    let mut out = b"XDIN\x01".to_vec();
                    out.extend(unhex(payload["namespace_epoch"].as_str().unwrap()).unwrap());
                    out.extend(unhex(object.nonce.as_deref().unwrap()).unwrap());
                    out.extend(unhex(row["hmac_sha256"].as_str().unwrap()).unwrap());
                    assert_eq!(hex(&out), row["xattr_hex"]);
                }
            }
        }
    }
    #[test]
    fn host_capability() {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cache/implementation/worktree-capability")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&p).unwrap();
        let o = observe(&p, None).unwrap();
        assert_eq!(o.path, p.to_str().unwrap());
        let _lease = access::AccessLease::acquire(&[p.clone()]).unwrap();
        blobs::fsync_dir(&p).unwrap();
    }

    fn fixture() -> (PathBuf, PathBuf, PathBuf) {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cache/implementation/location")
            .join(uuid::Uuid::new_v4().to_string());
        let home = base.join("home");
        let primary = base.join("primary");
        let linked = base.join("linked");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir(&primary).unwrap();
        for args in [
            vec!["init", "-q"],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "fixture",
            ],
            vec![
                "worktree",
                "add",
                "-q",
                "--detach",
                linked.to_str().unwrap(),
            ],
        ] {
            let output = Command::new("git")
                .current_dir(&primary)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        (home, primary, linked)
    }

    #[test]
    fn linked_defaults_are_scoped_alias_stable_and_preserve_global_siblings() {
        let (home, primary, linked) = fixture();
        let unrelated = home.join(".xdesign/orca-xdesign");
        fs::create_dir_all(&unrelated).unwrap();
        fs::write(unrelated.join("keep"), b"untouched").unwrap();
        let ctx = context(&linked).unwrap().unwrap();
        assert!(context(&primary).unwrap().is_none());
        assert_eq!(
            access::reason(&resolve_linked(&ctx, &home, false).unwrap_err()),
            Some("binding_missing")
        );
        assert!(!home.join(".xdesign/worktree-stores").exists());
        let selected = resolve_linked(&ctx, &home, true).unwrap();
        assert!(selected.starts_with(home.join(".xdesign/worktree-stores/v1/projects")));
        super::super::RsStore::init(&selected).unwrap();
        assert_eq!(resolve_linked(&ctx, &home, false).unwrap(), selected);
        let alias = linked.parent().unwrap().join("alias");
        std::os::unix::fs::symlink(&linked, &alias).unwrap();
        assert_eq!(
            resolve_linked(&context(&alias).unwrap().unwrap(), &home, false).unwrap(),
            selected
        );
        let nested = linked.join("nested");
        fs::create_dir(&nested).unwrap();
        let other = resolve_linked(&context(&nested).unwrap().unwrap(), &home, true).unwrap();
        assert_ne!(selected, other);
        let sibling = primary.parent().unwrap().join("second");
        assert!(Command::new("git")
            .current_dir(&primary)
            .args(["worktree", "add", "-q", "--detach"])
            .arg(&sibling)
            .status()
            .unwrap()
            .success());
        let third = resolve_linked(&context(&sibling).unwrap().unwrap(), &home, true).unwrap();
        assert_ne!(third, selected);
        assert_ne!(third, other);
        assert_eq!(fs::read(unrelated.join("keep")).unwrap(), b"untouched");
        assert!(!linked.join(".xdesign").exists());
        let journal = selected.parent().unwrap().join("initialization.json");
        let original = json_file(&journal).unwrap();
        let mut corrupted = original.clone();
        corrupted["foreign_member"] = json!(true);
        replace_json(&journal, &corrupted).unwrap();
        assert_eq!(
            access::reason(&super::super::RsStore::open(&selected).err().unwrap()),
            Some("coordination_corrupt")
        );
        replace_json(&journal, &original).unwrap();
        super::super::RsStore::open(&selected).unwrap();
    }

    #[test]
    fn local_candidate_and_partial_global_store_are_never_adopted() {
        let (home, _, linked) = fixture();
        let ctx = context(&linked).unwrap().unwrap();
        fs::create_dir(linked.join(".xdesign")).unwrap();
        assert_eq!(
            access::reason(&resolve_linked(&ctx, &home, true).unwrap_err()),
            Some("migration_required")
        );
        assert!(!home.join(".xdesign").exists());
        fs::create_dir(home.join(".xdesign")).unwrap();
        fs::write(home.join(".xdesign/store.db"), b"SQLite format 3\0foreign").unwrap();
        assert_eq!(
            access::reason(&enroll(&ctx, &home, "collision", None).unwrap_err()),
            Some("unsafe_path")
        );
        assert_eq!(
            fs::read(home.join(".xdesign/store.db")).unwrap(),
            b"SQLite format 3\0foreign"
        );
        assert!(!home.join(".xdesign/worktree-stores").exists());
    }

    #[test]
    fn a_path_index_cannot_redirect_to_another_valid_project_binding() {
        let (home, _, linked) = fixture();
        let ctx = context(&linked).unwrap().unwrap();
        let first = enroll(&ctx, &home, "first", None).unwrap();
        let nested = linked.join("nested");
        fs::create_dir(&nested).unwrap();
        let other = enroll(&context(&nested).unwrap().unwrap(), &home, "second", None).unwrap();
        let path = first.namespace.join("paths").join(format!(
            "{}.json",
            path_key(&first.namespace, &ctx.project).unwrap()
        ));
        let mut value = json_file(&path).unwrap();
        value["active_key"] = json!(other.key());
        value["binding_keys"] = json!([other.key()]);
        replace_json(&path, &value).unwrap();
        assert_eq!(
            access::reason(&resolve_linked(&ctx, &home, false).unwrap_err()),
            Some("coordination_corrupt")
        );
        assert_eq!(
            access::reason(&enroll(&ctx, &home, "third", None).unwrap_err()),
            Some("coordination_corrupt")
        );
        assert!(!first.root().exists());
        assert!(!other.root().exists());
    }

    #[test]
    fn changed_incarnation_is_refused_and_enrollment_retry_finishes_routes() {
        let (home, _, linked) = fixture();
        let ctx = context(&linked).unwrap().unwrap();
        let binding = enroll(&ctx, &home, "repeat", None).unwrap();
        // Losing a derivative route is recoverable only by explicit enrollment.
        let route_dir = linked.join(".xdesign-access-v1/routes");
        for bucket in fs::read_dir(&route_dir).unwrap() {
            for file in fs::read_dir(bucket.unwrap().path()).unwrap() {
                fs::remove_file(file.unwrap().path()).unwrap();
            }
        }
        assert!(resolve_linked(&ctx, &home, false).is_err());
        let again = enroll(&ctx, &home, "repeat", None).unwrap();
        assert_eq!(again.bytes, binding.bytes);
        assert_eq!(resolve_linked(&ctx, &home, false).unwrap(), binding.root());
        let moved = linked.parent().unwrap().join("moved");
        fs::rename(&linked, &moved).unwrap();
        fs::create_dir(&linked).unwrap();
        assert!(binding.verify().is_err());
        assert!(enroll(&ctx, &home, "replacement", None).is_err());
    }
}

pub(crate) const INCARNATION: &str = "org.xdesign.incarnation.v1";
pub(crate) const ACCESS_ATTRIBUTE: &str = "org.xdesign.access.v1";
#[cfg(target_os = "macos")]
pub(crate) fn attribute(path: &Path, name: &str) -> Result<Option<Vec<u8>>, RsStoreError> {
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let name = CString::new(name).map_err(corrupt)?;
    let n = unsafe { libc::fgetxattr(f.as_raw_fd(), name.as_ptr(), std::ptr::null_mut(), 0, 0, 0) };
    if n < 0 {
        let e = std::io::Error::last_os_error();
        if e.raw_os_error() == Some(libc::ENOATTR) {
            return Ok(None);
        }
        return Err(diagnostic("identity_unprovable", e));
    }
    if n > 65536 {
        return Err(corrupt("oversized attribute"));
    }
    let mut b = vec![0; n as usize];
    if unsafe {
        libc::fgetxattr(
            f.as_raw_fd(),
            name.as_ptr(),
            b.as_mut_ptr().cast(),
            b.len(),
            0,
            0,
        )
    } != n
    {
        return Err(diagnostic("identity_unprovable", "attribute changed"));
    }
    Ok(Some(b))
}
#[cfg(target_os = "linux")]
pub(crate) fn attribute(path: &Path, name: &str) -> Result<Option<Vec<u8>>, RsStoreError> {
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let native = name.replacen("org.xdesign.", "user.xdesign.", 1);
    let name = CString::new(native).map_err(corrupt)?;
    let n = unsafe { libc::fgetxattr(f.as_raw_fd(), name.as_ptr(), std::ptr::null_mut(), 0) };
    if n < 0 {
        let e = std::io::Error::last_os_error();
        if matches!(e.raw_os_error(), Some(libc::ENODATA) | Some(libc::ENOTSUP)) {
            return Ok(None);
        }
        return Err(io_error(e));
    }
    if n > 65536 {
        return Err(corrupt("oversized attribute"));
    }
    let mut b = vec![0; n as usize];
    if unsafe { libc::fgetxattr(f.as_raw_fd(), name.as_ptr(), b.as_mut_ptr().cast(), b.len()) } != n
    {
        return Err(corrupt("attribute changed"));
    }
    Ok(Some(b))
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) fn attribute(_: &Path, _: &str) -> Result<Option<Vec<u8>>, RsStoreError> {
    Err(diagnostic(
        "coordination_unavailable",
        "unsupported attribute profile",
    ))
}
#[cfg(target_os = "macos")]
pub(crate) fn install_attribute(path: &Path, name: &str, bytes: &[u8]) -> Result<(), RsStoreError> {
    if let Some(old) = attribute(path, name)? {
        if old != bytes {
            return Err(diagnostic(
                "identity_unprovable",
                "existing attribute differs",
            ));
        }
        return Ok(());
    }
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let name = CString::new(name).map_err(corrupt)?;
    if unsafe {
        libc::fsetxattr(
            f.as_raw_fd(),
            name.as_ptr(),
            bytes.as_ptr().cast(),
            bytes.len(),
            0,
            libc::XATTR_CREATE,
        )
    } != 0
    {
        return Err(diagnostic(
            "identity_unprovable",
            std::io::Error::last_os_error(),
        ));
    }
    blobs::sync_durable(&f).map_err(|e| diagnostic("coordination_unavailable", e))?;
    if attribute(path, name.to_str().unwrap())?.as_deref() != Some(bytes) {
        return Err(diagnostic(
            "identity_unprovable",
            "attribute readback failed",
        ));
    }
    Ok(())
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) fn install_attribute(_: &Path, _: &str, _: &[u8]) -> Result<(), RsStoreError> {
    Err(diagnostic(
        "identity_unprovable",
        "unsupported attribute profile",
    ))
}

fn publication_pool(parent: &Path) -> PathBuf {
    if let Some(access) = parent
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == ".xdesign-access-v1"))
    {
        return access.join("reservation-drafts");
    }
    if parent
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|n| n == "migrations")
    {
        parent
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("reservation-drafts")
    } else {
        parent.to_owned()
    }
}
pub(crate) fn replace_json(path: &Path, value: &Value) -> Result<(), RsStoreError> {
    let parent = path.parent().unwrap();
    let pool = publication_pool(parent);
    safe_dir(parent)?;
    safe_dir(&pool)?;
    let draft = pool.join(format!(".draft-{}", random()?));
    let bytes = serde_json::to_vec(value).map_err(corrupt)?;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&draft)?;
    f.write_all(&bytes)?;
    blobs::sync_durable(&f)?;
    drop(f);
    fs::rename(&draft, path)?;
    blobs::fsync_dir(&pool)?;
    blobs::fsync_dir(parent)
}

#[derive(Clone, Debug)]
pub(crate) struct Binding {
    home_ancestry: std::sync::Arc<HomeAncestry>,
    pub namespace: PathBuf,
    pub key_bytes: Vec<u8>,
    pub payload: Value,
    pub bytes: Vec<u8>,
}
impl Binding {
    pub(crate) fn root(&self) -> PathBuf {
        PathBuf::from(self.payload["store_root"].as_str().unwrap())
    }
    pub(crate) fn key(&self) -> &str {
        self.payload["key"].as_str().unwrap()
    }
    pub(crate) fn digest(&self) -> String {
        sha256(&self.bytes)
    }
    pub(crate) fn project(&self) -> PathBuf {
        PathBuf::from(self.payload["project"]["path"].as_str().unwrap())
    }
    /// Authentication of the retained location does not require the old Git
    /// checkout to remain live. Explicit read-only inspection uses this subset.
    pub(crate) fn verify_archive(&self) -> Result<(), RsStoreError> {
        self.home_ancestry.verify()?;
        let ns = json_file(&self.namespace.join("namespace.json"))?;
        if ns["epoch"] != self.payload["namespace_epoch"]
            || ns["owner_uid"] != self.payload["owner_uid"]
        {
            return Err(corrupt("namespace identity differs"));
        }
        if blobs::read_store_file(&self.namespace.join("key"), 0o600)? != self.key_bytes {
            return Err(corrupt("namespace key replaced"));
        }
        let home: Object = serde_json::from_value(self.payload["home"].clone()).map_err(corrupt)?;
        if observe(Path::new(&home.path), home.nonce.clone())? != home {
            return Err(diagnostic("identity_unprovable", "home identity changed"));
        }
        Ok(())
    }
    pub(crate) fn verify(&self) -> Result<(), RsStoreError> {
        self.verify_archive()?;
        for role in ["home", "common", "admin", "worktree", "project"] {
            let object: Object =
                serde_json::from_value(self.payload[role].clone()).map_err(corrupt)?;
            let live = observe(Path::new(&object.path), object.nonce.clone())?;
            if live != object {
                return Err(diagnostic(
                    "identity_unprovable",
                    format!("{role} identity changed"),
                ));
            }
            if role != "home" {
                let expected =
                    incarnation(&self.key_bytes, &self.payload["namespace_epoch"], &object)?;
                if attribute(Path::new(&object.path), INCARNATION)?.as_deref() != Some(&expected) {
                    return Err(diagnostic(
                        "identity_unprovable",
                        format!("{role} incarnation changed"),
                    ));
                }
                let record = json_file(
                    &self
                        .namespace
                        .join("objects")
                        .join(format!("{}.json", object.nonce.as_deref().unwrap())),
                )?;
                let registry = authenticate(&self.key_bytes, "object", 4, &record)?;
                if registry["object"] != object.value()
                    || registry["namespace_epoch"] != self.payload["namespace_epoch"]
                    || !registry["roles"]
                        .as_array()
                        .is_some_and(|a| a.contains(&json!(role)))
                {
                    return Err(corrupt("object registry differs"));
                }
            }
        }
        let ctx = context(&self.project())?
            .ok_or_else(|| diagnostic("identity_unprovable", "no longer a linked checkout"))?;
        if self.payload["common"]["path"] != json!(ctx.common)
            || self.payload["admin"]["path"] != json!(ctx.admin)
            || self.payload["worktree"]["path"] != json!(ctx.worktree)
            || self.payload["project_relative"] != ctx.relative
        {
            return Err(diagnostic("identity_unprovable", "Git binding changed"));
        }
        Ok(())
    }
}
fn incarnation(key: &[u8], epoch: &Value, object: &Object) -> Result<Vec<u8>, RsStoreError> {
    let payload = json!({"schema_version":1,"namespace_epoch":epoch,"owner_uid":unsafe{libc::geteuid()}.to_string(),"object":object});
    let mac = hmac(key, &domain("incarnation", &encode(9, &payload)?));
    Ok([
        b"XDIN\x01".as_slice(),
        &unhex(epoch.as_str().ok_or_else(|| corrupt("epoch"))?)?,
        &unhex(
            object
                .nonce
                .as_deref()
                .ok_or_else(|| corrupt("missing nonce"))?,
        )?,
        &unhex(&mac)?,
    ]
    .concat())
}
pub(crate) fn load_binding(namespace: &Path, key: &str) -> Result<Binding, RsStoreError> {
    if !super::valid_digest(key) {
        return Err(corrupt("invalid binding key"));
    }
    namespace_valid(namespace)?;
    let key_bytes = blobs::read_store_file(&namespace.join("key"), 0o600)?;
    let bytes = blobs::read_store_file(
        &namespace.join("projects").join(key).join("binding.json"),
        0o600,
    )?;
    let record = strict_json(&bytes).map_err(corrupt)?;
    let payload = authenticate(&key_bytes, "binding", 3, &record)?;
    let mut tuple = payload.clone();
    tuple.as_object_mut().unwrap().remove("key");
    tuple.as_object_mut().unwrap().remove("store_root");
    if sha256(&domain("binding-key", &encode(2, &tuple)?)) != key
        || payload["key"] != key
        || payload["store_root"] != json!(namespace.join("projects").join(key).join("store"))
    {
        return Err(diagnostic(
            "binding_collision",
            "binding tuple or derived root differs",
        ));
    }
    let home_ancestry = std::sync::Arc::new(HomeAncestry::pin(Path::new(
        payload["home"]["path"]
            .as_str()
            .ok_or_else(|| corrupt("home path"))?,
    ))?);
    Ok(Binding {
        home_ancestry,
        namespace: namespace.to_owned(),
        key_bytes,
        payload,
        bytes,
    })
}
fn namespace_valid(ns: &Path) -> Result<(), RsStoreError> {
    safe_dir(ns)?;
    let marker = json_file(&ns.join("namespace.json"))?;
    if marker.as_object().map(|m| m.len()) != Some(4)
        || marker["format"] != "xdesign.worktree-namespace"
        || marker["schema_version"] != 1
        || marker["owner_uid"] != unsafe { libc::geteuid() }.to_string()
        || !marker["epoch"]
            .as_str()
            .is_some_and(|s| super::valid_digest(s) && s != "0".repeat(64))
    {
        return Err(corrupt("invalid namespace"));
    }
    let key = blobs::read_store_file(&ns.join("key"), 0o600)?;
    if key.len() != 32 || key == [0; 32] {
        return Err(corrupt("invalid namespace key"));
    }
    for item in fs::read_dir(ns)? {
        let item = item?;
        if ![
            "namespace.json",
            "key",
            "enrollments",
            "objects",
            "paths",
            "projects",
        ]
        .iter()
        .any(|n| item.file_name() == *n)
        {
            return Err(corrupt("foreign namespace entry"));
        }
    }
    Ok(())
}
fn path_key(ns: &Path, project: &Path) -> Result<String, RsStoreError> {
    let marker = json_file(&ns.join("namespace.json"))?;
    Ok(sha256(&domain(
        "project-path-key",
        &encode(
            5,
            &json!({"schema_version":1,"namespace_epoch":marker["epoch"],"owner_uid":marker["owner_uid"],"path":project.to_str().ok_or_else(||diagnostic("identity_unprovable","non-UTF-8 project"))?}),
        )?,
    )))
}
fn read_path_index(ns: &Path, project: &Path, path: &Path) -> Result<Value, RsStoreError> {
    let value = json_file(path)?;
    let names = [
        "schema_version",
        "path",
        "generation",
        "binding_keys",
        "active_key",
        "enrollment_id",
    ];
    if !value
        .as_object()
        .is_some_and(|m| m.len() == names.len() && m.keys().all(|k| names.contains(&k.as_str())))
        || value["schema_version"] != 1
        || value["path"] != json!(project)
        || !value["enrollment_id"].as_str().is_some_and(super::valid_id)
    {
        return Err(corrupt("path index shape or selected project differs"));
    }
    let keys = value["binding_keys"]
        .as_array()
        .ok_or_else(|| corrupt("path index has no binding list"))?;
    if keys.is_empty()
        || value["generation"] != keys.len().to_string()
        || !keys.contains(&value["active_key"])
    {
        return Err(corrupt("path index generation or active key differs"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for key in keys {
        let key = key
            .as_str()
            .filter(|key| super::valid_digest(key))
            .ok_or_else(|| corrupt("invalid indexed binding key"))?;
        if !seen.insert(key) || load_binding(ns, key)?.project() != project {
            return Err(corrupt(
                "path index references a different project or duplicate binding",
            ));
        }
    }
    Ok(value)
}
fn ensure_namespace(home: &Path) -> Result<PathBuf, RsStoreError> {
    let ancestry = HomeAncestry::pin(home)?;
    let container = home.join(".xdesign");
    for p in [&container, &container.join("worktree-stores")] {
        if [
            "format.json",
            "store.db",
            "store.db-wal",
            "store.db-shm",
            "blobs",
            "staging",
            "locks",
        ]
        .iter()
        .any(|name| fs::symlink_metadata(p.join(name)).is_ok())
        {
            return Err(diagnostic("unsafe_path","global container has recognizable store content; it cannot contain the worktree namespace"));
        }
    }
    for p in [&container, &container.join("worktree-stores")] {
        let _lease = access::AccessLease::acquire(&[p.parent().unwrap().to_owned()])?;
        ancestry.verify()?;
        if p.exists() {
            safe_dir(p)?;
        } else {
            mkdir(p)?;
        }
    }
    let versions = container.join("worktree-stores");
    for entry in fs::read_dir(&versions)? {
        let entry = entry?;
        let name = entry.file_name();
        let valid = name.to_str().is_some_and(|s| {
            s.strip_prefix('v')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        });
        if !valid || !entry.file_type()?.is_dir() {
            return Err(diagnostic(
                "binding_collision",
                "foreign content in reserved worktree-stores container",
            ));
        }
    }
    let ns = container.join("worktree-stores/v1");
    let _lease = access::AccessLease::acquire(&[ns.parent().unwrap().to_owned()])?;
    ancestry.verify()?;
    if ns.exists() {
        namespace_valid(&ns)?;
        return Ok(ns);
    }
    mkdir(&ns)?;
    let epoch = random()?;
    publish_json(
        &ns.join("namespace.json"),
        &json!({"format":"xdesign.worktree-namespace","schema_version":1,"owner_uid":unsafe{libc::geteuid()}.to_string(),"epoch":epoch}),
    )?;
    publish(&ns.join("key"), &unhex(&random()?)?)?;
    for child in ["enrollments", "objects", "paths", "projects"] {
        mkdir(&ns.join(child))?;
    }
    ancestry.verify()?;
    namespace_valid(&ns)?;
    Ok(ns)
}
pub(crate) fn enroll(
    ctx: &GitContext,
    home: &Path,
    id: &str,
    previous: Option<&str>,
) -> Result<Binding, RsStoreError> {
    if !super::valid_id(id) {
        return Err(diagnostic(
            "invalid_input",
            "invalid enrollment operation ID",
        ));
    }
    let ancestry = HomeAncestry::pin(home)?;
    let ns = ensure_namespace(home)?;
    let mut paths = vec![
        ns.clone(),
        ctx.project.clone(),
        ctx.common.clone(),
        ctx.admin.clone(),
        ctx.worktree.clone(),
    ];
    paths.push(home.to_owned());
    let lease = access::AccessLease::acquire(&paths)?;
    ancestry.verify()?;
    let marker = json_file(&ns.join("namespace.json"))?;
    let key = blobs::read_store_file(&ns.join("key"), 0o600)?;
    let journal = ns.join("enrollments").join(id);
    mkdir(&journal)?;
    let request = json!({"schema_version":1,"operation_id":id,"project_root":ctx.project,"home":home,"previous_binding":previous});
    if journal.join("request.json").exists() && json_file(&journal.join("request.json"))? != request
    {
        return Err(diagnostic(
            "operation_mismatch",
            "enrollment arguments changed",
        ));
    }
    publish_json(&journal.join("request.json"), &request)?;
    let index_path = ns
        .join("paths")
        .join(format!("{}.json", path_key(&ns, &ctx.project)?));
    let old = if index_path.exists() {
        Some(read_path_index(&ns, &ctx.project, &index_path)?)
    } else {
        None
    };
    if let Some(old) = &old {
        let active = old["active_key"]
            .as_str()
            .ok_or_else(|| corrupt("path has no active binding"))?;
        let binding = load_binding(&ns, active)?;
        if binding.verify().is_ok() {
            if previous.is_some() {
                return Err(diagnostic(
                    "invalid_input",
                    "current incarnation is still valid",
                ));
            }
            drop(lease);
            super::migration::install_routes(&binding)?;
            return Ok(binding);
        }
        if previous != Some(active) {
            return Err(diagnostic(
                "identity_unprovable",
                format!("prior binding {active}; explicit new-incarnation acknowledgment required"),
            ));
        }
    } else if previous.is_some() {
        return Err(diagnostic(
            "invalid_input",
            "no prior incarnation to acknowledge",
        ));
    }
    let mut payload = if journal.join("state.json").exists() {
        json_file(&journal.join("state.json"))?["binding_payload"].clone()
    } else {
        let mut payload = json!({"schema_version":1,"namespace_epoch":marker["epoch"],"owner_uid":marker["owner_uid"],"project_relative":ctx.relative});
        let mut objects = std::collections::BTreeMap::<PathBuf, Object>::new();
        for (role, path) in [
            ("home", home),
            ("common", ctx.common.as_path()),
            ("admin", ctx.admin.as_path()),
            ("worktree", ctx.worktree.as_path()),
            ("project", ctx.project.as_path()),
        ] {
            let object = if let Some(o) = objects.get(path) {
                o.clone()
            } else {
                let nonce = if role == "home" {
                    marker["epoch"].as_str().unwrap().to_owned()
                } else if let Some(attr) = attribute(path, INCARNATION)? {
                    if attr.len() != 101
                        || attr[..5] != *b"XDIN\x01"
                        || hex(&attr[5..37]) != marker["epoch"]
                    {
                        return Err(diagnostic("identity_unprovable", "foreign incarnation"));
                    }
                    let nonce = hex(&attr[37..69]);
                    let registry = authenticate(
                        &key,
                        "object",
                        4,
                        &json_file(&ns.join("objects").join(format!("{nonce}.json")))?,
                    )?;
                    let object = observe(path, Some(nonce.clone()))?;
                    if registry["object"] != object.value()
                        || incarnation(&key, &marker["epoch"], &object)? != attr
                    {
                        return Err(diagnostic(
                            "identity_unprovable",
                            "copied or changed incarnation",
                        ));
                    }
                    nonce
                } else {
                    random()?
                };
                let o = observe(path, Some(nonce))?;
                objects.insert(path.to_owned(), o.clone());
                o
            };
            payload[role] = object.value();
        }
        publish_json(
            &journal.join("state.json"),
            &json!({"schema_version":1,"phase":"reserved","binding_payload":payload}),
        )?;
        payload
    };
    for role in ["home", "common", "admin", "worktree", "project"] {
        let object: Object = serde_json::from_value(payload[role].clone()).map_err(corrupt)?;
        if observe(Path::new(&object.path), object.nonce.clone())? != object {
            return Err(diagnostic(
                "identity_unprovable",
                "enrollment object changed",
            ));
        }
        if role != "home" {
            let roles: Vec<_> = ["admin", "common", "project", "worktree"]
                .into_iter()
                .filter(|r| payload[*r]["path"] == object.path)
                .collect();
            let registry = sign(
                &key,
                "object",
                4,
                json!({"schema_version":1,"namespace_epoch":marker["epoch"],"owner_uid":marker["owner_uid"],"object":object,"roles":roles}),
            )?;
            let registry_path = ns
                .join("objects")
                .join(format!("{}.json", object.nonce.as_deref().unwrap()));
            if registry_path.exists() {
                let existing = authenticate(&key, "object", 4, &json_file(&registry_path)?)?;
                if existing["object"] != object.value()
                    || !roles.iter().all(|r| {
                        existing["roles"]
                            .as_array()
                            .is_some_and(|a| a.contains(&json!(r)))
                    })
                {
                    return Err(corrupt("object registry role mismatch"));
                }
            } else {
                publish_json(&registry_path, &registry)?;
            }
            install_attribute(
                Path::new(&object.path),
                INCARNATION,
                &incarnation(&key, &marker["epoch"], &object)?,
            )?;
        }
    }
    let binding_key = sha256(&domain("binding-key", &encode(2, &payload)?));
    payload["key"] = json!(binding_key);
    payload["store_root"] = json!(ns.join("projects").join(&binding_key).join("store"));
    let parent = ns.join("projects").join(&binding_key);
    mkdir(&parent)?;
    publish_json(
        &parent.join("binding.json"),
        &sign(&key, "binding", 3, payload.clone())?,
    )?;
    let candidate = load_binding(&ns, &binding_key)?;
    candidate.verify()?;
    let mut keys = old
        .as_ref()
        .and_then(|o| o["binding_keys"].as_array())
        .cloned()
        .unwrap_or_default();
    if !keys.contains(&json!(binding_key)) {
        keys.push(json!(binding_key));
    }
    replace_json(
        &index_path,
        &json!({"schema_version":1,"path":ctx.project,"generation":keys.len().to_string(),"binding_keys":keys,"active_key":binding_key,"enrollment_id":id}),
    )?;
    replace_json(
        &journal.join("state.json"),
        &json!({"schema_version":1,"phase":"published","binding_payload":payload}),
    )?;
    lease.verify()?;
    drop(lease);
    let binding = load_binding(&ns, &binding_key)?;
    binding.verify()?;
    super::migration::install_routes(&binding)?;
    Ok(binding)
}

#[cfg(target_os = "macos")]
pub(crate) fn replace_attribute(
    path: &Path,
    name: &str,
    expected: &[u8],
    bytes: &[u8],
) -> Result<(), RsStoreError> {
    if attribute(path, name)?.as_deref() != Some(expected) {
        return Err(diagnostic(
            "identity_unprovable",
            "attribute changed before replacement",
        ));
    }
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let n = CString::new(name).map_err(corrupt)?;
    if unsafe {
        libc::fsetxattr(
            f.as_raw_fd(),
            n.as_ptr(),
            bytes.as_ptr().cast(),
            bytes.len(),
            0,
            libc::XATTR_REPLACE,
        )
    } != 0
    {
        return Err(diagnostic(
            "identity_unprovable",
            std::io::Error::last_os_error(),
        ));
    }
    blobs::sync_durable(&f).map_err(|e| diagnostic("coordination_unavailable", e))?;
    if attribute(path, name)?.as_deref() != Some(bytes) {
        return Err(corrupt("attribute replacement readback differs"));
    }
    Ok(())
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) fn replace_attribute(_: &Path, _: &str, _: &[u8], _: &[u8]) -> Result<(), RsStoreError> {
    Err(diagnostic(
        "identity_unprovable",
        "unsupported attribute profile",
    ))
}

#[cfg(target_os = "linux")]
pub(crate) fn install_attribute(path: &Path, name: &str, bytes: &[u8]) -> Result<(), RsStoreError> {
    linux::write_attribute(path, name, None, bytes)
}
#[cfg(target_os = "linux")]
pub(crate) fn replace_attribute(
    path: &Path,
    name: &str,
    expected: &[u8],
    bytes: &[u8],
) -> Result<(), RsStoreError> {
    linux::write_attribute(path, name, Some(expected), bytes)
}
