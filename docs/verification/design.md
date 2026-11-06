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

The persistent adapter additionally exercises these retention oracles in isolated
fixtures, without requiring an operational wish for maintenance work:

| Condition | Required oracle |
| --- | --- |
| A later stage edits a previously assessed source URI | Same run progresses using the earlier retained bytes; original activities, assessments and canonical artifact snapshot remain unchanged. |
| Several attempts assess different bytes at one URI | Each checker input version remains retained by its own digest, including failed attempts. |
| Retained cache copy is missing or corrupted | Missing bytes reconstruct from the committed store without overwriting live edits; corruption refuses the next check with the offending path. |
| A semantic PASS omits changed upstream product bytes | Reject before checker execution; a current, fully bound result can be assessed. |
| A legacy record omits retention metadata | Keep live-file integrity checks and prior compatibility behavior. |

The Rust cases are in `tests/stage_cli.rs`; direct retained-versus-live checker
counterexamples are in `tests/test_simple_flow_checkers.py`. These oracles exercise
[E03](../design/evidence-and-revisions.md#e03); they do not add new semantic gates or
claim automatic upstream rework.

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
run record retains assessments and evidence. New syntactic bindings use script authority;
Agent CLI bindings are rejected and old frozen plans remain inspectable without new agent
execution ([D88](../design/flows/simple-flow/definition.md#d88)). The current
[requirement allocation](../requirements/requirements.md#r16) and
`CheckpointOperatorDesignReview` trace that boundary. These cases state obligations to
apply when the feature is exercised; they do not
extend the executed acceptance of the Rust slices, and they are not executed results.
Executed outcomes and retained command evidence belong to the branch's execution record.

| Case | Input or operation | Expected result |
| --- | --- | --- |
| CUSTOM-01 Default plan unchanged | Register a wish with no `--checks` option and register a second wish with a default-only binding file; compare each frozen plan and `plan_sha256` with the built-in plan, and evaluate the four syntactic gates. | Both runs carry the built-in plan bytes; no custom runner is serialized; the default assessor is still the checker id with script authority; the two frozen hashes agree and `flow-is-supported` holds for both runs. |
| CUSTOM-02 Agent CLI rejection and historical fail-closed behavior | Attempt to register a new `mode: "cli"` binding or save a CLI-backed library entry; inspect an older frozen CLI-bound run and call `check` on an unfinished stage. | New requests fail before a wish revision or library update; the older plan and retained assessments remain readable, while the attempted check records no passed assessment and never launches an agent. |
| CUSTOM-03 Script authority, assessor, and restore | Bind a project script, run a check, inspect its assessment, restore in a fresh process, run the record checker and web detail query, then edit the original `--checks` file and re-query. | The assessment records `judged_by = script` and the frozen checker id as assessor; restore keeps the completed visit authorized with its retained report; the store and web report the same frozen binding; editing the binding file changes nothing for the run. |
| CUSTOM-04 Invalid bindings rejected | Present malformed JSON, an unknown field, a wrong `schema_version`, an unknown or duplicate checkpoint id, an unknown mode including `cli`, and a `command` mode without `command_uri`; then register with a partial default file. | `init` exits 2 with the named validation error before creating the wish item or any store revision; the store head and cache are unchanged; the partial file succeeds and every omitted checkpoint is `default`. |
| CUSTOM-05 WishVerifyFlow rejects non-default bindings | Register `--flow wish-verify` with a non-default binding, and separately with a default-only file. | The non-default binding is rejected as an invalid request before registration because the flow declares only semantic checkpoints; the default-only registration succeeds and leaves its two semantic judgments supplied through the result exchange. |
| CUSTOM-06 Command binding, timeout, and read-only safety | Bind a checkpoint to `mode: "command"` with an existing project-relative `.py` stub implementing the checker invocation and report interface; also present a missing, non-normalized, or non-`.py` `command_uri`; repeat with a stub that hangs past `XDESIGN_CHECKER_TIMEOUT_SECS` and with a stub that returns a malformed report. | The invalid `command_uri` form is rejected at registration before any run exists; the valid command runs under the deadline, reports the declared rule ids, and is recorded with script authority and the checker id as assessor; the hanging process tree is killed, an explicit timeout is recorded, partial logs are retained, and the gate is unevaluated and blocks the visit; the malformed report creates no passed assessment. The bound command cannot alter the plan, the criteria, or the assessed revision. |

The authority oracle for CUSTOM-03 and CUSTOM-06 is the assessment record and
its retained report ([E04](../design/evidence-and-revisions.md#e04)); the report validation
requirements are those of the checker command contract ([D07](../design/flows/simple-flow/definition.md#d07)).
The default-behaviour guard for CUSTOM-01 and CUSTOM-05 is the unchanged serialization and
the existing default suites; the binding schema and exit codes are those of the
[stage CLI](../design/stage-cli.md#d89). The cases fix observable authority, evidence, and
rejection behavior rather than message wording: the exact validation text an invalid
binding prints is an implementation detail, so a case asserts failure and the absence of a
committed run rather than a specific message. CUSTOM-02 separately checks that an older
plan remains readable without permitting another agent execution.

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
| WEB-03 Lifecycle actions against a temporary store | From the page or its API, create a SimpleFlow wish with mixed `default` and `library` bindings, get next work, submit a check whose semantic judgment is filled from the work snapshot digest, repeat a duplicate create, and check a completed run. | Creation runs the CLI's `init` and commits one registration; the detail view reports the frozen binding per checkpoint; `next` returns work without changing the stored head; the check records outputs and assessments through the CLI's `check` and returns its exit-code semantics (`exit_code: 1` for a blocked stage as a successful envelope); a duplicate create is `409 already_exists` and a completed check is `409 already_delivered`; all writes land in the temporary project and store roots, not the repository's `.xdesign`. |
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
script stubs, and executed outcomes belong to the branch's execution
record. The [current requirements](../requirements/requirements.md#r16) and
`CheckpointOperatorDesignReview` allocate the source, authority and operator obligations.
The script-only policy and incomplete executable/provenance freezing remain explicit
in the [as-built project boundary](../design/checkpoint-library.md#d107).

| Case | Input or operation | Expected result |
| --- | --- | --- |
| LIB-01 Library load, missing file, and validation | With no `checkpoints/library.json`, list the library; then present malformed JSON, an unknown field, an unknown kind, a syntactic entry without rules/inputs/executor, a semantic entry with rules or without a true judge, a duplicate id, and a non-slug id. | The missing file is not an error and the effective library is the five built-ins; every malformed or invalid entry is an explicit invalid-request error (exit 2) that names the problem, and the runtime never guesses a kind or drops an entry silently. Historical CLI entries may still be loaded for inspection, but CUSTOM-02 covers rejection of new saves and bindings. |
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
| WEB2-03 Create-wish library bindings | Create a wish through `POST /api/wishes` with syntactic and semantic library bindings, a mixed default/library set, a kind mismatch, a missing library id, a `cli` mode, a `command` mode, and an unreadable library. | Valid library bindings are passed through unchanged and `init` commits the registration; the detail shows the frozen library plans; kind mismatch and unknown ids are `400 invalid_checks` (pre-validated from the current library, with the CLI's own rejection mapped to the same code when the library cannot be read); `cli` and `command` modes are rejected at the web boundary. |
| WEB2-04 Start-work handoff content and non-automation | Create a wish, inspect the run without calling `next`, then render the detail page and handoff card; copy the prompt and commands; run a successful check and a blocked check. | Creating the wish does not start any stage work or spawn a working agent: the run waits at its first stage and only `next`/`check` subprocesses occur; the handoff card explains that the flow does not auto-run, shows a copyable prompt containing the wish id, title, current stage, goal, instructions, the commands, and the result requirements (`state_token`, batch fields, and a judgment when the stage has a semantic gate), and shows copyable `next`/`check` commands; after a check the detail refreshes with the next handoff state, and a blocked check shows the corrections in the same handoff. |
| WEB2-05 Executor wording and control alignment | Render the wizard check cards in `Built-in script` and `Library`/`Library checkpoint` modes at full, 980 px, and 720 px widths; render the Checkpoints view, the semantic card, and a library-bound wish detail; copy the visible executor labels into the assessment rows. | The option group is labelled `Check executor` with exactly the two documented choices (`Library` may be the short segmented label; chips and detail rows say `Library checkpoint`); every check node states that stage work is performed by the external working agent or human and the choice selects only the verifier; the executor control and its dependent library selector form one stable grid and never overlap or shift baseline when the mode changes or the viewport narrows; the Checkpoints view, wizard review, lifecycle nodes, and assessment rows use the same vocabulary and show `judged_by` plus assessor. Screenshot evidence is retained with the case, and it establishes only the rendered state, not an engineering acceptance. |

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
| DESIGN-03 Policy and proposal boundary | Confirm new syntactic bindings are script-only, historical CLI plans remain readable without launching an agent, and proposed identity/package/history definitions have no implemented satisfiers | A compiler pass cannot evaluate assertion bodies or prove legacy replay safety |
| DESIGN-04 Mechanical consistency | Model-only and full pinned compiler parse/check, default documentation audit and strict docs build; retain reports and unevaluated diagnostics | These checks do not establish engineering acceptance or independent review |

`CrossProjectCheckpointReview` defines future acceptance of the proposed requirements.
These cases are not executable capabilities of the current release. Each needs a new
implementation flow and retained results before it may be reported passed.

| Case | Scenario | Required outcome |
| --- | --- | --- |
| PROJECT-01 Real-project boundary | Use the binary in two fresh projects with different roots, local stores and project-specific input layouts; omit the xdesign repository's scripts | Every source read/write and artifact collection uses the selected project; missing scripts/mappings fail explicitly; listing a built-in does not imply it is runnable |
| PROJECT-02 Pinned shared definitions | Import a shared package into A and B; include two namespaces with the same local checkpoint ID; update/remove upstream after registration | Qualified origins and complete bundle/definition digests remain inspectable; imports do not shadow; earlier bindings remain unchanged |
| PROJECT-03 Executor and input drift | Change a script, imported helper, rule-data file, environment identity or consumer mapping after binding | Frozen bundle contract detects drift or executes the retained bundle; the reviewed inputs are identified; re-execution creates a distinct attempt without erasing earlier evidence |
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
and the [engine design](../design/document-kit.md#d113). Its subject is
`PortableDocumentKit` because the cases cover both catalog/source obligations
and engine/adapter behavior. These planned cases will test the reusable
component against a temporary consumer project, not just this repository's
existing pages. DOC-01 through DOC-07 retain their original regression meaning.
DOC-08 through DOC-15 add the scenario catalog, mixed selection, provenance,
and retained-compatibility acceptance boundary.

| Case | Method and required observation |
| --- | --- |
| DOC-01 | Select the versioned profile from a separate project root. Plan each declared document class and inspect its allowed destination, required ordered headings, anchor policy and template source; compare the contract digest before and after one template-byte change. |
| DOC-02 | Create a complete document with caller-supplied content. Check the exact path, rendered chapters, stable anchors and managed registry entry; rerun the same request and refuse an unrequested overwrite without changing bytes. |
| DOC-03 | Reject an empty required chapter, unresolved template slot, missing or duplicate heading, reordered chapter, duplicate anchor, invalid filename, broken `DocSection`/`DocFigure` target, unsupported annotation form and removed managed marker with specific diagnostics. An existing page remains byte-identical on failed update. |
| DOC-04 | Refuse absolute paths, `..` traversal, symlink escapes and stale expected hashes. Keep all temporary output within the target project's `cache/` or an in-memory buffer, and reject a cross-filesystem atomic replacement cleanly. |
| DOC-05 | Given an existing canonical SVG beneath the carrying page's `assets/` child, attach its embed and six ordered figure fields. Reject a missing source, duplicate figure label, wrong section, broken asset and incomplete block. |
| DOC-06 | Compare the same plan/create/check request through the CLI and MCP adapter. A stdio MCP client can connect, list the tools and contract resources, and call a tool in the temporary project; a tool call cannot redirect to another project/profile. |
| DOC-07 | Run xdesign's existing documentation audit and strict docs build; parse/check the changed model with the pinned compiler and retain diagnostics and unevaluated rules. Review final prose and any figure against their model source separately. |
| DOC-08 | Load bundled 1.1.0 and a reviewed derived 1.1 fixture; enumerate every schema 1.1 entry/resource and corrupt each required declaration and default cardinality. Require every entry's nonempty known-source list, all eight research IDs in at least one bundled entry, primary/additional source links, license fields, and a `not-copied`/`copied` decision per entry-source pair; submit `copied` without `copying_review`. The bundled catalog has exactly 16 entries while derived inventory is unconstrained; invalid catalogs fail before writes. Execute S-01 through S-05: exact first-line/non-contract identity, no actual `D` in examples, total-file and whole-pack sentinel scans, and missing/duplicate/misplaced/malformed sentinel rejection as `invalid-catalog`. Retain the source matrix, inventory, example hashes/locations, diagnostics, and unchanged-target evidence; copying adequacy remains DOC-15. |
| DOC-09 | Explicitly plan/create all eleven scenario and five generic IDs; repeat generic selection by omission and compare effective identity and section contract. Exercise unknown, malformed, cross-class and output-path inputs and zero/two defaults for one class. Explicit generic IDs and omission select the same entry; invalid requests and malformed default cardinality fail with stable diagnostics and no writes. For S-06, inject a test-only provider returning `Z` at the catalog loader/digest-strategy test seam and call get/plan/create/check only in that test process; production SHA-256 and CLI/MCP wiring remain unchanged and product/pack inputs cannot reach the stub. Injected recomputation `D=Z` is `invalid-catalog` even if a descriptor agrees; independently, descriptor `contract_sha256=Z` with real nonzero recomputation is `contract-drift`, and request `expected_contract_sha256=Z` under the production provider is `invalid-request`. Neither identity is selected or written, and the injection is never exposed through CLI/MCP. |
| DOC-10 | In one temporary project create all five classes and at least two templates in one class with a concern-to-template/omission list; update sections, attach figures, change the current default, and independently alter the document's own marker, registry, class, template ID, profile/version, digest, or digest version. Inspect the mixed check response's selected catalog identity and every per-document `effective_identity`, digest, and digest version. Operations use the resolved original template and the check reports selected and per-document effective identities/digests/versions; actual mismatches fail without rewrites; the valid mixed example passes with its coverage list. A recognized legacy difference from the selected catalog is resolved under DOC-13 rather than treated as a mismatch. For S-07, sentinel markers/registries, copied examples, and forged sentinel registries are `registry-mismatch` without adoption, registration, or writes. Retain response identities and registry/before/after hashes. |
| DOC-11 | Under `cache/document-template-library-acceptance/`, create one temporary consumer root per bundled entry (all five generic and eleven scenario entries) and one separate root for the single mixed-template example; the mixed root is not required to instantiate all 16 examples. Require that example to use all five bundled classes, at least two template IDs in one class, and a concern-to-template/omission list. Record full plan/create caller inputs and selected profile/version/digest-version identity in each digest-covered bundled fixture, and materialize an external acceptance manifest outside the pack and its digest closure containing only actual expected `D` plus `fixture_path` and `fixture_sha256` references to that bundled fixture, not a second full-input copy; instantiate examples through those operations so creation establishes the registry. Supply parser-valid fictional `mbse/model/example.sysml` locators and, only for an example page that embeds a model figure, valid canonical SVGs beneath its `assets/` child; figure-free examples require no SVG. Apply section 9.1's exact one-field comparison and S-01, S-03/S-04, S-07 through S-12, then run `xdesign-doc-kit check-documents --target-root <temporary-project> --profile-root <bundled-1.1-pack>` for every isolated root and once for the mixed root. Exercise figure, reasoned table/prose alternative, missing decision, empty rationale, and malformed block fixtures. Every one of the 16 isolated examples and the single mixed whole-project example exit 0; the mixed example's five-class/two-template/coverage-list content is verified; invalid alternatives fail without a class-wide diagram mandate. Sentinel, stale-digest, copied-example, and unapproved-normalization cases fail closed with section 9.1 diagnostics. Retain bundled fixture paths/SHA-256 and full caller inputs, external acceptance-manifest SHA-256/actual `D`/fixture references, model/SVG hashes, commands, identities/hashes, unchanged example/registry evidence, and findings. |
| DOC-12 | Against derived v1 and bundled 1.1 packs, compare every named response field for get/plan/create/update/attach/check: catalog/source data, `template_id`, per-operation and per-path `effective_identity`, exact template bytes, and top-level/per-identity `contract_digest_version` (`"1"` for v1, `"2"` for 1.1). Compare existing MCP contract/rules/class-template resources and additive `xdesign-doc-kit://catalog` and `xdesign-doc-kit://template/{document_class}/{template_id}` with CLI `get_contract`; assert the synthesized v1 projection has `template_id=null` and null scenario-only fields, identifies only the actual pack, and is exempt from the 1.1 manifest/guide/example/source package rather than rejected as malformed. Attempt root/profile redirection and selected/compatibility drift. All existing and enumerated additive surfaces agree, including synthesized v1 data; redirection and drift fail closed. For S-06, S-12, and S-14, DOC-09's zero provider is unreachable through both adapters, selected and MCP-pinned identities use actual nonzero `D`, recomputed actual digests propagate, stale/zero digests fail without refresh, examples are not templates, and no example bytes become a selected contract. Retain actual stdio MCP transcripts and CLI JSON. |
| DOC-13 | Run unchanged DOC-01 through DOC-07 against bundled and custom-derived v1 packs, including creation of a new schema 1.0 document whose effective identity has `template_id=null` and digest version `"1"`. Positively check/update/attach an exact bundled-v1 document in a 1.1 mixed project, resolving retained v1 before any own-identity mismatch check and inferring absent legacy fields without rewriting them; independently mismatch its marker/registry/class/template/profile/version/digest/digest-version, including sentinel identities. Create a 1.1.0 document, derive 1.1.1 by a guide-only change, try the old identity under 1.1.1, then select retained 1.1.0 and update it. Attempt kit update/attach on an unmanaged page. Existing and newly created schema 1.0 documents use `template_id=null`; recognized legacy operations pass despite different selected-catalog identity; actual mismatches, unknown identities, and the 1.1.1 route fail without writes; original-pack operations succeed without migration and preserve marker/effective identity/registry bytes; unmanaged updates are refused byte-identically. For S-13, reproduce `1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6`, verify v1 has no examples/sentinel and unchanged marker/registry behavior, require no migration, and apply the new rule to derived 1.1 profiles. Retain independent contract/document hashes and marker/registry snapshots. |
| DOC-14 | Change one byte in every template, manifest, guide, example body, example non-contract marker, example sentinel, source register, rule, profile, fixture, and compatibility resource. Exercise traversal, absolute paths, symlinks, conflicting resources, malformed UTF-8/JSON, unsupported schemas/digest versions, supplied schema 1.0 `template_id`, `javascript:`, `file:`, `data:`, protocol-relative, credential-bearing, control-character, and unsafe Markdown URLs, plus &lt;script&gt;, event handlers, and disallowed raw HTML. Positively test `license_name=unknown` and `license_name=unlicensed` with `license_url=null`; negatively test `license_url` values `unknown`, `unlicensed`, and each forbidden scheme, and a known license with null URL. Covered raw changes alter the applicable digest even when the mutated catalog is invalid. License sentinel strings are valid only in license name/version with null URL; the separate 64-zero example sentinel is valid only in section 9.1's exact stored-example contract field and never as actual/registered identity. Execute S-02 through S-06, S-11, and S-12 using DOC-09's isolated test provider, leaving production SHA-256 and CLI/MCP wiring unchanged and attempting no real preimage; test-only injected zero recomputation and descriptor/request zero cases, misplaced sentinels, stale actual digests or fixture/external-manifest hashes, supplied v1 selection, and every unsafe fixture fail before writes with no digest-time normalization, network, or code execution/import/evaluation and unchanged pages/registry. Retain raw/released digests, fixtures, propagation evidence, and network-observation evidence. |
| DOC-15 | DeepSeek supplies the independent semantic review and Kimi supplies a separate runner evidence/limitations judgment for this wish, both permitted by the applicable SimpleFlow semantic checkpoint and neither substituting for the other. DeepSeek reviews all guides, examples, source/license/copying records, mixed coverage, final figures or diagram decisions, and S-01 through S-16 allocation/evidence together with DOC-08 through DOC-14 at one revision; Kimi separately reviews the assembled evidence and limits. The record identifies assessed correction run/revision, parent artifact preservation, effective digests and digest versions, commands/transcripts, isolated/mixed bundled fixture caller inputs and paths/SHA-256, external acceptance-manifest SHA-256, actual digest values, and fixture references, model stubs and conditional SVG hashes, before/after document/example/registry hashes, diagnostics, unevaluated checks, both identities/checkpoint authorities, findings, and residual risks. Judge scenario fit, useful depth, coverage/omissions, technical correctness, source/attribution/license/copying adequacy, sentinel/comparison consistency, figure fidelity/readability, and diagram alternatives. S-15 records design/API/model/product/test follow-up without claiming completion; S-16 retains correction hashes/logs and all 26 unevaluated rules. Mechanical reports and requirements-stage checks cannot substitute or execute the product cases. |

### 9.1 Example sentinel and digest correction

For all schema 1.1 stored examples, `Z` is exactly `0000000000000000000000000000000000000000000000000000000000000000`. The bundled scope is the 16
manifest-named files under `document-kit/catalog/examples/`; derived 1.1 profiles apply the same rule to
their declared examples. Each first line is exactly
`<!-- document-kit: profile={profile_id}@{version} class={document_class} template={template_id} contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->`
with actual non-contract identity and no leading whitespace or extra marker. `Z` occurs exactly once in
the entire example and nowhere else in any pack resource; target marker/registry sentinel identity is
separately `registry-mismatch` under S-07. Actual `D` occurs nowhere in the stored example. Every stored
example byte, including `Z`, is hashed exactly as stored with no digest-time normalization; computed
`D == Z` is explicitly invalid. Each digest-covered bundled fixture is the canonical full plan/create
caller-input record with selected profile/version/digest-version identity, but neither `D` nor `Z`.
An acceptance manifest outside the pack and digest closure contains exactly actual `D`, `fixture_path`,
and `fixture_sha256`, never a self-hash field. Separate S-12/S-16 evidence computes and retains the
manifest's SHA-256; any `manifest_sha256`, `self_sha256`, or equivalent field is `fixture-mismatch`
before creation without field-exclusion normalization. Digest, fixture, or retained manifest-hash
drift fails as stale.

For each comparison, let `E` be the exact stored example bytes, `C` the exact created
bytes, `Z` the fixed 64-zero sentinel, and `D` the actual selected catalog digest. The only
permitted comparison transformation is the following deterministic, test-only algorithm:

1. Recompute `D` from the exact selected pack and reject the catalog if `D == Z`. Each bundled
   caller-input fixture is digest-covered and is the canonical full plan/create caller-input
   record, including model/figure inputs, plus selected `profile_id`, `version`, and
   `contract_digest_version`; it stores neither `D` nor `Z`, because storing `D` there would
   reintroduce self-reference. Before creation, materialize an acceptance manifest outside the
   selected pack and its digest closure containing exactly three fields: actual
   `expected_contract_sha256=D`, `fixture_path`, and `fixture_sha256` for that bundled fixture.
   Do not duplicate caller inputs or put any manifest/self/own digest field in the external
   manifest. S-12 submits `manifest_sha256`, `self_sha256`, and equivalent self-hash fields and
   rejects each as `fixture-mismatch` before creation, with no field exclusion or other
   normalization. Separately computed S-12/S-16 evidence records the external manifest's SHA-256;
   the hash is never read from the manifest. Recompute the bundled fixture hash. On reuse, any
   digest, fixture path/hash, or retained external-manifest hash mismatch is a stale-fixture
   failure, never a silent refresh. Neither fixture form uses `Z` as expected or selected identity.
2. Validate that `E` is one of the manifest's stored examples and has exactly one first-line
   managed marker in the exact profile form, actual profile/version/class/template identity,
   `digest-version=2`, and `Z` only as its `contract` value. Validate the whole pack and all of
   `E` for sentinel location and count, and assert that `D` does not occur in `E`.
3. Independently parse and assert the actual `C` marker and registry entry before comparison:
   path, profile/version, class, template, `D`, and digest version must agree, and neither marker
   nor registry may contain `Z` as identity. Missing, duplicate, or mismatched markers/registries
   fail closed. A copied example or forged sentinel registry is not adopted or registered.
4. Locate only the `contract=` value span in `E`'s first-line marker. In an in-memory copy `E'`,
   replace that one field value, `Z`, with `D`. Do not globally replace either value and do not
   alter any other marker field, body byte, whitespace, line ending, or trailing newline.
5. Assert that the created marker equals the `E'` marker, including actual `D`, then compare all
   of `E'` with all of `C`. Any other difference is `fixture-mismatch`. The comparison never
   changes `E`, `C`, the selected digest, or the registry; assert their before/after hashes.
6. Run `check-documents` against the unchanged created document and registry and require the
   declared structural predicates to pass. The fixture transformation is not a product operation
   and is never applied by plan/create/update/attach/check to product bytes.
7. Apply the same algorithm separately to every mixed `documents[].expected_document` reference.
   Those references reuse the same 16 stored examples; the mixed fixture has no second stored
   example-byte convention. Retain its five-class, two-templates-in-one-class, and coverage-list
   assertions.

Marker stripping, body-only comparison, whitespace or line-ending normalization, normalization
of another field, global replacement, and every other shortcut are unapproved normalization.
They must fail crafted negative fixtures with `fixture-mismatch`; a comparator that accepts them
fails DOC-11 even if its positive fixtures pass. The actual digest may legitimately occur in a
created body, so only the parsed first-marker field span may be substituted.

Use existing stable diagnostics by context and evaluate request shape, selected pack, target
identity, then fixture comparison, in that order. `expected_contract_sha256=Z` is
`invalid-request` before any catalog or target write. A selected pack whose recomputed `D == Z`
is `invalid-catalog`, even when a test descriptor also claims `Z`; a descriptor that claims `Z`
while actual recomputation is nonzero is `contract-drift`. S-05 covers only sentinel defects in
the selected pack and its digest-covered resources, all reported as `invalid-catalog` before
target operations. S-07 covers a target document marker or target registry identity containing
`Z`, a copied example page, and a forged target registry, all reported as `registry-mismatch`;
a valid stored example does not make its copied target page a pack-validation case. Only after
the selected pack and target identity pass does a byte-comparison discrepancy become
`fixture-mismatch`. Every rejection leaves target documents and registry bytes unchanged and
never adopts, migrates, or registers an example.

The correction subcases below refine, but do not renumber or expand, DOC-01 through DOC-15. Their
DTL/project allocations preserve the canonical case unions. Every negative case asserts unchanged
target/registry bytes. S-01 through S-14 remain future product acceptance cases; S-15 is an obligation
and S-16 is requirements evidence, so declaring them here is not execution or acceptance.

| Case | Acceptance obligation | DTL / project requirement | DOC coverage |
| --- | --- | --- | --- |
| S-01 | Inventory all 16 bundled examples and mixed references; assert first-line exact marker, actual non-contract identity, one whole-file `Z` only in `contract`, and no actual `D` in stored example bytes. | DTL-002, DTL-011 / `documentTemplatesCoverScenarios` | DOC-08, DOC-11 |
| S-02 | Scan every example and every other pack resource; permit `Z` only in the 16 example contract fields and reject a planted sentinel elsewhere. | DTL-002, DTL-015 / coverage and compatibility | DOC-08, DOC-14 |
| S-03 | Replace the example contract with actual `D`, other 64-hex, empty, 63/65-zero, uppercase, prefixed, or unknown values. | DTL-002, DTL-011 / coverage | DOC-08, DOC-11, DOC-14 |
| S-04 | Plant two markers, two contract sentinels, or a second sentinel in the body; structural validation rejects every duplication. | DTL-002, DTL-011 / coverage | DOC-08, DOC-11, DOC-14 |
| S-05 | Plant `Z` in a stored example's non-contract marker field or another digest-covered pack resource: guide, template, bundled fixture, manifest, source register, model/asset, compatibility pack, mixed fixture, or coverage; expect `invalid-catalog`. Target markers/registries are excluded and belong to S-07. | DTL-002, DTL-015 / coverage and compatibility | DOC-08, DOC-14 |
| S-06 | At the catalog loader/digest-strategy test seam, inject a test-only provider returning `Z` with production SHA-256 and CLI/MCP wiring unchanged and no product/pack access to the stub. Call get/plan/create/check in that test process; independently test descriptor `contract_sha256=Z` with real nonzero recomputation and request `expected_contract_sha256=Z`. Expect `invalid-catalog`, `contract-drift`, and `invalid-request` respectively, with no selection or writes. | DTL-005, DTL-008, DTL-012 / selection, surface, compatibility | DOC-09, DOC-12, DOC-14 |
| S-07 | Use a sentinel target registry entry or created target marker, copy an example page, and forge a target sentinel registry. Expect `registry-mismatch` without adoption, registration, or writes; these are not S-05 pack defects. | DTL-006, DTL-007, DTL-011 / selection, coverage, compatibility | DOC-10, DOC-11, DOC-13 |
| S-08 | Exercise marker stripping, body-only comparison, whitespace/line-ending normalization, other-field normalization, and global replacement; only exact one-field substitution is accepted. | DTL-011 / coverage | DOC-11 |
| S-09 | Create each of the 16 isolated fixtures; independently assert actual marker/registry `D`, apply only the stored-example field substitution, and require full-byte equality and structural pass. | DTL-011 / coverage | DOC-11 |
| S-10 | Apply the same comparison to all six current mixed references; assert all five classes, two template IDs in one class, and the coverage list. | DTL-011 / coverage | DOC-10, DOC-11 |
| S-11 | Independently mutate one example body byte, one non-contract marker byte, and the sentinel; the raw v2 digest changes each time, invalid mutations are rejected, and valid drift needs a new release/digest with no stale acceptance. | DTL-011, DTL-012 / coverage and compatibility | DOC-11, DOC-14 |
| S-12 | Recompute actual `D` after valid drift, create documents, assert actual marker/registry propagation and unchanged stored sentinel, and reject stale actual `D`, `fixture_path`, `fixture_sha256`, or retained external-manifest SHA-256 without silent update. Submit `manifest_sha256`, `self_sha256`, and equivalent self-hash fields and reject each as `fixture-mismatch` before creation without field-exclusion normalization. The bundled fixture remains the full caller-input source with catalog identity but no `D`/`Z`; the external manifest contains exactly actual `D`, `fixture_path`, and `fixture_sha256`, while separate S-12/S-16 evidence computes and retains its SHA-256. | DTL-008, DTL-011, DTL-012 / coverage, surface, compatibility | DOC-11, DOC-12, DOC-14 |
| S-13 | Reproduce v1 digest `1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6`; verify v1 has no examples/sentinel and unchanged marker/registry behavior, with no migration; derived 1.1 profiles use the new rule. | DTL-012, DTL-013 / compatibility | DOC-13 |
| S-14 | Compare CLI/MCP get/catalog/template resources and pins: they use actual `D`, examples are not templates, no example bytes or sentinel become a selected contract, and S-06's test provider is unreachable through either adapter. | DTL-008 / `documentTemplateCatalogSurfaceIsShared` | DOC-12 |
| S-15 | Record later design/API/model reconciliation, comparator and sentinel validation, `test_examples.py` S-09/S-10, and catalog/security tests for isolated zero-digest injection; the correction requirements stage claims none complete. | DTL-016 / `documentTemplateEvidenceIsBound` | DOC-15 obligation |
| S-16 | Retain correction-run/revision identity, old evidence, changed files and hashes, commands/logs, bundled fixture paths/SHA-256, external acceptance-manifest SHA-256 and fixture references, SysML/docs/build/consistency results, and all 26 unevaluated rules; no old-evidence, design, product, frozen-record, or xdesign-state mutation is claimed. | DTL-016 / `documentTemplateEvidenceIsBound` | DOC-15 evidence |

S-15 separately requires later reconciliation of `docs/design/document-kit.md#d113`,
`docs/architecture/document-kit.md#b33`, `docs/interfaces/document-kit-api.md#i20`, the architecture and
detailed-design models, product validation/comparison code, full-input bundled fixtures with identity
references but no digest values, and external manifests with only actual `D` and fixture references.
`document-kit/tests/test_examples.py` owns S-09/S-10; catalog/security suites own S-06's test-only
catalog-loader/digest-strategy provider returning `Z`, descriptor and request negatives, proof that
production SHA-256 and CLI/MCP wiring are unchanged and cannot reach the stub, and S-12/S-16 manifest
hash binding. The current approved design's raw DOC-11 comparison and concurrent zero-bearing product
examples are not evidence that this correction is designed or implemented. The correction requirements
stage does not edit those design/product files or any parent evidence.

A deterministic pass establishes only the stated structural predicates. The
review must record whether the rules and templates express the intended
design-first practice and whether each sample communicates its stated engineering
claim. Scenario coverage, source suitability, and a reasoned decision not to draw
a figure remain semantic judgments. A SimpleFlow verification assessment must
cite the executed commands and final bytes at one implementation revision;
declaring these cases does not execute them. Executed case results belong to
the wish's revision-bound record.

<a id="d116"></a>

## 10. Structured document-kit verification plan

`XDesignVerification::StructuredDocumentKitVerification` links the eight R20
requirements to [SDK-01 through SDK-13](requirements.md#v19). Those cases are
planned; none is an execution result. The following implementation-facing allocation
uses isolated consumer roots, cache-local environments and no network dependency
after installation. The selected version and fixture inputs, exact paths, exit
codes, identities, before/after hashes and transcripts are retained per case.
The same fixtures drive direct Python, JSON CLI and an actual launched stdio MCP
server; a skipped MCP test cannot establish SDK-06 or SDK-13.

| Case | Concrete procedure and oracle | Intended test owner |
| --- | --- | --- |
| SDK-01 | Before model files exist, plan the two named API selections and all four `simple-flow` role selections. Materialize a parser-valid source, then create the two API pages and four role pages at the exact R20 paths with complete caller inputs, creating the landing page after its link targets. Compare each returned path/selection/identity/hash with registry and disk, exact ordered section keys/fields and caller bytes; check all pages and ordinary project context. | `test_structured.py` and `test_acceptance.py` |
| SDK-02 | Exercise same-stem pages in different layers. Update all six SDK-01 pages using current hashes; compare all unselected sections, marker and registry byte-for-byte. Attach supplied valid assets to every bundled structured role with null concern, recheck blocks/assets and identity, then independently test unknown concern and stale hash on every role. For each role, use a reviewed derived 1.2 entry with an optional concern: create a complete page with an empty decision array, attach a caller-supplied figure decision and block in one replacement, then check both; separately require refusal for stored table/prose/not-applicable decisions and mismatch or missing rationale. Report any role policy that forbids all figures. | `test_structured.py`, `test_mcp_server.py` |
| SDK-03 | Exercise the invalid name/layer/role, absolute, traversal, separator, symlink, route-overlap, registry collision and arbitrary-output-path cases in V19; test `api-design` name `index` against an existing layer landing page and a real flow index in a fresh target. Test every rejected create/update with before/after document, registry and outside-sentinel hashes. | `test_structured.py`, `test_security.py` |
| SDK-04 | Remove or reorder required sections/fields, introduce placeholders, duplicate `U` anchors, break a source locator/link and malform a figure block. Assert deterministic diagnostics and no invalid create/update. Inspect a reasoned non-applicable field and diagram alternative; keep `semantic_review_required` true on a structural pass. | `test_structured.py`, `test_core.py` |
| SDK-05 | Recompute the exact original v1 and 1.1.0 digests from their separately selectable retained roots; byte-compare every original closure resource and rerun original pack tests. Under 1.2, compare all five flat routes and all 16 existing template selections/defaults/sections/fields/view decisions against 1.1, and exercise representative original-shaped calls. Validate all 21 1.2 example/fixture packages and the selected 1.2 digest; mutation and identity negatives below must fail closed. | `test_compatibility.py`, `test_catalog.py`, `test_examples.py` |
| SDK-06 | Start a real stdio MCP process with project and 1.2 pack pinned. Against equivalent fresh roots call Python, CLI and MCP get/plan/create/whole and selected check/update/attach, and compare I21 result fields, content bytes, effective identities and exception/diagnostic codes. Read both template-resource forms and catalog, try root redirection, stale inputs and changed selected/child bytes, and retain actual transcripts and CLI JSON. | `test_mcp_server.py`, `test_structured.py` |
| SDK-07 | Inspect the operation/type API examples and all four flow-role templates/inputs for the D115 fields, caller-supplied realization or language-side reason, source projection, view decisions and no invented model/code/result. Confirm plan works without a model, whereas create/check require the stated sources. This inspection is not satisfied by a parser alone. | `test_structured.py` and permitted reviewer |
| SDK-08 | Copy the complete distribution to a second project, install it in that project's `cache/`, and follow its distributed `API.md` and README for plan/create/check/update using Python, CLI and MCP. Check that the example requires a real source and current hash, and that selected versus whole scope, release identities, access errors and incomplete publication are documented without an xdesign-only path. | `test_acceptance.py`, `test_mcp_server.py` |
| SDK-09 | Retain source revision, selected/effective digests, full inputs and fixture paths/hashes, external acceptance-manifest hash, generated output/registry hashes, all command exits, skips, diagnostics, unevaluated rules and reviewer authority. A permitted reviewer judges API/flow semantics, model and source alignment, attribution, links and any final figures; the checker result remains separate. | acceptance evidence and permitted reviewer |
| SDK-10 | Plan and check an ordinary unmanaged destination with a valid model, then attempt create/update/attach and selected managed check. Require correct existence/hash, ordinary checks, collision/refusal and unchanged page/registry; separately test a marker without registry. | `test_structured.py`, `test_security.py` |
| SDK-11 | In one root create v1, 1.1.0 and 1.2.0 pages using their own packs. Under 1.2, assert whole-project pass, exact per-path effective identities and individual selected checks; inject a duplicate anchor involving a selected page. Update both old pages under 1.2 and their original packs, comparing old markers and registry bytes. Original whole-project checks may report a foreign entry, but their updates must still load the registry. Verify the new entry uses only the original allowed registry keys and digest-version `"2"`; reject unknown/forged identities without migration. | `test_compatibility.py`, `test_structured.py`, `test_mcp_server.py` |
| SDK-12 | For each API/flow role require plan's four metadata names and create's individual valid model-source list. Exercise missing/empty/malformed metadata, duplicates, wrong suffix, unsafe, outside-root, nonexistent, unreadable and symlinked sources, including loss after creation; preserve existing bytes on rejection. Test language-side rationale with a real context source. | `test_structured.py`, `test_security.py` |
| SDK-13 | Deny model/pack/document/registry reads and route/lock/stage/page/registry writes in each public adapter, using a restricted process or an internal test-only filesystem fault seam. Assert I21 diagnostic codes/outcome phase, unchanged pre-existing files for prepublication denials, and no raw exception. Force registry publication failure with successful cleanup and with failed cleanup, inspect the latter as a reported orphan, then inject post-publication uncertainty and inspect without inferring rollback. The fault seam cannot be selected through profile, CLI or MCP production inputs. | `test_security.py`, `test_structured.py`, `test_mcp_server.py` |

### 10.1 Release and sentinel fault matrix

Before any target write, the 1.2 loader validates its own closure and recursively
loads each child at its own root. Test exact raw bytes and both fixed original
digests, v2 byte framing and ordering, duplicate paths/roles, unsafe or cyclic
child roots, duplicate descriptors, child identity/digest/version drift, and MCP
pin drift. Compare the root-level v1 copy with the nested 1.1 copy by relative path
and bytes, retain the original root-level v1 selection path, and reject any other
duplicate identity or differing copy. Registry fixtures must include a 1.2 entry using only the seven existing
allowed keys and digest version `"2"`, then prove original 1.0 and 1.1.0 updates
still work and original whole-project mismatch is preserved. A synthetic digest
version `"3"` must be rejected by the original readers; the design never emits it.

For each selected 1.2 example, require one zero sentinel in its exact first marker
contract span and nowhere else in its owning example; its actual digest is absent.
For its own non-example resources reject zero sentinels, except a parsed zero
compatibility descriptor is classified as `contract-drift` when the child digest
is nonzero. In the retained 1.1 closure permit only the child's 16 independently
validated example sentinels; v1 has none. Mutate each ownership domain, including
an undeclared extra zero in a child guide, a duplicate or malformed child marker,
a zero actual digest and a zero target marker/registry. Recompute/hash raw bytes
before descriptor comparison: a zero recomputation is `invalid-catalog`, a zero
expected request is `invalid-request`, and a zero/mismatched target identity is
`registry-mismatch`. No digest-time masking is permitted; descriptor-span handling
only selects the diagnostic. Re-run DOC-08 through DOC-14 and S-01 through S-14
against the original 1.1 selected root under its original rules.

For each 1.2 fixture, materialize the external three-field manifest outside the
pack, independently assert its hash and the created marker/registry identity,
replace only the parsed stored marker's sentinel field in memory, and compare all
created bytes. The fixture contains full caller input plus `layer`, never actual
or sentinel digest; the external manifest contains no self hash. Mutated example,
fixture, manifest or retained manifest-hash evidence must fail stale or
`fixture-mismatch` as applicable without rewriting any target or acceptance
resource. No fixture comparator or test fault provider is exposed through product
inputs, CLI or MCP.

### 10.2 Scope, compatibility and failure-order regressions

The following cases refine the existing SDK allocations; they are implementation
oracles, not executed results. Preserve original selected-pack regressions alongside
the new behavior; changes to schema 1.2 must not silently change the old loaders.

| Cases | Additional inputs and required observation |
| --- | --- |
| SDK-01, SDK-04, SDK-07 | Check natural headings for the flow index and API pages, decimal headings for the three formal flow roles, and per-page metadata. Submit a `purpose.fields.layer` different from the resolved API layer at create/update and tamper it before check; require `invalid-request`, preserving bytes for rejected writes. Do not infer API identity or engineering meaning from the filename. |
| SDK-03, SDK-05, SDK-11 | Bind a child with a different docs root, changed route for a shared class, cross-class route overlap, unsupported legacy route grammar, or conflicting digest for the same profile/version. Require `invalid-profile` after successful child/descriptor validation, with no project write. A wrong child descriptor remains `contract-drift`. Confirm identical bundled v1 copies remain the one allowed duplicate and both original selectable roots retain their digests. |
| SDK-04, SDK-06, SDK-11 | Class-filter each flow role and API pages across two layers: require exact `checked_paths`, including unmanaged full-route matches, excluding sibling roles. An off-route registered page in that class still receives its route diagnostic. Preserve global anchor, registry-integrity and model-locator checks in class mode; retain original selected-pack filter behavior. |
| SDK-04, SDK-11 | Put a selected-page anchor collision in lexically earlier and later unmanaged files, then duplicate within the selected file; require a collision in all three cases. Fenced anchors do not count; collisions solely between unrelated files do not fail selected mode. Local fragment links must resolve in the named target file, not another file with the same ID. Unreadable context prevents a successful selected report. |
| SDK-04, SDK-12 | Create against a valid model annotation targeting the candidate's not-yet-published path; then break that annotation's section/figure and require prepublication refusal. Check selected mode with an unrelated well-formed locator failure versus a malformed locator with unknown target; only the latter is an unavoidable context finding. Exercise differing compatible model roots and each effective pack's own source prerequisites. |
| SDK-02 | Clear an optional non-figure decision and then attach a supplied figure decision/block successfully. Refuse clearing or changing that figure decision when its retained block would no longer match; allow a rationale-only edit whose block still matches. Refuse `decision` for retained 1.0/1.1 effective pages even when 1.2 is selected. Compare registry and unselected sections after every case. |
| SDK-03, SDK-06, SDK-13 | Across adapters, combine malformed name/expected digest/figure shape with pack drift, and combine stale document hash with unknown concern, prohibited decision or empty caption. Assert the I21 precedence and complete matching error envelope. CLI extra `output_path`/root/payload keys must be rejected before publication. Distinguish readable-page check findings from aborted unreadable/malformed-registry scans; MCP must preserve `outcome` for access and publication faults. |

### 10.3 Evidence and semantic boundary

The compiler parse/check, documentation audit, strict build, full kit regression
suite, 21 isolated examples, a mixed structured project, real adapter transcripts,
and denied-I/O tests are all separately recorded. Preserve every compiler-reported
unevaluated rule and any skipped or unavailable case. A permitted reviewer assesses
the design claims, appropriate not-applicable explanations, source/model accuracy,
view choices, final figure readability and source/license adequacy at the exact
assessed revision. Neither the model's `verify` relation nor a structural pass
records that judgment or executes engineering acceptance.

<a id="d117"></a>

## 11. Linked-worktree store location and migration acceptance

`XDesignVerification::WorktreeStoreMigrationVerification` traces to the eight proposed
[location and migration requirements](../requirements/requirements.md#r21). The following
are planned implementation-verification cases; declaring them or checking the model does
not execute them. Use isolated fixtures under the project `cache/`, including a controlled
home directory, distinct Git repositories, a primary checkout, linked worktrees, and
actual old-version format stores. Capture source and destination inventories, root/store
identities, exact command lines and responses, per-case assertions and hashes. Never use
the operator's actual home or operational store as a fixture.

| Case | Setup and operation | Required observable result |
| --- | --- | --- |
| WTSTORE-01 Context selection | With no overrides, compare an ordinary non-Git project, a primary checkout and linked Git worktrees; exercise linked-worktree metadata rather than merely a `.git` filename, and a selected project nested within a checkout. Restart processes; separately make home or repository/worktree identity unavailable or ambiguous. | Ordinary projects and primary checkouts use their project-local default. Each linked-worktree project selects a stable dedicated root below the controlled `~/.xdesign`, and no local store is created by new default registration. Ambiguous detection or unavailable home fails explicitly, without choosing a different project or creating a local fallback. |
| WTSTORE-02 Identity and isolation | Use two same-basename repositories, two linked worktrees of one repository, distinct selected project roots in one checkout, equal wish IDs, and a symlink alias of the same selected root; repeat invocations and deliberately inject a destination-key/binding collision through a test seam. Exercise `git worktree move`, a removed and recreated worktree or repository at a reused path, and copied or forged worktree registration and binding metadata, including same-looking names and forced identity-reuse conditions. Check each documented explicit-root inspection or recovery path. | Each distinct project/worktree has a separate binding, store and history; a canonical alias selects the same binding. Colliding, copied, forged, recreated, mismatched or unprovable identities refuse adoption without changing either store. Relocation never silently adopts a different history; diagnostics identify the candidate roots, failed condition and safe next action, and the documented stability boundary and explicit selection/recovery remain usable where applicable. No path, branch or device/inode alone proves incarnation continuity. |
| WTSTORE-03 Global container | Prepopulate the controlled `~/.xdesign` with unrelated files and a separate valid store, snapshot their bytes, modes and root identity, and initialize another worktree. Separately make `~/.xdesign` itself an initialized store root. | Only the allocated namespace and store are created; unrelated content, permissions and the separate store remain usable and unchanged. An existing store used as the container is diagnosed and no child is created within it. A chosen conflicting/foreign child is never adopted. |
| WTSTORE-04 Root overrides and Web delegation | For ordinary and linked projects exercise absent, empty, relative and absolute `XDESIGN_STORE_ROOT`, explicit store CLI `--root`, direct store API root, and Web `--project-root`/`--store-root`. Give Web an explicit root different from the environment and default, then create, read, `next` and `check` through actual lifecycle subprocesses. | Each surface follows its stated precedence and relative-path base, and an empty explicit selection is refused. API calls use their supplied root. Web health, reads and all delegated writes refer to the same explicit root; neither the environment-selected nor default store is modified. Explicit roots do not trigger implicit migration. |
| WTSTORE-05 Complete history | Prepare an old local version-1 store with generic non-wish items, several revisions and manifests, shared and unique materials, active and completed wishes, committed and conflict receipts with exact request/result bytes; run supported explicit migration to an otherwise eligible destination. Compare source and target using independent enumeration and byte verification at every revision and inspect the identity mapping. | Every item, kind, head, revision, metadata, URI, role, length, digest, material byte, exact request, terminal outcome and result matches; store identity is preserved or the mapping is visible and unambiguous. No new judgment, item revision or receipt is invented, and the source is still recoverable. Raw copy/move alone remains refused by existing physical-identity checks. |
| WTSTORE-06 Lifecycle continuity | Migrate SimpleFlow and WishVerifyFlow examples containing active, blocked and completed runs, with retained checker bundles, evidence and a prepared result; compare work orders and tokens before/after. Delete only regenerable persistent cache and retry a retained terminal operation, then continue valid and stale checks and query completed runs. | Identities, plan/work, visits, revisions, assessments, artifacts, evidence, status and unchanged-head tokens agree. Valid active work continues; stale or drifted work is rejected under existing guards. Completed next/check remain read-only; exact retry has one effect and the original receipt. Cache recovery reproduces verified bytes without overwriting changed workspace content. |
| WTSTORE-07 Conflicting histories | Exercise a local source with no global destination, two independently initialized valid roots (including matching IDs/heads but different earlier data), and a proven migration with unchanged source and later target progress. Modify or recreate the old source after cutover using an old client while the new client is stopped; separately remove the retained source. Inspect both with explicit roots. | Before migration the default diagnoses the old source instead of making an empty global store. Independent roots, unclassifiable local candidates and changed/reappearing unproven source block default mutation without merge or overwrite. After cutover upgraded clients also refuse explicit writes to the retained source while still allowing inspection. Proven unchanged source plus advanced target remains usable; verified cutover remains usable if the old source is absent. Explicit inspection selects the requested root without altering either history. |
| WTSTORE-08 Legacy registrations | Keep genuine legacy active and completed cache registrations and Git traces beside store-backed runs. Remove only regenerable persistent cache, migrate the persistent local store, and exercise CLI and Web. Create a separate fixture with a genuine legacy registration and persistent item sharing an ID. | Legacy history/bytes remain on their original backend, active runs remain operable and completed runs remain completed; persistent cache is distinguished from genuine legacy authority and can be reconstructed. The shared-ID conflict is surfaced and mutation of either history is refused without loss or silent masking. |
| WTSTORE-09 Refusal and preconditions | Try missing/foreign/newer/corrupt sources, missing or changed referenced blobs, unsafe ownership/modes or symlinks, an occupied target, unreadable identity, unresolved pending operations, active/stale locks and a conflicting migration. Reconcile a pending receipt with the original store's supported exact-operation procedure and retry. | Unsafe or unverified inputs fail explicitly before cutover or source/history mutation; no foreign content is adopted. Diagnostics identify the obstacle and recovery action. Only after pending operations and locks are safely resolved and required quiescence is established may migration continue. |
| WTSTORE-10 Concurrency | Race two migration attempts and race supported upgraded lifecycle/store/Web writers and readers with migration at controlled barriers before source enumeration, verification and cutover. Hold direct source and target `RsStore` handles opened before migration across those barriers; attempt save, read and init via direct API and absolute, relative, canonical and symlink root aliases, including after the decision. Separately demonstrate the operator quiescence procedure for a non-cooperating older client. | At most one verified cutover occurs; upgraded access serializes or refuses with a retryable diagnostic, no source write commits after the verified snapshot while it is relied on or while cutover stands, and the retained source history remains unchanged. If preparation is safely rolled back before the decision, source writes resume only after invalidating the prepared snapshot. Reads return a consistent allowed snapshot or retryable refusal; no reader treats a partial destination as authoritative, and no acknowledged update disappears. Held handles and aliases cannot bypass the fence. The evidence states that older clients do not honor the new fence and must remain stopped; the test does not claim to fence them. |
| WTSTORE-11 Interrupted before cutover | Inject process termination after each durable preparation boundary, during material transfer, after destination verification and immediately before the durable decision; restart each time, query migration status, then perform exact retry or pre-decision rollback as documented. | The source remains recoverable and its recorded history is unchanged; default operations do not create or use a partial global store. Recovery identifies the same attempt and either safely resumes to one verified cutover or abandons it without altering authority or unrelated data. |
| WTSTORE-12 Interrupted after cutover | Terminate after the durable decision but before the success response, and after response with possible temporary residue; restart, query status, retry the identical migration and operate on the resulting store. Compare recorded source baseline to subsequent source changes. | A verified destination is the sole default authority; retry returns the same decision and does not create another destination, revision or receipt. Residue is reported or safely handled under the documented procedure. A changed retained source blocks default mutation instead of being silently merged. |
| WTSTORE-13 Regression and evidence | Run the relevant existing storage, lifecycle, Web, model and documentation suites plus WTSTORE-01 through WTSTORE-12 against the final implementation revision. Retain logs, exit codes, hashes, case results, diagnostic/unevaluated rules, independent semantic review and limits. | Existing supported root, integrity, save, retry and lifecycle behavior remains covered; every claimed case has an observed result. Any failure or unrun case is disclosed, and simulated process faults are not presented as physical power-loss certification. |

Before testing cutover, the old client must be stopped and pending exact operations
reconciled. The current per-item locks do not constitute a whole-store freeze, and an older
binary cannot be assumed to honor a future migration fence. Tests of cooperative processes
and injected crashes establish only the tested filesystem and implementation; arbitrary
uncooperative writers, whole-store backup/restore, and physical power-loss guarantees need
separate support. Source retention is not permission to use old and new roots as parallel
write authorities. Results and semantic judgments belong to the wish's revision-bound
operational evidence, not to this case declaration.

Detailed, unexecuted procedures for every row, including the complete operation/alias matrix,
are in [worktree store verification procedures](worktree-store.md#d120).
