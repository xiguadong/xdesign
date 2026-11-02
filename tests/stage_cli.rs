use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::SystemTime,
};
use xdesign::core::{sha256_bytes, snapshot_digest, RsArtifactDigest};
use xdesign::simple_flow::designed_simple_flow_plan;

struct Project {
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let root = std::env::current_dir().unwrap().join(format!(
            "cache/implementation/stage-cli-tests/{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(root.join("mbse/wishes")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("mbse/wishes/wish-test.sysml"),
            "package TestWish {}\n",
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
    fn simple(complete_docs: bool) -> Self {
        fn copy_tree(source: &Path, destination: &Path) {
            fs::create_dir_all(destination).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                let target = destination.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    copy_tree(&entry.path(), &target);
                } else {
                    fs::copy(entry.path(), target).unwrap();
                }
            }
        }
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
        if complete_docs {
            copy_tree(&root.join("docs"), &project.root.join("docs"));
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
    fn json(&self, args: &[&str], code: i32) -> Value {
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
    fn error(&self, args: &[&str]) -> String {
        let output = self.call(args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "args={args:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(output.stdout.is_empty());
        String::from_utf8(output.stderr).unwrap()
    }
    fn legacy_init(&self, flow: &str) -> Value {
        self.json(&["init", "wish-test", "--flow", flow], 0);
        // Recreate the actual legacy contract: no retained bundle and live source paths.
        let plan_uri = "cache/xdesign/wish-test/plan.json";
        let mut plan: Value =
            serde_json::from_slice(&fs::read(self.root.join(plan_uri)).unwrap()).unwrap();
        let commands: Vec<Value> = plan["checkpoints"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .filter_map(|gate| {
                gate.get_mut("checker")
                    .and_then(Value::as_object_mut)
                    .map(|checker| {
                        checker.remove("bundle");
                        checker["command_uri"].clone()
                    })
            })
            .collect();
        let plan_bytes = serde_json::to_vec_pretty(&plan).unwrap();
        let plan_sha256 = sha256_bytes(&plan_bytes);
        self.write(plan_uri, &plan_bytes);
        let legacy_wish = "mbse/wishes/wish-test.sysml";
        let working_wish = "cache/wishes/wish-test.sysml";
        let work_uri = "cache/xdesign/wish-test/work.json";
        let mut work: Value =
            serde_json::from_slice(&fs::read(self.root.join(work_uri)).unwrap()).unwrap();
        for stage in work["stages"].as_array_mut().unwrap() {
            if stage["stage_id"] == "design" {
                stage["frozen_paths"]
                    .as_array_mut()
                    .unwrap()
                    .extend(commands.iter().cloned());
            }
            for path in stage["frozen_paths"].as_array_mut().unwrap() {
                if path == working_wish {
                    *path = json!(legacy_wish);
                }
            }
        }
        let work_bytes = serde_json::to_vec_pretty(&work).unwrap();
        self.write(work_uri, &work_bytes);
        let run_uri = "cache/xdesign/wish-test/run.json";
        let mut run: Value =
            serde_json::from_slice(&fs::read(self.root.join(run_uri)).unwrap()).unwrap();
        run["wish_model_uri"] = json!(legacy_wish);
        run["plan_sha256"] = json!(plan_sha256);
        self.write(run_uri, &serde_json::to_vec_pretty(&run).unwrap());
        let registry_uri = "cache/xdesign/wish-test/registry.json";
        let mut registry: Value =
            serde_json::from_slice(&fs::read(self.root.join(registry_uri)).unwrap()).unwrap();
        registry["wish_model_uri"] = json!(legacy_wish);
        registry["plan_sha256"] = json!(plan_sha256);
        registry["work_sha256"] = json!(sha256_bytes(&work_bytes));
        self.write(registry_uri, &serde_json::to_vec_pretty(&registry).unwrap());
        fs::remove_dir_all(self.root.join(".xdesign")).unwrap();
        fs::remove_dir_all(self.root.join("cache/xdesign/wish-test/responses")).unwrap();
        fs::remove_file(self.root.join(working_wish)).unwrap();
        self.next()
    }
    fn init(&self) -> Value {
        self.legacy_init("wish-verify")
    }
    fn next(&self) -> Value {
        self.json(&["next", "wish-test"], 0)
    }
    fn write(&self, uri: &str, content: &[u8]) {
        let path = self.root.join(uri);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    fn result(&self, response: &Value, artifacts: &[&str], outcome: Option<&str>) -> Value {
        let work = &response["work"];
        let supplied: Vec<_> = artifacts
            .iter()
            .map(|uri| RsArtifactDigest {
                uri: uri.to_string(),
                sha256: sha256_bytes(&fs::read(self.root.join(uri)).unwrap()),
            })
            .collect();
        let mut candidate: BTreeMap<String, RsArtifactDigest> =
            serde_json::from_value::<Vec<RsArtifactDigest>>(work["candidate_artifacts"].clone())
                .unwrap()
                .into_iter()
                .map(|artifact| (artifact.uri.clone(), artifact))
                .collect();
        for artifact in &supplied {
            candidate.insert(artifact.uri.clone(), artifact.clone());
        }
        let mut result = json!({"wish_id": response["wish_id"], "run_id": response["run_id"],
            "stage_id": work["stage_id"], "visit_index": work["visit_index"], "base_revision": work["artifact_revision"],
            "state_token": response["state_token"], "description": "Review retained test evidence", "performed_by": "test-agent", "artifacts": supplied});
        if let Some(outcome) = outcome {
            result["judgment"] = json!({"authority": "agent", "identity": "test-reviewer", "outcome": outcome,
                "rationale": "The fixture evidence meets the explicit test criterion.",
                "artifact_snapshot_digest": snapshot_digest(&candidate.into_values().collect::<Vec<_>>())});
        }
        result
    }
    fn save_result(&self, value: &Value) {
        self.write("result.json", &serde_json::to_vec_pretty(value).unwrap());
    }
    fn submit(
        &self,
        response: &Value,
        artifacts: &[&str],
        outcome: Option<&str>,
        code: i32,
    ) -> Value {
        self.save_result(&self.result(response, artifacts, outcome));
        self.json(&["check", "wish-test", "--result", "result.json"], code)
    }
    fn state(&self) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
        fn visit(root: &Path, path: &Path, output: &mut BTreeMap<PathBuf, (Vec<u8>, SystemTime)>) {
            let Ok(metadata) = fs::metadata(path) else {
                return;
            };
            let bytes = if metadata.is_file() {
                fs::read(path).unwrap()
            } else {
                vec![]
            };
            output.insert(
                path.strip_prefix(root).unwrap().to_owned(),
                (bytes, metadata.modified().unwrap()),
            );
            if metadata.is_dir() {
                for entry in fs::read_dir(path).unwrap() {
                    visit(root, &entry.unwrap().path(), output);
                }
            }
        }
        let mut output = BTreeMap::new();
        visit(&self.root, &self.root.join("cache/xdesign"), &mut output);
        output
    }
}

#[test]
fn process_restart_loop_preserves_assessed_visit_and_read_only_completion() {
    let project = Project::new();
    let initial = project.init();
    assert_eq!(initial["work"]["stage_id"], "wish");
    let state = project.state();
    assert_eq!(initial, project.next());
    assert_eq!(state, project.state());
    let blocked = project.json(&["check", "wish-test"], 1);
    assert_eq!(
        blocked["assessed"]["assessments"][0]["outcome"],
        "unevaluated"
    );
    assert_eq!(blocked["work"]["artifact_revision"], 1);
    let retry = project.json(&["check", "wish-test"], 1);
    assert_eq!(retry["work"]["artifact_revision"], 1);
    let verify = project.submit(&retry, &[], Some("passed"), 0);
    assert_eq!(verify["assessed"]["stage_id"], "wish");
    assert_eq!(verify["work"]["stage_id"], "verify");
    assert_eq!(verify["advance"], "enter_next_stage");
    let empty = project.json(&["check", "wish-test"], 1);
    assert_eq!(empty["work"]["artifact_revision"], 0);
    project.write("evidence.txt", b"first evidence\n");
    let failed = project.submit(&empty, &["evidence.txt"], Some("failed"), 1);
    assert_eq!(failed["assessed"]["assessments"][0]["outcome"], "failed");
    assert_eq!(failed["work"]["artifact_revision"], 1);
    project.write("evidence.txt", b"corrected evidence\n");
    let observed = project.next();
    let done = project.submit(&observed, &[], Some("passed"), 0);
    assert_eq!(done["assessed"]["artifact_revision"], 2);
    assert_eq!(done["completed"], true);
    assert!(done["work"].is_null());
    let state = project.state();
    assert_eq!(project.next()["completed"], true);
    let repeated = project.json(
        &["check", "wish-test", "--result", "does-not-exist.json"],
        0,
    );
    assert_eq!(repeated["completed"], true);
    assert!(repeated["assessed"].is_null());
    assert_eq!(state, project.state());
}

#[test]
fn stale_envelopes_and_changed_judgment_bytes_cannot_mutate_state() {
    let project = Project::new();
    let initial = project.init();
    let old_result = project.result(&initial, &[], Some("passed"));
    project.json(&["check", "wish-test"], 1);
    let state = project.state();
    project.save_result(&old_result);
    assert!(project
        .error(&["check", "wish-test", "--result", "result.json"])
        .contains("stale result"));
    assert_eq!(state, project.state());
    let current = project.next();
    project.save_result(&project.result(&current, &[], Some("passed")));
    project.write("mbse/wishes/wish-test.sysml", b"package RevisedWish {}\n");
    assert!(project
        .error(&["check", "wish-test", "--result", "result.json"])
        .contains("stale judgment"));
    assert_eq!(state, project.state());
    let edited = project.next();
    assert_ne!(
        edited["work"]["candidate_snapshot_digest"],
        current["work"]["candidate_snapshot_digest"]
    );
    project.submit(&edited, &[], Some("passed"), 0);
}

#[test]
fn missing_supplement_remains_blocked_across_retries() {
    let project = Project::new();
    let initial = project.init();
    project.write("answer.txt", b"answer\n");
    let recorded = project.submit(&initial, &["answer.txt"], None, 1);
    fs::remove_file(project.root.join("answer.txt")).unwrap();
    for _ in 0..2 {
        let missing = project.next();
        assert!(missing["work"]["collection_diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("answer.txt")));
        let result = project.submit(&missing, &[], Some("passed"), 1);
        assert_eq!(
            result["assessed"]["assessments"][0]["outcome"],
            "unevaluated"
        );
        assert_eq!(result["work"]["stage_id"], recorded["work"]["stage_id"]);
    }
    project.write("answer.txt", b"restored answer\n");
    project.submit(&project.next(), &[], Some("passed"), 0);
}

#[test]
fn strict_arguments_unknown_wishes_and_exclusive_initialization() {
    let project = Project::new();
    for args in [
        vec!["next", "unknown"],
        vec!["check", "unknown"],
        vec!["init", "../escape", "--flow", "wish-verify"],
        vec!["init", "wish-test", "--flow", "unknown"],
        vec![
            "init",
            "wish-test",
            "--flow",
            "wish-verify",
            "--flow",
            "simple-flow",
        ],
        vec!["next", "wish-test", "extra"],
        vec!["check", "wish-test", "--unknown", "value"],
        vec!["--help", "extra"],
    ] {
        project.error(&args);
        assert!(!project.root.join("cache/xdesign").exists());
    }
    assert!(project.call(&["--help"]).status.success());
    project.init();
    let state = project.state();
    project.error(&["init", "wish-test", "--flow", "wish-verify"]);
    assert_eq!(state, project.state());
}

#[test]
fn malformed_results_unsafe_artifacts_and_foreign_judges_are_rejected() {
    let project = Project::new();
    let initial = project.init();
    project.write("evidence.txt", b"evidence\n");
    let valid = project.result(&initial, &["evidence.txt"], Some("passed"));
    let mut invalids = vec![];
    let mut item = valid.clone();
    item["unexpected"] = json!(true);
    invalids.push(item);
    let mut item = valid.clone();
    item["judgment"]["authority"] = json!("script");
    invalids.push(item);
    let mut item = valid.clone();
    item["judgment"]["rationale"] = json!("");
    invalids.push(item);
    let mut item = valid.clone();
    item["artifacts"][0]["sha256"] = json!("0".repeat(64));
    invalids.push(item);
    let mut item = valid.clone();
    item["artifacts"][0]["uri"] = json!("../escape");
    invalids.push(item);
    let mut item = valid.clone();
    item["artifacts"][0]["uri"] = json!("cache/xdesign/wish-test/run.json");
    invalids.push(item);
    let mut item = valid.clone();
    item["artifacts"][0]["extra"] = json!(true);
    invalids.push(item);
    let mut item = valid.clone();
    item["description"] = json!("");
    invalids.push(item);
    let state = project.state();
    for value in invalids {
        project.save_result(&value);
        project.error(&["check", "wish-test", "--result", "result.json"]);
        assert_eq!(state, project.state());
    }
    project.error(&["check", "wish-test", "--result", "/tmp/result.json"]);
    assert_eq!(state, project.state());
}

#[cfg(unix)]
#[test]
fn symlink_escape_and_state_alias_are_rejected() {
    use std::os::unix::fs::symlink;
    let project = Project::new();
    let initial = project.init();
    project.write("evidence.txt", b"evidence\n");
    let outside = project
        .root
        .parent()
        .unwrap()
        .join(format!("outside-{}.txt", uuid::Uuid::new_v4()));
    fs::write(&outside, b"outside\n").unwrap();
    symlink(&outside, project.root.join("escape.txt")).unwrap();
    symlink(
        project.root.join("cache/xdesign/wish-test/run.json"),
        project.root.join("alias.json"),
    )
    .unwrap();
    let state = project.state();
    for uri in ["escape.txt", "alias.json"] {
        let result = project.result(&initial, &[uri], Some("passed"));
        project.save_result(&result);
        project.error(&["check", "wish-test", "--result", "result.json"]);
        assert_eq!(state, project.state());
    }
}

#[test]
fn frozen_file_drift_corrupt_records_and_busy_writers_do_not_get_repaired() {
    let project = Project::new();
    project.init();
    for uri in [
        "cache/xdesign/wish-test/plan.json",
        "cache/xdesign/wish-test/work.json",
        "cache/xdesign/wish-test/run.json",
    ] {
        let original = fs::read(project.root.join(uri)).unwrap();
        project.write(uri, b"{}\n");
        let corrupted = project.state();
        project.error(&["next", "wish-test"]);
        project.error(&["check", "wish-test"]);
        assert_eq!(corrupted, project.state());
        project.write(uri, &original);
    }
    project.write(
        "cache/xdesign/wish-test/write.lock",
        b"pid=existing-writer\n",
    );
    let state = project.state();
    project.next();
    assert!(project.error(&["check", "wish-test"]).contains("busy"));
    assert_eq!(state, project.state());
}

#[test]
fn real_simple_flow_requirements_checker_retains_immutable_input() {
    let project = Project::simple(false);
    let initial = project.legacy_init("simple-flow");
    assert_eq!(initial["work"]["stage_id"], "requirements");
    assert_eq!(
        initial["work"]["candidate_artifacts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let checked = project.json(&["check", "wish-test"], 0);
    assert_eq!(checked["assessed"]["stage_id"], "requirements");
    assert_eq!(checked["assessed"]["assessments"][0]["judged_by"], "script");
    assert_eq!(checked["work"]["stage_id"], "design");
    let report_uri = checked["assessed"]["assessments"][0]["evidence_uris"][0]
        .as_str()
        .unwrap();
    let report: Value =
        serde_json::from_slice(&fs::read(project.root.join(report_uri)).unwrap()).unwrap();
    let immutable = report["input_sha256"]
        .as_object()
        .unwrap()
        .keys()
        .find(|uri| uri.ends_with(".record.json"))
        .unwrap();
    let bytes = fs::read(project.root.join(immutable)).unwrap();
    assert_eq!(sha256_bytes(&bytes), report["input_sha256"][immutable]);
    let saved: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(saved["visits"][0]["stage_id"], "requirements");
    assert_eq!(saved["visits"][0]["status"], "active");
    assert_ne!(
        bytes,
        fs::read(project.root.join("cache/xdesign/wish-test/run.json")).unwrap()
    );
}

#[test]
fn simple_flow_discovers_and_removes_sources_then_runs_both_terminal_gates() {
    let project = Project::simple(true);
    project.legacy_init("simple-flow");
    project.json(&["check", "wish-test"], 0);
    let implementation = project.json(&["check", "wish-test"], 0);
    assert_eq!(implementation["work"]["stage_id"], "implement");
    project.write("src/added.rs", b"pub fn added() {}\n");
    let trace_uri = "mbse/model/traceability.sysml";
    let original_trace = fs::read(project.root.join(trace_uri)).unwrap();
    let mut edited = original_trace.clone();
    edited.extend_from_slice(b"\n// Deliberate earlier snapshot drift.\n");
    project.write(trace_uri, &edited);
    let blocked = project.json(&["check", "wish-test"], 1);
    assert_eq!(blocked["assessed"]["assessments"][0]["outcome"], "passed");
    assert_eq!(blocked["work"]["stage_id"], "implement");
    assert!(blocked["work"]["candidate_artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["uri"] == "src/added.rs"));
    project.write(trace_uri, &original_trace);
    fs::remove_file(project.root.join("src/added.rs")).unwrap();
    let verify = project.json(&["check", "wish-test"], 0);
    assert_eq!(verify["assessed"]["artifact_revision"], 2);
    assert_eq!(verify["work"]["stage_id"], "verify");
    let record: Value = serde_json::from_slice(
        &fs::read(project.root.join("cache/xdesign/wish-test/run.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        record["visits"][2]["activities"][1]["removed_artifact_uris"],
        json!(["src/added.rs"])
    );
    project.write(
        "verification.txt",
        b"Retained executable fixture results.\n",
    );
    let complete = project.submit(&verify, &["verification.txt"], Some("passed"), 0);
    assert_eq!(complete["completed"], true);
    let assessments = complete["assessed"]["assessments"].as_array().unwrap();
    assert_eq!(assessments.len(), 2);
    assert_eq!(assessments[0]["checkpoint_id"], "simpleRecordAccepted");
    assert_eq!(assessments[0]["judged_by"], "script");
    assert_eq!(assessments[1]["checkpoint_id"], "verificationAccepted");
    assert_eq!(assessments[1]["judged_by"], "agent");
}

#[test]
fn simultaneous_check_processes_cannot_overwrite_each_other() {
    let project = Project::simple(false);
    let script_uri = "tools/checks/simple_requirements.py";
    let script = fs::read_to_string(project.root.join(script_uri)).unwrap();
    let instrumented = script.replace("if __name__ == \"__main__\":", "if __name__ == \"__main__\":\n    import sys, time\n    from common import ROOT\n    if '--list-inputs' not in sys.argv:\n        (ROOT / 'cache' / 'checker-entered').write_text('ready')\n        while not (ROOT / 'cache' / 'checker-release').exists():\n            time.sleep(0.01)");
    project.write(script_uri, instrumented.as_bytes());
    project.legacy_init("simple-flow");
    let mut first = Command::new(env!("CARGO_BIN_EXE_xdesign"))
        .current_dir(&project.root)
        .args(["check", "wish-test"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    for _ in 0..500 {
        if project.root.join("cache/checker-entered").exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if !project.root.join("cache/checker-entered").exists() {
        first.kill().unwrap();
        panic!("first process never reached the controlled checker barrier");
    }
    let state = project.state();
    project.next();
    assert!(project.error(&["check", "wish-test"]).contains("busy"));
    assert_eq!(state, project.state());
    project.write("cache/checker-release", b"release");
    let output = first.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(project.next()["work"]["stage_id"], "design");
    assert!(!project
        .root
        .join("cache/xdesign/wish-test/write.lock")
        .exists());
}

#[test]
fn init_rejects_agent_cli_checks_and_default_plan_remains_stable() {
    let project = Project::simple(false);
    project.write(
        "checks.json",
        br#"{"schema_version":1,"checks":[{"checkpoint_id":"simpleRequirementsAccepted","mode":"cli","cli":"codex","model":"gpt-test-model"}]}"#,
    );
    let error = project.error(&[
        "init",
        "wish-test",
        "--flow",
        "simple-flow",
        "--wish",
        "mbse/wishes/wish-test.sysml",
        "--checks",
        "checks.json",
    ]);
    assert!(error.to_ascii_lowercase().contains("cli"), "{error}");
    assert!(!project
        .root
        .join("cache/xdesign/wish-test/plan.json")
        .exists());

    // A run without --checks adds resource retention while preserving the logical
    // default plan and deterministic script authority.
    project.write("mbse/wishes/wish-plain.sysml", b"package PlainWish {}\n");
    project.json(&["init", "wish-plain", "--flow", "simple-flow"], 0);
    let expected_dir = format!("cache/implementation/rust-tests/{}", uuid::Uuid::new_v4());
    fs::create_dir_all(&expected_dir).unwrap();
    let mut expected = designed_simple_flow_plan();
    expected
        .freeze(format!("{expected_dir}/plan.json"))
        .unwrap();
    let expected_bytes = fs::read(format!("{expected_dir}/plan.json")).unwrap();
    let frozen = fs::read(project.root.join("cache/xdesign/wish-plain/plan.json")).unwrap();
    let mut logical: Value = serde_json::from_slice(&frozen).unwrap();
    for gate in logical["checkpoints"].as_array_mut().unwrap() {
        if let Some(checker) = gate["checker"].as_object_mut() {
            assert_eq!(checker["bundle"]["source"], "builtin-v1");
            checker.remove("bundle");
        }
    }
    assert_eq!(
        logical,
        serde_json::from_slice::<Value>(&expected_bytes).unwrap(),
        "new executor retention must preserve the logical default gates and rules"
    );
    assert!(!String::from_utf8_lossy(&frozen).contains("\"custom\""));
    let plain = project.json(&["check", "wish-plain"], 0);
    assert_eq!(plain["assessed"]["assessments"][0]["judged_by"], "script");
    assert_eq!(
        plain["assessed"]["assessments"][0]["assessor"],
        "simpleRequirementsCheck"
    );

    // Every --checks argument combination reaches the binding reader, and an
    // invalid binding config is rejected with a clear message.
    for args in [
        vec![
            "init",
            "wish-combo-1",
            "--flow",
            "simple-flow",
            "--checks",
            "missing-bindings.json",
        ],
        vec![
            "init",
            "wish-combo-2",
            "--flow",
            "simple-flow",
            "--wish",
            "mbse/wishes/wish-test.sysml",
            "--checks",
            "missing-bindings.json",
        ],
        vec![
            "init",
            "wish-combo-3",
            "--flow",
            "simple-flow",
            "--checks",
            "missing-bindings.json",
            "--wish",
            "mbse/wishes/wish-test.sysml",
        ],
    ] {
        let error = project.error(&args);
        assert!(error.contains("missing-bindings.json"), "{error}");
    }
    project.write(
        "bad-checks.json",
        br#"{"schema_version":1,"checks":[{"checkpoint_id":"verificationAccepted","mode":"default"}]}"#,
    );
    let error = project.error(&[
        "init",
        "wish-combo-4",
        "--flow",
        "simple-flow",
        "--checks",
        "bad-checks.json",
    ]);
    assert!(
        error.contains("unknown checkpoint in check bindings"),
        "{error}"
    );
    assert!(!project.root.join("cache/xdesign/wish-combo-4").exists());
}

#[test]
fn init_checks_binds_a_custom_command_script_and_check_uses_it() {
    let project = Project::simple(false);
    project.write(
        "cache/custom_requirements.py",
        b"import subprocess\nimport sys\nraise SystemExit(subprocess.call([sys.executable, 'tools/checks/simple_requirements.py', *sys.argv[1:]]))\n",
    );
    project.write(
        "command-checks.json",
        br#"{"schema_version":1,"checks":[{"checkpoint_id":"simpleRequirementsAccepted","mode":"command","command_uri":"cache/custom_requirements.py"}]}"#,
    );
    project.json(
        &[
            "init",
            "wish-test",
            "--flow",
            "simple-flow",
            "--checks",
            "command-checks.json",
        ],
        0,
    );
    let plan: Value = serde_json::from_slice(
        &fs::read(project.root.join("cache/xdesign/wish-test/plan.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        plan["checkpoints"][0]["checker"]["custom"],
        json!({"runner": "process"})
    );
    assert_eq!(
        plan["checkpoints"][0]["checker"]["command_uri"],
        "cache/custom_requirements.py"
    );
    let checked = project.json(&["check", "wish-test"], 0);
    let assessment = &checked["assessed"]["assessments"][0];
    assert_eq!(assessment["outcome"], "passed");
    assert_eq!(assessment["judged_by"], "script");
    assert_eq!(assessment["assessor"], "simpleRequirementsCheck");
    assert_eq!(checked["work"]["stage_id"], "design");
    assert_eq!(project.next()["work"]["stage_id"], "design");
}

fn checkpoint_entry_json(id: &str, kind: &str) -> Value {
    json!({
        "checkpoint_id": id,
        "title": format!("Checkpoint {id}"),
        "description": format!("Library checkpoint {id} used by the CLI integration tests."),
        "kind": kind,
        "tags": ["test"],
        "version": 1,
        "created_at": "2026-10-26T00:00:00Z",
        "updated_at": "2026-10-26T00:00:00Z",
        "source": "manual",
        "history": [],
    })
}

fn syntactic_library_entry(id: &str, script_uri: &str, input_uri: &str, rule_id: &str) -> Value {
    let mut entry = checkpoint_entry_json(id, "syntactic");
    entry["rules"] = json!([{
        "rule_id": rule_id,
        "condition": format!("The declared rule {rule_id} holds for the recorded revision."),
        "reads": [input_uri],
    }]);
    entry["inputs"] = json!([input_uri]);
    entry["executor"] = json!({"mode": "script", "command_uri": script_uri});
    entry
}

fn cli_library_entry(id: &str, cli: &str, model: &str, input_uri: &str, rule_id: &str) -> Value {
    let mut entry = syntactic_library_entry(id, "", input_uri, rule_id);
    entry["executor"] = json!({"mode": "cli", "cli": cli, "model": model});
    entry
}

fn semantic_library_entry(id: &str, criteria: &[&str], agent: bool, human: bool) -> Value {
    let mut entry = checkpoint_entry_json(id, "semantic");
    entry["criteria"] = json!(criteria);
    entry["judges"] = json!({"agent": agent, "human": human});
    entry
}

fn library_binding(checkpoint_id: &str, library_id: &str) -> Value {
    json!({
        "checkpoint_id": checkpoint_id,
        "mode": "library",
        "library_id": library_id,
    })
}

impl Project {
    fn library(&self, entries: &[Value]) {
        self.write(
            "checkpoints/library.json",
            &serde_json::to_vec_pretty(&json!({
                "schema_version": 1,
                "checkpoints": entries,
            }))
            .unwrap(),
        );
    }
    fn save_checkpoint(&self, request: &Value, code: i32) -> Value {
        self.write(
            "checkpoint-request.json",
            &serde_json::to_vec_pretty(request).unwrap(),
        );
        self.json(
            &["checkpoint", "save", "--request", "checkpoint-request.json"],
            code,
        )
    }
    fn init_with_checks(&self, id: &str, checks: &[Value]) -> Value {
        self.write(
            "checks.json",
            &serde_json::to_vec_pretty(&json!({
                "schema_version": 1,
                "checks": checks,
            }))
            .unwrap(),
        );
        self.json(
            &[
                "init",
                id,
                "--flow",
                "simple-flow",
                "--checks",
                "checks.json",
            ],
            0,
        )
    }
    fn plan_json(&self, id: &str) -> Value {
        serde_json::from_slice(
            &fs::read(self.root.join(format!("cache/xdesign/{id}/plan.json"))).unwrap(),
        )
        .unwrap()
    }
}

#[test]
fn checkpoint_library_cli_lists_shows_and_saves_round_trip() {
    let project = Project::simple(false);
    // A missing library.json is the built-ins and zero user entries.
    let list = project.json(&["checkpoint", "list"], 0);
    assert_eq!(list["schema_version"], 1);
    assert_eq!(list["library_uri"], "checkpoints/library.json");
    assert_eq!(list["issues"], json!([]));
    let builtins = list["checkpoints"].as_array().unwrap();
    assert_eq!(builtins.len(), 5);
    let ids: Vec<&str> = builtins
        .iter()
        .map(|entry| entry["checkpoint_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec![
            "builtin-simple-requirements",
            "builtin-simple-design",
            "builtin-simple-implementation",
            "builtin-simple-record",
            "builtin-simple-verification"
        ]
    );
    assert!(builtins.iter().all(|entry| entry["builtin"] == true));
    assert_eq!(builtins[0]["rule_count"], 2);
    assert_eq!(builtins[0]["checker_id"], "simpleRequirementsCheck");
    assert_eq!(builtins[4]["criteria_count"], 2);
    assert_eq!(builtins[4]["checker_id"], Value::Null);
    let show = project.json(&["checkpoint", "show", "builtin-simple-requirements"], 0);
    assert_eq!(show["checkpoint"]["builtin"], true);
    assert_eq!(show["checkpoint"]["script_exists"], true);
    assert_eq!(show["checkpoint"]["rules"].as_array().unwrap().len(), 2);
    assert_eq!(show["checkpoint"]["checker_id"], "simpleRequirementsCheck");
    let unknown = project.json(&["checkpoint", "show", "does-not-exist"], 2);
    assert_eq!(unknown["ok"], false);
    assert_eq!(unknown["error"]["code"], "unknown_checkpoint");
    assert!(unknown["error"]["message"]
        .as_str()
        .unwrap()
        .contains("does-not-exist"));

    // Save a manual syntactic entry, then update it.
    let entry = syntactic_library_entry(
        "energy-budget-positive",
        "tools/checks/simple_requirements.py",
        "mbse/model/requirements.sysml",
        "budget-declared",
    );
    let saved = project.save_checkpoint(&entry, 0);
    assert_eq!(saved["created"], true);
    assert_eq!(saved["version"], 1);
    assert_eq!(saved["checkpoint"]["builtin"], false);
    assert_eq!(saved["script_uri"], "tools/checks/simple_requirements.py");
    let mut updated = entry.clone();
    updated["title"] = json!("Energy budget stays non-negative (updated)");
    let saved = project.save_checkpoint(&updated, 0);
    assert_eq!(saved["created"], false);
    assert_eq!(saved["version"], 2);
    let show = project.json(&["checkpoint", "show", "energy-budget-positive"], 0);
    assert_eq!(show["checkpoint"]["builtin"], false);
    assert_eq!(show["checkpoint"]["version"], 2);
    assert_eq!(show["checkpoint"]["source"], "manual");
    assert_eq!(show["checkpoint"]["history"].as_array().unwrap().len(), 1);
    let list = project.json(&["checkpoint", "list"], 0);
    let checkpoints = list["checkpoints"].as_array().unwrap();
    assert_eq!(checkpoints.len(), 6);
    let user = checkpoints
        .iter()
        .find(|entry| entry["checkpoint_id"] == "energy-budget-positive")
        .unwrap();
    assert_eq!(user["builtin"], false);
    assert_eq!(user["version"], 2);
    assert_eq!(user["rule_count"], 1);
    assert_eq!(user["criteria_count"], 0);
    assert_eq!(
        user["executor"],
        json!({"mode": "script", "command_uri": "tools/checks/simple_requirements.py"})
    );

    // Semantic entries cannot claim a checker identity.
    let mut semantic_with_id = semantic_library_entry(
        "semantic-with-identity",
        &["The evidence stands on its own."],
        true,
        true,
    );
    semantic_with_id["checker_id"] = json!("someChecker");
    let error = project.save_checkpoint(&semantic_with_id, 2);
    assert_eq!(error["error"]["code"], "invalid_checkpoint");
    // A syntactic entry may declare the identity its reused script reports.
    let mut reused = entry.clone();
    reused["checkpoint_id"] = json!("requirements-reuse");
    reused["checker_id"] = json!("simpleRequirementsCheck");
    let saved = project.save_checkpoint(&reused, 0);
    assert_eq!(saved["checkpoint"]["checker_id"], "simpleRequirementsCheck");
    let show = project.json(&["checkpoint", "show", "requirements-reuse"], 0);
    assert_eq!(show["checkpoint"]["checker_id"], "simpleRequirementsCheck");

    // A built-in id cannot be mutated, but it can be shown.
    let mut builtin_attempt = entry.clone();
    builtin_attempt["checkpoint_id"] = json!("builtin-simple-requirements");
    let error = project.save_checkpoint(&builtin_attempt, 2);
    assert_eq!(error["ok"], false);
    assert_eq!(error["error"]["code"], "builtin_checkpoint");

    // Invalid definitions carry the stable invalid_checkpoint code.
    let mut invalid = entry.clone();
    invalid["rules"] = json!([]);
    let error = project.save_checkpoint(&invalid, 2);
    assert_eq!(error["error"]["code"], "invalid_checkpoint");
    let mut invalid = entry.clone();
    invalid["kind"] = json!("fuzzy");
    let error = project.save_checkpoint(&invalid, 2);
    assert_eq!(error["error"]["code"], "invalid_checkpoint");
    project.write("broken-request.json", b"not json");
    let error = project.json(
        &["checkpoint", "save", "--request", "broken-request.json"],
        2,
    );
    assert_eq!(error["error"]["code"], "invalid_checkpoint");

    // A malformed library file is an explicit error, never guessed.
    project.write("checkpoints/library.json", b"{ not json");
    let error = project.json(&["checkpoint", "list"], 2);
    assert_eq!(error["error"]["code"], "invalid_checkpoint");
    fs::remove_file(project.root.join("checkpoints/library.json")).unwrap();
    let list = project.json(&["checkpoint", "list"], 0);
    assert_eq!(list["checkpoints"].as_array().unwrap().len(), 5);
}

#[test]
fn scaffold_save_writes_a_protocol_compatible_checker_script() {
    let project = Project::simple(false);
    let mut entry = syntactic_library_entry(
        "energy-budget-positive",
        "checkpoints/scripts/energy_budget_positive.py",
        "mbse/model/requirements.sysml",
        "budget-declared",
    );
    entry["rules"].as_array_mut().unwrap().push(json!({
        "rule_id": "budget-non-negative",
        "condition": "Every declared energyBudget is >= 0.",
        "reads": ["mbse/model/requirements.sysml"],
    }));
    entry["create_script"] = json!("scaffold");
    let saved = project.save_checkpoint(&entry, 0);
    assert_eq!(saved["created"], true);
    assert_eq!(
        saved["script_uri"],
        "checkpoints/scripts/energy_budget_positive.py"
    );
    let script_uri = saved["script_uri"].as_str().unwrap();
    let script = fs::read_to_string(project.root.join(script_uri)).unwrap();
    assert!(script.starts_with("#!/usr/bin/env python3"));
    assert!(script.contains("rule not implemented: budget-declared"));
    assert!(script.contains("rule not implemented: budget-non-negative"));
    assert!(script.contains("--list-inputs"));
    assert!(script.contains("input_sha256"));
    assert!(script.contains("snapshot_digest"));
    let output = Command::new("python3")
        .current_dir(&project.root)
        .arg(script_uri)
        .arg("--list-inputs")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let declared: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(declared, vec!["mbse/model/requirements.sysml"]);
    // The generated entry is listable and shows its generated source.
    let list = project.json(&["checkpoint", "list"], 0);
    let generated = list["checkpoints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["checkpoint_id"] == "energy-budget-positive")
        .unwrap()
        .clone();
    assert_eq!(generated["rule_count"], 2);
    let show = project.json(&["checkpoint", "show", "energy-budget-positive"], 0);
    assert_eq!(show["checkpoint"]["source"], "generated");
}

#[test]
fn init_with_scaffolded_script_library_binding_runs_the_script() {
    let project = Project::simple(false);
    let entry = syntactic_library_entry(
        "requirements-library",
        "checkpoints/scripts/requirements_library.py",
        "mbse/model/requirements.sysml",
        "requirements-implemented",
    );
    let mut entry = entry;
    entry["checker_id"] = json!("requirementsLibraryIdentity");
    entry["create_script"] = json!("scaffold");
    let saved = project.save_checkpoint(&entry, 0);
    let script_uri = saved["script_uri"].as_str().unwrap().to_owned();
    // The authored checkpoint implements its rule; the frozen library entry
    // reuses that existing project script without create_script.
    let script = fs::read_to_string(project.root.join(&script_uri)).unwrap();
    let script = script.replace(
        "return False, [\"rule not implemented: requirements-implemented\"]",
        "return True, []",
    );
    assert!(!script.contains("rule not implemented: requirements-implemented"));
    project.write(&script_uri, script.as_bytes());
    project.init_with_checks(
        "wish-test",
        &[library_binding(
            "simpleRequirementsAccepted",
            "requirements-library",
        )],
    );
    let plan = project.plan_json("wish-test");
    let checkpoint = &plan["checkpoints"][0];
    assert_eq!(checkpoint["library"], "requirements-library");
    let checker = &checkpoint["checker"];
    assert_eq!(checker["checker_id"], "requirementsLibraryIdentity");
    assert_eq!(checker["command_uri"], script_uri);
    assert_eq!(checker["custom"], json!({"runner": "process"}));
    assert_eq!(checker["rules"][0]["rule_id"], "requirements-implemented");
    let work: Value = serde_json::from_slice(
        &fs::read(project.root.join("cache/xdesign/wish-test/work.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        work["stages"][0]["frozen_paths"],
        json!(["mbse/model/requirements.sysml"]),
        "library entry inputs replace the --list-inputs invocation"
    );
    let checked = project.json(&["check", "wish-test"], 0);
    let assessment = &checked["assessed"]["assessments"][0];
    assert_eq!(assessment["outcome"], "passed");
    assert_eq!(assessment["judged_by"], "script");
    assert_eq!(assessment["assessor"], "requirementsLibraryIdentity");
    assert_eq!(checked["advance"], "enter_next_stage");
    assert_eq!(checked["work"]["stage_id"], "design");
    let report_uri = assessment["evidence_uris"][0].as_str().unwrap();
    let report: Value =
        serde_json::from_slice(&fs::read(project.root.join(report_uri)).unwrap()).unwrap();
    assert_eq!(report["checker_id"], "requirementsLibraryIdentity");
    assert_eq!(report["overall_verdict"], "passed");
    assert!(report["input_sha256"].as_object().unwrap().contains_key(
        project.plan_json("wish-test")["checkpoints"][0]["checker"]["bundle"]["entry_uri"]
            .as_str()
            .unwrap()
    ));
    // next reloads the frozen library plan from the store.
    assert_eq!(project.next()["work"]["stage_id"], "design");
}

#[test]
fn init_with_unedited_scaffold_library_binding_reports_honest_failure() {
    let project = Project::simple(false);
    let mut entry = syntactic_library_entry(
        "honest-scaffold",
        "checkpoints/scripts/honest_scaffold.py",
        "mbse/model/requirements.sysml",
        "not-yet-implemented",
    );
    entry["create_script"] = json!("scaffold");
    project.save_checkpoint(&entry, 0);
    project.init_with_checks(
        "wish-test",
        &[library_binding(
            "simpleRequirementsAccepted",
            "honest-scaffold",
        )],
    );
    let blocked = project.json(&["check", "wish-test"], 1);
    let assessment = &blocked["assessed"]["assessments"][0];
    assert_eq!(assessment["outcome"], "failed");
    assert_eq!(assessment["judged_by"], "script");
    assert_eq!(assessment["assessor"], "honest-scaffold");
    assert!(assessment["rationale"]
        .as_str()
        .unwrap()
        .contains("rule not implemented: not-yet-implemented"));
    assert_eq!(blocked["work"]["stage_id"], "requirements");
    assert_eq!(blocked["advance"], "remain_blocked");
}

#[test]
fn checkpoint_library_rejects_agent_cli_executor() {
    let project = Project::simple(false);
    let entry = cli_library_entry(
        "agent-requirements-library",
        "codex",
        "gpt-library-model",
        "mbse/model/requirements.sysml",
        "requirements-agent-rule",
    );
    let response = project.save_checkpoint(&entry, 2);
    assert_eq!(response["ok"], false);
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .to_ascii_lowercase()
        .contains("cli"));
    assert!(!project.root.join("checkpoints/library.json").exists());
}

#[test]
fn existing_cli_library_entry_is_readable_but_cannot_bind_a_new_run() {
    let project = Project::simple(false);
    let entry = cli_library_entry(
        "legacy-agent-check",
        "codex",
        "legacy-model",
        "mbse/model/requirements.sysml",
        "legacy-rule",
    );
    project.library(&[entry]);
    let shown = project.json(&["checkpoint", "show", "legacy-agent-check"], 0);
    assert_eq!(shown["checkpoint"]["executor"]["mode"], "cli");
    project.write(
        "checks.json",
        &serde_json::to_vec(&json!({
            "schema_version": 1,
            "checks": [library_binding("simpleRequirementsAccepted", "legacy-agent-check")]
        }))
        .unwrap(),
    );
    let error = project.error(&[
        "init",
        "wish-test",
        "--flow",
        "simple-flow",
        "--checks",
        "checks.json",
    ]);
    assert!(error.to_ascii_lowercase().contains("cli"), "{error}");
    assert!(!project
        .root
        .join("cache/xdesign/wish-test/plan.json")
        .exists());
}

#[test]
fn init_with_semantic_library_binding_freezes_criteria_and_judges() {
    let project = Project::simple(false);
    let entry = semantic_library_entry(
        "verification-evidence-complete",
        &[
            "Every requirement has retained evidence that a reader can re-examine.",
            "The evidence and the record stand on their own.",
        ],
        true,
        false,
    );
    project.save_checkpoint(&entry, 0);
    project.init_with_checks(
        "wish-test",
        &[library_binding(
            "verificationAccepted",
            "verification-evidence-complete",
        )],
    );
    let plan = project.plan_json("wish-test");
    let semantic = &plan["checkpoints"][4];
    assert_eq!(semantic["checkpoint_id"], "verificationAccepted");
    assert_eq!(semantic["library"], "verification-evidence-complete");
    assert_eq!(semantic["checker"], Value::Null);
    assert_eq!(semantic["agent_may_judge"], true);
    assert_eq!(semantic["human_may_judge"], false);
    assert_eq!(
        semantic["criteria"],
        json!([
            "Every requirement has retained evidence that a reader can re-examine.",
            "The evidence and the record stand on their own.",
        ])
    );
    // The frozen plan reloads and validates through next.
    assert_eq!(project.next()["work"]["stage_id"], "requirements");

    // Kind mismatch, unknown id, invalid id and unsupported-flow bindings fail.
    let mut wrong_kind = entry.clone();
    wrong_kind["checkpoint_id"] = json!("syntactic-lookalike");
    wrong_kind["kind"] = json!("syntactic");
    wrong_kind.as_object_mut().unwrap().remove("criteria");
    wrong_kind.as_object_mut().unwrap().remove("judges");
    wrong_kind["rules"] = json!([{
        "rule_id": "rule-one",
        "condition": "The rule holds.",
        "reads": ["mbse/model/requirements.sysml"],
    }]);
    wrong_kind["inputs"] = json!(["mbse/model/requirements.sysml"]);
    wrong_kind["executor"] =
        json!({"mode": "script", "command_uri": "tools/checks/simple_requirements.py"});
    project.save_checkpoint(&wrong_kind, 0);
    let mut checks = json!([library_binding(
        "verificationAccepted",
        "syntactic-lookalike"
    ),]);
    project.write(
        "checks.json",
        &serde_json::to_vec_pretty(&json!({"schema_version": 1, "checks": checks.clone()}))
            .unwrap(),
    );
    let error = project.error(&[
        "init",
        "wish-mismatch",
        "--flow",
        "simple-flow",
        "--checks",
        "checks.json",
    ]);
    assert!(error.contains("kind mismatch"), "{error}");
    checks[0]["library_id"] = json!("missing-library");
    project.write(
        "checks.json",
        &serde_json::to_vec_pretty(&json!({"schema_version": 1, "checks": checks})).unwrap(),
    );
    let error = project.error(&[
        "init",
        "wish-missing",
        "--flow",
        "simple-flow",
        "--checks",
        "checks.json",
    ]);
    assert!(error.contains("unknown library checkpoint"), "{error}");
    // wish-verify has no library binding in this round.
    project.write("mbse/wishes/wish-onlyverify.sysml", b"package Wish {}\n");
    project.write(
        "checks.json",
        &serde_json::to_vec_pretty(&json!({
            "schema_version": 1,
            "checks": [library_binding("verificationAccepted", "verification-evidence-complete")],
        }))
        .unwrap(),
    );
    let error = project.error(&[
        "init",
        "wish-onlyverify",
        "--flow",
        "wish-verify",
        "--checks",
        "checks.json",
    ]);
    assert!(
        error.contains("custom checks are supported for simple-flow only"),
        "{error}"
    );
}

#[test]
fn init_without_checks_retains_builtins_and_keeps_the_default_logical_plan() {
    let project = Project::simple(false);
    project.json(&["init", "wish-test", "--flow", "simple-flow"], 0);
    let plan = project.plan_json("wish-test");
    for gate in plan["checkpoints"].as_array().unwrap() {
        if gate["checker"].is_object() {
            assert_eq!(gate["checker"]["bundle"]["source"], "builtin-v1");
            assert!(gate["checker"].get("custom").is_none());
        }
        assert!(gate.get("library").is_none());
    }
}

#[test]
fn init_with_reused_shipped_checker_library_binding_passes() {
    let project = Project::simple(false);
    let default_plan = designed_simple_flow_plan();
    let requirements_rules = default_plan.checkpoints[0]
        .checker
        .as_ref()
        .unwrap()
        .rules
        .clone();
    let mut entry = checkpoint_entry_json("requirements-reuse", "syntactic");
    entry["rules"] = serde_json::to_value(&requirements_rules).unwrap();
    entry["inputs"] = json!([
        "mbse/model/requirements.sysml",
        "mbse/model/traceability.sysml",
    ]);
    entry["executor"] = json!({
        "mode": "script",
        "command_uri": "tools/checks/simple_requirements.py",
        "files": ["tools/checks/common.py"],
    });
    entry["checker_id"] = json!("simpleRequirementsCheck");
    project.save_checkpoint(&entry, 0);
    project.init_with_checks(
        "wish-test",
        &[library_binding(
            "simpleRequirementsAccepted",
            "requirements-reuse",
        )],
    );
    let plan = project.plan_json("wish-test");
    let checkpoint = &plan["checkpoints"][0];
    assert_eq!(checkpoint["library"], "requirements-reuse");
    assert_eq!(
        checkpoint["checker"]["checker_id"],
        "simpleRequirementsCheck"
    );
    assert_eq!(
        checkpoint["checker"]["command_uri"],
        "tools/checks/simple_requirements.py"
    );
    assert_eq!(
        checkpoint["checker"]["custom"],
        json!({"runner": "process"})
    );
    let work: Value = serde_json::from_slice(
        &fs::read(project.root.join("cache/xdesign/wish-test/work.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        work["stages"][0]["frozen_paths"],
        json!([
            "mbse/model/requirements.sysml",
            "mbse/model/traceability.sysml",
        ])
    );
    let checked = project.json(&["check", "wish-test"], 0);
    let assessment = &checked["assessed"]["assessments"][0];
    assert_eq!(assessment["outcome"], "passed");
    assert_eq!(assessment["judged_by"], "script");
    assert_eq!(assessment["assessor"], "simpleRequirementsCheck");
    assert_eq!(checked["advance"], "enter_next_stage");
    assert_eq!(checked["work"]["stage_id"], "design");
    let report_uri = assessment["evidence_uris"][0].as_str().unwrap();
    let report: Value =
        serde_json::from_slice(&fs::read(project.root.join(report_uri)).unwrap()).unwrap();
    assert_eq!(report["checker_id"], "simpleRequirementsCheck");
    assert_eq!(report["overall_verdict"], "passed");
    // The frozen plan reloads from the store on next and keeps the reused
    // script identity while checkpoint.library keeps its provenance.
    assert_eq!(project.next()["work"]["stage_id"], "design");
    let reloaded_plan = project.plan_json("wish-test");
    assert_eq!(
        reloaded_plan["checkpoints"][0]["library"],
        "requirements-reuse"
    );
    assert_eq!(
        reloaded_plan["checkpoints"][0]["checker"]["checker_id"],
        "simpleRequirementsCheck"
    );

    // The built-in syntactic seed reuses the shipped checker identity too.
    project.write("mbse/wishes/wish-builtin.sysml", b"package Wish {}\n");
    project.init_with_checks(
        "wish-builtin",
        &[library_binding(
            "simpleRequirementsAccepted",
            "builtin-simple-requirements",
        )],
    );
    let builtin_plan = project.plan_json("wish-builtin");
    assert_eq!(
        builtin_plan["checkpoints"][0]["library"],
        "builtin-simple-requirements"
    );
    assert_eq!(
        builtin_plan["checkpoints"][0]["checker"]["checker_id"],
        "simpleRequirementsCheck"
    );
    let builtin_checked = project.json(&["check", "wish-builtin"], 0);
    assert_eq!(
        builtin_checked["assessed"]["assessments"][0]["outcome"],
        "passed"
    );
    assert_eq!(
        builtin_checked["assessed"]["assessments"][0]["assessor"],
        "simpleRequirementsCheck"
    );
}
