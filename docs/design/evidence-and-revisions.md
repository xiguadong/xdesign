# Evidence and revisions



The evidence and revision semantics shared by all flows. The record data contract and its field-level rules are in interfaces/record-format.md; this page explains why revisions, assessments, artifact identity, and retained evidence behave as they do.



<a id="e02"></a>

## 1. Activities and stage visits

- An activity records work, the actor, the time and the artifacts it touched. Artifact
  references are URIs to the real files — model sources, documents, logs, evidence.
- A `FlowExecution` records one `StageRecord` per stage of its assigned flow and no
  others (`recordsEveryFlowStage`, `recordsUseAssignedFlow`). `currentRecord` is the visit
  being worked on.
- A stage visit keeps its activities and assessments in execution order (`ordered`).
  `currentAssessments` is the latest assessment per checkpoint, not the list of all
  attempts (`selectsLatestAssessments`).
- `StageStatus` has the literals `pending`, `active`, `blocked`, and `completed` for
  one visit. A flow execution can have a different `FlowStatus` while a visit is
  being prepared or recovered.

<a id="e03"></a>

## 2. Revisions and re-assessment

The revision is what keeps evidence honest:

- `artifactRevision` starts at 0 for a visit and increments when the visit's artifacts
  change. Old passes do not carry over.
- A stage's checkpoints count as satisfied only when each exit checkpoint has a current
  assessment with `outcome = passed` and `evaluatedRevision == artifactRevision`
  (`StageRecord::checkpointsSatisfied`).
- A completed stage must stay authorized (`completedStageIsAuthorized`), so new evidence
  is recorded as a further assessment; an existing passed assessment is never edited.
- Corrective work on a blocked SimpleFlow visit stays in that visit and increments
  its artifact revision when content changes. Explicitly revisiting a completed
  terminal stage appends the next visit (`visitIndex` + 1); earlier visits and
  their evidence stay. The generic model does not yet define downstream
  invalidation for reopening an earlier completed stage.

Worked example, wish-0002: verification was assessed three times — revision 1 failed (the
export showed only one of two permitted judges), revision 2 passed after one flag per
authority replaced the list, revision 3 passed again after another wish moved that wish's
requirements into the model. See the journey in [E06](../records/executions/wish-journeys.md#e06).

<a id="e04"></a>

## 3. Assessments and evidence

An assessment is the only thing that authorizes progress, and it carries its reasons:

- `outcome` starts `unevaluated`; criteria that were not evaluated provide no
  authorization to advance.
- `passed` requires evidence: `passedAssessmentHasEvidence` refuses a pass with no
  `evidenceUris`.
- `judgedBy` must match the checkpoint's kind: a script may only judge a syntactic
  checkpoint, an agent or a human only a semantic one
  (`judgmentMatchesCheckpointKind`, see [F02](adapters.md#f02)).
- `evidenceUris` point at retained files — logs, exports, rendered figures — so a
  later reader can re-examine the judgment, not just its conclusion. Legacy examples
  used ignored `cache/` alone. New store-backed revisions also bind each current
  artifact and evidence URI to archived SHA-256 bytes, while checker record inputs and
  reports remain separate exchange materials. Historical revisions retain their own
  manifest when a URI later names different bytes.

The [storage design](storage.md#d86) defines publication, revision and recovery. A stored
manifest does not rewrite the run record's existing URI fields or turn an unchecked claim
into a pass; the fixed record checker and semantic judgment still evaluate their stated
snapshot. Checkout supplies the archived bytes for independent re-examination.

<a id="e05"></a>

## 4. Delivery invariants

A delivered wish has come all the way through, and the model says so in four places:

| Invariant | Where |
| --- | --- |
| Every wish has exactly one trace, and every trace a wish | `XDesign::everyWishHasOneTrace` |
| Every stage of the assigned flow has a record, and no other stage does | `FlowExecution::recordsEveryFlowStage`, `recordsUseAssignedFlow` |
| A completed flow has an authorized terminal stage | `FlowExecution::completedFlowHasAuthorizedTerminalStage` |
| Only a completed root flow delivers the wish | `Trace::deliveredWishHasCompletedRootFlow` |

The compiler does not evaluate these constraint bodies ([T04](../architecture/constraints.md#t04)),
so they are obligations verified by reading; each wish's verification activity says how.
For runs on the designed simple flow, the declared `simpleRecordCheck` gate checks
its D06 rule set: at least one visit per stage, authorization of completed visits
at their recorded revision and artifact bytes, evidence that exists, and the run
typed by the design's class. It does not establish every generic delivery
invariant above or validate all historical wish SysML records
([D06](flows/simple-flow/definition.md#d06), [D07](flows/simple-flow/definition.md#d07)).
Its command is `tools/checks/simple_record.py`; executed results and limits are
recorded in the [SimpleFlow implementation plan](../records/plans/simple-flow.md#p06).
