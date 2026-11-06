# Verification of requirements

<a id="r11"></a>

## 1. Verification and acceptance plan

The available verification methods are analysis, demonstration, inspection, and test.
Method, phase, acceptance criterion, and case must be assigned per requirement before a
requirements baseline can be frozen. The table names current coverage without treating a
declared case or a clean compiler result as an executed acceptance test.

| Scope | Method and phase | Evidence or case | Acceptance state |
| --- | --- | --- | --- |
| Checkpoint source, configured authority and local operator | Source/model inspection at design reconciliation, with existing runtime case families | `CheckpointOperatorDesignReview`; [R16](../requirements/requirements.md#r16), [D110](design.md#d110) | Current script-only syntactic scope; prior Agent CLI assessments remain historical evidence |
| Shared checkpoint origins and cross-project history | Future integration test and restore demonstration | `CrossProjectCheckpointReview`; PROJECT-01..06 in [D110](design.md#d110) | Proposed requirements; no implemented satisfier or executed cross-project result |
| Portable document and scenario-template contracts and agent interfaces | Cross-project CLI/MCP demonstration, catalog and compatibility tests, negative-path tests, and permitted semantic review | `DocumentKitVerification`; DOC-01..15 in [D114](design.md#d114) | DOC-01..07 retain their original coverage; DOC-08..15 are planned; executed status belongs to the wish record |
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
| Linked-worktree location and verified local-store migration | Isolated process, integrity, concurrency and fault tests plus design and documentation review at implementation verification | `WorktreeStoreMigrationVerification`; WTSTORE-01 through WTSTORE-13 in [D117](design.md#d117) | Planned design allocation and criteria; no migration case is executed by this declaration |

The scenario-template requirements have the following planned acceptance mapping at
implementation verification. No case is executed by its declaration or by a requirements-stage
compiler/documentation check.

| Requirement | Method and planned acceptance |
| --- | --- |
| `documentTemplatesCoverScenarios` (proposed) | Inspect bundled inventory and exactly-one-default rules (DOC-08, DOC-09), the single mixed example's five-class/two-template/coverage-list content (DOC-10), all 16 isolated examples and that one whole-project mixed example (DOC-11), and malformed resources (DOC-14). Assert exact sentinel inventory/location/exclusivity, actual non-contract identity, missing/duplicate/misplaced rejection, one-field full-byte comparison, mixed reuse, drift and real-digest propagation (S-01 through S-05, S-07 through S-12). |
| `documentTemplateSourcesAreInspectable` (proposed) | Test source lists, all eight bundled roles, license/copying records (DOC-08), safe locators and license sentinels (DOC-14), and permitted source/license review (DOC-15). |
| `documentTemplateSelectionIsExplicit` (proposed) | Exercise explicit/omission selection and output-path/default failures (DOC-09, DOC-14), mixed per-document identity reporting (DOC-10), and effective-contract legacy reporting/resolution (DOC-13). Reject zero expected/selected digests and sentinel marker/registry identities without writes or adoption (S-06, S-07). |
| `documentTemplateChecksUseSelectedContract` (proposed) | Exercise regression and selected-template structural defects with existing diagnostic fields and byte-preserving failures (DOC-03, DOC-10), figure/table alternatives (DOC-11), and semantic figure/rationale review (DOC-15). |
| `documentTemplateCatalogSurfaceIsShared` (proposed) | Compare every enumerated CLI/MCP field/resource, exact template bytes, effective identities, digest versions, and the synthesized schema 1.0 compatibility projection (DOC-12, DOC-13). Assert actual nonzero selected/pinned digests, real-digest propagation, and that examples never become templates or returned contracts (S-06, S-12, S-14). |
| `documentTemplateCatalogPreservesCompatibility` (proposed) | Run unchanged DOC-01..07, selected-contract and registry-byte preservation (DOC-10, DOC-13), original v1/1.1 pack selection, bundled-v1 identity resolution, unmanaged refusal, and digest/link/HTML security cases (DOC-12 through DOC-14). Include exact sentinel-byte digest coverage, zero/stale digest refusal, valid example drift, and unchanged v1 no-sentinel behavior (S-02, S-05 through S-07, S-11 through S-13). |
| `documentTemplateEvidenceIsBound` (proposed) | Inspect DOC-15 for assessed revision, effective digests/versions, commands/transcripts, full-input bundled fixture paths/SHA-256, external acceptance-manifest SHA-256/actual digests/fixture references, before/after hashes, diagnostics, unevaluated checks, separate DeepSeek independent and Kimi runner judgments and authorities, findings, and residual risks. Require S-01 through S-16 allocation, parent-evidence preservation, correction hashes/logs, all 26 unevaluated rules, and explicit S-15 deferred design/API/model/product/test obligations without completion claims. |

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

The linked-worktree requirements in [R21](../requirements/requirements.md#r21) have
the following planned acceptance allocation. The [planned design](../design/storage.md#d118)
declares responsibility, not implementation. These cases require implementation and retained
evidence; no requirement is established by a compiler or documentation audit.

| Requirement | Planned cases and acceptance focus |
| --- | --- |
| `linkedWorktreeStoreIsIsolated` | WTSTORE-01 and WTSTORE-02: primary, non-Git and linked contexts, aliases, same-named projects, distinct worktrees, failure to resolve identity/home, and stable selected roots. |
| `storeRootSelectionIsConsistent` | WTSTORE-04 and WTSTORE-07: exact root precedence, relative paths, direct API, explicit inspection and Web reads plus real delegated lifecycle operations against the same selected root. |
| `globalStoreNamespaceProtectsOtherContent` | WTSTORE-02, WTSTORE-03 and WTSTORE-09: validated bindings, deliberate collision, unrelated home content, existing home-root store, unsafe or foreign destinations. |
| `worktreeStoreMigrationPreservesHistory` | WTSTORE-05, WTSTORE-09, WTSTORE-11 and WTSTORE-12: verified full history, bytes, receipts and identity mapping, invalid/pending refusal, and the result at interrupted cutovers. |
| `migratedRunsKeepTheirLifecycle` | WTSTORE-05 and WTSTORE-06: both flows, active/blocked/completed states, exact retries, state tokens, artifact drift, cache-loss recovery and unchanged completed heads. |
| `worktreeStoreConflictsFailClosed` | WTSTORE-07, WTSTORE-09 and WTSTORE-12: local/global ambiguity, provenance, changed/reappearing sources, explicit inspection and no automatic merging. |
| `worktreeMigrationHasRecoverableCutover` | WTSTORE-09 through WTSTORE-12: quiescence, contention, injected phase failures, durable decision, inspection, idempotent recovery and the non-cooperating-client boundary. |
| `legacyCacheRegistrationsStayLegacy` | WTSTORE-08: real active/completed legacy registrations and traces, persistent cache classification, collision diagnostics and unchanged legacy bytes. |

WTSTORE-13 applies the affected regression suites and retains their limits. Independent
review must assess the requirements and evidence separately from the syntactic
SimpleFlow gate.

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
