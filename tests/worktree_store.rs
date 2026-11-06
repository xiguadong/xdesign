use std::{fs, path::PathBuf, process::Command};
use xdesign::storage::cli::resolve_store_root;
fn fixture() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("cache/implementation/worktree-selection")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&p).unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .arg(&p)
        .status()
        .unwrap()
        .success());
    p
}
#[test]
fn primary_project_keeps_selected_directory_and_explicit_roots() {
    let p = fixture();
    let child = p.join("nested");
    fs::create_dir(&child).unwrap();
    assert_eq!(
        resolve_store_root(&p, None, false).unwrap(),
        p.join(".xdesign")
    );
    assert_eq!(
        resolve_store_root(&child, None, false).unwrap(),
        child.join(".xdesign")
    );
    assert_eq!(
        resolve_store_root(&child, Some(std::path::Path::new("different")), false).unwrap(),
        child.join("different")
    );
    assert!(resolve_store_root(&p, Some(std::path::Path::new("")), false).is_err());
}
#[test]
fn explicit_root_does_not_require_valid_git_metadata() {
    let p = fixture();
    fs::remove_dir_all(p.join(".git")).unwrap();
    fs::write(p.join(".git"), b"broken git link").unwrap();
    assert_eq!(
        resolve_store_root(&p, Some(std::path::Path::new("known")), false).unwrap(),
        p.join("known")
    );
    assert!(resolve_store_root(&p, None, false)
        .unwrap_err()
        .to_string()
        .contains("identity_unprovable"));
}
