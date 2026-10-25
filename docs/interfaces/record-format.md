# Record format

<a id="e01"></a>

## 1. The record vocabulary

![The record definitions, their fields and their rules](assets/records.svg)

> **Figure E01-1 — Record vocabulary.**  
> **Question:** Which types hold a wish's execution history and evidence?  
> **Model view:** `XDesignArchitecture::recordStructure`, a browser view of the record definitions.  
> **Companions:** [E03](../design/evidence-and-revisions.md#e03) for revision rules and [F01](../design/flow.md#f01) for flow definitions.  
> **Status:** renderable-now.  
> **Reading:** The structure separates a wish and its trace from the flow, stage, activity, and assessment records. Each definition in the figure has its role summarized in the table below.  
> **Limits:** The figure shows record types, not the history or validity of any one wish.

| Definition | Role |
| --- | --- |
| `Wish` | The requested outcome: `wishId`, `description`. Carries no work history |
| `Trace` | One wish's state: `wish`, `status`, `traceabilityModelUri`, and its root `execution`. Declares the `delivered` event |
| `FlowExecution` | The execution history of one flow — the wish's root flow, or a subflow inside a stage visit: `assignedFlow`, `status`, `stageRecords [1..*]`, `currentRecord` |
| `StageRecord` | One planned or actual visit to one stage: `visitIndex`, `status`, `artifactRevision`, `subFlowExecution`, `activities`, `assessments` |
| `ActivityRecord` | One piece of work: `description`, `performedBy`, `occurredAt` (ISO 8601), `artifactUris` |
| `CheckpointAssessment` | One preserved evaluation of one checkpoint for one revision: `outcome`, `judgedBy`, `assessor`, `rationale`, `evidenceUris` |

`Trace` and its records are the working record of one wish. Definitions stay in
`mbse/model/`. A legacy/bootstrap wish SysML file declares usages of them; a store-backed
run retains their instance data in the revisioned run record instead. The wish source itself
is an imported SysML material with a regenerable working copy, not a second Git authority
([A02](../architecture/index.md#a02), [store format](store-format.md#i11)).
`FlowStatus` has the literals `pending`, `active`, `blocked`, and `completed` for
the execution as a whole. It is separate from each visit's `StageStatus`. Legacy traces
retain an explicit `WishStatus`; store queries project wish status from `assignedFlow`,
`current_visit`, and the run's status rather than storing another editable flag.
