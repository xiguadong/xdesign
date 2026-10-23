# WishVerifyFlow design



WishVerifyFlow is the two-stage judged flow: plan acceptance and outcome acceptance, both semantic. Read the [definition](definition.md) for its stages, gates, and judgment contract; the [implementation](implementation.md) for its run, factory, stub, module design, Rust design, and API mapping; and the [views](views.md) for its coverage and analytic views. The common [flow model](../../flow.md), [lifecycle](../../lifecycle.md), [evidence](../../evidence-and-revisions.md), and [adapter](../../adapters.md) semantics are not repeated here.



<a id="d56"></a>

## Module documents and boundaries

The designed wish-verify flow refines the architecture's two-stage `WishVerifyFlow`
([F03](../../flow.md#f03)) into a gated specialization: the same two
stages in the same order, with an explicit semantic exit gate on each stage. Both gates
are judged by a permitted agent or human; the flow declares no syntactic checkpoint and
no checker command, because the architecture states that nothing in this flow's outcome
is established by a fixed check. The design lives in `mbse/model/detailedDesign.sysml`
after the `DesignedWishVerifyFlow` marker; its selectable usage is
`xdesign.designedWishVerifyFlow` in `mbse/model/composition.sysml`. The
[implementation plan](../../../records/plans/wish-verify.md#p25)
fixes the task, decisions, file ownership, and acceptance cases.

| Document | Design responsibility | Evidence limit |
| --- | --- | --- |
| [Run design](definition.md#d57) | Stage plan, run class, factory, and gate/advance/reopen functions | `xdesign::wish_verify` realizes this slice; the generic controller remains separate |
| [Gates and judgment](definition.md#d61) | The two semantic gates, the judgment contract, and the checkpoint class | No fixed command exists; every pass is a recorded, evidence-citing judgment |
| [Nine-view coverage](views.md#d64) | Applicability of the nine SysML questions and where each view is embedded | Navigation and coverage record, not a figure gallery |
| [Simplest CLI interaction](views.md#d84) | Timeline and swimlane for the same `next` / work / `check` pattern at `wish` and `verify` | Action-order and responsibility projections; local CLI execution has separate evidence |
| [Module design](implementation.md#d65) | Packages, types, judgment interface, key flows, parametrics, actor goals, errors, tests, and status | Working design; no frozen module baseline |
| [Rust UML design](implementation.md#d75) | The `xdesign::wish_verify` module on `xdesign::core`, its realization mapping, and view applicability | The crate slice exists; the API design set records its boundary |

The [requirements baseline](../../../requirements/requirements.md#r01) states the
obligations this flow answers (`designedFlowGatesEveryStage`,
`flowRunIsRealizedByDesign`, `runRealizesItsDesign`), the
[requirements trace](../../../requirements/requirements.md#r02) records the project's
satisfy links — one satisfier per requirement, so this flow's coverage is recorded by
its planned verification case instead ([D69](views.md#d69)) —
and the [stage review checklist](../../../verification/review-criteria.md#v08) states what a CDR
decision would still need. Executed verification is recorded in
[P28](../../../records/plans/wish-verify.md#p28); a declared case
in `mbse/model/verification.sysml` is a plan, not a result.

For a first look at how an agent would use the flow, start with the
[simplest WishVerifyFlow CLI timeline and swimlane](views.md#d84).

Detailed reading order: run design → gates and judgment → view coverage → module design →
Rust UML design, then the [API pages](../../api/wish-verify/index.md#u73)
for the class-and-function contract.
