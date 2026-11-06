use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::{Command, Output},
};
use xdesign::core::{sha256_bytes, snapshot_digest, RsArtifactDigest};

struct Project {
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let root = std::env::current_dir().unwrap().join(format!(
            "cache/implementation/storage-cli-tests/{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(root.join("incoming")).unwrap();
        fs::create_dir_all(root.join("mbse/wishes")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("incoming/wish.sysml"),
            "package ExternalWish {}\n",
        )
        .unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        Self { root }
    }

    fn simple() -> Self {
        let project = Self::new();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for uri in [
            "tools/checks/common.py",
            "tools/checks/simple_requirements.py",
            "tools/checks/simple_design.py",
            "tools/checks/simple_implementation.py",
            "tools/checks/simple_record.py",
            "mbse/model/requirements.sysml",
            "mbse/model/traceability.sysml",
            "mbse/model/detailedDesign.sysml",
            "mbse/model/composition.sysml",
            "tools/sysml_web/bin/sysml",
        ] {
            let destination = project.root.join(uri);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(root.join(uri), destination).unwrap();
        }
        project
    }

    fn call(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_xdesign"))
            .current_dir(&self.root)
            .env_remove("XDESIGN_STORE_ROOT")
            .args(args)
            .output()
            .unwrap()
    }

    fn call_with_root(&self, args: &[&str], root: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_xdesign"))
            .current_dir(&self.root)
            .env("XDESIGN_STORE_ROOT", root)
            .args(args)
            .output()
            .unwrap()
    }

    fn stage(&self, args: &[&str], code: i32) -> Value {
        let output = self.call(args);
        assert_eq!(
            output.status.code(),
            Some(code),
            "args={args:?}, stderr={}, stdout={}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }

    fn stage_error(&self, args: &[&str]) -> String {
        let output = self.call(args);
        assert_eq!(output.status.code(), Some(2), "args={args:?}",);
        assert!(output.stdout.is_empty());
        String::from_utf8(output.stderr).unwrap()
    }

    fn store(&self, args: &[&str], code: i32) -> Value {
        let output = self.call(args);
        assert_eq!(
            output.status.code(),
            Some(code),
            "args={args:?}, stderr={}, stdout={}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let bytes = if code == 0 {
            assert!(output.stderr.is_empty());
            &output.stdout
        } else {
            assert!(output.stdout.is_empty());
            &output.stderr
        };
        let envelope: Value = serde_json::from_slice(bytes).unwrap();
        envelope
    }

    fn init(&self, id: &str, flow: &str) -> Value {
        self.stage(
            &["init", id, "--flow", flow, "--wish", "incoming/wish.sysml"],
            0,
        )
    }

    fn next(&self, id: &str) -> Value {
        self.stage(&["next", id], 0)
    }

    fn head(&self, id: &str) -> Value {
        self.store(&["store", "get", id], 0)["data"]["revision"].clone()
    }

    fn write(&self, uri: &str, bytes: &[u8]) {
        let path = self.root.join(uri);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn result(&self, id: &str, response: &Value, artifacts: &[&str], outcome: &str) -> Value {
        let work = &response["work"];
        let supplied: Vec<_> = artifacts
            .iter()
            .map(|uri| RsArtifactDigest {
                uri: uri.to_string(),
                sha256: sha256_bytes(&fs::read(self.root.join(uri)).unwrap()),
            })
            .collect();
        let mut candidates: BTreeMap<String, RsArtifactDigest> =
            serde_json::from_value::<Vec<RsArtifactDigest>>(work["candidate_artifacts"].clone())
                .unwrap()
                .into_iter()
                .map(|artifact| (artifact.uri.clone(), artifact))
                .collect();
        for artifact in &supplied {
            candidates.insert(artifact.uri.clone(), artifact.clone());
        }
        json!({
            "wish_id": id,
            "run_id": response["run_id"],
            "stage_id": work["stage_id"],
            "visit_index": work["visit_index"],
            "base_revision": work["artifact_revision"],
            "state_token": response["state_token"],
            "description": "Review retained process evidence",
            "performed_by": "storage-process-test",
            "artifacts": supplied,
            "judgment": {
                "authority": "agent",
                "identity": "storage-test-reviewer",
                "outcome": outcome,
                "rationale": "The exact retained bytes satisfy this fixture's semantic gate.",
                "artifact_snapshot_digest": snapshot_digest(&candidates.into_values().collect::<Vec<_>>())
            }
        })
    }

    fn save_json(&self, uri: &str, value: &Value) {
        self.write(uri, &serde_json::to_vec_pretty(value).unwrap());
    }

    fn submit(
        &self,
        id: &str,
        response: &Value,
        artifacts: &[&str],
        operation_id: &str,
        code: i32,
    ) -> Value {
        let result = self.result(id, response, artifacts, "passed");
        self.save_json("result.json", &result);
        self.stage(
            &[
                "check",
                id,
                "--result",
                "result.json",
                "--operation-id",
                operation_id,
            ],
            code,
        )
    }

    fn verify_checkout(&self, head: &Value, destination: &str) {
        for material in head["materials"].as_array().unwrap() {
            let uri = material["uri"].as_str().unwrap();
            let bytes = fs::read(self.root.join(destination).join(uri)).unwrap();
            assert_eq!(
                bytes.len() as u64,
                material["byte_length"].as_u64().unwrap()
            );
            assert_eq!(sha256_bytes(&bytes), material["sha256"]);
        }
    }
}

#[test]
fn wish_verify_store_queries_conflict_cache_recovery_and_exact_completed_replay() {
    let project = Project::new();
    project.store(&["store", "init"], 0);
    project.store(&["store", "info"], 0);
    let initial = project.init("wish-test", "wish-verify");
    assert_eq!(initial["store_revision"], 1);
    assert!(initial["operation_id"].is_string());
    assert_eq!(initial["work"]["stage_id"], "wish");
    assert!(!project.root.join("mbse/wishes/wish-test.sysml").exists());
    assert_eq!(
        fs::read(project.root.join("cache/wishes/wish-test.sysml")).unwrap(),
        b"package ExternalWish {}\n"
    );
    project.write("incoming/wish.sysml", b"package ChangedImport {}\n");
    assert_eq!(
        project.next("wish-test")["state_token"],
        initial["state_token"]
    );
    assert_eq!(
        fs::read(project.root.join("cache/wishes/wish-test.sysml")).unwrap(),
        b"package ExternalWish {}\n"
    );
    let active = project.store(
        &["store", "list", "--kind", "wish", "--status", "active"],
        0,
    );
    assert_eq!(
        active["data"]["items"][0]["projection"]["current_stage"],
        "wish"
    );

    let blocked = project.stage(&["check", "wish-test"], 1);
    assert_eq!(blocked["store_revision"], 2);
    assert_eq!(blocked["advance"], "remain_blocked");
    let listed = project.store(&["store", "list", "--status", "blocked"], 0);
    assert_eq!(
        listed["data"]["items"][0]["projection"]["status"],
        "blocked"
    );

    let stale = project.result("wish-test", &initial, &[], "passed");
    project.save_json("stale.json", &stale);
    assert!(project
        .stage_error(&[
            "check",
            "wish-test",
            "--result",
            "stale.json",
            "--operation-id",
            "stale-result",
        ])
        .contains("stale result"));
    assert_eq!(project.head("wish-test")["revision"], 2);

    let verify = project.submit("wish-test", &blocked, &[], "wish-pass", 0);
    assert_eq!(verify["work"]["stage_id"], "verify");
    assert_eq!(verify["store_revision"], 3);
    project.write(
        "cache/evidence/wish-test/verification.json",
        b"{\"passed\":true}\n",
    );
    let verify = project.next("wish-test");
    let completed = project.submit(
        "wish-test",
        &verify,
        &["cache/evidence/wish-test/verification.json"],
        "final-check",
        0,
    );
    assert_eq!(completed["completed"], true);
    assert_eq!(completed["store_revision"], 4);
    assert_eq!(completed["operation_id"], "final-check");
    let replayed = project.stage(
        &[
            "check",
            "wish-test",
            "--result",
            "result.json",
            "--operation-id",
            "final-check",
        ],
        0,
    );
    assert_eq!(replayed, completed);
    assert_eq!(project.head("wish-test")["revision"], 4);
    let outcome = project.store(&["store", "outcome", "final-check"], 0);
    assert_eq!(outcome["data"]["receipt"]["outcome"], "committed");
    assert_eq!(outcome["data"]["receipt"]["revision"], 4);
    assert_eq!(outcome["data"]["receipt"]["result"]["response"], completed);

    let mut changed_result: Value =
        serde_json::from_slice(&fs::read(project.root.join("result.json")).unwrap()).unwrap();
    changed_result["description"] = json!("different request bytes");
    project.save_json("changed-result.json", &changed_result);
    assert!(project
        .stage_error(&[
            "check",
            "wish-test",
            "--result",
            "changed-result.json",
            "--operation-id",
            "final-check",
        ])
        .contains("operation_mismatch"));
    let history = project.store(&["store", "history", "wish-test"], 0);
    assert_eq!(history["data"]["revisions"].as_array().unwrap().len(), 4);
    assert_eq!(history["data"]["complete"], true);

    let head = project.head("wish-test");
    let conflict = json!({
        "schema_version": 1,
        "operation_id": "conflicting-check",
        "item_id": "wish-test",
        "kind": "wish",
        "expected_revision": 1,
        "metadata": head["metadata"],
        "materials": head["materials"].as_array().unwrap().iter().map(|material| json!({
            "uri": material["uri"], "role": material["role"], "sha256": material["sha256"]
        })).collect::<Vec<_>>(),
        "result": {"command": "check", "response": {"not_applied": true}, "exit_code": 0}
    });
    project.save_json("conflict-request.json", &conflict);
    let conflicted = project.store(
        &[
            "store",
            "save",
            "--request",
            "conflict-request.json",
            "--project-root",
            ".",
        ],
        4,
    );
    assert_eq!(conflicted["error"]["code"], "revision_conflict");
    assert_eq!(project.head("wish-test")["revision"], 4);
    assert_eq!(
        project.store(&["store", "outcome", "conflicting-check"], 0)["data"]["receipt"]["outcome"],
        "conflict"
    );
    assert!(project
        .stage_error(&[
            "check",
            "wish-test",
            "--result",
            "result.json",
            "--operation-id",
            "conflicting-check",
        ])
        .contains("revision_conflict"));
    assert_eq!(project.head("wish-test")["revision"], 4);

    fs::create_dir_all(project.root.join("checkout")).unwrap();
    project.store(
        &[
            "store",
            "checkout",
            "wish-test",
            "--destination",
            "checkout/complete",
        ],
        0,
    );
    project.verify_checkout(&head, "checkout/complete");
    fs::remove_dir_all(project.root.join("cache")).unwrap();
    let recovered = project.next("wish-test");
    assert_eq!(recovered["completed"], true);
    assert_eq!(recovered["store_revision"], 4);
    assert_eq!(recovered["state_token"], completed["state_token"]);
    assert!(project.root.join("cache/xdesign/wish-test/heads").is_dir());
    project.store(
        &[
            "store",
            "checkout",
            "wish-test",
            "--destination",
            "cache/recovered-checkout",
        ],
        0,
    );
    project.verify_checkout(&head, "cache/recovered-checkout");
    let completed_again = project.stage(
        &[
            "check",
            "wish-test",
            "--result",
            "missing-result-is-ignored.json",
            "--operation-id",
            "new-completion-observation",
        ],
        0,
    );
    assert!(completed_again["assessed"].is_null());
    assert_eq!(completed_again["store_revision"], 4);
    assert!(!project.root.join("mbse/wishes/wish-test.sysml").exists());
}

#[test]
fn simple_flow_commits_checker_exchange_and_failed_gate_then_restores_reader_snapshot() {
    let project = Project::simple();
    let initial = project.init("wish-test", "simple-flow");
    assert_eq!(initial["store_revision"], 1);
    let checked = project.stage(&["check", "wish-test"], 0);
    assert_eq!(checked["store_revision"], 2);
    assert_eq!(checked["work"]["stage_id"], "design");
    let report_uri = checked["assessed"]["assessments"][0]["evidence_uris"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let report_bytes = fs::read(project.root.join(&report_uri)).unwrap();
    let report: Value = serde_json::from_slice(&report_bytes).unwrap();
    let head = project.head("wish-test");
    let materials: BTreeMap<_, _> = head["materials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|material| {
            (
                material["uri"].as_str().unwrap().to_owned(),
                material["sha256"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(materials[&report_uri], sha256_bytes(&report_bytes));
    let run: Value = serde_json::from_slice(
        &fs::read(project.root.join("cache/xdesign/wish-test/run.json")).unwrap(),
    )
    .unwrap();
    for (uri, digest) in report["input_sha256"].as_object().unwrap() {
        let retained = run["visits"][0]["artifact_snapshots"][uri]
            .as_str()
            .unwrap_or(uri);
        assert_eq!(materials[retained], digest.as_str().unwrap());
        assert!(project.root.join(retained).is_file(), "missing {retained}");
    }
    for suffix in [".stdout.log", ".stderr.log"] {
        assert!(materials.contains_key(&format!("{report_uri}{suffix}")));
    }
    assert!(head["materials"]
        .as_array()
        .unwrap()
        .iter()
        .any(|material| material["role"] == "stage-response"));

    let blocked = project.stage(&["check", "wish-test"], 1);
    assert_eq!(blocked["store_revision"], 3);
    assert_eq!(blocked["work"]["stage_id"], "design");
    assert_eq!(project.head("wish-test")["revision"], 3);
    let blocked_head = project.head("wish-test");
    fs::remove_dir_all(project.root.join("cache/xdesign")).unwrap();
    fs::remove_dir_all(project.root.join("cache/wishes")).unwrap();
    let recovered = project.next("wish-test");
    assert_eq!(recovered["store_revision"], 3);
    assert_eq!(recovered["work"]["stage_id"], "design");
    assert_eq!(project.head("wish-test")["revision"], 3);
    assert_eq!(
        fs::read(project.root.join(&report_uri)).unwrap(),
        report_bytes
    );
    project.store(
        &[
            "store",
            "checkout",
            "wish-test",
            "--destination",
            "cache/simple-checkout",
        ],
        0,
    );
    project.verify_checkout(&blocked_head, "cache/simple-checkout");

    let plan_uri = "cache/xdesign/wish-test/plan.json";
    let plan = fs::read(project.root.join(plan_uri)).unwrap();
    project.write(plan_uri, b"{}\n");
    assert!(project
        .stage_error(&["next", "wish-test"])
        .contains("drift"));
    project.write(plan_uri, &plan);
    let run_uri = "cache/xdesign/wish-test/run.json";
    let committed = fs::read(project.root.join(run_uri)).unwrap();
    project.write(run_uri, b"{}\n");
    // The run record is a projection of the committed head: readers repair
    // drift from the store instead of failing on it or adopting the bytes.
    assert_eq!(project.next("wish-test")["store_revision"], 3);
    assert_eq!(fs::read(project.root.join(run_uri)).unwrap(), committed);
    // A writer likewise restarts from the repaired projection instead of
    // wedging the wish on immutable-record drift.
    let repaired = project.stage(&["check", "wish-test"], 1);
    assert_eq!(repaired["store_revision"], 4);
    assert_eq!(
        sha256_bytes(&fs::read(project.root.join(run_uri)).unwrap()),
        project.head("wish-test")["materials"]
            .as_array()
            .unwrap()
            .iter()
            .find(|material| material["uri"] == run_uri)
            .unwrap()["sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(project.head("wish-test")["revision"], 4);
}

#[test]
fn legacy_cache_run_stays_legacy_beside_persistent_items_and_store_root_can_be_overridden() {
    let project = Project::new();
    let persistent = project.init("wish-test", "wish-verify");
    assert_eq!(persistent["store_revision"], 1);

    let source_dir = project.root.join("cache/xdesign/wish-test");
    let legacy_dir = project.root.join("cache/xdesign/wish-legacy");
    fs::create_dir_all(&legacy_dir).unwrap();
    fs::copy(source_dir.join("plan.json"), legacy_dir.join("plan.json")).unwrap();
    project.write("mbse/wishes/wish-legacy.sysml", b"package LegacyWish {}\n");
    let mut work: Value =
        serde_json::from_slice(&fs::read(source_dir.join("work.json")).unwrap()).unwrap();
    for stage in work["stages"].as_array_mut().unwrap() {
        for path in stage["frozen_paths"].as_array_mut().unwrap() {
            if path == "cache/wishes/wish-test.sysml" {
                *path = json!("mbse/wishes/wish-legacy.sysml");
            }
        }
    }
    let work_bytes = serde_json::to_vec_pretty(&work).unwrap();
    project.write("cache/xdesign/wish-legacy/work.json", &work_bytes);
    let mut run: Value =
        serde_json::from_slice(&fs::read(source_dir.join("run.json")).unwrap()).unwrap();
    run["plan_uri"] = json!("cache/xdesign/wish-legacy/plan.json");
    run["wish_model_uri"] = json!("mbse/wishes/wish-legacy.sysml");
    run.as_object_mut()
        .unwrap()
        .remove("artifact_snapshot_directory");
    project.write(
        "cache/xdesign/wish-legacy/run.json",
        &serde_json::to_vec_pretty(&run).unwrap(),
    );
    let mut registry: Value =
        serde_json::from_slice(&fs::read(source_dir.join("registry.json")).unwrap()).unwrap();
    registry["wish_id"] = json!("wish-legacy");
    registry["wish_model_uri"] = json!("mbse/wishes/wish-legacy.sysml");
    registry["plan_uri"] = json!("cache/xdesign/wish-legacy/plan.json");
    registry["work_uri"] = json!("cache/xdesign/wish-legacy/work.json");
    registry["work_sha256"] = json!(sha256_bytes(&work_bytes));
    registry["run_record_uri"] = json!("cache/xdesign/wish-legacy/run.json");
    project.write(
        "cache/xdesign/wish-legacy/registry.json",
        &serde_json::to_vec_pretty(&registry).unwrap(),
    );

    let legacy = project.next("wish-legacy");
    assert!(legacy.get("store_revision").is_none());
    project.write(
        "cache/xdesign/wish-legacy/write.lock",
        b"pid=existing-legacy-writer\n",
    );
    assert!(project
        .stage_error(&["check", "wish-legacy"])
        .contains("busy"));
    fs::remove_file(project.root.join("cache/xdesign/wish-legacy/write.lock")).unwrap();
    let legacy = project.stage(&["check", "wish-legacy"], 1);
    assert!(legacy.get("store_revision").is_none());
    assert!(!project
        .root
        .join(".xdesign/locks/items/wish-legacy.lock")
        .exists());
    project.store(&["store", "get", "wish-legacy"], 3);
    assert_eq!(project.head("wish-test")["revision"], 1);

    let overridden = Project::new();
    let output = overridden.call_with_root(
        &[
            "init",
            "wish-test",
            "--flow",
            "wish-verify",
            "--wish",
            "incoming/wish.sysml",
        ],
        "alternate-store",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(overridden
        .root
        .join("alternate-store/format.json")
        .is_file());
    assert!(!overridden.root.join(".xdesign").exists());
    let output = overridden.call_with_root(&["next", "wish-test"], "alternate-store");
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["store_revision"], 1);
}

#[test]
fn persistent_writer_lock_is_manual_and_next_preserves_working_wish_edits() {
    let project = Project::new();
    let initial = project.init("wish-test", "wish-verify");
    let edited = b"package WorkingWishEdit {}\n";
    project.write("cache/wishes/wish-test.sysml", edited);
    let observed = project.next("wish-test");
    assert_eq!(observed["store_revision"], 1);
    assert_eq!(observed["state_token"], initial["state_token"]);
    assert_eq!(
        observed["work"]["candidate_artifacts"][0]["sha256"],
        sha256_bytes(edited)
    );
    assert_eq!(
        fs::read(project.root.join("cache/wishes/wish-test.sysml")).unwrap(),
        edited
    );

    let lock = project.root.join(".xdesign/locks/items/wish-test.lock");
    fs::write(&lock, b"pid=operator-held-writer\n").unwrap();
    assert_eq!(project.next("wish-test")["store_revision"], 1);
    let error = project.stage_error(&["check", "wish-test", "--operation-id", "held-lock"]);
    assert!(error.contains("busy"));
    assert!(error.contains("operation_id=held-lock"));
    assert!(lock.is_file());
    assert_eq!(project.head("wish-test")["revision"], 1);

    fs::remove_file(&lock).unwrap();
    let blocked = project.stage(&["check", "wish-test", "--operation-id", "after-lock"], 1);
    assert_eq!(blocked["store_revision"], 2);
    assert_eq!(blocked["operation_id"], "after-lock");
    assert!(!lock.exists());
    assert_eq!(project.head("wish-test")["revision"], 2);
    assert_eq!(
        fs::read(project.root.join("cache/wishes/wish-test.sysml")).unwrap(),
        edited
    );
}

#[test]
fn simple_flow_failed_checker_can_commit_a_corrected_snapshot() {
    let project = Project::simple();
    project.init("wish-test", "simple-flow");
    let requirements_uri = "mbse/model/requirements.sysml";
    let original = fs::read(project.root.join(requirements_uri)).unwrap();
    let broken = String::from_utf8(original.clone()).unwrap().replacen(
        "attribute redefines rationale =",
        "attribute redefines omitted_rationale =",
        1,
    );
    project.write(requirements_uri, broken.as_bytes());
    let failed = project.stage(&["check", "wish-test"], 1);
    assert_eq!(failed["store_revision"], 2);
    assert_eq!(failed["assessed"]["assessments"][0]["outcome"], "failed");
    let failed_report = failed["assessed"]["assessments"][0]["evidence_uris"][0]
        .as_str()
        .unwrap()
        .to_owned();

    project.write(requirements_uri, &original);
    let corrected = project.stage(&["check", "wish-test"], 0);
    assert_eq!(corrected["store_revision"], 3);
    assert_eq!(corrected["work"]["stage_id"], "design");
    assert_eq!(corrected["assessed"]["assessments"][0]["outcome"], "passed");
    let corrected_report = corrected["assessed"]["assessments"][0]["evidence_uris"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let head = project.head("wish-test");
    let materials: BTreeMap<_, _> = head["materials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|material| {
            (
                material["uri"].as_str().unwrap().to_owned(),
                material["sha256"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    for report_uri in [&failed_report, &corrected_report] {
        assert!(materials.contains_key(report_uri));
        let report: Value =
            serde_json::from_slice(&fs::read(project.root.join(report_uri)).unwrap()).unwrap();
        let record_uri = report["input_sha256"]
            .as_object()
            .unwrap()
            .keys()
            .find(|uri| uri.ends_with(".record.json"))
            .unwrap();
        assert_eq!(
            materials[record_uri],
            report["input_sha256"][record_uri].as_str().unwrap()
        );
    }
    assert_eq!(
        project.store(&["store", "history", "wish-test"], 0)["data"]["revisions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    fs::create_dir_all(project.root.join("checkout")).unwrap();
    project.store(
        &[
            "store",
            "checkout",
            "wish-test",
            "--destination",
            "checkout/corrected",
        ],
        0,
    );
    project.verify_checkout(&head, "checkout/corrected");
}

#[test]
fn pending_stage_operation_recovers_from_retained_raw_request_without_engine() {
    let project = Project::new();
    let initial = project.init("wish-test", "wish-verify");
    let head = project.head("wish-test");
    let run_uri = "cache/xdesign/wish-test/run.json";
    let run_before = fs::read(project.root.join(run_uri)).unwrap();
    let response_bytes = br#"{"pending_replay":true}"#;
    let mut materials: Vec<_> = head["materials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|material| {
            json!({
                "uri": material["uri"],
                "role": material["role"],
                "sha256": material["sha256"]
            })
        })
        .collect();
    materials.push(json!({
        "uri": "cache/xdesign/wish-test/responses/pending-recovery.json",
        "role": "stage-response",
        "sha256": sha256_bytes(response_bytes),
        "source": "pending-input.json"
    }));
    let mut request = json!({
        "schema_version": 1,
        "operation_id": "pending-recovery",
        "item_id": "wish-test",
        "kind": "wish",
        "expected_revision": 1,
        "metadata": head["metadata"],
        "materials": materials,
        "result": {
            "command": "check",
            "response": {"pending_replay": true},
            "exit_code": 0
        }
    });
    request["metadata"]["stage_attempt"] = json!({
        "base_state_token": initial["state_token"],
        "result_sha256": sha256_bytes(&[]),
        "command": "check"
    });
    project.save_json("pending-request.json", &request);
    let output = project.call(&[
        "store",
        "save",
        "--request",
        "pending-request.json",
        "--project-root",
        ".",
    ]);
    assert_ne!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    let outcome = project.store(&["store", "outcome", "pending-recovery"], 0);
    assert_eq!(outcome["data"]["receipt"]["outcome"], "pending");
    assert_eq!(project.head("wish-test")["revision"], 1);

    project.write("pending-input.json", response_bytes);
    let recovered = project.stage(
        &["check", "wish-test", "--operation-id", "pending-recovery"],
        0,
    );
    assert_eq!(recovered, json!({"pending_replay": true}));
    assert_eq!(project.head("wish-test")["revision"], 2);
    assert_eq!(fs::read(project.root.join(run_uri)).unwrap(), run_before);
    let outcome = project.store(&["store", "outcome", "pending-recovery"], 0);
    assert_eq!(outcome["data"]["receipt"]["outcome"], "committed");
    assert_eq!(outcome["data"]["receipt"]["revision"], 2);
    assert_eq!(project.next("wish-test")["store_revision"], 2);
}

#[test]
fn save_times_out_before_reservation_and_exact_retry_commits() {
    let project = Project::new();
    project.store(&["store", "init"], 0);
    project.write("incoming/material.txt", b"timeout retry material");
    let request = json!({
        "schema_version": 1,
        "operation_id": "timeout-op",
        "item_id": "timeout-item",
        "kind": "artifact",
        "expected_revision": 0,
        "metadata": {},
        "materials": [{
            "uri": "material.txt",
            "role": "artifact",
            "sha256": sha256_bytes(&fs::read(project.root.join("incoming/material.txt")).unwrap()),
            "source": "incoming/material.txt",
        }],
        "result": {"saved": true},
    });
    project.save_json("request.json", &request);

    // A second process holds the SQLite write lock past the store's busy
    // timeout: the save must fail definitively without reserving anything.
    let guard = rusqlite::Connection::open(project.root.join(".xdesign/store.db")).unwrap();
    guard.execute_batch("BEGIN IMMEDIATE;").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xdesign"))
        .current_dir(&project.root)
        .env_remove("XDESIGN_STORE_ROOT")
        .env("XDESIGN_STORE_BUSY_TIMEOUT_MS", "400")
        .args(["store", "save", "--request", "request.json"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(6),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["error"]["code"], "database");
    guard.execute_batch("ROLLBACK").unwrap();
    drop(guard);

    let missing = project.store(&["store", "outcome", "timeout-op"], 3);
    assert_eq!(missing["error"]["code"], "not_found");

    let receipt = project.store(&["store", "save", "--request", "request.json"], 0);
    assert_eq!(receipt["data"]["receipt"]["outcome"], "committed");
    let revision = project.store(&["store", "get", "timeout-item"], 0);
    assert_eq!(revision["data"]["revision"]["revision"], 1);
}

#[test]
fn abandoned_stage_attempt_is_discarded_and_the_check_commits() {
    let project = Project::new();
    project.store(&["store", "init"], 0);
    project.init("wish-abandoned", "wish-verify");
    let blocked = project.stage(&["check", "wish-abandoned"], 1);
    assert_eq!(blocked["store_revision"], 2);

    // Simulate a crashed attempt: speculative engine output leaked into the
    // canonical run record and stale attempt files survived for its operation.
    let run_uri = project.root.join("cache/xdesign/wish-abandoned/run.json");
    let committed = fs::read(&run_uri).unwrap();
    let mut drifted = committed.clone();
    drifted.extend_from_slice(b"\n{\"abandoned\":\"speculative engine output\"}");
    fs::write(&run_uri, &drifted).unwrap();
    project.save_json(
        "cache/xdesign/wish-abandoned/responses/check-abandoned-op.json",
        &json!({"stale": true}),
    );
    fs::write(
        project
            .root
            .join("cache/xdesign/wish-abandoned/attempts/run-abandoned-op.json"),
        &drifted,
    )
    .unwrap();

    let reply = project.stage(
        &["check", "wish-abandoned", "--operation-id", "fresh-op"],
        1,
    );
    assert_eq!(reply["store_revision"], 3);
    assert_eq!(reply["operation_id"], "fresh-op");

    let head = project.head("wish-abandoned");
    let run_material = head["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|material| material["uri"] == "cache/xdesign/wish-abandoned/run.json")
        .unwrap();
    let current = fs::read(&run_uri).unwrap();
    assert_eq!(
        sha256_bytes(&current),
        run_material["sha256"].as_str().unwrap()
    );
    assert!(!project
        .root
        .join("cache/xdesign/wish-abandoned/attempts/run-fresh-op.json")
        .exists());
    let published_response = fs::read(
        project
            .root
            .join("cache/xdesign/wish-abandoned/responses/check-fresh-op.json"),
    )
    .unwrap();
    let published: Value = serde_json::from_slice(&published_response).unwrap();
    assert_eq!(published["operation_id"], "fresh-op");
    assert_eq!(published["store_revision"], 3);
}

#[test]
fn init_retries_after_abandoned_files_and_blocks_on_pending_operations() {
    let project = Project::new();
    project.store(&["store", "init"], 0);

    // Leftovers of an init that never reserved anything are reclaimed.
    let dir = project.root.join("cache/xdesign/wish-retry");
    fs::create_dir_all(dir.join("responses")).unwrap();
    fs::write(dir.join("registry.json"), b"{}").unwrap();
    fs::create_dir_all(project.root.join("cache/wishes")).unwrap();
    fs::copy(
        project.root.join("incoming/wish.sysml"),
        project.root.join("cache/wishes/wish-retry.sysml"),
    )
    .unwrap();
    let initialized = project.init("wish-retry", "wish-verify");
    assert_eq!(initialized["store_revision"], 1);

    // A committed item, not leftover files, is what blocks reinitialization.
    let error = project.stage_error(&[
        "init",
        "wish-retry",
        "--flow",
        "wish-verify",
        "--wish",
        "incoming/wish.sysml",
    ]);
    assert!(error.contains("already initialized"), "{error}");

    // Leftover files of an item with a genuinely pending operation require
    // operator reconciliation before that wish can be reinitialized.
    let request = json!({
        "schema_version": 1,
        "operation_id": "stuck-pending",
        "item_id": "wish-stuck",
        "kind": "wish",
        "expected_revision": 0,
        "metadata": {
            "flow_id": "wish-verify",
            "registry_uri": "cache/xdesign/wish-stuck/registry.json",
            "run_uri": "cache/xdesign/wish-stuck/run.json"
        },
        "materials": [{
            "uri": "cache/xdesign/wish-stuck/registry.json",
            "role": "registration",
            "sha256": sha256_bytes(b"registry"),
            "source": "absent/registry.json"
        }, {
            "uri": "cache/xdesign/wish-stuck/run.json",
            "role": "run-record",
            "sha256": sha256_bytes(b"run"),
            "source": "absent/run.json"
        }],
        "result": {"command": "init", "response": {}, "exit_code": 0}
    });
    project.save_json("stuck-request.json", &request);
    let failed = project.store(&["store", "save", "--request", "stuck-request.json"], 3);
    assert_eq!(failed["error"]["code"], "not_found");
    assert_eq!(
        project.store(&["store", "outcome", "stuck-pending"], 0)["data"]["receipt"]["outcome"],
        "pending"
    );
    let stuck_dir = project.root.join("cache/xdesign/wish-stuck");
    fs::create_dir_all(&stuck_dir).unwrap();
    fs::write(stuck_dir.join("registry.json"), b"registry").unwrap();
    let blocked = project.stage_error(&[
        "init",
        "wish-stuck",
        "--flow",
        "wish-verify",
        "--wish",
        "incoming/wish.sysml",
    ]);
    assert!(blocked.contains("pending operations"), "{blocked}");
}

#[test]
fn committed_wish_verify_check_replays_from_committed_materials_after_cache_deletion() {
    let project = Project::new();
    project.store(&["store", "init"], 0);
    let initial = project.init("wish-replay", "wish-verify");
    assert_eq!(initial["store_revision"], 1);

    // A blocked check advances the wish stage, then the judged identical check
    // commits under a fixed operation id.
    let blocked = project.stage(&["check", "wish-replay", "--operation-id", "blocked-op"], 1);
    assert_eq!(blocked["store_revision"], 2);
    let committed = project.submit("wish-replay", &blocked, &[], "fixed-op", 0);
    assert_eq!(committed["store_revision"], 3);
    assert_eq!(committed["operation_id"], "fixed-op");
    assert_eq!(project.head("wish-replay")["revision"], 3);
    assert_eq!(
        project.store(&["store", "history", "wish-replay"], 0)["data"]["revisions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let result_bytes = fs::read(project.root.join("result.json")).unwrap();

    // Exact replay with the cache present is idempotent even though the
    // attempt scratch was discarded after the original commit.
    let replayed = project.stage(
        &[
            "check",
            "wish-replay",
            "--result",
            "result.json",
            "--operation-id",
            "fixed-op",
        ],
        0,
    );
    assert_eq!(replayed, committed);

    // Capture the committed revision's exact material bytes from the store.
    fs::create_dir_all(project.root.join("checkout")).unwrap();
    project.store(
        &[
            "store",
            "checkout",
            "wish-replay",
            "--revision",
            "3",
            "--destination",
            "checkout/committed",
        ],
        0,
    );
    let run_uri = "cache/xdesign/wish-replay/run.json";
    let response_uri = "cache/xdesign/wish-replay/responses/check-fixed-op.json";
    let committed_bytes =
        |uri: &str| fs::read(project.root.join("checkout/committed").join(uri)).unwrap();
    let committed_run = committed_bytes(run_uri);
    let committed_response = committed_bytes(response_uri);
    let head = project.head("wish-replay");
    let material_sha = |uri: &str| {
        head["materials"]
            .as_array()
            .unwrap()
            .iter()
            .find(|material| material["uri"] == uri)
            .unwrap()["sha256"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    assert_eq!(material_sha(run_uri), sha256_bytes(&committed_run));
    assert_eq!(
        material_sha(response_uri),
        sha256_bytes(&committed_response)
    );
    assert_eq!(
        head["materials"]
            .as_array()
            .unwrap()
            .iter()
            .find(|material| material["uri"] == response_uri)
            .unwrap()["role"],
        "stage-response"
    );

    // The attempt scratch and canonical run/response projections are
    // disposable: remove them all while keeping the CLI result input file.
    let attempts = project.root.join("cache/xdesign/wish-replay/attempts");
    assert!(attempts.is_dir());
    fs::remove_dir_all(&attempts).unwrap();
    fs::remove_file(project.root.join(run_uri)).unwrap();
    fs::remove_dir_all(project.root.join("cache/xdesign/wish-replay/responses")).unwrap();
    assert!(!project.root.join(run_uri).exists());
    assert!(!project.root.join(response_uri).exists());

    // The identical retry must replay the stored response without an engine
    // run, a new revision, or any dependence on the deleted cache.
    let recovered = project.stage(
        &[
            "check",
            "wish-replay",
            "--result",
            "result.json",
            "--operation-id",
            "fixed-op",
        ],
        0,
    );
    assert_eq!(recovered, committed);
    assert_eq!(
        fs::read(project.root.join("result.json")).unwrap(),
        result_bytes
    );
    let outcome = project.store(&["store", "outcome", "fixed-op"], 0);
    assert_eq!(outcome["data"]["receipt"]["outcome"], "committed");
    assert_eq!(outcome["data"]["receipt"]["revision"], 3);
    assert_eq!(outcome["data"]["receipt"]["result"]["response"], committed);
    assert_eq!(outcome["data"]["receipt"]["result"]["exit_code"], 0);
    assert_eq!(project.head("wish-replay")["revision"], 3);
    assert_eq!(
        project.store(&["store", "history", "wish-replay"], 0)["data"]["revisions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(fs::read(project.root.join(run_uri)).unwrap(), committed_run);
    assert_eq!(
        fs::read(project.root.join(response_uri)).unwrap(),
        committed_response
    );

    // The repaired projections keep working for later reads.
    let next = project.next("wish-replay");
    assert_eq!(next["store_revision"], 3);
    assert_eq!(next["state_token"], committed["state_token"]);
    assert_eq!(next["work"]["stage_id"], "verify");
    assert_eq!(fs::read(project.root.join(run_uri)).unwrap(), committed_run);
}

/// N3: crash residue under `locks/items/` must be a doctor finding — it blocks
/// every future `check` of that wish — while a lock whose recorded process is
/// still alive is normal concurrent operation and stays unreported.
#[test]
fn stale_writer_lock_is_a_doctor_finding_while_live_locks_are_not() {
    let project = Project::new();
    project.store(&["store", "init"], 0);
    let locks = project.root.join(".xdesign/locks/items");
    // 999998 exceeds the platform pid range, so it can never be running.
    fs::write(
        locks.join("wish-stale.lock"),
        b"pid=999998 token=8b4e0e90-1111-4111-8111-111111111111\n",
    )
    .unwrap();
    fs::write(
        locks.join("wish-live.lock"),
        format!(
            "pid={} token=8b4e0e90-2222-4222-8222-222222222222\n",
            std::process::id()
        )
        .as_bytes(),
    )
    .unwrap();
    // `Lock::acquire` creates writer locks as owner-only files.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in ["wish-stale.lock", "wish-live.lock"] {
            fs::set_permissions(locks.join(name), fs::Permissions::from_mode(0o600)).unwrap();
        }
    }

    let report = project.store(&["store", "doctor"], 5);
    let findings = report["data"]["report"]["findings"].as_array().unwrap();
    let stale = findings
        .iter()
        .find(|finding| finding["kind"] == "stale_lock")
        .expect("the dead-pid lock must be reported");
    assert_eq!(stale["subject_uri"], "locks/items/wish-stale.lock");
    assert!(
        findings
            .iter()
            .all(|finding| finding["subject_uri"] != "locks/items/wish-live.lock"),
        "a live-pid lock is not a finding: {findings:?}"
    );

    // Removing the named file is the documented operator recovery, and doctor
    // is clean again afterwards.
    fs::remove_file(locks.join("wish-stale.lock")).unwrap();
    let clean = project.store(&["store", "doctor"], 0);
    assert_eq!(
        clean["data"]["report"]["findings"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

/// N4: the closed OS-generated allowlist keeps a browsed store openable — a
/// `.DS_Store` in the root changes no command result and is not a finding —
/// while any other foreign entry still refuses `open`.
#[test]
fn ds_store_in_the_store_root_is_tolerated_and_other_entries_stay_foreign() {
    let project = Project::new();
    project.store(&["store", "init"], 0);
    fs::write(project.root.join(".xdesign/.DS_Store"), b"").unwrap();
    project.store(&["store", "info"], 0);
    let clean = project.store(&["store", "doctor"], 0);
    assert_eq!(
        clean["data"]["report"]["findings"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    fs::write(project.root.join(".xdesign/README.txt"), b"foreign").unwrap();
    let refused = project.store(&["store", "info"], 5);
    assert_eq!(refused["error"]["code"], "unsupported_format");
    fs::remove_file(project.root.join(".xdesign/README.txt")).unwrap();
    project.store(&["store", "info"], 0);

    // A directory that only an OS browsed remains a safe empty root for init.
    let browsed = project.root.join("browsed-store");
    fs::create_dir_all(&browsed).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&browsed, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::write(browsed.join(".DS_Store"), b"").unwrap();
    let initialized = project.store(&["store", "init", "--root", "browsed-store"], 0);
    assert!(initialized["data"]["store_id"].is_string());
}

/// N6: `store request` recovers the exact retained bytes of a pending
/// operation, so the documented "retry the exact request" recovery works even
/// when the original request file is gone.
#[test]
fn store_request_recovers_exact_pending_bytes_for_a_retry() {
    let project = Project::new();
    project.store(&["store", "init"], 0);
    let bytes = b"late-arriving material";
    let request = format!(
        "{{\"schema_version\":1,\"operation_id\":\"late-op\",\"item_id\":\"late-item\",\
         \"kind\":\"artifact\",\"expected_revision\":0,\"metadata\":{{}},\
         \"materials\":[{{\"uri\":\"late.bin\",\"role\":\"artifact\",\"sha256\":\"{}\",\
         \"source\":\"late.bin\"}}],\"result\":{{\"probe\":true}}}}",
        sha256_bytes(bytes)
    );
    project.write("late-request.json", request.as_bytes());
    // The source does not exist yet: the reservation succeeds and the save
    // fails not_found while the operation stays pending.
    let failed = project.store(
        &[
            "store",
            "save",
            "--request",
            "late-request.json",
            "--project-root",
            ".",
        ],
        3,
    );
    assert_eq!(failed["error"]["receipt_state"], "pending");
    assert_eq!(
        project.store(&["store", "outcome", "late-op"], 0)["data"]["receipt"]["outcome"],
        "pending"
    );

    let recovered = project.store(
        &[
            "store",
            "request",
            "late-op",
            "--output",
            "recovered-request.json",
        ],
        0,
    );
    assert_eq!(recovered["data"]["receipt_state"], "pending");
    assert_eq!(
        recovered["data"]["request_sha256"],
        sha256_bytes(request.as_bytes())
    );
    assert_eq!(
        fs::read(project.root.join("recovered-request.json")).unwrap(),
        request.as_bytes(),
        "the recovered file must be byte-identical to the original request"
    );

    project.write("late.bin", bytes);
    let saved = project.store(
        &[
            "store",
            "save",
            "--request",
            "recovered-request.json",
            "--project-root",
            ".",
        ],
        0,
    );
    assert_eq!(saved["data"]["receipt"]["outcome"], "committed");
    assert_eq!(
        project.store(&["store", "get", "late-item"], 0)["data"]["revision"]["revision"],
        1
    );
}
