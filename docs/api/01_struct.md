# Crate structure

> Project: xdesign | Status: implemented slice in three layers | Audience:
> application integrators, module owners, and reviewers.

<a id="u68"></a>

## U68 Crate structure

The `xdesign` crate is divided into three layers, and dependencies point one way:
`simple_flow` and `wish_verify` may depend on `core`, never the reverse, and the two
flow layers do not depend on each other. `core` owns the
flow-agnostic infrastructure — status vocabulary, plan data and structural
validation, records, the check-command and judge trait boundaries, gate
evaluation, the run engine, and generic construction. `simple_flow` owns the
designed four-stage flow built on it: the blueprint as data, blueprint
conformance, the designed-run factory, and the fixed-checker adapter.
`wish_verify` owns the designed two-stage judged flow: the blueprint as data,
blueprint conformance, the designed-run factory, and the unused-check stub — it
declares no checker adapter because both its gates are semantic. A further
designed flow becomes another sibling flow layer, building on `core` the
same way.

![Crate layers](assets/01_crate_layers.svg)

> **Figure U68-1 — Crate layers.**
> **Question:** How is the xdesign crate divided into layers, and which direction may dependencies point?
> **Model view:** `XDesignRustUml::rustCrateLayers` (GeneralView projection).
> **Companions:** the module-level layout and class views in [D45–D46](../details/simple_flow/06_rust_uml_design.md#d45) and [D76–D77](../details/wish_verify_flow/05_rust_uml_design.md#d76).
> **Status:** renderable-now.
> **Reading:** The crate contains `core`, `simpleFlow`, and `wishVerify`; each flow layer depends on `core`; `core` depends on nothing outside itself.
> **Limits:** The figure shows layer and module dependencies, not individual types; class-level relations stay in the module design.

| Layer | API directory | Source module | Group overview |
| --- | --- | --- | --- |
| `core` | [core/](core/00_overview.md#u71) | `src/core/` | [core/00_overview.md](core/00_overview.md#u71) |
| `simple_flow` | [simple_flow/](simple_flow/00_overview.md#u72) | `src/simple_flow/` | [simple_flow/00_overview.md](simple_flow/00_overview.md#u72) |
| `wish_verify` | [wish_verify_flow/](wish_verify_flow/00_overview.md#u73) | `src/wish_verify/` | [wish_verify_flow/00_overview.md](wish_verify_flow/00_overview.md#u73) |

The `core` and `simple_flow` source modules landed with step 3 of the
[restructure plan](../prj_manage/08_api_layering_restructure_plan.md#p22);
`src/wish_verify/` lands with the
[wish-verify implementation plan](../prj_manage/09_wish_verify_flow_implementation_plan.md#p27).
The model's `XDesignRustUml::crate` packages and this directory layout are the
contract they move to.
