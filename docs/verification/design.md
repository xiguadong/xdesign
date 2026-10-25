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

`XDesignVerification::StageCliInteractionVerification` specifies planned coverage
for the [two-command stage interaction](../design/lifecycle.md#f04). These CLI cases
have no executed result and do not extend the acceptance claims of the Rust slices.
Apply them to two different configured flows whose frozen stage definitions supply
the required work content.

| Case | Input or operation | Expected result |
| --- | --- | --- |
| CLI-01 Current work | Call `next` repeatedly, then retrieve the same wish in a fresh process. | The work order reflects the assigned stage's goal, instructions, artifacts, exit checkpoints, current assessments, and reasons. Visits, revisions, activities, and assessments are unchanged. |
| CLI-02 Check and continue | Perform workspace work, then call `check`, including a result file where declarations cannot locate the work. | Outputs are recorded before the current visit's exit checkpoints are evaluated; evidence and assessments are retained. An authorized pass returns successor work or terminal completion without another `next`; the result file never bypasses a gate. |
| CLI-03 Block and correct | Fail a gate, correct work, and call `check` again; repeat once with unchanged content. | Failure returns reasons and the same blocked visit. Only changed content increases its revision and invalidates prior passes; no corrective visit is appended. |
| CLI-04 Authorization failures | Supply missing required artifacts, unavailable checks, stale passes, or an unpermitted judge. | None authorizes progress. A valid negative evaluation is failed; inability to evaluate is unevaluated. |
| CLI-05 Completed and unknown wishes | Call both commands on a completed run and an unknown wish. | Completion produces no work or writes and does not reopen. An unknown wish produces an error without registration. |
| CLI-06 Prepared work and subflows | Omit required work content or leave a required child uninitialized; separately query an initialized incomplete child and then its completed state. | Incomplete definitions and unprepared nested paths are rejected without mutation. An initialized incomplete child supplies current work; a completed child resumes its parent. Querying never initializes a subflow. |

Registration and final human acceptance are outside this repeated interaction.
The cases require no separate public record, assessment, or advancement command;
compiler checks of their declarations do not execute them.

<a id="d87"></a>

## 2. Storage verification cases

`XDesignVerification::StorageVerification` covers the seven storage requirements. The
following cases are designed verification obligations, not executed results. The
[storage plan](../records/plans/storage.md#p39) owns observed commands and outcomes.

| Case | Input or operation | Expected result |
| --- | --- | --- |
| STORE-01 Format lifecycle | Initialize and reopen a store; then present missing, foreign, truncated and newer-version databases. | Same-format reopen is idempotent; incompatible state is refused without adoption, migration or input-byte changes. |
| STORE-02 Revision queries | Create a generic item and append revisions with distinct metadata/materials; call list/get/history/read. | Numbered immutable revisions and complete manifests are returned exactly, including a selected earlier revision. |
| STORE-03 Material identity and paths | Publish duplicate bytes, altered bytes, and unsafe path/source forms. | Identical bytes deduplicate; all reads match length and SHA-256; unsafe or mismatched input is rejected before a row becomes authoritative. |
| STORE-04 Exact-operation retry | Commit a request, reopen the store, repeat the same operation ID and bytes, then reuse the ID with different bytes. | The committed receipt is identical and no second revision appears; changed request bytes produce `operation_mismatch`. |
| STORE-05 Atomic guarded save | Start two writers at one expected head and exercise a complete stage check. | One expected-head save commits and the other receives a durable conflict; the stage's run, reports, artifacts, evidence and receipt become visible together. |
| STORE-06 Crash reconciliation | Inject failure after reservation, after blob publication, before commit, and after commit before response. | `outcome` distinguishes pending from committed; exact retry resumes or returns the original receipt without double effects or rows lacking blobs. |
| STORE-07 Integrity diagnosis | Remove or alter referenced bytes, add an orphan and pending operation, and call doctor/read/checkout. | Findings identify each defect; digest mismatches and missing bytes cannot be returned as trusted content. |
| STORE-08 Flow integration and projection | Register SimpleFlow and WishVerifyFlow runs from wish sources outside `mbse/wishes/`, then query active, blocked and completed states. | Both flows persist through the store without new Git traces; flow/current-stage/status projections match the run record and selected flow. |
| STORE-09 Cache-loss and replay | Delete generated runtime cache, call `next`, checkout a complete revision, verify all manifest digests, and invoke `simple_record.py` on the materialization. | Stored-head work/completion is recovered, every manifest byte is restored exactly, and the existing record checker passes the restored snapshot. |
| STORE-10 Regression closure | Run applicable Rust, Python, compiler, figure and documentation suites. | All changed-artifact checks pass; existing failures and unevaluated rules are reported with evidence rather than omitted or relabeled. |
| STORE-11 Review-challenge regressions | Exercise the accepted review-challenge decisions: rejected integer-overflow, decimal-drift and unsupported-extreme-exponent numbers with typed errors; NFD-plus-case-fold-equivalent manifest names and implied directories at save validation and again at checkout materialization; a symlink ancestor above an existing destination parent for both read output and checkout; an exact terminal retry after the project root or source paths are gone; source omission backed by a committed blob row versus a same-operation publication versus an unrelated orphan; bounded checkout verification with many aliases of one blob; committed stage-check replay after attempt and canonical cache deletion; and a checker descendant that inherits stdout/stderr under the deadline. | Each planned condition matches the documented contract: typed rejection without silent rounding, panic, or unrelated-blob adoption; canonical-equivalent names never commit a manifest that cannot materialize; earlier symlink ancestors are refused; terminal receipts and canonical replay survive disposable-cache loss; checkout correctness is independent of manifest size and alias count; and checker deadlines are enforced across the whole process tree with an explicit timeout result. |
