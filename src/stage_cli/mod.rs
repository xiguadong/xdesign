mod persistent;
mod store;
mod work;

use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
use store::{Backend, Stored};
use work::Collection;
use xdesign::core::{
    hash_artifact, restore_run, valid_digest, RsActivityEntry, RsAdvanceOutcome, RsArtifactDigest,
    RsAssessmentEntry, RsCheckCommand, RsCheckKind, RsCheckerSpec, RsCheckpointOutcome, RsFlowRun,
    RsFlowStatus, RsGateError, RsGateReport, RsJudgeAuthority, RsSuppliedJudge,
};
use xdesign::simple_flow::RsProcessCheck;

pub const HELP: &str =
    "Usage: xdesign init WISH_ID --flow simple-flow|wish-verify [--wish FILE] [--checks FILE]
       xdesign next WISH_ID
       xdesign check WISH_ID [--result FILE] [--operation-id ID]
       xdesign store COMMAND [OPTIONS]
       xdesign checkpoint list|show CHECKPOINT_ID|save --request FILE
       xdesign web [--host HOST] [--port PORT] [--open]
       xdesign --help

Run from the project root. init imports mbse/wishes/WISH_ID.sysml by default.
--checks FILE binds SimpleFlow checkpoints to the default deterministic checker, a
project command script, or a reusable checkpoint from checkpoints/library.json
(mode library with library_id); the
binding is frozen into the run plan. The checkpoint command lists, shows and saves
the project checkpoint library. next returns current work without changing the stored head.
check records outputs, evaluates the stage's gates and returns correction work,
successor work or completion. Semantic gates require a snapshot-bound judgment in
the optional result JSON; see docs/design/stage-cli.md for the exchange schema.
New runs use .xdesign/ and materialize operational state in cache/. Existing
cache-backed runs stay legacy. web serves the embedded operator web UI; loopback
by default. Exit codes: 0 work/completed, 1 blocked check, 2 invalid request or
command error. Legacy commands: xdesign simple-flow demo ...; xdesign wish-verify demo ...";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultArtifact {
    uri: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Judgment {
    authority: RsJudgeAuthority,
    identity: String,
    outcome: RsCheckpointOutcome,
    rationale: String,
    artifact_snapshot_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StageResult {
    wish_id: String,
    run_id: String,
    stage_id: String,
    visit_index: u32,
    base_revision: u32,
    state_token: String,
    description: String,
    performed_by: String,
    #[serde(default)]
    artifacts: Vec<ResultArtifact>,
    judgment: Option<Judgment>,
}

fn unjudged() -> RsSuppliedJudge {
    RsSuppliedJudge {
        authority: RsJudgeAuthority::Agent,
        identity: "no-supplied-judgment".into(),
        outcome: RsCheckpointOutcome::Unevaluated,
        rationale: "No snapshot-bound semantic judgment was supplied.".into(),
    }
}

struct RetainedCheck {
    directory: String,
    missing_outputs: bool,
}

impl RsCheckCommand for RetainedCheck {
    fn run_check(
        &self,
        checker: &RsCheckerSpec,
        plan_id: &str,
        plan_uri: &str,
        plan_sha256: &str,
        wish_model: &str,
        run_record: &str,
        stage: &str,
        artifact_revision: u32,
        digest: &str,
    ) -> Result<RsGateReport, RsGateError> {
        if self.missing_outputs {
            return Err(RsGateError::MissingEvidence);
        }
        store::state_path(&self.directory).map_err(|_| RsGateError::CommandFailed)?;
        fs::create_dir_all(&self.directory).map_err(|_| RsGateError::CommandFailed)?;
        let snapshot = format!(
            "{}/{}_{}_{}.record.json",
            self.directory,
            stage,
            artifact_revision,
            uuid::Uuid::new_v4()
        );
        let bytes = store::read_file(run_record).map_err(|_| RsGateError::CommandFailed)?;
        store::write_new(&snapshot, &bytes).map_err(|_| RsGateError::CommandFailed)?;
        // The process checker and its report bind this retained immutable exchange,
        // while the engine continues to update the canonical run.json atomically.
        RsProcessCheck {
            report_directory: self.directory.clone(),
        }
        .run_check(
            checker,
            plan_id,
            plan_uri,
            plan_sha256,
            wish_model,
            &snapshot,
            stage,
            artifact_revision,
            digest,
        )
    }
}

fn restore(
    stored: &Stored,
    judge: RsSuppliedJudge,
    missing_outputs: bool,
) -> Result<RsFlowRun, String> {
    let run = restore_run(
        stored.plan.clone(),
        stored.registry.wish_model_uri.clone(),
        stored.restore_record_uri.clone(),
        Box::new(RetainedCheck {
            directory: format!("{}/reports", store::directory(&stored.registry.wish_id)),
            missing_outputs,
        }),
        Box::new(judge),
    )?;
    if run.run_id() != stored.registry.run_id {
        return Err("run identity differs from registration".into());
    }
    if let Some(directory) = run.artifact_snapshot_directory() {
        if stored.backend != Backend::Persistent
            || directory
                != format!(
                    "{}/artifact-snapshots",
                    store::directory(&stored.registry.wish_id)
                )
        {
            return Err("artifact retention directory does not match its persistent run".into());
        }
    }
    // The generic restore validates shape/history; this boundary also constrains
    // every stored artifact and evidence path to the local project.
    for visit in run.visits() {
        for artifact in visit.artifact_digests() {
            let retained = visit.retained_artifact_uri(&artifact.uri);
            if retained == artifact.uri {
                store::artifact_path(&artifact.uri)?;
            } else {
                store::contained_path(retained)?;
            }
        }
        for assessment in visit.assessments() {
            for uri in &assessment.evidence_uris {
                store::contained_path(uri)?;
            }
        }
    }
    Ok(run)
}

fn current_assessments(run: &RsFlowRun) -> Vec<RsAssessmentEntry> {
    let mut latest = BTreeMap::new();
    for assessment in run.current().assessments() {
        latest.insert(assessment.checkpoint_id.clone(), assessment.clone());
    }
    latest.into_values().collect()
}

fn response(stored: &Stored, run: &RsFlowRun) -> Result<Value, String> {
    let completed = run.status() == RsFlowStatus::Completed;
    let projected = if completed {
        Value::Null
    } else {
        let visit = run.current();
        let work = stored.work.stage(visit.stage_id())?;
        let collection = work::collect(work, visit, &[])?;
        let assessments = current_assessments(run);
        let mut corrections = collection.diagnostics.clone();
        corrections.extend(
            run.visits()
                .iter()
                .filter(|v| v.status() == xdesign::core::RsStageStatus::Completed)
                .flat_map(|v| v.artifact_diagnostics()),
        );
        for gate in run.plan().checkpoints_of(visit.stage_id()) {
            match assessments
                .iter()
                .find(|assessment| assessment.checkpoint_id == gate.checkpoint_id)
            {
                Some(assessment)
                    if assessment.evaluated_revision == visit.artifact_revision()
                        && assessment.evaluated_snapshot_digest
                            == visit.artifact_snapshot_digest() =>
                {
                    if assessment.outcome != RsCheckpointOutcome::Passed {
                        corrections
                            .push(format!("{}: {}", gate.checkpoint_id, assessment.rationale));
                    }
                }
                _ => corrections.push(format!(
                    "{} has no assessment for this artifact revision.",
                    gate.checkpoint_id
                )),
            }
        }
        if run.status() == RsFlowStatus::Blocked && corrections.is_empty() {
            corrections.push("Advancement was refused: inspect earlier completed artifact digests, retained evidence and persistence diagnostics.".into());
        }
        json!({ "stage_id": visit.stage_id(), "visit_index": visit.visit_index(),
            "artifact_revision": visit.artifact_revision(), "artifact_snapshot_digest": visit.artifact_snapshot_digest(),
            "goal": work.goal, "instructions": work.instructions, "inputs": work.inputs,
            "input_artifacts": [stored.registry.wish_model_uri],
            "expected_outputs": work.expected_outputs, "frozen_output_paths": work.frozen_paths,
            "exit_checkpoints": run.plan().checkpoints_of(visit.stage_id()), "current_assessments": assessments,
            "correction_reasons": corrections, "collection_diagnostics": collection.diagnostics,
            "candidate_artifacts": collection.artifacts, "candidate_snapshot_digest": collection.snapshot,
            "review_contract": {
                "syntactic": run.plan().checkpoints_of(visit.stage_id()).into_iter().filter(|g| g.kind == xdesign::core::RsCheckKind::Syntactic).map(compact_gate).collect::<Vec<_>>(),
                "semantic": run.plan().checkpoints_of(visit.stage_id()).into_iter().filter(|g| g.kind == xdesign::core::RsCheckKind::Semantic).map(compact_gate).collect::<Vec<_>>(),
                "independent_review_is_exit_gate": false,
                "scope": "Apply exactly these frozen checkpoint criteria; independent review does not replace script checks or advancement integrity."
            },
            "workspace_changes": workspace_changes(run) })
    };
    let mut reply = json!({ "schema_version": 1, "wish_id": stored.registry.wish_id, "run_id": run.run_id(),
        "flow_id": stored.registry.flow_id, "status": run.status(), "completed": completed,
        "state_token": stored.token, "work": projected });
    reply["artifact_retention"] = json!(if run.artifact_snapshot_directory().is_some() {
        "completed-stage-snapshots"
    } else {
        "legacy-workspace"
    });
    reply["workspace_changes"] = json!(workspace_changes(run));
    if stored.backend == Backend::Persistent {
        reply["store_revision"] = json!(stored
            .store_revision
            .ok_or("persistent head has no revision")?);
    }
    Ok(reply)
}

fn compact_gate(gate: &xdesign::core::RsCheckpointSpec) -> Value {
    json!({"checkpoint_id": gate.checkpoint_id, "kind": gate.kind,
        "criteria": gate.criteria, "agent_may_judge": gate.agent_may_judge,
        "human_may_judge": gate.human_may_judge,
        "checker": gate.checker.as_ref().map(|c| json!({"checker_id": c.checker_id,
            "command_uri": c.command_uri, "rules": c.rules}))})
}

/// Latest completed evidence per source URI, without conflating earlier versions.
fn workspace_changes(run: &RsFlowRun) -> Vec<Value> {
    let mut latest = BTreeMap::new();
    for visit in run
        .visits()
        .iter()
        .filter(|v| v.status() == xdesign::core::RsStageStatus::Completed)
    {
        for artifact in visit.artifact_digests() {
            if !artifact.uri.starts_with("cache/") || artifact.uri.starts_with("cache/wishes/") {
                latest.insert(&artifact.uri, (visit.stage_id(), &artifact.sha256));
            }
        }
    }
    latest.into_iter().filter_map(|(uri, (stage, expected))| {
        let actual = store::artifact_path(uri).ok().and_then(|_| xdesign::core::hash_artifact(uri).ok()).map(|a| a.sha256);
        (actual.as_ref() != Some(expected)).then(|| json!({"uri": uri, "stage_id": stage,
            "assessed_sha256": expected, "workspace_sha256": actual,
            "action": "Bind and review the current bytes in a later stage; earlier approval remains attached to its retained snapshot."}))
    }).collect()
}

fn observe(id: &str) -> Result<(Stored, RsFlowRun, Value), String> {
    // A reader never takes a writer lock. The final comparison rejects a mixed
    // view if restoration or collection overlapped another process's commit.
    for _ in 0..3 {
        let stored = store::load(id)?;
        let run = restore(&stored, unjudged(), false)?;
        let reply = response(&stored, &run)?;
        let current = store::load(id)?;
        if current.token == stored.token && current.store_revision == stored.store_revision {
            return Ok((stored, run, reply));
        }
    }
    Err("run changed during inspection; retry next".into())
}

fn init(id: &str, flow: &str, wish: Option<&str>, checks: Option<&str>) -> Result<Value, String> {
    persistent::init(id, flow, wish, checks)
}

fn read_result_bytes(uri: Option<&str>) -> Result<Vec<u8>, String> {
    uri.map(store::read_file)
        .transpose()
        .map(Option::unwrap_or_default)
}

fn parse_result(uri: Option<&str>, bytes: &[u8]) -> Result<Option<StageResult>, String> {
    uri.map(|_| serde_json::from_slice(bytes).map_err(|e| format!("invalid result JSON: {e}")))
        .transpose()
}

fn read_result(uri: Option<&str>) -> Result<Option<StageResult>, String> {
    let bytes = read_result_bytes(uri)?;
    parse_result(uri, &bytes)
}

fn validate_result(
    result: Option<&StageResult>,
    stored: &Stored,
    run: &RsFlowRun,
) -> Result<Vec<RsArtifactDigest>, String> {
    let Some(result) = result else {
        return Ok(vec![]);
    };
    let visit = run.current();
    if result.wish_id != stored.registry.wish_id
        || result.run_id != run.run_id()
        || result.stage_id != visit.stage_id()
        || result.visit_index != visit.visit_index()
        || result.base_revision != visit.artifact_revision()
        || result.state_token != stored.token
    {
        return Err(
            "stale result: wish, run, stage, visit, revision or state token differs".into(),
        );
    }
    if result.description.trim().is_empty() || result.performed_by.trim().is_empty() {
        return Err("result description and performed_by must be nonempty".into());
    }
    let mut seen = BTreeSet::new();
    let mut artifacts = vec![];
    for supplied in &result.artifacts {
        store::artifact_path(&supplied.uri)?;
        if !seen.insert(&supplied.uri) || !valid_digest(&supplied.sha256) {
            return Err("invalid or duplicate result artifact digest".into());
        }
        let actual = hash_artifact(&supplied.uri)?;
        if actual.sha256 != supplied.sha256 {
            return Err(format!("stale result artifact digest: {}", supplied.uri));
        }
        artifacts.push(actual);
    }
    if let Some(judgment) = &result.judgment {
        let gates = run.plan().checkpoints_of(visit.stage_id());
        let semantic: Vec<_> = gates
            .into_iter()
            .filter(|gate| gate.kind == RsCheckKind::Semantic)
            .collect();
        if semantic.is_empty()
            || judgment.authority == RsJudgeAuthority::Script
            || judgment.identity.trim().is_empty()
            || judgment.rationale.trim().is_empty()
            || !valid_digest(&judgment.artifact_snapshot_digest)
            || semantic.iter().any(|gate| {
                judgment.authority == RsJudgeAuthority::Agent && !gate.agent_may_judge
                    || judgment.authority == RsJudgeAuthority::Human && !gate.human_may_judge
            })
        {
            return Err(
                "judgment does not match a permitted semantic gate or is incomplete".into(),
            );
        }
    }
    Ok(artifacts)
}

fn judge(result: Option<&StageResult>, collection: &Collection) -> Result<RsSuppliedJudge, String> {
    let Some(judgment) = result.and_then(|result| result.judgment.as_ref()) else {
        return Ok(unjudged());
    };
    if judgment.artifact_snapshot_digest != collection.snapshot {
        return Err("stale judgment: collected artifact snapshot differs".into());
    }
    Ok(RsSuppliedJudge {
        authority: judgment.authority,
        identity: judgment.identity.clone(),
        outcome: if collection.diagnostics.is_empty() {
            judgment.outcome
        } else {
            RsCheckpointOutcome::Unevaluated
        },
        rationale: if collection.diagnostics.is_empty() {
            judgment.rationale.clone()
        } else {
            collection.diagnostics.join("; ")
        },
    })
}

fn check_legacy(id: &str, result_uri: Option<&str>) -> Result<(Value, i32), String> {
    let (initial, initial_run, mut initial_reply) = observe(id)?;
    if initial_run.status() == RsFlowStatus::Completed {
        initial_reply["assessed"] = Value::Null;
        initial_reply["advance"] = json!(RsAdvanceOutcome::CompleteRun);
        return Ok((initial_reply, 0));
    }
    let result = read_result(result_uri)?;
    let supplied = validate_result(result.as_ref(), &initial, &initial_run)?;
    let observed = work::collect(
        initial.work.stage(initial_run.current().stage_id())?,
        initial_run.current(),
        &supplied,
    )?;
    judge(result.as_ref(), &observed)?;
    let _lock = store::Lock::acquire(id)?;
    let stored = store::load(id)?;
    let loaded = restore(&stored, unjudged(), false)?;
    if loaded.status() == RsFlowStatus::Completed {
        let mut reply = response(&stored, &loaded)?;
        reply["assessed"] = Value::Null;
        reply["advance"] = json!(RsAdvanceOutcome::CompleteRun);
        return Ok((reply, 0));
    }
    let supplied = validate_result(result.as_ref(), &stored, &loaded)?;
    let collection = work::collect(
        stored.work.stage(loaded.current().stage_id())?,
        loaded.current(),
        &supplied,
    )?;
    let judgment = judge(result.as_ref(), &collection)?;
    let mut run = restore(&stored, judgment, !collection.diagnostics.is_empty())?;
    let prior = run.current().artifact_digests();
    let updated: Vec<_> = collection
        .artifacts
        .iter()
        .filter(|artifact| !prior.contains(artifact))
        .cloned()
        .collect();
    let removed: Vec<_> = prior
        .iter()
        .filter(|artifact| {
            !collection
                .artifacts
                .iter()
                .any(|candidate| candidate.uri == artifact.uri)
        })
        .map(|artifact| artifact.uri.clone())
        .collect();
    let changed = !updated.is_empty() || !removed.is_empty();
    let named = if changed {
        updated
            .iter()
            .map(|artifact| artifact.uri.clone())
            .chain(removed.iter().cloned())
            .collect()
    } else {
        collection
            .artifacts
            .iter()
            .map(|artifact| artifact.uri.clone())
            .collect()
    };
    run.record_activity(RsActivityEntry {
        description: result
            .as_ref()
            .map(|r| r.description.clone())
            .unwrap_or_else(|| format!("Collect {} stage outputs", run.current().stage_id())),
        performed_by: result
            .as_ref()
            .map(|r| r.performed_by.clone())
            .unwrap_or_else(|| "xdesign check caller".into()),
        occurred_at: super::timestamp(),
        artifact_uris: named,
        artifacts_changed: changed,
        artifact_digests: updated,
        removed_artifact_uris: removed,
    })?;
    let assessments = run.evaluate_gates();
    if let Some(error) = run.last_error() {
        return Err(format!("cannot retain assessment: {error}"));
    }
    let assessed = json!({ "stage_id": run.current().stage_id(), "visit_index": run.current().visit_index(),
        "artifact_revision": run.current().artifact_revision(), "artifact_snapshot_digest": run.current().artifact_snapshot_digest(),
        "assessments": assessments });
    let advance = run.advance();
    if let Some(error) = run.last_error() {
        return Err(format!("cannot retain advancement: {error}"));
    }
    let final_state = store::load(id)?;
    let mut reply = response(&final_state, &run)?;
    reply["assessed"] = assessed;
    reply["advance"] = json!(advance);
    Ok((
        reply,
        if advance == RsAdvanceOutcome::RemainBlocked {
            1
        } else {
            0
        },
    ))
}

fn check(
    id: &str,
    result_uri: Option<&str>,
    operation_id: Option<&str>,
) -> Result<(Value, i32), String> {
    if store::is_persistent(id)? {
        let operation_id = operation_id
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        persistent::check(id, result_uri, &operation_id)
    } else {
        check_legacy(id, result_uri)
    }
}

pub fn dispatch(args: &[String]) -> Result<(Value, i32), String> {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["init", id, "--flow", flow] => Ok((init(id, flow, None, None)?, 0)),
        ["init", id, "--flow", flow, "--wish", uri] => Ok((init(id, flow, Some(uri), None)?, 0)),
        ["init", id, "--flow", flow, "--checks", uri] => Ok((init(id, flow, None, Some(uri))?, 0)),
        ["init", id, "--flow", flow, "--wish", wish, "--checks", checks] => {
            Ok((init(id, flow, Some(wish), Some(checks))?, 0))
        }
        ["init", id, "--flow", flow, "--checks", checks, "--wish", wish] => {
            Ok((init(id, flow, Some(wish), Some(checks))?, 0))
        }
        ["next", id] => Ok((observe(id)?.2, 0)),
        ["check", id] => check(id, None, None),
        ["check", id, "--result", uri] => check(id, Some(uri), None),
        ["check", id, "--operation-id", operation] => check(id, None, Some(operation)),
        ["check", id, "--result", uri, "--operation-id", operation] => {
            check(id, Some(uri), Some(operation))
        }
        ["check", id, "--operation-id", operation, "--result", uri] => {
            check(id, Some(uri), Some(operation))
        }
        _ => Err(format!("invalid arguments\n{HELP}")),
    }
}
