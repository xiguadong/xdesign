# Stage CLI implementation plan

> Registered: 2026-10-22. Status: implementation and final technical acceptance run
> completed; final human acceptance remains pending.
> Wish: `wish-0010`, selecting `xdesign.designedSimpleFlow`.
> Scope: the flat local CLI for the existing four-stage SimpleFlow and two-stage
> WishVerifyFlow slices. This does not implement the default V-model, nested
> controller execution, multi-user persistence, or separate final human acceptance.

<a id="p30"></a>

## Task and acceptance goal

Implement the agreed agent interaction as one repeating pair of commands:
`xdesign next WISH_ID` returns the current work order without changing the run;
after the agent performs that work, `xdesign check WISH_ID [--result FILE]`
collects the current artifacts, records the work, evaluates the actual flow gates,
and returns correction work, the successor work order, or completion. Registration
uses `xdesign init WISH_ID --flow simple-flow|wish-verify` once outside that loop.
It uses an existing `mbse/wishes/WISH_ID.sysml` and retains a frozen plan, work
configuration, and run state under `cache/xdesign/WISH_ID/`.

The implementation must resume from a validated local record in a later process,
bind supplied results to the work order they answer, retain failed or unevaluated
assessments, and never invent a semantic pass. The reusable requirements are
`stageCliUsesUniformInteraction` and `stageCliPreservesRunIntegrity`; their source
is `mbse/model/requirements.sysml`. The [runtime contract](../../design/stage-cli.md#d85)
owns command behavior and result fields. The wish's independent trace is
`mbse/wishes/wish-0010.sysml`; [wish history](../wishes/wish-history.md#v17)
records its actual status.

<a id="p31"></a>

## Basis and execution sequence

Read the existing [SimpleFlow plan](simple-flow.md#p04), its
[source-file plan](simple-flow-file-plan.md#d53), the
[fixed gates](../../design/flows/simple-flow/definition.md#d05),
[evidence and revision contract](../../design/evidence-and-revisions.md#e03), and
[API design conventions](../../design/api/index.md#u47) before changing the
corresponding artifact. This wish reuses the four implemented fixed checkers;
it does not declare a new checker or reinterpret their rule sets.

| Stage | Work to perform | Actual exit gate and required evidence |
| --- | --- | --- |
| requirements | State the uniform interaction and persisted integrity requirements, rationales, acceptance cases, and wish derivation links. | `simpleRequirementsAccepted`: run `simpleRequirementsCheck` with its two declared rules on the frozen requirements and traceability inputs. A compiler pass alone is insufficient. |
| design | Reconcile the model, command/result/state contracts, restore API, source ownership, stale-result handling, and documentation. | `simpleDesignAccepted`: run `simpleDesignCheck` with its seven declared rules and all enumerated design inputs. Review CLI behavior separately because these rules chiefly establish the existing SimpleFlow design structure. |
| implement | Add flat command dispatch, local initialization and resume, work-order output, collection and checking, explicit semantic result handling, and behavioral tests. | `simpleImplementationAccepted`: run `simpleImplementationCheck` with its two declared rules on the collected source artifacts. Executed tests are retained at verification; artifact presence does not establish correct behavior. |
| verify | Exercise subprocess restarts, both flows, correction and refusal paths; inspect real gate reports, model checks, documentation, and retained evidence. | `simpleRecordAccepted`: all seven record rules; then `verificationAccepted`: a permitted agent or human reads the requirements and executed evidence and supplies an explicit judgment, identity, and rationale. |

Requirements and design preparation checks are retained with their actual timestamps.
The implemented CLI then performs a **final acceptance run** over the final artifact
set. Its four gate timestamps prove that final run, not that the new CLI authorized
earlier implementation work. Keep this distinction in the execution record and the
wish model. Do not pre-fill passed assessments while writing the plan.

The initial trace stays active until there is evidence for advancement. Copy a
runtime assessment into the SysML wish trace only with the same stage, visit,
artifact revision, authority, outcome, rationale, and re-openable evidence. Every
visited stage retains an activity record and its actual assessments. Pending
records do not count as completed engineering work.

<a id="p32"></a>

## Source ownership and snapshot boundaries

| Area | Files or owning location | Responsibility |
| --- | --- | --- |
| Requirements, design, and verification | `mbse/model/requirements.sysml`, `traceability.sysml`, `workflow.sysml`, `uml.sysml`, `verification.sysml`; owning requirements/design/API pages | Keep reusable behavior and acceptance claims in the project model and documentation. |
| Command adapter | `src/main.rs`, `src/stage_cli/mod.rs`, `src/stage_cli/work.rs`, `src/stage_cli/store.rs`, `tests/stage_cli.rs` | Parse the flat interface, initialize and load state, emit work orders, accept result files, call existing flow factories and gate engine. |
| Run restoration | `src/core/restore.rs`, `src/core/record.rs`, `src/core/mod.rs`, `tests/run_restore.rs`; [restore API](../../design/api/core/restore-run.md#u78) | Strictly deserialize and validate the retained plan, visit history, replayed revision snapshots, assessment ownership, and completed-stage authorization before restoring executable state. |
| Existing flow adapters | `src/simple_flow/`, `src/wish_verify/`, `tools/checks/` | Reuse the declared plans, fixed checker invocation, and permitted semantic boundary without silently changing gates. |
| Usage | `docs/README.md`, root `README.md`, interface and runtime-design pages | Give runnable commands and honest completion, failure, restart, and evidence limits. |
| Trace and evidence | `mbse/wishes/wish-0010.sysml`, this plan, wish history, `cache/evidence/wish-0010/`, `cache/xdesign/wish-0010/` | Retain actual execution history and distinguish planned coverage from observed results. |

The implementation stage collects `Cargo.toml` and `src/**/*.rs`; executed test
results are retained as verification evidence rather than implying the implementation
presence gate ran the tests.

Before final gate execution, freeze the complete relevant artifact set and regenerate
any required figures. Completed SimpleFlow stages remain authorized only while their
recorded artifact bytes still match their hashes. The requirements checker lists its
inputs; the design checker enumerates its model, linked documents, figures, and checker
commands through `--list-inputs`. Include those exact inputs rather than a handpicked
subset. Later edits to a completed stage's artifacts require an explicit new run or
supported rework; do not replace historical digests to preserve a pass.

Do not include the mutable `run.json` in its own artifact snapshot: this would make
recording the snapshot invalidate it. The checker invocation authenticates its input
record separately and records that byte hash in the report. Likewise, keep the live
wish trace outside completed stage artifact snapshots while adding its later
assessments. Retain immutable checker-input record copies where needed to re-examine
the exact bytes a report judged. Neither a snapshot copy nor a cache path establishes
durable archival storage; that remains outside this local slice.

<a id="p33"></a>

## Planned acceptance cases

All rows below are planned until their command, input, expected result, actual result,
and retained evidence are entered in the completion record.

| Case | Scenario and observable acceptance condition |
| --- | --- |
| CLI-01 | Initialize each supported flow from an existing wish. Reject an unknown wish, unsupported flow, malformed arguments, or attempt to overwrite an existing run without changing prior state. |
| CLI-02 | Run `next` repeatedly and in a fresh process. Receive the same current stage, visit, revision, work instructions, expected artifacts, and gate criteria; every state file remains byte-identical. |
| CLI-03 | Complete the real SimpleFlow requirements → design → implement → verify path. Four fixed reports contain exactly 2, 7, 2, and 7 rule results; the last stage also retains the explicit semantic judgment. Every successor response identifies the newly current stage. |
| CLI-04 | Complete the real WishVerifyFlow wish → verify path with explicit stage-specific semantic results and no syntactic checker invocation. The final work order declares completion and the preserved run remains inspectable. |
| CLI-05 | Omit, fail, or decline the semantic judgment. The stage remains blocked or unevaluated; no default pass or inferred authority appears. A valid correction retains the previous assessment and can resume the same visit. |
| CLI-06 | Supply a result for a different wish, plan, stage, visit, revision, or prior work-order state. Refuse it before mutating the run; retain the authentic current work order. |
| CLI-07 | Restart between stages, after blocked checking, and after completion. Restore the same plan identity, current visit, activities, assessments, and completion state; reject malformed records and unauthorized or inconsistent completed visits. |
| CLI-08 | Change frozen plan/work bytes or corrupt the record and require refusal. Change a completed stage artifact or required evidence before a later advance and require that advancement remain blocked. `next` permits inspection of workspace edits without silently repairing them; after completion, `next` and `check` return recorded completion without writes or a new live-artifact assessment. That response is a historical result, not evidence that the current workspace still satisfies every completed snapshot. |
| CLI-09 | Exercise both default artifact collection and an explicit result file. Missing expected outputs, unreadable files, duplicate artifacts, and malformed or unauthorized judge data cannot create a pass. |
| CLI-10 | Run the documented quickstart as separate CLI processes, inspect JSON responses and exit codes, and compare resulting records with the model and runtime/API contracts. Record the limits of local single-writer persistence and external-project support. |

Existing core, flow, and checker cases remain regression coverage. Run the checks
actually required by the changed source and model: Rust tests; Python SimpleFlow
checker tests; pinned model parse and model-only/full checks; documentation audit and
strict build; renderer tests and regenerated-figure inspection when affected. Keep
reports under `cache/evidence/wish-0010/` with the tested Git base plus a manifest of
uncommitted source hashes. Compiler unevaluated rules and independently reproduced
pre-existing failures remain explicit limits rather than being relabeled as passes.

<a id="p34"></a>

## Execution and acceptance record

Registration created the independent active trace on 2026-10-22. Requirements and
design preparation preceded implementation. The completed CLI then executed a
**final acceptance run after implementation**, from
`2026-10-22T13:15:53.082922Z` to `2026-10-22T13:15:54.984763Z` with run ID
`6d35a14e-9230-4cf9-a14e-d699ed72a2ea`. These timestamps do not establish that the
new CLI authorized earlier development. The existing model calls root-flow
completion `WishStatus::delivered`; it is a technical completion state here.
**Final human acceptance has not been given.**

The final run's evidence directory is
`cache/evidence/wish-0010/final-cli-20261022T131553082464Z/`. Its `manifest.json`
retains all 16 command events, raw stdout/stderr, exit codes, JSON responses,
binary and report hashes, and 11 successful read-only state comparisons.
`completed-run.json` preserves the completed runtime history; the live copy is
`cache/xdesign/wish-0010/run.json`. The four visits all have `visit_index = 0`
and `artifact_revision = 1`:

| Stage | Recorded activity time (UTC) | Artifacts | Actual gate outcome |
| --- | --- | --- | --- |
| requirements | 2026-10-22T13:15:53Z | 2 | `simpleRequirementsAccepted` passed; 2/2 fixed rules |
| design | 2026-10-22T13:15:53Z | 52 | `simpleDesignAccepted` passed; 7/7 fixed rules |
| implement | 2026-10-22T13:15:54Z | 23 | `simpleImplementationAccepted` passed; 2/2 fixed rules |
| verify | 2026-10-22T13:15:54Z | 1 | `simpleRecordAccepted` passed, 7/7 fixed rules; `verificationAccepted` passed by agent `codex-root-cli-review` |

The five assessments retain their exact runtime authority, assessor, rationale,
revision, and evidence URIs in `mbse/wishes/wish-0010.sysml`. The JSON remains the
source for executable-only snapshot fields. The harness rehashed every report's
declared input and checked the exact 2/7/2/7 rule lists before recording success;
the final 78 stage-artifact entries also retain their file digests. The semantic
assessment cites `cache/evidence/wish-0010/verification-summary.json`, SHA-256
`9852bf657d27b80da4c1daa3b6244c3faea948c293f3ad7a1cd1d0a960b63283`, and the
verify-stage record-check report. It was explicitly supplied after reading the
executed evidence; the fixed checks did not manufacture it.

The run read the initial wish model with SHA-256
`10fe7cd7cfd9fc0757f19859bf512d2e7ed5516d4bc5505fc2a48129413c68f7`.
Its exact bytes remain in `wish-source.sysml` beside the final manifest. The later
SysML history update is outside the completed artifact sets and does not replace
that historical checker input. Every fixed report points to its own immutable
`.record.json` checker input under `cache/xdesign/wish-0010/reports/`.

### Executed verification and source identity

The tested Git base is `2dbcfead06c736636b736d716936bd1df7427992`; uncommitted
source and document hashes are retained in
`cache/evidence/wish-0010/tested-source-manifest.json`. The tested binary has
SHA-256 `293b2977f1446fa2ad4ef4d8001be89fd1e332b02140563cfb7e4b688746ea5c`.
The frozen verification summary binds the following logs by hash. Later wish and
record reconciliation is separately checked and is not silently folded into that
tested manifest.

| Executed check | Observed result | Evidence under `cache/evidence/wish-0010/` |
| --- | --- | --- |
| Rust suite | 53 passed: 2 binary, 4 plan/model sync, 4 process-check contract, 11 restore, 14 SimpleFlow, 10 CLI process, 8 WishVerifyFlow | `cargo-tests.log` |
| Python SimpleFlow checker suite | 22 passed | `python-checker-tests.log` |
| Pinned SysML parse and model-only/full checks | Parse errors 0; checks passed, 129 held and 26 unevaluated rules. Before record reconciliation, full references resolved 21980/21980 and model-only 20312/20312. | `parse.json`, `check.json`, `model-only-check.json` |
| Documentation audit and strict build | Passed | `docs-audit.json`, `docs-build.log` |
| Renderer suite | 66 passed, 2 failed existing edge-count assertions | `renderer-tests.log`, `baseline-renderer-review.json` |
| CLI figure tests after caption refresh | 4 passed | `cli-figure-tests.log` |
| Figure verification and review | Nine primary figure checks passed; five changed previews reviewed | `figures/figure-manifest.json`, `changed-figure-previews/manifest.json` |

The two renderer failures expect six edges in each of the requirement and state
views; the unchanged assets contain five and nine respectively. The relevant
assets, assertions, and rendering functions are byte-identical to the tested Git
base. This classification is a byte comparison, not a separate clean-checkout
suite execution; neither failure is relabeled as a pass. Compiler unevaluated
rules remain unevaluated and do not establish engineering acceptance.

### Acceptance-case coverage

The named process cases below live in `tests/stage_cli.rs`; the named restoration
cases live in `tests/run_restore.rs`. Their passing executions are in
`cargo-tests.log`. The final acceptance harness adds the real wish-0010 evidence
described above.

| Case | Executed coverage and observed result |
| --- | --- |
| CLI-01 | `strict_arguments_unknown_wishes_and_exclusive_initialization` passed; both flow initializations ran through process tests, and the real SimpleFlow final run initialized exclusively. Invalid arguments and repeated initialization left prior state unchanged. |
| CLI-02 | `process_restart_loop_preserves_assessed_visit_and_read_only_completion` and `restores_without_writing_or_invoking_boundaries` passed. The final harness confirmed identical repeated work orders and 11 read-only state comparisons. |
| CLI-03 | The real final run passed all four fixed gates with exact 2/7/2/7 reports and the explicit terminal semantic judgment. `real_simple_flow_requirements_checker_retains_immutable_input` and `simple_flow_discovers_and_removes_sources_then_runs_both_terminal_gates` also passed. |
| CLI-04 | `process_restart_loop_preserves_assessed_visit_and_read_only_completion` completed WishVerifyFlow through separate processes, preserving its distinct wish and verify assessments; the eight WishVerifyFlow library cases also passed. |
| CLI-05 | The process loop retained missing, failed, and corrected semantic outcomes without an inferred pass; `refused_or_unpermitted_judge_records_remain_restorable` passed. Unchanged retries preserved the revision, while changed evidence advanced it. |
| CLI-06 | `stale_envelopes_and_changed_judgment_bytes_cannot_mutate_state` passed for an old state token and changed judgment bytes. Restore cases rejected foreign identities and stale/foreign assessments. Each individual result-envelope identity field was not separately mutated in a process test; its tuple comparison was inspected in the implementation review. |
| CLI-07 | The process restart loop and all 11 restoration cases passed, covering current edits, terminal reopens, replayed activity history, malformed schemas, foreign assessments, and forged completion. Terminal reopening is a core API case; the public CLI exposes no reopen command. |
| CLI-08 | `frozen_file_drift_corrupt_records_and_busy_writers_do_not_get_repaired`, `completed_artifact_drift_is_inspectable_but_cannot_authorize_progress`, and the SimpleFlow source-discovery process case passed. The core case `missing_passed_evidence_blocks_current_and_completed_visits` passed. Completed CLI inspection remained read-only and reported the retained result. |
| CLI-09 | Missing supplemental artifacts, malformed result fields, invalid digests, unauthorized script judgment, path escape, symlink escape, and runtime-state aliases were rejected in the process tests. Automatic source additions/removals and result-supplied evidence ran successfully. Duplicate supplemental entries and OS permission-denied files were not separate process fixtures. |
| CLI-10 | The documented flat command exchange ran in separate processes in the final harness and the WishVerifyFlow process loop; JSON/exit-code behavior matched the contract. `simultaneous_check_processes_cannot_overwrite_each_other` passed using an overlapping writer and controlled checker barrier. No separate manual copy-and-paste walkthrough or external-project deployment is claimed. |

The accepted scope remains the two built-in repository-bound flat plans with local
single-writer state. It excludes arbitrary flows, nested controller execution,
durable archival storage, remote judge authentication, and final human acceptance.
The local record validator checks consistency rather than authenticating its author.
