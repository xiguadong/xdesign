//! Linux I21 observations. O_PATH avoids dropping SQLite's process-owned locks
//! when observing an already open database (filp_close skips locks for FMODE_PATH).
use super::*;

fn unprovable(detail: impl std::fmt::Display) -> RsStoreError {
    diagnostic("identity_unprovable", detail)
}
fn statx(file: &File) -> Result<libc::statx, RsStoreError> {
    let mut value: libc::statx = unsafe { std::mem::zeroed() };
    let requested = libc::STATX_BASIC_STATS | libc::STATX_BTIME | libc::STATX_MNT_ID;
    if unsafe {
        libc::statx(
            file.as_raw_fd(),
            c"".as_ptr(),
            libc::AT_EMPTY_PATH | libc::AT_SYMLINK_NOFOLLOW,
            requested,
            &mut value,
        )
    } != 0
    {
        return Err(unprovable(std::io::Error::last_os_error()));
    }
    let required = libc::STATX_INO
        | libc::STATX_BTIME
        | libc::STATX_MNT_ID
        | libc::STATX_UID
        | libc::STATX_MODE
        | libc::STATX_TYPE;
    if value.stx_mask & required != required
        || value.stx_btime.tv_sec == 0
        || value.stx_btime.tv_nsec >= 1_000_000_000
        || value.stx_mnt_id == 0
    {
        return Err(unprovable(
            "statx lacks valid birth, inode, mount, owner or mode evidence",
        ));
    }
    let m = file.metadata()?;
    if value.stx_ino != m.ino()
        || value.stx_dev_major != libc::major(m.dev())
        || value.stx_dev_minor != libc::minor(m.dev())
        || value.stx_uid != m.uid()
        || u32::from(value.stx_mode) != m.mode()
    {
        return Err(unprovable("statx/fstat identity mismatch"));
    }
    Ok(value)
}
fn mount_path(encoded: &str) -> Result<PathBuf, RsStoreError> {
    let mut bytes = Vec::new();
    let mut input = encoded.as_bytes().iter().copied();
    while let Some(b) = input.next() {
        if b == b'\\' {
            let code: Vec<_> = input.by_ref().take(3).collect();
            bytes.push(match code.as_slice() {
                b"040" => b' ',
                b"011" => b'\t',
                b"012" => b'\n',
                b"134" => b'\\',
                _ => return Err(unprovable("invalid mountinfo escape")),
            });
        } else {
            bytes.push(b);
        }
    }
    let text = String::from_utf8(bytes).map_err(unprovable)?;
    let path = PathBuf::from(text);
    if !path.is_absolute() {
        return Err(unprovable("relative mountinfo path"));
    }
    Ok(path)
}
fn volume(file: &File, path: &Path, x: &libc::statx) -> Result<String, RsStoreError> {
    let mut st: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstatfs(file.as_raw_fd(), &mut st) } != 0 {
        return Err(unprovable(std::io::Error::last_os_error()));
    }
    // Encode the two fsid words, never padding or a signed/native-endian struct.
    let mut fsid = [0u32; 2];
    if std::mem::size_of_val(&st.f_fsid) != 8 {
        return Err(unprovable("unsupported fsid width"));
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            (&st.f_fsid as *const libc::fsid_t).cast::<u8>(),
            fsid.as_mut_ptr().cast::<u8>(),
            8,
        );
    }
    if fsid == [0, 0] {
        return Err(unprovable("zero filesystem ID"));
    }
    let expected_type = match st.f_type as u64 {
        0xef53 => "ext4",
        0x58465342 => "xfs",
        0x9123683e => "btrfs",
        _ => {
            return Err(unprovable(
                "filesystem lacks an admitted local birth/xattr/lock/flush profile",
            ))
        }
    };
    let info = fs::read_to_string("/proc/self/mountinfo").map_err(unprovable)?;
    let device = format!("{}:{}", x.stx_dev_major, x.stx_dev_minor);
    let mut matches = Vec::new();
    for line in info.lines() {
        let (left, right) = line
            .split_once(" - ")
            .ok_or_else(|| unprovable("invalid mountinfo"))?;
        let fields: Vec<_> = left.split_whitespace().collect();
        let after: Vec<_> = right.split_whitespace().collect();
        if fields.len() < 6 || after.len() < 3 {
            return Err(unprovable("short mountinfo"));
        }
        // A second mount can expose the same filesystem ID through a different
        // device number. Check admitted local mounts as well as equal devices.
        if fields[2] != device && after[0] == expected_type {
            let other_path = mount_path(fields[4])?;
            let other = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(&other_path)
                .map_err(unprovable)?;
            let mut other_st: libc::statfs = unsafe { std::mem::zeroed() };
            if unsafe { libc::fstatfs(other.as_raw_fd(), &mut other_st) } != 0 {
                return Err(unprovable("cannot verify other local mount"));
            }
            if other_st.f_type == st.f_type
                && unsafe {
                    libc::memcmp(
                        (&other_st.f_fsid as *const libc::fsid_t).cast(),
                        (&st.f_fsid as *const libc::fsid_t).cast(),
                        8,
                    )
                } == 0
            {
                return Err(unprovable("ambiguous filesystem ID mount mapping"));
            }
        }
        if fields[2] == device {
            matches.push((
                fields[0].parse::<u64>().map_err(unprovable)?,
                mount_path(fields[3])?,
                mount_path(fields[4])?,
                after[0].to_owned(),
            ));
        }
    }
    if matches.len() != 1 {
        return Err(unprovable(
            "ambiguous device mount mapping (bind/duplicate mount)",
        ));
    }
    let (id, root, mount, fs_type) = &matches[0];
    if *id != x.stx_mnt_id
        || root != Path::new("/")
        || fs_type != expected_type
        || !path.starts_with(mount)
    {
        return Err(unprovable(
            "nonlocal, aliased or inconsistent mount topology",
        ));
    }
    let mounted = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(mount)?;
    let mounted_x = statx(&mounted)?;
    let mut mounted_st: libc::statfs = unsafe { std::mem::zeroed() };
    if mounted_x.stx_mnt_id != *id
        || mounted.metadata()?.dev() != file.metadata()?.dev()
        || unsafe { libc::fstatfs(mounted.as_raw_fd(), &mut mounted_st) } != 0
        || mounted_st.f_type != st.f_type
        || unsafe {
            libc::memcmp(
                (&mounted_st.f_fsid as *const libc::fsid_t).cast(),
                (&st.f_fsid as *const libc::fsid_t).cast(),
                8,
            )
        } != 0
    {
        return Err(unprovable("mount root identity mismatch"));
    }
    let mut bytes = vec![2];
    bytes.extend((st.f_type as u64).to_be_bytes());
    for word in fsid {
        bytes.extend(word.to_be_bytes());
    }
    Ok(hex(&bytes))
}
pub(crate) fn observe(path: &Path, nonce: Option<String>) -> Result<Object, RsStoreError> {
    let canonical = fs::canonicalize(path)?;
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(diagnostic("unsafe_path", "symbolic identity object"));
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&canonical)?;
    let m = file.metadata()?;
    if (!m.is_dir() && !m.is_file())
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o022 != 0
    {
        return Err(diagnostic("unsafe_path", canonical.display()));
    }
    blobs::check_acl(&canonical)?;
    let x = statx(&file)?;
    let volume_id = volume(&file, &canonical, &x)?;
    let after = fs::symlink_metadata(&canonical)?;
    let again = statx(&file)?;
    if after.file_type().is_symlink()
        || (after.dev(), after.ino()) != (m.dev(), m.ino())
        || again.stx_mnt_id != x.stx_mnt_id
        || again.stx_btime.tv_sec != x.stx_btime.tv_sec
        || again.stx_btime.tv_nsec != x.stx_btime.tv_nsec
    {
        return Err(unprovable("identity changed during observation"));
    }
    Ok(Object {
        path: canonical
            .to_str()
            .ok_or_else(|| unprovable("non-UTF-8 identity"))?
            .to_owned(),
        device: m.dev().to_string(),
        inode: m.ino().to_string(),
        volume_id,
        birth_seconds: x.stx_btime.tv_sec.to_string(),
        birth_nanoseconds: x.stx_btime.tv_nsec.to_string(),
        nonce,
    })
}
pub(super) fn check_ancestor(file: &File, path: &Path) -> Result<(), RsStoreError> {
    volume(file, path, &statx(file)?)?;
    Ok(())
}
pub(crate) fn rename_exclusive(from: &Path, to: &Path) -> Result<(), RsStoreError> {
    let a = CString::new(from.as_os_str().as_bytes()).map_err(corrupt)?;
    let b = CString::new(to.as_os_str().as_bytes()).map_err(corrupt)?;
    if unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            a.as_ptr(),
            libc::AT_FDCWD,
            b.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } != 0
    {
        let e = std::io::Error::last_os_error();
        return Err(
            if matches!(
                e.raw_os_error(),
                Some(libc::ENOSYS | libc::EINVAL | libc::EOPNOTSUPP)
            ) {
                diagnostic("coordination_unavailable", e)
            } else {
                io_error(e)
            },
        );
    }
    Ok(())
}
pub(super) fn write_attribute(
    path: &Path,
    name: &str,
    expected: Option<&[u8]>,
    bytes: &[u8],
) -> Result<(), RsStoreError> {
    let before = observe(path, None)?;
    let old = attribute(path, name)?;
    if let Some(expected) = expected {
        if old.as_deref() != Some(expected) {
            return Err(unprovable("attribute changed before replacement"));
        }
    } else if let Some(old) = old {
        if old != bytes {
            return Err(unprovable("existing attribute differs"));
        }
        return Ok(());
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    if (
        file.metadata()?.dev().to_string(),
        file.metadata()?.ino().to_string(),
    ) != (before.device.clone(), before.inode.clone())
    {
        return Err(unprovable("attribute descriptor differs"));
    }
    let native =
        CString::new(name.replacen("org.xdesign.", "user.xdesign.", 1)).map_err(corrupt)?;
    let flag = if expected.is_some() {
        libc::XATTR_REPLACE
    } else {
        libc::XATTR_CREATE
    };
    if unsafe {
        libc::fsetxattr(
            file.as_raw_fd(),
            native.as_ptr(),
            bytes.as_ptr().cast(),
            bytes.len(),
            flag,
        )
    } != 0
    {
        return Err(unprovable(std::io::Error::last_os_error()));
    }
    blobs::sync_durable(&file).map_err(|e| diagnostic("coordination_unavailable", e))?;
    if attribute(path, name)?.as_deref() != Some(bytes) || observe(path, None)? != before {
        return Err(unprovable("attribute readback or identity differs"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linux_live_capabilities_and_unsupported_filesystems() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cache/linux-profile")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&path).unwrap();
        let object = observe(&path, None).unwrap();
        assert!(object.volume_id.starts_with("02"));
        assert_eq!(object.volume_id.len(), 34);
        let lease = access::AccessLease::acquire(&[path.clone()]).unwrap();
        install_attribute(&path, "org.xdesign.profile-test", b"first").unwrap();
        assert_eq!(
            attribute(&path, "org.xdesign.profile-test")
                .unwrap()
                .unwrap(),
            b"first"
        );
        replace_attribute(&path, "org.xdesign.profile-test", b"first", b"second").unwrap();
        assert!(install_attribute(&path, "org.xdesign.profile-test", b"wrong").is_err());
        let from = path.join("from");
        let to = path.join("to");
        fs::write(&from, b"first").unwrap();
        rename_exclusive(&from, &to).unwrap();
        fs::write(&from, b"second").unwrap();
        assert!(rename_exclusive(&from, &to).is_err());
        assert_eq!(fs::read(&to).unwrap(), b"first");
        blobs::fsync_dir(&path).unwrap();
        drop(lease);
        // /proc is a real unsupported filesystem and this read creates nothing.
        assert!(observe(Path::new("/proc"), None).is_err());
    }
    #[test]
    fn default_acl_on_home_ancestor_refuses_before_namespace_creation() {
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cache/linux-acl-profile")
            .join(uuid::Uuid::new_v4().to_string());
        let home = parent.join("home");
        fs::create_dir_all(&home).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        let file = File::open(&parent).unwrap();
        let mut acl = 2u32.to_le_bytes().to_vec();
        for (tag, permissions, id) in [
            (1u16, 7u16, u32::MAX),
            (2, 7, 65534),
            (4, 0, u32::MAX),
            (16, 7, u32::MAX),
            (32, 0, u32::MAX),
        ] {
            acl.extend(tag.to_le_bytes());
            acl.extend(permissions.to_le_bytes());
            acl.extend(id.to_le_bytes());
        }
        assert_eq!(
            unsafe {
                libc::fsetxattr(
                    file.as_raw_fd(),
                    c"system.posix_acl_default".as_ptr(),
                    acl.as_ptr().cast(),
                    acl.len(),
                    libc::XATTR_CREATE,
                )
            },
            0
        );
        // Default ACL grants do not appear in the parent's POSIX mode bits.
        assert_eq!(file.metadata().unwrap().mode() & 0o022, 0);
        assert!(blobs::check_acl(&parent).is_err());
        assert!(ensure_namespace(&home).is_err());
        assert!(!home.join(".xdesign").exists());
    }

    #[test]
    fn mount_paths_decode_only_kernel_escapes() {
        assert_eq!(
            mount_path("/a\\040b\\134c").unwrap(),
            PathBuf::from("/a b\\c")
        );
        assert!(mount_path("/a\\000b").is_err());
        assert!(mount_path("relative").is_err());
    }
}
