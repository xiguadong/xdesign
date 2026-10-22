# Project records

> Status: first executable SimpleFlow slice recorded. This directory records
> decisions and acceptance work; it does not itself authorize a wish.

<a id="p01"></a>

## Reading order and ownership

Read the [review synthesis](reviews/initial-synthesis.md#p02), then the
[SimpleFlow implementation plan](plans/simple-flow.md#p03),
[initial execution record](executions/simple-flow.md#p08), and
[review fix record](reviews/simple-flow-fixes.md#p11), followed by the
[cross-layer review reconciliation](reviews/cross-layer-reconciliation.md#p14) and
[xdesign integration reconciliation](reviews/integration-reconciliation.md#p17).
The [API layering restructure plan](plans/api-layering.md#p20)
records the core/simple_flow split of the model, the `docs/api/` reference, and
the crate source. The
[WishVerifyFlow implementation plan](plans/wish-verify.md#p25)
records the designed two-stage judged flow: its model, documentation, the
`xdesign::wish_verify` module, and its acceptance cases.
The [SimpleFlow file plan and verification cases](plans/simple-flow-file-plan.md#d53)
record source ownership and planned acceptance in this project-management area. The project SysML model and per-wish models remain
authoritative for structure and recorded history; these pages define the work sequence,
acceptance checks, and limits of this implementation slice.

The plan uses the task, analysis, implementation, and acceptance sections of the local
`cn_todos` task template as a structural reference. The repository's English language,
model, documentation, and cache rules apply. Update the plan when scope or acceptance
changes, and record the verification actually run before calling an item complete.
