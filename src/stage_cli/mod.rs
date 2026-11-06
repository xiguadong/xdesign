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

#[derive(Debug)]
enum StageError {
    Store(xdesign::storage::RsStoreError),
    Message(String),
}
impl From<String> for StageError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}
impl From<&str> for StageError {
    fn from(value: &str) -> Self {
        Self::Message(value.into())
    }
}
impl From<xdesign::storage::RsStoreError> for StageError {
    fn from(value: xdesign::storage::RsStoreError) -> Self {
        Self::Store(value)
    }
}
impl From<xdesign::storage::cli::WithProjectStoreError<StageError>> for StageError {
    fn from(value: xdesign::storage::cli::WithProjectStoreError<StageError>) -> Self {
        match value {
            xdesign::storage::cli::WithProjectStoreError::Store(e) => Self::Store(e),
            xdesign::storage::cli::WithProjectStoreError::Action(e) => e,
        }
    }
}
impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => {
                write!(f, "{e}")?;
                use xdesign::storage::RsStoreError;
                let id = match e {
                    RsStoreError::Io { operation_id, .. }
                    | RsStoreError::Database { operation_id, .. } => operation_id.as_deref(),
                    RsStoreError::OperationMismatch { operation_id }
                    | RsStoreError::UnknownOutcome { operation_id } => Some(operation_id.as_str()),
                    _ => None,
                };
                if let Some(id) = id {
                    write!(f, " (operation_id={id})")?;
                }
                Ok(())
            }
            Self::Message(s) => write!(f, "stage_error; {s}"),
        }
    }
}

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

fn observe(
    id: &str,
    handle: Option<&xdesign::storage::RsStore>,
) -> Result<(Stored, RsFlowRun, Value), StageError> {
    // A reader never takes a writer lock. The final comparison rejects a mixed
    // view if restoration or collection overlapped another process's commit.
    for _ in 0..3 {
        let stored = store::load(id, handle)?;
        let run = restore(&stored, unjudged(), false)?;
        let reply = response(&stored, &run)?;
        let current = store::load(id, handle)?;
        if current.token == stored.token && current.store_revision == stored.store_revision {
            return Ok((stored, run, reply));
        }
    }
    Err("run changed during inspection; retry next".into())
}

fn init(
    id: &str,
    flow: &str,
    wish: Option<&str>,
    checks: Option<&str>,
    handle: &xdesign::storage::RsStore,
) -> Result<Value, StageError> {
    persistent::init(id, flow, wish, checks, handle)
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

fn check_legacy(id: &str, result_uri: Option<&str>) -> Result<(Value, i32), StageError> {
    let handle = None;
    let (initial, initial_run, mut initial_reply) = observe(id, handle)?;
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
    let stored = store::load(id, handle)?;
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
        return Err(format!("cannot retain assessment: {error}").into());
    }
    let assessed = json!({ "stage_id": run.current().stage_id(), "visit_index": run.current().visit_index(),
        "artifact_revision": run.current().artifact_revision(), "artifact_snapshot_digest": run.current().artifact_snapshot_digest(),
        "assessments": assessments });
    let advance = run.advance();
    if let Some(error) = run.last_error() {
        return Err(format!("cannot retain advancement: {error}").into());
    }
    let final_state = store::load(id, handle)?;
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
    handle: Option<&xdesign::storage::RsStore>,
) -> Result<(Value, i32), StageError> {
    if store::is_persistent(id, handle)? {
        let operation_id = operation_id
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        persistent::check(
            id,
            result_uri,
            &operation_id,
            handle.ok_or("missing persistent handle")?,
        )
    } else {
        check_legacy(id, result_uri)
    }
}

fn dispatch_guarded(
    args: &[String],
    handle: Option<&xdesign::storage::RsStore>,
) -> Result<(Value, i32), StageError> {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["init", id, "--flow", flow] => Ok((
            init(
                id,
                flow,
                None,
                None,
                handle.ok_or("missing persistent handle")?,
            )?,
            0,
        )),
        ["init", id, "--flow", flow, "--wish", uri] => Ok((
            init(
                id,
                flow,
                Some(uri),
                None,
                handle.ok_or("missing persistent handle")?,
            )?,
            0,
        )),
        ["init", id, "--flow", flow, "--checks", uri] => Ok((
            init(
                id,
                flow,
                None,
                Some(uri),
                handle.ok_or("missing persistent handle")?,
            )?,
            0,
        )),
        ["init", id, "--flow", flow, "--wish", wish, "--checks", checks] => Ok((
            init(
                id,
                flow,
                Some(wish),
                Some(checks),
                handle.ok_or("missing persistent handle")?,
            )?,
            0,
        )),
        ["init", id, "--flow", flow, "--checks", checks, "--wish", wish] => Ok((
            init(
                id,
                flow,
                Some(wish),
                Some(checks),
                handle.ok_or("missing persistent handle")?,
            )?,
            0,
        )),
        ["next", id] => {
            if let Some(store) = handle {
                if let Some(response) = xdesign::storage::cli::read_completed_project(store, id)? {
                    return Ok((response, 0));
                }
            }
            Ok((observe(id, handle)?.2, 0))
        }
        ["check", id] => check(id, None, None, handle),
        ["check", id, "--result", uri] => check(id, Some(uri), None, handle),
        ["check", id, "--operation-id", operation] => check(id, None, Some(operation), handle),
        ["check", id, "--result", uri, "--operation-id", operation] => {
            check(id, Some(uri), Some(operation), handle)
        }
        ["check", id, "--operation-id", operation, "--result", uri] => {
            check(id, Some(uri), Some(operation), handle)
        }
        _ => Err(format!("invalid arguments\n{HELP}").into()),
    }
}

fn backend_error(message: String) -> StageError {
    StageError::Store(xdesign::storage::RsStoreError::Integrity(message))
}
fn conflict() -> StageError {
    backend_error("location_reason=backend_conflict; legacy and persistent histories share this wish ID; use read-only inspect --backend legacy|store".into())
}
fn validate_cache(
    id: &str,
    cache: &crate::project_backend::Cache,
    handle: &xdesign::storage::RsStore,
) -> Result<(), StageError> {
    use crate::project_backend::Cache;
    let exists = persistent::item_exists(id, handle)?;
    match cache {
        Cache::Legacy(_) => {
            let project = std::env::current_dir().map_err(|e| e.to_string())?;
            if exists || xdesign::storage::cli::project_candidate_contains(handle, &project, id)? {
                return Err(conflict());
            }
        }
        Cache::Persistent(registry) => {
            if !exists {
                return Err(backend_error("location_reason=backend_ambiguous; cached persistent registration has no selected item".into()));
            }
            let raw =
                fs::read(format!("cache/xdesign/{id}/registry.json")).map_err(|e| e.to_string())?;
            let mut proven = false;
            for revision in handle.history(id)? {
                let uri = format!("cache/xdesign/{id}/registry.json");
                if handle.read_material(id, Some(revision.revision), &uri)? == raw
                    && revision.metadata.get("flow_id").and_then(Value::as_str)
                        == Some(&registry.flow_id)
                {
                    proven = true;
                    break;
                }
            }
            if !proven {
                return Err(backend_error("location_reason=backend_ambiguous; cache registration has no exact store provenance".into()));
            }
        }
        Cache::Absent => {}
    }
    Ok(())
}
fn inspect_backend(
    id: &str,
    backend: &str,
    revision: Option<u64>,
    handle: Option<&xdesign::storage::RsStore>,
) -> Result<(Value, i32), StageError> {
    if backend == "legacy" {
        if revision.is_some() {
            return Err("legacy inspection has no revision selector".into());
        }
        let stored = store::load_legacy(id)?;
        let run = restore(&stored, unjudged(), false)?;
        return Ok((
            json!({"backend":"legacy","registry":stored.registry,"plan":serde_json::from_slice::<Value>(&store::read_file(&stored.registry.plan_uri)?).map_err(|e|e.to_string())?,"work":serde_json::from_slice::<Value>(&store::read_file(&stored.registry.work_uri)?).map_err(|e|e.to_string())?,"run":serde_json::from_slice::<Value>(&store::read_file(&stored.registry.run_record_uri)?).map_err(|e|e.to_string())?,"completed":run.status()==RsFlowStatus::Completed}),
            0,
        ));
    }
    if backend != "store" {
        return Err("inspection backend must be legacy or store".into());
    }
    let handle = handle.ok_or("no selected persistent store")?;
    let head = handle.get(id, revision)?;
    let mut result = json!({"backend":"store","revision":head,"history":handle.history(id)?});
    for (field, uri) in [
        ("registry", format!("cache/xdesign/{id}/registry.json")),
        ("plan", format!("cache/xdesign/{id}/plan.json")),
        ("work", format!("cache/xdesign/{id}/work.json")),
        ("run", format!("cache/xdesign/{id}/run.json")),
    ] {
        result[field] =
            serde_json::from_slice(&handle.read_material(id, Some(head.revision), &uri)?)
                .map_err(|e| e.to_string())?;
    }
    Ok((result, 0))
}
pub fn dispatch(args: &[String]) -> Result<(Value, i32), String> {
    dispatch_command(args).map_err(|e| e.to_string())
}
fn dispatch_command(args: &[String]) -> Result<(Value, i32), StageError> {
    use crate::project_backend::{self, Cache};
    use xdesign::storage::{cli, RsStore};
    let id = args.get(1).ok_or("missing wish ID")?;
    store::wish_id(id)?;
    let project = std::env::current_dir()
        .and_then(fs::canonicalize)
        .map_err(|e| e.to_string())?;
    let cache = project_backend::classify(&project, id).map_err(backend_error)?;
    let inspection = if args.first().is_some_and(|a| a == "inspect") {
        if args.get(2).map(String::as_str) != Some("--backend") || ![4, 6].contains(&args.len()) {
            return Err("inspect requires --backend legacy|store and optional --revision N".into());
        }
        let revision = if args.len() == 6 {
            if args[4] != "--revision" {
                return Err("unknown inspect option".into());
            }
            Some(args[5].parse::<u64>().map_err(|e| e.to_string())?)
        } else {
            None
        };
        Some((args[3].as_str(), revision))
    } else {
        None
    };
    if inspection.is_some_and(|(b, _)| b == "legacy") {
        let _lease = project_backend::LegacyLease::acquire(&project).map_err(backend_error)?;
        return inspect_backend(id, "legacy", inspection.unwrap().1, None);
    }
    let explicit = std::env::var_os("XDESIGN_STORE_ROOT").map(std::path::PathBuf::from);
    if explicit.as_ref().is_some_and(|p| p.as_os_str().is_empty()) {
        return Err(StageError::Store(
            xdesign::storage::RsStoreError::InvalidInput(
                "location_reason=invalid_input; empty store root override".into(),
            ),
        ));
    }
    let candidates =
        project_backend::candidates(&project, explicit.as_deref()).map_err(backend_error)?;
    let mut existing = None;
    for root in &candidates {
        match fs::symlink_metadata(root) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(backend_error(format!(
                    "location_reason=backend_ambiguous; {e}"
                )))
            }
            Ok(_) => {}
        }
        let h = RsStore::open(root).map_err(|e| {
            backend_error(format!(
                "location_reason=backend_ambiguous; cannot inspect known candidate {}: {e}",
                root.display()
            ))
        })?;
        if persistent::item_exists(id, &h)?
            && matches!(cache, Cache::Legacy(_))
            && inspection.is_none()
        {
            return Err(conflict());
        }
        existing = Some(root.clone());
    }
    if matches!(cache, Cache::Legacy(_)) && inspection.is_none() {
        if args.first().is_some_and(|a| a == "init") {
            return Err("wish already has a legacy registration".into());
        }
        if let Some(explicit) = explicit.as_ref() {
            let selected = if explicit.is_absolute() {
                explicit.clone()
            } else {
                project.join(explicit)
            };
            if selected.exists() {
                existing = Some(selected);
            }
        }
        if let Some(root) = existing {
            let _root_context = persistent::RootContext::enter(root.clone());
            return cli::with_project_store(&project, &root, false, |h| {
                let now = project_backend::classify(&project, id).map_err(backend_error)?;
                validate_cache(id, &now, h)?;
                dispatch_guarded(args, None)
            })
            .map_err(StageError::from);
        }
        let _lease = project_backend::LegacyLease::acquire(&project).map_err(backend_error)?;
        if project_backend::candidates(&project, explicit.as_deref())
            .map_err(backend_error)?
            .iter()
            .any(|p| p.exists())
        {
            return Err(backend_error("location_reason=backend_ambiguous; persistent candidate appeared during classification".into()));
        }
        project_backend::classify(&project, id).map_err(backend_error)?;
        return dispatch_guarded(args, None);
    }
    let initialize = args.first().is_some_and(|a| a == "init");
    let root = cli::resolve_store_root(&project, None, initialize)?;
    let _root_context = persistent::RootContext::enter(root.clone());
    cli::with_project_store(&project, &root, initialize, |h| {
        let cache = project_backend::classify(&project, id).map_err(backend_error)?;
        if let Some((backend, revision)) = inspection {
            return inspect_backend(id, backend, revision, Some(h));
        }
        validate_cache(id, &cache, h)?;
        dispatch_guarded(args, Some(h))
    })
    .map_err(StageError::from)
}
