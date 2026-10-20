# WishVerifyFlow view coverage

The nine selected SysML questions form a coverage index for the designed wish-verify
flow, as the documentation contract's [D29](../05_documentation_contract.md#d29)
requires of each designed flow. Every view is declared in
`mbse/model/detailedDesign.sysml` with a `DocumentationViewpoint` usage whose
`concernText` and stakeholder role explain why the view selects its elements; the
viewpoint's `subject` binds it to the native view, and the view's `DocSection` and
`DocFigure` locate the explanation and the rendered asset. For this flow all nine
questions are applicable; the figures themselves sit in the sections that answer their
questions, not here.

<a id="d64"></a>

## D64 Nine-question coverage index

| Concern | Viewpoint → native view in `XDesignDetailedDesign` | Applicability and reason | Embedded at | Question answered |
| --- | --- | --- | --- | --- |
| Block definition | `wishVerifyBlockDefinitionConcern` → `wishVerifyBlockDefinitionDiagram` | Required — the run, factory, and checkpoint types are the design's core | [D66](04_module_design.md#d66) | Which reusable types make up a wish-verify run? |
| Internal block | `wishVerifyInternalBlockConcern` → `wishVerifyInternalBlockDiagram` | Required — the automation wires exactly one exchange, and showing that no checker is wired is the point | [D67](04_module_design.md#d67) | How do the run and the judge connect? |
| Requirements | `wishVerifyRequirementsConcern` → `wishVerifyRequirementsDiagram` | Required — the requirements this flow answers and its planned verification case must be visible | [D69](04_module_design.md#d69) | Which requirements does this flow satisfy, and what verification is planned? |
| Parametric | `wishVerifyParametricConcern` → `wishVerifyParametricDiagram` | Required — visit completion authorization is a bound relation, not a narration | [D70](04_module_design.md#d70) | Which current visit values authorize advancement? |
| Use case | `wishVerifyUseCaseConcern` → `wishVerifyUseCaseDiagram` | Required — the requester and judge goals define the automation boundary | [D71](04_module_design.md#d71) | Which goals belong to each external actor? |
| Activity | `wishVerifyActivityConcern` → `wishVerifyActivityDiagram` | Required — one gate's evaluation has a single judged path, and saying so is a design fact | [D63](02_gates_and_judgment.md#d63) | How does one gate produce an assessment? |
| Sequence | `wishVerifySequenceConcern` → `wishVerifySequenceDiagram` | Required — the normal run's message order across controller, run, and stages | [D68](04_module_design.md#d68) | In which order are the two stages entered and assessed? |
| State machine | `wishVerifyStateMachineConcern` → `wishVerifyStateMachineDiagram` | Required — blocking, correction, completion, and reopen are guarded transitions | [D68](04_module_design.md#d68) | When does a wish-verify run activate, block, resume or complete? |
| Package | `wishVerifyPackageConcern` → `wishVerifyPackageDiagram` | Required — the flow's design spans architecture, detailed design, composition, Rust UML, and verification packages | [D65](04_module_design.md#d65) | Which model packages carry this flow's design? |

All nine rows are applicable for this flow — unlike a software-design document, none is
reasonably omitted, because the flow is small and each question has real modeled
content. The activity view lives with the gates document at
[D63](02_gates_and_judgment.md#d63), because the checkpoint's evaluate operation *is*
this flow's activity answer; the other eight sit in the module design beside the
behavior they explain.

Four supporting views are not part of the nine questions but are declared and carried
by the run design: `wishVerifyFlowTree` ([D57](01_run_design.md#d57)),
`wishVerifyDesignTree` ([D58](01_run_design.md#d58)), `wishVerifyFactoryFlow`
([D59](01_run_design.md#d59)), and `wishVerifyGateEvaluationFlow`
([D60](01_run_design.md#d60)). The Rust UML views `wishVerifyRustModules` and
`wishVerifyRustTypes` belong to the software design and are indexed in
[D80](05_rust_uml_design.md#d80). The file-plan view `wishVerifyFlowFilePlanView` is
carried by the implementation plan
([P27](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p27)).

| Model source | Elements | Documented at |
| --- | --- | --- |
| `mbse/model/architecture.sysml` | `WishVerifyFlow` (base flow, `outcomeAccepted`) | [F03](../01_flows_and_checkpoints.md#f03), [D57](01_run_design.md#d57), [D61](02_gates_and_judgment.md#d61) |
| `mbse/model/detailedDesign.sysml` | `DesignedWishVerifyFlow` | [D57](01_run_design.md#d57), [D61](02_gates_and_judgment.md#d61) |
| `mbse/model/detailedDesign.sysml` | `WishVerifyFlowRun`, `WishVerifyFlowRunFactory`, `EvaluateWishVerifyGate`, `AdvanceWishVerifyRun`, `ReopenWishVerifyStage` | [D58–D60](01_run_design.md#d58) |
| `mbse/model/detailedDesign.sysml` | `WishVerifyCheckpoint`, `WishVerifySemanticCheckpoint`, `EvaluateWishVerifyCheckpoint`, `TakeWishVerifyJudgment` | [D61–D63](02_gates_and_judgment.md#d61) |
| `mbse/model/detailedDesign.sysml` | `WishVerifyAutomation`, `WishVerifyRunTimeline`, `WishVerifyRunLifecycle`, `WishVerifyAuthorization`, use cases | [D65–D71](04_module_design.md#d65) |
| `mbse/model/detailedDesign.sysml` | `wishVerifyFlowFilePlan` | [P27](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p27) |
| `mbse/model/composition.sysml` | `xdesign.designedWishVerifyFlow` | [F03](../01_flows_and_checkpoints.md#f03), [D57](01_run_design.md#d57) |
| `mbse/model/traceability.sysml` | One satisfier per requirement kept; this flow's coverage recorded by its verification case instead (export limitation, [D69](04_module_design.md#d69)) | [D69](04_module_design.md#d69), [R02](../../requirements/01_baseline.md#r02) |
| `mbse/model/verification.sysml` | `WishVerifyFlowBehaviorVerification`, `WishVerifyFlowFilePlanVerification` | [D73](04_module_design.md#d73), [P28](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p28) |
| `mbse/model/uml.sysml` | `crate::wishVerify`, `RsWishVerifyFlowPlan`, `RsValidateWishVerifyPlan`, `RsCreateWishVerifyRun`, realization dependencies | [D75–D77](05_rust_uml_design.md#d75) |
