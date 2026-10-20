# `create_run`

> `xdesign::core::factory` · function · implemented xdesign slice · language-side construction of a `FlowExecution` (model `RsCreateRun` → `XDesignArchitecture::FlowExecution`, [E01](../../details/02_records_and_evidence.md#e01))

<a id="u16"></a>

## U16 create_run

```rust
pub fn create_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsFlowRun, RsPlanError>
```

Build one run from any structurally valid plan: validate the plan, build one
pending visit per stage, activate the initial stage's visit, then construct an
`Active` run. The wish model URI identifies the independent SysML trace; the run record
URI identifies the JSON snapshot passed to fixed checkers. The wish and trace
stay with the controller; this function produces the run value it holds.

The function is generic over any plan: [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26)
performs **structural** validation only. Conformance to a designed flow's blueprint
is the flow module's concern — for the designed simple flow,
[`validate_designed_plan`](../simple_flow/02_validate_designed_plan.md#u69) — and is
checked by the flow's own factory before it delegates here.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `plan` | `RsFlowPlan` | must pass [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26) | Read-only plan data for the run's whole life |
| `wish_model_uri` | `String` | existing repository-relative `.sysml` trace file with normal path components | Independent trace identity |
| `run_record_uri` | `String` | repository-relative JSON record path with normal path components | Passed to each fixed checker invocation |
| `check` | `Box<dyn RsCheckCommand>` | handles every checker spec in the plan | Fixed-check boundary owned by the run |
| `judgment` | `Box<dyn RsJudge>` | identifies an agent or human | Semantic judgment boundary owned by the run |

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(run)` | `RsFlowRun` | `run.status() == RsFlowStatus::Active`; `plan_id`, `plan_uri`, and `plan_sha256` copy the validated frozen plan; `visits.len() == plan.stages.len()` in stage order; the initial stage's visit is `Active` with `artifact_revision == 0`, all others `Pending` |
| `Err(e)` | `RsPlanError` | No run was created; the error names the first inconsistency found |

**Errors** — `plan.validate()` may return any of the structural variants listed at
[`RsFlowPlan::validate`](11_flow_plan_validate.md#u26). After validation succeeds,
an invalid, nonexistent, or non-`.sysml` wish path returns
`MissingWishModelUri`; an invalid repository-relative run record path returns
`MissingRunRecordUri`. The initial-stage lookup also returns
`MissingInitialStage` if its validated invariant is unexpectedly broken.

Panics: none.

**Logic**

```
1. plan.validate()?            — reject an inconsistent plan
   if !valid_relative_path(wish_model_uri) or !ends_with('.sysml')
      or !file_exists(wish_model_uri) → MissingWishModelUri
   if !valid_relative_path(run_record_uri) → MissingRunRecordUri
2. visits := plan.stages().map(|stage| RsStageVisit::pending(stage.stage_id, 0))
3. current_visit := index of plan.initial_stage_id in plan.stages
   visits[current_visit].status := Active
4. return Ok(RsFlowRun {
       run_id: new_uuid(),
       plan_id: plan.plan_id.clone(), plan_uri: plan.plan_uri.clone(),
       plan_sha256: plan.plan_sha256.clone(),
       plan, wish_model_uri, run_record_uri, status: Active,
       visits, current_visit,
       check, judgment, persistence_error: false, last_error: None
   })
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Reads the frozen plan file during validation, allocates a `run_id`, and takes ownership of the supplied trait objects; does not write a record |
| Complexity | O(number of stages) = O(4) for the designed flow |
| Thread safety | Returns an owned value; `RsFlowRun: Send` because both trait objects require `Send` |

**Example** (illustrative)

```rust
let run = create_run(plan, wish_model_uri, run_record_uri, check, judgment)?;
assert_eq!(run.status(), RsFlowStatus::Active);
```

For the designed simple flow, [`create_designed_run`](../simple_flow/03_create_designed_run.md#u70)
checks blueprint conformance and then delegates to this function.

**See also:** [`RsFlowRun`](40_flow_run.md#u17), [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26), [`RsPlanError`](08_plan_error.md#u45), [`create_designed_run`](../simple_flow/03_create_designed_run.md#u70), [run design D03](../../details/simple_flow/01_run_design.md#d03).
