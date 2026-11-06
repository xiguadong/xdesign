use super::{
    valid_digest, valid_relative_path, RsActivityEntry, RsArtifactDigest, RsCheckCommand,
    RsCheckKind, RsCheckpointOutcome, RsCustomRunner, RsFlowPlan, RsFlowRun, RsFlowStatus, RsJudge,
    RsJudgeAuthority, RsStageStatus, RsStageVisit,
};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredRun {
    run_id: String,
    execution_type: String,
    plan_id: String,
    plan_uri: String,
    plan_sha256: String,
    wish_model_uri: String,
    status: RsFlowStatus,
    current_visit: usize,
    visits: Vec<RsStageVisit>,
    #[serde(default)]
    artifact_snapshot_directory: Option<String>,
}

fn uri_set<'a>(uris: impl IntoIterator<Item = &'a str>) -> Result<HashSet<&'a str>, String> {
    let mut found = HashSet::new();
    for uri in uris {
        if !valid_relative_path(uri) || !found.insert(uri) {
            return Err(format!("invalid or duplicate artifact/evidence URI: {uri}"));
        }
    }
    Ok(found)
}

fn validate_activity(activity: &RsActivityEntry, replay: &RsStageVisit) -> Result<(), String> {
    if activity.description.trim().is_empty()
        || activity.performed_by.trim().is_empty()
        || activity.occurred_at.trim().is_empty()
    {
        return Err("activity description, actor, and time are required".into());
    }
    let updated = uri_set(activity.artifact_digests.iter().map(|a| a.uri.as_str()))?;
    let removed = uri_set(activity.removed_artifact_uris.iter().map(String::as_str))?;
    let named = uri_set(activity.artifact_uris.iter().map(String::as_str))?;
    if activity
        .artifact_digests
        .iter()
        .any(|a| !valid_digest(&a.sha256))
        || updated.intersection(&removed).next().is_some()
        || removed
            .iter()
            .any(|uri| !replay.artifact_digests().iter().any(|a| a.uri == *uri))
    {
        return Err("invalid activity digest or removal".into());
    }
    let changed: HashSet<_> = updated.union(&removed).copied().collect();
    if activity.artifacts_changed != !changed.is_empty()
        || activity.artifacts_changed && named != changed
        || activity.artifacts_changed && replay.artifact_revision() == u32::MAX
    {
        return Err("activity change declaration disagrees with its artifacts".into());
    }
    Ok(())
}

fn validate_history(visit: &RsStageVisit, plan: &RsFlowPlan) -> Result<(), String> {
    let mut replay = RsStageVisit::pending(visit.stage_id().to_owned(), visit.visit_index());
    let mut snapshots: Vec<(String, Vec<RsArtifactDigest>)> =
        vec![(replay.artifact_snapshot_digest().to_owned(), vec![])];
    for activity in visit.activities() {
        validate_activity(activity, &replay)?;
        if replay.record_activity(activity.clone()) {
            snapshots.push((
                replay.artifact_snapshot_digest().to_owned(),
                replay.artifact_digests().to_vec(),
            ));
        }
    }
    if replay.artifact_revision() != visit.artifact_revision()
        || replay.artifact_digests() != visit.artifact_digests()
        || replay.artifact_snapshot_digest() != visit.artifact_snapshot_digest()
    {
        return Err(format!(
            "{} visit has inconsistent artifact history",
            visit.stage_id()
        ));
    }
    let mut prior_revision = 0;
    for (index, assessment) in visit.assessments().iter().enumerate() {
        let gate = plan
            .checkpoints_of(visit.stage_id())
            .into_iter()
            .find(|g| g.checkpoint_id == assessment.checkpoint_id)
            .ok_or_else(|| {
                format!(
                    "assessment names foreign checkpoint: {}",
                    assessment.checkpoint_id
                )
            })?;
        let Some((digest, artifacts)) = snapshots.get(assessment.evaluated_revision as usize)
        else {
            return Err("assessment names an unknown artifact revision".into());
        };
        if assessment.evaluated_revision < prior_revision
            || &assessment.evaluated_snapshot_digest != digest
            || assessment.assessor.trim().is_empty()
            || assessment.rationale.trim().is_empty()
        {
            return Err("assessment revision, digest, assessor, or rationale is invalid".into());
        }
        prior_revision = assessment.evaluated_revision;
        let evidence = uri_set(assessment.evidence_uris.iter().map(String::as_str))?;
        // Gate errors preserve the attempted authority, even when that authority
        // caused the refusal. These entries cannot authorize completion.
        if assessment.outcome == RsCheckpointOutcome::Unevaluated {
            if assessment.assessor != "unavailable" || !evidence.is_empty() {
                return Err("unevaluated assessment is not an unavailable evaluation".into());
            }
            continue;
        }
        match gate.kind {
            RsCheckKind::Syntactic => {
                // The deterministic checker records script authority named by
                // its checker id. A CLI-bound gate additionally records agent
                // authority named `<cli>:<model>`; nothing else may authorize
                // a syntactic exit.
                let script_authority = assessment.judged_by == RsJudgeAuthority::Script
                    && gate
                        .checker
                        .as_ref()
                        .is_some_and(|c| c.checker_id == assessment.assessor);
                let agent_authority = match gate.checker.as_ref().and_then(|c| c.custom.as_ref()) {
                    Some(custom) if custom.runner == RsCustomRunner::Cli => {
                        let cli = custom.cli.as_deref().unwrap_or_default();
                        let model = custom.model.as_deref().unwrap_or_default();
                        assessment.judged_by == RsJudgeAuthority::Agent
                            && assessment.assessor == format!("{cli}:{model}")
                    }
                    _ => false,
                };
                if !(script_authority || agent_authority) || evidence.len() != 1 {
                    return Err(
                        "syntactic assessment has invalid checker authority or evidence".into(),
                    );
                }
            }
            RsCheckKind::Semantic => {
                let permitted = match assessment.judged_by {
                    RsJudgeAuthority::Agent => gate.agent_may_judge,
                    RsJudgeAuthority::Human => gate.human_may_judge,
                    RsJudgeAuthority::Script => false,
                };
                let mut expected: HashSet<&str> =
                    artifacts.iter().map(|a| a.uri.as_str()).collect();
                for earlier in &visit.assessments()[..index] {
                    if earlier.evaluated_revision == assessment.evaluated_revision {
                        expected.extend(earlier.evidence_uris.iter().map(String::as_str));
                    }
                }
                if !permitted
                    || evidence != expected
                    || assessment.outcome == RsCheckpointOutcome::Passed && artifacts.is_empty()
                {
                    return Err(
                        "semantic assessment has invalid judge authority or evidence".into(),
                    );
                }
            }
        }
    }
    Ok(())
}

fn validate_lifecycle(record: &StoredRun, plan: &RsFlowPlan) -> Result<(), String> {
    if record
        .artifact_snapshot_directory
        .as_ref()
        .is_some_and(|uri| !valid_relative_path(uri))
    {
        return Err("invalid artifact snapshot directory".into());
    }
    for visit in &record.visits {
        let snapshots = visit.artifact_snapshots();
        match (&record.artifact_snapshot_directory, visit.status()) {
            (Some(directory), RsStageStatus::Completed) => {
                if snapshots.len() != visit.artifact_digests().len()
                    || visit.artifact_digests().iter().any(|a| {
                        snapshots.get(&a.uri) != Some(&format!("{directory}/{}", a.sha256))
                    })
                {
                    return Err(format!(
                        "{} has incomplete or inconsistent retained artifacts",
                        visit.stage_id()
                    ));
                }
            }
            _ if !snapshots.is_empty() => {
                return Err("unexpected retained artifacts on a legacy or uncompleted visit".into())
            }
            _ => (),
        }
    }
    let stage_count = plan.stages.len();
    if record.visits.len() < stage_count || record.current_visit >= record.visits.len() {
        return Err("record is missing stage visits or has an invalid current index".into());
    }
    for (stage, visit) in plan.stages.iter().zip(&record.visits) {
        if stage.stage_id != visit.stage_id() || visit.visit_index() != 0 {
            return Err(
                "original visits must follow plan stage order with visit index zero".into(),
            );
        }
    }
    let current = &record.visits[record.current_visit];
    let terminal = plan
        .stage(current.stage_id())
        .is_some_and(|s| s.completes_flow);
    if record.visits.len() > stage_count {
        if record.current_visit != record.visits.len() - 1 || !terminal {
            return Err("reopened terminal visit must be the final current visit".into());
        }
        for (index, visit) in record.visits[stage_count..].iter().enumerate() {
            if visit.stage_id() != current.stage_id() || visit.visit_index() as usize != index + 1 {
                return Err("reopened terminal visits must have contiguous identities".into());
            }
        }
    }
    let mut path = HashSet::new();
    let mut stage = plan.initial_stage();
    loop {
        if !path.insert(stage.stage_id.as_str()) {
            return Err(
                "current visit cannot be reached without revisiting a completed stage".into(),
            );
        }
        if stage.stage_id == current.stage_id() {
            break;
        }
        if stage.completes_flow {
            return Err("current visit follows an already terminal stage".into());
        }
        stage = plan
            .next_stage(&stage.stage_id)
            .ok_or_else(|| "current visit is unreachable from the initial stage".to_owned())?;
    }
    for (index, visit) in record.visits.iter().enumerate() {
        validate_history(visit, plan)?;
        if index == record.current_visit {
            continue;
        }
        if path.contains(visit.stage_id()) {
            if visit.status() != RsStageStatus::Completed || !visit.completion_authorized(plan) {
                return Err("predecessor visit has no authorized completion".into());
            }
        } else if visit.status() != RsStageStatus::Pending
            || !visit.activities().is_empty()
            || !visit.assessments().is_empty()
        {
            return Err("unvisited stage must remain pristine and pending".into());
        }
    }
    match record.status {
        RsFlowStatus::Active if current.status() == RsStageStatus::Active => Ok(()),
        RsFlowStatus::Blocked if current.status() == RsStageStatus::Blocked => Ok(()),
        RsFlowStatus::Completed
            if terminal
                && current.status() == RsStageStatus::Completed
                && current.completion_authorized(plan) =>
        {
            Ok(())
        }
        _ => Err("run and current visit statuses disagree or completion is unauthorized".into()),
    }
}

/// Restore and validate local run history without writing or checking live artifacts.
pub fn restore_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsFlowRun, String> {
    plan.validate()
        .map_err(|e| format!("invalid restoration plan: {e:?}"))?;
    if !valid_relative_path(&wish_model_uri)
        || !wish_model_uri.ends_with(".sysml")
        || !Path::new(&wish_model_uri).is_file()
    {
        return Err("invalid restoration wish model URI".into());
    }
    if !valid_relative_path(&run_record_uri) {
        return Err("invalid restoration run record URI".into());
    }
    let bytes = fs::read(&run_record_uri).map_err(|e| format!("cannot read run record: {e}"))?;
    let record = validated_record(&plan, &wish_model_uri, &bytes)?;
    Ok(RsFlowRun {
        run_id: record.run_id,
        plan_id: record.plan_id,
        plan_uri: record.plan_uri,
        plan_sha256: record.plan_sha256,
        wish_model_uri,
        run_record_uri,
        plan,
        status: record.status,
        visits: record.visits,
        current_visit: record.current_visit,
        check,
        judgment,
        persistence_error: false,
        last_error: None,
        artifact_snapshot_directory: record.artifact_snapshot_directory,
    })
}

fn validated_record(
    plan: &RsFlowPlan,
    wish_model_uri: &str,
    bytes: &[u8],
) -> Result<StoredRun, String> {
    let record: StoredRun =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid run record schema: {e}"))?;
    let run_id = Uuid::parse_str(&record.run_id).map_err(|_| "invalid run UUID".to_owned())?;
    if run_id.is_nil()
        || run_id.to_string() != record.run_id
        || record.execution_type != "FlowRun"
        || record.plan_id != plan.plan_id
        || record.plan_uri != plan.plan_uri
        || record.plan_sha256 != plan.plan_sha256
        || record.wish_model_uri != wish_model_uri
    {
        return Err("run identity, plan identity, or wish binding disagrees".into());
    }
    validate_lifecycle(&record, &plan)?;
    Ok(record)
}

/// Migration validation shares restoration's history rules but needs no cache files.
pub(crate) fn validate_retained_run(
    plan: &RsFlowPlan,
    wish_model_uri: &str,
    bytes: &[u8],
) -> Result<(), String> {
    validated_record(plan, wish_model_uri, bytes).map(|_| ())
}
