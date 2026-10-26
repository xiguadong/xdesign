use std::{collections::VecDeque, fs, path::Path, sync::Mutex};
use uuid::Uuid;
use xdesign::checkpoint::RsCheckpointLibrary;
use xdesign::core::*;
use xdesign::simple_flow::{
    apply_check_bindings, apply_check_bindings_with_library, RsCheckBinding, RsCheckBindingConfig,
    RsCheckBindingMode,
};
use xdesign::wish_verify::*;

fn test_dir() -> String {
    let dir = format!("cache/implementation/rust-tests/{}", Uuid::new_v4());
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture(dir: &str, name: &str, content: &str) -> String {
    let path = format!("{dir}/{name}");
    fs::write(&path, content).unwrap();
    path
}

fn plan(dir: &str) -> RsFlowPlan {
    let mut plan = wish_verify_flow_plan();
    plan.freeze(format!("{dir}/plan.json")).unwrap();
    plan
}

fn judge(outcome: RsCheckpointOutcome) -> RsSuppliedJudge {
    RsSuppliedJudge {
        authority: RsJudgeAuthority::Human,
        identity: "reviewer".into(),
        outcome,
        rationale: "Reviewed the cited evidence".into(),
    }
}

fn run_with(dir: &str, judgment: impl RsJudge + 'static) -> RsFlowRun {
    let wish = fixture(dir, "wish.sysml", "package Wish {}\n");
    create_wish_verify_run(plan(dir), &wish, &format!("{dir}/run.json"), judgment).unwrap()
}

fn add_artifact(run: &mut RsFlowRun, dir: &str, name: &str, content: &str) -> String {
    let uri = fixture(dir, name, content);
    let digest = hash_artifact(&uri).unwrap();
    run.record_activity(RsActivityEntry {
        description: "record evidence".into(),
        performed_by: "test".into(),
        occurred_at: "2026-10-20T00:00:00Z".into(),
        artifact_uris: vec![uri.clone()],
        artifacts_changed: true,
        artifact_digests: vec![digest],
        removed_artifact_uris: vec![],
    })
    .unwrap();
    uri
}

fn complete_wish_stage(run: &mut RsFlowRun, dir: &str) {
    add_artifact(run, dir, "wish-record.txt", "the recorded wish");
    let entries = run.evaluate_gates();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].checkpoint_id, "wishAccepted");
    assert_eq!(entries[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    assert_eq!(run.current().stage_id(), "verify");
}

fn complete_run(run: &mut RsFlowRun, dir: &str) {
    complete_wish_stage(run, dir);
    add_artifact(run, dir, "outcome.txt", "the delivered outcome");
    let entries = run.evaluate_gates();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].checkpoint_id, "outcomeAccepted");
    assert_eq!(entries[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
    assert_eq!(run.status(), RsFlowStatus::Completed);
}

struct ScriptedJudge {
    verdicts: Mutex<VecDeque<RsCheckpointOutcome>>,
}

impl RsJudge for ScriptedJudge {
    fn review(
        &self,
        _criteria: &[String],
        _evidence_uris: &[String],
    ) -> (RsCheckpointOutcome, String) {
        let outcome = self
            .verdicts
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(RsCheckpointOutcome::Passed);
        (outcome, "Scripted judgment".into())
    }
    fn authority(&self) -> RsJudgeAuthority {
        RsJudgeAuthority::Human
    }
    fn identity(&self) -> &str {
        "scripted-reviewer"
    }
}

fn sneaky_checker() -> RsCheckerSpec {
    RsCheckerSpec {
        checker_id: "unmodeledCheck".into(),
        command_uri: "tools/checks/simple_requirements.py".into(),
        establishes: vec![RsConformanceKind::ArtifactPresence],
        rules: vec![RsCheckerRuleSpec {
            rule_id: "always-true".into(),
            condition: "Always true.".into(),
            reads: vec!["Cargo.toml".into()],
        }],
        custom: None,
    }
}

#[test]
fn non_blueprint_plans_are_rejected_before_run() {
    let dir = test_dir();
    let valid = plan(&dir);
    let mut p = valid.clone();
    p.stages[0].stage_id = "request".into();
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.stages[1].stage_id = "wish".into();
    assert_eq!(p.validate(), Err(RsPlanError::DuplicateStageId));
    let mut p = valid.clone();
    p.stages[0].exit_checkpoint_ids.clear();
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.checkpoints.push(p.checkpoints[0].clone());
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    assert_eq!(p.validate(), Err(RsPlanError::InvalidDesignedFlow));
    let mut p = valid.clone();
    p.checkpoints[1].checkpoint_id = "foreignGate".into();
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.checkpoints[0].kind = RsCheckKind::Syntactic;
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.checkpoints[0].checker = Some(sneaky_checker());
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidChecker)
    );
    let mut p = valid.clone();
    p.checkpoints[1].human_may_judge = false;
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.checkpoints[0].criteria[0] = "weaker criterion".into();
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.plan_id = "designedSimpleFlow".into();
    assert_eq!(
        validate_wish_verify_plan(&p),
        Err(RsPlanError::InvalidPlanFile)
    );

    // Construction rejects a frozen but non-blueprint plan; no run record exists.
    let wish = fixture(&dir, "wish.sysml", "package Wish {}\n");
    let mut drifted = wish_verify_flow_plan();
    drifted.checkpoints[0].criteria[0] = "weaker criterion".into();
    drifted.freeze(format!("{dir}/drifted-plan.json")).unwrap();
    let record = format!("{dir}/drifted-run.json");
    let result =
        create_wish_verify_run(drifted, &wish, &record, judge(RsCheckpointOutcome::Passed));
    assert!(matches!(result, Err(RsPlanError::InvalidDesignedFlow)));
    assert!(!Path::new(&record).exists());
}

#[test]
fn two_stage_run_completes_and_record_is_re_readable() {
    let dir = test_dir();
    let mut run = run_with(&dir, judge(RsCheckpointOutcome::Passed));
    assert_eq!(run.status(), RsFlowStatus::Active);
    assert_eq!(
        run.visits()
            .iter()
            .map(|v| v.stage_id())
            .collect::<Vec<_>>(),
        vec!["wish", "verify"]
    );
    assert!(run
        .visits()
        .iter()
        .all(|v| v.visit_index() == 0 && v.artifact_revision() == 0));
    assert_eq!(
        run.visits().iter().map(|v| v.status()).collect::<Vec<_>>(),
        vec![RsStageStatus::Active, RsStageStatus::Pending]
    );
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);

    let wish_artifact = add_artifact(&mut run, &dir, "wish-record.txt", "the recorded wish");
    let wish_entries = run.evaluate_gates();
    assert_eq!(wish_entries.len(), 1);
    assert_eq!(wish_entries[0].checkpoint_id, "wishAccepted");
    assert_eq!(wish_entries[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(wish_entries[0].judged_by, RsJudgeAuthority::Human);
    assert_eq!(wish_entries[0].assessor, "reviewer");
    assert_eq!(wish_entries[0].evaluated_revision, 1);
    assert_eq!(wish_entries[0].evidence_uris, vec![wish_artifact]);
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    assert_eq!(run.current().stage_id(), "verify");

    let outcome_artifact = add_artifact(&mut run, &dir, "outcome.txt", "the delivered outcome");
    let verify_entries = run.evaluate_gates();
    assert_eq!(verify_entries.len(), 1);
    assert_eq!(verify_entries[0].checkpoint_id, "outcomeAccepted");
    assert_eq!(verify_entries[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(verify_entries[0].evidence_uris, vec![outcome_artifact]);
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
    assert_eq!(run.status(), RsFlowStatus::Completed);

    // The run completed with RsUnusedCheck wired in: the all-semantic plan
    // never invoked the check boundary, or these gates could not have passed.
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(run.run_record_uri()).unwrap()).unwrap();
    assert_eq!(record["execution_type"], "FlowRun");
    assert_eq!(record["plan_id"], "designedWishVerifyFlow");
    assert_eq!(record["plan_uri"], run.plan().plan_uri);
    assert_eq!(record["plan_sha256"], run.plan().plan_sha256);
    assert_eq!(record["status"], "completed");
    let visits = record["visits"].as_array().unwrap();
    assert_eq!(visits.len(), 2);
    assert_eq!(visits[0]["stage_id"], "wish");
    assert_eq!(visits[1]["stage_id"], "verify");
    assert_eq!(visits[0]["artifact_revision"], 1);
    assert_eq!(visits[1]["artifact_revision"], 1);
    assert_eq!(
        visits[0]["artifact_digests"][0]["sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert_eq!(visits[0]["assessments"][0]["checkpoint_id"], "wishAccepted");
    assert_eq!(visits[0]["assessments"][0]["outcome"], "passed");
    assert_eq!(
        visits[1]["assessments"][0]["checkpoint_id"],
        "outcomeAccepted"
    );
    assert_eq!(visits[1]["assessments"][0]["outcome"], "passed");
    assert_eq!(
        visits[1]["assessments"][0]["evaluated_snapshot_digest"],
        visits[1]["artifact_snapshot_digest"]
    );
}

#[test]
fn unpermitted_or_unreasoned_judgment_never_passes() {
    for supplied in [
        RsSuppliedJudge {
            authority: RsJudgeAuthority::Script,
            ..judge(RsCheckpointOutcome::Passed)
        },
        RsSuppliedJudge {
            identity: String::new(),
            ..judge(RsCheckpointOutcome::Passed)
        },
        RsSuppliedJudge {
            rationale: String::new(),
            ..judge(RsCheckpointOutcome::Passed)
        },
        judge(RsCheckpointOutcome::Unevaluated),
    ] {
        let dir = test_dir();
        let mut run = run_with(&dir, supplied);
        add_artifact(&mut run, &dir, "wish-record.txt", "recorded");
        let entries = run.evaluate_gates();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].outcome, RsCheckpointOutcome::Unevaluated);
        assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
        assert_eq!(run.status(), RsFlowStatus::Blocked);
    }
    let dir = test_dir();
    let mut run = run_with(&dir, judge(RsCheckpointOutcome::Failed));
    add_artifact(&mut run, &dir, "wish-record.txt", "recorded");
    let entries = run.evaluate_gates();
    assert_eq!(entries[0].outcome, RsCheckpointOutcome::Failed);
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);

    // Both gates permit an agent judge; only a script authority is refused.
    let dir = test_dir();
    let mut run = run_with(
        &dir,
        RsSuppliedJudge {
            authority: RsJudgeAuthority::Agent,
            ..judge(RsCheckpointOutcome::Passed)
        },
    );
    add_artifact(&mut run, &dir, "wish-record.txt", "recorded");
    let entries = run.evaluate_gates();
    assert_eq!(entries[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(entries[0].judged_by, RsJudgeAuthority::Agent);
}

#[test]
fn semantic_pass_requires_recorded_existing_evidence() {
    // No recorded artifacts: a passing judgment cannot produce a pass.
    let dir = test_dir();
    let mut run = run_with(&dir, judge(RsCheckpointOutcome::Passed));
    let entries = run.evaluate_gates();
    assert_eq!(entries[0].outcome, RsCheckpointOutcome::Unevaluated);
    assert_eq!(
        entries[0].rationale,
        format!("{:?}", RsGateError::MissingEvidence)
    );
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.current().status(), RsStageStatus::Blocked);

    // A recorded artifact deleted afterward fails the visit's own verification.
    let dir = test_dir();
    let mut run = run_with(&dir, judge(RsCheckpointOutcome::Passed));
    let uri = add_artifact(&mut run, &dir, "wish-record.txt", "recorded");
    fs::remove_file(uri).unwrap();
    let entries = run.evaluate_gates();
    assert_eq!(entries[0].outcome, RsCheckpointOutcome::Unevaluated);
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.current().status(), RsStageStatus::Blocked);
}

#[test]
fn revision_bump_stales_earlier_pass_and_latest_verdict_governs() {
    let dir = test_dir();
    let mut run = run_with(
        &dir,
        ScriptedJudge {
            verdicts: Mutex::new(VecDeque::from([
                RsCheckpointOutcome::Passed,
                RsCheckpointOutcome::Failed,
                RsCheckpointOutcome::Unevaluated,
                RsCheckpointOutcome::Passed,
            ])),
        },
    );
    let uri = add_artifact(&mut run, &dir, "wish-record.txt", "first");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert!(run.current().completion_authorized(run.plan()));
    assert_eq!(run.current().artifact_revision(), 1);

    // A content change at the same URI bumps the revision and stales the pass.
    fs::write(&uri, "second").unwrap();
    let digest = hash_artifact(&uri).unwrap();
    run.record_activity(RsActivityEntry {
        description: "same URI correction".into(),
        performed_by: "test".into(),
        occurred_at: "2026-10-20T00:00:01Z".into(),
        artifact_uris: vec![uri],
        artifacts_changed: true,
        artifact_digests: vec![digest],
        removed_artifact_uris: vec![],
    })
    .unwrap();
    assert_eq!(run.current().artifact_revision(), 2);
    assert!(!run.current().completion_authorized(run.plan()));
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);

    // A later failed verdict supersedes the stale pass; advance stays blocked.
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Failed);
    assert!(!run.current().completion_authorized(run.plan()));
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);

    // A refused judgment records unevaluated and supersedes the failure.
    assert_eq!(
        run.evaluate_gates()[0].outcome,
        RsCheckpointOutcome::Unevaluated
    );
    assert!(!run.current().completion_authorized(run.plan()));
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);

    // A fresh pass at the current revision re-authorizes the exit.
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert!(run.current().completion_authorized(run.plan()));
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
}

#[test]
fn completed_run_reopens_only_the_terminal_verify_stage() {
    let dir = test_dir();
    let mut run = run_with(&dir, judge(RsCheckpointOutcome::Passed));
    complete_run(&mut run, &dir);
    let count = run.visits().len();
    assert!(run
        .record_activity(RsActivityEntry {
            description: "invalid".into(),
            performed_by: "test".into(),
            occurred_at: "now".into(),
            artifact_uris: vec![],
            artifacts_changed: false,
            artifact_digests: vec![],
            removed_artifact_uris: vec![]
        })
        .is_err());
    assert!(run.evaluate_gates().is_empty());
    assert!(!run.reopen_stage("wish"));
    assert_eq!(run.visits().len(), count);
    assert!(run.reopen_stage("verify"));
    assert_eq!(run.status(), RsFlowStatus::Active);
    assert_eq!(run.visits().len(), count + 1);
    assert_eq!(run.current().visit_index(), 1);
    assert_eq!(run.current().artifact_revision(), 0);
    assert_eq!(run.current().status(), RsStageStatus::Active);
    assert!(run.visits()[..count]
        .iter()
        .all(|v| v.status() == RsStageStatus::Completed));
    assert_eq!(
        run.visits()[0].assessments()[0].checkpoint_id,
        "wishAccepted"
    );
    assert!(!run.reopen_stage("verify"));

    add_artifact(&mut run, &dir, "rework.txt", "reworked outcome");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
    assert_eq!(run.current().visit_index(), 1);
    assert_eq!(run.visits().len(), count + 1);
}

#[test]
fn frozen_plan_drift_blocks_evaluation_and_advance() {
    let dir = test_dir();
    let mut run = run_with(&dir, judge(RsCheckpointOutcome::Passed));
    add_artifact(&mut run, &dir, "wish-record.txt", "recorded");
    fs::write(&run.plan().plan_uri, "tampered plan").unwrap();
    assert_eq!(
        run.evaluate_gates()[0].outcome,
        RsCheckpointOutcome::Unevaluated
    );
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
}

#[test]
fn unused_check_stub_fails_any_invocation() {
    let checker = sneaky_checker();
    assert_eq!(
        RsUnusedCheck
            .run_check(&checker, "plan", "uri", "sha", "wish", "record", "wish", 0, "digest"),
        Err(RsGateError::CommandFailed)
    );
}

#[test]
fn custom_check_bindings_are_rejected_for_wish_verify() {
    fn binding(
        checkpoint_id: &str,
        mode: RsCheckBindingMode,
        cli: Option<&str>,
        model: Option<&str>,
        command_uri: Option<&str>,
    ) -> RsCheckBinding {
        RsCheckBinding {
            checkpoint_id: checkpoint_id.into(),
            mode,
            cli: cli.map(str::to_owned),
            model: model.map(str::to_owned),
            command_uri: command_uri.map(str::to_owned),
            library_id: None,
        }
    }
    let mut plan = wish_verify_flow_plan();
    let non_default = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![binding(
            "simpleRequirementsAccepted",
            RsCheckBindingMode::Cli,
            Some("codex"),
            Some("gpt-6-astra"),
            None,
        )],
    };
    assert_eq!(
        apply_check_bindings(&mut plan, &non_default),
        Err("custom checks are supported for simple-flow only".to_string())
    );
    let command = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![binding(
            "simpleDesignAccepted",
            RsCheckBindingMode::Command,
            None,
            None,
            Some("tools/checks/simple_requirements.py"),
        )],
    };
    assert_eq!(
        apply_check_bindings(&mut plan, &command),
        Err("custom checks are supported for simple-flow only".to_string())
    );
    // Default bindings are a no-op even on the all-semantic flow, and an empty
    // binding list is accepted.
    let defaults = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![binding(
            "simpleRequirementsAccepted",
            RsCheckBindingMode::Default,
            None,
            None,
            None,
        )],
    };
    apply_check_bindings(&mut plan, &defaults).unwrap();
    apply_check_bindings(
        &mut plan,
        &RsCheckBindingConfig {
            schema_version: 1,
            checks: vec![],
        },
    )
    .unwrap();
    assert!(plan.checkpoints.iter().all(|gate| gate.checker.is_none()));
    assert_eq!(validate_wish_verify_plan(&plan), Ok(()));
}

#[test]
fn library_check_bindings_are_rejected_for_wish_verify() {
    let dir = test_dir();
    let library = RsCheckpointLibrary::load(Path::new(&dir)).unwrap();
    let mut plan = wish_verify_flow_plan();
    let config = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![
            RsCheckBinding {
                checkpoint_id: "simpleRequirementsAccepted".into(),
                mode: RsCheckBindingMode::Library,
                cli: None,
                model: None,
                command_uri: None,
                library_id: Some("some-library-checkpoint".into()),
            },
            RsCheckBinding {
                checkpoint_id: "verificationAccepted".into(),
                mode: RsCheckBindingMode::Library,
                cli: None,
                model: None,
                command_uri: None,
                library_id: Some("some-semantic-checkpoint".into()),
            },
        ],
    };
    assert_eq!(
        apply_check_bindings(&mut plan, &config),
        Err("custom checks are supported for simple-flow only".to_string())
    );
    assert_eq!(
        apply_check_bindings_with_library(&mut plan, &config, &library),
        Err("custom checks are supported for simple-flow only".to_string())
    );
    assert_eq!(validate_wish_verify_plan(&plan), Ok(()));
}
