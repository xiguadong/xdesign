use simpflow::*;
use std::{
    collections::VecDeque,
    fs,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

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
) -> (RsSimpleFlowRun, Arc<Mutex<Vec<String>>>) {
    let wish = fixture(dir, "wish.sysml", "package Wish {}\n");
    let stages = Arc::new(Mutex::new(vec![]));
    let check = Box::new(MockCheck {
        dir: dir.into(),
        verdicts: Arc::new(Mutex::new(verdicts.iter().copied().collect())),
        stages: stages.clone(),
    });
    let run = create_run(
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

fn add_artifact(run: &mut RsSimpleFlowRun, dir: &str, name: &str, content: &str) -> String {
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

fn reach_verify(run: &mut RsSimpleFlowRun, dir: &str) {
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
    assert_eq!(p.validate(), Err(RsPlanError::InvalidDesignedFlow));
    let mut p = valid.clone();
    p.stages[1].stage_id = "requirements".into();
    assert_eq!(p.validate(), Err(RsPlanError::DuplicateStageId));
    let mut p = valid.clone();
    p.checkpoints[0].stage_id = "verify".into();
    assert_eq!(p.validate(), Err(RsPlanError::InvalidDesignedFlow));
    let mut p = valid.clone();
    p.stages[0].next_stage_ids.clear();
    assert_eq!(p.validate(), Err(RsPlanError::InvalidDesignedFlow));
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
    assert_eq!(p.validate(), Err(RsPlanError::InvalidChecker));
    let mut p = valid.clone();
    p.checkpoints[0].criteria[0] = "weaker criterion".into();
    assert_eq!(p.validate(), Err(RsPlanError::InvalidDesignedFlow));
    let mut p = valid.clone();
    p.checkpoints[0].checker.as_mut().unwrap().rules[0].condition = "always true".into();
    assert_eq!(p.validate(), Err(RsPlanError::InvalidDesignedFlow));
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
    assert_eq!(record["execution_type"], "SimpleFlowRun");
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
    let mut run = create_run(
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
    assert_eq!(
        run.evaluate_gates()[0].outcome,
        RsCheckpointOutcome::Unevaluated
    );
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    assert!(run.last_error().is_some());
    assert_eq!(run.current_visit_index(), 0);
}
