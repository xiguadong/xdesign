# `create_run`

> factory · function · designed-only · realizes `SimpleFlowRunFactory::createRun` ([D03](../../details/simple_flow/01_run_design.md#d03))

<a id="u16"></a>

## U16 create_run

```rust
pub fn create_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsSimpleFlowRun, RsPlanError>
```

Build one run of the designed simple flow from a plan: validate the plan, create the
run, push one pending visit per stage, activate the initial stage's visit and the run
itself. The wish model URI identifies the independent SysML trace; the run record
URI identifies the JSON snapshot passed to fixed checkers. The wish and trace
stay with the controller; this function produces the run value it holds.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `plan` | `RsFlowPlan` | must pass [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26) | Read-only plan data for the run's whole life |
| `wish_model_uri` | `String` | non-empty, repository-relative SysML trace path | Independent trace identity |
| `run_record_uri` | `String` | non-empty, repository-relative JSON record path | Passed to each fixed checker invocation |
| `check` | `Box<dyn RsCheckCommand>` | handles every checker spec in the plan | Fixed-check boundary owned by the run |
| `judgment` | `Box<dyn RsJudge>` | identifies an agent or human | Semantic judgment boundary owned by the run |

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(run)` | `RsSimpleFlowRun` | `run.status() == RsFlowStatus::Active`; `plan_id`, `plan_uri`, and `plan_sha256` copy the validated frozen plan; `visits.len() == plan.stages.len()` in stage order; the initial stage's visit is `Active` with `artifact_revision == 0`, all others `Pending` |
| `Err(e)` | `RsPlanError` | No run was created; the error names the first inconsistency found |

**Errors**

| Variant | Raised when | Caller action |
| --- | --- | --- |
| `RsPlanError::MissingInitialStage` | `initial_stage_id` matches no stage | Fix the flow definition data |
| `RsPlanError::DuplicateStageId` | two stages share an id | Fix the flow definition data |
| `RsPlanError::UnknownStageReference` | a `next_stage_ids` entry matches no stage | Fix the flow definition data |
| `RsPlanError::SyntacticSpecWithoutChecker` | a syntactic gate has no checker | Supply the checker or reclassify the gate |
| `RsPlanError::SemanticSpecWithChecker` | a semantic gate names a checker | Remove the checker or reclassify the gate |
| `RsPlanError::MissingWishModelUri` | the wish model path is empty | Supply the independent wish trace path |
| `RsPlanError::MissingRunRecordUri` | the run record path is empty | Supply the runtime JSON record path |

Panics: none.

**Logic**

```
1. plan.validate()?            — reject an inconsistent plan
   if wish_model_uri.is_empty() → MissingWishModelUri
   if run_record_uri.is_empty() → MissingRunRecordUri
2. run := RsSimpleFlowRun {
       run_id: new_uuid(),
       plan_id: plan.plan_id.clone(), plan_uri: plan.plan_uri.clone(),
       plan_sha256: plan.plan_sha256.clone(),
       plan, wish_model_uri, run_record_uri, status: Pending,
       visits: [], current_visit: 0,
       check, judgment          — trait objects supplied by the caller
   }
3. for stage in plan.stages():
       run.visits.push(RsStageVisit::pending(stage.stage_id, 0))
4. run.visits[initial].status       := Active
   run.current_visit := initial_index
5. run.status := Active
6. return Ok(run)
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Reads the frozen plan file during validation, allocates a `run_id`, and takes ownership of the supplied trait objects; does not write a record |
| Complexity | O(number of stages) = O(4) for the designed flow |
| Thread safety | Returns an owned value; `RsSimpleFlowRun: Send` because both trait objects require `Send` |

**Example** (designed-only)

```rust
let plan = designed_simple_flow_plan();
let mut run = create_run(plan, wish_model_uri, run_record_uri, check, judgment)?;
assert_eq!(run.status(), RsFlowStatus::Active);
```

**See also:** [`RsSimpleFlowRun`](02_simple_flow_run.md#u17), [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26), [`RsPlanError`](30_plan_error.md#u45), [run design D03](../../details/simple_flow/01_run_design.md#d03).
