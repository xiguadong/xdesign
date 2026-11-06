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
| Structured API and flow document authoring | Isolated-project operation and safety tests, compatibility and adapter regressions, and permitted semantic review | `StructuredDocumentKitVerification`; SDK-01..13 in [V19](#v19) | Proposed cases; no product execution or semantic acceptance is claimed |
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

<a id="v19"></a>

### 1.2 Structured-document acceptance

These cases are planned for implementation verification of [R20](../requirements/requirements.md#r20),
not executed by the requirements-stage compiler or documentation checks. Use isolated
consumer projects under `cache/`, a selected new structured contract, caller-authored
fictional content, and a configured `mbse/model/` stub. Retain the precise inputs and
actual outputs; these paths are acceptance examples, not files to overwrite in this project.

| Case | Method, inputs, and expected result |
| --- | --- |
| SDK-01 | **Demonstration and test:** Before creating the model stub, plan each role and assert no project write. Then create the configured model source and plan/create/check API pages exactly at `docs/design/api/core/create-run.md` and `docs/design/api/storage/rs-store-save.md`, and all four pages at `docs/design/flows/simple-flow/index.md`, `definition.md`, `implementation.md`, and `views.md` (create the linked landing page after its targets). Assert exact destination, selected and effective contract/template identity, required inputs and fields, caller-supplied content, unique anchors, returned versus actual file SHA-256, registry ownership, and a whole-project structural check. Inspect that each role's content meets its stated purpose. |
| SDK-02 | **Per-role operations:** In that project create same-stem API pages in two permitted layers and verify independent paths and ownership. For each of the six SDK-01 documents, update an existing section with its current SHA-256; require the intended content and hash to change, other section and registry bytes to remain unchanged, and recheck to pass. For each API and each flow role, inspect its selected contract: where a figure decision is allowed, supply a valid asset, attach it, recheck its section/block/asset and confirm identity and registry preservation. For every role, submit a prohibited or unknown concern and a table/prose/not-applicable decision where allowed; require deterministic prepublication refusals with unchanged page and registry. Use a valid derived fixture to exercise a non-figure decision where the role supports such a decision but the bundled fixture does not select one; otherwise record that it is inapplicable and verify the role's documented refusal. If a role forbids all figures, record that policy and verify refusal instead of fabricating a permitted case. Submit a stale-hash attachment independently for every role, using an otherwise permitted decision where available and a prohibited request otherwise; require `stale-document` for the otherwise valid request, documented deterministic precedence for the prohibited request, and unchanged bytes. |
| SDK-03 | **Negative-path test:** Exercise empty, uppercase, underscore, slash, backslash, `.`, `..`, `../outside`, absolute and unknown route/layer/role/name values, a caller-supplied arbitrary output path, and symlink components in the project route, destination, model source and contract resource, including links to both inside and outside the project. Under the xdesign profile, planning/creating API basename `index` must refuse it without affecting the layer landing page; in a fresh valid flow target, planning/creating the genuine flow `index.md` role remains valid once any required link targets exist. Attempt collisions with existing managed files and occupied registry paths; SDK-10 covers unmanaged pages. Plan/create/update/check reject applicable unsafe or inconsistent inputs before unsafe access or writes. Compare pre/post hashes of existing documents, registry, contract and an outside sentinel, and assert no outside or unexpected target file was created. |
| SDK-04 | **Validation test:** Omit required API and flow content, use unresolved placeholders, duplicate an allocated `U` anchor, break a model locator or local link, and introduce a malformed figure block. Require actionable structural diagnostics and no successful incomplete creation or invalid update. Exercise a justified not-applicable API field and a valid diagram alternative without requiring a figure for every API. The checker must still indicate semantic review is required after structural success. |
| SDK-05 | **Compatibility and regression test:** Retain and explicitly select the original schema 1.0 pack with digest `1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6` and the original 1.1.0 pack with digest `e2b827c6e929f4a8d03344a91ca72f2eb837962e916d5e7f0487e315e1440cef`. Compare their original resource bytes, routes, marker/registry forms, effective identities, and checks/updates before and after adding the new pack. Under the new pack, exercise all five flat class routes, omission defaults and all 16 original template IDs; compare their paths, required sections, fields and view decisions with 1.1.0, and create/check/update representative pages with the new identity using existing calls. Verify the original 1.1 pack's automatic exact bundled-v1 resolution, explicit original custom-v1 selection, refusal of mismatched or unknown identities, and byte preservation of unmanaged and historical artifacts. SDK-11 tests the new release's required exact bundled-version coexistence; other old or future identities require an explicit verified binding or are refused without migration. Run the existing document-kit regression suite and record every skip; the actual MCP cases in SDK-06 must execute even if optional regression tests are skipped. |
| SDK-06 | **Transport test:** Run matching get-contract, plan, create, whole-project and selected-document check, update and applicable figure-attachment requests against equivalent independent projects via the Python public API, CLI and actual stdio MCP server. Compare resolved paths, required inputs, rendered bytes, effective identities, hashes and structured success/failure diagnostics within each transport envelope. Compare contract/template resources and test invalid routes, stale hashes, contract drift and MCP root/profile redirection; no adapter may bypass the common decisions. SDK-13 tests denied I/O and publication failures across the adapters. Retain CLI JSON and MCP transcripts. |
| SDK-07 | **Content and model boundary inspection:** Confirm the API template covers the R20 contract information for both an operation and a type, with a model realization or reasoned language-side exception, implementation projection and explicit applicable/non-applicable fields. Confirm the four flow roles have different required author inputs, and that the generated pages contain only the supplied claims and links. Verify planning is possible before a model exists while creation/check use configured readable model sources; SDK-12 exercises per-page metadata. A language-side rationale does not invent a realization. No SysML file, diagram, source code or execution result is invented. |
| SDK-08 | **Consumer documentation demonstration:** From a copied distribution in a second project, follow its documented Python, CLI and MCP entry points and the plan/create/check/update example. Confirm the documentation explains route selectors, per-page inputs, model prerequisites, identities, mixed-release and selected-check scope, error/overwrite behavior, and incomplete or unknown publication outcomes. The detailed contract and remaining semantic-review boundary are discoverable without relying on an unpublished local path. |
| SDK-09 | **Evidence inspection and permitted semantic review:** Bind source revision, contract digests/versions, full inputs, generated paths and hashes, transcript/log paths, test exits and skips, failure-preservation hashes, model diagnostics and unevaluated checks, and reviewer identity/authority to the assessed revision. Review API and flow content, applicability rationale, model and source alignment, links, and any supplied figures for fidelity and readability. Record findings and residual risks separately from deterministic results; a structural pass alone cannot establish engineering acceptance. |
| SDK-10 | **Unmanaged-page test:** Put a readable ordinary unmanaged page at a valid structured destination in a project with valid model inputs. Plan must report existence and its actual hash without writing; a whole-project check must apply ordinary project checks and may pass without imposing the managed template. Create refuses the collision; update and attachment refuse the unmanaged target, and selected managed-document check does not adopt it. Compare the page and registry before/after every operation, including registry absence when none existed; no entry is added. Separately show that a managed marker with no registry entry yields an identity finding rather than being treated as ordinary unmanaged content. |
| SDK-11 | **Mixed-release test:** Create distinct flat managed pages with the original bundled 1.0.0 and 1.1.0 packs, then a structured page under the new pack in the same project. Under the structured pack, whole-project check must pass for otherwise valid inputs and report each page's actual effective identity; selected-document check must separately pass for each old and new page, report its scope and identity, and not claim whole-project acceptance. Introduce a duplicate anchor relevant to one selected page and require its global-context diagnostic. Under each original pack, preserve the original behavior: its own page remains updatable, while a whole-project check may report foreign-identity mismatch. Compare old page/marker bytes and old registry-entry identities before/after new-page creation and checks; adding a new entry may change the registry as a whole. Then update each old page through the structured pack with its current hash and assert the original effective identity, marker and registry bytes are preserved while only the requested section changes. Mismatched/forged identities and an unbound custom or later release must fail affected checks/writes without altering pages or registry. No check, update or compatibility resolution silently migrates an identity. |
| SDK-12 | **Per-page metadata and source test:** For both API examples and every flow role, plan without a model and assert the required `title`, `status`, `revision` and `model_sources` inputs. Create using complete nonempty metadata and a caller-supplied readable `.sysml` source under a configured root; verify that source is recorded per page. For each role independently reject absent, empty or malformed metadata, missing/empty/non-list source lists, malformed or duplicate entries, absolute/traversing/out-of-root or symlinked sources, wrong extensions, nonexistent sources and unreadable declared sources. Even with another valid model file present, these failures must not publish a page or modify registry/other files. Make a previously declared source unavailable and require applicable check/update/attachment to report the issue without a prepublication write. For a language-side API, retain a valid context source and a reasoned exception without asserting a model realization. |
| SDK-13 | **Denied I/O and publication fault test:** Across Python, CLI and actual stdio MCP, deny reads of a selected pack resource, model source, target document and registry, and deny writes at route creation, lock, staging, page replacement and registry replacement. Use OS permissions where reliable or deterministic test-only fault injection; record which operation and phase was denied. Require an unsuccessful structured error with stable applicable code, safe path and matching adapter semantics, not an uncaught exception. For prepublication failures compare pre-existing documents, registry and outside sentinel bytes; for create assert that no new page was published, and for update/attachment assert that the existing target remains unchanged. Deny registry publication after a new page is published: test both successful cleanup, with page absent and registry unchanged, and denied cleanup, with an explicitly reported incomplete pair and a later mismatch check once reads work. Inject an error after publication or an indeterminate outcome and require an honest phase/outcome report followed by read-only inspection; never infer no effect or delete a pre-existing file. For failed single-file replacement, require no partial-write fallback and unchanged target bytes. |

| Requirement | Planned cases |
| --- | --- |
| `structuredDocumentationHasNamedRoutes` | SDK-01, SDK-02, SDK-03, SDK-06, SDK-10 |
| `apiDesignDocumentsCoverContract` | SDK-01, SDK-04, SDK-07, SDK-09, SDK-12 |
| `flowDesignDocumentsCoverLayers` | SDK-01, SDK-04, SDK-07, SDK-09, SDK-12 |
| `structuredDocumentationUsesSafeOperations` | SDK-01 through SDK-04, SDK-06, SDK-10, SDK-12, SDK-13 |
| `structuredDocumentationKeepsVersionIdentity` | SDK-02, SDK-03, SDK-05, SDK-06, SDK-11 |
| `structuredDocumentationHasSharedSurfaces` | SDK-01, SDK-06, SDK-08, SDK-13 |
| `documentKitPublicApiIsDocumented` | SDK-07, SDK-08 |
| `structuredDocumentationEvidenceIsReviewable` | SDK-04, SDK-06, SDK-09, SDK-13 |

`XDesignVerification::StructuredDocumentKitVerification` links these requirements to the
planned case family. A successful model parse/check and this mapping only establish the
declared relationships and supported compiler predicates; the cases and semantic judgment
require separate execution and revision-bound evidence.
