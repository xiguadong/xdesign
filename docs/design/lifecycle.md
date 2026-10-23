# Run lifecycle



The generic run lifecycle shared by all designed flows: run creation, stage visits, activity and assessment records, gate evaluation, advancement, reopening, blocking, and rework. Flow-specific run and factory classes are described in each flow's implementation page; the common record contract is in interfaces/record-format.md.



<a id="f04"></a>

## 1. The controller and how a wish advances

`XDesignWorkflow::StageCli` defines a two-command interaction for every stage:
retrieve the current work, let the agent perform it in the project workspace, and
ask xdesign to check the result. `AgentStageWork` declares one iteration and documents
its repetition until completion.
The [local CLI](stage-cli.md#d85) implements this boundary for the two built-in flat
flows. The generic controller and nested-flow integration remain separate obligations.

The [simplest WishVerifyFlow timeline and swimlane](flows/wish-verify/views.md#d84)
apply this interaction to `wish` → `verify` → completed. The
[package view](../architecture/system-composition.md#b01) locates the CLI boundary
and its controller binding in the system model.

| Public operation | Result and state effect |
| --- | --- |
| `xdesign next WISH_ID` | Returns the registered wish's current `StageWorkOrder`, or completion with no work. It does not record, assess, change revisions, advance, initialize a subflow, or reopen a visit. |
| `xdesign check WISH_ID [--result FILE]` | Collects and records the current stage's outputs, evaluates its exit checkpoints, and advances when authorized. Returns the assessed visit's assessments and either the same blocked visit with correction reasons, the successor's work order, or completion. |

A `StageWorkOrder` is a read-only projection of the current visit, not another
editable execution record. It identifies the wish, execution, visit, stage, and
artifact revision, and supplies the goal, instructions, declared inputs and outputs,
exit checkpoint criteria and permitted checkers or judges, current assessments,
and correction reasons. Its work content comes from the assigned frozen flow plan
([stage content](flow.md#f01)). The agent edits workspace artifacts; xdesign owns
the activity, assessment, and execution records.

`check` collects the outputs named by that plan. The optional result file supplies
work or evidence that those declarations cannot locate; it cannot bypass a gate.
Recording precedes assessment and preserves the [revision and evidence
rules](evidence-and-revisions.md#e03): changed content raises the revision and
invalidates old passes, while unchanged content does not raise it. Fixed checkers
and permitted semantic judges retain their [existing authority](adapters.md#f02).
A valid negative evaluation is failed; a missing or refused evaluation remains
unevaluated. Evidence and assessments are retained before the response is returned.
The agent can continue directly from the returned work order without another
`next` call. `next` also retrieves the work after a process restart.

Both commands resolve an existing wish through the controller's traces. An unknown
wish is an error, not registration. Both return completion without writing or
reopening when the root run is complete. They reject incomplete work definitions or
an uninitialized required subflow with a diagnostic and no mutation. Selection
follows the deepest initialized incomplete child execution; a completed child
returns selection to its parent. Registration and progression own entry preparation;
the read-only query never initializes subflows. Integration for unprepared nested
flows remains unavailable.

Wish registration and final human acceptance remain boundary interactions outside
this repeated loop. The local slice provides explicit `init` registration and leaves the separate
acceptance-state implementation open. Its planned verification is
[StageCliInteractionVerification](../verification/design.md#d54).

The controller operations below support the CLI internally; they are not additional
public commands in the stage loop.

![The controller's actions](assets/controller.svg)

> **Figure F04-1 — Controller operations.**  
> **Question:** Which operations register, record, and advance a wish?  
> **Model view:** Internal-structure projection of `XDesignArchitecture::Controller`.  
> **Companions:** [A03](../architecture/wish-lifecycle.md#a03) for lifecycle and [E02](evidence-and-revisions.md#e02) for visit records.  
> **Status:** renderable-now.  
> **Reading:** The controller owns operations for registration, activity recording, and checkpoint-driven progress. The table below supplies each operation's effect on the trace.  
> **Limits:** The figure names operations; it does not execute a checker or judge semantic criteria.

| Action | What it does |
| --- | --- |
| `receiveWish` | Registers the wish and a separate trace with an independent model reference; assigns `selectedFlow` to the trace's root execution; creates one record per stage of that flow; enters the initial stage; activates the trace and execution |
| `recordActivity` | Follows the trace's active execution path to the deepest current visit and appends the activity there. If its artifacts change, it increments that visit's revision before further assessment |
| `progressWish` | `selectCurrentExecution` (descend into an open subflow) → `evaluateCheckpoints` (assess each exit checkpoint against current artifacts, keep the latest per checkpoint) → if `completionAuthorized`: `decideNextStep` and `updateExecutionState`; otherwise `retainCurrentStage` → `updateTraceStatus` |

The existing `StageWork` action remains the historical lifecycle scenario's
orchestration of work within one stage. It takes the controller, wish trace,
and stage as inputs; `produceArtifacts` creates an `ActivityRecord`, then calls the
controller's `recordActivity` and `progressWish` in order. The controller still owns
checkpoint assessment and trace-status updates. This is the modeled lifecycle action,
not a claim that the generic controller runs in the current crate.

<a id="f05"></a>

## 2. Rework, blocking and subflows

- **Blocked is resumable.** A visit that does not pass stays the current record, marked
  blocked, with all prior activities retained. Corrective work appends activities and
  raises the revision, and the next assessment judges the current revision
  (`Workflow::executeWish`, the `blocked` → `correctiveWork` → `verificationReady` path).
- **Rework appends a visit.** A new attempt at a stage is a new `StageRecord` with the
  next `visitIndex`; earlier visits and their evidence are kept
  ([E03](evidence-and-revisions.md#e03)).
- **A subflow must finish first.** If a stage declares a `subFlow`
  (`XDesignComposition::xdesign.nestedFlowExample` shows one), the parent stage cannot complete
  until that execution is completed (`StageRecord::subFlowSatisfied`).
- **The terminal stage decides.** Only a stage with `completesFlow` leaves the flow with no
  successor, and its exit checkpoint is what authorizes delivery
  ([B03](../architecture/system-composition.md#b03)).
