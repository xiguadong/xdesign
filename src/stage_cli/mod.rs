mod store;
mod work;

use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
use store::{Registry, Stored};
use work::{Collection, WorkDefinition};
use xdesign::core::{
    create_run, hash_artifact, restore_run, sha256_bytes, valid_digest, RsActivityEntry,
    RsAdvanceOutcome, RsArtifactDigest, RsAssessmentEntry, RsCheckCommand, RsCheckKind,
    RsCheckerSpec, RsCheckpointOutcome, RsFlowRun, RsFlowStatus, RsGateError, RsGateReport,
    RsJudgeAuthority, RsSuppliedJudge,
};
use xdesign::simple_flow::RsProcessCheck;

pub const HELP: &str = "Usage: xdesign init WISH_ID --flow simple-flow|wish-verify
       xdesign next WISH_ID
       xdesign check WISH_ID [--result FILE]
       xdesign --help

Run from the project root. init registers mbse/wishes/WISH_ID.sysml once.
next returns current work without writing. check records outputs, evaluates the
stage's gates and returns correction work, successor work or completion.
Semantic gates require a snapshot-bound judgment in the optional result JSON;
see docs/design/stage-cli.md for the exchange schema. State is in cache/xdesign/.
Exit codes: 0 work/completed, 1 blocked check, 2 invalid request or command error.
Legacy commands: xdesign simple-flow demo ...; xdesign wish-verify demo ...";

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
        stored.registry.run_record_uri.clone(),
        Box::new(RetainedCheck {
            directory: format!("{}/reports", store::directory(&stored.registry.wish_id)),
            missing_outputs,
        }),
        Box::new(judge),
    )?;
    if run.run_id() != stored.registry.run_id {
        return Err("run identity differs from registration".into());
    }
    // The generic restore validates shape/history; this boundary also constrains
    // every stored artifact and evidence path to the local project.
    for visit in run.visits() {
        for artifact in visit.artifact_digests() {
            store::artifact_path(&artifact.uri)?;
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
            "candidate_artifacts": collection.artifacts, "candidate_snapshot_digest": collection.snapshot })
    };
    Ok(
        json!({ "schema_version": 1, "wish_id": stored.registry.wish_id, "run_id": run.run_id(),
        "flow_id": stored.registry.flow_id, "status": run.status(), "completed": completed,
        "state_token": stored.token, "work": projected }),
    )
}

fn observe(id: &str) -> Result<(Stored, RsFlowRun, Value), String> {
    // A reader never takes a writer lock. The final token comparison rejects a
    // mixed view if restore or collection overlapped another process's write.
    for _ in 0..3 {
        let stored = store::load(id)?;
        let run = restore(&stored, unjudged(), false)?;
        let reply = response(&stored, &run)?;
        if store::load(id)?.token == stored.token {
            return Ok((stored, run, reply));
        }
    }
    Err("run changed during inspection; retry next".into())
}

fn init(id: &str, flow: &str) -> Result<Value, String> {
    store::wish_id(id)?;
    let wish = format!("mbse/wishes/{id}.sysml");
    store::artifact_path(&wish)?;
    store::read_file(&wish)?;
    let mut plan = store::blueprint(flow)?;
    let work = WorkDefinition::freeze(flow, &plan, &wish)?;
    let dir = store::directory(id);
    store::state_path(&dir)?;
    fs::create_dir_all("cache/xdesign").map_err(|e| e.to_string())?;
    fs::create_dir(&dir)
        .map_err(|e| format!("wish is already initialized or directory cannot be reserved: {e}"))?;
    let initialize = || -> Result<(), String> {
        let plan_uri = format!("{dir}/plan.json");
        let work_uri = format!("{dir}/work.json");
        let run_record_uri = format!("{dir}/run.json");
        plan.freeze(plan_uri.clone())
            .map_err(|e| format!("cannot freeze plan: {e:?}"))?;
        let work_bytes = serde_json::to_vec_pretty(&work).map_err(|e| e.to_string())?;
        store::write_new(&work_uri, &work_bytes)?;
        let run = create_run(
            plan.clone(),
            wish.clone(),
            run_record_uri.clone(),
            Box::new(RetainedCheck {
                directory: format!("{dir}/reports"),
                missing_outputs: false,
            }),
            Box::new(unjudged()),
        )
        .map_err(|e| format!("cannot initialize run: {e:?}"))?;
        run.write_record()?;
        let registry = Registry {
            schema_version: 1,
            wish_id: id.into(),
            flow_id: flow.into(),
            run_id: run.run_id().into(),
            wish_model_uri: wish,
            plan_uri,
            plan_sha256: plan.plan_sha256.clone(),
            work_uri,
            work_sha256: sha256_bytes(&work_bytes),
            run_record_uri,
        };
        store::write_new(
            &format!("{dir}/registry.json"),
            &serde_json::to_vec_pretty(&registry).map_err(|e| e.to_string())?,
        )
    };
    if let Err(error) = initialize() {
        let _ = fs::remove_dir_all(&dir);
        return Err(error);
    }
    Ok(observe(id)?.2)
}

fn read_result(uri: Option<&str>) -> Result<Option<StageResult>, String> {
    uri.map(|uri| {
        serde_json::from_slice(&store::read_file(uri)?)
            .map_err(|e| format!("invalid result JSON: {e}"))
    })
    .transpose()
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

fn check(id: &str, result_uri: Option<&str>) -> Result<(Value, i32), String> {
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

pub fn dispatch(args: &[String]) -> Result<(Value, i32), String> {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["init", id, "--flow", flow] => Ok((init(id, flow)?, 0)),
        ["next", id] => Ok((observe(id)?.2, 0)),
        ["check", id] => check(id, None),
        ["check", id, "--result", uri] => check(id, Some(uri)),
        _ => Err(format!("invalid arguments\n{HELP}")),
    }
}
