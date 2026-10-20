# xdesign core API overview

> Layer: `xdesign::core` | Status: implemented xdesign slice | Audience:
> application integrators, flow module owners, and reviewers.
> The flow-agnostic infrastructure of the crate — nothing here knows the designed
> four-stage flow. The single crate inventory is
> [docs/api/00_overview.md](../00_overview.md#u14); this page routes within the
> layer only.

<a id="u71"></a>

## U71 What the core layer owns

`xdesign::core` owns the flow-agnostic groups of the [crate
inventory](../00_overview.md#u14): status and error enums; plan types with their
validation utilities; record types; the trait boundaries and their reports; gate
evaluation; the run engine; and generic construction. A plan is any structurally
valid flow; conformance to a designed flow's blueprint belongs to the flow layer
([simple_flow/00_overview.md](../simple_flow/00_overview.md#u72)).

Module-to-file map (one file per module):

| Module | Source file | Inventory groups |
| --- | --- | --- |
| `status` | `src/core/status.rs` | Status and error enums |
| `plan` | `src/core/plan.rs` | Plan types; validation utilities |
| `record` | `src/core/record.rs` | Record types |
| `ports` | `src/core/ports.rs` | Trait boundaries and reports |
| `gate` | `src/core/gate.rs` | Gate evaluation |
| `run` | `src/core/run.rs` | Run engine |
| `factory` | `src/core/factory.rs` | Construction |

Reading order for a newcomer: `status` → `plan` → `record` → `ports` → `gate` →
`run` → `factory` — each module depends only on the ones before it.

Status: implemented. The source moved from `src/simple_flow/` to
`src/core/` with step 3 of the
[restructure plan](../../prj_manage/08_api_layering_restructure_plan.md#p22);
this map matches the code.
