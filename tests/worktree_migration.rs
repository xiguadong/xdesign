use std::{fs, path::PathBuf, process::Command};
#[test]
fn malformed_migration_and_false_quiescence_do_not_create_store() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("cache/implementation/worktree-migration-cli")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&p).unwrap();
    let req = p.join("request.json");
    fs::write(&req, b"{\"schema_version\":1,\"schema_version\":1}").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_xdesign"))
        .current_dir(&p)
        .args(["store", "migrate", "apply", "--request"])
        .arg(&req)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    let e: serde_json::Value = serde_json::from_slice(&o.stderr).unwrap();
    assert_eq!(e["error"]["code"], "invalid_input");
    assert!(!p.join(".xdesign").exists());
}
