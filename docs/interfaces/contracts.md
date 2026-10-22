# Interface control document

> Project: xdesign | Revision: 0.3 | Status: draft design inventory | Audience:
> model authors, module implementers, integrators, and reviewers.  
> Basis: `XDesignWorkflow` and `XDesignDetailedDesign` in the project SysML v2 model.
> No interface baseline has been frozen or implementation conformance demonstrated.

| Revision | Date | Change | Review |
| --- | --- | --- | --- |
| 0.1 | 2026-10-19 | Record the four declared logical interfaces and unresolved field-level contracts | Pending |
| 0.2 | 2026-10-19 | Clarify that all four draft interfaces exist, while operational fields, owners, and baseline approval remain open | Pending |
| 0.3 | 2026-10-19 | Bind fixed checks to a checker, run-record snapshot, artifact revision, and exact rule report | Pending |

<a id="i01"></a>

## 1. Scope and interface inventory

This document inventories four existing *draft modeled* exchanges at the workflow boundary
and within the designed SimpleFlow automation. IF-001 through IF-004 have definitions,
typed ports, directions, and item flows; they are not missing interfaces. The provider
and consumer names below are model roles, not deployed processes. Transport,
remote serialization, timing, failure behavior, and version policy remain open in [I09](#i09),
so the ICD has no frozen implementation baseline. The
[architecture interface view](../architecture/system-composition.md#b02)
and [SimpleFlow interface view](../design/flows/simple-flow/views.md#d10)
show the same connections.

| ID | Model interface | Provider or initiator | Consumer or recipient | Exchange class | State |
| --- | --- | --- | --- | --- | --- |
| IF-001 | `XDesignWorkflow::WishInterface` | `WishRequester` submits; `WorkflowController` returns registration | Controller and requester in opposite directions | Workflow boundary | Modeled; implementation unverified |
| IF-002 | `XDesignWorkflow::FlowInterface` | `WorkflowFlow` | `WorkflowController` | Internal flow selection | Modeled; implementation unverified |
| IF-003 | `XDesignDetailedDesign::CheckCommandInterface` | `SimpleFlowRun` invokes; `CheckerCommand` reports | Command and run in opposite directions | Local fixed check | Rust subprocess adapter and four Python commands execute the first JSON exchange; no remote baseline |
| IF-004 | `XDesignDetailedDesign::JudgmentInterface` | `SimpleFlowRun` presents criteria and artifacts; `Judge` returns assessment | Judge and run in opposite directions | Supplied semantic judgment | Local trait and explicit supplied-judge adapter execute; identity proof and independent decision process remain open |

Criticality has not been assigned for these interfaces. A modeled connection is not an
acceptance result. Sections I02–I05 record existing draft fields and distinguish them
from operational details still to be decided.

| Interface class | ID | Contract owner | Review authority | State |
| --- | --- | --- | --- | --- |
| Workflow boundary | IF-001 | Unassigned | Unassigned | Assign before an external interface baseline |
| Internal flow selection | IF-002 | Unassigned | Unassigned | Assign before architecture review |
| Fixed checker exchange | IF-003 | Unassigned | Unassigned | Assign before a controlled ICD baseline and CDR |
| Semantic judgment exchange | IF-004 | Unassigned | Unassigned | Assign before identity and authority design |

The model names communicating roles but no person or team that owns each contract or can
approve its revision. Contract ownership and review authority are both unassigned for
every ID; neither can be inferred from a port name. Assign them before baseline approval.

<a id="i02"></a>

## 2. IF-001 — wish submission and registration

`WishExchange` connects `requester.wishPort : ~WishExchange` to
`controller.wishPort : WishExchange`. The controller's `receiveWish` action is bound to
the port values. The workflow owns the `Wish` and `Trace`; the requester receives the
registration item and trace reference ([B02](../architecture/system-composition.md#b02)).

| Model flow | From → to | Field and type | Meaning |
| --- | --- | --- | --- |
| `submission` | Requester → controller | `description : String` | Requested outcome text |
| `wishRegistration` | Controller → requester | `registeredWish : Wish` | Registered wish item |
| `traceRegistration` | Controller → requester | `registeredTrace : Trace` reference | Independent execution trace reference |

| Operational contract field | Current state |
| --- | --- |
| Provider and consumer | `WishRequester` and `WorkflowController`, with bidirectional registration exchange |
| Protocol, transport, payload layout | Open; only SysML item types are declared |
| Direction and frequency | Directions above; rate and ordering guarantee open |
| Timeout, retry, idempotency | Open |
| Version and compatibility | Open |
| Security and criticality | Authentication, access, protection, and rating open |
| Error codes and caller action | Open; no executable endpoint exists |

Protocol/transport, payload schema beyond these types, response ordering, timeout,
retry/idempotency, versioning, authentication, and error responses are **not specified**.
The one-trace-per-wish invariant is defined in [B01](../architecture/system-composition.md#b01);
it does not itself specify the external registration protocol.

<a id="i03"></a>

## 3. IF-002 — selected flow publication

`FlowInterface` connects `'flow'.flowPort : FlowExchange` to
`controller.flowPort : ~FlowExchange`. The provider publishes
`selectedFlow : Flow` as a reference through the `flowDefinition` item flow; the
controller binds it to `defaultFlow` ([B02](../architecture/system-composition.md#b02)).
The model names neither a wire format nor a dynamic flow-switch procedure. Ownership,
replacement rules, version compatibility, error behavior, and update timing remain open.

| Operational contract field | Current state |
| --- | --- |
| Provider and consumer | `WorkflowFlow` publishes to `WorkflowController` |
| Protocol, transport, payload layout | Open; modeled content is a `Flow` reference |
| Direction and frequency | Flow → controller; selection/update timing and rate open |
| Timeout, retry, idempotency | Open |
| Version and compatibility | Open |
| Security and criticality | Access and rating open |
| Error codes and caller action | Open; no executable endpoint exists |

<a id="i04"></a>

## 4. IF-003 — fixed checker invocation and result

`CheckCommandInterface` connects `SimpleFlowRun.check : ~CheckCommandExchange` to
`CheckerCommand.command : CheckCommandExchange` within `SimpleFlowAutomation`.
The [gate rules](../design/flows/simple-flow/definition.md#d06) define what the planned
commands must evaluate; [D07](../design/flows/simple-flow/definition.md#d07)
records their proposed argument and report contracts.

| Model flow | From → to | Field and type | Meaning |
| --- | --- | --- | --- |
| `invocation` | Run → command | `wishModel : String` | URI of the independent wish SysML trace that gives the run its wish identity |
| `recordArgument` | Run → command | `recordUri : String` | URI of the executable run-record snapshot judged by this invocation |
| `checkerArgument` | Run → command | `checkerId : String` | Exact declared checker whose rule set applies |
| `stageArgument` | Run → command | `stage : String` | Stage identifier of the visit being judged |
| `revisionArgument` | Run → command | `artifactRevision : Integer` | Content revision being judged |
| `ruleReport` | Command → run | `report : String` | Per-rule result report and its retained evidence URI |
| `reportRevision` | Command → run | `reportedRevision : Integer` | Revision the report actually assessed; must equal the request |
| `ruleIdentifiers` | Command → run | `reportedRuleIds : String [1..*]` | One result per declared rule, with no missing, extra, or duplicate identifier |
| `verdict` | Command → run | `holds : Boolean` | Aggregate fixed-check verdict, equal to the conjunction of the rule verdicts |

| Operational contract field | Current state |
| --- | --- |
| Provider and consumer | `SimpleFlowRun` invokes; `CheckerCommand` returns report and verdict |
| Protocol, transport, payload layout | Logical fields above are modeled; the first SimpleFlow slice implements a local Python command with a JSON report, without freezing a remote protocol |
| Direction and frequency | Request and response directions above; invocation rate open |
| Timeout, retry, idempotency | Open |
| Version and compatibility | Open |
| Security and criticality | Command authority and rating open |
| Error codes and caller action | In the local slice, process or input failure records `Unevaluated`; a trusted report with a false rule records `Failed`. Remote error codes remain open. |

The caller rejects a report whose revision differs from `artifactRevision`, whose rule
identifiers differ from the named checker's declarations, or whose aggregate verdict
contradicts its rule verdicts. A failed fixed rule is a failed result; a missing input,
command failure, or malformed report is unevaluated and cannot authorize advancement.
The initial implementation transports the run snapshot as a local JSON file. That
snapshot is an execution artifact alongside the independent wish SysML trace; it does
not replace the trace model. A durable protocol, timeout, retry, idempotency, and
owner-approved baseline remain open. The separate `tools/check_documentation.py`
audit checks document locators and does not by itself implement this flow gate
([R03](../records/status.md#r03)).

<a id="i05"></a>

## 5. IF-004 — semantic judgment request and response

`JudgmentInterface` connects `SimpleFlowRun.judgment : ~JudgmentExchange` to
`Judge.judgment : JudgmentExchange`. The checkpoint's judge flags determine whether an
agent or human may return an outcome ([F02](../design/adapters.md#f02)).

| Model flow | From → to | Field and type | Meaning |
| --- | --- | --- | --- |
| `criteriaPresented` | Run → judge | `criteria : String [1..*]` | Criteria for this checkpoint |
| `artifactsPresented` | Run → judge | `artifactsUris : String [0..*]` | Artifact references to review |
| `outcomeReturned` | Judge → run | `outcome : CheckpointOutcome` | Recorded judgment outcome |
| `rationaleReturned` | Judge → run | `rationale : String` | Explanation accompanying the outcome |

| Operational contract field | Current state |
| --- | --- |
| Provider and consumer | `SimpleFlowRun` presents; permitted `Judge` returns outcome and rationale |
| Protocol, transport, payload layout | Open; modeled fields above, presentation schema open |
| Direction and frequency | Request and response directions above; review cadence open |
| Timeout, retry, idempotency | Open |
| Version and compatibility | Open |
| Security and criticality | Identity proof, authority enforcement, and rating open |
| Error codes and caller action | Open; no automated judgment endpoint exists |

The model does not prescribe a user interface, transport, identity proof, response
deadline, retry behavior, or URI resolution policy. Assessment recording and revision
binding are described in [E04](../design/evidence-and-revisions.md#e04); the interface
alone does not prove that a permitted judge considered the evidence.

<a id="i06"></a>

## 6. Change and compatibility management

An interface change must identify the affected model interface and fields, its provider
and consumers, the design sections above, and any requirement or test impact. Review the
change with both sides before changing the SysML definition, dependent design text, and
verification cases. Record the revision and any incompatible consumer migration in this
document. No compatibility window or version scheme has been selected, so a change cannot
yet be classified as safe for deployed consumers.

Until a baseline review explicitly accepts the interface set, these are draft contracts.
After a baseline is accepted, a breaking field or direction change requires a new
controlled revision and an explicit migration decision. No such review is recorded here.

<a id="i07"></a>

## 7. Traceability and verification

| Interface | Source definition | Design explanation | Relevant requirement or verification concern | Evidence state |
| --- | --- | --- | --- | --- |
| IF-001 | `mbse/model/workflow.sysml`: `WishExchange`, `WishInterface` | [B02](../architecture/system-composition.md#b02) | Wish registration and the one-trace-per-wish invariant in [B01](../architecture/system-composition.md#b01) | Model connection only; no interface test |
| IF-002 | `mbse/model/workflow.sysml`: `FlowExchange`, `FlowInterface` | [B02](../architecture/system-composition.md#b02) | Controller selection of the assigned flow in [F04](../design/lifecycle.md#f04) | Model connection only; no interface test |
| IF-003 | `mbse/model/detailedDesign.sysml`: `CheckCommandExchange`, `CheckCommandInterface` | [D10](../design/flows/simple-flow/views.md#d10), [D06–D07](../design/flows/simple-flow/definition.md#d06) | `checkerRulesAreImplementationReady` and syntactic gate execution in [R03](../records/status.md#r03) | Local command and contract tests executed for SimpleFlow; [P08](../records/executions/simple-flow.md#p08) |
| IF-004 | `mbse/model/detailedDesign.sysml`: `JudgmentExchange`, `JudgmentInterface` | [D10](../design/flows/simple-flow/views.md#d10), [E04](../design/evidence-and-revisions.md#e04) | Judge authority and rationale in [R01](../requirements/requirements.md#r01) | Local supplied-judge boundary and authority tests executed; identity proof and independent engineering judgment remain open |

These are design-context links. The project model does not declare a dedicated
requirement-to-interface `satisfy` relationship for each row. Verification must exercise
both successful and failed exchanges, field validation, revision binding, and both
endpoints before an implementation can claim interface conformance.

<a id="i08"></a>

## 8. Current review position

The four existing draft interface definitions, their port types, directions, and selected
flows can be inspected in the bundled SysML source and in
[B02](../architecture/system-composition.md#b02) and
[D10](../design/flows/simple-flow/views.md#d10). Their operational contracts
have unresolved fields and no frozen ICD baseline. IF-003 and IF-004 now have
local command and supplied-judgment boundaries with targeted tests; IF-001 and
IF-002 remain modeled only. There is no remote compatibility result. A compiler pass on the
model establishes supported syntax and name resolution, not message delivery or
compatibility ([T04](../architecture/constraints.md#t04)).

<a id="i09"></a>

## 9. Open fields and waivers

| Open field | Interfaces | Impact | Resolution needed |
| --- | --- | --- | --- |
| Remote transport, serialization, deployment boundary | IF-001–IF-004 | Cannot implement or test wire compatibility | Select at implementation or architecture review; IF-003 has only a local JSON command exchange |
| Timeout, retry, idempotency, remote error codes | IF-001–IF-004 | Remote failure behavior is undefined | Define with provider and consumer; local IF-003 distinguishes failed rules from input/command errors |
| Version scheme and compatibility window | IF-001–IF-004 | A breaking change has no migration rule | Decide before a frozen baseline |
| Identity and access control | IF-001, IF-004; assess IF-002–IF-003 | Registration and judgment authority are not enforced by this model | Security design and verification |
| Versioned stage encoding and per-rule remote schema | IF-003 | The local command schema is implemented but has no frozen remote compatibility contract | Baseline the [D07](../design/flows/simple-flow/definition.md#d07) fields and migration rules before a remote endpoint |
| Contract owner and review authority | IF-001–IF-004 | No accountable approval or controlled baseline revision | Assign named owners and approvers before baseline review |
| Criticality, quantitative budgets, test cases | IF-001–IF-004 | PDR/CDR evidence is incomplete | Allocate owners and verification methods |

No waiver has been approved. These rows are design gaps, not implicit interface values.
