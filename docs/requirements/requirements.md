# Requirements specification

> Project: xdesign | Status: working requirements, functional baseline not frozen |
> Audience: wish owners, architects, implementers, integrators, and reviewers.

The requirements the tool must meet, why each exists, and what currently establishes it.
The modelled requirements live in `mbse/model/requirements.sysml`; their satisfaction
links live in `mbse/model/traceability.sysml`. This document states the requirements,
their rationale, and the current gaps in the baseline; the model remains the normative
source.

| Revision | Date | Change | Review state |
| --- | --- | --- | --- |
| Working update | 2026-10-19 | Organized the requirements-stage topics around the existing model inventory and trace | Not an SRR or frozen functional baseline |
| Working update | 2026-11-03 | Added scenario-based document template catalog requirements and acceptance allocation | Proposed; independent wish requirements review pending |
| Working update | 2026-11-05 | Added linked-worktree store requirements and preliminary planned design allocations | Proposed; review of the revised allocation pending |

<a id="r04"></a>

## 1. Introduction

### 1.1 Purpose and readers

This specification gives the wish owner and reviewer an inspectable statement of what
xdesign promises. Architects and module owners use it to allocate behavior and constraints;
integrators and verifiers use its trace and evidence references to plan checks. A
requirements review would assess the completeness and quality of this specification before
claiming a functional baseline. No such SRR outcome is recorded here.

### 1.2 Project scope and boundary

xdesign accepts a wish, assigns an engineering flow, records the work and assessments at
each stage, and authorizes advancement or rework under that flow's checkpoints. The project
model describes the system and retains engineering products. A trace records why a particular
outcome was produced and judged: existing wishes use independent Git SysML traces, while the
storage requirements below define a durable operational store for new wish and run records.
The repository currently contains the SysML model, documentation, renderer, four SimpleFlow
checker commands, and local flat-flow Rust runs. The complete workflow application and three
VModel gate commands are outside the implemented scope
([R03](../records/status.md#r03), [P08](../records/executions/simple-flow.md#p08)).

### 1.3 Terms and references

| Term | Meaning here |
| --- | --- |
| Wish | A requested outcome assigned a flow and a trace. |
| Trace | The operational account of a wish's execution, retained in a legacy SysML model or the new store according to the run's registration boundary. |
| Flow | An ordered set of stages and applicable exit checkpoints. |
| Assessment | A judgment of one checkpoint at a stated artifact revision, with evidence. |
| RTM | Requirements traceability matrix; the human view of the model's satisfaction and planned verification links. |
| Functional baseline | Requirements accepted at requirements review; no freeze is claimed for this working document. |

Primary project references are [the documentation entry](../index.md),
`mbse/model/requirements.sysml`, `mbse/model/traceability.sysml`, and
[the architecture overview](../architecture/index.md#a01). The model files are the primary
source; this page records the readable requirement statements and their rationale.

<a id="r05"></a>

## 2. Product position and goals

The tool's purpose is to keep system composition and per-wish rationale inspectable as a
wish advances. Its intended success condition is an authorized delivery backed by current
stage assessments and retained model products ([A03](../architecture/wish-lifecycle.md#a03),
[B03](../architecture/system-composition.md#b03)). The model specifies the relevant completion
invariants; the project has not set service-level targets such as throughput, availability,
or response time. Such targets must be stated and verified before they can become a
performance baseline.

<a id="r06"></a>

## 3. User roles and scenarios

| Role | Goal | Current model or document |
| --- | --- | --- |
| Wish owner | Submit and progress a wish while retaining its trace and outcome. | `ProgressWish`; [F04](../design/lifecycle.md#f04) |
| Reviewer or permitted judge | Inspect current evidence and decide whether a semantic checkpoint passes. | `ReviewEvidence`; [F02](../design/adapters.md#f02) |
| Model author and implementer | Produce model and application artifacts that satisfy the declared requirements. | [A06](../architecture/index.md#a06), [D01](../design/flows/simple-flow/definition.md#d01) |
| Integrator and verifier | Check the relevant requirements, interfaces, stage records, and evidence. | [R02](#r02), [V01](../records/wishes/wish-history.md#v01) |

The first two roles are actors in the declared SimpleFlow use-case view. The other roles
describe readers and work responsibilities; they are not additional declared use-case
actors. A wish can legitimately enter rework when evidence is insufficient; the user goal
does not imply that every checkpoint must pass.

<a id="d17"></a>

### 3.1 Use cases: user goals at the system boundary

![Wish owner and reviewer goals for the simple flow](assets/use-cases.svg)

> **Figure D17-1 — Use cases.**  
> **Question:** Who interacts with the simple-flow automation, and for which goals?  
> **Model view:** `XDesignDetailedDesign::useCaseDiagram`, a projection of `ProgressWish` and `ReviewEvidence`.  
> **Companions:** [D18](../design/flows/simple-flow/views.md#d18) for internal actions and [B02](../architecture/system-composition.md#b02) for the system interfaces.  
> **Status:** renderable-now; design reviewed.  
> **Reading:** Actors sit outside the subject boundary and their goals sit inside it. Their associations identify the value sought by the wish owner and reviewer.  
> **Limits:** A goal does not prescribe the internal action order or guarantee a passing judgment.

`useCaseDiagram` selects `ProgressWish` and `ReviewEvidence`. The wish owner's goal is to
progress a wish toward delivery with a trace that retains the produced artifacts and
decisions. The reviewer or permitted judge's goal is to inspect the evidence and decide
whether the declared criteria hold. The subject is the simple-flow automation; the actors
and their goals establish its boundary. The diagram places the two goals inside that
subject boundary and the actors outside, with short actor-to-goal connections.

The use cases describe value to participants. Reading rules, invoking a command, and
appending an assessment are internal actions shown in [D18](../design/flows/simple-flow/views.md#d18). A user goal can end with
a blocked run or a request for rework when the evidence is insufficient; successful use of
the tool does not require a passing engineering judgment.

These goals support requirements and acceptance-scenario review. Their implementation is
still a design commitment, with concrete gate behavior defined by [D05](../design/flows/simple-flow/definition.md#d05).

<a id="r07"></a>

## 4. Operating environment, assumptions, and dependencies

The current delivered artifacts run from this repository: the pinned local SysML compiler,
Python documentation and rendering tools, generated SVGs, and the static documentation
site ([T01](../architecture/constraints.md#t01)). The model defines typed wish and flow
interfaces; it does not choose an application deployment platform, network transport,
external service, or device environment. Those choices and the resulting environmental
requirements remain open for implementation design.

The design assumes that a wish has an assigned flow, one trace, and revisioned assessment
evidence; a completed stage requires a current passing assessment. The model's current
constraint bodies are not executed by the compiler, so these assumptions still need
runtime and verification evidence ([B03](../architecture/system-composition.md#b03),
[T04](../architecture/constraints.md#t04)).

<a id="r01"></a>

## 5. Functional and supporting requirements inventory

The inventory combines workflow behavior,
model/document consistency, presentation, and checker obligations; the last three are
supporting quality constraints rather than user actions. Every requirement specializes
`XDesignRequirement`, which carries a `rationale`. No requirement names the wish that
asked for it. The table is an inventory of existing model statements, not a claim that
each already has full per-requirement acceptance and verification fields. [R11](../verification/requirements.md#r11) identifies that gap.

| Requirement | What it demands | Why it exists | Satisfied by | Status |
| --- | --- | --- | --- | --- |
| `checkpointKindIsDeclared` | Every checkpoint declares whether its criteria are established syntactically or semantically | A flow can route a check to the right executor only if the checkpoint says which kind it is | `Checkpoint::kind` | met in the model |
| `syntacticCheckIsFixed` | A syntactic checkpoint names at least one checker whose command fixes the check | Repeating criteria by hand invites drift; one fixed command is reproducible and comparable across revisions | `Checkpoint::checkers` | met by `designFlow` and the four declared SimpleFlow syntactic gates; `VModel` commands remain absent |
| `syntacticAssessmentIsScripted` | An assessment of a syntactic checkpoint is judged by a script, and its output for the assessed revision is the evidence | A script cannot rationalize a result, and its output can be re-examined | `CheckpointAssessment::judgedBy` | met in the model |
| `semanticCheckNamesItsJudges` | A semantic checkpoint names no checker and states whether an agent or a human may judge it | Criteria that no command can establish need a stated competent judge | `Checkpoint::agentMayJudge`, `Checkpoint::humanMayJudge` | met in the model |
| `checkerStatesItsConformance` | A checker states which kinds of conformance it establishes: artifact presence, identifier conformance, definition coverage, trace conformance | Knowing what a check covers is knowing what it leaves uncovered | `Checker::establishes` | met in the model |
| `wishProductsAreRetained` | The requirements a wish produces, and the model changes it causes, are retained in the model; a wish record never holds the only copy | A wish record is a working record; design kept only there would be lost to the project | — | unmet, maintained by hand |
| `modelStandsWithoutWishes` | The project model resolves and checks on its own; no model element names a wish or carries its identity | A model that needed wish records to check would make those records load-bearing | — | unmet, maintained by hand |
| `flowRunIsRealizedByDesign` | A flow's run is realized by design elements: a class for the run, a factory that creates it, and the functions that evaluate its gates and advance it | The architecture says what a flow is made of; only a design says what it is made into | `SimpleFlowRun` | met for the designed flow ([D02](../design/flows/simple-flow/implementation.md#d02)) |
| `designedFlowGatesEveryStage` | Every stage of a designed flow declares the exit checkpoint that must pass before the flow leaves it | A stage left without a gate completes on the mere passage of work | `designedSimpleFlow` | met for the designed flow ([D01](../design/flows/simple-flow/definition.md#d01)) |
| `gateSyntaxIsRunnable` | Every syntactic gate's checker names a command that exists in the repository; its output for the assessed revision is the evidence | A gate whose command does not exist can never pass by its own rule, and the work would stall or fall back to unrecorded judgment | `designFlow` | pinned compiler and all four SimpleFlow commands exist and have executed with retained reports ([D06](../design/flows/simple-flow/definition.md#d06), [R03](../records/status.md#r03)); unmet for `VModel` |
| `runRecordIsScriptChecked` | The conformance of a run's record — one visit per stage, assessments cited at the visited revision, evidence that exists — is established by a checker, not by reading | These invariants were kept by reading; reading does not re-run per revision and does not fail a record | — (no project-wide satisfier) | `simpleRecordCheck` is declared and executed for the new SimpleFlow JSON run; default V-model and historical SysML wish records remain outside it ([D06](../design/flows/simple-flow/definition.md#d06), [R03](../records/status.md#r03)) |
| `designCarriesItsDocSections` | Every element of the detailed design carries the documentation section that covers it, and the design's checker establishes the sections are present and well formed | A design read from declarations alone forces the reader to reconstruct the reasoning; a checker keeps the map from rotting | `DocSection` | documentation audit and `simpleDesignCheck` execute locator checks; semantic adequacy still needs reading ([D31](../interfaces/documentation-contract.md#d31)) |
| `checkerRulesAreDeclared` | Every checker of a designed flow declares the rules by which its outcome is determined — each rule's exact condition and what it reads — and its command reports each rule's verdict; a check establishes the declared rules and the command's rule list agree | A checker that only names a command hides its logic in code, and the model cannot say why a gate passed or what would make it fail | `CheckerRule` | 18 rules declared and reported by the four SimpleFlow commands ([D06](../design/flows/simple-flow/definition.md#d06)); `VModel`'s checkers declare none |
| `structureGraphsAreDocumented` | Nine views of one model cover block definitions, internal blocks, requirements, parameters, use cases, activities, sequences, states, and packages; each locates a rendered figure and its documentation section | Three overview graphs leave goals, traceability, constraints, lifecycle conditions, and model dependencies unexplained | `DocFigure` | nine views and figures have design and presentation review; the audit and SimpleFlow design checker now check locators and embeds ([D12](../design/flows/simple-flow/views.md#d12), [D31](../interfaces/documentation-contract.md#d31)) |
| `runRealizesItsDesign` | A run of a designed flow is recorded with the design's class, so the design is realized by the run rather than declared beside it | A class nothing is typed by is a picture of a run, not a run's design | `SimpleFlowRun` | the SimpleFlow JSON run records `execution_type = FlowRun` with `plan_id = designedSimpleFlow`; the record checker tests it for new runs, while historical SysML records are not migrated |
| `diagramViewsAreReadable` | The nine primary views remain legible at a 900-pixel article width with text at least 11 pixels; boundaries, ports, labels, and relationship endpoints remain clear | A correct model inventory can still produce unusable diagrams | `DiagramDocumentationRenderer` | implemented; final figures and built-article evidence are reviewed separately ([D24](../interfaces/documentation-contract.md#d24), [V06](../records/wishes/wish-history.md#v06)) |
| `diagramPresentationRetainsSourceIdentity` | Short model-owned labels preserve native identities and full source details; final SVGs and previews have matching provenance | Presentation needs concise text without creating a disconnected second model or accepting stale previews | `DiagramDocumentationRenderer` | implemented by metadata, native SVG references, and hash-bound final-figure evidence ([D23](../interfaces/documentation-contract.md#d23), [D26](../interfaces/documentation-contract.md#d26)) |
| `documentationEntryMapsDesign` | One entry gives each reader a path through the current documents and maps model modules and current design concerns to detailed sections | A reader should not have to reconstruct the document set or its relationship to the model | documentation set (no SysML satisfier) | [A09–A11](../index.md#a09); stage paths were reviewed at wish-0008 revision 2, and revision 3 adds directory overviews ([V07](../records/wishes/wish-history.md#v07)) |
| `documentationSupportsMultipleFlows` | Each defined flow has an addressable overview; each developed flow design has its own detailed location, and a new flow can be added without renumbering another flow's sections | Multiple selectable flows should not be forced into SimpleFlow's design pages | documentation set (no SysML satisfier) | per-flow map and extension rule in [D30](../interfaces/documentation-contract.md#d30); revision 3 also gives the nested SimpleFlow directory its own overview ([V07](../records/wishes/wish-history.md#v07)) |
| `figuresStayWithDesignExplanation` | Each model-derived design figure appears beside its question, source view, companions, status, reading, and limits | An image catalog separated from its design claim is hard to interpret or audit | `DocFigure` | local figure blocks are covered by the [D28](../interfaces/documentation-contract.md#d28) audit; nine primary figures moved to stage-owned sections at revision 2, revision 3 renamed them with their documents, and revision 4 colocates them in each directory's `assets/` ([V07](../records/wishes/wish-history.md#v07)) |
| `modelDocumentationLinksResolve` | Every `DocSection` anchor and `DocFigure` asset and embed resolves; the module-level reverse index agrees with the annotated modules and sections it lists | A locator to a missing section or figure gives a false explanation path | `DocSection` | [D31](../interfaces/documentation-contract.md#d31) checks the current revision-4 paths; prior relocation reviews remain historical ([V07](../records/wishes/wish-history.md#v07)) |
| `viewAndViewpointRolesAreSpecified` | The viewpoint question and stakeholder, native view selection, rendered figure, and carrying section have distinct roles; nine reference concerns have flow-specific applicability | A figure needs an explicit concern and coverage decision to make omissions understandable | `DocumentationViewpoint` | nine SimpleFlow viewpoint usages and [D12](../design/flows/simple-flow/views.md#d12) applicability reasons were reviewed at revision 2 with [D29](../interfaces/documentation-contract.md#d29); their current stage locations are indexed at revision 3 |
| `checkerRulesAreImplementationReady` | Each documented design-stage checker rule states inputs, exact pass condition, failure diagnostic, evidence output, and implementation status | A later implementer should reproduce the intended check without inventing its verdict | `CheckerRule` | SimpleFlow contracts in [D06](../design/flows/simple-flow/definition.md#d06) now have command implementations and targeted rule tests; the broader V-model remains open |
| `rustUmlDesignMapsEveryDesignElement` | A Rust software design of a designed flow names one Rust element per design element — run, factory, checkpoint class and adaptations, gate and advance functions, records, statuses, ports — and records what each realizes | An unmapped design lets an implementation re-decide fixed structure and hides coverage gaps | `XDesignCrate` | mapped in `XDesignRustUml` with realization dependencies ([D46](../design/flows/simple-flow/implementation.md#d46)); the first executable module implements the SimpleFlow slice |
| `umlViewsAreSelectedWithReasons` | Every view of a module's software design is required, recommended, or not applicable with a reason; adopted views embed with figure block and honest status | A view set without stated applicability hides whether omissions were considered | documentation set (no SysML satisfier) | contract in [U09](../interfaces/documentation-contract.md#u09); first applicability table in [D52](../design/flows/simple-flow/views.md#d52) |
| `softwareDesignClaimsNoImplementation` | A software design distinguishes designed elements from implemented code and claims no implementation, test or build the repository lacks | A design read as a finished product hides remaining work | documentation set (no SysML satisfier) | stated in [D44](../design/flows/simple-flow/implementation.md#d44) and reviewed per [U12](../verification/review-criteria.md#u12) |
| `stageCliUsesUniformInteraction` | Every supported stage uses read-only `next` and a single `check` that records, assesses and advances, returning corrections or successor work | Agents follow flow content instead of coordinating internal commands | `StageCli` | [Local CLI contract](../design/stage-cli.md#d85); execution recorded by the new wish |
| `stageCliPreservesRunIntegrity` | Restore the same run across processes; reject corrupt state, plan drift, stale submissions and concurrent writers; bind judgments to artifact bytes | A short interface must preserve authority and evidence across restarts | `LocalStageCliRuntime` | [Runtime verification](../design/stage-cli.md#d85) and [storage evidence](../records/plans/storage.md#p38); local store, no authentication or remote-service claim |
| `operationalRecordsHaveDurableAuthority` | Persist new wish/run facts, revision manifests, material bytes and receipts outside regenerable cache; retain engineering products in Git | A run must not lose the facts and judged bytes when cache is discarded | `XDesignStorage` allocation in [R02](#r02) | Implemented; [STORE evidence](../records/plans/storage.md#p38) executed within disclosed limits |
| `wishHistoryIsRevisioned` | Append immutable numbered revisions with stable item identity/kind, typed metadata and complete manifests; support list/get/history/read/checkout | Earlier checked states must remain inspectable instead of being replaced by one mutable snapshot | `XDesignStorage` allocation in [R02](#r02) | Implemented; [STORE evidence](../records/plans/storage.md#p38) executed within disclosed limits |
| `materialsAreContentAddressed` | Bind logical URIs to exact SHA-256-identified bytes; atomically publish, deduplicate and verify reads through EOF | A path alone cannot establish which evidence bytes were judged | `XDesignStorage` allocation in [R02](#r02) | Implemented; [STORE evidence](../records/plans/storage.md#p38) executed within disclosed limits |
| `storageSaveIsAtomicAndIdempotent` | Reserve exact operations, guard expected heads, commit each complete revision and receipt together, and reconcile outcome without duplicate effects | Crash/retry and competing writers must not silently duplicate or overwrite run facts | `XDesignStorage` allocation in [R02](#r02) | Implemented; [STORE evidence](../records/plans/storage.md#p38) executed within disclosed limits |
| `workingCacheIsRecoverable` | Regenerate runtime cache and checkout every manifest byte; replay the SimpleFlow record checker on restored evidence | Cache is disposable only if durable copies reconstruct a usable record | `XDesignStorage` allocation in [R02](#r02) | Implemented; [STORE evidence](../records/plans/storage.md#p38) executed within disclosed limits |
| `wishStatusProjectsFlowExecution` | Derive wish status/current stage from the selected flow's run record and support kind/status queries | An independent status flag could contradict the flow's actual position | `XDesignStorage` allocation in [R02](#r02) | Implemented; [STORE evidence](../records/plans/storage.md#p38) executed within disclosed limits |
| `storageInputAndIntegrityAreChecked` | Refuse incompatible, malformed, excessive, unsafe or replaced state; verify digests and report missing/altered/orphan/pending data | Durable but corrupt or unsafe bytes must not enter engineering decisions | `XDesignStorage` allocation in [R02](#r02) | Implemented; [STORE evidence](../records/plans/storage.md#p38) executed within disclosed limits |
| `linkedWorktreeStoreIsIsolated` | Use an identity-bound home-directory store for each linked worktree project, retaining the ordinary project default | Independent workspaces need separate durable histories without project-local worktree stores | `PlannedWorktreeStoreSelection` | Planned; [R20](#r20) |
| `storeRootSelectionIsConsistent` | Preserve root override precedence and make Web and delegated lifecycle operations use the same selection | Explicit locations and operator actions must refer to the same authority | `PlannedWorktreeStoreAccess` | Planned; [R20](#r20) |
| `globalStoreNamespaceProtectsOtherContent` | Use a private bounded namespace and refuse occupied, unsafe or mismatched roots | Existing home-directory content and other stores must not be adopted or altered | `PlannedWorktreeStoreSelection` | Planned; [R20](#r20) |
| `worktreeStoreMigrationPreservesHistory` | Verify and migrate the complete logical local-store history and map physical identity safely | A raw copy is not a valid location migration | `PlannedWorktreeStoreMigration` | Planned; [R20](#r20) |
| `migratedRunsKeepTheirLifecycle` | Preserve active, blocked and completed run behavior, evidence and exact retries | Historical identity and safe continuation must survive cutover | `PlannedWorktreeStoreIntegration` | Planned; [R20](#r20) |
| `worktreeStoreConflictsFailClosed` | Detect ambiguous local/global stores and prevent automatic merge or silent history loss | Similar IDs and paths cannot prove histories are the same | `PlannedWorktreeStoreAccess` | Planned; [R20](#r20) |
| `worktreeMigrationHasRecoverableCutover` | Require quiescence, coordinate supported clients and recover interrupted cutovers | Concurrent writers and process crashes can otherwise create divergent authorities | `PlannedWorktreeStoreMigration` | Planned; [R20](#r20) |
| `legacyCacheRegistrationsStayLegacy` | Preserve genuine legacy cache registrations and refuse ID ambiguity with stored wishes | Legacy authority differs from a disposable store-backed cache | `PlannedWorktreeStoreAccess` | Planned; [R20](#r20) |
| `checkpointSourceIsProjectOwned` | Keep reusable source in the selected project and resolved run fields in its store | Tool installation and source edits must not redefine a registered run | `CheckpointLibrarySource` | As-built allocation; executor bytes are not pinned at init; [R16](#r16) |
| `configuredChecksRetainAuthority` | Retain actual executor authority, identity, revision and evidence | An agent answer cannot be represented as deterministic script evidence | `RsCustomChecker` | As-built allocation; generic scripted-only policy conflict remains; [R16](#r16) |
| `operatorDelegatesRunState` | Project stored facts and delegate lifecycle to the stage runtime | UI state must not become a second run authority | `OperatorWebProgram` | As-built allocation; [R16](#r16) |
| `sharedChecksHavePinnedOrigins` | Pin shared definition origin, executor bundle and input mapping | Equal local names do not identify equal content across projects | No implemented satisfier | Proposed extension; [R16](#r16) |
| `crossProjectHistoryKeepsItsOwner` | Identify source project/store/revision and retain restorable evidence | A derived central index is not the source authority | No implemented satisfier | Proposed extension; [R16](#r16) |

<a id="r08"></a>

## 6. Non-functional requirements

### 6.1 Performance

The explicit presentation target is `diagramViewsAreReadable`: the nine primary
SimpleFlow figures must remain legible at a 900-pixel article width, with displayed text
at least 11 pixels and identifiable boundaries, ports, labels, and relationship endpoints
([V06](../records/wishes/wish-history.md#v06)). No runtime latency, throughput, capacity, or
memory target is in the requirements model. A benchmark cannot pass a target that has not
been set.

### 6.2 Reliability and availability

Current requirements address correctness of stage advancement, evidence freshness, and
record completeness, especially `runRecordIsScriptChecked`. No service availability,
recovery-time, redundancy, or failure-budget requirement has been set. The SimpleFlow
record checker covers only its new JSON run format ([R03](../records/status.md#r03)).

### 6.3 Safety, security, and compliance

The model distinguishes agent, human, and script judgment authority and requires evidence
for each assessment. It has no explicit authentication, authorization, privacy,
data-protection, safety, or regulatory target. These must be specified before a deployed
service or sensitive data is assumed.

### 6.4 Maintainability and extensibility

`modelStandsWithoutWishes`, `documentationSupportsMultipleFlows`, and
`modelDocumentationLinksResolve` constrain separation of the project model, extension to
new flows, and location integrity. The independent model check and documentation audit
provide partial, repeatable evidence; they do not establish future deployment or migration
behavior ([R10](#r10)).

### 6.5 Compliance and accessibility

No regulatory, industry, or deployed-service accessibility obligation has been selected
for this tool. The documentation has a specific readability requirement for nine primary
figures, but that result does not establish product-wide accessibility compliance. An
applicable standard, objective criterion, method, and review phase must be added before a
compliance claim can be assessed.

<a id="r09"></a>

## 7. Interface requirements

The model specifies conceptual exchanges at the system boundary and within SimpleFlow
([B02](../architecture/system-composition.md#b02),
[D10](../design/flows/simple-flow/views.md#d10)). The draft
[interface control document](../interfaces/contracts.md#i01) already assigns IF-001 through
IF-004 and records their modeled fields and directions. Protocols, complete payload
schemas, timeouts, retries, criticality classes, contract owners, and review authorities
remain open; the ICD baseline has not been frozen for implementation or integration review.

| Model interface | Parties | Exchanged content | Contract state |
| --- | --- | --- | --- |
| `WishInterface` (IF-001) | requester and controller | submission description in; registered `Wish` and `Trace` references out | Typed ports and item flows modeled; transport and error contract unspecified ([I02](../interfaces/contracts.md#i02)) |
| `FlowInterface` (IF-002) | selected flow and controller | `Flow` definition supplied to controller | Typed ports and item flow modeled; selection timing and version contract unspecified ([I03](../interfaces/contracts.md#i03)) |
| `CheckCommandInterface` (IF-003) | SimpleFlow run and checker command | wish model and stage out; per-rule report and verdict in | Local command endpoint and JSON report schema implemented for SimpleFlow ([I04](../interfaces/contracts.md#i04)); transport and compatibility policy open |
| `JudgmentInterface` (IF-004) | SimpleFlow run and permitted judge | criteria and artifact URIs out; outcome and rationale in | Local judge response contract implemented for SimpleFlow ([I05](../interfaces/contracts.md#i05)); external identity enforcement and transport policy open |

IF-001 and IF-002 are architecture-level exchanges; IF-003 and IF-004 refine the
designed SimpleFlow module. The draft ICD records their modeled provider/consumer and
field information. Compatibility and failure semantics remain open in
[I09](../interfaces/contracts.md#i09).

<a id="r10"></a>

## 8. Data requirements

| Data or record | Required relation or lifecycle | Source |
| --- | --- | --- |
| `Wish` and `Trace` | Each wish has exactly one trace; a trace belongs to its wish. | [B01](../architecture/system-composition.md#b01) |
| `FlowExecution` and `StageRecord` | Record each assigned flow stage, including visits and the current artifact revision. | [E02–E03](../design/evidence-and-revisions.md#e02) |
| `CheckpointAssessment` | Identify checkpoint, evaluated revision, judge, outcome, and evidence URIs. | [E04](../design/evidence-and-revisions.md#e04) |
| Project engineering files | Keep project requirements, designs, implementations and verification definitions in Git, including `mbse/model/`; a store revision never holds their only authoritative copy. | [A02](../architecture/index.md#a02), `wishProductsAreRetained` |
| New wish and run records | Persist registration, flow identity, visits, assessments, revision manifests, judged material bytes and receipts in the operational store; derive status from the run. | `operationalRecordsHaveDurableAuthority`, `wishHistoryIsRevisioned`, `wishStatusProjectsFlowExecution` |
| Legacy wish models | Preserve existing `mbse/wishes/` traces as Git history without automatic import or retroactively changing their authority. | [A02](../architecture/index.md#a02), `operationalRecordsHaveDurableAuthority` |
| Working cache | Treat materialized sources, runtime copies and generated reports as regenerable after their durable revision is committed. | `workingCacheIsRecoverable` |

The existing model defines record identity and trace relationships; the new storage design
must define format/schema versions, request and manifest contracts, expected-head guards,
receipts and integrity inspection. Retention deletion, privacy classification, whole-store
backup and migration from other store formats remain outside this slice. Generated compiler
reports and local verification logs stay under `cache/`; once cited by a stored revision,
their required bytes must also have a durable material copy.

<a id="r12"></a>

## 9. Requirement quality review

A requirement quality review checks one objectively decidable behavior per requirement, an
explicit condition, subject, action, object, measurable target where relevant, and a
verification method and phase. It also checks consistent terminology, no duplicate or
conflicting targets, and coverage from each upper-level requirement into design and
verification. These are review criteria for this working document; the model's requirement
statements do not yet provide every field. In particular, per-requirement acceptance
criteria and method/phase are incomplete, and several operational targets are unset.

Before SRR, review each [R01](#r01) row against those criteria, record its parent or
derived relation where applicable, and complete its planned case and pass condition in
[R02](#r02). The current satisfaction links and case declarations are useful trace, but
they do not by themselves certify requirements quality or test coverage.

<a id="r02"></a>

## 10. Requirements traceability matrix

`mbse/model/traceability.sysml` carries the `satisfy` links from requirements to the
model features allocated to answer them:

| Requirement | Satisfier |
| --- | --- |
| `checkpointKindIsDeclared` | `Checkpoint::kind` |
| `syntacticCheckIsFixed` | `Checkpoint::checkers` |
| `syntacticAssessmentIsScripted` | `CheckpointAssessment::judgedBy` |
| `semanticCheckNamesItsJudges` | `Checkpoint::agentMayJudge` |
| `semanticCheckNamesItsJudges` | `Checkpoint::humanMayJudge` |
| `checkerStatesItsConformance` | `Checker::establishes` |
| `flowRunIsRealizedByDesign` | `SimpleFlowRun` |
| `designedFlowGatesEveryStage` | `designedSimpleFlow` |
| `gateSyntaxIsRunnable` | `designFlow` |
| `designCarriesItsDocSections` | `DocSection` |
| `checkerRulesAreDeclared` | `CheckerRule` |
| `structureGraphsAreDocumented` | `DocFigure` |
| `runRealizesItsDesign` | `SimpleFlowRun` |
| `diagramViewsAreReadable` | `DiagramDocumentationRenderer` |
| `diagramPresentationRetainsSourceIdentity` | `DiagramDocumentationRenderer` |
| `figuresStayWithDesignExplanation` | `DocFigure` |
| `modelDocumentationLinksResolve` | `DocSection` |
| `viewAndViewpointRolesAreSpecified` | `DocumentationViewpoint` |
| `checkerRulesAreImplementationReady` | `CheckerRule` |
| `rustUmlDesignMapsEveryDesignElement` | `XDesignCrate` |
| `stageCliUsesUniformInteraction` | `StageCli` |
| `stageCliPreservesRunIntegrity` | `LocalStageCliRuntime` |
| `operationalRecordsHaveDurableAuthority` | `ProjectStore` |
| `wishHistoryIsRevisioned` | `ItemRevision` |
| `materialsAreContentAddressed` | `Material` |
| `storageSaveIsAtomicAndIdempotent` | `SaveOperation` |
| `workingCacheIsRecoverable` | `CheckoutOperation` |
| `wishStatusProjectsFlowExecution` | `WishProjection` |
| `storageInputAndIntegrityAreChecked` | `DoctorOperation` |
| `linkedWorktreeStoreIsIsolated` | `PlannedWorktreeStoreSelection` (planned) |
| `storeRootSelectionIsConsistent` | `PlannedWorktreeStoreAccess` (planned) |
| `globalStoreNamespaceProtectsOtherContent` | `PlannedWorktreeStoreSelection` (planned) |
| `worktreeStoreMigrationPreservesHistory` | `PlannedWorktreeStoreMigration` (planned) |
| `migratedRunsKeepTheirLifecycle` | `PlannedWorktreeStoreIntegration` (planned) |
| `worktreeStoreConflictsFailClosed` | `PlannedWorktreeStoreAccess` (planned) |
| `worktreeMigrationHasRecoverableCutover` | `PlannedWorktreeStoreMigration` (planned) |
| `legacyCacheRegistrationsStayLegacy` | `PlannedWorktreeStoreAccess` (planned) |
| `checkpointSourceIsProjectOwned` | `CheckpointLibrarySource` |
| `configuredChecksRetainAuthority` | `RsCustomChecker` |
| `operatorDelegatesRunState` | `OperatorWebProgram` |
| `documentContractIsPortable` | `PortableDocumentKit` |
| `documentAuthoringFollowsContract` | `DocumentContractEngine` |
| `documentChecksReportContractGaps` | `DocumentContractEngine` |
| `documentToolsShareOneContract` | `DocumentContractEngine` |

`semanticCheckNamesItsJudges` is the one requirement with two links; every requirement
with a model satisfier added since has one link. The compiler export cannot attribute
two `satisfy` usages of one requirement to their requirement — the question wish-0004
keeps open ([V04](../records/wishes/wish-history.md#v04)). Four later requirements have no
model satisfier, as described below; they are not counterexamples to that attribution rule.
The seven proposed scenario-template requirements in [R19](#r19) also have no `satisfy` links at
this requirements stage. `DocumentContractSource` and `DocumentContractEngine` are candidates for
later design allocation, not established satisfaction; design reconciliation must establish
an answer before adding those links.

`wishProductsAreRetained` and `modelStandsWithoutWishes` have no satisfier: nothing in the
model holds a wish's products to the model, and nothing keeps wish identity out of it.
They are the two requirements this project keeps by hand — the first is what moved
requirements out of wish-0002's file ([V02](../records/wishes/wish-history.md#v02)), the
second is what lets the model check without the wish directory.
`runRecordIsScriptChecked` has no project-wide SysML satisfier either. The new
`simpleRecordCheck` is a declared and executable checker for a SimpleFlow JSON run,
but it does not cover the default V-model or historical wish SysML records
([R03](../records/status.md#r03)).

`documentationEntryMapsDesign`, `documentationSupportsMultipleFlows`,
`umlViewsAreSelectedWithReasons`, and `softwareDesignClaimsNoImplementation` have no SysML
satisfier because their products are the [documentation entry](../index.md), the
per-flow document structure, [UML view applicability](../design/flows/simple-flow/views.md#d52),
and an honest software design status. Their absence from the `satisfy` table is explicit
rather than an omitted obligation. The proposed `sharedChecksHavePinnedOrigins` and
`crossProjectHistoryKeepsItsOwner` also have no implemented satisfier; their proposed
design and planned verification are in [R16](#r16).
The eight proposed worktree-location requirements in [R20](#r20) have planned design
allocations in [D116](../design/storage.md#d116) and verification links in
`WorktreeStoreMigrationVerification`. The `satisfy` usages express planned design
responsibilities, not proof of implementation, verification or acceptance. The existing
`ProjectStore`, `LocalStageCliRuntime`, and `OperatorWebProgram` are integration points
and do not yet implement the proposed behavior.

`XDesignVerification::DocumentationContractReview` declares
planned coverage for all six wish-0008 requirements; its declaration records no pass.
The revision-1, corrective revision-2, directory-layout revision-3, and asset-layout
revision-4 assessments are
recorded in [V07](../records/wishes/wish-history.md#v07) and the wish trace. The declared review case
remains planned coverage rather than a separate passed model-level result.

`XDesignVerification::SimpleFlowViewReview` specifies planned verification coverage for
`flowRunIsRealizedByDesign`, `designedFlowGatesEveryStage`, and
`structureGraphsAreDocumented`. Its `verify` relationships appear alongside the satisfaction
links in the [requirements view](../verification/requirements.md#d15). Declaring
coverage records no passed result; the design review and compiler reports for the current
revision are recorded in the independent wish trace.

`DiagramPresentationReview` specifies verification coverage for the two presentation
requirements. It requires final-asset typography and source-identity checks plus visual
review in the built documentation; the actual outcome is recorded in
[V06](../records/wishes/wish-history.md#v06).

<a id="r13"></a>

## 11. Baseline, margin, and change control

The functional baseline is **working**, not frozen: no SRR decision or approval record is
present. The allocated and product baselines also need their own design and release reviews
([D30](../interfaces/documentation-contract.md#d30)). There is no quantified engineering
margin or post-baseline volatility series to report. Neither should be inferred from a
compiler result or a delivered design wish.

For a requirement change, the intended record is its revised statement and rationale,
affected `satisfy` and `verify` links, design and interface impact, evidence revision,
reviewer decision, and new baseline revision. Independent wish traces record why a change
was requested and judged; the project requirements remain in `mbse/model/`. The current
repository does not implement formal change approval or a frozen RTM workflow.

<a id="r14"></a>

## 12. Model mapping and open issues

| Document concern | Model source | Current limit |
| --- | --- | --- |
| Requirement and rationale | `XDesignRequirements::XDesignRequirement` usages; `DocSection` to [R01](#r01) | Requirement fields are not yet a full SRS acceptance record. |
| Design allocation | `XDesignTraceability` `satisfy` links; [R02](#r02) | Three model obligations, four documentation obligations, two proposed cross-project requirements, and seven proposed scenario-template requirements have no design or implemented satisfier. |
| Software design documentation | UML guide and SimpleFlow design; [D52](../design/flows/simple-flow/views.md#d52) | UML applicability and status have a planned review, not a model satisfier. |
| Planned verification | `XDesignVerification` cases and `verify` links | Case declaration does not record a passed result. |
| User goals | `XDesignDetailedDesign::ProgressWish`, `ReviewEvidence`, and `useCaseDiagram`; [R06](#r06) | The view concerns SimpleFlow and is not a complete deployed-product scenario set. |
| Per-wish outcome | Legacy/bootstrap `mbse/wishes/wish-<id>.sysml`; new store revisions and [V18](../records/wishes/wish-history.md#v18) | A wish outcome does not freeze the project baseline; STORE-01 through STORE-10 evidence applies to its recorded runs and limits. |
| Operational persistence | `XDesignStorage` and [D86](../design/storage.md#d86) | Local store implemented and exercised; whole-store backup, migration, deletion, privacy and remote operation remain outside this slice. |

Open requirements-stage work is to classify and complete every requirement's method,
phase, and acceptance criterion; decide runtime performance, security, deployment, data
retention, and interface contracts as needed; and hold an explicit requirements review.

<a id="r16"></a>

## 13. Checkpoint and operator reconciliation requirements

These requirements distinguish allocation of the existing implementation from proposed
real-project extensions. Their rationale is normative in `XDesignRequirements`; their
design is in the [checkpoint project boundary](../design/checkpoint-library.md#d107).
They do not supply a missing historical requirements review.

| Requirement | Acceptance condition | Verification allocation |
| --- | --- | --- |
| `checkpointSourceIsProjectOwned` | Selecting project B reads/writes B's checkpoint source; initializing a run retains resolved rules/criteria independently of subsequent library edits | `CheckpointOperatorDesignReview`; existing LIB-01, LIB-04, LIB-05, LIB-08 and WEB2-01 cases |
| `configuredChecksRetainAuthority` | Configured syntactic evaluations identify script authority, checker identity, reviewed snapshot and retained report/log bytes; semantic judgments remain separately supplied | `CheckpointOperatorDesignReview`; CUSTOM-02, CUSTOM-03, CUSTOM-06 and LIB-07 |
| `operatorDelegatesRunState` | Operator status/history derive from the selected store; lifecycle actions use the stage CLI; source saves do not assess gates or launch stage work | `CheckpointOperatorDesignReview`; WEB-02..05 and WEB2-01..05 |
| `sharedChecksHavePinnedOrigins` (proposed) | Two projects import an identified immutable definition/bundle with explicit input mapping; updating or removing upstream cannot change an existing binding; ambiguity and digest drift fail explicitly | `CrossProjectCheckpointReview`; future PROJECT-02 and PROJECT-03 |
| `crossProjectHistoryKeepsItsOwner` (proposed) | Colliding wish IDs remain distinct; history links identify a pinned source revision/assessment; unavailable stores remain visible; a supported archival/restore exercise retains original evidence and provenance | `CrossProjectCheckpointReview`; future PROJECT-04..06 |

Launching and managing a working agent, including its selection, monitoring, cancellation,
and session recovery, is outside this requirement baseline. The operator hands off the
current work order and records submitted stage results. Agent CLI execution is also outside
the syntactic checkpoint executor contract: a new binding selects a built-in script, a
project script, or a script-backed library entry. A permitted agent may still supply a
semantic judgment through the stage result; xdesign does not launch that agent. Earlier
agent-authority syntactic assessments remain historical records and do not authorize a new
Agent CLI binding.

The proposed requirements have planned verification links and no implemented satisfiers.
No shared registry, project identity format, cross-project history API, executable
dependency lock or store-transfer contract is delivered by their declaration.

<a id="r18"></a>

## 14. Released checker resources and project execution

| Requirement | Required behavior and acceptance |
| --- | --- |
| `releasedCheckersAreSelfContained` | Built-in scripts, helper and parser come from the executable. A copied release runs without a source checkout or consumer `tools/`; defaults cannot be shadowed by a project file. ISO-01, ISO-02. |
| `checkerExecutorsAreRetained` | New registrations retain the entry and every declared helper/data file by digest. Edits, source deletion, upgrade and cache loss preserve the selected bundle; retained-byte changes refuse execution. ISO-03–05. |
| `checkerExecutionUsesProjectContext` | Execute against the selected project, project-owned environment and unique attempt paths, with retained interpreter identity. Independent projects retain independent results even for equal local IDs. ISO-06–08. |

The model rationales remain normative. [Checker execution](../design/checker-execution.md#d111)
states the ownership and trust boundary; [ISO-01–09](../verification/design.md#d112) defines
observable acceptance. These requirements add no generic input mapper, shared-package
registry, cross-project history federation or operating-system sandbox.

<a id="r19"></a>

## 15. Portable documentation contract

The document kit is a separate, versioned component under `document-kit/`. Its
[rules and templates](../design/document-kit.md#d113) make the document path and
required content visible before a writer creates a page. The selected project
profile is an input to the kit, so a consumer project can use the same operations
with its own root. The original authoring rules in `docs/AGENTS.md` and the
[SysML documentation contract](../interfaces/documentation-contract.md#d27)
remain the basis for this repository. The bundled initial scenario extension
stays within one selected profile and preserves the five existing document
classes and routes; a reviewed derived profile may extend them under its own
identity and digest.

| Requirement | Acceptance condition |
| --- | --- |
| `documentContractIsPortable` | A versioned profile, human-readable rules, and real Markdown templates reside together outside `docs/`; a temporary second project resolves paths from its own root. |
| `documentAuthoringFollowsContract` | Given a document class and slug, planning resolves one permitted destination and required sections; creation and section updates enforce that contract, require caller-provided content, refuse traversal and unexpected overwrites, and leave an existing page intact on failure. |
| `documentChecksReportContractGaps` | Validation reports missing or reordered headings, empty required content, unresolved template slots, invalid paths, broken model locators, and incomplete figure blocks as structured findings. Checks state their scope; model meaning and visual readability require review. |
| `documentToolsShareOneContract` | The CLI and stdio MCP adapter call one engine and return equivalent contract decisions; the skill names when an agent plans, writes, checks, and reviews documents before implementation claims. |
| `documentTemplatesCoverScenarios` (proposed) | The bundled schema 1.1, version 1.1.0 initial catalog contains exactly the five generic defaults and eleven scenario entries below, with exactly one generic default per class; derived 1.1 profiles do not inherit that exact inventory. Every schema 1.1 entry supplies unique identity, class, selection and omission guidance, required inputs/dependencies, template, guide, complete fictional example, implemented checks, view concerns, and sources. Each stored example has exactly one 64-zero sentinel in its exact first-line managed marker's `contract` field, actual non-contract identity and digest version `"2"`, no actual catalog digest, and no sentinel elsewhere in its bytes or another pack resource; missing, duplicate, misplaced, or malformed sentinels fail before writes. All 16 generic/scenario examples pass isolated consumer checks using only DOC-11's exact one-field full-byte comparison and independent actual marker/registry assertions. The single mixed-template example uses all five bundled classes, at least two template IDs in one class, and a concern-to-template/omission list, reuses those stored examples for its `expected_document` references, and passes one whole-project check; it is not a requirement to instantiate all 16 examples in that mixed root. DOC-08, DOC-09, DOC-10, DOC-11, DOC-14; correction subcases S-01 through S-05 and S-07 through S-12. |
| `documentTemplateSourcesAreInspectable` (proposed) | Every schema 1.1 entry has a nonempty known-source list; all eight named research IDs occur in at least one bundled entry. Sources record primary/additional links, retrieval, ideas, and limits. License name/version may be `unknown`/`unlicensed`; `license_url` is then `null`, otherwise it is an absolute HTTP(S) URL without credentials/control characters. Each entry-source pair records `not-copied` or `copied`, and `copied` requires a reviewer/date/permission/evidence record. Originality applies to all catalog content; permitted reviewers judge adequacy. DOC-08, DOC-14, DOC-15. |
| `documentTemplateSelectionIsExplicit` (proposed) | For schema 1.1 plan/create, every generic or scenario ID is valid explicitly and omission selects the same generic default; schema 1.0 projections have no selectable template IDs and reject a supplied selection. New 1.1 creation records the same actual nonzero effective identity in marker and registry. Mixed checks report each document's effective identity, including `template_id=null` and digest version `"1"` for any existing or newly created schema 1.0 document. Operations first resolve the original or bundled-v1 contract, then compare the document's own identity against that contract. Unknown/class-mismatched IDs, caller-supplied output paths, zero or multiple generic defaults for a class, and actual identity mismatches fail without writes. `expected_contract_sha256=Z` is `invalid-request`; sentinel marker/registry identities, copied examples, and forged sentinel registries are `registry-mismatch` without adoption or registration. DOC-09, DOC-10, DOC-13, DOC-14; S-06 and S-07. |
| `documentTemplateChecksUseSelectedContract` (proposed) | The validator applies each document's effective sections, fields, and checks. Every applicable view concern has either a conforming figure or a nonempty decision naming the concern, table/prose/not-applicable alternative, and rationale; there is no class-wide diagram mandate. Missing decisions and malformed figures produce diagnostics with the existing structured fields, failed updates leave existing document bytes unchanged, and rationale and figure quality require review. DOC-03, DOC-10, DOC-11, DOC-15. |
| `documentTemplateCatalogSurfaceIsShared` (proposed) | CLI and MCP expose the same catalog/source data and exact template bytes, optional `template_id`, per-operation and per-path `effective_identity`, and `contract_digest_version` (`"1"` for schema 1.0, `"2"` for 1.1) in every result. Existing MCP resources remain; additive `xdesign-doc-kit://catalog` and `xdesign-doc-kit://template/{document_class}/{template_id}` correspond to CLI `get_contract`. The synthesized v1 projection has `template_id=null` and null scenario-only fields, is exempt from the 1.1 manifest/guide/example/source package, and identifies only the actual selected pack. Selected and MCP-pinned identities use actual `D`, never the example sentinel; examples are not templates and no example bytes become a selected contract. DOC-12, DOC-13; S-06, S-12, and S-14. |
| `documentTemplateCatalogPreservesCompatibility` (proposed) | Explicitly selected valid schema 1.0 and 1.1 packs retain their computed digests and original behavior; every 1.1 release is immutable and later distributions retain prior selectable packs. Under 1.1, automatic legacy routing requires exact bundled `xdesign-sysml-docs@1.0.0` marker/registry identity and digest `1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6`; unknown/custom identities and different 1.1 releases fail without migration. Unmanaged pages are not registered, reclassified, rewritten, or forced into a template, and kit update/attach refuses them. Valid updates preserve effective identity and registry bytes while changing only authorized document content. The v2 digest covers exact sentinel-bearing example bytes without normalization, computed `D=Z` is rejected, and valid example drift requires a new version/digest and refreshed actual `D`. The bundled fixture remains the canonical full caller-input record with catalog identity but neither `D` nor `Z`; the external manifest contains only actual `D`, `fixture_path`, and `fixture_sha256`, while separate S-12/S-16 evidence computes and retains the external manifest's SHA-256; v1 has no examples/sentinel and remains unchanged. Digest drift, unsafe paths/links/HTML, unsupported versions, and network-dependent resources fail closed. DOC-01–07, DOC-10, and DOC-12–14; S-02, S-05 through S-07, and S-11 through S-13. |
| `documentTemplateEvidenceIsBound` (proposed) | Verification records assessed revision, effective contract digests and digest versions, commands/transcripts, isolated and mixed bundled fixtures with full caller inputs and paths/SHA-256, external acceptance-manifest SHA-256/actual digests/fixture references, before/after hashes, diagnostics, unevaluated checks, reviewer identities and semantic-checkpoint authorities, findings, and residual risks. Correction evidence also binds S-01 through S-16, preserves the parent run/revisions and evidence, records the separate design/API/model/product/test follow-up without claiming it complete, and retains all 26 unevaluated SysML rules. DeepSeek supplies the independent semantic review for this wish and Kimi supplies a separate runner evidence/limitations judgment; both are required and neither substitutes for the other or for structural output. DOC-15; S-15 and S-16 are obligations/evidence, not executed product cases. |

The bundled initial scenario IDs are normative scope, not a requirement to use every
scenario in every project or to constrain a reviewed derived profile:

- Requirements: `system-requirements`, `change-requirements`.
- Architecture: `system-context`, `component-composition`.
- Design: `interaction-flow`, `state-machine`, `data-model`.
- Interface: `operation-contract`, `data-contract`.
- Verification: `verification-test-plan`, `design-review`.

The retained generic IDs map one-to-one to their named classes: `generic-requirements`,
`generic-architecture`, `generic-design`, `generic-interface`, and `generic-verification`.
They preserve the corresponding v1 chapter and field contracts and are valid explicit
selections as well as omission defaults. Each class declares exactly one default. A catalog
routing guide and mixed-template example map concerns to selected templates or reasoned
omissions; neither automatic selection nor the list establishes semantic coverage. A new
SysML template catalog or skeleton generator, in-place template migration, bundled
flow-specific/per-API route generation, remote retrieval, and automatic engineering acceptance
are outside this extension. Reviewed derived profiles may own additional entries/routes under
their own identity and digest.

For schema 1.1 stored examples, `Z` means exactly 64 lowercase ASCII zero characters and `D`
means the actual selected catalog digest. The first line has the exact managed-marker form
`<!-- document-kit: profile={profile_id}@{version} class={document_class} template={template_id} contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->`.
`Z` occurs exactly once in the complete example and nowhere else in the pack. The digest covers those
exact stored bytes without normalization, and `D == Z` is always invalid. Each digest-covered bundled
fixture is the canonical full plan/create caller-input record with selected profile/version/digest-version
identity, but neither `D` nor `Z`. An external acceptance manifest outside the digest closure contains
exactly actual `D`, `fixture_path`, and `fixture_sha256`, never a self-hash field. Separate S-12/S-16
evidence computes and retains the manifest's SHA-256; any `manifest_sha256`, `self_sha256`, or
equivalent field is `fixture-mismatch` before creation without field-exclusion normalization. Digest,
fixture, or retained manifest-hash drift fails as stale. DOC-11 copies the stored example only in
memory, replacing only its parsed first-marker `contract`
span with `D`, and compares every resulting byte with the created document after independently asserting
actual created marker/registry identity. It does not modify product bytes, stored examples, digests, or
registries. Global replacement, marker stripping, body-only comparison, whitespace/line-ending
normalization, and other-field normalization are forbidden.

[DocumentKitVerification](../verification/design.md#d114) defines the
cross-project and failure cases. The current SimpleFlow wish records actual
commands and evidence separately; these acceptance conditions alone are not
an executed result. DOC-01 through DOC-07 retain their regression meaning;
DOC-08 through DOC-15 are planned coverage, not passed results.

The versioned profile, rules, templates, and skill are design source. The
scenario catalog and new acceptance cases are proposed requirements, not a
claim of implementation. Their seven proposed requirements have planned verification
links but no `satisfy` links until design establishes an answer; the existing
source and engine elements are allocation candidates only. Implementation status
and executed results are tracked in wish and review records, not inferred from
these requirement statements.


<a id="r20"></a>

## 16. Linked-worktree operational store location and migration

The eight model requirements below are proposed behavior. The currently implemented
[storage design](../design/storage.md#d86) and [format contract](../interfaces/store-format.md#i11)
select a project-local store by default and bind a store to its root and database's
filesystem identities; they provide no supported store migration. This requirement adds a
specific verified local-to-home migration, not a general backup, restore, schema migration,
history federation, or assumption that Git worktrees have a shared project identity.
The [planned cases](../verification/design.md#d115) are not executed results.

| Requirement | Normative behavior and rationale |
| --- | --- |
| `linkedWorktreeStoreIsIsolated` | When the selected project is in a valid linked Git worktree and no root override is given, the effective store is a dedicated directory under the effective user's `~/.xdesign`; the ordinary primary checkout and non-Git project retain `<project>/.xdesign`. The canonical selected project root, validated repository identity and worktree identity bind one deterministic location across invocations. Path aliases of the same project agree; distinct selected project roots, linked worktrees and repositories remain isolated even if basenames, branches or wish IDs match. If identity or home resolution is ambiguous or fails, report it and refuse instead of falling back to a local or another project's store. This prevents both worktree-local collisions and global history mixing. |
| `storeRootSelectionIsConsistent` | `xdesign store ... --root` overrides `XDESIGN_STORE_ROOT`, which overrides the context default; `xdesign web --store-root` has the same precedence over environment and default. `init`, `next`, and `check` use the environment before the default; direct store API calls retain their explicit-root behavior. Relative store CLI and lifecycle roots resolve against the invocation directory, while relative Web roots resolve against `--project-root` (or the default project directory). An empty explicit root is refused. A supplied root selects that exact root subject to integrity and migration fences, with no automatic migration, import or fallback from another root. Web status/health, reads and delegated lifecycle writes use the same effective root, including when its explicit option differs from the process environment. This preserves caller control and prevents split read/write authority. |
| `globalStoreNamespaceProtectsOtherContent` | Treat `~/.xdesign` as a container, never as the worktree store itself. Create or open only a dedicated, binding-checked child location; leave existing unrelated files and permissions untouched. If the container is itself an existing store root, refuse this default with an actionable diagnostic instead of adding a foreign child to that store. A foreign, unsafe, incompatible, inaccessible or mismatched destination or detected binding collision is refused without adoption, overwrite or damage. Report the selected root. This protects other uses of the home directory. |
| `worktreeStoreMigrationPreservesHistory` | Offer an explicit, supported migration for a valid quiescent old `<project>/.xdesign` in a linked worktree into its global default. Before cutover compare all generic item IDs/kinds and heads, every immutable revision's metadata and complete URI/role/digest/length manifest, every referenced byte and its digest, and exact operation request bytes, terminal outcomes and results. Keep all of them, including earlier revisions, non-wish records and evidence; invent no revision, judgment or receipt. Preserve the store identity or retain an inspectable source-to-destination identity mapping. Destination filesystem identity must be established under a verified migration, not by treating a raw copy or move as a valid store. Refuse unsupported or corrupt sources and unresolved pending operations. This preserves logical authority across an identity-bound physical relocation. |
| `migratedRunsKeepTheirLifecycle` | For both supported flat flows, preserve wish/run identity, frozen plan/work, visited stages, artifact revisions, assessments, evidence, derived status and unchanged-head state tokens. After cutover an active or blocked run can continue under existing stale-result, artifact-drift and expected-head rules; an already completed run returns completion without reopening or adding a revision. Exact terminal operation retry returns the original outcome without another effect. Removing only regenerable store-backed cache still permits reconstruction without overwriting changed working artifacts. This keeps old runs usable and auditable. |
| `worktreeStoreConflictsFailClosed` | If an old local store or an unclassifiable local `.xdesign` candidate exists, a default operation must diagnose it rather than silently initialize or use an unrelated empty global store: require the explicit migration or explicit root selection. If both local and global roots exist, the global default is permitted only when durable migration provenance proves the relationship and the retained local source still matches the recorded cutover state. A source absent after verified cutover need not remain present; a newly appearing or changed source without that proof makes the default ambiguous. Diagnose the roots and refuse mutation; do not deduplicate, merge, overwrite or choose by name, store ID, matching head, timestamp or presumed shared Git identity. Explicit root selection permits safe inspection subject to normal integrity and migration fencing. This prevents hidden or diverging histories. |
| `worktreeMigrationHasRecoverableCutover` | Migration requires operator-confirmed quiescence of pre-upgrade/non-cooperating clients, and no unresolved pending operations or writer locks. All supported upgraded readers and writers coordinate with migration at both source and destination so no writes race verification or cutover. Migration preserves the source and never deletes or overwrites it; supported clients refuse writes to that retained source while cutover stands. Durable state distinguishes incomplete preparation from verified cutover: before the decision the destination is non-authoritative, after it the verified destination is authoritative. Interrupted or response-uncertain operations expose status and allow exact retry to a single outcome; before the decision a safe rollback may instead retain source authority. Recovery never uses a partial target or silently creates a fresh store. Diagnostics give source, target, decision status and actionable recovery. Older processes cannot be fenced by the new binary; the operator must keep them stopped through cutover and avoid writing the retained source afterward. Later source changes must be detected before default mutation. This boundary avoids pretending that locks unknown to old clients are sufficient. |
| `legacyCacheRegistrationsStayLegacy` | Genuine legacy registrations under `cache/xdesign` and their Git wish traces remain on their legacy backend, preserving active/completed behavior and history; neither migration nor cache cleanup treats them as disposable store-backed copies. Distinguish them from regenerated persistent-run cache. If a genuine legacy registration and selected store item share a wish ID, surface the conflict, provide read-only inspection of both, and refuse mutation of either until resolved; do not silently mask, overwrite or merge them. This protects the authoritative older record. |

A new worktree with no local store can initialize the isolated global default on demand.
A retained local source after successful migration is a recovery copy, not a second
write authority. Explicit inspection can still select it; a subsequent write by an
older client is outside the coordinated-client guarantee and must be diagnosed when
observed. Renaming, copying or moving a project/worktree or a home directory is not itself
proof of identity or a supported whole-store restore: the design must document the identity
stability boundary and an explicit selection/recovery route, and must refuse ambiguous
adoption. User-managed backups, physical power-loss certification and synchronization
with unsupported or hostile writers are outside this requirement's acceptance claim.

The model allocates these requirements to the explicitly planned selection, migration,
access and integration boundaries in [D116](../design/storage.md#d116), and
`WorktreeStoreMigrationVerification` supplies planned verification links. These links do
not establish a current implementation or an executed case. Later design must reconcile the
existing storage, API, Web, lifecycle and operator contracts with these requirements,
including the current default of a project-local `.xdesign`.
