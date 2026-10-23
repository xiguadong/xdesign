use serde_json::{json, Value};
use std::{fs, path::Path};
use uuid::Uuid;
use xdesign::core::*;
use xdesign::wish_verify::{create_wish_verify_run, wish_verify_flow_plan, RsUnusedCheck};

struct Fixture {
    dir: String,
    wish: String,
    record: String,
    plan: RsFlowPlan,
}

impl Fixture {
    fn new() -> Self {
        let dir = format!("cache/evidence/wish-0010/restore-tests/{}", Uuid::new_v4());
        fs::create_dir_all(&dir).unwrap();
        let wish = format!("{dir}/wish.sysml");
        let record = format!("{dir}/run.json");
        fs::write(&wish, "package Wish {}\n").unwrap();
        let mut plan = wish_verify_flow_plan();
        plan.freeze(format!("{dir}/plan.json")).unwrap();
        Self {
            dir,
            wish,
            record,
            plan,
        }
    }

    fn run(&self, outcome: RsCheckpointOutcome) -> RsFlowRun {
        let run =
            create_wish_verify_run(self.plan.clone(), &self.wish, &self.record, judge(outcome))
                .unwrap();
        run.write_record().unwrap();
        run
    }

    fn restore(&self) -> Result<RsFlowRun, String> {
        restore_run(
            self.plan.clone(),
            self.wish.clone(),
            self.record.clone(),
            Box::new(RsUnusedCheck),
            Box::new(judge(RsCheckpointOutcome::Passed)),
        )
    }

    fn read(&self) -> Value {
        serde_json::from_slice(&fs::read(&self.record).unwrap()).unwrap()
    }

    fn write(&self, value: &Value) {
        fs::write(&self.record, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }

    fn artifact(&self, run: &mut RsFlowRun, name: &str, bytes: &str) -> String {
        let uri = format!("{}/{name}", self.dir);
        fs::write(&uri, bytes).unwrap();
        run.record_activity(RsActivityEntry {
            description: "produce work".into(),
            performed_by: "test agent".into(),
            occurred_at: "2026-10-22T00:00:00Z".into(),
            artifact_uris: vec![uri.clone()],
            artifacts_changed: true,
            artifact_digests: vec![hash_artifact(&uri).unwrap()],
            removed_artifact_uris: vec![],
        })
        .unwrap();
        uri
    }

    fn complete(&self, run: &mut RsFlowRun) {
        self.artifact(run, "wish.txt", "requested work");
        run.evaluate_gates();
        assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
        self.artifact(run, "verify.txt", "verified work");
        run.evaluate_gates();
        assert_eq!(run.advance(), RsAdvanceOutcome::CompleteRun);
    }
}

fn judge(outcome: RsCheckpointOutcome) -> RsSuppliedJudge {
    RsSuppliedJudge {
        authority: RsJudgeAuthority::Agent,
        identity: "test reviewer".into(),
        outcome,
        rationale: "Reviewed retained test evidence".into(),
    }
}

#[test]
fn restores_without_writing_or_invoking_boundaries() {
    struct PanicBoundary;
    impl RsJudge for PanicBoundary {
        fn review(&self, _: &[String], _: &[String]) -> (RsCheckpointOutcome, String) {
            panic!("restore must not judge")
        }
        fn authority(&self) -> RsJudgeAuthority {
            panic!("restore must not consult judge")
        }
        fn identity(&self) -> &str {
            panic!("restore must not consult judge")
        }
    }
    impl RsCheckCommand for PanicBoundary {
        fn run_check(
            &self,
            _: &RsCheckerSpec,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: u32,
            _: &str,
        ) -> Result<RsGateReport, RsGateError> {
            panic!("restore must not check")
        }
    }
    let f = Fixture::new();
    let original = f.run(RsCheckpointOutcome::Passed);
    let before = fs::read(&f.record).unwrap();
    let modified = fs::metadata(&f.record).unwrap().modified().unwrap();
    let file_count = fs::read_dir(&f.dir).unwrap().count();
    let restored = restore_run(
        f.plan.clone(),
        f.wish.clone(),
        f.record.clone(),
        Box::new(PanicBoundary),
        Box::new(PanicBoundary),
    )
    .unwrap();
    assert_eq!(restored.run_id(), original.run_id());
    assert_eq!(restored.visits(), original.visits());
    assert_eq!(restored.status(), original.status());
    assert_eq!(
        restored.current_visit_index(),
        original.current_visit_index()
    );
    assert_eq!(before, fs::read(&f.record).unwrap());
    assert_eq!(
        modified,
        fs::metadata(&f.record).unwrap().modified().unwrap()
    );
    assert_eq!(file_count, fs::read_dir(&f.dir).unwrap().count());
}

#[test]
fn current_edits_restore_and_continue_with_a_new_revision() {
    let f = Fixture::new();
    let mut run = f.run(RsCheckpointOutcome::Failed);
    let uri = f.artifact(&mut run, "wish.txt", "first content");
    run.evaluate_gates();
    assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
    fs::write(&uri, "corrected content").unwrap();
    let mut restored = f.restore().unwrap();
    assert_eq!(restored.status(), RsFlowStatus::Blocked);
    f.artifact(&mut restored, "wish.txt", "corrected content");
    assert_eq!(restored.current().artifact_revision(), 2);
    restored.evaluate_gates();
    assert_eq!(restored.advance(), RsAdvanceOutcome::EnterNextStage);
    let resumed = f.restore().unwrap();
    assert_eq!(resumed.current().stage_id(), "verify");
    assert_eq!(resumed.visits()[0].assessments().len(), 2);
}

#[test]
fn terminal_reopens_restore_with_contiguous_visit_history() {
    let f = Fixture::new();
    let mut run = f.run(RsCheckpointOutcome::Passed);
    f.complete(&mut run);
    assert_eq!(f.restore().unwrap().status(), RsFlowStatus::Completed);
    assert!(run.reopen_stage("verify"));
    let mut restored = f.restore().unwrap();
    assert_eq!(restored.current().visit_index(), 1);
    f.artifact(&mut restored, "verify-again.txt", "reverified work");
    restored.evaluate_gates();
    assert_eq!(restored.advance(), RsAdvanceOutcome::CompleteRun);
    assert!(restored.reopen_stage("verify"));
    assert_eq!(f.restore().unwrap().current().visit_index(), 2);
}

#[test]
fn completed_artifact_drift_is_inspectable_but_cannot_authorize_progress() {
    let f = Fixture::new();
    let mut run = f.run(RsCheckpointOutcome::Passed);
    f.complete(&mut run);
    fs::remove_file(format!("{}/wish.txt", f.dir)).unwrap();
    let mut restored = f.restore().unwrap();
    assert_eq!(restored.status(), RsFlowStatus::Completed);
    assert_eq!(restored.advance(), RsAdvanceOutcome::RemainBlocked);
    assert_eq!(f.restore().unwrap().status(), RsFlowStatus::Blocked);
}

#[test]
fn refused_or_unpermitted_judge_records_remain_restorable() {
    for authority in [RsJudgeAuthority::Agent, RsJudgeAuthority::Script] {
        let f = Fixture::new();
        let mut supplied = judge(RsCheckpointOutcome::Unevaluated);
        supplied.authority = authority;
        let mut run = create_wish_verify_run(f.plan.clone(), &f.wish, &f.record, supplied).unwrap();
        f.artifact(&mut run, "wish.txt", "unreviewed work");
        let assessment = run.evaluate_gates().remove(0);
        assert_eq!(assessment.outcome, RsCheckpointOutcome::Unevaluated);
        assert_eq!(run.advance(), RsAdvanceOutcome::RemainBlocked);
        assert_eq!(f.restore().unwrap().status(), RsFlowStatus::Blocked);
    }
}

#[test]
fn rejects_unknown_fields_duplicate_fields_and_wrong_identity() {
    let f = Fixture::new();
    f.run(RsCheckpointOutcome::Passed);
    let original = f.read();
    let changes: Vec<(&str, Value)> = vec![
        ("/unrecognized", json!(true)),
        ("/run_id", json!("not-a-uuid")),
        ("/execution_type", json!("OtherRun")),
        ("/plan_id", json!("other-plan")),
        ("/plan_sha256", json!("0".repeat(64))),
        ("/plan_uri", json!("cache/other.json")),
        ("/wish_model_uri", json!("mbse/wishes/wish-0008.sysml")),
        ("/visits/0/unrecognized", json!(true)),
    ];
    for (pointer, value) in changes {
        let mut changed = original.clone();
        if pointer == "/unrecognized" {
            changed["unrecognized"] = value;
        } else if pointer.ends_with("/unrecognized") {
            changed["visits"][0]["unrecognized"] = value;
        } else {
            *changed.pointer_mut(pointer).unwrap() = value;
        }
        f.write(&changed);
        assert!(f.restore().is_err(), "accepted {pointer}");
    }
    let bytes = serde_json::to_string(&original).unwrap();
    let duplicate = bytes.replacen('{', "{\"run_id\":\"duplicate\",", 1);
    fs::write(&f.record, duplicate).unwrap();
    assert!(f.restore().is_err());
}

#[test]
fn rejects_unreachable_visits_and_forged_completion() {
    let f = Fixture::new();
    f.run(RsCheckpointOutcome::Passed);
    let original = f.read();
    for (pointer, value) in [
        ("/current_visit", json!(99)),
        ("/status", json!("pending")),
        ("/visits/0/stage_id", json!("unknown")),
        ("/visits/0/visit_index", json!(1)),
        ("/visits/1/status", json!("completed")),
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.write(&changed);
        assert!(f.restore().is_err(), "accepted {pointer}");
    }
    let mut forged = original.clone();
    forged["status"] = json!("completed");
    forged["current_visit"] = json!(1);
    forged["visits"][0]["status"] = json!("completed");
    forged["visits"][1]["status"] = json!("completed");
    f.write(&forged);
    assert!(f.restore().is_err());
    let mut missing = original;
    missing["visits"].as_array_mut().unwrap().pop();
    f.write(&missing);
    assert!(f.restore().is_err());
}

#[test]
fn replays_digest_updates_and_removals_and_rejects_inconsistent_history() {
    let f = Fixture::new();
    let mut run = f.run(RsCheckpointOutcome::Passed);
    let uri = f.artifact(&mut run, "wish.txt", "content");
    let original = f.read();
    for (pointer, value) in [
        ("/visits/0/artifact_revision", json!(2)),
        ("/visits/0/artifact_snapshot_digest", json!("0".repeat(64))),
        ("/visits/0/artifact_digests/0/sha256", json!("0".repeat(64))),
        (
            "/visits/0/activities/0/artifact_digests/0/sha256",
            json!("bad-digest"),
        ),
        ("/visits/0/activities/0/artifacts_changed", json!(false)),
        ("/visits/0/activities/0/description", json!("")),
        ("/visits/0/activities/0/artifact_uris", json!([])),
        (
            "/visits/0/activities/0/removed_artifact_uris",
            json!(["cache/missing.txt"]),
        ),
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.write(&changed);
        assert!(f.restore().is_err(), "accepted {pointer}");
    }
    run.record_activity(RsActivityEntry {
        description: "remove output".into(),
        performed_by: "test".into(),
        occurred_at: "now".into(),
        artifact_uris: vec![uri.clone()],
        artifacts_changed: true,
        artifact_digests: vec![],
        removed_artifact_uris: vec![uri],
    })
    .unwrap();
    let restored = f.restore().unwrap();
    assert_eq!(restored.current().artifact_revision(), 2);
    assert!(restored.current().artifact_digests().is_empty());
}

#[test]
fn rejects_foreign_stale_and_unauthorized_assessments() {
    let f = Fixture::new();
    let mut run = f.run(RsCheckpointOutcome::Passed);
    f.artifact(&mut run, "wish.txt", "content");
    run.evaluate_gates();
    let original = f.read();
    for (field, value) in [
        ("checkpoint_id", json!("outcomeAccepted")),
        ("evaluated_revision", json!(2)),
        ("evaluated_snapshot_digest", json!("0".repeat(64))),
        ("judged_by", json!("script")),
        ("evidence_uris", json!([])),
        ("evidence_uris", json!(["../outside.txt"])),
        ("assessor", json!("")),
        ("rationale", json!("")),
    ] {
        let mut changed = original.clone();
        changed["visits"][0]["assessments"][0][field] = value;
        f.write(&changed);
        assert!(f.restore().is_err(), "accepted {field}");
    }
    let mut changed = original;
    changed["visits"][0]["assessments"][0]["extra"] = json!(true);
    f.write(&changed);
    assert!(f.restore().is_err());
}

#[test]
fn syntactic_assessments_restore_only_with_the_declared_checker() {
    struct ReportCheck(String);
    impl RsCheckCommand for ReportCheck {
        fn run_check(
            &self,
            checker: &RsCheckerSpec,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            revision: u32,
            digest: &str,
        ) -> Result<RsGateReport, RsGateError> {
            fs::write(&self.0, "retained test checker report").unwrap();
            Ok(RsGateReport {
                holds: true,
                artifact_revision: revision,
                artifact_snapshot_digest: digest.to_owned(),
                report_uri: self.0.clone(),
                rules: checker
                    .rules
                    .iter()
                    .map(|r| RsRuleReport {
                        rule_id: r.rule_id.clone(),
                        holds: true,
                        diagnostic: None,
                    })
                    .collect(),
            })
        }
    }
    let mut f = Fixture::new();
    f.plan = xdesign::simple_flow::designed_simple_flow_plan();
    f.plan.freeze(format!("{}/plan.json", f.dir)).unwrap();
    let mut run = xdesign::simple_flow::create_designed_run(
        f.plan.clone(),
        f.wish.clone(),
        f.record.clone(),
        Box::new(ReportCheck(format!("{}/report.txt", f.dir))),
        Box::new(judge(RsCheckpointOutcome::Passed)),
    )
    .unwrap();
    f.artifact(&mut run, "requirements.txt", "requirement evidence");
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    assert_eq!(f.restore().unwrap().current().stage_id(), "design");
    let original = f.read();
    for (field, value) in [
        ("judged_by", json!("agent")),
        ("assessor", json!("foreign-checker")),
        ("evidence_uris", json!([])),
    ] {
        let mut changed = original.clone();
        changed["visits"][0]["assessments"][0][field] = value;
        f.write(&changed);
        assert!(f.restore().is_err(), "accepted {field}");
    }
    f.write(&original);
    fs::remove_file(format!("{}/report.txt", f.dir)).unwrap();
    assert_eq!(f.restore().unwrap().current().stage_id(), "design");
}

#[test]
fn missing_or_rebound_plan_and_wish_do_not_create_records() {
    let f = Fixture::new();
    assert!(f.restore().is_err());
    assert!(!Path::new(&f.record).exists());
    f.run(RsCheckpointOutcome::Passed);
    let bytes = fs::read(&f.record).unwrap();
    fs::write(&f.plan.plan_uri, "{}").unwrap();
    assert!(f.restore().is_err());
    assert_eq!(bytes, fs::read(&f.record).unwrap());
}
