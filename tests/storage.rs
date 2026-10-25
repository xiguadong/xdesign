use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use xdesign::storage::{
    cli, RsOperationOutcome, RsSaveMaterial, RsSaveRequest, RsStore, RsStoreError,
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
            .join("cache/implementation/storage-tests")
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
            metadata: json!({"z": {"b": 2, "a": 1}, "a": operation})
                .as_object()
                .cloned()
                .unwrap(),
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

#[cfg(unix)]
fn mode(path: impl AsRef<Path>, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[test]
fn init_reopen_and_foreign_formats() {
    let fixture = Fixture::new();
    assert!(matches!(
        RsStore::open(&fixture.root),
        Err(RsStoreError::NotFound(_))
    ));
    let store = fixture.store();
    let marker: Value =
        serde_json::from_slice(&fs::read(fixture.root.join("format.json")).unwrap()).unwrap();
    let store_id = marker["store_id"].clone();
    drop(store);
    let reopened = RsStore::open(&fixture.root).unwrap();
    drop(reopened);
    let initialized = RsStore::init(&fixture.root).unwrap();
    drop(initialized);
    let second: Value =
        serde_json::from_slice(&fs::read(fixture.root.join("format.json")).unwrap()).unwrap();
    assert_eq!(second["store_id"], store_id);

    let foreign = fixture.base.join("foreign");
    fs::create_dir(&foreign).unwrap();
    #[cfg(unix)]
    mode(&foreign, 0o700);
    fs::write(foreign.join("operator.txt"), b"do not adopt").unwrap();
    assert!(matches!(
        RsStore::init(&foreign),
        Err(RsStoreError::UnsupportedFormat(_))
    ));
    assert_eq!(
        fs::read(foreign.join("operator.txt")).unwrap(),
        b"do not adopt"
    );

    let newer = fixture.base.join("newer");
    fs::create_dir(&newer).unwrap();
    #[cfg(unix)]
    mode(&newer, 0o700);
    let marker = json!({"format":"xdesign.store", "schema_version":2, "store_id":"newer"});
    let marker_path = newer.join("format.json");
    fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
    #[cfg(unix)]
    mode(&marker_path, 0o600);
    assert!(RsStore::open(&newer).is_err());
}

#[test]
fn truncated_format_is_refused_without_adoption() {
    let fixture = Fixture::new();
    let truncated = fixture.base.join("truncated");
    fs::create_dir(&truncated).unwrap();
    #[cfg(unix)]
    mode(&truncated, 0o700);
    let marker_path = truncated.join("format.json");
    let bytes = b"{\"format\":\"xdesign.store\",\"schema_version\":1,\"store_id\":";
    fs::write(&marker_path, bytes).unwrap();
    #[cfg(unix)]
    mode(&marker_path, 0o600);
    assert!(matches!(
        RsStore::open(&truncated),
        Err(RsStoreError::UnsupportedFormat(_))
    ));
    assert!(matches!(
        RsStore::init(&truncated),
        Err(RsStoreError::UnsupportedFormat(_))
    ));
    assert_eq!(fs::read(&marker_path).unwrap().as_slice(), bytes);
    assert!(!truncated.join("store.db").exists());
}

#[test]
fn revisions_queries_carry_and_dedup() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let a = b"alpha";
    let b = b"shared bytes";
    let create = fixture.request(
        "create-1",
        "item-1",
        0,
        vec![fixture.material("a.txt", a), fixture.material("b.txt", b)],
    );
    let first = store.save(&fixture.project, &create).unwrap();
    assert_eq!(first.outcome, RsOperationOutcome::Committed);
    assert_eq!(first.revision, Some(1));
    let raw = cli::request_bytes(&store, "create-1").unwrap().unwrap();
    let decoded: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(raw, serde_json::to_vec(&decoded).unwrap());

    let c = b"gamma";
    let append = fixture.request(
        "append-1",
        "item-1",
        1,
        vec![fixture.carried("a.txt", a), fixture.material("c.txt", c)],
    );
    assert_eq!(
        store.save(&fixture.project, &append).unwrap().revision,
        Some(2)
    );
    let second = fixture.request(
        "create-2",
        "item-2",
        0,
        vec![fixture.material("copy.txt", b)],
    );
    store.save(&fixture.project, &second).unwrap();

    let old = store.get("item-1", Some(1)).unwrap();
    assert_eq!(old.materials.len(), 2);
    assert_eq!(old.metadata["a"], "create-1");
    assert!(old.projection.is_none());
    let head = store.get("item-1", None).unwrap();
    assert_eq!(head.revision, 2);
    assert_eq!(
        head.materials
            .iter()
            .map(|m| m.uri.as_str())
            .collect::<Vec<_>>(),
        ["a.txt", "c.txt"]
    );
    assert_eq!(store.history("item-1").unwrap().len(), 2);
    assert_eq!(store.read_material("item-1", Some(1), "b.txt").unwrap(), b);
    assert_eq!(store.read_material("item-1", None, "a.txt").unwrap(), a);
    assert!(matches!(
        store.read_material("item-1", None, "b.txt"),
        Err(RsStoreError::NotFound(_))
    ));
    let list = store.list(Some("artifact"), None).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].item_id, "item-1");
    assert_eq!(list[0].head_revision, 2);
    assert!(store.list(None, Some("active")).unwrap().is_empty());
    assert!(store.doctor(true).unwrap().findings.is_empty());
}

#[test]
fn exact_retry_mismatch_and_conflicts() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "raw-operation",
        "raw-item",
        0,
        vec![fixture.material("raw.txt", b"raw")],
    );
    let raw = serde_json::to_vec_pretty(&request).unwrap();
    let first = store.save_bytes(&fixture.project, &raw).unwrap();
    drop(store);
    fs::remove_file(fixture.project.join("raw.txt")).unwrap();
    let store = RsStore::open(&fixture.root).unwrap();
    assert_eq!(store.save_bytes(&fixture.project, &raw).unwrap(), first);
    assert_eq!(store.history("raw-item").unwrap().len(), 1);
    let marker_before = fs::read(fixture.root.join("format.json")).unwrap();
    drop(RsStore::init(&fixture.root).unwrap());
    assert_eq!(
        fs::read(fixture.root.join("format.json")).unwrap(),
        marker_before
    );
    assert_eq!(store.history("raw-item").unwrap().len(), 1);
    assert_eq!(
        cli::request_bytes(&store, "raw-operation").unwrap(),
        Some(raw.clone())
    );
    let compact = serde_json::to_vec(&request).unwrap();
    assert!(matches!(
        store.save_bytes(&fixture.project, &compact),
        Err(RsStoreError::OperationMismatch { .. })
    ));
    assert_eq!(cli::request_bytes(&store, "unknown").unwrap(), None);

    let winner = fixture.request("winner", "guarded", 0, vec![]);
    store.save(&fixture.project, &winner).unwrap();
    let create_conflict = fixture.request("create-conflict", "guarded", 0, vec![]);
    let conflict = store.save(&fixture.project, &create_conflict).unwrap();
    assert_eq!(conflict.outcome, RsOperationOutcome::Conflict);
    assert_eq!(conflict.revision, None);
    assert_eq!(store.get("guarded", None).unwrap().revision, 1);
    assert_eq!(store.outcome("create-conflict").unwrap(), conflict);

    let append = fixture.request("append-winner", "guarded", 1, vec![]);
    store.save(&fixture.project, &append).unwrap();
    let stale = fixture.request("append-stale", "guarded", 1, vec![]);
    assert_eq!(
        store.save(&fixture.project, &stale).unwrap().outcome,
        RsOperationOutcome::Conflict
    );
    assert_eq!(store.get("guarded", None).unwrap().revision, 2);
    let mut wrong_kind = fixture.request("kind-conflict", "guarded", 2, vec![]);
    wrong_kind.kind = "other".into();
    assert_eq!(
        store.save(&fixture.project, &wrong_kind).unwrap().outcome,
        RsOperationOutcome::Conflict
    );
    assert_eq!(store.get("guarded", None).unwrap().revision, 2);
}

fn minimal_raw(operation: &str, metadata: &str, result: &str) -> Vec<u8> {
    format!(
        "{{\"schema_version\":1,\"operation_id\":\"{operation}\",\"item_id\":\"strict-item\",\"kind\":\"artifact\",\"expected_revision\":0,\"metadata\":{metadata},\"materials\":[],\"result\":{result}}}"
    )
    .into_bytes()
}

#[test]
fn strict_json_and_path_validation() {
    let fixture = Fixture::new();
    let store = fixture.store();
    for (operation, raw) in [
        (
            "duplicate-metadata",
            minimal_raw("duplicate-metadata", "{\"x\":1,\"x\":2}", "null"),
        ),
        (
            "duplicate-result",
            minimal_raw("duplicate-result", "{}", "{\"x\":1,\"x\":2}"),
        ),
    ] {
        assert!(matches!(
            store.save_bytes(&fixture.project, &raw),
            Err(RsStoreError::InvalidInput(_))
        ));
        assert!(matches!(
            store.outcome(operation),
            Err(RsStoreError::NotFound(_))
        ));
        let _ = raw;
    }
    let unknown = minimal_raw("unknown-field", "{}", "null");
    let mut unknown: Value = serde_json::from_slice(&unknown).unwrap();
    unknown["unexpected"] = json!(true);
    assert!(store
        .save_bytes(&fixture.project, &serde_json::to_vec(&unknown).unwrap())
        .is_err());
    let null_source = fixture.request(
        "null-source",
        "null-source",
        0,
        vec![fixture.material("x", b"x")],
    );
    let mut null_source = serde_json::to_value(null_source).unwrap();
    null_source["materials"][0]["source"] = Value::Null;
    assert!(matches!(
        store.save_bytes(&fixture.project, &serde_json::to_vec(&null_source).unwrap()),
        Err(RsStoreError::InvalidInput(_))
    ));
    let mut deep = fixture.request("deep", "deep", 0, vec![]);
    deep.result = (0..65).fold(Value::Null, |value, _| json!([value]));
    assert!(store.save(&fixture.project, &deep).is_err());
    let unsupported = minimal_raw("unsupported", "{}", "null");
    let mut unsupported: Value = serde_json::from_slice(&unsupported).unwrap();
    unsupported["schema_version"] = json!(2);
    assert!(matches!(
        store.save_bytes(&fixture.project, &serde_json::to_vec(&unsupported).unwrap()),
        Err(RsStoreError::UnsupportedFormat(_))
    ));
    let trailing = [minimal_raw("trailing", "{}", "null"), b" x".to_vec()].concat();
    assert!(store.save_bytes(&fixture.project, &trailing).is_err());

    for (index, uri) in [
        "../escape",
        "/absolute",
        "a\\b",
        "a\0b",
        "a?b",
        "scheme:value",
        "a//b",
    ]
    .into_iter()
    .enumerate()
    {
        let request = fixture.request(
            &format!("bad-uri-{index}"),
            &format!("bad-item-{index}"),
            0,
            vec![RsSaveMaterial {
                uri: uri.into(),
                role: "artifact".into(),
                sha256: digest(b"x"),
                source: None,
            }],
        );
        assert!(store.save(&fixture.project, &request).is_err());
    }
    let mut traversal = fixture.request("traversal", "traversal", 0, vec![]);
    traversal.materials.push(RsSaveMaterial {
        uri: "safe".into(),
        role: "artifact".into(),
        sha256: digest(b"x"),
        source: Some(PathBuf::from("../escape")),
    });
    assert!(matches!(
        store.save(&fixture.project, &traversal),
        Err(RsStoreError::UnsafePath(_))
    ));

    let mismatch = fixture.request(
        "mismatch",
        "mismatch",
        0,
        vec![RsSaveMaterial {
            uri: "mismatch.txt".into(),
            role: "artifact".into(),
            sha256: digest(b"expected"),
            source: Some(PathBuf::from("actual.txt")),
        }],
    );
    fixture.write("actual.txt", b"actual");
    assert!(matches!(
        store.save(&fixture.project, &mismatch),
        Err(RsStoreError::InvalidInput(_))
    ));
    assert_eq!(
        store.outcome("mismatch").unwrap().outcome,
        RsOperationOutcome::Pending
    );
    assert!(matches!(
        store.get("mismatch", None),
        Err(RsStoreError::NotFound(_))
    ));

    // Numeric-looking text inside a JSON string is data, not a numeric token:
    // it must save and echo exactly even when the same text as a number would
    // be rejected as storage-unstable.
    let mut text_request = fixture.request("numeric-string", "numeric-string-item", 0, vec![]);
    text_request.metadata.insert(
        "text".into(),
        json!("9764.519195169985 1.0468000232875951e-07 0.0e-2147483648"),
    );
    let expected_text = text_request.metadata.get("text").unwrap().clone();
    store.save(&fixture.project, &text_request).unwrap();
    let echoed = store.get("numeric-string-item", None).unwrap();
    assert_eq!(echoed.metadata.get("text"), Some(&expected_text));

    // Malformed numeric syntax is a typed invalid input with no operation
    // receipt, never a panic.
    for (index, token) in ["01", "+1", ".5", "1.", "1e", "1e+", "--1", "1..2"]
        .into_iter()
        .enumerate()
    {
        let operation = format!("bad-number-{index}");
        let raw = minimal_raw(&operation, &format!("{{\"n\":{token}}}"), "null");
        let error = store.save_bytes(&fixture.project, &raw).unwrap_err();
        assert!(
            matches!(&error, RsStoreError::InvalidInput(_)),
            "token {token}: {error}"
        );
        assert!(matches!(
            store.outcome(&operation),
            Err(RsStoreError::NotFound(_))
        ));
    }
}

#[cfg(unix)]
#[test]
fn symlink_sources_are_refused() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let store = fixture.store();
    fixture.write("actual.txt", b"actual");
    symlink("actual.txt", fixture.project.join("link.txt")).unwrap();
    let request = fixture.request(
        "symlink-source",
        "symlink-source",
        0,
        vec![RsSaveMaterial {
            uri: "safe.txt".into(),
            role: "artifact".into(),
            sha256: digest(b"actual"),
            source: Some(PathBuf::from("link.txt")),
        }],
    );
    assert!(matches!(
        store.save(&fixture.project, &request),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(matches!(
        store.get("symlink-source", None),
        Err(RsStoreError::NotFound(_))
    ));
}

#[test]
fn verified_read_checkout_and_doctor() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "checkout-save",
        "checkout-item",
        0,
        vec![
            fixture.material("tree/a.txt", b"aaa"),
            fixture.material("tree/b.txt", b"bbb"),
        ],
    );
    store.save(&fixture.project, &request).unwrap();
    let destination = fixture.base.join("checkout");
    let report = store.checkout("checkout-item", None, &destination).unwrap();
    assert_eq!(report.material_count, 2);
    assert_eq!(fs::read(destination.join("tree/a.txt")).unwrap(), b"aaa");
    assert!(store.checkout("checkout-item", None, &destination).is_err());
    assert!(store.doctor(true).unwrap().findings.is_empty());

    let orphan = b"orphan";
    let orphan_digest = digest(orphan);
    let orphan_path = fixture.blob(&orphan_digest);
    fs::create_dir_all(orphan_path.parent().unwrap()).unwrap();
    #[cfg(unix)]
    mode(orphan_path.parent().unwrap(), 0o700);
    fs::write(&orphan_path, orphan).unwrap();
    #[cfg(unix)]
    mode(&orphan_path, 0o400);
    assert!(store
        .doctor(false)
        .unwrap()
        .findings
        .iter()
        .any(|finding| { finding.kind == xdesign::storage::RsDoctorFindingKind::OrphanBlob }));

    let altered = fixture.blob(&digest(b"aaa"));
    #[cfg(unix)]
    mode(&altered, 0o600);
    fs::write(&altered, b"zzz").unwrap();
    #[cfg(unix)]
    mode(&altered, 0o400);
    assert!(store
        .read_material("checkout-item", None, "tree/a.txt")
        .is_err());
    let rejected_destination = fixture.base.join("rejected-checkout");
    assert!(store
        .checkout("checkout-item", None, &rejected_destination)
        .is_err());
    assert!(!rejected_destination.exists());
    assert!(store
        .doctor(true)
        .unwrap()
        .findings
        .iter()
        .any(|finding| { finding.kind == xdesign::storage::RsDoctorFindingKind::AlteredBlob }));

    let missing = fixture.blob(&digest(b"bbb"));
    fs::remove_file(missing).unwrap();
    assert!(store
        .doctor(false)
        .unwrap()
        .findings
        .iter()
        .any(|finding| { finding.kind == xdesign::storage::RsDoctorFindingKind::MissingBlob }));
}

#[test]
fn doctor_reports_pending_and_staging() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "pending-op",
        "pending-item",
        0,
        vec![RsSaveMaterial {
            uri: "pending.txt".into(),
            role: "artifact".into(),
            sha256: digest(b"expected"),
            source: Some(PathBuf::from("pending-source")),
        }],
    );
    fixture.write("pending-source", b"actual");
    assert!(store.save(&fixture.project, &request).is_err());
    let staging = fixture.root.join("staging/pending-op");
    fs::create_dir_all(&staging).unwrap();
    #[cfg(unix)]
    mode(&staging, 0o700);
    fs::write(staging.join("residue"), b"residue").unwrap();
    #[cfg(unix)]
    mode(staging.join("residue"), 0o600);
    let report = store.doctor(true).unwrap();
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.kind == xdesign::storage::RsDoctorFindingKind::PendingOperation));
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.kind == xdesign::storage::RsDoctorFindingKind::StagingArtifact));
}

#[test]
fn root_and_database_replacement_are_refused() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let moved = fixture.base.join("moved-store");
    fs::rename(&fixture.root, &moved).unwrap();
    assert!(store.list(None, None).is_err());
    assert!(store.doctor(false).unwrap().findings.iter().any(|finding| {
        finding.kind == xdesign::storage::RsDoctorFindingKind::IdentityMismatch
    }));
    drop(store);
    fs::rename(&moved, &fixture.root).unwrap();

    let store = RsStore::open(&fixture.root).unwrap();
    let backup = fixture.base.join("database-backup");
    fs::rename(fixture.root.join("store.db"), &backup).unwrap();
    fs::copy(&backup, fixture.root.join("store.db")).unwrap();
    #[cfg(unix)]
    mode(fixture.root.join("store.db"), 0o600);
    assert!(matches!(
        store.list(None, None),
        Err(RsStoreError::Integrity(_))
    ));
    assert!(store.doctor(false).unwrap().findings.iter().any(|finding| {
        finding.kind == xdesign::storage::RsDoctorFindingKind::IdentityMismatch
    }));
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn cli_envelopes_commands_and_degraded_doctor() {
    let fixture = Fixture::new();
    let root = fixture.root.to_string_lossy().into_owned();
    let (initialized, code) = cli::dispatch(&strings(&["store", "init", "--root", &root]));
    assert_eq!(code, 0, "{initialized}");
    assert_eq!(initialized["schema_version"], 1);
    assert_eq!(initialized["ok"], true);

    let request = fixture.request(
        "cli-save",
        "cli-item",
        0,
        vec![fixture.material("cli.txt", b"cli bytes")],
    );
    let request_path = fixture.base.join("request.json");
    fs::write(&request_path, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    let request_path = request_path.to_string_lossy().into_owned();
    let project = fixture.project.to_string_lossy().into_owned();
    let (saved, code) = cli::dispatch(&strings(&[
        "save",
        "--request",
        &request_path,
        "--project-root",
        &project,
        "--root",
        &root,
    ]));
    assert_eq!(code, 0, "{saved}");
    assert_eq!(saved["data"]["receipt"]["revision"], 1);
    let (outcome, code) = cli::dispatch(&strings(&["outcome", "cli-save", "--root", &root]));
    assert_eq!(code, 0);
    assert_eq!(outcome["data"]["receipt"]["outcome"], "committed");
    let (listed, code) = cli::dispatch(&strings(&["list", "--root", &root]));
    assert_eq!(code, 0);
    assert_eq!(listed["data"]["complete"], true);
    let (history, code) = cli::dispatch(&strings(&["history", "cli-item", "--root", &root]));
    assert_eq!(code, 0);
    assert_eq!(history["data"]["revisions"].as_array().unwrap().len(), 1);
    let (read, code) = cli::dispatch(&strings(&[
        "read", "cli-item", "--uri", "cli.txt", "--root", &root,
    ]));
    assert_eq!(code, 2, "{read}");
    assert_eq!(read["ok"], false);

    let output = fixture.base.join("read-output");
    let output_string = output.to_string_lossy().into_owned();
    let (read, code) = cli::dispatch(&strings(&[
        "read",
        "cli-item",
        "--uri",
        "cli.txt",
        "--output",
        &output_string,
        "--root",
        &root,
    ]));
    assert_eq!(code, 0, "{read}");
    assert_eq!(fs::read(output).unwrap(), b"cli bytes");
    let destination = fixture.base.join("cli-checkout");
    let destination_string = destination.to_string_lossy().into_owned();
    let (checkout, code) = cli::dispatch(&strings(&[
        "checkout",
        "cli-item",
        "--to",
        &destination_string,
        "--root",
        &root,
    ]));
    assert_eq!(code, 0, "{checkout}");
    assert_eq!(fs::read(destination.join("cli.txt")).unwrap(), b"cli bytes");
    let (doctor, code) = cli::dispatch(&strings(&["doctor", "--verify", "--root", &root]));
    assert_eq!(code, 0, "{doctor}");

    let conflict = fixture.request("cli-conflict", "cli-item", 0, vec![]);
    fs::write(&request_path, serde_json::to_vec(&conflict).unwrap()).unwrap();
    let (conflict, code) = cli::dispatch(&strings(&[
        "save",
        "--request",
        &request_path,
        "--project-root",
        &project,
        "--root",
        &root,
    ]));
    assert_eq!(code, 4, "{conflict}");
    assert_eq!(conflict["ok"], false);
    assert_eq!(conflict["error"]["code"], "revision_conflict");
    assert_eq!(conflict["data"]["receipt"]["outcome"], "conflict");
    let (invalid, code) = cli::dispatch(&strings(&["get", "cli-item", "--unknown"]));
    assert_eq!(code, 2);
    assert_eq!(invalid["error"]["code"], "invalid_input");

    let foreign = fixture.base.join("doctor-foreign");
    fs::create_dir(&foreign).unwrap();
    #[cfg(unix)]
    mode(&foreign, 0o700);
    fs::write(foreign.join("format.json"), b"not json").unwrap();
    let foreign = foreign.to_string_lossy().into_owned();
    let (report, code) = cli::dispatch(&strings(&["doctor", "--root", &foreign]));
    assert_eq!(code, 5, "{report}");
    assert_eq!(report["error"]["code"], "integrity_findings");
}

fn wish_files(
    fixture: &Fixture,
    item_id: &str,
    status: &str,
    current_visit: usize,
) -> (Vec<RsSaveMaterial>, serde_json::Map<String, Value>) {
    let plan = xdesign::simple_flow::designed_simple_flow_plan();
    let plan_bytes = serde_json::to_vec_pretty(&json!({
        "plan_id": plan.plan_id,
        "initial_stage_id": plan.initial_stage_id,
        "stages": plan.stages,
        "checkpoints": plan.checkpoints,
    }))
    .unwrap();
    let plan_uri = "flow/plan.json";
    fixture.write(plan_uri, &plan_bytes);
    let work_bytes = b"{\"work\":true}".to_vec();
    fixture.write("flow/work.json", &work_bytes);
    let wish_bytes = b"package WishExample { }".to_vec();
    fixture.write("wish.sysml", &wish_bytes);
    let run_id = "123e4567-e89b-42d3-a456-426614174000".to_owned();
    let empty_snapshot = digest(b"");
    let visits: Vec<_> = ["requirements", "design", "implement", "verify"]
        .into_iter()
        .enumerate()
        .map(|(index, stage)| {
            json!({
                "stage_id": stage,
                "visit_index": 0,
                "status": if index == current_visit { "active" } else { "pending" },
                "artifact_revision": 0,
                "artifact_snapshot_digest": empty_snapshot,
                "artifact_digests": [],
                "activities": [],
                "assessments": [],
            })
        })
        .collect();
    let run_bytes = serde_json::to_vec_pretty(&json!({
        "run_id": run_id,
        "execution_type": "FlowRun",
        "plan_id": "designedSimpleFlow",
        "plan_uri": plan_uri,
        "plan_sha256": digest(&plan_bytes),
        "wish_model_uri": "wish.sysml",
        "status": status,
        "current_visit": current_visit,
        "visits": visits,
    }))
    .unwrap();
    fixture.write("flow/run.json", &run_bytes);
    let registry_bytes = serde_json::to_vec_pretty(&json!({
        "schema_version": 1,
        "wish_id": item_id,
        "flow_id": "simple-flow",
        "run_id": run_id,
        "wish_model_uri": "wish.sysml",
        "plan_uri": plan_uri,
        "plan_sha256": digest(&plan_bytes),
        "work_uri": "flow/work.json",
        "work_sha256": digest(&work_bytes),
        "run_record_uri": "flow/run.json",
    }))
    .unwrap();
    fixture.write("flow/registry.json", &registry_bytes);
    let materials = vec![
        RsSaveMaterial {
            uri: plan_uri.into(),
            role: "plan".into(),
            sha256: digest(&plan_bytes),
            source: Some(PathBuf::from(plan_uri)),
        },
        RsSaveMaterial {
            uri: "flow/work.json".into(),
            role: "work".into(),
            sha256: digest(&work_bytes),
            source: Some(PathBuf::from("flow/work.json")),
        },
        RsSaveMaterial {
            uri: "flow/run.json".into(),
            role: "run-record".into(),
            sha256: digest(&run_bytes),
            source: Some(PathBuf::from("flow/run.json")),
        },
        RsSaveMaterial {
            uri: "flow/registry.json".into(),
            role: "registry".into(),
            sha256: digest(&registry_bytes),
            source: Some(PathBuf::from("flow/registry.json")),
        },
        RsSaveMaterial {
            uri: "wish.sysml".into(),
            role: "wish-source".into(),
            sha256: digest(&wish_bytes),
            source: Some(PathBuf::from("wish.sysml")),
        },
    ];
    let metadata = json!({
        "flow_id": "simple-flow",
        "registry_uri": "flow/registry.json",
        "run_uri": "flow/run.json",
    })
    .as_object()
    .cloned()
    .unwrap();
    (materials, metadata)
}

#[test]
fn wish_projection_and_status_filters() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let (materials, metadata) = wish_files(&fixture, "wish-active", "active", 0);
    let create = RsSaveRequest {
        schema_version: 1,
        operation_id: "wish-active-save".into(),
        item_id: "wish-active".into(),
        kind: "wish".into(),
        expected_revision: 0,
        metadata: metadata.clone(),
        materials,
        result: json!({"status": "saved"}),
    };
    store.save(&fixture.project, &create).unwrap();
    let active = store.get("wish-active", None).unwrap();
    let projection = active.projection.unwrap();
    assert_eq!(projection.status, xdesign::storage::RsWishStatus::Active);
    assert_eq!(projection.current_stage.as_deref(), Some("requirements"));
    assert_eq!(projection.revision, 1);

    let carried = create.materials.clone();
    let (next_materials, _) = wish_files(&fixture, "wish-active", "blocked", 1);
    let next_run = next_materials
        .into_iter()
        .find(|material| material.uri == "flow/run.json")
        .unwrap();
    let mut append_materials: Vec<_> = carried
        .into_iter()
        .filter(|material| material.uri != "flow/run.json")
        .map(|mut material| {
            material.source = None;
            material
        })
        .collect();
    append_materials.push(next_run);
    let append = RsSaveRequest {
        operation_id: "wish-blocked-save".into(),
        expected_revision: 1,
        materials: append_materials.clone(),
        result: json!({"status": "blocked"}),
        ..create.clone()
    };
    store.save(&fixture.project, &append).unwrap();
    let blocked = store.get("wish-active", None).unwrap().projection.unwrap();
    assert_eq!(blocked.status, xdesign::storage::RsWishStatus::Blocked);
    assert_eq!(blocked.current_stage.as_deref(), Some("design"));
    assert_eq!(
        store
            .get("wish-active", Some(1))
            .unwrap()
            .projection
            .unwrap()
            .status,
        xdesign::storage::RsWishStatus::Active
    );

    let (completed_materials, _) = wish_files(&fixture, "wish-active", "completed", 3);
    let completed_run = completed_materials
        .into_iter()
        .find(|material| material.uri == "flow/run.json")
        .unwrap();
    let mut completed_materials: Vec<_> = append_materials
        .into_iter()
        .filter(|material| material.uri != "flow/run.json")
        .collect();
    completed_materials.push(completed_run);
    let completed = RsSaveRequest {
        operation_id: "wish-completed-save".into(),
        expected_revision: 2,
        materials: completed_materials,
        result: json!({"status": "completed"}),
        ..append.clone()
    };
    store.save(&fixture.project, &completed).unwrap();
    let delivered = store.get("wish-active", None).unwrap().projection.unwrap();
    assert_eq!(delivered.status, xdesign::storage::RsWishStatus::Delivered);
    assert_eq!(delivered.current_stage, None);
    assert_eq!(store.list(None, Some("delivered")).unwrap().len(), 1);
    assert!(store
        .list(Some("wish"), Some("received"))
        .unwrap()
        .is_empty());

    let (bad_materials, bad_metadata) = wish_files(&fixture, "wish-bad", "active", 99);
    let bad = RsSaveRequest {
        schema_version: 1,
        operation_id: "wish-bad-save".into(),
        item_id: "wish-bad".into(),
        kind: "wish".into(),
        expected_revision: 0,
        metadata: bad_metadata,
        materials: bad_materials,
        result: Value::Null,
    };
    store.save(&fixture.project, &bad).unwrap();
    assert!(matches!(
        store.get("wish-bad", None),
        Err(RsStoreError::Integrity(_))
    ));
}

#[test]
fn empty_and_partial_roots() {
    let fixture = Fixture::new();
    let empty = fixture.base.join("empty");
    fs::create_dir(&empty).unwrap();
    #[cfg(unix)]
    mode(&empty, 0o700);
    drop(RsStore::init(&empty).unwrap());
    assert!(RsStore::open(&empty).is_ok());

    let partial = fixture.base.join("partial");
    fs::create_dir(&partial).unwrap();
    #[cfg(unix)]
    mode(&partial, 0o700);
    let marker = json!({"format":"xdesign.store", "schema_version":1, "store_id":"partial"});
    let marker_path = partial.join("format.json");
    fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
    #[cfg(unix)]
    mode(&marker_path, 0o600);
    assert!(matches!(
        RsStore::open(&partial),
        Err(RsStoreError::UnsupportedFormat(_))
    ));
    assert!(matches!(
        RsStore::init(&partial),
        Err(RsStoreError::UnsupportedFormat(_))
    ));
    assert!(!partial.join("store.db").exists());
}

#[cfg(unix)]
#[test]
fn permission_truncation_and_missing_reads() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "integrity-save",
        "integrity-item",
        0,
        vec![fixture.material("integrity.txt", b"abcdef")],
    );
    store.save(&fixture.project, &request).unwrap();
    let blob = fixture.blob(&digest(b"abcdef"));
    mode(&blob, 0o600);
    let file = fs::OpenOptions::new().write(true).open(&blob).unwrap();
    file.set_len(1).unwrap();
    drop(file);
    mode(&blob, 0o400);
    assert!(matches!(
        store.read_material("integrity-item", None, "integrity.txt"),
        Err(RsStoreError::Integrity(_))
    ));
    assert!(store
        .doctor(false)
        .unwrap()
        .findings
        .iter()
        .any(|finding| { finding.kind == xdesign::storage::RsDoctorFindingKind::MissingBlob }));
    fs::remove_file(&blob).unwrap();
    assert!(matches!(
        store.read_material("integrity-item", None, "integrity.txt"),
        Err(RsStoreError::Integrity(_))
    ));

    mode(&fixture.root, 0o755);
    assert!(matches!(
        store.list(None, None),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(store.doctor(false).unwrap().findings.iter().any(|finding| {
        finding.kind == xdesign::storage::RsDoctorFindingKind::PermissionDefect
    }));
    mode(&fixture.root, 0o700);
    fs::write(&blob, b"abcdef").unwrap();
    mode(&blob, 0o600);
    assert!(matches!(
        store.read_material("integrity-item", None, "integrity.txt"),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(store.doctor(false).unwrap().findings.iter().any(|finding| {
        finding.kind == xdesign::storage::RsDoctorFindingKind::PermissionDefect
    }));
}

#[test]
fn own_store_source_and_output_are_refused() {
    let fixture = Fixture::new();
    let root = fixture.project.join(".xdesign");
    let store = RsStore::init(&root).unwrap();
    let request = fixture.request(
        "safe-save",
        "safe-item",
        0,
        vec![fixture.material("safe.txt", b"safe")],
    );
    store.save(&fixture.project, &request).unwrap();
    let own_source = fixture.request(
        "own-source",
        "own-source",
        0,
        vec![RsSaveMaterial {
            uri: "copied.db".into(),
            role: "artifact".into(),
            sha256: digest(b"not read"),
            source: Some(PathBuf::from(".xdesign/store.db")),
        }],
    );
    assert!(matches!(
        store.save(&fixture.project, &own_source),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(matches!(
        store.outcome("own-source"),
        Err(RsStoreError::NotFound(_))
    ));

    let output = root.join("foreign-output");
    let (response, code) = cli::dispatch(&strings(&[
        "read",
        "safe-item",
        "--uri",
        "safe.txt",
        "--out",
        &output.to_string_lossy(),
        "--root",
        &root.to_string_lossy(),
    ]));
    assert_eq!(code, 5, "{response}");
    assert!(!output.exists());
}

#[test]
fn cli_pending_diagnostics_and_info_dedup() {
    let fixture = Fixture::new();
    drop(fixture.store());
    let request = fixture.request(
        "pending-cli",
        "pending-cli",
        0,
        vec![RsSaveMaterial {
            uri: "pending.txt".into(),
            role: "artifact".into(),
            sha256: digest(b"expected"),
            source: Some(PathBuf::from("pending.txt")),
        }],
    );
    fixture.write("pending.txt", b"actual");
    let request_path = fixture.base.join("pending-request.json");
    fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
    let (response, code) = cli::dispatch(&strings(&[
        "save",
        "--request",
        &request_path.to_string_lossy(),
        "--project-root",
        &fixture.project.to_string_lossy(),
        "--root",
        &fixture.root.to_string_lossy(),
    ]));
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["operation_id"], "pending-cli");
    assert_eq!(response["error"]["receipt_state"], "pending");

    let store = RsStore::open(&fixture.root).unwrap();
    let bytes = b"deduplicated";
    store
        .save(
            &fixture.project,
            &fixture.request(
                "dedup-one",
                "dedup-one",
                0,
                vec![fixture.material("one.txt", bytes)],
            ),
        )
        .unwrap();
    store
        .save(
            &fixture.project,
            &fixture.request(
                "dedup-two",
                "dedup-two",
                0,
                vec![fixture.material("two.txt", bytes)],
            ),
        )
        .unwrap();
    let (info, code) = cli::dispatch(&strings(&[
        "info",
        "--root",
        &fixture.root.to_string_lossy(),
    ]));
    assert_eq!(code, 0, "{info}");
    assert_eq!(info["data"]["blob_count"], 1);
    assert_eq!(info["data"]["item_count"], 2);
    assert_eq!(info["data"]["pending_operation_count"], 1);
}

#[test]
fn received_projection() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let (materials, metadata) = wish_files(&fixture, "wish-received", "pending", 0);
    let request = RsSaveRequest {
        schema_version: 1,
        operation_id: "wish-received-save".into(),
        item_id: "wish-received".into(),
        kind: "wish".into(),
        expected_revision: 0,
        metadata,
        materials,
        result: Value::Null,
    };
    store.save(&fixture.project, &request).unwrap();
    let projection = store
        .get("wish-received", None)
        .unwrap()
        .projection
        .unwrap();
    assert_eq!(projection.status, xdesign::storage::RsWishStatus::Received);
    assert_eq!(projection.current_stage.as_deref(), Some("requirements"));
    assert_eq!(store.list(Some("wish"), Some("received")).unwrap().len(), 1);
}

#[test]
fn json_size_and_depth_boundaries() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let oversized = vec![b' '; 16 * 1024 * 1024 + 1];
    assert!(matches!(
        store.save_bytes(&fixture.project, &oversized),
        Err(RsStoreError::InvalidInput(message)) if message.contains("16 MiB")
    ));

    // A result nested to the request limit would make the receipt envelope
    // exceed the same limit, so the deepest accepted result leaves the
    // reserved envelope margin (64 - 6 levels) free.
    let mut accepted = fixture.request("depth-64", "depth-64", 0, vec![]);
    accepted.result = (0..58).fold(Value::Null, |value, _| json!([value]));
    let expected = accepted.result.clone();
    store.save(&fixture.project, &accepted).unwrap();
    assert_eq!(store.outcome("depth-64").unwrap().result, expected);

    let mut beyond_reservation = fixture.request("depth-59", "depth-59", 0, vec![]);
    beyond_reservation.result = (0..59).fold(Value::Null, |value, _| json!([value]));
    assert!(matches!(
        store.save(&fixture.project, &beyond_reservation),
        Err(RsStoreError::InvalidInput(message))
            if message.contains("response envelope reservation")
    ));
    assert!(matches!(
        store.outcome("depth-59"),
        Err(RsStoreError::NotFound(_))
    ));

    let mut rejected = fixture.request("depth-65", "depth-65", 0, vec![]);
    rejected.result = (0..64).fold(Value::Null, |value, _| json!([value]));
    assert!(matches!(
        store.save(&fixture.project, &rejected),
        Err(RsStoreError::InvalidInput(_))
    ));
    assert!(matches!(
        store.outcome("depth-65"),
        Err(RsStoreError::NotFound(_))
    ));
}

#[test]
fn response_envelope_bytes_are_reserved_before_commitment() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let mut request = fixture.request("envelope-bytes", "envelope-item", 0, vec![]);
    request.result = json!({"blob": "a".repeat(16_776_000)});
    let error = store.save(&fixture.project, &request).unwrap_err();
    assert!(
        matches!(&error, RsStoreError::InvalidInput(message)
            if message.contains("response envelope")),
        "{error}"
    );
    assert!(matches!(
        store.outcome("envelope-bytes"),
        Err(RsStoreError::NotFound(_))
    ));
    assert!(matches!(
        store.get("envelope-item", None),
        Err(RsStoreError::NotFound(_))
    ));
}

#[test]
fn numeric_precision_is_exact_or_rejected() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let cases: [(&str, bool); 13] = [
        ("18446744073709551617", false),
        ("18446744073709551615", true),
        ("-9223372036854775809", false),
        ("-9223372036854775808", true),
        ("0.30000000000000000004", false),
        ("9007199254740993.0", false),
        // Review S1 reproductions: the serde_json decoder that storage uses
        // parses these to a value whose stored decimal form differs from the
        // token, so they must be rejected before reservation instead of being
        // committed with a silently altered number.
        ("9764.519195169985", false),
        ("1.0468000232875951e-07", false),
        // Its decimal value survives, but serde_json does not reach a stable
        // encode/decode fixed point for it, so a committed revision could read
        // back as a different value.
        ("1.9432074656054558e-171", false),
        ("0.1", true),
        ("1e2", true),
        ("1.0000000000000002", true),
        ("-0", true),
    ];
    for (index, (token, exact)) in cases.iter().enumerate() {
        let operation = format!("numeric-{index}");
        let item = format!("numeric-item-{index}");
        let raw = format!(
            "{{\"schema_version\":1,\"operation_id\":\"{operation}\",\"item_id\":\"{item}\",\"kind\":\"artifact\",\"expected_revision\":0,\"metadata\":{{\"n\":{token}}},\"materials\":[],\"result\":{{\"ok\":true}}}}"
        )
        .into_bytes();
        let result = store.save_bytes(&fixture.project, &raw);
        if !*exact {
            let error = result.unwrap_err();
            assert!(
                matches!(&error, RsStoreError::InvalidInput(message)
                    if message.contains("precision")
                        || message.contains("64-bit integer range")
                        || message.contains("stable")),
                "token {token}: {error}"
            );
            assert!(matches!(
                store.outcome(&operation),
                Err(RsStoreError::NotFound(_))
            ));
            assert!(matches!(
                store.get(&item, None),
                Err(RsStoreError::NotFound(_))
            ));
            // Retrying the rejected request stays pre-reservation: still a
            // typed error and still no operation receipt.
            assert!(matches!(
                store.save_bytes(&fixture.project, &raw),
                Err(RsStoreError::InvalidInput(_))
            ));
            assert!(matches!(
                store.outcome(&operation),
                Err(RsStoreError::NotFound(_))
            ));
            continue;
        }
        let receipt =
            result.unwrap_or_else(|error| panic!("token {token} should be accepted: {error}"));
        assert_eq!(
            receipt.outcome,
            RsOperationOutcome::Committed,
            "token {token}"
        );
        let expected: Value = serde_json::from_str(token).unwrap();
        let revision = store.get(&item, None).unwrap();
        assert_eq!(revision.metadata.get("n"), Some(&expected), "token {token}");
        let outcome = store.outcome(&operation).unwrap();
        assert_eq!(outcome.result, receipt.result, "token {token}");
        // Exact retry returns the same terminal receipt, and neither get nor
        // outcome reports an integrity failure for an accepted number.
        let retry = store.save_bytes(&fixture.project, &raw).unwrap();
        assert_eq!(
            retry.outcome,
            RsOperationOutcome::Committed,
            "token {token}"
        );
        assert_eq!(retry.revision, receipt.revision, "token {token}");
        assert_eq!(store.get(&item, None).unwrap(), revision, "token {token}");
    }

    // A drifting number in a result (the review's second probe) is rejected
    // with a typed error and leaves no operation receipt.
    let result_raw = b"{\"schema_version\":1,\"operation_id\":\"numeric-result-token\",\"item_id\":\"numeric-result-token\",\"kind\":\"artifact\",\"expected_revision\":0,\"metadata\":{},\"materials\":[],\"result\":{\"n\":1.0468000232875951e-07}}".to_vec();
    assert!(matches!(
        store.save_bytes(&fixture.project, &result_raw),
        Err(RsStoreError::InvalidInput(_))
    ));
    assert!(matches!(
        store.outcome("numeric-result-token"),
        Err(RsStoreError::NotFound(_))
    ));

    // S8: extreme exponents are syntactically valid JSON numbers. They must
    // produce a typed error before reservation, never an arithmetic panic.
    for (index, token) in [
        "0.0e-2147483648",
        "1e999999999999999999999",
        "0e-999999999999999999999",
        "1e-400",
        "0e2147483647",
        "123456789012345678901234567890.123456789e-99999",
    ]
    .into_iter()
    .enumerate()
    {
        let operation = format!("numeric-extreme-{index}");
        let item = format!("numeric-extreme-item-{index}");
        let raw = format!(
            "{{\"schema_version\":1,\"operation_id\":\"{operation}\",\"item_id\":\"{item}\",\"kind\":\"artifact\",\"expected_revision\":0,\"metadata\":{{\"n\":{token}}},\"materials\":[],\"result\":{{\"ok\":true}}}}"
        )
        .into_bytes();
        let error = store.save_bytes(&fixture.project, &raw).unwrap_err();
        assert!(
            matches!(&error, RsStoreError::InvalidInput(_)),
            "extreme token {token}: {error}"
        );
        assert!(matches!(
            store.outcome(&operation),
            Err(RsStoreError::NotFound(_))
        ));
    }
}

#[test]
fn manifest_name_collisions_are_rejected() {
    let fixture = Fixture::new();
    let store = fixture.store();
    // Validation rejects a colliding manifest before any source is read, so
    // these URIs deliberately have no backing files on disk.
    let declared = |uri: &str| RsSaveMaterial {
        uri: uri.into(),
        role: "artifact".into(),
        sha256: digest(uri.as_bytes()),
        source: Some(PathBuf::from(uri)),
    };
    let cases: Vec<Vec<&str>> = vec![
        vec!["a", "a-b", "a/x"],
        vec!["File.txt", "file.txt"],
        vec!["A/x", "a/y"],
        vec!["dir/nested.txt", "DIR"],
        // S4 review reproductions: lowercase-only folding missed canonical
        // decomposition and full case folding, so these committed but could not
        // be materialized on a case- and normalization-insensitive volume.
        vec!["\u{03a3}.txt", "\u{03c2}.txt"],
        vec!["\u{00e9}.txt", "e\u{0301}.txt"],
        vec!["\u{03a3}/x", "\u{03c3}/x"],
        vec!["\u{03a3}/x", "\u{03c3}/y"],
        vec!["\u{212b}.txt", "\u{00c5}.txt"],
        // Folded file-vs-implied-directory collisions.
        vec!["A", "a/x"],
        vec!["\u{00c9}", "\u{00e9}/x"],
    ];
    for (index, uris) in cases.iter().enumerate() {
        let uris = &uris[..];
        let operation = format!("collide-{index}");
        let materials: Vec<_> = uris.iter().map(|uri| declared(uri)).collect();
        let request = fixture.request(&operation, "collide-item", 0, materials);
        let error = store.save(&fixture.project, &request).unwrap_err();
        assert!(
            matches!(&error, RsStoreError::InvalidInput(message) if message.contains("collide")),
            "uris {uris:?}: {error}"
        );
        assert!(matches!(
            store.outcome(&operation),
            Err(RsStoreError::NotFound(_))
        ));
    }

    // Adjacent sorted names without a real filesystem collision stay valid
    // and materialize on checkout.
    let materials = vec![
        fixture.material("plain.txt", b"plain"),
        fixture.material("plain.txt-with-suffix", b"suffixed"),
    ];
    let request = fixture.request("collide-valid", "collide-valid-item", 0, materials);
    store.save(&fixture.project, &request).unwrap();
    let destination = fixture.base.join("materialized");
    store
        .checkout("collide-valid-item", None, &destination)
        .unwrap();
    assert_eq!(fs::read(destination.join("plain.txt")).unwrap(), b"plain");
    assert_eq!(
        fs::read(destination.join("plain.txt-with-suffix")).unwrap(),
        b"suffixed"
    );

    // Single Unicode names are not collisions: composed and decomposed
    // spellings of one name, final sigma and an uppercase sigma directory all
    // save and check out successfully.
    let materials = vec![
        fixture.material("\u{00e9}.txt", b"composed"),
        fixture.material("e\u{0301}-decomposed.txt", b"decomposed"),
        fixture.material("\u{03c2}.txt", b"final sigma"),
        fixture.material("\u{03a3}/child.txt", b"sigma dir"),
    ];
    let request = fixture.request(
        "collide-unicode-valid",
        "collide-unicode-valid-item",
        0,
        materials,
    );
    store.save(&fixture.project, &request).unwrap();
    let destination = fixture.base.join("materialized-unicode");
    store
        .checkout("collide-unicode-valid-item", None, &destination)
        .unwrap();
    assert_eq!(
        fs::read(destination.join("\u{00e9}.txt")).unwrap(),
        b"composed"
    );
    assert_eq!(
        fs::read(destination.join("e\u{0301}-decomposed.txt")).unwrap(),
        b"decomposed"
    );
    assert_eq!(
        fs::read(destination.join("\u{03c2}.txt")).unwrap(),
        b"final sigma"
    );
    assert_eq!(
        fs::read(destination.join("\u{03a3}").join("child.txt")).unwrap(),
        b"sigma dir"
    );
}

#[test]
fn pinned_revision_reads_are_stable_across_appends() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let first = fixture.request(
        "pin-1",
        "pin-item",
        0,
        vec![fixture.material("data.txt", b"revision one")],
    );
    store.save(&fixture.project, &first).unwrap();
    let second = fixture.request(
        "pin-2",
        "pin-item",
        1,
        vec![fixture.material("data.txt", b"revision two")],
    );
    store.save(&fixture.project, &second).unwrap();

    assert_eq!(
        store
            .read_material("pin-item", Some(1), "data.txt")
            .unwrap(),
        b"revision one"
    );
    assert_eq!(
        store.read_material("pin-item", None, "data.txt").unwrap(),
        b"revision two"
    );

    let output = fixture.base.join("pinned-read.txt");
    let (response, code) = cli::dispatch(&strings(&[
        "read",
        "pin-item",
        "--uri",
        "data.txt",
        "--revision",
        "1",
        "--out",
        &output.to_string_lossy(),
        "--root",
        &fixture.root.to_string_lossy(),
    ]));
    assert_eq!(code, 0, "{response}");
    assert_eq!(fs::read(&output).unwrap(), b"revision one");
    assert_eq!(response["data"]["sha256"], json!(digest(b"revision one")));
}

#[test]
fn concurrent_reads_stay_consistent_while_head_advances() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let seed = fixture.request(
        "advance-seed",
        "shared-item",
        0,
        vec![fixture.material("shared.txt", b"advance 0")],
    );
    store.save(&fixture.project, &seed).unwrap();
    drop(store);

    let root = fixture.root.clone();
    let project = fixture.project.clone();
    let writer = std::thread::spawn(move || {
        let store = RsStore::open(&root).unwrap();
        for revision in 1..=40u64 {
            let content = format!("advance {revision}").into_bytes();
            let material = RsSaveMaterial {
                uri: "shared.txt".into(),
                role: "artifact".into(),
                sha256: digest(&content),
                source: Some(PathBuf::from("shared.txt")),
            };
            fs::write(project.join("shared.txt"), &content).unwrap();
            let request = RsSaveRequest {
                schema_version: 1,
                operation_id: format!("advance-{revision}"),
                item_id: "shared-item".into(),
                kind: "artifact".into(),
                expected_revision: revision,
                metadata: Map::new(),
                materials: vec![material],
                result: json!({"revision": revision}),
            };
            let receipt = store.save(&project, &request).unwrap();
            assert_eq!(receipt.outcome, RsOperationOutcome::Committed);
        }
    });
    let reader_store = RsStore::open(&fixture.root).unwrap();
    let mut reads = 0u64;
    while !writer.is_finished() {
        let selected = reader_store.get("shared-item", None).unwrap();
        let material = selected
            .materials
            .iter()
            .find(|material| material.uri == "shared.txt")
            .cloned()
            .unwrap();
        let bytes = reader_store
            .read_material("shared-item", Some(selected.revision), "shared.txt")
            .unwrap();
        assert_eq!(bytes.len() as u64, material.byte_length);
        assert_eq!(digest(&bytes), material.sha256);
        reads += 1;
    }
    writer.join().unwrap();
    assert!(reads > 0, "reader never observed the item");
    let head = reader_store.get("shared-item", None).unwrap();
    assert_eq!(head.revision, 41);
    assert_eq!(
        reader_store
            .read_material("shared-item", Some(41), "shared.txt")
            .unwrap(),
        b"advance 40"
    );
}

#[cfg(unix)]
#[test]
fn destination_ancestor_symlinks_are_refused() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "ancestor-symlink-save",
        "ancestor-symlink-item",
        0,
        vec![fixture.material("through.txt", b"through an alias")],
    );
    store.save(&fixture.project, &request).unwrap();

    let actual = fixture.base.join("actual-tree/nested");
    fs::create_dir_all(&actual).unwrap();
    symlink(&actual, fixture.base.join("read-alias")).unwrap();

    let output = fixture.base.join("read-alias/nested/output.txt");
    let (response, code) = cli::dispatch(&strings(&[
        "read",
        "ancestor-symlink-item",
        "--uri",
        "through.txt",
        "--out",
        &output.to_string_lossy(),
        "--root",
        &fixture.root.to_string_lossy(),
    ]));
    assert_eq!(code, 5, "{response}");
    assert!(!actual.join("output.txt").exists());

    let checkout_alias = fixture.base.join("checkout-tree/nested");
    fs::create_dir_all(&checkout_alias).unwrap();
    symlink(&checkout_alias, fixture.base.join("checkout-alias")).unwrap();
    assert!(matches!(
        store.checkout(
            "ancestor-symlink-item",
            None,
            &fixture.base.join("checkout-alias/nested/destination"),
        ),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(!checkout_alias.join("destination").exists());
}

#[cfg(target_os = "macos")]
#[test]
fn store_self_ingestion_case_alias_is_refused() {
    let fixture = Fixture::new();
    let root = fixture.project.join(".xdesign");
    let store = RsStore::init(&root).unwrap();
    let aliased = fixture.request(
        "case-alias",
        "case-alias-item",
        0,
        vec![RsSaveMaterial {
            uri: "stolen-format.json".into(),
            role: "artifact".into(),
            sha256: digest(b"not read"),
            source: Some(PathBuf::from(".XDESIGN/format.json")),
        }],
    );
    assert!(matches!(
        store.save(&fixture.project, &aliased),
        Err(RsStoreError::UnsafePath(message)) if message.contains("ingest its own root")
    ));
    assert!(matches!(
        store.outcome("case-alias"),
        Err(RsStoreError::NotFound(_))
    ));
}

#[cfg(unix)]
#[test]
fn checkout_rejects_symlinked_parent() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "checkout-symlink-save",
        "checkout-symlink-item",
        0,
        vec![fixture.material("checkout.txt", b"checkout")],
    );
    store.save(&fixture.project, &request).unwrap();
    let actual_parent = fixture.base.join("actual-parent");
    fs::create_dir(&actual_parent).unwrap();
    let link_parent = fixture.base.join("link-parent");
    symlink(&actual_parent, &link_parent).unwrap();
    assert!(matches!(
        store.checkout(
            "checkout-symlink-item",
            None,
            &link_parent.join("destination"),
        ),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(!actual_parent.join("destination").exists());
}

#[test]
fn revision_metadata_must_match_producing_request() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request("metadata-save", "metadata-item", 0, vec![]);
    store.save(&fixture.project, &request).unwrap();
    let connection = rusqlite::Connection::open(fixture.root.join("store.db")).unwrap();
    connection
        .execute_batch(
            "DROP TRIGGER revision_no_update;
             UPDATE revisions SET metadata_json = '{\"changed\":true}'
             WHERE item_id = 'metadata-item' AND revision = 1;",
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        store.get("metadata-item", Some(1)),
        Err(RsStoreError::Integrity(_))
    ));
    let report = store.doctor(false).unwrap();
    assert!(report
        .findings
        .iter()
        .any(|finding| { finding.kind == xdesign::storage::RsDoctorFindingKind::InvalidRecord }));
}

#[test]
fn revision_manifest_must_match_producing_request() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "manifest-save",
        "manifest-item",
        0,
        vec![fixture.material("manifest.txt", b"manifest")],
    );
    store.save(&fixture.project, &request).unwrap();
    let connection = rusqlite::Connection::open(fixture.root.join("store.db")).unwrap();
    connection
        .execute_batch(
            "DROP TRIGGER material_no_update;
             UPDATE materials SET role = 'forged'
             WHERE item_id = 'manifest-item' AND revision = 1 AND uri = 'manifest.txt';",
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        store.get("manifest-item", Some(1)),
        Err(RsStoreError::Integrity(_))
    ));
    let report = store.doctor(false).unwrap();
    assert!(report
        .findings
        .iter()
        .any(|finding| { finding.kind == xdesign::storage::RsDoctorFindingKind::InvalidRecord }));
}
