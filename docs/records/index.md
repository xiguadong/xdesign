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
The [stage CLI implementation plan](plans/stage-cli.md#p30)
records wish-0010's SimpleFlow implementation sequence for the flat `next` and
`check` commands and local run restoration. Its
[execution record](plans/stage-cli.md#p34) retains the completed technical
acceptance run, observed test coverage, remaining limits, and pending final
human acceptance. The [storage redesign and implementation plan](plans/storage.md#p35)
records wish-0011's SimpleFlow route for durable operational storage, its acceptance
cases, executed evidence, and remaining limits. The
[storage acceptance-challenge review fixes](reviews/storage-challenge.md#p40) record the
nine reproduced defects, their dispositions, the recovery-boundary decision, and the
executed re-verification. The
[storage acceptance-challenge round-2 review fixes](reviews/storage-challenge-round2.md#p42)
record the six further reproduced defects, the envelope-budget, bounded save working-set,
lock-visibility, root-allowlist, materializability, and request-recovery decisions, and
the executed re-verification; the requester's final human acceptance is recorded in the
[storage plan](plans/storage.md#p44).
The [SimpleFlow file plan and verification cases](plans/simple-flow-file-plan.md#d53)
record source ownership and planned acceptance in this project-management area. The project SysML model and per-wish models remain
authoritative for structure and recorded history; these pages define the work sequence,
acceptance checks, and limits of this implementation slice.

The plan uses the task, analysis, implementation, and acceptance sections of the local
`cn_todos` task template as a structural reference. The repository's English language,
model, documentation, and cache rules apply. Update the plan when scope or acceptance
changes, and record the verification actually run before calling an item complete.
