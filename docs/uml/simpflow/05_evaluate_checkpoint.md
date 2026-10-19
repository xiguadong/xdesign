# `evaluate_checkpoint`

> gate · function · implemented SimpleFlow slice · realizes `EvaluateSimpleFlowCheckpoint` ([D11](../../details/simple_flow/02_gates_and_rules.md#d11))

<a id="u20"></a>

## U20 evaluate_checkpoint

```rust
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
) -> Result<RsAssessmentEntry, RsGateError>
```

Evaluate one gate for one visit. The spec's `kind` selects the path: a syntactic spec
runs its checker through the check trait and holds only when every rule holds; a
semantic spec presents its criteria and the visit's artifact and evidence URIs
through the judge trait and records the returned verdict. Either path produces at
most one assessment, cited at the visit's current `artifact_revision`.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `spec` | `&RsCheckpointSpec` | from the run's plan | The gate to evaluate |
| `visit` | `&RsStageVisit` | the run's current visit | Supplies artifact and evidence URIs and the revision |
| `plan_id` | `&str` | `designedSimpleFlow` | Frozen modeled usage identity |
| `plan_uri` | `&str` | frozen plan JSON path | Checker reads exact plan bytes |
| `plan_sha256` | `&str` | SHA-256 of exact plan file bytes | Checker rejects plan drift |
| `wish_model_uri` | `&str` | independent SysML wish trace path | Identifies the wish trace |
| `run_record_uri` | `&str` | runtime JSON run record path | Identifies the executable record to the fixed checker |
| `check` | `&dyn RsCheckCommand` | used only when `spec.kind == Syntactic` | Command invocation boundary |
| `judge` | `&dyn RsJudge` | used only when `spec.kind == Semantic` | Judgment boundary |

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(entry)` | `RsAssessmentEntry` | `checkpoint_id == spec.checkpoint_id`; revision and snapshot digest equal the visited values; syntactic path: `judged_by == Script`, `Passed` only when every rule holds; semantic path: `judged_by` matches the judge's authority |
| `Err(e)` | `RsGateError` | No conclusion was possible; no assessment is produced |

**Errors**

| Variant | Raised when | Caller action |
| --- | --- | --- |
| `RsGateError::CheckerMissing` | syntactic spec with `checker == None` | Reject the plan earlier via `validate`; treat as plan defect |
| `RsGateError::CommandFailed` | artifact bytes differ from the visit snapshot, command failed, report file is absent, or report invariants fail | Record `Unevaluated`; inspect the command and inputs |
| `RsGateError::JudgmentRefused` | judge returned `Unevaluated` or supplied no rationale | Record `Unevaluated`; route to a reasoned judgment |
| `RsGateError::JudgeNotPermitted` | judge authority is not allowed or its identity is empty | Record `Unevaluated`; use a permitted, identified judge |
| `RsGateError::MissingEvidence` | semantic pass has no current visit artifact, cites no evidence, or cites a missing file | Record `Unevaluated`; supply a retained verification artifact before judging |

Panics: none. A rule that does not hold is `RsRuleReport { holds: false }` data, not
an error.

**Logic**

```
1. if !visit.verify_artifacts(): return Err(GateError::CommandFailed)
   match spec.kind:
2.   Syntactic:
3.     checker := spec.checker.ok_or(GateError::CheckerMissing)?
4.     report  := check.run_check(checker, plan_id, plan_uri,
                 plan_sha256, wish_model_uri, run_record_uri,
                 spec.stage_id, visit.artifact_revision,
                 &visit.artifact_snapshot_digest)
                     .map_err(|_| GateError::CommandFailed)?
5.     if report.artifact_revision != visit.artifact_revision or
       report.artifact_snapshot_digest != visit.artifact_snapshot_digest or
       report.report_uri is not a valid relative path to an existing file,
       or the report's rule IDs are not exactly the checker's distinct rule IDs,
       or any failed rule has no nonempty diagnostic,
       or report.holds != every(rule.holds):
           return Err(GateError::CommandFailed)
       holds := report.holds
6.     return Ok(AssessmentEntry {
           outcome: if holds { Passed } else { Failed },
           judged_by: Script, assessor: checker.checker_id,
           evaluated_snapshot_digest: visit.artifact_snapshot_digest.clone(),
           evidence: [report.report_uri], .. })
7.   Semantic:
8.     authority := judge.authority()
       if (authority == Agent and not spec.agent_may_judge) or
          (authority == Human and not spec.human_may_judge) or
          authority == Script or judge.identity() is empty:
              return Err(GateError::JudgeNotPermitted)
9.     evidence := visit.evidence_uris()
       (outcome, rationale) := judge.review(&spec.criteria, &evidence)
10.    if outcome == Unevaluated or rationale is empty:
           return Err(GateError::JudgmentRefused)
       if outcome == Passed and (visit.artifact_digests is empty or
           evidence.is_empty() or any cited file is missing) {
           return Err(GateError::MissingEvidence)
       }
11.    return Ok(AssessmentEntry {
           outcome, judged_by: authority,
           assessor: judge.identity(), rationale, evidence_uris: evidence,
           evaluated_snapshot_digest: visit.artifact_snapshot_digest.clone(), .. })
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Syntactic path executes an external command; semantic path consults an external judge; no state mutation |
| Complexity | O(rules) for the scripted path; the judged path is out of band |
| Thread safety | Shared references only; re-entrant |

**Example** (illustrative)

```rust
let entry = evaluate_checkpoint(spec, visit, plan_id, plan_uri, plan_sha256,
    wish_model_uri, run_record_uri, check, judge)?;
```

**See also:** [`RsSimpleFlowRun::evaluate_gates`](03_evaluate_gates.md#u18), [`RsCheckCommand::run_check`](07_run_check.md#u22), [`RsJudge::review`](09_review.md#u24), [`RsGateError`](31_gate_error.md#u46), [checkpoint evaluation D11](../../details/simple_flow/02_gates_and_rules.md#d11).
