use std::{
    collections::VecDeque,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use xdesign::checkpoint::RsCheckpointLibrary;
use xdesign::core::*;
use xdesign::simple_flow::*;

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
    let mut plan = designed_simple_flow_plan();
    plan.freeze(format!("{dir}/plan.json")).unwrap();
    plan
}

#[derive(Clone, Copy)]
enum MockVerdict {
    Pass,
    Fail,
    Missing,
    Duplicate,
    Extra,
    Contradict,
    Stale,
    MissingDiagnostic,
    NoReport,
}

struct MockCheck {
    dir: String,
    verdicts: Arc<Mutex<VecDeque<MockVerdict>>>,
    stages: Arc<Mutex<Vec<String>>>,
}

impl RsCheckCommand for MockCheck {
    fn run_check(
        &self,
        checker: &RsCheckerSpec,
        _plan_id: &str,
        _plan_uri: &str,
        _plan_sha256: &str,
        _wish_model: &str,
        _record: &str,
        stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
    ) -> Result<RsGateReport, RsGateError> {
        self.stages.lock().unwrap().push(stage.into());
        let verdict = self
            .verdicts
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(MockVerdict::Pass);
        let report_uri = format!("{}/report-{}.json", self.dir, Uuid::new_v4());
        if !matches!(verdict, MockVerdict::NoReport) {
            fs::write(&report_uri, "mock report").unwrap();
        }
        let mut rules: Vec<_> = checker
            .rules
            .iter()
            .map(|r| RsRuleReport {
                rule_id: r.rule_id.clone(),
                holds: true,
                diagnostic: None,
            })
            .collect();
        match verdict {
            MockVerdict::Fail => {
                rules[0].holds = false;
                rules[0].diagnostic = Some("rule failed".into());
            }
            MockVerdict::Missing => {
                rules.pop();
            }
            MockVerdict::Duplicate => {
                rules.push(rules[0].clone());
            }
            MockVerdict::Extra => {
                rules.push(RsRuleReport {
                    rule_id: "unknown".into(),
                    holds: true,
                    diagnostic: None,
                });
            }
            MockVerdict::MissingDiagnostic => {
                rules[0].holds = false;
            }
            _ => (),
        }
        Ok(RsGateReport {
            holds: !matches!(verdict, MockVerdict::Fail | MockVerdict::Contradict),
            artifact_revision: if matches!(verdict, MockVerdict::Stale) {
                artifact_revision + 1
            } else {
                artifact_revision
            },
            artifact_snapshot_digest: snapshot_digest.into(),
            report_uri,
            rules,
        })
    }
}

fn run_with(
    dir: &str,
    verdicts: &[MockVerdict],
    judge: RsSuppliedJudge,
) -> (RsFlowRun, Arc<Mutex<Vec<String>>>) {
    let wish = fixture(dir, "wish.sysml", "package Wish {}\n");
    let stages = Arc::new(Mutex::new(vec![]));
    let check = Box::new(MockCheck {
        dir: dir.into(),
        verdicts: Arc::new(Mutex::new(verdicts.iter().copied().collect())),
        stages: stages.clone(),
    });
    let run = create_designed_run(
        plan(dir),
        wish,
        format!("{dir}/run.json"),
        check,
        Box::new(judge),
    )
    .unwrap();
    (run, stages)
}

fn judge(outcome: RsCheckpointOutcome) -> RsSuppliedJudge {
    RsSuppliedJudge {
        authority: RsJudgeAuthority::Human,
        identity: "reviewer".into(),
        outcome,
        rationale: "Reviewed the cited evidence".into(),
    }
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

fn reach_verify(run: &mut RsFlowRun, dir: &str) {
    for stage in ["requirements", "design", "implement"] {
        add_artifact(run, dir, &format!("{stage}.txt"), stage);
        assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
        assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    }
    assert_eq!(run.current().stage_id(), "verify");
}

#[test]
fn malformed_plans_are_rejected_before_run() {
    let dir = test_dir();
    let valid = plan(&dir);
    let mut p = valid.clone();
    p.stages[0].exit_checkpoint_ids.clear();
    assert_eq!(
        validate_designed_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.stages[1].stage_id = "requirements".into();
    assert_eq!(p.validate(), Err(RsPlanError::DuplicateStageId));
    let mut p = valid.clone();
    p.checkpoints[1].checkpoint_id = p.checkpoints[0].checkpoint_id.clone();
    assert_eq!(p.validate(), Err(RsPlanError::InvalidDesignedFlow));
    let mut p = valid.clone();
    p.checkpoints[0].stage_id = "verify".into();
    assert_eq!(
        validate_designed_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.stages[0].next_stage_ids.clear();
    assert_eq!(
        validate_designed_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.stages.pop();
    assert_eq!(p.validate(), Err(RsPlanError::UnknownStageReference));
    let mut p = valid.clone();
    p.checkpoints[0].agent_may_judge = true;
    assert_eq!(p.validate(), Err(RsPlanError::InvalidChecker));
    let mut p = valid.clone();
    p.checkpoints[1].checker.as_mut().unwrap().checker_id = "simpleRequirementsCheck".into();
    assert_eq!(p.validate(), Err(RsPlanError::InvalidChecker));
    let mut p = valid.clone();
    p.checkpoints[1].checker.as_mut().unwrap().rules[0].rule_id = "wrong".into();
    assert_eq!(validate_designed_plan(&p), Err(RsPlanError::InvalidChecker));
    let mut p = valid.clone();
    p.checkpoints[0].criteria[0] = "weaker criterion".into();
    assert_eq!(
        validate_designed_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = valid.clone();
    p.checkpoints[0].checker.as_mut().unwrap().rules[0].condition = "always true".into();
    assert_eq!(
        validate_designed_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    fs::write(&valid.plan_uri, "tampered").unwrap();
    assert_eq!(valid.validate(), Err(RsPlanError::InvalidPlanFile));
}

#[test]
fn initial_visits_stage_scope_and_mixed_verify_gates() {
    let dir = test_dir();
    let (mut run, stages) = run_with(&dir, &[], judge(RsCheckpointOutcome::Passed));
    assert_eq!(run.status(), RsFlowStatus::Active);
    assert_eq!(
        run.visits()
            .iter()
            .map(|v| v.stage_id())
            .collect::<Vec<_>>(),
        vec!["requirements", "design", "implement", "verify"]
    );
    assert!(run
        .visits()
        .iter()
        .all(|v| v.visit_index() == 0 && v.artifact_revision() == 0));
    assert_eq!(
        run.visits().iter().map(|v| v.status()).collect::<Vec<_>>(),
        vec![
            RsStageStatus::Active,
            RsStageStatus::Pending,
            RsStageStatus::Pending,
            RsStageStatus::Pending,
        ]
    );
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    reach_verify(&mut run, &dir);
    add_artifact(&mut run, &dir, "verification.txt", "verification fixture");
    let verify = run.evaluate_gates();
    assert_eq!(verify.len(), 2);
    assert_eq!(
        verify
            .iter()
            .map(|x| x.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["simpleRecordAccepted", "verificationAccepted"]
    );
    assert_eq!(verify[1].judged_by, RsJudgeAuthority::Human);
    assert!(verify[1]
        .evidence_uris
        .iter()
        .any(|x| x.contains("report-")));
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
    assert_eq!(run.status(), RsFlowStatus::Completed);
    assert_eq!(
        *stages.lock().unwrap(),
        vec!["requirements", "design", "implement", "verify"]
    );
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(run.run_record_uri()).unwrap()).unwrap();
    assert_eq!(record["execution_type"], "FlowRun");
    assert_eq!(record["plan_sha256"], run.plan().plan_sha256);
    assert_eq!(record["visits"].as_array().unwrap().len(), 4);
}

#[test]
fn malformed_reports_never_pass_but_rule_failure_is_failed() {
    for verdict in [
        MockVerdict::Missing,
        MockVerdict::Duplicate,
        MockVerdict::Extra,
        MockVerdict::Contradict,
        MockVerdict::Stale,
        MockVerdict::MissingDiagnostic,
        MockVerdict::NoReport,
    ] {
        let dir = test_dir();
        let (mut run, _) = run_with(&dir, &[verdict], judge(RsCheckpointOutcome::Passed));
        add_artifact(&mut run, &dir, "source.txt", "source");
        assert_eq!(
            run.evaluate_gates()[0].outcome,
            RsCheckpointOutcome::Unevaluated
        );
        assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    }
    let dir = test_dir();
    let (mut run, _) = run_with(
        &dir,
        &[MockVerdict::Fail],
        judge(RsCheckpointOutcome::Passed),
    );
    add_artifact(&mut run, &dir, "source.txt", "source");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Failed);
}

#[test]
fn latest_verdict_and_revision_control_authorization() {
    let dir = test_dir();
    let (mut run, _) = run_with(
        &dir,
        &[MockVerdict::Pass, MockVerdict::Fail, MockVerdict::Pass],
        judge(RsCheckpointOutcome::Passed),
    );
    let uri = add_artifact(&mut run, &dir, "same.txt", "first");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert!(run.current().completion_authorized(run.plan()));
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Failed);
    assert!(!run.current().completion_authorized(run.plan()));
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert!(run.current().completion_authorized(run.plan()));
    fs::write(&uri, "changed without a recorded event").unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    let old_digest = run.current().artifact_snapshot_digest().to_owned();
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
    assert_ne!(run.current().artifact_snapshot_digest(), old_digest);
    assert!(!run.current().completion_authorized(run.plan()));
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
}

#[test]
fn activity_artifact_uris_match_digest_and_removal_changes() {
    let dir = test_dir();
    let (mut run, _) = run_with(&dir, &[], judge(RsCheckpointOutcome::Passed));
    let old = fixture(&dir, "old.txt", "old content");
    let new = fixture(&dir, "new.txt", "new content");
    let entry = RsActivityEntry {
        description: "record changed artifact".into(),
        performed_by: "test".into(),
        occurred_at: "2026-10-20T00:00:00Z".into(),
        artifact_uris: vec![old.clone()],
        artifacts_changed: true,
        artifact_digests: vec![hash_artifact(&old).unwrap()],
        removed_artifact_uris: vec![],
    };
    for names in [
        vec![format!("{dir}/missing.txt")],
        vec![],
        vec![old.clone(), old.clone()],
    ] {
        let mut invalid = entry.clone();
        invalid.artifact_uris = names;
        assert!(run.record_activity(invalid).is_err());
        assert_eq!(run.current().artifact_revision(), 0);
        assert!(run.current().activities().is_empty());
    }
    run.record_activity(entry).unwrap();
    assert_eq!(run.current().artifact_revision(), 1);

    let replacement = RsActivityEntry {
        description: "replace artifact".into(),
        performed_by: "test".into(),
        occurred_at: "2026-10-20T00:00:01Z".into(),
        artifact_uris: vec![old.clone(), new.clone()],
        artifacts_changed: true,
        artifact_digests: vec![hash_artifact(&new).unwrap()],
        removed_artifact_uris: vec![old.clone()],
    };
    let mut missing_removal_name = replacement.clone();
    missing_removal_name.artifact_uris = vec![new.clone()];
    assert!(run.record_activity(missing_removal_name).is_err());
    run.record_activity(replacement).unwrap();
    assert_eq!(run.current().artifact_revision(), 2);
    assert_eq!(run.current().artifact_digests()[0].uri, new);
    assert_eq!(run.current().artifact_digests().len(), 1);

    let no_change = RsActivityEntry {
        description: "inspect artifact".into(),
        performed_by: "test".into(),
        occurred_at: "2026-10-20T00:00:02Z".into(),
        artifact_uris: vec![format!("{dir}/missing.txt")],
        artifacts_changed: false,
        artifact_digests: vec![],
        removed_artifact_uris: vec![],
    };
    assert!(run.record_activity(no_change.clone()).is_err());
    assert_eq!(run.current().artifact_revision(), 2);
    let mut inspected = no_change;
    inspected.artifact_uris = vec![new];
    run.record_activity(inspected).unwrap();
    assert_eq!(run.current().artifact_revision(), 2);
}

#[test]
fn missing_passed_evidence_blocks_current_and_completed_visits() {
    let dir = test_dir();
    let (mut run, _) = run_with(&dir, &[], judge(RsCheckpointOutcome::Passed));
    add_artifact(&mut run, &dir, "requirements.txt", "requirements");
    let first = run.evaluate_gates();
    assert_eq!(first[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    let first_report = &first[0].evidence_uris[0];
    let first_bytes = fs::read(first_report).unwrap();
    fs::remove_file(first_report).unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
    assert_eq!(run.current().status(), RsStageStatus::Blocked);
    fs::write(first_report, first_bytes).unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);

    for stage in ["design", "implement"] {
        add_artifact(&mut run, &dir, &format!("{stage}.txt"), stage);
        assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
        assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    }
    add_artifact(&mut run, &dir, "verification.txt", "verification");
    let verification = run.evaluate_gates();
    assert!(verification
        .iter()
        .all(|entry| entry.outcome == RsCheckpointOutcome::Passed));

    let prior_report = run.visits()[0].assessments()[0].evidence_uris[0].clone();
    let prior_bytes = fs::read(&prior_report).unwrap();
    fs::remove_file(&prior_report).unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.current().status(), RsStageStatus::Blocked);
    fs::write(&prior_report, prior_bytes).unwrap();

    let verify_report = verification[0].evidence_uris[0].clone();
    let verify_bytes = fs::read(&verify_report).unwrap();
    fs::remove_file(&verify_report).unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
    fs::write(&verify_report, verify_bytes).unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);

    fs::remove_file(&prior_report).unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
    fs::write(&prior_report, "mock report").unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
}

#[test]
fn changed_completed_visit_artifact_blocks_terminal_advance() {
    let dir = test_dir();
    let (mut run, _) = run_with(&dir, &[], judge(RsCheckpointOutcome::Passed));
    reach_verify(&mut run, &dir);
    add_artifact(&mut run, &dir, "verification.txt", "verification");
    assert!(run
        .evaluate_gates()
        .iter()
        .all(|entry| entry.outcome == RsCheckpointOutcome::Passed));

    let earlier = format!("{dir}/requirements.txt");
    fs::write(&earlier, "changed after record check").unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
    fs::write(&earlier, "requirements").unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);

    fs::write(&earlier, "changed after completion").unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
}

#[test]
fn later_unevaluated_supersedes_pass_at_same_revision() {
    let dir = test_dir();
    let (mut run, _) = run_with(
        &dir,
        &[MockVerdict::Pass, MockVerdict::Missing],
        judge(RsCheckpointOutcome::Passed),
    );
    add_artifact(&mut run, &dir, "evidence.txt", "unchanged");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert!(run.current().completion_authorized(run.plan()));
    assert_eq!(
        run.evaluate_gates()[0].outcome,
        RsCheckpointOutcome::Unevaluated
    );
    assert!(!run.current().completion_authorized(run.plan()));
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.current().artifact_revision(), 1);
    assert_eq!(run.current_visit_index(), 0);
}

#[test]
fn failed_correction_reuses_visit_and_completed_run_needs_reopen() {
    let dir = test_dir();
    let (mut run, _) = run_with(
        &dir,
        &[MockVerdict::Fail],
        judge(RsCheckpointOutcome::Passed),
    );
    add_artifact(&mut run, &dir, "requirements.txt", "bad");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Failed);
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.current_visit_index(), 0);
    add_artifact(&mut run, &dir, "corrected.txt", "good");
    assert_eq!(run.current_visit_index(), 0);
    assert_eq!(run.current().status(), RsStageStatus::Active);
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    for stage in ["design", "implement", "verify"] {
        add_artifact(&mut run, &dir, &format!("{stage}.txt"), stage);
        assert!(run
            .evaluate_gates()
            .iter()
            .all(|x| x.outcome == RsCheckpointOutcome::Passed));
        run.advance();
    }
    assert_eq!(run.status(), RsFlowStatus::Completed);
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
    assert!(!run.reopen_stage("requirements"));
    assert_eq!(run.visits().len(), count);
    assert!(run.reopen_stage("verify"));
    assert_eq!(run.visits().len(), count + 1);
    assert_eq!(run.current().visit_index(), 1);
    assert_eq!(run.current().artifact_revision(), 0);
    assert_eq!(run.current().status(), RsStageStatus::Active);
    assert!(!run.reopen_stage("verify"));

    add_artifact(&mut run, &dir, "verify-revisit.txt", "revisit");
    assert!(run
        .evaluate_gates()
        .iter()
        .all(|entry| entry.outcome == RsCheckpointOutcome::Passed));
    let prior_verify_artifact = format!("{dir}/verify.txt");
    fs::write(&prior_verify_artifact, "changed after reopen").unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
    assert_eq!(run.visits()[3].status(), RsStageStatus::Completed);
    fs::write(&prior_verify_artifact, "verify").unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
    assert_eq!(run.current().visit_index(), 1);
    assert_eq!(run.visits().len(), count + 1);
}

#[test]
fn unauthorized_or_unreasoned_semantic_judge_cannot_complete() {
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
        let (mut run, _) = run_with(&dir, &[], supplied);
        reach_verify(&mut run, &dir);
        add_artifact(&mut run, &dir, "evidence.txt", "fixture");
        let entries = run.evaluate_gates();
        assert_eq!(entries[0].outcome, RsCheckpointOutcome::Passed);
        assert_eq!(entries[1].outcome, RsCheckpointOutcome::Unevaluated);
        assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    }
}

#[test]
fn semantic_pass_requires_a_verification_artifact_not_just_record_report() {
    let dir = test_dir();
    let (mut run, _) = run_with(&dir, &[], judge(RsCheckpointOutcome::Passed));
    reach_verify(&mut run, &dir);
    let entries = run.evaluate_gates();
    assert_eq!(entries[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(entries[1].outcome, RsCheckpointOutcome::Unevaluated);
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
}

#[test]
fn plan_file_drift_blocks_new_assessment_and_advancement() {
    let dir = test_dir();
    let (mut run, _) = run_with(&dir, &[], judge(RsCheckpointOutcome::Passed));
    add_artifact(&mut run, &dir, "artifact.txt", "content");
    fs::write(&run.plan().plan_uri, "tampered plan").unwrap();
    assert_eq!(
        run.evaluate_gates()[0].outcome,
        RsCheckpointOutcome::Unevaluated
    );
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
}

#[test]
fn failed_record_write_cannot_authorize_advancement() {
    let dir = test_dir();
    let wish = fixture(&dir, "wish.sysml", "package Wish {}\n");
    let check = Box::new(MockCheck {
        dir: dir.clone(),
        verdicts: Arc::new(Mutex::new(VecDeque::new())),
        stages: Arc::new(Mutex::new(vec![])),
    });
    // A directory is a normalized URI but cannot be replaced by the JSON file.
    let mut run = create_designed_run(
        plan(&dir),
        wish,
        dir.clone(),
        check,
        Box::new(judge(RsCheckpointOutcome::Passed)),
    )
    .unwrap();
    assert!(run.write_record().is_err());
    let uri = fixture(&dir, "evidence.txt", "content");
    assert!(run
        .record_activity(RsActivityEntry {
            description: "evidence".into(),
            performed_by: "test".into(),
            occurred_at: "2026-10-20T00:00:00Z".into(),
            artifact_uris: vec![uri.clone()],
            artifacts_changed: true,
            artifact_digests: vec![hash_artifact(&uri).unwrap()],
            removed_artifact_uris: vec![]
        })
        .is_err());
    assert_eq!(run.status(), RsFlowStatus::Active);
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
    assert_eq!(run.current().status(), RsStageStatus::Blocked);
    assert_eq!(
        run.evaluate_gates()[0].outcome,
        RsCheckpointOutcome::Unevaluated
    );
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert!(run.last_error().is_some());
    assert_eq!(run.current_visit_index(), 0);
}

#[test]
fn failed_transition_write_rolls_back_to_blocked_visit() {
    let dir = test_dir();
    let (mut run, _) = run_with(&dir, &[], judge(RsCheckpointOutcome::Passed));
    add_artifact(&mut run, &dir, "requirements.txt", "requirements");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    let record_path = run.run_record_uri().to_owned();
    fs::remove_file(&record_path).unwrap();
    fs::create_dir(&record_path).unwrap();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(run.current_visit_index(), 0);
    assert_eq!(run.status(), RsFlowStatus::Blocked);
    assert_eq!(run.current().status(), RsStageStatus::Blocked);
    assert_eq!(run.visits()[1].status(), RsStageStatus::Pending);
    assert!(run.last_error().is_some());
}

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

#[test]
fn default_plan_serialization_and_validation_are_unchanged() {
    let dir = test_dir();
    let mut plan = designed_simple_flow_plan();
    plan.freeze(format!("{dir}/default-plan.json")).unwrap();
    let bytes = fs::read(&plan.plan_uri).unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(
        !text.contains("\"custom\""),
        "default plan bytes gained a custom key: {text}"
    );
    assert!(
        !text.contains("\"library\""),
        "default plan bytes gained a library key: {text}"
    );
    // The default SimpleFlow plan bytes must stay exactly as before either
    // custom check bindings round (verified against the pre-change binary).
    assert_eq!(
        sha256_bytes(&bytes),
        "10864848c1cae28233827d7cd8d82b7a4c6a677a9c16a5687fcbdc3e9227ca04",
        "default SimpleFlow plan bytes changed"
    );
    let decoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for checkpoint in decoded["checkpoints"].as_array().unwrap() {
        let Some(checker) = checkpoint["checker"].as_object() else {
            continue;
        };
        let mut keys: Vec<_> = checker.keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["checker_id", "command_uri", "establishes", "rules"]
        );
    }
    assert!(plan
        .checkpoints
        .iter()
        .filter_map(|gate| gate.checker.as_ref())
        .all(|checker| checker.custom.is_none()));
    assert_eq!(validate_designed_plan(&plan), Ok(()));
    let reloaded = RsFlowPlan::load_frozen(&plan.plan_uri, &plan.plan_sha256).unwrap();
    assert_eq!(reloaded, plan);
    assert!(!validate_designed_plan(&reloaded).is_err());
}

#[test]
fn designed_plan_accepts_cli_and_command_bindings_and_rejects_invalid_ones() {
    let dir = test_dir();
    let mut plan = designed_simple_flow_plan();
    let config = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![binding(
            "simpleRequirementsAccepted",
            RsCheckBindingMode::Cli,
            Some("codex"),
            Some("gpt-6-astra"),
            None,
        )],
    };
    apply_check_bindings(&mut plan, &config).unwrap();
    plan.freeze(format!("{dir}/cli-plan.json")).unwrap();
    assert_eq!(validate_designed_plan(&plan), Ok(()));
    let checker = plan.checkpoints[0].checker.as_ref().unwrap();
    assert_eq!(
        checker.custom,
        Some(RsCustomChecker {
            runner: RsCustomRunner::Cli,
            cli: Some("codex".into()),
            model: Some("gpt-6-astra".into()),
        })
    );
    assert_eq!(
        checker.command_uri, "tools/checks/simple_requirements.py",
        "a cli binding must keep the default command URI"
    );
    let frozen = fs::read(&plan.plan_uri).unwrap();
    let frozen: serde_json::Value = serde_json::from_slice(&frozen).unwrap();
    assert_eq!(
        frozen["checkpoints"][0]["checker"]["custom"],
        serde_json::json!({"runner": "cli", "cli": "codex", "model": "gpt-6-astra"})
    );
    assert!(frozen["checkpoints"][1]["checker"].get("custom").is_none());

    // Unknown CLI and empty model are invalid checker bindings.
    let mut p = plan.clone();
    p.checkpoints[0]
        .checker
        .as_mut()
        .unwrap()
        .custom
        .as_mut()
        .unwrap()
        .cli = Some("other".into());
    assert_eq!(validate_designed_plan(&p), Err(RsPlanError::InvalidChecker));
    let mut p = plan.clone();
    p.checkpoints[0]
        .checker
        .as_mut()
        .unwrap()
        .custom
        .as_mut()
        .unwrap()
        .model = Some("  ".into());
    assert_eq!(validate_designed_plan(&p), Err(RsPlanError::InvalidChecker));
    // A cli binding may not move the covered default command URI.
    let mut p = plan.clone();
    p.checkpoints[0].checker.as_mut().unwrap().command_uri = "tools/checks/other.py".into();
    assert_eq!(validate_designed_plan(&p), Err(RsPlanError::InvalidChecker));
    // Tampered rules on a custom binding are still rejected.
    let mut p = plan.clone();
    p.checkpoints[0].checker.as_mut().unwrap().rules[0].condition = "always true".into();
    assert_eq!(
        validate_designed_plan(&p),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut p = plan.clone();
    p.checkpoints[0].checker.as_mut().unwrap().rules[0].rule_id = "forged-rule".into();
    assert_eq!(validate_designed_plan(&p), Err(RsPlanError::InvalidChecker));

    // A command binding swaps the command URI and records the process runner.
    let script = fixture(&dir, "custom_requirements.py", "print('{}')\n");
    let mut plan = designed_simple_flow_plan();
    let config = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![binding(
            "simpleRequirementsAccepted",
            RsCheckBindingMode::Command,
            None,
            None,
            Some(&script),
        )],
    };
    apply_check_bindings(&mut plan, &config).unwrap();
    plan.freeze(format!("{dir}/command-plan.json")).unwrap();
    assert_eq!(validate_designed_plan(&plan), Ok(()));
    let checker = plan.checkpoints[0].checker.as_ref().unwrap();
    assert_eq!(checker.command_uri, script);
    assert_eq!(
        checker.custom,
        Some(RsCustomChecker {
            runner: RsCustomRunner::Process,
            cli: None,
            model: None,
        })
    );

    // Unknown checkpoint ids (including the semantic gate) and missing command
    // files are rejected before a plan is frozen.
    let mut plan = designed_simple_flow_plan();
    for bad in [
        binding(
            "verificationAccepted",
            RsCheckBindingMode::Default,
            None,
            None,
            None,
        ),
        binding(
            "simpleUnknownAccepted",
            RsCheckBindingMode::Default,
            None,
            None,
            None,
        ),
    ] {
        let config = RsCheckBindingConfig {
            schema_version: 1,
            checks: vec![bad.clone()],
        };
        let error = apply_check_bindings(&mut plan, &config).unwrap_err();
        assert!(
            error.contains("unknown checkpoint in check bindings"),
            "{error}"
        );
    }
    let config = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![binding(
            "simpleRequirementsAccepted",
            RsCheckBindingMode::Command,
            None,
            None,
            Some("cache/implementation/missing-checker.py"),
        )],
    };
    assert!(apply_check_bindings(&mut plan, &config)
        .unwrap_err()
        .contains("command mode requires"));
    let duplicate = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![
            binding(
                "simpleRequirementsAccepted",
                RsCheckBindingMode::Default,
                None,
                None,
                None,
            ),
            binding(
                "simpleRequirementsAccepted",
                RsCheckBindingMode::Cli,
                Some("kimi"),
                Some("model"),
                None,
            ),
        ],
    };
    assert!(apply_check_bindings(&mut plan, &duplicate)
        .unwrap_err()
        .contains("duplicate check binding"));
    let wrong_schema = RsCheckBindingConfig {
        schema_version: 2,
        checks: vec![],
    };
    assert!(apply_check_bindings(&mut plan, &wrong_schema).is_err());
}

fn write_checkpoint_library(dir: &str, entries: serde_json::Value) {
    fs::create_dir_all(format!("{dir}/checkpoints")).unwrap();
    fs::write(
        format!("{dir}/checkpoints/library.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 1,
            "checkpoints": entries,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn checkpoint_entry(
    id: &str,
    kind: &str,
    rules: serde_json::Value,
    inputs: serde_json::Value,
    executor: serde_json::Value,
    criteria: serde_json::Value,
    judges: serde_json::Value,
) -> serde_json::Value {
    let mut entry = serde_json::json!({
        "checkpoint_id": id,
        "title": format!("Library checkpoint {id}"),
        "description": "A reusable library checkpoint used by the tests.",
        "kind": kind,
        "tags": ["test"],
        "version": 3,
        "created_at": "2026-10-26T00:00:00Z",
        "updated_at": "2026-10-26T00:05:00Z",
        "source": "manual",
        "history": [],
    });
    if !rules.is_null() {
        entry["rules"] = rules;
    }
    if !inputs.is_null() {
        entry["inputs"] = inputs;
    }
    if !executor.is_null() {
        entry["executor"] = executor;
    }
    if !criteria.is_null() {
        entry["criteria"] = criteria;
    }
    if !judges.is_null() {
        entry["judges"] = judges;
    }
    entry
}

fn library_binding(checkpoint_id: &str, library_id: &str) -> RsCheckBinding {
    RsCheckBinding {
        checkpoint_id: checkpoint_id.into(),
        mode: RsCheckBindingMode::Library,
        cli: None,
        model: None,
        command_uri: None,
        library_id: Some(library_id.into()),
    }
}

fn library_plan(dir: &str, entries: serde_json::Value, config: RsCheckBindingConfig) -> RsFlowPlan {
    write_checkpoint_library(dir, entries);
    let library = RsCheckpointLibrary::load(Path::new(dir)).unwrap();
    let mut plan = designed_simple_flow_plan();
    apply_check_bindings_with_library(&mut plan, &config, &library).unwrap();
    plan.freeze(format!("{dir}/library-plan.json")).unwrap();
    plan
}

#[test]
fn designed_plan_accepts_syntactic_and_semantic_library_bindings() {
    let dir = test_dir();
    fs::create_dir_all(format!("{dir}/tools/checks")).unwrap();
    fs::create_dir_all(format!("{dir}/mbse/model")).unwrap();
    fs::write(
        format!("{dir}/tools/checks/simple_requirements.py"),
        "print('library checker')\n",
    )
    .unwrap();
    fs::write(
        format!("{dir}/mbse/model/requirements.sysml"),
        "package XDesignRequirements {}\n",
    )
    .unwrap();
    let entries = serde_json::json!([
        checkpoint_entry(
            "energy-budget-positive",
            "syntactic",
            serde_json::json!([
                {"rule_id": "budget-declared", "condition": "Every actuator declares an energyBudget.", "reads": ["mbse/model/requirements.sysml"]},
            ]),
            serde_json::json!(["mbse/model/requirements.sysml"]),
            serde_json::json!({"mode": "script", "command_uri": "tools/checks/simple_requirements.py"}),
            serde_json::Value::Null,
            serde_json::Value::Null,
        ),
        checkpoint_entry(
            "verification-evidence-complete",
            "semantic",
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::json!([
                "Every requirement has retained evidence that a reader can re-examine.",
                "The evidence and the record stand on their own.",
            ]),
            serde_json::json!({"agent": true, "human": false}),
        ),
    ]);
    let config = RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![
            library_binding("simpleRequirementsAccepted", "energy-budget-positive"),
            library_binding("verificationAccepted", "verification-evidence-complete"),
        ],
    };
    let plan = library_plan(&dir, entries, config);
    assert_eq!(validate_designed_plan(&plan), Ok(()));
    let syntactic = &plan.checkpoints[0];
    assert_eq!(syntactic.library.as_deref(), Some("energy-budget-positive"));
    let checker = syntactic.checker.as_ref().unwrap();
    assert_eq!(checker.checker_id, "energy-budget-positive");
    assert_eq!(
        checker
            .rules
            .iter()
            .map(|rule| rule.rule_id.as_str())
            .collect::<Vec<_>>(),
        vec!["budget-declared"]
    );
    assert_eq!(
        syntactic.criteria,
        designed_simple_flow_plan().checkpoints[0].criteria
    );
    assert_eq!(checker.command_uri, "tools/checks/simple_requirements.py");
    assert_eq!(
        checker.custom,
        Some(RsCustomChecker {
            runner: RsCustomRunner::Process,
            cli: None,
            model: None,
        })
    );
    let semantic = &plan.checkpoints[4];
    assert_eq!(
        semantic.library.as_deref(),
        Some("verification-evidence-complete")
    );
    assert!(semantic.checker.is_none());
    assert_eq!(semantic.criteria.len(), 2);
    assert!(semantic.agent_may_judge);
    assert!(!semantic.human_may_judge);

    // The frozen plan reloads and validates without the library file.
    let reloaded = RsFlowPlan::load_frozen(&plan.plan_uri, &plan.plan_sha256).unwrap();
    assert_eq!(validate_designed_plan(&reloaded), Ok(()));
    assert_eq!(reloaded, plan);

    // Tampered library rules/criteria/library ids are rejected.
    let mut tampered = plan.clone();
    tampered.checkpoints[0].checker.as_mut().unwrap().rules[0].condition = "".into();
    assert_eq!(
        validate_designed_plan(&tampered),
        Err(RsPlanError::InvalidChecker)
    );
    let mut tampered = plan.clone();
    let duplicate_rule = tampered.checkpoints[0].checker.as_ref().unwrap().rules[0].clone();
    tampered.checkpoints[0]
        .checker
        .as_mut()
        .unwrap()
        .rules
        .push(duplicate_rule);
    assert_eq!(
        validate_designed_plan(&tampered),
        Err(RsPlanError::InvalidChecker)
    );
    // The frozen checker identity may be the reused script identity (for
    // example simpleRequirementsCheck) instead of the library id; an invalid
    // identity shape is still rejected.
    let mut renamed = plan.clone();
    renamed.checkpoints[0].checker.as_mut().unwrap().checker_id = "simpleRequirementsCheck".into();
    assert_eq!(validate_designed_plan(&renamed), Ok(()));
    let mut tampered = plan.clone();
    tampered.checkpoints[0].checker.as_mut().unwrap().checker_id = "not a checker id".into();
    assert_eq!(
        validate_designed_plan(&tampered),
        Err(RsPlanError::InvalidChecker)
    );
    let mut tampered = plan.clone();
    tampered.checkpoints[0].library = None;
    assert_eq!(
        validate_designed_plan(&tampered),
        Err(RsPlanError::InvalidChecker)
    );
    let mut tampered = plan.clone();
    tampered.checkpoints[4].criteria.clear();
    assert_eq!(
        validate_designed_plan(&tampered),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    let mut tampered = plan.clone();
    tampered.checkpoints[4].agent_may_judge = false;
    tampered.checkpoints[4].human_may_judge = false;
    assert_eq!(
        validate_designed_plan(&tampered),
        Err(RsPlanError::InvalidDesignedFlow)
    );
    // A non-library plan cannot gain a library marker out of nowhere.
    let mut tampered = designed_simple_flow_plan();
    tampered.checkpoints[0].library = Some("energy-budget-positive".into());
    assert_eq!(
        validate_designed_plan(&tampered),
        Err(RsPlanError::InvalidChecker)
    );
}

#[test]
fn library_binding_resolution_rejects_kind_missing_and_unknown_ids() {
    let dir = test_dir();
    fs::create_dir_all(format!("{dir}/tools/checks")).unwrap();
    fs::create_dir_all(format!("{dir}/mbse/model")).unwrap();
    fs::write(
        format!("{dir}/tools/checks/simple_requirements.py"),
        "print('library checker')\n",
    )
    .unwrap();
    fs::write(
        format!("{dir}/mbse/model/requirements.sysml"),
        "package XDesignRequirements {}\n",
    )
    .unwrap();
    let entries = serde_json::json!([
        checkpoint_entry(
            "syntactic-library",
            "syntactic",
            serde_json::json!([
                {"rule_id": "rule-one", "condition": "The rule holds.", "reads": ["mbse/model/requirements.sysml"]},
            ]),
            serde_json::json!(["mbse/model/requirements.sysml"]),
            serde_json::json!({"mode": "script", "command_uri": "tools/checks/simple_requirements.py"}),
            serde_json::Value::Null,
            serde_json::Value::Null,
        ),
        checkpoint_entry(
            "semantic-library",
            "semantic",
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::json!(["The evidence stands on its own."]),
            serde_json::json!({"agent": true, "human": true}),
        ),
    ]);
    write_checkpoint_library(&dir, entries);
    let library = RsCheckpointLibrary::load(Path::new(&dir)).unwrap();
    let mut plan = designed_simple_flow_plan();
    let config = |checkpoint_id: &str, library_id: &str| RsCheckBindingConfig {
        schema_version: 1,
        checks: vec![library_binding(checkpoint_id, library_id)],
    };
    let error = apply_check_bindings_with_library(
        &mut plan,
        &config("simpleRequirementsAccepted", "semantic-library"),
        &library,
    )
    .unwrap_err();
    assert!(error.contains("kind mismatch"), "{error}");
    let error = apply_check_bindings_with_library(
        &mut plan,
        &config("verificationAccepted", "syntactic-library"),
        &library,
    )
    .unwrap_err();
    assert!(error.contains("kind mismatch"), "{error}");
    let error = apply_check_bindings_with_library(
        &mut plan,
        &config("simpleRequirementsAccepted", "does-not-exist"),
        &library,
    )
    .unwrap_err();
    assert!(error.contains("unknown library checkpoint"), "{error}");
    let mut missing_id = config("simpleRequirementsAccepted", "syntactic-library");
    missing_id.checks[0].library_id = None;
    let error = apply_check_bindings_with_library(&mut plan, &missing_id, &library).unwrap_err();
    assert!(error.contains("requires a library_id"), "{error}");
    let mut invalid_id = config("simpleRequirementsAccepted", "syntactic-library");
    invalid_id.checks[0].library_id = Some("Not_A_Slug".into());
    let error = apply_check_bindings_with_library(&mut plan, &invalid_id, &library).unwrap_err();
    assert!(error.contains("invalid library_id"), "{error}");
    // Library mode without the project library resolver is refused.
    let error = apply_check_bindings(
        &mut plan,
        &config("simpleRequirementsAccepted", "syntactic-library"),
    )
    .unwrap_err();
    assert!(
        error.contains("require the project checkpoint library"),
        "{error}"
    );
    // The plan was not mutated by the rejected bindings.
    assert_eq!(validate_designed_plan(&plan), Ok(()));
}

#[derive(Clone)]
struct CriteriaCapturingJudge {
    criteria: Arc<Mutex<Vec<String>>>,
}

impl RsJudge for CriteriaCapturingJudge {
    fn review(
        &self,
        criteria: &[String],
        _evidence_uris: &[String],
    ) -> (RsCheckpointOutcome, String) {
        *self.criteria.lock().unwrap() = criteria.to_vec();
        (
            RsCheckpointOutcome::Passed,
            "Reviewed the frozen library criteria".into(),
        )
    }
    fn authority(&self) -> RsJudgeAuthority {
        RsJudgeAuthority::Agent
    }
    fn identity(&self) -> &str {
        "library-reviewer"
    }
}

#[test]
fn semantic_library_gate_is_judged_with_its_frozen_criteria_and_restores() {
    let dir = test_dir();
    let entries = serde_json::json!([checkpoint_entry(
        "verification-evidence-complete",
        "semantic",
        serde_json::Value::Null,
        serde_json::Value::Null,
        serde_json::Value::Null,
        serde_json::json!([
            "Every requirement has retained evidence that a reader can re-examine.",
            "The evidence and the record stand on their own.",
        ]),
        serde_json::json!({"agent": true, "human": false}),
    ),]);
    let mut plan = library_plan(
        &dir,
        entries,
        RsCheckBindingConfig {
            schema_version: 1,
            checks: vec![library_binding(
                "verificationAccepted",
                "verification-evidence-complete",
            )],
        },
    );
    assert_eq!(validate_designed_plan(&plan), Ok(()));
    let frozen_criteria = plan.checkpoints[4].criteria.clone();
    let wish = fixture(&dir, "wish.sysml", "package Wish {}\n");
    let criteria_seen = Arc::new(Mutex::new(Vec::new()));
    let mut run = create_designed_run(
        plan.clone(),
        wish.clone(),
        format!("{dir}/run.json"),
        Box::new(MockCheck {
            dir: dir.clone(),
            verdicts: Arc::new(Mutex::new(VecDeque::new())),
            stages: Arc::new(Mutex::new(vec![])),
        }),
        Box::new(CriteriaCapturingJudge {
            criteria: criteria_seen.clone(),
        }),
    )
    .unwrap();
    for stage in ["requirements", "design", "implement"] {
        add_artifact(&mut run, &dir, &format!("{stage}.txt"), stage);
        assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
        assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    }
    add_artifact(&mut run, &dir, "verification.txt", "verification fixture");
    let entries = run.evaluate_gates();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[1].checkpoint_id, "verificationAccepted",
        "library semantic gate keeps its plan checkpoint id"
    );
    assert_eq!(entries[1].judged_by, RsJudgeAuthority::Agent);
    assert_eq!(entries[1].assessor, "library-reviewer");
    assert_eq!(*criteria_seen.lock().unwrap(), frozen_criteria);
    assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
    assert_eq!(run.status(), RsFlowStatus::Completed);
    // Restoring the completed run re-validates the frozen library criteria and
    // the agent-authority semantic assessment through the plan alone.
    let restored = xdesign::core::restore_run(
        plan,
        wish,
        format!("{dir}/run.json"),
        Box::new(MockCheck {
            dir: dir.clone(),
            verdicts: Arc::new(Mutex::new(VecDeque::new())),
            stages: Arc::new(Mutex::new(vec![])),
        }),
        Box::new(CriteriaCapturingJudge {
            criteria: Arc::new(Mutex::new(Vec::new())),
        }),
    )
    .unwrap();
    assert_eq!(restored.status(), RsFlowStatus::Completed);
    assert_eq!(restored.plan().checkpoints[4].criteria, frozen_criteria);
}
