# WishVerifyFlow detailed design overview

> Project: xdesign | Status: designed two-stage flow with an implemented Rust slice;
> executed behavior verification recorded in the plan ([P28](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p28)) | Audience: module
> implementers, integrators, verifiers, and reviewers. No CDR or project-level
> acceptance is claimed.

<a id="d56"></a>

## D56 Module documents and boundaries

The designed wish-verify flow refines the architecture's two-stage `WishVerifyFlow`
([F03](../01_flows_and_checkpoints.md#f03)) into a gated specialization: the same two
stages in the same order, with an explicit semantic exit gate on each stage. Both gates
are judged by a permitted agent or human; the flow declares no syntactic checkpoint and
no checker command, because the architecture states that nothing in this flow's outcome
is established by a fixed check. The design lives in `mbse/model/detailedDesign.sysml`
after the `DesignedWishVerifyFlow` marker; its selectable usage is
`xdesign.designedWishVerifyFlow` in `mbse/model/composition.sysml`. The
[implementation plan](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p25)
fixes the task, decisions, file ownership, and acceptance cases.

| Document | Design responsibility | Evidence limit |
| --- | --- | --- |
| [Run design](01_run_design.md#d57) | Stage plan, run class, factory, and gate/advance/reopen functions | `xdesign::wish_verify` realizes this slice; the generic controller remains separate |
| [Gates and judgment](02_gates_and_judgment.md#d61) | The two semantic gates, the judgment contract, and the checkpoint class | No fixed command exists; every pass is a recorded, evidence-citing judgment |
| [Nine-view coverage](03_sysml_diagrams.md#d64) | Applicability of the nine SysML questions and where each view is embedded | Navigation and coverage record, not a figure gallery |
| [Module design](04_module_design.md#d65) | Packages, types, judgment interface, key flows, parametrics, actor goals, errors, tests, and status | Working design; no frozen module baseline |
| [Rust UML design](05_rust_uml_design.md#d75) | The `xdesign::wish_verify` module on `xdesign::core`, its realization mapping, and view applicability | The crate slice exists; the API reference records its boundary |

The [requirements baseline](../../requirements/01_baseline.md#r01) states the
obligations this flow answers (`designedFlowGatesEveryStage`,
`flowRunIsRealizedByDesign`, `runRealizesItsDesign`), the
[requirements trace](../../requirements/01_baseline.md#r02) records the project's
satisfy links — one satisfier per requirement, so this flow's coverage is recorded by
its planned verification case instead ([D69](04_module_design.md#d69)) —
and the [stage review checklist](../../review/02_checklist.md#v08) states what a CDR
decision would still need. Executed verification is recorded in
[P28](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p28); a declared case
in `mbse/model/verification.sysml` is a plan, not a result.

Reading order: run design → gates and judgment → view coverage → module design →
Rust UML design, then the [API pages](../../api/wish_verify_flow/00_overview.md#u73)
for the class-and-function contract.
