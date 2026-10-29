# xdesign — architecture overview

This document explains the architecture after the [documentation entry](../index.md).
Each section names the model elements it documents; the model carries the reverse reference
(`XDesignCommon::DocSection`, see [B04](system-composition.md#b04)). Every figure is
rendered from the model by `tools/render_docs.py`; none is drawn by hand.

Read [B30, the wish-to-delivery route](wish-lifecycle.md#b30) for the complete
default stage and gate sequence, then [A03](wish-lifecycle.md#a03) for the controller behavior that
repeats within each stage. [System composition](system-composition.md#b01)
shows the parts; [architecture design](system-context.md#b05) records the design status
and open decisions. The [portable document kit](document-kit.md#b33) is a
separate contract authoring component for other project roots.

<a id="a01"></a>

## What the project does

xdesign manages engineering development with SysML v2. A requester states a wish; the
system records the work it causes and the evidence that answers it. The tool must answer
two questions:

1. What makes up the system being developed, and how do its parts relate and work together?
2. Why did each wish lead to these requirements, designs, implementations and verification results?

The first question is answered by the project MBSE model under `mbse/model/`. For new
store-backed runs, `XDesignStorage` answers the second through operational items, immutable
revisions, materials, and receipts. Independent SysML traces under `mbse/wishes/` remain
the authority for legacy and bootstrap runs. `XDesignArchitecture::XDesign` owns the
composition and its `ProjectStore`; `XDesignComposition::xdesign` instantiates it.

<a id="a02"></a>

## Engineering models and operational traces

| Record class | Lives in | Holds | Changes when |
| --- | --- | --- | --- |
| Project MBSE model | `mbse/model/` | Requirements baseline, architecture, record and storage definitions, workflow definitions and traceability links | The project's own design changes |
| Legacy/bootstrap wish traces | `mbse/wishes/wish-<id>.sysml` | One wish, its trace, stage records, activities and assessments | That legacy wish advances |
| New operational traces | `.xdesign/` metadata and blobs; regenerable working copies under `cache/` | Wish registrations, runs, complete revision manifests, materials, evidence and operation receipts | A guarded save appends a revision |
| Workflow definitions | inside the project model | `Flow`, `Stage`, `Checkpoint`, `Checker`, `Controller` | The way work is organised changes |

The dependency runs one way. A wish references model elements; no model element names a
wish (`XDesignRequirements::modelStandsWithoutWishes`). A wish's products — the
requirements it produces and the model changes it causes — are retained in `mbse/model/`;
a wish record may describe them but never holds the only copy
(`XDesignRequirements::wishProductsAreRetained`). Operational status is projected from the
stored run rather than maintained as a second editable flag.

<a id="a04"></a>

## Repository layout

```text
mbse/
  model/    common.sysml        shared statuses, kinds, metadata (DocSection/DocFigure)
            requirements.sysml  the requirements baseline
            architecture.sysml  flows, checkpoints, records, controller, XDesign
            detailedDesign.sysml  the designed flows: runs, factories, checkpoints, gates
            composition.sysml  the xdesign instance and selectable flow portfolio
            verification.sysml  planned design-review coverage, separate from results
            workflow.sysml      ports, interfaces and the executed lifecycle scenario
            traceability.sysml  which model element satisfies which requirement
            uml.sysml           the xdesign Rust package and API design elements
            storage.sysml       operational store records, save, query and recovery
  wishes/   wish-0001.sysml ...   legacy and bootstrap SysML traces
.xdesign/   store.db and content-addressed blobs for new operational runs (ignored)
docs/
  index.md                  reader routes and model-to-document map
  AGENTS.md                 authoring/naming/ID/figure/link rules (site-excluded)
  requirements/             requirements, rationale, acceptance conditions
  architecture/             context, composition, behavior, lifecycle, constraints
  design/                   common flow design, flow-specific design, API design
    index.md flow.md lifecycle.md evidence-and-revisions.md adapters.md
    flows/ simple-flow/ wish-verify/
    api/                    one design document per API, grouped by layer
  interfaces/               ICD, CLI contract, record data contract, documentation contract
  verification/             verification design: methods, cases, criteria, acceptance
  records/                  plans, reviews, executions, evidence summaries, status, wish history
tools/      sysml_web/        the pinned compiler and local workbench
            render_docs.py    renders figures into each carrying page's assets/ child
            check_documentation.py  validates document/model location links
cache/      compiler reports, evidence, render logs, research prototypes (not versioned)
```

The [documentation entry](../index.md) gives reader routes, the document-class map, and
the model-to-document map. Document classes are not numbered directories; page names are
semantic, `index.md` is used only for a class or layer landing page, and figures live beside
the design page they explain. Class ownership and the one-home rule are defined in
[the documentation contract](../interfaces/documentation-contract.md#d30).

| Class | Holds | V-model role |
| --- | --- | --- |
| `requirements/` | The system requirements, rationale, and acceptance conditions | `systemRequirements` |
| `architecture/` | Composition, behavior, lifecycle, constraints, and design choices | `systemArchitecture` |
| `design/` | Shared module design and the API design set | `detailedDesign`, `implementation` |
| `interfaces/` | Cross-boundary contracts and data formats | `detailedDesign` |
| `verification/` | Verification design, cases, criteria, and acceptance boundaries | `systemVerification`, `acceptanceValidation` |
| `records/` | Plans, reviews, executions, evidence, status, and wish history | Project records across stages |

<a id="a05"></a>

## The model at a glance

![The project model's definitions and their relationships](assets/model-definitions.svg)

> **Figure A05-1 — Project model definitions.**  
> **Question:** Which core definitions and specializations make up the project model?  
> **Model view:** Definitions-and-specializations projection of `mbse/model/`.  
> **Companions:** [B01](system-composition.md#b01) for owned parts and [F01](../design/flow.md#f01) for flow vocabulary.  
> **Status:** renderable-now.  
> **Reading:** The graph connects the main flow, record, controller, and system definitions. Specialization edges show where a specific flow or record refines a common type.  
> **Limits:** This broad inventory omits the detailed behavior and interface contracts of each definition.

Rendered from `mbse/model` as a definitions-and-specializations diagram: `Flow` and its
specializations, the record definitions, `Controller`, and `XDesign`.

<a id="a06"></a>

## Documentation map

Where each model module and sub-module is documented. Section identifiers are stable;
`document + section` is the reference form the model itself carries.

| Model element | Documented in |
| --- | --- |
| `XDesignCommon` | [F02](../design/adapters.md#f02) |
| `XDesignCommon::WishStatus` | [A03](wish-lifecycle.md#a03) |
| `XDesignCommon::StageStatus` | [E02](../design/evidence-and-revisions.md#e02) |
| `XDesignCommon::FlowStatus` | [E01](../interfaces/record-format.md#e01) |
| `XDesignCommon::CheckpointOutcome` | [E04](../design/evidence-and-revisions.md#e04) |
| `XDesignCommon::CheckKind`, `JudgeAuthority`, `ConformanceKind` | [F02](../design/adapters.md#f02) |
| `XDesignCommon::DocSection` | [D27](../interfaces/documentation-contract.md#d27) |
| `XDesignCommon::DocFigure` | [D28](../interfaces/documentation-contract.md#d28) |
| SysML view/viewpoint to document figure contract | [D29](../interfaces/documentation-contract.md#d29) |
| `DocumentationViewpoint` and the nine SimpleFlow concern usages | [D29](../interfaces/documentation-contract.md#d29), [D12](../design/flows/simple-flow/views.md#d12) |
| Documentation set and per-flow growth | [D30](../interfaces/documentation-contract.md#d30) |
| Documentation audit and SimpleFlow checker rules | [D31](../interfaces/documentation-contract.md#d31), [D06](../design/flows/simple-flow/definition.md#d06) |
| `XDesignRequirements`, `XDesignRequirement`, every requirement | [R01](../requirements/requirements.md#r01) |
| Portable document-contract requirements | [R19](../requirements/requirements.md#r19) |
| `XDesignTraceability` | [R02](../requirements/requirements.md#r02) |
| `XDesignStorage`, `ProjectStore`, revision/material/save/query/recovery elements | [D86](../design/storage.md#d86), [I11](../interfaces/store-format.md#i11), [storage API](../design/api/storage/index.md) |
| `StorageVerification` | [D87](../verification/design.md#d87) |
| `XDesignRustUml` and the xdesign crate's API elements | [D44](../design/flows/simple-flow/implementation.md#d44), [U65](../design/api/index.md#u65), [U14](../design/api/index.md#u14) |
| `RustUmlDesignReview` | [D52](../design/flows/simple-flow/views.md#d52) |
| `simpleFlowFilePlan`, `simpleFlowFilePlanView` | [D53](../records/plans/simple-flow-file-plan.md#d53) |
| `SimpleFlowGateBehaviorVerification`, `SimpleFlowRecoveryVerification`, `SimpleFlowFilePlanVerification` | [D54](../verification/design.md#d54) |
| `XDesignArchitecture` | [A05](#a05), [B05](system-context.md#b05) |
| `PortableDocumentKit`, `DocumentContractSource`, and the authoring/CLI/MCP boundaries | [B33](document-kit.md#b33) |
| `Flow`, `Stage` | [F01](../design/flow.md#f01) |
| `Checkpoint`, `Checker` | [F02](../design/adapters.md#f02) |
| `CheckerRule` | [D06](../design/flows/simple-flow/definition.md#d06) |
| `VModel`, `SimpleFlow`, `WishVerifyFlow`, `DesignFlow`, `simpleFlow`, `wishVerifyFlow`, `designFlow` | [F03](../design/flow.md#f03) |
| `sysmlDeliverableAccepted`, `sysmlCompilerCheck` | [F03](../design/flow.md#f03) |
| `nestedFlowExample` | [F05](../design/lifecycle.md#f05) |
| `Wish`, `Trace` | [E01](../interfaces/record-format.md#e01) |
| `FlowExecution`, `ActivityRecord` | [E02](../design/evidence-and-revisions.md#e02) |
| `StageRecord` | [E03](../design/evidence-and-revisions.md#e03) |
| `CheckpointAssessment` | [E04](../design/evidence-and-revisions.md#e04) |
| `Controller` | [F04](../design/lifecycle.md#f04) |
| `XDesignDetailedDesign`, `designedSimpleFlow` | [D01](../design/flows/simple-flow/definition.md#d01), [D32](../design/flows/simple-flow/implementation.md#d32) |
| `DocumentContractEngine`, its six actions, and CLI/MCP adapter designs | [D113](../design/document-kit.md#d113) |
| `DocumentKitVerification` | [D114](../verification/design.md#d114) |
| `SimpleFlowRun` | [D02](../design/flows/simple-flow/implementation.md#d02) |
| `SimpleFlowRunFactory` | [D03](../design/flows/simple-flow/implementation.md#d03) |
| `EvaluateSimpleFlowGate`, `AdvanceSimpleFlowRun` | [D04](../design/flows/simple-flow/implementation.md#d04) |
| `SimpleFlowCheckpoint`, `SimpleFlowSyntacticCheckpoint`, `SimpleFlowSemanticCheckpoint` | [D11](../design/flows/simple-flow/definition.md#d11) |
| `EvaluateSimpleFlowCheckpoint`, `RunSyntacticCheck`, `TakeSemanticJudgment` | [D11](../design/flows/simple-flow/definition.md#d11) |
| The designed flow's gates, their class and documented rules | [D05](../design/flows/simple-flow/definition.md#d05), [D06](../design/flows/simple-flow/definition.md#d06), [D11](../design/flows/simple-flow/definition.md#d11) |
| The designed flow's implemented local checks and declared rule contract | [D06](../design/flows/simple-flow/definition.md#d06), [D07](../design/flows/simple-flow/definition.md#d07) |
| `SimpleFlowRunTimeline` | [D08](../design/flows/simple-flow/views.md#d08) |
| The nine coordinated SysML views | [D12](../design/flows/simple-flow/views.md#d12) |
| `DiagramDocumentationRenderer` | [D22](../interfaces/documentation-contract.md#d22) |
| `DiagramPresentation` | [D23](../interfaces/documentation-contract.md#d23) |
| `DiagramPresentationReview` | [D25](../interfaces/documentation-contract.md#d25) |
| `DocumentationContractReview` | [V07](../records/wishes/wish-history.md#v07) |
| `blockDefinitionDiagram`, `internalBlockDiagram` | [D13](../design/flows/simple-flow/views.md#d13), [D14](../design/flows/simple-flow/views.md#d14) |
| `requirementsDiagram`, `XDesignVerification`, `SimpleFlowViewReview` | [D15](../verification/requirements.md#d15) |
| `parametricDiagram`, `GateAuthorization`, `SimpleFlowAuthorization` | [D16](../design/flows/simple-flow/views.md#d16) |
| `useCaseDiagram`, `ProgressWish`, `ReviewEvidence`, `progressWish`, `reviewEvidence` | [D17](../requirements/requirements.md#d17) |
| `activityDiagram`, `sequenceDiagram` | [D18](../design/flows/simple-flow/views.md#d18), [D19](../design/flows/simple-flow/views.md#d19) |
| `stateMachineDiagram`, `SimpleFlowRunLifecycle` | [D20](../design/flows/simple-flow/views.md#d20) |
| `packageDiagram` | [D21](system-composition.md#d21) |
| `projectPackageIntegration` | [D21](system-composition.md#d21) |
| `CheckCommandExchange`, `JudgmentExchange`, their interfaces, `CheckerCommand`, `Judge`, `SimpleFlowAutomation` | [D10](../design/flows/simple-flow/views.md#d10) |
| `XDesign`, `xdesign` | [B01](system-composition.md#b01) |
| `recordStructure` (view) | [T03](constraints.md#t03) |
| `XDesignWorkflow` | [B02](system-composition.md#b02), [I01](../interfaces/contracts.md#i01) |
| `WishExchange`, `FlowExchange`, `WishInterface`, `FlowInterface`, `WishRequester`, `WorkflowFlow`, `WorkflowController`, `Workflow` | [B02](system-composition.md#b02) |
| `StageWork` | [F04](../design/lifecycle.md#f04) |
| `workflow` (usage) | [E06](../records/executions/wish-journeys.md#e06) |
| `wishLifecycle`, `workflowInterfaces` (views) | [T03](constraints.md#t03) |
| `mbse/wishes/wish-0001.sysml` … `wish-0003.sysml` | [V02](../records/wishes/wish-history.md#v02) |
| `mbse/wishes/wish-0004.sysml` | [V04](../records/wishes/wish-history.md#v04) |
| `mbse/wishes/wish-0005.sysml` | [V03](../records/wishes/wish-history.md#v03) |
| `mbse/wishes/wish-0006.sysml` | [V05](../records/wishes/wish-history.md#v05) |
| `mbse/wishes/wish-0007.sysml` | [V06](../records/wishes/wish-history.md#v06) |
| `mbse/wishes/wish-0008.sysml` | [V07](../records/wishes/wish-history.md#v07) |
| `mbse/wishes/wish-0009.sysml` | [V16](../records/wishes/wish-history.md#v16) |
| `mbse/wishes/wish-0010.sysml` | [V17](../records/wishes/wish-history.md#v17) |
| `mbse/wishes/wish-0011.sysml` | [V18](../records/wishes/wish-history.md#v18) |
| `designedFlowTree` (view) | [D01](../design/flows/simple-flow/definition.md#d01) |
| `designTree` (view) | [D02](../design/flows/simple-flow/implementation.md#d02) |
| `factoryFlow` (view) | [D03](../design/flows/simple-flow/implementation.md#d03) |
| `gateEvaluationFlow` (view) | [D04](../design/flows/simple-flow/implementation.md#d04) |
| `checkpointEvaluationFlow` (view) | [D11](../design/flows/simple-flow/definition.md#d11) |
| `interfaceStructure` (view) | [D10](../design/flows/simple-flow/views.md#d10) |
| `tools/sysml_web/`, `tools/render_docs.py` | [T01](constraints.md#t01) |
