use super::*;
use uuid::Uuid;
use xdesign::core::{
    RsCheckCommand, RsCheckerSpec, RsGateError, RsGateReport, RsRuleReport, RsStageStatus,
};

struct PassingCheck {
    report_uri: String,
}

impl RsCheckCommand for PassingCheck {
    fn run_check(
        &self,
        checker: &RsCheckerSpec,
        _plan_id: &str,
        _plan_uri: &str,
        _plan_sha256: &str,
        _wish_model: &str,
        _record: &str,
        _stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
    ) -> Result<RsGateReport, RsGateError> {
        fs::write(&self.report_uri, "requirements check passed").unwrap();
        Ok(RsGateReport {
            holds: true,
            artifact_revision,
            artifact_snapshot_digest: snapshot_digest.into(),
            report_uri: self.report_uri.clone(),
            rules: checker
                .rules
                .iter()
                .map(|rule| RsRuleReport {
                    rule_id: rule.rule_id.clone(),
                    holds: true,
                    diagnostic: None,
                })
                .collect(),
        })
    }
}

#[test]
fn edited_wish_does_not_stale_completed_requirements_artifact_snapshot() {
    let dir = format!("cache/implementation/wish-snapshot/{}", Uuid::new_v4());
    fs::create_dir_all(&dir).unwrap();
    let wish = format!("{dir}/wish.sysml");
    fs::write(&wish, "package Wish { doc /* Initial wish. */ }\n").unwrap();
    let mut plan = designed_simple_flow_plan();
    plan.freeze(format!("{dir}/plan.json")).unwrap();
    let report_uri = format!("{dir}/requirements-report.json");
    let mut run = create_designed_run(
        plan,
        wish.clone(),
        format!("{dir}/run.json"),
        Box::new(PassingCheck { report_uri }),
        Box::new(RsSuppliedJudge {
            authority: RsJudgeAuthority::Human,
            identity: "reviewer".into(),
            outcome: RsCheckpointOutcome::Passed,
            rationale: "Reviewed test evidence".into(),
        }),
    )
    .unwrap();
    run.write_record().unwrap();

    let paths = stage_artifact_paths("requirements", "unused").unwrap();
    assert!(!paths.is_empty());
    assert!(!paths.contains(&wish));
    let digests = paths
        .iter()
        .map(|path| hash_artifact(path).unwrap())
        .collect();
    run.record_activity(RsActivityEntry {
        description: "Record requirements stage input artifacts".into(),
        performed_by: "demo regression test".into(),
        occurred_at: "2026-10-20T00:00:00Z".into(),
        artifact_uris: paths.into_iter().collect(),
        artifacts_changed: true,
        artifact_digests: digests,
        removed_artifact_uris: vec![],
    })
    .unwrap();
    assert_eq!(run.evaluate_gates()[0].outcome, RsCheckpointOutcome::Passed);
    assert_eq!(run.advance(), RsAdvanceOutcome::EnterNextStage);
    assert_eq!(run.visits()[0].status(), RsStageStatus::Completed);
    let requirements_digest = run.visits()[0].artifact_snapshot_digest().to_owned();

    fs::write(&wish, "package Wish { doc /* Revised wish. */ }\n").unwrap();
    assert!(run.visits()[0].verify_artifacts());
    assert_eq!(
        run.visits()[0].artifact_snapshot_digest(),
        requirements_digest
    );
    assert!(run.visits()[0]
        .artifact_digests()
        .iter()
        .all(|digest| digest.uri != wish));
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(run.run_record_uri()).unwrap()).unwrap();
    assert_eq!(record["wish_model_uri"], wish);
}
