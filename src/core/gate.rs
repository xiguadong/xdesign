use super::{
    plan::{valid_relative_path, RsCheckpointSpec, RsCustomRunner},
    ports::{RsCheckCommand, RsJudge},
    record::{RsAssessmentEntry, RsStageVisit},
    status::{RsCheckKind, RsCheckpointOutcome, RsGateError, RsJudgeAuthority},
};
use std::{collections::HashSet, path::Path};

pub fn evaluate_checkpoint(
    spec: &RsCheckpointSpec,
    visit: &RsStageVisit,
    plan_id: &str,
    plan_uri: &str,
    plan_sha256: &str,
    wish_model_uri: &str,
    run_record_uri: &str,
    check: &dyn RsCheckCommand,
    judge: &dyn RsJudge,
) -> Result<RsAssessmentEntry, RsGateError> {
    if !visit.verify_artifacts() {
        return Err(RsGateError::CommandFailed);
    }
    match spec.kind {
        RsCheckKind::Syntactic => {
            let checker = spec.checker.as_ref().ok_or(RsGateError::CheckerMissing)?;
            let report = check.run_check(
                checker,
                plan_id,
                plan_uri,
                plan_sha256,
                wish_model_uri,
                run_record_uri,
                &spec.stage_id,
                visit.artifact_revision(),
                visit.artifact_snapshot_digest(),
            )?;
            let mut ids = HashSet::new();
            let declared: HashSet<_> = checker.rules.iter().map(|r| r.rule_id.as_str()).collect();
            if report.artifact_revision != visit.artifact_revision()
                || report.artifact_snapshot_digest != visit.artifact_snapshot_digest()
                || !valid_relative_path(&report.report_uri)
                || !Path::new(&report.report_uri).is_file()
                || report.rules.len() != checker.rules.len()
                || report.rules.iter().any(|r| {
                    !ids.insert(r.rule_id.as_str())
                        || !declared.contains(r.rule_id.as_str())
                        || (!r.holds && r.diagnostic.as_ref().is_none_or(|d| d.trim().is_empty()))
                })
                || report.holds != report.rules.iter().all(|r| r.holds)
            {
                return Err(RsGateError::CommandFailed);
            }
            let failures = report
                .rules
                .iter()
                .filter_map(|r| r.diagnostic.as_deref())
                .collect::<Vec<_>>()
                .join("; ");
            // A CLI-bound syntactic gate is answered by an agent, not by the
            // deterministic script; the assessment records that authority and
            // the exact `<cli>:<model>` executor so a replay can bind it.
            let agent = checker
                .custom
                .as_ref()
                .and_then(|custom| match custom.runner {
                    RsCustomRunner::Cli => custom.cli.as_ref().zip(custom.model.as_ref()),
                    RsCustomRunner::Process => None,
                });
            let (judged_by, assessor, passed_rationale) = match agent {
                Some((cli, model)) => (
                    RsJudgeAuthority::Agent,
                    format!("{cli}:{model}"),
                    format!(
                        "Agent CLI {cli} (model {model}) reported that all declared checker rules passed"
                    ),
                ),
                None => (
                    RsJudgeAuthority::Script,
                    checker.checker_id.clone(),
                    "All declared checker rules passed".to_owned(),
                ),
            };
            Ok(RsAssessmentEntry {
                checkpoint_id: spec.checkpoint_id.clone(),
                evaluated_revision: visit.artifact_revision(),
                evaluated_snapshot_digest: visit.artifact_snapshot_digest().to_owned(),
                outcome: if report.holds {
                    RsCheckpointOutcome::Passed
                } else {
                    RsCheckpointOutcome::Failed
                },
                judged_by,
                assessor,
                rationale: if report.holds {
                    passed_rationale
                } else {
                    failures
                },
                evidence_uris: vec![report.report_uri],
            })
        }
        RsCheckKind::Semantic => {
            let authority = judge.authority();
            if authority == RsJudgeAuthority::Script
                || authority == RsJudgeAuthority::Agent && !spec.agent_may_judge
                || authority == RsJudgeAuthority::Human && !spec.human_may_judge
                || judge.identity().trim().is_empty()
            {
                return Err(RsGateError::JudgeNotPermitted);
            }
            let evidence = visit.evidence_uris();
            let (outcome, rationale) = judge.review(&spec.criteria, &evidence);
            if outcome == RsCheckpointOutcome::Unevaluated || rationale.trim().is_empty() {
                return Err(RsGateError::JudgmentRefused);
            }
            if outcome == RsCheckpointOutcome::Passed
                && (visit.artifact_digests().is_empty()
                    || evidence.is_empty()
                    || evidence
                        .iter()
                        .any(|uri| !valid_relative_path(uri) || !Path::new(uri).is_file()))
            {
                return Err(RsGateError::MissingEvidence);
            }
            Ok(RsAssessmentEntry {
                checkpoint_id: spec.checkpoint_id.clone(),
                evaluated_revision: visit.artifact_revision(),
                evaluated_snapshot_digest: visit.artifact_snapshot_digest().to_owned(),
                outcome,
                judged_by: authority,
                assessor: judge.identity().to_owned(),
                rationale,
                evidence_uris: evidence,
            })
        }
    }
}
