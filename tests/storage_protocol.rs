//! Protocol-level regression tests for the storage review findings S6 and S9:
//!
//! - S6: an exact retry of a committed operation must return the stored
//!   terminal receipt even after the original project root and source paths
//!   are destroyed, while request/path-syntax validation still precedes
//!   reservation and a new operation with a broken project root never leaves a
//!   pending reservation behind.
//! - S9: source omission may reuse published bytes only when a committed blob
//!   row exists or the bytes are attributable to an earlier attempt of the
//!   same operation. An orphan published by an unrelated conflicting create
//!   must never be adopted; a crashed operation's own publication must still
//!   recover after its source disappears.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use xdesign::storage::{
    cli, RsDoctorFindingKind, RsOperationOutcome, RsSaveMaterial, RsSaveRequest, RsStore,
    RsStoreError,
};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

struct Fixture {
    base: PathBuf,
    project: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("cache/implementation/storage-protocol-tests")
            .join(uuid::Uuid::new_v4().to_string());
        let project = base.join("project");
        fs::create_dir_all(&project).unwrap();
        Self {
            root: base.join("store"),
            base,
            project,
        }
    }

    fn write(&self, uri: &str, bytes: &[u8]) -> PathBuf {
        let path = self.project.join(uri);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }

    fn material(&self, uri: &str, bytes: &[u8]) -> RsSaveMaterial {
        self.write(uri, bytes);
        RsSaveMaterial {
            uri: uri.into(),
            role: "artifact".into(),
            sha256: digest(bytes),
            source: Some(PathBuf::from(uri)),
        }
    }

    fn carried(&self, uri: &str, bytes: &[u8]) -> RsSaveMaterial {
        RsSaveMaterial {
            uri: uri.into(),
            role: "artifact".into(),
            sha256: digest(bytes),
            source: None,
        }
    }

    fn request(
        &self,
        operation: &str,
        item: &str,
        expected_revision: u64,
        materials: Vec<RsSaveMaterial>,
    ) -> RsSaveRequest {
        RsSaveRequest {
            schema_version: 1,
            operation_id: operation.into(),
            item_id: item.into(),
            kind: "artifact".into(),
            expected_revision,
            metadata: Map::new(),
            materials,
            result: json!({"operation": operation}),
        }
    }

    fn store(&self) -> RsStore {
        RsStore::init(&self.root).unwrap()
    }

    fn blob(&self, sha256: &str) -> PathBuf {
        self.root
            .join("blobs/sha256")
            .join(&sha256[..2])
            .join(&sha256[2..])
    }
}

fn set_mode(path: &Path, mode: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
}

fn dispatch_save(fixture: &Fixture, request_path: &Path) -> (Value, i32) {
    let args: Vec<String> = vec![
        "save".into(),
        "--request".into(),
        request_path.to_string_lossy().into_owned(),
        "--project-root".into(),
        fixture.project.to_string_lossy().into_owned(),
        "--root".into(),
        fixture.root.to_string_lossy().into_owned(),
    ];
    cli::dispatch(&args)
}

fn scalar(root: &Path, sql: &str) -> i64 {
    let connection = rusqlite::Connection::open(root.join("store.db")).unwrap();
    connection
        .query_row(sql, [], |row| row.get::<_, i64>(0))
        .unwrap()
}

/// S6: a committed exact retry must replay the stored receipt from the store
/// alone, before the transient project tree is consulted at all.
#[test]
fn committed_exact_retry_survives_source_and_project_root_destruction() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let evidence = b"review S6 evidence bytes";
    let request = fixture.request(
        "retry-operation",
        "retry-item",
        0,
        vec![fixture.material("evidence/retry.txt", evidence)],
    );
    let raw = serde_json::to_vec_pretty(&request).unwrap();
    let request_path = fixture.base.join("request.json");
    fs::write(&request_path, &raw).unwrap();
    let first = store.save_bytes(&fixture.project, &raw).unwrap();
    assert_eq!(first.outcome, RsOperationOutcome::Committed);
    assert_eq!(first.revision, Some(1));
    let head_before = store.get("retry-item", None).unwrap();

    // The baseline failure mode: the source parent is replaced by a regular
    // file, so resolving the source fails with ENOTDIR after the commit.
    fs::remove_dir_all(fixture.project.join("evidence")).unwrap();
    fs::write(
        fixture.project.join("evidence"),
        b"a file where a directory used to be",
    )
    .unwrap();
    assert_eq!(store.save_bytes(&fixture.project, &raw).unwrap(), first);

    // The other baseline failure mode: the project root itself is gone.
    fs::remove_dir_all(&fixture.project).unwrap();
    assert!(!fixture.project.exists());
    assert_eq!(store.save_bytes(&fixture.project, &raw).unwrap(), first);
    assert_eq!(
        store
            .save_bytes(&fixture.base.join("never-existed"), &raw)
            .unwrap(),
        first
    );

    // The CLI mirrors the same contract: exit 0 with the committed receipt.
    let (response, code) = dispatch_save(&fixture, &request_path);
    assert_eq!(code, 0, "{response}");
    assert_eq!(
        response["data"]["receipt"],
        serde_json::to_value(&first).unwrap()
    );

    // Different exact bytes on a reserved operation ID are still a mismatch,
    // even when the project root is missing.
    let mut changed = request.clone();
    changed.result = json!({"operation": "different request bytes"});
    let changed_raw = serde_json::to_vec_pretty(&changed).unwrap();
    assert!(matches!(
        store.save_bytes(&fixture.project, &changed_raw),
        Err(RsStoreError::OperationMismatch { operation_id })
            if operation_id == "retry-operation"
    ));

    // Request/path syntax validation still precedes reservation: an invalid
    // project-relative source is refused without recording an operation.
    let mut syntax = fixture.request("syntax-operation", "syntax-item", 0, vec![]);
    syntax.materials.push(RsSaveMaterial {
        uri: "syntax.txt".into(),
        role: "artifact".into(),
        sha256: digest(b"x"),
        source: Some(PathBuf::from("../escape")),
    });
    assert!(matches!(
        store.save_bytes(
            &fixture.project,
            &serde_json::to_vec_pretty(&syntax).unwrap()
        ),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(matches!(
        store.outcome("syntax-operation"),
        Err(RsStoreError::NotFound(_))
    ));

    // A genuinely new operation with an invalid project root still errors and
    // must not leave a pending reservation behind.
    let fresh = fixture.request(
        "fresh-operation",
        "fresh-item",
        0,
        vec![RsSaveMaterial {
            uri: "fresh.txt".into(),
            role: "artifact".into(),
            sha256: digest(b"fresh"),
            source: Some(PathBuf::from("fresh.txt")),
        }],
    );
    assert!(matches!(
        store.save_bytes(
            &fixture.project,
            &serde_json::to_vec_pretty(&fresh).unwrap()
        ),
        Err(RsStoreError::NotFound(_) | RsStoreError::UnsafePath(_))
    ));
    assert!(matches!(
        store.outcome("fresh-operation"),
        Err(RsStoreError::NotFound(_))
    ));

    // outcome/get/read remain mutually consistent; no new revision appeared.
    assert_eq!(store.outcome("retry-operation").unwrap(), first);
    assert_eq!(store.get("retry-item", None).unwrap(), head_before);
    assert_eq!(store.history("retry-item").unwrap().len(), 1);
    assert_eq!(
        store
            .read_material("retry-item", Some(1), "evidence/retry.txt")
            .unwrap(),
        evidence
    );
}

/// S6 note from the review: the typed `save` path must replay too, which means
/// the early inspection has to compare the compact bytes that typed saves
/// record in the first place.
#[test]
fn typed_save_replays_after_project_root_destruction() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let bytes = b"typed save source bytes";
    let request = fixture.request(
        "typed-operation",
        "typed-item",
        0,
        vec![fixture.material("typed.txt", bytes)],
    );
    let first = store.save(&fixture.project, &request).unwrap();
    assert_eq!(first.outcome, RsOperationOutcome::Committed);
    assert_eq!(first.revision, Some(1));

    fs::remove_dir_all(&fixture.project).unwrap();
    assert_eq!(store.save(&fixture.project, &request).unwrap(), first);
    assert_eq!(store.history("typed-item").unwrap().len(), 1);
    assert_eq!(
        store
            .read_material("typed-item", Some(1), "typed.txt")
            .unwrap(),
        bytes
    );
}

/// S9: a conflicting create can publish a blob that never earns a committed
/// blob row. Source omission by an unrelated operation must not adopt it.
#[test]
fn unrelated_orphan_cannot_be_adopted_without_a_source() {
    let fixture = Fixture::new();
    let store = fixture.store();
    store
        .save(
            &fixture.project,
            &fixture.request("winner", "guarded", 0, vec![]),
        )
        .unwrap();
    let orphan = b"orphan bytes published by a losing create";
    let loser = fixture.request(
        "losing-create",
        "guarded",
        0,
        vec![fixture.material("evidence/orphan.txt", orphan)],
    );
    let conflict = store.save(&fixture.project, &loser).unwrap();
    assert_eq!(conflict.outcome, RsOperationOutcome::Conflict);
    assert_eq!(conflict.revision, None);
    assert_eq!(scalar(&fixture.root, "SELECT COUNT(*) FROM blobs"), 0);
    assert_eq!(scalar(&fixture.root, "SELECT COUNT(*) FROM items"), 1);

    // Only the unpublished blob file survives; the source is disposable.
    fs::remove_file(fixture.project.join("evidence/orphan.txt")).unwrap();
    assert!(fixture.blob(&digest(orphan)).is_file());
    let report = store.doctor(true).unwrap();
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.kind == RsDoctorFindingKind::OrphanBlob),
        "{report:?}"
    );

    // A different operation/item carrying the same digest without a source
    // must be refused instead of committing the orphan.
    let adopt = fixture.request(
        "adopt-orphan",
        "new-item",
        0,
        vec![fixture.carried("evidence", orphan)],
    );
    let error = store.save(&fixture.project, &adopt).unwrap_err();
    assert!(
        matches!(
            error,
            RsStoreError::NotFound(_) | RsStoreError::InvalidInput(_)
        ),
        "{error:?}"
    );
    assert!(
        matches!(store.get("new-item", None), Err(RsStoreError::NotFound(_))),
        "the unrelated orphan must not be committed to a new item"
    );
    assert_eq!(
        scalar(
            &fixture.root,
            "SELECT COUNT(*) FROM items WHERE item_id = 'new-item'"
        ),
        0
    );
    assert_eq!(
        scalar(
            &fixture.root,
            "SELECT COUNT(*) FROM revisions WHERE item_id = 'new-item'"
        ),
        0
    );
    assert_eq!(scalar(&fixture.root, "SELECT COUNT(*) FROM blobs"), 0);
    let failure = store.outcome("adopt-orphan").unwrap();
    assert_ne!(failure.outcome, RsOperationOutcome::Committed);
    assert!(store
        .doctor(false)
        .unwrap()
        .findings
        .iter()
        .any(|finding| finding.kind == RsDoctorFindingKind::OrphanBlob));
}

/// Construct the exact on-disk state of a process that crashed after
/// `publish_blobs` returned but before `finalize_operation` committed: a
/// pending operation row, its pending blob reservations, an attempt-staged
/// digest file, and the hard-linked published blob. This is the same layout
/// `blobs::publish_blobs` produces (staging/<operation_id>/<attempt>/<digest>),
/// which is the attribution evidence used for recovery.
fn simulate_crash_after_publish(
    fixture: &Fixture,
    request: &RsSaveRequest,
    raw: &[u8],
    bytes: &[u8],
) {
    let result_json = String::from_utf8(serde_json::to_vec(&request.result).unwrap()).unwrap();
    let connection = rusqlite::Connection::open(fixture.root.join("store.db")).unwrap();
    connection
        .execute(
            "INSERT INTO operations
             (operation_id, request_bytes, request_sha256, outcome, item_id,
              expected_revision, revision, result_json)
             VALUES (?1, ?2, ?3, 'pending', ?4, ?5, NULL, ?6)",
            rusqlite::params![
                request.operation_id,
                raw,
                digest(raw),
                request.item_id,
                request.expected_revision as i64,
                result_json,
            ],
        )
        .unwrap();
    for material in &request.materials {
        connection
            .execute(
                "INSERT INTO pending_blobs (operation_id, sha256) VALUES (?1, ?2)",
                rusqlite::params![request.operation_id, material.sha256],
            )
            .unwrap();
    }
    drop(connection);

    let sha256 = request.materials[0].sha256.clone();
    assert_eq!(digest(bytes), sha256);
    let operation_directory = fixture.root.join("staging").join(&request.operation_id);
    let attempt_directory = operation_directory.join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&attempt_directory).unwrap();
    set_mode(&operation_directory, 0o700);
    set_mode(&attempt_directory, 0o700);
    let staged = attempt_directory.join(&sha256);
    fs::write(&staged, bytes).unwrap();
    set_mode(&staged, 0o400);
    let blob = fixture.blob(&sha256);
    fs::create_dir_all(blob.parent().unwrap()).unwrap();
    set_mode(blob.parent().unwrap(), 0o700);
    fs::hard_link(&staged, &blob).unwrap();
}

/// S9 converse: a pending operation whose own attempt published blobs keeps
/// recovering from its own bytes after the source disappears, while nobody
/// else may adopt them; once committed, the digest becomes a normal carried
/// digest for other operations.
#[test]
fn pending_own_publication_recovers_after_source_loss() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let bytes = b"own publication survives its source";
    let request = fixture.request(
        "pending-own",
        "pending-item",
        0,
        vec![fixture.material("own.txt", bytes)],
    );
    let raw = serde_json::to_vec_pretty(&request).unwrap();
    simulate_crash_after_publish(&fixture, &request, &raw, bytes);
    fs::remove_file(fixture.project.join("own.txt")).unwrap();

    // Doctor attributes the pending attempt and its staging residue.
    let report = store.doctor(false).unwrap();
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.kind == RsDoctorFindingKind::PendingOperation));
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.kind == RsDoctorFindingKind::StagingArtifact));

    // The staged bytes are not attributable to another operation.
    let pilfer = fixture.request(
        "pilfer",
        "pilfer-item",
        0,
        vec![fixture.carried("evidence", bytes)],
    );
    assert!(matches!(
        store.save(&fixture.project, &pilfer),
        Err(RsStoreError::NotFound(_) | RsStoreError::InvalidInput(_))
    ));
    assert!(matches!(
        store.get("pilfer-item", None),
        Err(RsStoreError::NotFound(_))
    ));

    // The original operation still recovers from its own publication.
    let receipt = store.save_bytes(&fixture.project, &raw).unwrap();
    assert_eq!(receipt.outcome, RsOperationOutcome::Committed);
    assert_eq!(receipt.revision, Some(1));
    assert_eq!(
        store
            .read_material("pending-item", Some(1), "own.txt")
            .unwrap(),
        bytes
    );
    assert!(
        !fixture.root.join("staging/pending-own").exists(),
        "a terminal receipt must release the recovery staging"
    );

    // A committed blob row now authorizes source-free carried digests.
    let carried = fixture.request(
        "carried-after-commit",
        "carried-item",
        0,
        vec![fixture.carried("carried.txt", bytes)],
    );
    let receipt = store.save(&fixture.project, &carried).unwrap();
    assert_eq!(receipt.outcome, RsOperationOutcome::Committed);
    assert_eq!(
        store
            .read_material("carried-item", Some(1), "carried.txt")
            .unwrap(),
        bytes
    );
}
