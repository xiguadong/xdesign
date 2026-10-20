use simpflow::*;
use std::{fs, process::Command};
use uuid::Uuid;

fn fixture_dir() -> String {
    let dir = format!(
        "cache/implementation/process-check-tests/{}",
        Uuid::new_v4()
    );
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn prepared_requirements_run(dir: &str) -> (RsSimpleFlowRun, String) {
    let wish = format!("{dir}/wish.sysml");
    fs::write(&wish, "package Wish {}\n").unwrap();
    let mut plan = designed_simple_flow_plan();
    plan.freeze(format!("{dir}/plan.json")).unwrap();
    let mut run = create_run(
        plan,
        wish.clone(),
        format!("{dir}/run.json"),
        Box::new(RsProcessCheck {
            report_directory: format!("{dir}/reports"),
        }),
        Box::new(RsSuppliedJudge {
            authority: RsJudgeAuthority::Human,
            identity: "test reviewer".into(),
            outcome: RsCheckpointOutcome::Passed,
            rationale: "Reviewed test evidence".into(),
        }),
    )
    .unwrap();
    let artifacts = [
        "mbse/model/requirements.sysml",
        "mbse/model/traceability.sysml",
    ];
    run.record_activity(RsActivityEntry {
        description: "Record requirements inputs".into(),
        performed_by: "test".into(),
        occurred_at: "2026-10-20T00:00:00Z".into(),
        artifact_uris: artifacts.iter().map(|s| (*s).into()).collect(),
        artifacts_changed: true,
        artifact_digests: artifacts
            .iter()
            .map(|uri| hash_artifact(uri).unwrap())
            .collect(),
        removed_artifact_uris: vec![],
    })
    .unwrap();
    (run, wish)
}

fn invoke_requirements(
    run: &RsSimpleFlowRun,
    wish: &str,
    dir: &str,
    checker: &RsCheckerSpec,
) -> Result<RsGateReport, RsGateError> {
    RsProcessCheck {
        report_directory: format!("{dir}/reports"),
    }
    .run_check(
        checker,
        &run.plan().plan_id,
        &run.plan().plan_uri,
        &run.plan().plan_sha256,
        wish,
        run.run_record_uri(),
        "requirements",
        run.current().artifact_revision(),
        run.current().artifact_snapshot_digest(),
    )
}

fn prepared_design_run(dir: &str) -> (RsSimpleFlowRun, String) {
    let (mut run, wish) = prepared_requirements_run(dir);
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    let output = Command::new("python3")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .args(["tools/checks/simple_design.py", "--list-inputs"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let artifacts: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
    run.record_activity(RsActivityEntry {
        description: "Record design inputs".into(),
        performed_by: "test".into(),
        occurred_at: "2026-10-20T00:00:00Z".into(),
        artifact_uris: artifacts.clone(),
        artifacts_changed: true,
        artifact_digests: artifacts
            .iter()
            .map(|uri| hash_artifact(uri).unwrap())
            .collect(),
        removed_artifact_uris: vec![],
    })
    .unwrap();
    (run, wish)
}

#[test]
fn real_requirements_report_binds_every_rule_input() {
    let dir = fixture_dir();
    let (run, wish) = prepared_requirements_run(&dir);
    let checker = run.plan().checkpoints[0].checker.as_ref().unwrap();
    let report = invoke_requirements(&run, &wish, &dir, checker).unwrap();
    assert!(report.holds);
    assert_eq!(report.rules.len(), checker.rules.len());
    let raw: serde_json::Value =
        serde_json::from_slice(&fs::read(&report.report_uri).unwrap()).unwrap();
    let hashes = raw["input_sha256"].as_object().unwrap();
    for rule in raw["rules"].as_array().unwrap() {
        for input in rule["inputs"].as_array().unwrap() {
            assert!(hashes.contains_key(input.as_str().unwrap()));
        }
    }
    assert!(hashes.contains_key(run.run_record_uri()));
    assert!(hashes.contains_key(&run.plan().plan_uri));
}

#[test]
fn forged_report_with_unrelated_or_incomplete_inputs_is_rejected() {
    let dir = fixture_dir();
    let (run, wish) = prepared_requirements_run(&dir);
    let unrelated = format!("{dir}/unrelated.txt");
    fs::write(&unrelated, "unrelated").unwrap();
    let fake_checker = format!("{dir}/fake_checker.py");
    let fake_source = r#"import hashlib
import json
import sys

args = dict(zip(sys.argv[1::2], sys.argv[2::2]))
paths = [args['--wish-model']]
hashes = {path: hashlib.sha256(open(path, 'rb').read()).hexdigest() for path in paths}
report = {
    'checker_id': args['--checker-id'],
    'stage': args['--stage'],
    'artifact_revision': int(args['--artifact-revision']),
    'snapshot_digest': args['--snapshot-digest'],
    'report_uri': args['--report'],
    'overall_verdict': 'passed',
    'rules': [
        {'rule_id': rule_id, 'verdict': 'passed', 'inputs': [__file__.replace('fake_checker.py', 'unrelated.txt')], 'observed': {'forged': True}, 'failures': []}
        for rule_id in ('requirements-carry-rationale', 'satisfy-links-resolve')
    ],
    'input_sha256': hashes,
}

with open(args['--report'], 'w', encoding='utf-8') as output:
    json.dump(report, output)
"#;
    fs::write(&fake_checker, fake_source).unwrap();
    let mut checker = run.plan().checkpoints[0].checker.as_ref().unwrap().clone();
    checker.command_uri = fake_checker;
    assert_eq!(
        invoke_requirements(&run, &wish, &dir, &checker),
        Err(RsGateError::CommandFailed)
    );

    let incomplete_source = fake_source
        .replace(
            "paths = [args['--wish-model']]",
            "paths = [args['--wish-model'], args['--record'], args['--plan-uri'], 'mbse/model/requirements.sysml']",
        )
        .replace(
            "[__file__.replace('fake_checker.py', 'unrelated.txt')]",
            "['mbse/model/requirements.sysml']",
        );
    fs::write(&checker.command_uri, incomplete_source).unwrap();
    assert_eq!(
        invoke_requirements(&run, &wish, &dir, &checker),
        Err(RsGateError::CommandFailed)
    );
}

#[test]
fn design_report_must_name_all_declared_checker_commands() {
    let dir = fixture_dir();
    let (run, wish) = prepared_design_run(&dir);
    let checker = run.plan().checkpoints[1].checker.as_ref().unwrap();
    let process = RsProcessCheck {
        report_directory: format!("{dir}/reports"),
    };
    let run_check = |checker: &RsCheckerSpec| {
        process.run_check(
            checker,
            &run.plan().plan_id,
            &run.plan().plan_uri,
            &run.plan().plan_sha256,
            &wish,
            run.run_record_uri(),
            "design",
            run.current().artifact_revision(),
            run.current().artifact_snapshot_digest(),
        )
    };
    assert!(run_check(checker).unwrap().holds);

    let wrapper = format!("{dir}/omit_command.py");
    fs::write(
        &wrapper,
        r#"import json
import subprocess
import sys

arguments = sys.argv[1:]
completed = subprocess.run([sys.executable, 'tools/checks/simple_design.py', *arguments])
if completed.returncode:
    sys.exit(completed.returncode)
report_uri = arguments[arguments.index('--report') + 1]
with open(report_uri, encoding='utf-8') as source:
    report = json.load(source)
for rule in report['rules']:
    if rule['rule_id'] == 'checker-commands-exist':
        rule['inputs'] = [uri for uri in rule['inputs'] if uri != 'tools/checks/simple_record.py']
with open(report_uri, 'w', encoding='utf-8') as destination:
    json.dump(report, destination)
"#,
    )
    .unwrap();
    let mut omitted = checker.clone();
    omitted.command_uri = wrapper;
    assert_eq!(run_check(&omitted), Err(RsGateError::CommandFailed));
}

#[test]
fn retained_plan_read_is_covered_by_the_frozen_plan_uri() {
    let dir = fixture_dir();
    let (run, wish) = prepared_requirements_run(&dir);
    let wrapper = format!("{dir}/include_plan.py");
    fs::write(
        &wrapper,
        r#"import json
import subprocess
import sys

arguments = sys.argv[1:]
completed = subprocess.run([sys.executable, 'tools/checks/simple_requirements.py', *arguments])
if completed.returncode:
    sys.exit(completed.returncode)
report_uri = arguments[arguments.index('--report') + 1]
plan_uri = arguments[arguments.index('--plan-uri') + 1]
with open(report_uri, encoding='utf-8') as source:
    report = json.load(source)
report['rules'][0]['inputs'].append(plan_uri)
with open(report_uri, 'w', encoding='utf-8') as destination:
    json.dump(report, destination)
"#,
    )
    .unwrap();
    let mut checker = run.plan().checkpoints[0].checker.as_ref().unwrap().clone();
    checker.command_uri = wrapper;
    checker.rules[0]
        .reads
        .push("retained plan serialization".into());
    assert!(
        invoke_requirements(&run, &wish, &dir, &checker)
            .unwrap()
            .holds
    );
}
