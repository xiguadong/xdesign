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
[the stage CLI](../design/stage-cli.md#d89) and are frozen into the plan. The resolved
rules and executor selection are retained in the frozen plan, while the
run record retains assessments and evidence. A `cli` binding records the explicit
agent-authority extension described in [D88](../design/flows/simple-flow/definition.md#d88).
The current [requirement allocation](../requirements/requirements.md#r16) and
`CheckpointOperatorDesignReview` now trace this extension. The original delivery did not
include that model allocation. A CLI agent does not satisfy the generic scripted-only
requirement or assessment constraint; the [authority deviation](../design/checkpoint-library.md#d107)
remains explicit. These cases state obligations to apply when the feature is exercised; they do not
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

<a id="d103"></a>

## 5. Checkpoint library and library-binding verification cases

The round-2 checkpoint library ([D97](../design/checkpoint-library.md#d97)) is
Git-tracked authoring source: reusable syntactic rule sets with their executors and
semantic criterion sets with their judges, versioned in `checkpoints/library.json`. Wish
initialization may bind those definitions to the four syntactic SimpleFlow gates and the
semantic `verificationAccepted` gate through the `library` mode of the binding file
([D101](../design/checkpoint-library.md#d101), [D88](../design/flows/simple-flow/definition.md#d88)).
The command surface is in the [stage CLI](../design/stage-cli.md#d85) and the library
itself remains authoring source while `.xdesign` remains the operational record. The cases
below are verification design, not executed results: each uses a temporary project root and
stub executables, never a real agent, and executed outcomes belong to the branch's execution
record. The [current requirements](../requirements/requirements.md#r16) and
`CheckpointOperatorDesignReview` allocate the source, authority and operator obligations.
The CLI authority deviation and incomplete executable/provenance freezing remain explicit
in the [as-built project boundary](../design/checkpoint-library.md#d107).

| Case | Input or operation | Expected result |
| --- | --- | --- |
| LIB-01 Library load, missing file, and validation | With no `checkpoints/library.json`, list the library; then present malformed JSON, an unknown field, an unknown kind, a syntactic entry without rules/inputs/executor, a semantic entry with rules or without a true judge, a duplicate id, and a non-slug id. | The missing file is not an error and the effective library is the five built-ins; every malformed or invalid entry is an explicit invalid-request error (exit 2) that names the problem, and the runtime never guesses a kind or drops an entry silently. |
| LIB-02 Built-in entries and immutability | List the library and show each built-in; attempt to save or overwrite each built-in id; duplicate a built-in into a new user id. | The listing contains the four syntactic built-ins and the semantic built-in with `builtin: true`, their rules or criteria copied from the default plan; mutation of a built-in id is rejected without changing the file; duplicating into a new user id succeeds and is version 1. |
| LIB-03 Versions and history | Save a new user entry, then update it twice; inspect the saved entry and its history. | The first save is version 1 with empty history; each update increments the version and pushes the previous snapshot to the front of a history of at most ten items with the documented fields; built-ins are not versioned in the library file. |
| LIB-04 Script reuse and scaffold generation | Save a syntactic script entry with `create_script: "scaffold"` at the required `checkpoints/scripts/` URI; run `--list-inputs` and the full invocation against it; save another entry reusing an existing script without `create_script`. | The scaffold is deterministic and self-contained: it lists the declared inputs, verifies the plan/record/visit binding, writes the raw report with `input_sha256` covering the wish model, record snapshot, plan, declared inputs, and its own bytes, and returns a failed `rule not implemented` verdict for each declared rule; the reused script path succeeds with no generation and the entry's source records `generated` versus `manual`. |
| LIB-05 `checkpoint list`/`show`/`save` round trip | Run the three commands against a temporary library; inspect replies and exit codes; repeat with an unknown id, an invalid save request, a built-in id, and an unreadable library. | Each command prints one JSON object on stdout; list and show never execute a checker; show returns the full entry with `builtin` and `script_exists`; save reports the entry, version, `created`, and `script_uri`; invalid requests exit 2 and internal failures exit 1 without a partial library file. |
| LIB-06 Syntactic library binding end to end | Create a syntactic library entry with one stub script; register a SimpleFlow run with a `library` binding for one syntactic gate, then run `next` and `check`. | The frozen plan carries `checkpoint.library`, the library's rules, the frozen checker id (the entry's optional `checker_id`, otherwise the library id), and the entry's command URI; the work definition freezes the entry's `inputs` instead of calling `--list-inputs`; `check` executes the stub, creates an assessment with script authority and the frozen checker id as assessor, and retains the report; `next` reloads the same frozen plan. Reusing a shipped `tools/checks/*.py` script whose hardcoded `--checker-id` differs from the library id requires the entry's `checker_id`, and the frozen plan then records that identity. |
| LIB-07 Semantic library binding | Create a semantic library entry with criteria and judges; register a SimpleFlow run with a `library` binding for `verificationAccepted`; submit a permitted judgment, and separately omit the judgment. | The frozen plan carries `checkpoint.library`, the entry's criteria, and its judge permissions, with no checker; a permitted agent or human judgment over the frozen criteria creates the assessment; an omitted judgment records an unevaluated semantic assessment and does not authorize completion; a script authority judgment remains rejected. |
| LIB-08 Kind mismatch and unsupported library bindings rejected | Bind a semantic entry to a syntactic gate, a syntactic entry to `verificationAccepted`, an unknown library id, a library binding while the library file is unreadable, and a library binding for `wish-verify`. | Every case is an initialization error (exit 2) before a wish item or store revision exists; the store head is unchanged; a default-only file for `wish-verify` remains a no-op. |
| LIB-09 Default plan bytes unchanged | Register one run with no `--checks` and one with a default-only file; compare each frozen plan byte-for-byte with the built-in plan and compare their hashes. | Both runs freeze the unchanged built-in plan (no library or custom fields) and pass the existing record checker; library mode adds frozen fields only to a run that chose it. |
| LIB-10 Frozen-plan durability across library edits | After a library-bound run is initialized, edit the bound entry's rules and executor, rename it, then delete `checkpoints/library.json`; run `next`, `check`, restore, and the record checker; inspect the operator detail. | The run keeps using its frozen plan fields in all operations and the completed visit stays authorized with its retained evidence; the detail shows the frozen `library_id` with a warning when the current library no longer resolves it; a new registration sees the edited library. |

The frozen-plan oracle for LIB-06 to LIB-10 is the binding contract in
[D88](../design/flows/simple-flow/definition.md#d88) and [D101](../design/checkpoint-library.md#d101);
the report and evidence obligations are those of the checker command contract
([D07](../design/flows/simple-flow/definition.md#d07)) and the assessment record
([E04](../design/evidence-and-revisions.md#e04)); the authoring-side model is
[D98](../design/checkpoint-library.md#d98) and [D100](../design/checkpoint-library.md#d100).
A validated library entry, a generated scaffold, or a green round-trip is not an engineering
acceptance; the executed outcomes belong to the branch's execution record.

<a id="d104"></a>

## 6. Round-2 operator UI and start-work handoff verification cases

The round-2 operator surfaces add the checkpoint library API ([I19](../interfaces/web-api.md#i19)),
library bindings in the wish detail and wizard, the unambiguous check-executor wording
([D105](../design/web-operator.md#d105)), and the explicit start-work handoff
([D106](../design/web-operator.md#d106)). These cases are verification design, not executed
acceptance: a screenshot, a passing curl, or a copied prompt establishes only what it shows,
and executed results belong to the branch's execution record. Run them against a disposable
temporary project root and never write inside the repository's own store.

| Case | Input or operation | Expected result |
| --- | --- | --- |
| WEB2-01 Checkpoint HTTP endpoints | Exercise `GET /api/checkpoints`, `GET /api/checkpoints/{id}`, and `POST /api/checkpoints` against a temporary library; then send a malformed definition, a built-in mutation, an unknown id, a bad slug, and a failing save request. | List and show pass the CLI reply through as `data`; save returns the save reply with `checkpoint`, `version`, `created`, and `script_uri`; the documented errors map to `400 invalid_checkpoint`, `409 builtin_checkpoint`, `404`, and `500 checkpoint_failed` with `error.detail`; envelopes, headers, and body limits match the interface page. |
| WEB2-02 Wish detail library join and warnings | Detail a wish whose frozen plan binds a library entry; then remove the entry from the current library, rename the library, and make the library unreadable. | The frozen plan fields are authoritative: `library_id` and `binding.mode = "library"` come from the plan; the display `library` summary resolves `{checkpoint_id, title, kind, version, builtin}` while the entry exists; when it cannot be resolved the summary is `null`, a `warnings` array names the frozen id, and the detail request still succeeds. |
| WEB2-03 Create-wish library bindings | Create a wish through `POST /api/wishes` with syntactic and semantic library bindings, a mixed default/library set, a kind mismatch, a missing library id, a `command` mode, and an unreadable library. | Valid library bindings are passed through unchanged and `init` commits the registration; the detail shows the frozen library plans; kind mismatch and unknown ids are `400 invalid_checks` (pre-validated from the current library, with the CLI's own rejection mapped to the same code when the library cannot be read); `command` mode is still rejected at the web boundary; default and CLI modes keep their round-1 behavior. |
| WEB2-04 Start-work handoff content and non-automation | Create a wish, inspect the run without calling `next`, then render the detail page and handoff card; copy the prompt and commands; run a successful check and a blocked check. | Creating the wish does not start any stage work or spawn a working agent: the run waits at its first stage and only `next`/`check` subprocesses occur; the handoff card explains that the flow does not auto-run, shows a copyable prompt containing the wish id, title, current stage, goal, instructions, the commands, and the result requirements (`state_token`, batch fields, and a judgment when the stage has a semantic gate), and shows copyable `next`/`check` commands; after a check the detail refreshes with the next handoff state, and a blocked check shows the corrections in the same handoff. |
| WEB2-05 Executor wording and control alignment | Render the wizard check cards in `Built-in script`, `Agent CLI`, and `Library`/`Library checkpoint` modes at full, 980 px, and 720 px widths; render the Checkpoints view, the semantic card, and a library-bound wish detail; copy the visible executor labels into the assessment rows. | The option group is labelled `Check executor` with exactly the three documented choices (`Library` may be the short segmented label; chips and detail rows say `Library checkpoint`); every check node states that stage work is performed by the working agent and the choice selects only the verifier; the executor control and its dependent columns form one stable grid and never overlap or shift baseline when the mode changes or the viewport narrows; the Checkpoints view, wizard review, lifecycle nodes, and assessment rows use the same vocabulary and show `judged_by` plus assessor. Screenshot evidence is retained with the case, and it establishes only the rendered state, not an engineering acceptance. |

The interface oracle for WEB2-01 to WEB2-03 is [I19](../interfaces/web-api.md#i19) and the
detail fields in [I17](../interfaces/web-api.md#i17); the create request fields are in
[I18](../interfaces/web-api.md#i18). The handoff and wording claims are the design in
[D105](../design/web-operator.md#d105) and [D106](../design/web-operator.md#d106), and the
stage interaction they expose is the [stage CLI contract](../design/stage-cli.md#d85). The
frozen-plan durability check is LIB-10; a UI alignment screenshot is not executed
acceptance.


<a id="d110"></a>

## 7. Checkpoint/operator reconciliation and project portability cases

`XDesignVerification::CheckpointOperatorDesignReview` allocates the current source,
configured-authority and operator requirements in [R16](../requirements/requirements.md#r16).
The review compares the as-built model with `src/checkpoint/library.rs`,
`src/simple_flow/blueprint.rs`, `src/simple_flow/check.rs`, `src/stage_cli/work.rs`,
`src/stage_cli/persistent.rs`, `src/core/plan.rs` and `web/server/`. The CUSTOM, LIB, WEB
and WEB2 matrices remain behavior verification design; their earlier execution records
are not rerun or replaced by a documentation review.

| Review | Required observation | Limit |
| --- | --- | --- |
| DESIGN-01 Source/model allocation | Project library/source paths, frozen plan/work fields, actual authority and delegated web mutations agree with the implementation and model/source map | Inspection establishes this allocation, not a new runtime acceptance |
| DESIGN-02 History and preservation | Distinguish partial definition history, frozen binding fields and immutable execution facts; identify missing version/digests, current script execution, live version summaries and identity-bound store recovery | Do not infer executor pinning, migration or backup support |
| DESIGN-03 Policy and proposal boundary | Retain the generic scripted-only requirement; expose the implemented CLI deviation; proposed identity/package/history definitions have no implemented satisfiers | A compiler pass cannot resolve policy or evaluate assertion bodies |
| DESIGN-04 Mechanical consistency | Model-only and full pinned compiler parse/check, default documentation audit and strict docs build; retain reports and unevaluated diagnostics | These checks do not establish engineering acceptance or independent review |

`CrossProjectCheckpointReview` defines future acceptance of the proposed requirements.
These cases are not executable capabilities of the current release. Each needs a new
implementation flow and retained results before it may be reported passed.

| Case | Scenario | Required outcome |
| --- | --- | --- |
| PROJECT-01 Real-project boundary | Use the binary in two fresh projects with different roots, local stores and project-specific input layouts; omit the xdesign repository's scripts | Every source read/write and artifact collection uses the selected project; missing scripts/mappings fail explicitly; listing a built-in does not imply it is runnable |
| PROJECT-02 Pinned shared definitions | Import a shared package into A and B; include two namespaces with the same local checkpoint ID; update/remove upstream after registration | Qualified origins and complete bundle/definition digests remain inspectable; imports do not shadow; earlier bindings remain unchanged |
| PROJECT-03 Executor and input drift | Change a script, imported helper, rule-data file, environment identity or consumer mapping after binding; repeat with a stub agent | Frozen bundle contract detects drift or executes the retained bundle; the reviewed inputs are identified; re-execution creates a distinct attempt and no deterministic agent guarantee |
| PROJECT-04 Federated history | Give A and B identical wish/checkpoint IDs, multiple failed/passed attempts and the same project across two stores; then make one store unavailable | Project/store/revision/assessment references distinguish results; source authority is preserved; unavailable/stale sources are explicit; the index cannot authorize a gate |
| PROJECT-05 Clone, worktree and archive | Fresh clone, moved workspace, concurrent worktrees, project fork and archived project | Explicit project identity and workspace/store selection preserve provenance and isolate materialization; Git clone alone does not claim to restore operational history |
| PROJECT-06 Verified restore | Remove disposable cache; separately lose or relocate a store and perform a future supported verified archival/restore procedure on another machine | Cache restoration preserves retained material bytes; store restore preserves logical history and evidence under its designed identity rules; raw copy is never reported as supported restore |

PROJECT-02..06 require interfaces beyond the current strict schemas. No passing SysML
analysis, single-project test result or record backfill substitutes for those executions.

<a id="d112"></a>

## 8. Released checker and script isolation acceptance

`CheckerIsolationVerification` plans the following executable cases. The design is
[checker execution D111](../design/checker-execution.md#d111), covering
[requirements R18](../requirements/requirements.md#r18). Freeze this matrix and review
its methods before implementation. Results belong to store record
`record-execution-checker-script-isolation`; a compiler pass is not case acceptance.

| Case | Method and required observation |
| --- | --- |
| ISO-01 | Build a release, copy only its executable to a separate installation directory, invoke it in a consumer project without `tools/`; default input discovery and the real requirements checker use embedded scripts/helper/parser. A malformed requirement fails; valid fixture requirements pass. |
| ISO-02 | Put an incompatible same-named `tools/checks/` script in the project; default selection still executes embedded bytes. Explicit command selection uses project bytes. Built-in library listing shows available resources without a customer script file. |
| ISO-03 | Register an entry with a helper and rule data, edit or delete all authored files/library, then evaluate: only the retained bundle answers the frozen rules. A new registration sees the edited version. Undeclared/missing helper fails honestly. |
| ISO-04 | Delete the run's executor cache after registration, recover with `next`, and execute. Check all recovered file hashes against revision 1 and checkout. No authoritative executor bytes exist only in cache. |
| ISO-05 | Change a retained file, provide path traversal, symlink entry/helper, duplicate dependency or missing interpreter. Refuse initialization or evaluation with no passed gate and no source fallback. |
| ISO-06 | Two independent projects use the same wish/checkpoint IDs and distinct helpers/data. Interleave process calls and verify cwd, root, outputs, reports and store manifests remain project-specific. |
| ISO-07 | Select a Python environment under project `cache/`, inject inherited Python import settings, and check that execution uses the selected interpreter, clears import settings, disables user-site writes and retains interpreter version/digest plus all executor-file hashes. |
| ISO-08 | Reevaluate a failing checker and compare attempt directories: temporary/cache paths and stdout/stderr do not overlap. Input discovery and evaluation obey deadlines; a failing/malformed/timed-out script never passes. |
| ISO-09 | Run existing Rust/Python checks, model-only/full parse/check, documentation audit/build and affected figure checks; inspect old plans without `bundle`. Preserve diagnostics and unevaluated constraints; disclose unrelated failures. |

The release demonstration is host-specific to the selected bundled parser. Retained lock
files plus interpreter identity are inspectable provenance, not a copy of installed Python
packages. No OS sandbox, full environment replay, unified history service or supported
whole-store restore is established by these cases. Authoring-agent review and technical
acceptance do not claim independent review or final human acceptance.

<a id="d114"></a>

## 9. Portable document kit verification

`DocumentKitVerification` covers [R19](../requirements/requirements.md#r19)
and the [engine design](../design/document-kit.md#d113). These planned cases
will test the reusable component against a temporary consumer project, not
just this repository's existing pages.

| Case | Method and required observation |
| --- | --- |
| DOC-01 | Select the versioned profile from a separate project root. Plan each declared document class and inspect its allowed destination, required ordered headings, anchor policy and template source; compare the contract digest before and after one template-byte change. |
| DOC-02 | Create a complete document with caller-supplied content. Check the exact path, rendered chapters, stable anchors and managed registry entry; rerun the same request and refuse an unrequested overwrite without changing bytes. |
| DOC-03 | Reject an empty required chapter, unresolved template slot, missing or duplicate heading, reordered chapter, duplicate anchor, invalid filename, broken `DocSection`/`DocFigure` target, unsupported annotation form and removed managed marker with specific diagnostics. An existing page remains byte-identical on failed update. |
| DOC-04 | Refuse absolute paths, `..` traversal, symlink escapes and stale expected hashes. Keep all temporary output within the target project's `cache/` or an in-memory buffer, and reject a cross-filesystem atomic replacement cleanly. |
| DOC-05 | Given an existing canonical SVG beneath the carrying page's `assets/` child, attach its embed and six ordered figure fields. Reject a missing source, duplicate figure label, wrong section, broken asset and incomplete block. |
| DOC-06 | Compare the same plan/create/check request through the CLI and MCP adapter. A stdio MCP client can connect, list the tools and contract resources, and call a tool in the temporary project; a tool call cannot redirect to another project/profile. |
| DOC-07 | Run xdesign's existing documentation audit and strict docs build; parse/check the changed model with the pinned compiler and retain diagnostics and unevaluated rules. Review final prose and any figure against their model source separately. |

A deterministic pass establishes only the stated structural predicates. The
review must record whether the rules and templates express the intended
design-first practice and whether the sample document communicates a real
design claim. A SimpleFlow verification assessment must cite the
executed commands and final bytes; declaring these cases does not execute
them. Executed case results belong to the wish's revision-bound record.
