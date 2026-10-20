# xdesign wish_verify API overview

> Layer: `xdesign::wish_verify` | Status: implemented wish-verify slice | Audience:
> application integrators, flow module owners, and reviewers.
> The designed two-stage wish-verify flow built on `xdesign::core`. The single crate
> inventory is [docs/api/00_overview.md](../00_overview.md#u14); this page routes
> within the layer only.

<a id="u73"></a>

## U73 What the wish-verify layer owns

`xdesign::wish_verify` owns exactly four APIs — the designed judged flow and nothing
else:

1. [01_wish_verify_flow_plan.md](01_wish_verify_flow_plan.md#u74) — the blueprint as
   data: the modeled `designedWishVerifyFlow` as an `RsFlowPlan` before freezing binds
   its bytes.
2. [02_validate_wish_verify_plan.md](02_validate_wish_verify_plan.md#u75) — blueprint
   conformance: reject any plan that differs from the blueprint, including one carrying
   a syntactic gate.
3. [03_create_wish_verify_run.md](03_create_wish_verify_run.md#u76) — the designed-run
   factory: conformance, then construction through the generic engine.
4. [04_unused_check.md](04_unused_check.md#u77) — `RsUnusedCheck`, the never-invoked
   check stub that keeps the generic engine's check boundary honest for an
   all-semantic plan.

Module-to-file map (one file per module):

| Module | Source file | APIs |
| --- | --- | --- |
| `blueprint` | `src/wish_verify/blueprint.rs` | `wish_verify_flow_plan`, `validate_wish_verify_plan` |
| `factory` | `src/wish_verify/factory.rs` | `create_wish_verify_run`, `RsUnusedCheck` |

Reading order: 01 → 04 — blueprint data, conformance, construction, then the stub.

The layer builds on core: [`create_run`](../core/46_create_run.md#u16) constructs the
run after conformance, and [`RsFlowRun`](../core/40_flow_run.md#u17) is the engine that
executes it — a run whose plan is the frozen `designedWishVerifyFlow` blueprint plays
the `WishVerifyFlowRun` role. Both gates are semantic, so the layer has no checker
adapter; [`RsJudge`](../core/31_judge.md#u23) is the only automation boundary the run
uses. The design this layer realizes is the
[wish-verify module design](../../details/wish_verify_flow/04_module_design.md#d65);
the [implementation plan](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p25)
records the sequence and its acceptance cases.
