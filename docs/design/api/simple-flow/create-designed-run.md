# `create_designed_run`

> `xdesign::simple_flow::factory` · function · implemented SimpleFlow slice · realizes `SimpleFlowRunFactory::createRun` ([D03](../../flows/simple-flow/implementation.md#d03))

<a id="u70"></a>

## create_designed_run

```rust
pub fn create_designed_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsFlowRun, RsPlanError>
```

Build one run of the designed simple flow from a plan: check the plan's blueprint
conformance, then delegate to the generic
[`create_run`](../core/create-run.md#u16), which structurally validates the
plan and constructs an `Active` [`RsFlowRun`](../core/rs-flow-run.md#u17) with one
pending visit per stage. The wish and trace stay with the controller; this
function produces the run value it holds.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `plan` | `RsFlowPlan` | must pass [`validate_designed_plan`](validate-designed-plan.md#u69) and [`RsFlowPlan::validate`](../core/rs-flow-plan-validate.md#u26) | Read-only plan data for the run's whole life |
| `wish_model_uri` | `String` | existing repository-relative `.sysml` trace file with normal path components | Independent trace identity |
| `run_record_uri` | `String` | repository-relative JSON record path with normal path components | Passed to each fixed checker invocation |
| `check` | `Box<dyn RsCheckCommand>` | handles the blueprint's four checker specs — the designed flow's adapter is [`RsProcessCheck`](rs-process-check.md#u58) | Fixed-check boundary owned by the run |
| `judgment` | `Box<dyn RsJudge>` | identifies an agent or human | Semantic judgment boundary owned by the run |

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(run)` | `RsFlowRun` | The guarantees of [`create_run`](../core/create-run.md#u16), and the plan equals the designed blueprint |
| `Err(e)` | `RsPlanError` | No run was created; the error names the first failure found |

**Errors** — the conformance variants of
[`validate_designed_plan`](validate-designed-plan.md#u69)
(`InvalidPlanFile` for a wrong `plan_id`, `InvalidDesignedFlow` for a shape or
blueprint mismatch, `InvalidChecker` for a checker identity or rule mismatch),
then every error of [`create_run`](../core/create-run.md#u16): the structural
[`RsPlanError`](../core/rs-plan-error.md#u45) variants plus `MissingWishModelUri`
and `MissingRunRecordUri`.

Panics: none.

**Logic**

```
1. validate_designed_plan(&plan)?   — reject a plan that is not the blueprint
2. return core::factory::create_run(plan, wish_model_uri, run_record_uri,
       check, judgment)             — structural validation and construction
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Those of [`create_run`](../core/create-run.md#u16): reads the frozen plan file during validation, allocates a `run_id`, and takes ownership of the supplied trait objects; does not write a record |
| Complexity | O(stages + checkpoints + rules + plan file bytes) |
| Thread safety | Returns an owned value; `RsFlowRun: Send` because both trait objects require `Send` |

**Example** (illustrative)

```rust
let mut plan = designed_simple_flow_plan();
plan.freeze("cache/example/plan.json")?;
let run = create_designed_run(plan, wish_model_uri, run_record_uri, check, judgment)?;
assert_eq!(run.status(), RsFlowStatus::Active);
```

**See also:** [`designed_simple_flow_plan`](designed-simple-flow-plan.md#u52), [`validate_designed_plan`](validate-designed-plan.md#u69), [`create_run`](../core/create-run.md#u16), [`RsFlowRun`](../core/rs-flow-run.md#u17), [run design D03](../../flows/simple-flow/implementation.md#d03).
