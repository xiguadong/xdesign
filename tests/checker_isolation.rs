//! Process acceptance for release resources and project execution (ISO-01..09).
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use xdesign::core::{sha256_bytes, snapshot_digest, RsArtifactDigest};

struct Project {
    root: PathBuf,
    binary: PathBuf,
}
impl Project {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cache/checker-isolation/cases")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(root.join("installation")).unwrap();
        let source = std::env::var_os("XDESIGN_ACCEPTANCE_BINARY")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_xdesign")));
        let binary = root.join("installation/xdesign");
        fs::copy(source, &binary).unwrap();
        let project = Self { root, binary };
        project.write("wish.sysml", "package ConsumerWish {}\n");
        project.write(
            "Cargo.toml",
            "[package]\nname='consumer'\nversion='0.1.0'\n",
        );
        project.write("src/main.rs", "fn main() {}\n");
        project.write("inputs/data.txt", "pass\n");
        project
    }
    fn write(&self, uri: &str, text: &str) {
        let path = self.root.join(uri);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn call(&self, args: &[&str]) -> Output {
        Command::new(&self.binary)
            .current_dir(&self.root)
            .env_remove("XDESIGN_STORE_ROOT")
            .env("PYTHONPATH", "/deliberate/inherited/import/path")
            .env("PYTHONHOME", "/invalid/inherited/python/home")
            .args(args)
            .output()
            .unwrap()
    }
    fn value(&self, args: &[&str], exit: i32) -> Value {
        let output = self.call(args);
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{args:?}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn init(&self, id: &str) -> Value {
        self.value(
            &[
                "init",
                id,
                "--flow",
                "simple-flow",
                "--wish",
                "wish.sysml",
                "--checks",
                "checks.json",
            ],
            0,
        )
    }
    fn plan(&self, id: &str) -> Value {
        serde_json::from_slice(
            &fs::read(self.root.join(format!("cache/xdesign/{id}/plan.json"))).unwrap(),
        )
        .unwrap()
    }
    fn reports(&self, id: &str) -> Vec<Value> {
        let mut reports = vec![];
        for entry in fs::read_dir(self.root.join(format!("cache/xdesign/{id}/reports"))).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "json") {
                let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
                if value["overall_verdict"].is_string() {
                    reports.push(value);
                }
            }
        }
        reports
    }
    fn library(&self, python: Option<&str>) {
        self.write("checkpoints/scripts/budget/check.py", SCRIPT);
        self.write("checkpoints/scripts/budget/helper.py", "VALUE = 41\n");
        self.write(
            "checkpoints/scripts/budget/rules.json",
            "{\"expected\":41}\n",
        );
        let entries:Vec<Value>=["req","design"].iter().map(|id| json!({
            "checkpoint_id":format!("consumer-{id}"),"title":"Consumer check","description":"Check this project using declared helpers and rule data.","kind":"syntactic","tags":[],
            "rules":[{"rule_id":"project-data","condition":"Project data and declared helper agree.","reads":["inputs/data.txt"]}],"inputs":["inputs/data.txt"],
            "executor":{"mode":"script","command_uri":"checkpoints/scripts/budget/check.py","files":["checkpoints/scripts/budget/helper.py","checkpoints/scripts/budget/rules.json"],"python":python},
            "version":1,"created_at":"2026-10-26T00:00:00Z","updated_at":"2026-10-26T00:00:00Z","source":"manual","history":[]
        })).collect();
        self.write(
            "checkpoints/library.json",
            &json!({"schema_version":1,"checkpoints":entries}).to_string(),
        );
        self.write("checks.json",&json!({"schema_version":1,"checks":[
            {"checkpoint_id":"simpleRequirementsAccepted","mode":"library","library_id":"consumer-req"},
            {"checkpoint_id":"simpleDesignAccepted","mode":"library","library_id":"consumer-design"}
        ]}).to_string());
    }
    fn default_models(&self) {
        for name in [
            "requirements.sysml",
            "traceability.sysml",
            "detailedDesign.sysml",
            "composition.sysml",
        ] {
            self.write(
                &format!("mbse/model/{name}"),
                &fs::read_to_string(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("mbse/model")
                        .join(name),
                )
                .unwrap(),
            );
        }
    }
}

const SCRIPT: &str = r#"import argparse,hashlib,json,os,sys
from pathlib import Path
from helper import VALUE
p=argparse.ArgumentParser()
p.add_argument('--list-inputs',action='store_true')
for key in ['wish-model','record','checker-id','plan-id','plan-uri','plan-sha256','stage','artifact-revision','snapshot-digest','report']:p.add_argument('--'+key)
a=p.parse_args()
if a.list_inputs:print(json.dumps(['inputs/data.txt']));sys.exit(0)
plan=json.loads(Path(a.plan_uri).read_text())
gate=next(g for g in plan['checkpoints'] if (g.get('checker') or {}).get('checker_id')==a.checker_id)
expected=json.loads(Path(__file__).with_name('rules.json').read_text())['expected']
assert Path(os.environ['XDESIGN_PROJECT_ROOT'])==Path.cwd()
assert 'PYTHONPATH' not in os.environ and 'PYTHONHOME' not in os.environ
assert sys.flags.no_user_site==1
assert Path(os.environ['TMPDIR']).is_relative_to(Path(os.environ['XDESIGN_ATTEMPT_DIRECTORY']))
Path(os.environ['TMPDIR'],'isolated-output').write_text(str(VALUE))
passed=VALUE==expected and Path('inputs/data.txt').read_text().strip()=='pass'
verdict='passed' if passed else 'failed'
inputs=['inputs/data.txt']
hashes={uri:hashlib.sha256(Path(uri).read_bytes()).hexdigest() for uri in inputs+[a.wish_model,a.record,a.plan_uri]}
rules=[{'rule_id':r['rule_id'],'verdict':verdict,'inputs':inputs,'observed':{'helper':VALUE,'cwd':str(Path.cwd()),'attempt':os.environ['XDESIGN_ATTEMPT_DIRECTORY']},'failures':[] if passed else ['project data or helper mismatch']} for r in gate['checker']['rules']]
Path(a.report).write_text(json.dumps({'checker_id':a.checker_id,'stage':a.stage,'artifact_revision':int(a.artifact_revision),'snapshot_digest':a.snapshot_digest,'report_uri':a.report,'overall_verdict':verdict,'rules':rules,'input_sha256':hashes}))
"#;

#[test]
fn copied_binary_defaults_use_embedded_scripts_helper_and_parser_without_tools() {
    let p = Project::new();
    p.default_models();
    p.value(
        &[
            "init",
            "default",
            "--flow",
            "simple-flow",
            "--wish",
            "wish.sysml",
        ],
        0,
    );
    assert!(!p.root.join("tools").exists());
    let plan = p.plan("default");
    let bundle = &plan["checkpoints"][0]["checker"]["bundle"];
    assert_eq!(bundle["source"], "builtin-v1");
    let r = p.value(&["check", "default"], 0);
    assert_eq!(r["work"]["stage_id"], "design");
    let report = &p.reports("default")[0];
    assert_eq!(report["overall_verdict"], "passed");
    assert!(report["input_sha256"]
        .as_object()
        .unwrap()
        .keys()
        .any(|k| k.ends_with("tools/sysml_web/bin/sysml")));
    p.write("checks.json", &json!({"schema_version":1,"checks":[{"checkpoint_id":"simpleRequirementsAccepted","mode":"library","library_id":"builtin-simple-requirements"}]}).to_string());
    p.init("builtin-library");
    assert_eq!(
        p.plan("builtin-library")["checkpoints"][0]["checker"]["bundle"]["source"],
        "builtin-v1"
    );
    p.value(&["check", "builtin-library"], 0);
    // Default resource ownership is explicit even if a project authors a same-named file.
    p.write(
        "tools/checks/simple_requirements.py",
        "raise RuntimeError('must not shadow default')\n",
    );
    p.value(
        &[
            "init",
            "shadow",
            "--flow",
            "simple-flow",
            "--wish",
            "wish.sysml",
        ],
        0,
    );
    p.value(&["check", "shadow"], 0);
    let library = p.value(&["checkpoint", "list"], 0);
    assert!(!library.to_string().contains("missing script"));
    // Exercise a real negative rule, not only availability of the executable.
    let uri = "mbse/model/requirements.sysml";
    let source = fs::read_to_string(p.root.join(uri)).unwrap();
    p.write(
        uri,
        &source.replacen(
            "attribute redefines rationale =",
            "attribute redefines omittedRationale =",
            1,
        ),
    );
    p.value(
        &[
            "init",
            "invalid",
            "--flow",
            "simple-flow",
            "--wish",
            "wish.sysml",
        ],
        0,
    );
    p.value(&["check", "invalid"], 1);
    assert_eq!(p.reports("invalid")[0]["overall_verdict"], "failed");
}

#[test]
fn retained_helpers_survive_source_edits_deletion_and_cache_loss_and_new_runs_upgrade() {
    let p = Project::new();
    p.library(None);
    p.init("retained");
    let original = p.plan("retained");
    p.write("checkpoints/scripts/budget/helper.py", "VALUE = 99\n");
    p.init("upgraded");
    p.value(&["check", "upgraded"], 1);
    fs::remove_dir_all(p.root.join("checkpoints")).unwrap();
    fs::remove_dir_all(p.root.join("cache/xdesign/retained/executors")).unwrap();
    p.value(&["next", "retained"], 0);
    assert_eq!(p.plan("retained"), original);
    let head = p.value(&["store", "get", "retained", "--revision", "1"], 0);
    for material in head["data"]["revision"]["materials"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "checker-executor")
    {
        let uri = material["uri"].as_str().unwrap();
        assert_eq!(
            sha256_bytes(&fs::read(p.root.join(uri)).unwrap()),
            material["sha256"]
        );
    }
    p.value(&["check", "retained"], 0);
    assert_eq!(
        p.reports("retained")[0]["rules"][0]["observed"]["helper"],
        41
    );
    p.value(&["check", "retained"], 0);
    p.value(&["check", "retained"], 0);
    p.write(
        "evidence.txt",
        "Retained helpers and source-independent checks passed.\n",
    );
    let n = p.value(&["next", "retained"], 0);
    let w = &n["work"];
    let evidence = RsArtifactDigest {
        uri: "evidence.txt".into(),
        sha256: sha256_bytes(&fs::read(p.root.join("evidence.txt")).unwrap()),
    };
    let mut artifacts: Vec<RsArtifactDigest> =
        serde_json::from_value(w["candidate_artifacts"].clone()).unwrap();
    artifacts.push(evidence.clone());
    let result = json!({"wish_id":"retained","run_id":n["run_id"],"stage_id":"verify","visit_index":w["visit_index"],"base_revision":w["artifact_revision"],"state_token":n["state_token"],"description":"Verify retained execution","performed_by":"acceptance-test","artifacts":[evidence],"judgment":{"authority":"agent","identity":"acceptance-test","outcome":"passed","rationale":"The retained bundle and exact reports demonstrate this fixture.","artifact_snapshot_digest":snapshot_digest(&artifacts)}});
    p.write("result.json", &result.to_string());
    assert_eq!(
        p.value(&["check", "retained", "--result", "result.json"], 0)["completed"],
        true
    );
    p.value(
        &[
            "store",
            "checkout",
            "retained",
            "--revision",
            "1",
            "--destination",
            "checkout",
        ],
        0,
    );
    assert!(p
        .root
        .join("checkout/cache/xdesign/retained/executors")
        .is_dir());
}

#[test]
fn independent_projects_and_attempts_use_their_own_context_and_interpreter_evidence() {
    let a = Project::new();
    let b = Project::new();
    a.library(None);
    b.library(None);
    // A project-local interpreter can be a venv executable symlink; scripts/helpers cannot.
    #[cfg(unix)]
    {
        let python = Command::new("python3")
            .args(["-c", "import sys;print(sys.executable)"])
            .output()
            .unwrap();
        a.write("cache/checkers/python-placeholder", "");
        std::os::unix::fs::symlink(
            String::from_utf8(python.stdout).unwrap().trim(),
            a.root.join("cache/checkers/python3"),
        )
        .unwrap();
        a.library(Some("cache/checkers/python3"));
    }
    b.write("checkpoints/scripts/budget/helper.py", "VALUE = 99\n");
    a.init("same");
    b.init("same");
    a.value(&["check", "same"], 0);
    b.value(&["check", "same"], 1);
    b.value(&["check", "same"], 1);
    let ar = a.reports("same");
    let br = b.reports("same");
    assert_eq!(br.len(), 2);
    assert_ne!(
        br[0]["execution"]["attempt_directory"],
        br[1]["execution"]["attempt_directory"]
    );
    for (p, reports) in [(&a, ar), (&b, br)] {
        for report in reports {
            assert_eq!(
                report["execution"]["project_root"],
                p.root.to_string_lossy().as_ref()
            );
            assert!(report["execution"]["interpreter"]["version"]
                .as_str()
                .unwrap()
                .contains("3."));
            assert_eq!(
                report["execution"]["interpreter"]["sha256"]
                    .as_str()
                    .unwrap()
                    .len(),
                64
            );
            for file in report["execution"]["files"].as_array().unwrap() {
                assert_eq!(
                    report["input_sha256"][file["uri"].as_str().unwrap()],
                    file["sha256"]
                );
            }
            let attempt = report["execution"]["attempt_directory"].as_str().unwrap();
            assert!(p.root.join(attempt).join("tmp/isolated-output").is_file());
        }
    }
}

#[test]
fn tampering_and_invalid_dependencies_are_refused_without_source_fallback() {
    let p = Project::new();
    p.library(None);
    p.init("tamper");
    let plan = p.plan("tamper");
    let entry = plan["checkpoints"][0]["checker"]["bundle"]["entry_uri"]
        .as_str()
        .unwrap();
    p.write(entry, "print('tampered')\n");
    let output = p.call(&["check", "tamper"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("drifted"));
    for files in [
        vec!["../outside.py"],
        vec!["checkpoints/scripts/budget/missing.py"],
        vec![
            "checkpoints/scripts/budget/helper.py",
            "checkpoints/scripts/budget/helper.py",
        ],
    ] {
        let mut library: Value =
            serde_json::from_slice(&fs::read(p.root.join("checkpoints/library.json")).unwrap())
                .unwrap();
        library["checkpoints"][0]["executor"]["files"] = json!(files);
        p.write("checkpoints/library.json", &library.to_string());
        assert_eq!(
            p.call(&[
                "init",
                "invalid",
                "--flow",
                "simple-flow",
                "--wish",
                "wish.sysml",
                "--checks",
                "checks.json"
            ])
            .status
            .code(),
            Some(2)
        );
        assert_eq!(p.call(&["store", "get", "invalid"]).status.code(), Some(3));
    }
    p.library(Some("cache/checkers/nonexistent/python3"));
    assert_eq!(
        p.call(&[
            "init",
            "missing-python",
            "--flow",
            "simple-flow",
            "--wish",
            "wish.sysml",
            "--checks",
            "checks.json"
        ])
        .status
        .code(),
        Some(2)
    );
    #[cfg(unix)]
    {
        p.library(None);
        fs::remove_file(p.root.join("checkpoints/scripts/budget/helper.py")).unwrap();
        std::os::unix::fs::symlink(
            p.root.join("inputs/data.txt"),
            p.root.join("checkpoints/scripts/budget/helper.py"),
        )
        .unwrap();
        assert_eq!(
            p.call(&[
                "init",
                "linked-helper",
                "--flow",
                "simple-flow",
                "--wish",
                "wish.sysml",
                "--checks",
                "checks.json"
            ])
            .status
            .code(),
            Some(2)
        );
    }
}

#[test]
fn omitted_helper_is_an_explicit_failure_and_custom_builtin_name_keeps_project_ownership() {
    let p = Project::new();
    p.library(None);
    let mut library: Value =
        serde_json::from_slice(&fs::read(p.root.join("checkpoints/library.json")).unwrap())
            .unwrap();
    library["checkpoints"][0]["executor"]["files"] = json!([]);
    p.write("checkpoints/library.json", &library.to_string());
    p.init("undeclared");
    p.value(&["check", "undeclared"], 1);
    let logs = fs::read_dir(p.root.join("cache/xdesign/undeclared/reports"))
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            p.to_str()
                .unwrap()
                .ends_with(".stderr.log")
                .then(|| fs::read_to_string(p).unwrap())
        })
        .collect::<String>();
    assert!(logs.contains("No module named 'helper'"));
    let head = p.value(&["store", "get", "undeclared"], 0);
    assert!(head["data"]["revision"]["materials"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["role"] == "checker-attempt"
            && m["uri"].as_str().unwrap().ends_with(".stderr.log")));

    p.library(None);
    p.write("tools/checks/simple_requirements.py", SCRIPT);
    let mut library: Value =
        serde_json::from_slice(&fs::read(p.root.join("checkpoints/library.json")).unwrap())
            .unwrap();
    library["checkpoints"][0]["executor"]["command_uri"] =
        json!("tools/checks/simple_requirements.py");
    // Copy dependencies explicitly beside this deliberately reused logical source path.
    p.write("tools/checks/helper.py", "VALUE = 41\n");
    p.write("tools/checks/rules.json", "{\"expected\":41}\n");
    library["checkpoints"][0]["executor"]["files"] =
        json!(["tools/checks/helper.py", "tools/checks/rules.json"]);
    p.write("checkpoints/library.json", &library.to_string());
    p.init("explicit-project");
    assert_eq!(
        p.plan("explicit-project")["checkpoints"][0]["checker"]["bundle"]["source"],
        "project"
    );
    p.value(&["check", "explicit-project"], 0);
}

#[test]
fn input_discovery_has_a_deadline_and_interpreter_loss_retains_a_failed_attempt() {
    let p = Project::new();
    p.write(
        "checkpoints/scripts/hung.py",
        "import time\ntime.sleep(60)\n",
    );
    p.write("checks.json", &json!({"schema_version":1,"checks":[{"checkpoint_id":"simpleRequirementsAccepted","mode":"command","command_uri":"checkpoints/scripts/hung.py"}]}).to_string());
    let start = std::time::Instant::now();
    let output = Command::new(&p.binary)
        .current_dir(&p.root)
        .env_remove("XDESIGN_STORE_ROOT")
        .env("XDESIGN_CHECKER_TIMEOUT_SECS", "1")
        .args([
            "init",
            "hung",
            "--flow",
            "simple-flow",
            "--wish",
            "wish.sysml",
            "--checks",
            "checks.json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("deadline"));
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(p.call(&["store", "get", "hung"]).status.code(), Some(3));
    #[cfg(unix)]
    {
        let python = Command::new("python3")
            .args(["-c", "import sys;print(sys.executable)"])
            .output()
            .unwrap();
        p.write("cache/checkers/placeholder", "");
        std::os::unix::fs::symlink(
            String::from_utf8(python.stdout).unwrap().trim(),
            p.root.join("cache/checkers/python3"),
        )
        .unwrap();
        p.library(Some("cache/checkers/python3"));
        p.init("lost-python");
        fs::remove_file(p.root.join("cache/checkers/python3")).unwrap();
        let blocked = p.value(&["check", "lost-python"], 1);
        assert_eq!(
            blocked["assessed"]["assessments"][0]["outcome"],
            "unevaluated"
        );
        let head = p.value(&["store", "get", "lost-python"], 0);
        assert!(head["data"]["revision"]["materials"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["role"] == "checker-attempt"
                && m["uri"].as_str().unwrap().ends_with(".execution.json")));
    }
}
