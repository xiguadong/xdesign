use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
};
use xdesign::storage::{
    cli::{with_project_store, WithProjectStoreError},
    RsStore, RsStoreError,
};
fn fixture() -> PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cache/implementation/worktree-access")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&p).unwrap();
    p
}
#[test]
fn command_bridge_guards_held_and_fresh_handles_and_unwind() {
    let p = fixture();
    let root = p.join("store");
    let held = RsStore::init(&root).unwrap();
    with_project_store(&p, &root, false, |store| -> Result<(), String> {
        assert!(store.list(None, None).unwrap().is_empty());
        assert!(store.doctor(true).unwrap().findings.is_empty());
        assert!(RsStore::open(&root)
            .err()
            .unwrap()
            .to_string()
            .starts_with("location_reason=migration_busy;"));
        assert!(held
            .list(None, None)
            .unwrap_err()
            .to_string()
            .starts_with("location_reason=migration_busy;"));
        Ok(())
    })
    .unwrap();
    let error = with_project_store(&p, &root, false, |_| -> Result<(), String> {
        Err("unchanged callback error".into())
    })
    .unwrap_err();
    assert!(matches!(error,WithProjectStoreError::Action(s) if s=="unchanged callback error"));
    let _ = std::panic::catch_unwind(|| {
        with_project_store(&p, &root, false, |_| -> Result<(), String> {
            panic!("release lease on unwind")
        })
    });
    assert!(held.list(None, None).unwrap().is_empty());
}
#[test]
fn different_thread_cannot_reenter_directory_lock() {
    let p = fixture();
    let root = p.join("store");
    RsStore::init(&root).unwrap();
    let (tx, rx) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let peer = root.clone();
    let project = p.clone();
    let thread = std::thread::spawn(move || {
        with_project_store(&project, &peer, false, |_| -> Result<(), String> {
            tx.send(()).unwrap();
            wait.recv().unwrap();
            Ok(())
        })
        .unwrap()
    });
    rx.recv().unwrap();
    assert!(RsStore::open(&root)
        .err()
        .unwrap()
        .to_string()
        .contains("location_reason=migration_busy;"));
    release.send(()).unwrap();
    thread.join().unwrap();
    RsStore::open(&root).unwrap();
}
#[test]
fn retained_handle_does_not_retarget_replaced_root() {
    let p = fixture();
    let root = p.join("store");
    let h = RsStore::init(&root).unwrap();
    fs::rename(&root, p.join("previous")).unwrap();
    RsStore::init(&root).unwrap();
    assert!(matches!(
        h.list(None, None),
        Err(RsStoreError::Integrity(_))
    ));
}
#[test]
fn parent_alias_joins_same_lock_and_final_symlink_refuses() {
    use std::os::unix::fs::symlink;
    let p = fixture();
    let parent = p.join("project");
    fs::create_dir(&parent).unwrap();
    let root = parent.join("store");
    RsStore::init(&root).unwrap();
    symlink(&parent, p.join("alias")).unwrap();
    let through_alias = RsStore::open(&p.join("alias/store")).unwrap();
    assert!(through_alias.list(None, None).unwrap().is_empty());
    symlink(&root, p.join("store-link")).unwrap();
    assert!(matches!(
        RsStore::open(&p.join("store-link")),
        Err(RsStoreError::UnsafePath(_))
    ));
}

#[test]
fn backend_candidate_reads_use_only_the_complete_command_lease() {
    use xdesign::storage::{cli::project_candidate_contains, RsSaveRequest};
    let p = fixture();
    let local = p.join(".xdesign");
    let explicit_parent = p.join("elsewhere");
    fs::create_dir(&explicit_parent).unwrap();
    let explicit = explicit_parent.join("store");
    let local_store = RsStore::init(&local).unwrap();
    RsStore::init(&explicit).unwrap();
    local_store
        .save(
            &p,
            &RsSaveRequest {
                schema_version: 1,
                operation_id: "candidate".into(),
                item_id: "same-id".into(),
                kind: "record".into(),
                expected_revision: 0,
                metadata: Default::default(),
                materials: vec![],
                result: serde_json::json!({}),
            },
        )
        .unwrap();
    with_project_store(&p, &explicit, false, |h| -> Result<(), RsStoreError> {
        assert!(project_candidate_contains(h, &p, "same-id")?);
        assert!(!project_candidate_contains(h, &p, "absent")?);
        Ok(())
    })
    .unwrap();
}
