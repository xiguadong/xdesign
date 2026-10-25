# System context

<a id="b05"></a>

## 1. Introduction and constraints

This is the architecture of the xdesign tool, not of a system that an xdesign user might
develop. Its modeled boundary comprises the workflow definition, controller, wishes, traces,
and the operational `ProjectStore`. Its repository also contains the SysML model, documents,
renderer, compiler integration, and documentation audit. A complete workflow application
has not been implemented ([R03](../records/status.md#r03)).

<a id="b06"></a>

### 1.1 Goals and scope

The architecture must make both system composition and the reason for each wish's outcome
inspectable. The project model states reusable requirements, structure, workflows, and
constraints; one trace per wish records selected flow, visits, assessments, evidence, and
the path to delivery. The requirements inventory and satisfaction links are in
[R01–R02](../requirements/requirements.md#r01). [Storage design](../design/storage.md#d86)
specifies the local operational boundary; external service APIs and production operations
remain undesigned.

<a id="b07"></a>

### 1.2 Terminology

| Term | Meaning in this architecture |
| --- | --- |
| Project model | Wish-independent SysML definitions and requirements in `mbse/model/`. |
| Wish trace | One wish's execution and rationale record: a legacy/bootstrap SysML model or a store-backed revisioned run. |
| Flow definition | Reusable plan of stages and checkpoints; it is distinct from a `FlowExecution`. |
| Stage visit | One `StageRecord` for an attempt at a stage; rework appends another visit. |
| Current assessment | The latest assessment per checkpoint for a visit, selected independently of revision; it authorizes progress only when its evaluated revision equals the visit's artifact revision and its outcome passed ([E02–E03](../design/evidence-and-revisions.md#e02)). |
| Syntactic checkpoint | Gate judged by a fixed checker and its reported evidence. |
| Semantic checkpoint | Gate judged by a permitted agent or human with rationale and evidence. |
| PDR | Preliminary design review of an allocated architecture baseline; no such review is recorded. |

The record definitions and revision semantics are specified in
[E01–E04](../interfaces/record-format.md#e01).

<a id="b09"></a>

### 1.3 Stakeholder concerns

| Stakeholder | Concern | Addressed in |
| --- | --- | --- |
| Wish requester | A submitted wish keeps an inspectable outcome and trace | [B11](#b11), [A03](wish-lifecycle.md#a03) |
| Model author | Reusable model content remains separate from each wish's record | [B13](system-behavior.md#b13), [A02](index.md#a02) |
| Reviewer or judge | Current evidence, criteria, and authority are visible before advancement | [B14](system-behavior.md#b14), [E04](../design/evidence-and-revisions.md#e04) |
| Implementer | Parts, interfaces, and unresolved choices are distinguishable | [B13](system-behavior.md#b13), [B26](system-behavior.md#b26) |
| Verifier | Requirement links and planned verification can be inspected without treating a declared test as a pass | [R02](../requirements/requirements.md#r02), [B28](../records/reviews/architecture-baseline.md#b28) |

<a id="b10"></a>

### 1.4 Architecture drivers

The leading drivers are traceability of each wish, independence of reusable model
content, authorization at the current artifact revision, and a repeatable split between
fixed checks and engineering judgment. They produced the `XDesign` composition, the
`Trace`/`FlowExecution`/`StageRecord` record hierarchy, and the `Checkpoint`/`Checker`
split. Documentation readability and model/document locators later shaped the view and
metadata contract ([D27–D31](../interfaces/documentation-contract.md#d27)). The project
has not assigned numerical priorities or weights to these drivers.

<a id="b11"></a>

## 2. System context and boundary

The human requester is an external actor. The model represents that role with a
`WishRequester` part inside the `Workflow` composition and connects its port to the
controller through `XDesignWorkflow::WishInterface`. That interface carries a submission
toward the controller and returns references to the registered wish and trace. The
composition's selected `Flow` is supplied through `FlowInterface`; the controller uses
it to create the execution plan. The typed context and directions are visible in the
[workflow interface figure](system-composition.md#b02). The
[wish lifecycle figure](wish-lifecycle.md#a03) shows what happens after registration.

| Participant or dependency | Boundary exchange | Current specification |
| --- | --- | --- |
| Human requester, represented by the `WishRequester` part | Description in; registered `Wish` and `Trace` references out | Native `WishExchange` ports and `WishInterface` |
| Selected flow definition | `Flow` reference into the controller | Native `FlowExchange` and `FlowInterface`; flow is part of the modeled composition |
| Checker process or semantic judge | Artifacts and criteria out; report or rationale and evidence in | Generic checkpoint contract; designed SimpleFlow ports in [D10](../design/flows/simple-flow/views.md#d10) |
| Local compiler and repository files | Model sources in; parse/check reports and documentation assets out | Development toolchain in [T01](constraints.md#t01), not an application deployment interface |
| Operational store | Revision manifests and exact operation requests in; stored heads, materials, query views and receipts out | Local [store format](../interfaces/store-format.md#i11); SQLite and content blobs beneath `.xdesign/` or an explicit override |

The current context model identifies information crossing typed ports. It does not
establish a network boundary, user identity protocol, external service, transport, or
operational trust boundary.
