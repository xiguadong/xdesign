use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::SystemTime,
};
use xdesign::core::{sha256_bytes, snapshot_digest, RsArtifactDigest};

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
        let legacy_wish = "mbse/wishes/wish-test.sysml";
        let working_wish = "cache/wishes/wish-test.sysml";
        let work_uri = "cache/xdesign/wish-test/work.json";
        let mut work: Value =
            serde_json::from_slice(&fs::read(self.root.join(work_uri)).unwrap()).unwrap();
        for stage in work["stages"].as_array_mut().unwrap() {
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
        self.write(run_uri, &serde_json::to_vec_pretty(&run).unwrap());
        let registry_uri = "cache/xdesign/wish-test/registry.json";
        let mut registry: Value =
            serde_json::from_slice(&fs::read(self.root.join(registry_uri)).unwrap()).unwrap();
        registry["wish_model_uri"] = json!(legacy_wish);
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
