# System behavior and architecture design

> Project: xdesign | Version: working draft, 19 October 2026 | Status: working design;
> no allocated baseline or PDR review | Audience: wish owners, architects, module owners,
> integrators, and reviewers.
>
> Basis: [requirements inventory](../requirements/requirements.md#r01),
> [system composition](system-composition.md#b01), and
> `mbse/model/architecture.sysml`. The page records the architecture design and its
> unresolved choices; a topic appears only where the project has a design position or an
> open issue that affects a decision.

| Version | Date | Change | Review |
| --- | --- | --- | --- |
| Working draft | 19 October 2026 | Collected the current architecture decisions, interfaces, open allocations, and PDR gaps | No PDR recorded |

<a id="b12"></a>

## 1. Architecture design

The architecture defines a logical composition and a revision-aware progression rule.
The detailed SimpleFlow design refines that composition for one selected flow; it does
not turn every flow into the same implementation. The subsections below separate modeled
structure and behavior from open physical, data, and security choices.

<a id="b13"></a>

### 1.1 Logical structure, interfaces, and model organization

`XDesign` owns a flow, controller, wishes, and traces and binds the controller to the
flow and collections. Its one-trace-per-wish assertion states the central ownership
invariant. The [composition figure and part table](system-composition.md#b01) show
this structure; the [interface figure](system-composition.md#b02) shows the two
typed exchanges and their item directions. `Stage`, `Checkpoint`, and `Checker` belong
to reusable flow definitions; `FlowExecution`, `StageRecord`, `ActivityRecord`, and
`CheckpointAssessment` record one wish's use of them ([F01](../design/flow.md#f01),
[E01](../interfaces/record-format.md#e01)).

| Interface | From → to | Content and direction | Medium or protocol | Draft ICD |
| --- | --- | --- | --- | --- |
| `WishInterface` (IF-001) | Requester ↔ controller | Submission in; registered wish and trace references out | SysML typed port; runtime transport undecided | [I02](../interfaces/contracts.md#i02) |
| `FlowInterface` (IF-002) | Selected flow → controller | Flow definition into controller | SysML typed port; runtime transport undecided | [I03](../interfaces/contracts.md#i03) |
| `CheckCommandInterface` (IF-003) | Designed run ↔ checker command | Command request and result for a syntactic gate | Detailed-design interface; runtime protocol undecided | [I04](../interfaces/contracts.md#i04) |
| `JudgmentInterface` (IF-004) | Designed run ↔ judge | Criteria/evidence and semantic outcome | Detailed-design interface; runtime protocol undecided | [I05](../interfaces/contracts.md#i05) |

All four IF identifiers already designate draft, modeled interfaces; they are document
inventory IDs, not production interface versions. Their directions and item types are
specified in the model and [ICD](../interfaces/contracts.md#i01). Runtime payload schemas,
failure and timing behavior, contract owners, review authorities, criticality ratings,
and a frozen ICD baseline remain open. Package organization is modeled by
`XDesignRustUml::projectPackageIntegration`; its documentation
belongs here because dependency direction is an architecture concern. The Rust UML
package models the implemented `xdesign::core` + `xdesign::simple_flow` slice, not a
separately owned application or independent Rust crate
([U65](../design/api/index.md#u65)).

<a id="b14"></a>

### 1.2 Runtime behavior

`Controller::receiveWish` registers the wish and trace, selects a flow, makes planned
stage records, and enters the initial stage. `recordActivity` appends work at the active
visit and raises its revision when artifacts change. `progressWish` assesses current
exit checkpoints, advances only if each passes for that revision and any subflow is
complete, or retains the stage as blocked for corrective work. The
[lifecycle figure](wish-lifecycle.md#a03), [controller figure](../design/lifecycle.md#f04),
and [rework account](../design/lifecycle.md#f05) give the complete
modeled path.

The selectable flows have different stage and gate sets ([F03](../design/flow.md#f03)).
For `designedSimpleFlow`, the [activity](../design/flows/simple-flow/views.md#d18),
[sequence](../design/flows/simple-flow/views.md#d19), and
[state](../design/flows/simple-flow/views.md#d20) views refine evaluation,
creation, blocked status, and rework. Concurrency, scheduling, timeout behavior,
retry limits, and failure recovery outside the modeled blocked/rework path have no
architecture decision or verified runtime result.

<a id="b15"></a>

### 1.3 Physical and deployment view

The selected storage boundary is local: one application process uses SQLite and
content-addressed files beneath `.xdesign/` or `XDESIGN_STORE_ROOT`; working material is
under `cache/`. [D86](../design/storage.md#d86) defines the write and failure boundaries.
This is not a network service, production deployment, multi-user allocation, or capacity
claim. Those concerns, execution-node diagrams, and operational service levels remain open.

<a id="b16"></a>

### 1.4 Data view

The logical information model consists of `Wish` identity and description; `Trace`
status, model URI, and root execution; `FlowExecution` and its stage visits; and
revisioned activities and checkpoint assessments ([E01–E04](../interfaces/record-format.md#e01)).
The reference from a trace to its wish and the requirement that every wish has exactly
one trace define logical cardinality. Evidence URIs refer to artifacts retained in the
current repository workflow; [E04](../design/evidence-and-revisions.md#e04)
explains the record semantics.

`XDesignStorage` adds generic item identity/kind, immutable numbered revisions, complete
URI-to-SHA-256 manifests, blobs, and operation receipts. The [store format](../interfaces/store-format.md#i11)
owns physical keys and versions; [D86](../design/storage.md#d86) owns save, checkout and
integrity behavior. Retention deletion, whole-store backup, migration from other stores,
and remote data services remain outside this slice. The design must preserve visit history,
assessment revision, and evidence references; compiler checks do not prove that behavior.

<a id="b17"></a>

### 1.5 Security view

`agentMayJudge` and `humanMayJudge` constrain who *may provide an assessment* in the
model ([F02](../design/adapters.md#f02)); they do not authenticate an
actor or authorize access to files. The project has not specified identity providers,
roles and permissions, trust boundaries, encryption, secret handling, audit retention,
or threat scenarios. The existing trace records an assessor and evidence URI, but does
not establish a tamper-resistant audit log. Security design and verification remain
open before any shared or networked deployment.

<a id="b18"></a>

## 2. Function–concept–form and concept selection

The current model gives a traceable partial mapping from purpose to form. It does not
contain a completed architecture trade study, so the existing design is described here
without a numerical selection score.

<a id="b19"></a>

### 2.1 Function decomposition

| Solution-neutral function | Input → output | Modeled refinement |
| --- | --- | --- |
| Register a requested outcome | Description and selected flow → wish, trace, initial execution | `Controller::receiveWish` |
| Preserve work and evidence | Current visit and artifact changes → activity, revision, assessment history; exact save → stored revision and receipt | `Controller::recordActivity`, `StageRecord`, `ProjectStore`, `SaveOperation` |
| Decide whether work may advance | Stage, current artifacts, checkpoint criteria → authorized next step or blocked visit | `Controller::progressWish`, `Checkpoint` |
| Explain system and wish rationale | Reusable model and per-wish history → linked documentation and legacy/store trace | `DocSection`, `DocFigure`, `ItemRevision` |

The [flow and controller descriptions](../design/flow.md#f01)
give the native decomposition. The system does not yet model service performance or
parallel scheduling requirements for these functions.

<a id="b20"></a>

### 2.2 Concepts and working principles

The recorded working principle is a reusable project model plus an independent trace for
each wish, advanced through a selected flow by checkpoint assessments at an artifact
revision. Syntactic criteria use fixed commands; semantic criteria use accountable
judgment. This principle explains the `Flow`/`FlowExecution` and
`Checkpoint`/`CheckpointAssessment` separations. `VModel`, `SimpleFlow`, `WishVerifyFlow`,
and `DesignFlow` are selectable workflow forms for different outcomes
([F03](../design/flow.md#f03)); they are not scored candidate
architectures. No alternative storage, orchestration, or deployment concepts have
been compared in a recorded selection study.

<a id="b21"></a>

### 2.3 Form and allocation

| Function | Working principle | Form that carries it | Derived obligation |
| --- | --- | --- | --- |
| Register and assign | One wish, one trace, selected flow | `XDesign`, `Controller`, `Trace` | `modelStandsWithoutWishes`, `wishProductsAreRetained` ([R01](../requirements/requirements.md#r01)) |
| Record and revisit | Append visits and assess the current revision | `FlowExecution`, `StageRecord`, `CheckpointAssessment` | `runRecordIsScriptChecked` is exercised for the new SimpleFlow JSON run; VModel and historical SysML records remain outside this checker ([R03](../records/status.md#r03)) |
| Gate progress | Fixed syntactic check or permitted semantic judgment | `Checkpoint`, `Checker`, `CheckerRule`; SimpleFlow checkpoint design | `checkpointKindIsDeclared`, `syntacticCheckIsFixed`, `checkerRulesAreDeclared` ([R01](../requirements/requirements.md#r01)) |
| Present and locate design | Native views plus model/document metadata | `DocSection`, `DocFigure`, renderer, documentation audit | `figuresStayWithDesignExplanation`, `modelDocumentationLinksResolve` ([D31](../interfaces/documentation-contract.md#d31)) |

This is a design allocation table, not evidence that the application functions have
executed. The satisfaction and planned verification links are in
[R02](../requirements/requirements.md#r02).

<a id="b22"></a>

### 2.4 Design variables and derived requirements

The model fixes some variable domains: `selectedFlow` chooses a defined flow usage;
`Checkpoint::kind` is syntactic or semantic; stages own successors and exit checkpoints;
`artifactRevision` identifies the assessed artifact state. The requirements in
[R01](../requirements/requirements.md#r01) derive obligations from these decisions,
including a current-revision assessment, retained wish products, and declared checker
rules. No numeric ranges for capacity, throughput, latency, storage size, or reliability
have been set. Those values cannot be inferred from the SysML type declarations.

<a id="b23"></a>

### 2.5 Candidate generation and selection

The repository records no morphological matrix, screening criteria, weighted trade
study, Pareto comparison, or rejected-concept rationale for the architecture as a whole.
The existing flow variants are defined because different wishes need different stage
sets, but their presence does not prove an architecture-wide trade study. A PDR-ready
selection record would compare candidate working principles against agreed criteria,
record infeasible combinations, and identify the chosen configuration and residual
risks. That record is an open item, not a retrospective score assigned here.

<a id="b24"></a>

## 3. Key technical decisions and ADR index

| Current decision | Evidence of chosen form | Alternatives and ADR status |
| --- | --- | --- |
| Keep reusable model content independent of wish records | [A02](index.md#a02), `modelStandsWithoutWishes` | No ADR or recorded candidate comparison |
| Gate each stage exit using current-revision assessments | [B03](system-composition.md#b03), [E03](../design/evidence-and-revisions.md#e03) | No ADR or quantified trade-off |
| Route syntactic checks and semantic judgments separately | [F02](../design/adapters.md#f02) | No ADR; four SimpleFlow commands execute, while VModel checker implementations remain open |
| Present model-derived figures with source and document locators | [D27–D31](../interfaces/documentation-contract.md#d27) | Design and audit exist; no architecture ADR |

The table indexes choices visible in the model. It is not an ADR collection and does not
claim that alternatives were formally rejected.

<a id="b25"></a>

## 4. Quality attribute design

| Attribute | Current target or invariant | Design means | Verification and margin |
| --- | --- | --- | --- |
| Trace integrity | Exactly one trace per wish; completed visits and delivery require authorization | `XDesign`, `StageRecord`, `FlowExecution`, `Trace` constraints | SysML declarations and historical assessments exist; compiler does not evaluate constraint bodies. No measured margin ([B03](system-composition.md#b03)) |
| Model/document consistency | Every modeled locator resolves and every declared figure has a local explanation | `DocSection`, `DocFigure`, documentation audit | Stand-alone audit rerun for wish-0008 revision 4 after asset placement; semantic agreement remains a review judgment ([V07](../records/wishes/wish-history.md#v07)) |
| Primary diagram readability | Nine SimpleFlow figures: at least 11 CSS-pixel text at 900-pixel article width, with no clipped labels or ambiguous overlaps | Model-owned presentation metadata and diagram-specific renderer | Final-figure review for wish-0007 revision 1; this is limited to nine primary figures ([V06](../records/wishes/wish-history.md#v06)) |
| Reproducible model checking | Pinned compiler reports parsing and supported name/rule checks | Bundled `sysml` executable and retained reports | Parse/check evidence exists; unevaluated rules are not successes ([T04](constraints.md#t04)) |
| Performance, availability, security | No quantified targets | No allocated architecture means yet | No analysis, test, or margin recorded |

Quality claims must be tied to the revision and scope of their evidence. A documented
constraint is not a measured runtime property.

<a id="b26"></a>

## 5. Interfaces and cross-disciplinary design

The four named model interfaces and their content are summarized in [B13](#b13).
`WishInterface` and `FlowInterface` are architecture-level; the checker-command and
judgment interfaces refine one designed flow. The
[draft ICD](../interfaces/contracts.md#i01) records all four interface IDs, modeled
fields and directions, and unassigned contract owners and review authorities. The
remaining protocol, failure, compatibility, and criticality decisions prevent a frozen
ICD baseline. There is no N²/DSM coupling analysis. Integration
verification is a stage of `VModel` and points back to system architecture
([F03](../design/flow.md#f03)); it has no executed interface
verification result for this design.

The work spans model, documents, renderer, and future application code. Their current
cross-artifact link is the model's `DocSection`/`DocFigure` metadata and the generated
asset path; the audit checks mechanical links ([D31](../interfaces/documentation-contract.md#d31)).
No hardware/software allocation or other physical discipline interface has been
selected.

<a id="b27"></a>

## 6. Risks and trade-offs

| Risk or failure | Source | Effect | Design response | Status |
| --- | --- | --- | --- | --- |
| A declared constraint is treated as an executed proof | Compiler capability review ([T04](constraints.md#t04)) | Unauthorized progress could be inferred from a clean parse/check report | Preserve unevaluated-rule diagnostics; require separate check or judgment evidence | Open |
| A planned checker URI is mistaken for a runnable gate | [R03](../records/status.md#r03) | A `VModel` syntactic gate cannot pass by its absent command | Implement and verify each VModel command and rule-list agreement; the four SimpleFlow commands are already executable | Open for VModel |
| A wish trace becomes the only copy of a requirement or design | Model boundary requirement ([A02](index.md#a02)) | Rewriting or moving a wish record could lose project design | Keep reusable products in `mbse/model/`; inspect this boundary during review | Open as an unenforced obligation |
| Documentation links resolve but the explanation is wrong | Audit scope ([D31](../interfaces/documentation-contract.md#d31)) | Reviewers may trust a technically valid but misleading figure | Keep semantic review separate from the locator audit | Open for every new revision |
| Deployment and security assumptions are left implicit | Open physical and security views ([B15](#b15), [B17](#b17)) | A later implementation may choose incompatible persistence or trust boundaries | Define environment, threat model, data policy, and allocation before deployment baseline | Open |

No FMEA, FTA, causal model, quantified risk score, or formal acceptance of these residual
risks is recorded.
