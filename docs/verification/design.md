# Verification case design

<a id="d54"></a>

## 1. Verification case contract

`XDesignVerification::SimpleFlowGateBehaviorVerification`,
`SimpleFlowRecoveryVerification`, and `SimpleFlowFilePlanVerification` are case
definitions in `mbse/model/verification.sysml`. Their objectives trace to the named
project requirements. The following inputs and expected results make those definitions
executable; actual case results are recorded in
[P08](../records/executions/simple-flow.md#p08).

| Case | Input or operation | Expected result |
| --- | --- | --- |
| SF-01 Plan rejection | Remove an exit gate, duplicate a checkpoint ID, point an exit to another stage, remove a nonterminal successor, or remove the terminal stage. | `validate` rejects every malformed plan before creating a run; an empty gate set cannot authorize a designed stage. |
| SF-02 Initial visit | Create a valid four-stage run. | Four planned stage visits in stage order, each at `visitIndex = 0` and `artifactRevision = 0`; only the requirements visit is active. |
| SF-03 Stage scope | Evaluate the requirements, design, and implementation visits in turn. | Each receives only its own syntactic gate assessment; no other stage's gate is invoked. |
| SF-04 Mixed verify gates | Evaluate the verify visit. | The record gate and semantic gate each produce one assessment in plan order; both are required to complete the run. |
| SF-05 Report and authority failures | Return a missing, duplicate, extra, or contradictory rule report; try a judge whose authority the gate forbids. | No malformed report or unpermitted judge yields a passed assessment; a fixed rule failure is `Failed`, while a channel unable to conclude is `Unevaluated`. |
| SF-06 Latest verdict | At one artifact revision, append pass then fail, pass then unevaluated, and fail then pass for the same gate. | The latest assessment for that gate controls authorization; only the last sequence can authorize it, if all other gates pass. |
| SF-07 Content revision | Record a content change to an artifact, including a change at an unchanged URI. | The current visit's revision increments once for the content event and its artifact-set digest changes; old assessments remain retained but cannot authorize the new revision. |
| SF-08 Failed advancement | Call `advance` with a missing, failed, stale, or unevaluated current gate. Correct the artifact and reassess. | The same visit remains current and blocked until corrected; no new visit is appended merely because a gate failed. |
| SF-09 Terminal reopen | Complete and explicitly reopen the terminal stage, then attempt to reopen an active run or a nonterminal stage. | A valid reopen appends the next visit index at revision 0 and preserves the completed visit and its evidence; invalid requests are rejected. |
| SF-10 Command conformance | Run the four fixed gate commands on passing and targeted failing fixtures. | Their reports contain exactly the 18 declared rule IDs across the four rule sets, one result per rule, with a diagnostic for each failure and an aggregate verdict equal to the conjunction. |
| SF-11 File and model integration | Compare the file-plan paths in both directions against the source/test inventory, crate exports, command URIs, declared rule IDs, model links, and generated figures; run the compiler and documentation checks. | Every declared source path exists and every in-scope source/test file is declared; mappings resolve; model `parse`/`check`, documentation audit, figure render, and strict site build pass at the same source revision. |

The authorization oracle for SF-06 and SF-08 is the generic
`StageRecord.currentAssessments` latest-per-checkpoint selection and
`completionAuthorized` predicate ([E03](../design/evidence-and-revisions.md#e03),
[D16](../design/flows/simple-flow/views.md#d16)). SF-07 treats revision as a recorded content event,
not a difference between URI lists. SF-08 follows the controller's same-visit correction
path ([F04](../design/lifecycle.md#f04)). SF-09 uses the explicit
`ReopenSimpleFlowStage` design action, currently limited to a completed terminal run;
upstream-stage reopening needs downstream invalidation rules before implementation.
