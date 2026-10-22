# xdesign simple_flow API overview

> Layer: `xdesign::simple_flow` | Status: implemented SimpleFlow slice | Audience:
> application integrators, flow module owners, and reviewers.
> The designed four-stage flow built on `xdesign::core`. The single crate
> inventory is [API inventory](../index.md#u14); this page routes
> within the layer only.

<a id="u72"></a>

## What the flow layer owns

`xdesign::simple_flow` owns exactly four APIs — the designed flow and nothing
else:

1. [01_designed_simple_flow_plan.md](designed-simple-flow-plan.md#u52) — the
   blueprint as data: the modeled `designedSimpleFlow` as an `RsFlowPlan` before
   freezing binds its bytes.
2. [02_validate_designed_plan.md](validate-designed-plan.md#u69) — blueprint
   conformance: reject any plan that differs from the blueprint.
3. [03_create_designed_run.md](create-designed-run.md#u70) — the designed-run
   factory: conformance, then construction.
4. [04_process_check.md](rs-process-check.md#u58) — the fixed-checker adapter:
   `RsProcessCheck`, bound to the four `tools/checks/simple_*.py` commands.

Module-to-file map (one file per module):

| Module | Source file | APIs |
| --- | --- | --- |
| `blueprint` | `src/simple_flow/blueprint.rs` | `designed_simple_flow_plan`, `validate_designed_plan` |
| `factory` | `src/simple_flow/factory.rs` | `create_designed_run` |
| `check` | `src/simple_flow/check.rs` | `RsProcessCheck` |

Reading order: 01 → 04 — blueprint data, conformance, construction, then the
checker adapter.

The layer builds on core: [`create_run`](../core/create-run.md#u16) constructs
the run after conformance, and [`RsFlowRun`](../core/rs-flow-run.md#u17) is the
engine that executes it — a run whose plan is the frozen `designedSimpleFlow`
blueprint plays the `SimpleFlowRun` role. The split's source move landed with
step 3 of the
[restructure plan](../../../records/plans/api-layering.md#p22);
the code now lives in `src/core/` + `src/simple_flow/`.
