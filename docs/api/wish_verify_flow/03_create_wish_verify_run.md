# `create_wish_verify_run`

> `xdesign::wish_verify::factory` · function · implemented wish-verify slice · realizes `WishVerifyFlowRunFactory::createRun` ([D59](../../details/wish_verify_flow/01_run_design.md#d59))

<a id="u76"></a>

## U76 create_wish_verify_run

```rust
pub fn create_wish_verify_run(
    plan: RsFlowPlan,
    wish_model_uri: &str,
    run_record_uri: &str,
    judgment: impl RsJudge + Send + 'static,
) -> Result<RsFlowRun, RsPlanError>
```

Build one run of the designed wish-verify flow from a plan: check the plan's blueprint
conformance, then delegate to the generic
[`create_run`](../core/46_create_run.md#u16), which structurally validates the plan and
constructs an `Active` [`RsFlowRun`](../core/40_flow_run.md#u17) with one pending visit
per stage. The generic construction requires a check-command boundary, so this factory
supplies [`RsUnusedCheck`](04_unused_check.md#u77): an all-semantic plan never invokes
it, and the stub fails any call that would prove otherwise. The wish and trace stay
with the controller; this function produces the run value it holds.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `plan` | `RsFlowPlan` | must pass [`validate_wish_verify_plan`](02_validate_wish_verify_plan.md#u75) and [`RsFlowPlan::validate`](../core/11_flow_plan_validate.md#u26) | Read-only plan data for the run's whole life |
| `wish_model_uri` | `&str` | existing repository-relative `.sysml` trace file with normal path components | Independent trace identity |
| `run_record_uri` | `&str` | repository-relative JSON record path with normal path components | The run's snapshot location |
| `judgment` | `impl RsJudge + Send + 'static` | identifies an agent or human | Semantic judgment boundary owned by the run; both gates of this flow are judged through it |

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(run)` | `RsFlowRun` | The guarantees of [`create_run`](../core/46_create_run.md#u16), and the plan equals the designed two-stage blueprint |
| `Err(e)` | `RsPlanError` | No run was created; the error names the first failure found |

**Errors** — the conformance variants of
[`validate_wish_verify_plan`](02_validate_wish_verify_plan.md#u75)
(`InvalidPlanFile` for a wrong `plan_id`, `InvalidDesignedFlow` for a shape, kind, or
blueprint mismatch, `InvalidChecker` for any declared checker), then every error of
[`create_run`](../core/46_create_run.md#u16): the structural
[`RsPlanError`](../core/08_plan_error.md#u45) variants plus `MissingWishModelUri` and
`MissingRunRecordUri`.

Panics: none.

**Logic**

```
1. validate_wish_verify_plan(&plan)?   — reject a plan that is not the blueprint
2. return core::factory::create_run(plan, wish_model_uri, run_record_uri,
       Box::new(RsUnusedCheck), Box::new(judgment))
                                     — structural validation and construction,
                                       with the never-invoked check stub
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Those of [`create_run`](../core/46_create_run.md#u16): reads the frozen plan file during validation, allocates a `run_id`, and takes ownership of the supplied judge; does not write a record |
| Complexity | O(stages + checkpoints + plan file bytes) |
| Thread safety | Returns an owned value; `RsFlowRun: Send` because the judge requires `Send` |

**Example** (illustrative)

```rust
let mut plan = wish_verify_flow_plan();
plan.freeze("cache/example/wish_verify_plan.json")?;
let run = create_wish_verify_run(plan, wish_model_uri, run_record_uri, judge)?;
assert_eq!(run.status(), RsFlowStatus::Active);
```

**See also:** [`wish_verify_flow_plan`](01_wish_verify_flow_plan.md#u74), [`validate_wish_verify_plan`](02_validate_wish_verify_plan.md#u75), [`RsUnusedCheck`](04_unused_check.md#u77), [`create_run`](../core/46_create_run.md#u16), [`RsFlowRun`](../core/40_flow_run.md#u17), [run design D59](../../details/wish_verify_flow/01_run_design.md#d59).
