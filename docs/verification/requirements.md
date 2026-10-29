# Verification of requirements

<a id="r11"></a>

## 1. Verification and acceptance plan

The available verification methods are analysis, demonstration, inspection, and test.
Method, phase, acceptance criterion, and case must be assigned per requirement before a
requirements baseline can be frozen. The table names current coverage without treating a
declared case or a clean compiler result as an executed acceptance test.

| Scope | Method and phase | Evidence or case | Acceptance state |
| --- | --- | --- | --- |
| Checkpoint source, configured authority and local operator | Source/model inspection at design reconciliation, with existing runtime case families | `CheckpointOperatorDesignReview`; [R16](../requirements/requirements.md#r16), [D110](design.md#d110) | As-built allocation; generic scripted-only authority policy remains unresolved |
| Shared checkpoint origins and cross-project history | Future integration test and restore demonstration | `CrossProjectCheckpointReview`; PROJECT-01..06 in [D110](design.md#d110) | Proposed requirements; no implemented satisfier or executed cross-project result |
| Portable document contract and agent interfaces | Cross-project CLI/MCP demonstration, negative-path tests, and design review | `DocumentKitVerification`; DOC-01..07 in [D114](design.md#d114) | Planned case coverage; executed status belongs to the wish record |
| SysML syntax and supported references | Automated analysis at design delivery | pinned compiler `parse` and `check`, including model-only check ([T02](../architecture/constraints.md#t02)) | Executed for design-stage wishes; unevaluated rules remain |
| Documentation locators, embeds, and figure blocks | Automated inspection at documentation design delivery | `tools/check_documentation.py`; [D31](../interfaces/documentation-contract.md#d31) | Executed as a standalone audit, not a workflow gate |
| Nine primary figure presentation | Measured inspection and visual review at design delivery | `DiagramPresentationReview`; [V06](../records/wishes/wish-history.md#v06) | Historical presentation review recorded |
| SimpleFlow structure and behavior | Planned design review | `SimpleFlowViewReview` | Case declared in the model; no standalone passed case result |
| Six documentation-contract requirements | Planned design review | `DocumentationContractReview`; [V07](../records/wishes/wish-history.md#v07) | Case declared; wish assessment is separate from case execution |
| Three Rust UML design requirements | Planned design review | `RustUmlDesignReview`; [D52](../design/flows/simple-flow/views.md#d52) | Case declared; no standalone passed case result |
| SimpleFlow gate, recovery, and file-plan behavior | Executable verification for the first implementation slice | `SimpleFlowGateBehaviorVerification`, `SimpleFlowRecoveryVerification`, `SimpleFlowFilePlanVerification`; [D54](design.md#d54) | Gate and recovery behavior tested in [P08](../records/executions/simple-flow.md#p08) and [P12](../records/reviews/simple-flow-fixes.md#p12); named model cases are declarations, not separately executed case records |
| Runtime gate and record conformance | Scripted test and inspection of local SimpleFlow run | four checks in [D06–D07](../design/flows/simple-flow/definition.md#d06) | Commands, Rust adapter tests, and a four-stage demo executed for this slice ([P08](../records/executions/simple-flow.md#p08), [P12](../records/reviews/simple-flow-fixes.md#p12)); VModel integration and project acceptance remain open |
| Uniform stage CLI and restart integrity | Process-level tests and corruption/recovery tests | `LocalStageCliRuntimeVerification`; [CLI contract](../design/stage-cli.md#d85) | Planned cases and executed results are distinguished in the [stage CLI plan](../records/plans/stage-cli.md) |
| Durable operational storage | Automated test, fault injection, process demonstration, and inspection at storage implementation verification | `StorageVerification`; STORE-01 through STORE-10 in the [storage plan](../records/plans/storage.md#p38) | Case and objective criteria defined; the storage plan records executed outcomes and their limits |

The seven storage requirements have the following objective acceptance mapping. These are
planned criteria, not results; the case identifiers refer to the storage plan.

| Requirement | Acceptance criterion and planned case |
| --- | --- |
| `operationalRecordsHaveDurableAuthority` | STORE-08 persists both flat flows' registrations, runs, reports, evidence and receipts with no new Git wish trace and no authoritative runtime byte existing only under `cache/`. |
| `wishHistoryIsRevisioned` | STORE-02 creates successive immutable revisions and returns their exact metadata and complete manifests through every query; STORE-09 reads a selected earlier revision. |
| `materialsAreContentAddressed` | STORE-03 demonstrates deduplication, exact digest/length verification and unsafe-path refusal; STORE-07 detects subsequent byte loss or alteration. |
| `storageSaveIsAtomicAndIdempotent` | STORE-04 repeats an exact operation after reopen without a second effect; STORE-05 produces one winner and one conflict; STORE-06 reconciles every injected crash boundary. |
| `workingCacheIsRecoverable` | STORE-09 deletes generated runtime cache, restores the stored head, checks out all manifest bytes and passes the existing record checker against that materialization. |
| `wishStatusProjectsFlowExecution` | STORE-08 obtains identical status/current-stage projections from the stored selected flow and run record across reopen, including a blocked and a completed run; no independent status edit exists. |
| `storageInputAndIntegrityAreChecked` | STORE-01 refuses foreign formats; STORE-03 rejects malformed and unsafe input; STORE-07 reports missing, altered, orphan and pending state without returning trusted bytes. |

Acceptance of a wish is recorded by that wish's applicable stage assessment at a stated
artifact revision. It does not freeze the project's entire requirements baseline.

<a id="d15"></a>

### 1.1 Requirements: satisfaction and planned verification

![SimpleFlow requirements, satisfiers, and planned verification](assets/requirements.svg)

> **Figure D15-1 — Requirements trace.**  
> **Question:** Which selected requirements are satisfied by design elements and planned for review?  
> **Model view:** `XDesignDetailedDesign::requirementsDiagram`, a focused requirement and relationship projection.  
> **Companions:** [R01](../requirements/requirements.md#r01) for the full baseline and [R02](../requirements/requirements.md#r02) for satisfaction links.  
> **Status:** renderable-now; design reviewed.  
> **Reading:** Three requirements connect to their satisfiers and to `SimpleFlowViewReview`. Distinct edge labels separate a design answer from a planned verification case.  
> **Limits:** The selected subset and planned review do not imply passed verification or coverage of every requirement.

`requirementsDiagram` selects three detailed-design requirements, their satisfaction links,
and `XDesignVerification::SimpleFlowViewReview`. The requirements remain declared in
`XDesignRequirements`; their satisfiers remain project-level relationships in
`XDesignTraceability`. The independent wish trace records why wish-0006 produced this
design ([R02](../requirements/requirements.md#r02)).

| Requirement | Selected satisfier | Planned verification case |
| --- | --- | --- |
| `flowRunIsRealizedByDesign` | `SimpleFlowRun` | `SimpleFlowViewReview` |
| `designedFlowGatesEveryStage` | `designedSimpleFlow` | `SimpleFlowViewReview` |
| `structureGraphsAreDocumented` | `DocFigure` | `SimpleFlowViewReview` |

Short captions identify the three requirements without displaying their complete
rationales inside boxes. Satisfaction and verification labels distinguish the two kinds
of relationship; the planned review remains distinct from a recorded assessment.

The planned design review checks these three obligations through the nine coordinated
views, their source elements, and their documentation. The other detailed-design
requirements remain in the requirements baseline; this focused view does not add
verification coverage for them. A satisfaction relationship identifies the design that
answers a requirement. A verification relationship identifies the case intended to
establish it; neither relationship alone records a passed result.

Compiler parse/check reports establish the supported syntactic checks. Semantic review
must assess whether the design answers the wish, including blocked and rework behavior.
Application execution, checker conformance, and engineering validation need their own
evidence when implemented ([V05](../records/wishes/wish-history.md#v05)).

Released-checker requirements have their acceptance mapping in
[R18](../requirements/requirements.md#r18) and [ISO-01–09](design.md#d112).
`CheckerIsolationVerification` verifies release resources, retained executors and project
execution context; its declaration alone records no executed result.
