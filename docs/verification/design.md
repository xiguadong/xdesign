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

<a id="d95"></a>

## 3. Configurable check-binding verification cases

The configurable SimpleFlow check bindings ([D88](../design/flows/simple-flow/definition.md#d88))
are selected at registration through the strict schema and validation in
[the stage CLI](../design/stage-cli.md#d89) and are frozen into the plan. They preserve the
existing syntactic-check requirements — the gate's criteria and declared rules stay fixed
(`syntacticCheckIsFixed`, `checkerRulesAreDeclared`, `gateSyntaxIsRunnable`) and the run
record stays authoritative (`stageCliPreservesRunIntegrity`) in
[R01](../requirements/requirements.md#r01) — while a `cli` binding records the explicit
agent-authority extension described in [D88](../design/flows/simple-flow/definition.md#d88).
The requirement inventory has no separate row for configurable bindings yet, so the design
section is the primary trace for the extension itself. The branch did not extend the
model's verification case definitions, so these cases are branch verification design
rather than members of the `XDesignVerification` cases in [§1](#d54). They state obligations to apply when the feature is exercised; they do not
extend the executed acceptance of the Rust slices, and they are not executed results.
Executed outcomes and retained command evidence belong to the branch's execution record.

| Case | Input or operation | Expected result |
| --- | --- | --- |
| CUSTOM-01 Default plan unchanged | Register a wish with no `--checks` option and register a second wish with a default-only binding file; compare each frozen plan and `plan_sha256` with the built-in plan, and evaluate the four syntactic gates. | Both runs carry the built-in plan bytes; no custom runner is serialized; the default assessor is still the checker id with script authority; the two frozen hashes agree and `flow-is-supported` holds for both runs. |
| CUSTOM-02 CLI binding end to end with a stub agent | Bind one checkpoint to `mode: "cli"` for `codex` and a named model, point `XDESIGN_AGENT_CODEX_BIN` at a stub executable that returns one rule verdict per declared rule, then do the stage work and call `check`; repeat with a stub that returns a rule-set mismatch. | The frozen plan names the `cli` runner, CLI, and model; the stub is invoked non-interactively with the checkpoint's declared rule ids and the reviewed snapshot; a valid verdict creates a passed or failed assessment with its retained report and `.stdout.log`/`.stderr.log`; a mismatched rule set or malformed answer creates no passed assessment and leaves the gate blocking. No real agent is invoked in the case. |
| CUSTOM-03 Agent authority, assessor, and restore | Inspect the assessment from CUSTOM-02; restore the run in a fresh process; run the record checker and the web detail query; then edit the original `--checks` file and re-query. | The assessment records `judged_by = agent` and the assessor `<cli>:<model>`; restore accepts an agent-authority assessment on a syntactic checkpoint only because the frozen plan names the matching `cli` binding, and keeps the completed visit authorized with its evidence; the store or web query reports the same binding; the edited file changes nothing for the registered run. |
| CUSTOM-04 Invalid bindings rejected | Present a malformed JSON file, an unknown field, a wrong `schema_version`, an unknown or duplicate checkpoint id, an unknown mode, a `cli` mode without `cli`/`model`, and a `command` mode without `command_uri`; then register with a partial default file. | `init` exits 2 with the named validation error before creating the wish item or any store revision; the store head and cache are unchanged; the partial file succeeds and every omitted checkpoint is `default`. |
| CUSTOM-05 WishVerifyFlow rejects non-default bindings | Register `--flow wish-verify` with a non-default binding, and separately with a default-only file. | The non-default binding is rejected as an invalid request before registration because the flow declares only semantic checkpoints; the default-only registration succeeds and leaves its two semantic judgments supplied through the result exchange. |
| CUSTOM-06 Command binding, timeout, and read-only safety | Bind a checkpoint to `mode: "command"` with an existing project-relative `.py` stub implementing the checker invocation and report interface; also present a missing, non-normalized, or non-`.py` `command_uri`; repeat with a stub that hangs past `XDESIGN_CHECKER_TIMEOUT_SECS` and with a stub that returns a malformed report. | The invalid `command_uri` form is rejected at registration before any run exists; the valid command runs under the deadline, reports the declared rule ids, and is recorded with script authority and the checker id as assessor; the hanging process tree is killed, an explicit timeout is recorded, partial logs are retained, and the gate is unevaluated and blocks the visit; the malformed report creates no passed assessment. The bound command cannot alter the plan, the criteria, or the assessed revision. |

The authority oracle for CUSTOM-02, CUSTOM-03, and CUSTOM-06 is the assessment record and
its retained report ([E04](../design/evidence-and-revisions.md#e04)); the report validation
requirements are those of the checker command contract ([D07](../design/flows/simple-flow/definition.md#d07)).
The default-behaviour guard for CUSTOM-01 and CUSTOM-05 is the unchanged serialization and
the existing default suites; the binding schema and exit codes are those of the
[stage CLI](../design/stage-cli.md#d89). The cases fix observable authority, evidence, and
rejection behavior rather than message wording: the exact validation text an invalid
binding prints is an implementation detail, so a case asserts failure and the absence of a
committed run rather than a specific message. The agent prompt contents are likewise
observable only through the stub the case supplies; the case requires the declared rule ids
and the reviewed snapshot binding, not a byte-for-byte prompt.

<a id="d96"></a>

## 4. Operator web program verification cases

The operator web program ([D91](../design/web-operator.md#d91)) is not modeled in the
SysML verification model either; its design claim and the
[operator web API](../interfaces/web-api.md#i16) are the trace basis. The web actions must
reuse the stage interaction the requirement inventory already states
(`stageCliUsesUniformInteraction`, `stageCliPreservesRunIntegrity` in
[R01](../requirements/requirements.md#r01)), and the architecture records that the
complete workflow application is not implemented ([B05](../architecture/system-context.md#b05));
the requirement inventory has no separate operator-web row. These cases are verification
design, not executed acceptance: a served page, a passing API smoke test, or a screenshot
is evidence for a case, not an engineering acceptance, and executed results belong to the
branch's execution record. Run them from a disposable root under `cache/`; never write
inside the repository's own `.xdesign` store.

| Case | Input or operation | Expected result |
| --- | --- | --- |
| WEB-01 Embedded assets without an extra runtime | Build the binary, copy it outside the source checkout, and run `xdesign web --port 0 --project-root TMP` with no Node.js, Python web server, static server, or rendered documentation site available; request `/`, the embedded assets, an unknown extensionless path, and an unknown asset-like path. | The startup line names the actual ephemeral address; the page and every embedded asset are served from the binary with generated content types; the extensionless path falls back to `index.html` and the asset-like path is `404`; no `web/assets/` or `cache/docs-site/` file is read at runtime, and `SIGINT` stops the process with default behavior. |
| WEB-02 API contract responses | Exercise `GET /api/health`, `GET /api/flows`, `GET /api/wishes`, a wish detail, work, and material against a prepared temporary store; then send a wrong method, an unknown `/api/` route, an invalid `wish_id` path segment and an invalid create body, an oversized body, and a missing artifact URI. | Responses use the documented success/failure envelopes and the documented 200/400/404/405/409/413 codes (an invalid path segment is `404 not_found`, an invalid create field is `400`); JSON headers carry the JSON content type, `no-store`, `nosniff`, and an explicit `Content-Length`; the unknown `/api/` path returns JSON `404`; the invalid create or check request writes nothing. |
| WEB-03 Lifecycle actions against a temporary store | From the page or its API, create a SimpleFlow wish with mixed `default` and `cli` bindings, get next work, submit a check whose semantic judgment is filled from the work snapshot digest, repeat a duplicate create, and check a completed run. | Creation runs the CLI's `init` and commits one registration; the detail view reports the frozen binding per checkpoint; `next` returns work without changing the stored head; the check records outputs and assessments through the CLI's `check` and returns its exit-code semantics (`exit_code: 1` for a blocked stage as a successful envelope); a duplicate create is `409 already_exists` and a completed check is `409 already_delivered`; all writes land in the temporary project and store roots, not the repository's `.xdesign`. |
| WEB-04 Read/write boundary and isolation | Compare the records created through the web actions with the same commands run directly in the CLI against an independent temporary root; inspect the project tree and the store after the actions; request a material URI outside the frozen revision and an arbitrary absolute path. | The plan, run records, assessments, and evidence agree with the CLI's records; the server's own writes stay in `cache/web/`; no Git tree, documentation file, or source file is written; materials serve only frozen head-revision bytes or the documented legacy allowlist and never an arbitrary path. |
| WEB-05 Independence from the documentation site | Run the server in a checkout where the documentation site was never built and where MkDocs and its virtual environment are absent; load the page and exercise the read endpoints; inspect the page's asset origins and the server's behavior. | The page loads and every endpoint answers without the docs site, its build output, or its interface; page assets are local and the page makes no external request; the docs tree is neither read nor modified, and no documentation build artifact is created. |

The observable contract for WEB-02 through WEB-04 is the [operator web API](../interfaces/web-api.md#i17);
the read/write boundary and release claims are [D92 to D94](../design/web-operator.md#d92);
the lifecycle semantics reused by WEB-03 are the [stage CLI's](../design/stage-cli.md#d85).
The screen-level page behavior follows the branch's working specification in
`web/SPEC.md`; a visual screenshot establishes only what it shows and is not acceptance.
