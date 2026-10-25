# Storage redesign and implementation plan

> Registered: 2026-10-24. Status: final durable SimpleFlow technical run completed at store revision 5 in `.xdesign-final`; final human acceptance pending.
> Wish: `wish-0011`, selecting `xdesign.designedSimpleFlow`.
> Scope: durable local operational storage and integration with the two existing flat flows.

<a id="p35"></a>

## Task and selected boundary

Build a generic item-and-revision store in Rust. One item represents one wish; `kind`
leaves room for other operational records. Git remains the authority for engineering
models, requirements, designs, source, and verification definitions. The store becomes
the authority for new wish registrations, run facts, immutable revision manifests,
material and evidence bytes, and operation receipts. Existing Git wish traces remain
legacy history and are not silently migrated.

The default root is `.xdesign/`, with `XDESIGN_STORE_ROOT` as an explicit override.
SQLite stores metadata and SHA-256-addressed blobs store exact bytes. Complete revisions
support list/get/history, verified read and checkout, expected-head conflict refusal,
exact-request idempotent retry, pending/outcome recovery, and integrity inspection.
`cache/` becomes regenerable working material. New stage runs may import a wish source
outside `mbse/wishes/`; their operational trace need not become a Git SysML file.

This slice excludes whole-store snapshot/backup, baselines, deletion and GC, automatic
legacy migration, remote service or authentication, encryption, model-claim audit,
arbitrary or nested flows, and Windows-specific guarantees. The draft proposal and prior
ProjectStore are design inputs, not executed evidence or accepted scope.

<a id="p36"></a>

## SimpleFlow sequence and gates

| Stage | Work product | Actual gate and separate review obligation |
| --- | --- | --- |
| requirements | Storage requirements, rationales, authority boundary, acceptance conditions, and wish derivation | `simpleRequirementsAccepted`: execute both declared requirements rules; separately review whether the storage wish and acceptance conditions are coherent and verifiable |
| design | `XDesignStorage`, Rust/API/interface contracts, verification cases, source-file plan, figures, and usage | `simpleDesignAccepted`: execute all seven declared design rules over frozen and supplemental inputs; separately review schema, save/recovery, compatibility, and API behavior |
| implement | SQLite store, content blobs, receipts, queries/checkout, stage adapter, command surface, and tests | `simpleImplementationAccepted`: execute both artifact rules; passing presence checks are not behavioral verification |
| verify | Fault, concurrency, restart, cache-loss, checkout/replay, two-flow, model, documentation, and regression evidence | `simpleRecordAccepted`: execute all seven record rules; `verificationAccepted`: an explicit agent judgment reads requirements and retained evidence against both semantic criteria |

The wish was registered before the storage implementation and its development runs use the
legacy backend. A separate new storage-backed SimpleFlow run is required acceptance evidence;
earlier gate timestamps must not be presented as storage-backed runs. After implementation,
executed verification, and replacement of designed-only status text with tested behavior,
a fresh final SimpleFlow run will use the delivered persistent backend to assess the final
requirements, design, source, and evidence. That final run, not the earlier timestamps, is
the technical acceptance run.

<a id="p37"></a>

## Source ownership and snapshot rules

| Area | Owning files |
| --- | --- |
| Requirements and trace | `mbse/model/requirements.sysml`, `traceability.sysml`, `verification.sysml`; requirements and verification pages |
| Storage architecture and design | `mbse/model/storage.sysml`, `architecture.sysml`, `composition.sysml`, `workflow.sysml`, `uml.sysml`; `docs/design/storage.md`; owning architecture/API/interface pages |
| Store library and commands | `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/main.rs`, `src/storage/{mod,db,blobs,ops,cli}.rs` |
| Stage integration | `src/stage_cli/{mod,store,work,persistent}.rs`; existing flow engine and checkers unchanged unless a demonstrated contract defect requires rework |
| Verification | `tests/storage.rs`, `tests/storage_cli.rs`, existing Rust/Python suites, and `cache/evidence/wish-0011/` |
| Execution history | `mbse/wishes/wish-0011.sysml`, this plan, wish history, and current status |

Do not include this mutable plan, wish history, the live wish trace, runtime run record, or
later execution summaries in completed stage artifact sets. Include newly annotated design
inputs explicitly when they were absent from registration's frozen work definition.
Changes to a completed stage's recorded bytes require correction rather than hiding drift.

<a id="p38"></a>

## Planned acceptance cases

Every row is planned until an executed command and retained result are recorded.

| Case | Observable acceptance condition |
| --- | --- |
| STORE-01 | Same-format initialization and reopen succeed; missing, foreign, truncated, or newer formats are refused without adoption or implicit migration |
| STORE-02 | Create and append generic item revisions; list/get/history/read return exact typed metadata and complete manifests |
| STORE-03 | Identical bytes deduplicate; altered, missing, truncated, traversal, absolute, backslash, NUL, and symlink inputs cannot become trusted materials |
| STORE-04 | Same operation ID and exact request returns one receipt with no second effect after reopen; changed bytes with that ID are rejected |
| STORE-05 | Two writers from one head produce one committed revision and one conflict; one logical stage check commits its run, reports, artifacts, evidence, and receipt together |
| STORE-06 | Faults after reservation/publication and around commit leave a recoverable pending or committed outcome; retry reconciles through `outcome` without duplicating effects |
| STORE-07 | `doctor` reports missing, altered, orphan, pending, and format defects; verified read and checkout reject digest mismatches |
| STORE-08 | A new SimpleFlow run and a WishVerifyFlow run persist through the store with a wish source outside `mbse/wishes/`; status follows the run record |
| STORE-09 | After deleting generated runtime cache, `next` restores the stored head and checkout recovers every manifest byte; the record checker replays against materialized evidence |
| STORE-10 | Existing Rust/Python/model/documentation suites pass for their applicable scopes; any pre-existing failure and unevaluated compiler rule remains explicitly disclosed |

<a id="p39"></a>

## Execution and acceptance record

Registration created run `a8143b34-fd78-4a9e-9749-042196aa631a`. Its requirements gate
passed at revision 1, then design preparation added storage satisfy links to
`traceability.sysml`, changing that completed visit's input. No design check was attempted
on the stale snapshot. The restart record at
`cache/evidence/wish-0011/restart-record.json` identifies the preserved run and replacement
`0d217f6f-1139-425f-bce1-c2e887ff4c05`.
The replacement requirements gate passed at revision 1 with both rules and snapshot
`bb6594ba5118b459dc025b056c0dce95c7dbbd91db0c95f98b258579b8a8ef03`.

The design gate then passed all seven rules at revision 1 with 98 recorded artifacts and
snapshot `15618f81e07c3bf0e338986bd7918074bc67103155fa3c5d5354dd1b312df96e`.
`cache/evidence/wish-0011/design-review.json` records the separate storage design review,
including the objective, canonicalization, initialization, renderer, and retry-order
corrections. A later source-plane audit found the regenerated controller figure outside
that artifact set; `cache/evidence/wish-0011/design-controller-review.json` discloses the
omission and its pre-implementation visual review. All 98 input files are also archived
and verified byte-for-byte under `cache/evidence/wish-0011/design-snapshot/`; four API
files later renamed for one-page-per-API compliance retain their exact original bytes there.
The final API set has 26 focused pages at U80–U105 with matching inventory, navigation,
model locators, strict build, and a clean diff check. Full model checks report no violations,
129 held and 26 unevaluated rules;
all 58 figures rendered, nine primary figure checks passed, and documentation audit/strict
build passed. The `sharp` module resolution failure was corrected by an absolute cached
module path and not counted as a pass. These were requirements/design results only.
At that earlier review point, implementation and STORE-01 through STORE-10 acceptance
remained unexecuted. The executed results follow; technical completion is not final human
acceptance.

### Storage-backed demonstrations

The persistent backend then completed `wish-storage-simple` at store revision 5 and
`wish-storage-verify` at revision 3. Both imported wish sources outside `mbse/wishes/`
and required no new Git trace. Exact completed replay returned the stored responses.
Cache removal followed by `next` restored their heads. Checkout verified 235 materials
for SimpleFlow and seven for WishVerifyFlow; the SimpleFlow checkout's own
`simple_record.py` and `common.py` replayed all seven record rules. See
`cache/evidence/wish-0011/storage-simple-completion.json` and
`storage-verify-checkout-verification.json`.

A separate-process writer race used one expected head. `race-a-append` committed
revision 2 and `race-b-append` retained a conflict with no revision. History contains only
revisions 1 and 2. Doctor retains the expected orphan at SHA-256
`ad0f1b89298187991b1a1947491903d67a4a992ddc2bd2f30c91f12e23b4f0a3`; this isolated
fixture is not a clean-store claim. Twenty-six CLI probes also exercised missing, foreign,
truncated, newer and partial formats, NUL/absolute/backslash/traversal paths, digest
mismatch, and pending inspection. Permanent regressions include truncated-format and NUL
cases. The ten-case matrix, exact operations, digests, raw evidence and harness corrections
are in `cache/evidence/wish-0011/storage-acceptance-summary.json`.

### Initial persistent acceptance run (superseded)

The initial persistent acceptance run was `32a3c820-8e19-4ee8-aa60-e9974cb9dca2`, using the
persistent backend and `cache/evidence/wish-0011/wish-source.sysml`. Its gates passed, but
a later full-render replay exposed nondeterministic timeline metadata ordering. It is
preserved as superseded history, not the corrected final run. The earlier blocked legacy run
was preserved with 20 digest-verified files under
`cache/evidence/wish-0011/runtime-before-final-0d217f6f/`; it was not final acceptance.

| Stage | Store revision | Operation ID | Artifact snapshot or registration |
| --- | --- | --- | --- |
| registration | 1 | `3717c655-0514-4e2d-93ee-f2cd9f49fe8a` | External wish import and frozen run |
| requirements | 2 | `wish-0011-final-requirements-001` | 3 artifacts; `190c5b9dba4bbe383e4d05467c92a19ffd1d83ae05a945af4df3a15fc7e7e848`; two rules passed |
| design | 3 | `wish-0011-final-design-001` | 133 artifacts; `2d65e0742c58073fd4bef9926fc9c3fab9dadf0b0408b6cd0498b6813b1290ef`; seven rules passed |
| implement | 4 | `wish-0011-final-implementation-001` | 42 artifacts; `131e104b843ab8a98bb2162d69815188afe32dc07058eb9c34b8c519bb75318a`; two rules passed |
| verify | 5 | `wish-0011-final-verify-001` | 92 artifacts; `28ae368cb7f280a9362018792e90f72ec39b8dad8a7ee4648ef4de8e36692337`; seven record rules and explicit agent semantic pass |

The design set explicitly includes `tools/checks/common.py`, the 26 focused storage API
pages, and `docs/design/assets/controller.svg`. The separate design review is
`cache/evidence/wish-0011/final-design-review.json`. The terminal judge is
`kimi-final-storage-verification`; its rationale and exact evidence set are in
`cache/evidence/wish-0011/final-check-verify.log`. The assessed pre-gate matrix is
`cache/evidence/wish-0011/verification-summary.json`; it remains unchanged after judgment.
`cache/evidence/wish-0011/final-acceptance-run.json` records the subsequent completion,
receipt, and recovery facts.

Exact terminal replay returned the identical response without another revision. After
removing `cache/xdesign/wish-0011` and `cache/wishes/wish-0011.sysml`, `next` recovered
the same completed head. `final-checkout` verified all 277 revision-5 material lengths
and SHA-256 digests. Its own record checker passed all seven rules at the terminal snapshot;
`final-independent-replay-invocation.json` names the exact command, checker, support module,
record, and report digests. Full-byte doctor reported no findings.

### Initial preflight results and shared limits

The initial preflight commands included the repository-standard locked/offline Rust tests,
SimpleFlow Python tests, renderer tests, documentation audit and strict build, SysML
parse/model-only/full checks, figure rendering and presentation checks, and `git diff
--check`. Logs use the `cache/evidence/wish-0011/final-preflight-` prefix. Cargo used the
cached toolchain and dependency home under `cache/` because `cargo` was not on PATH.

- 85 Rust tests and 22 SimpleFlow Python checker tests passed; the targeted storage suite
  passed 20 tests within that total.
- SysML parse had no errors; model-only and full checks had no violations, 129 held rules,
  and 26 unevaluated rules. Documentation audit had no diagnostics and strict build passed.
- All 58 figures rendered and nine primary figure checks passed. `git diff --check` passed.
- The 68-test renderer suite is **not green**: two pre-existing failures remain
  (`requirements` edges `5 != 6`, state guards `9 != 6`). The renderer source, tests and
  affected figures have no diff from `HEAD`; these failures are disclosed, not counted as
  storage regressions or passes.

Fault boundaries use injected in-process errors and process recovery, not physical
power-loss certification. Expected pending/orphan findings in isolated fixtures are not
clean-store results. Compiler unevaluated rules remain unevaluated. Whole-store
backup/snapshot/baseline, deletion/GC, automatic migration, remote service,
authentication, encryption, nested/arbitrary flows, model-claim audit and Windows-specific
guarantees remain outside scope. Final human acceptance is pending.

### Final durable acceptance after evidence corrections

A later full-render live replay failed `completed-visits-authorized` because
`cli-timeline.svg` changed after the initial design gate. Comparison found ordering-only
`source_relations` metadata: the same 36 relationships appeared in different orders because
the renderer consumed a set without canonical ordering. The failed report is preserved in
`cache/evidence/wish-0011/determinism-live-replay-failure.json`, not counted as a pass.
The correction in `tools/sysml_web/cli_views.py` sorts appended relationships by native ID.
`tools/sysml_web/tests/test_cli_views.py` adds a deterministic-order regression. Five focused
tests passed, two full 58-figure renders were byte-identical, and before/after PNG previews
received a renewed visual review with no presentation change.

Development run `3635d7c1-0466-434b-9bcf-92080edfd984` then passed its gates, recovered
330 materials, and passed checkout and live replays. Its initial acceptance root was under
`cache/`, however, so it was demoted to development evidence and moved intact to
`.xdesign-determinism`, where full-byte doctor passes. `authority-root-review.json` records
the boundary correction; no revision or receipt was deleted or rewritten.

The final durable run is `ebf368c0-7a0a-4715-9461-9aa07beabbd4`, store
`a55e726d-dd17-4536-b4c9-62313a5dfcaa` in `.xdesign-final`, outside cache. Both alternate
roots were added to `.gitignore` before its design gate.

| Stage | Store revision | Operation ID | Artifact snapshot or registration |
| --- | --- | --- | --- |
| registration | 1 | `881665c7-e265-444f-b434-b59f7edeee8e` | External wish import and frozen run |
| requirements | 2 | `wish-0011-authority-requirements-001` | 3 artifacts; `190c5b9dba4bbe383e4d05467c92a19ffd1d83ae05a945af4df3a15fc7e7e848`; two rules passed |
| design | 3 | `wish-0011-authority-design-001` | 153 artifacts; `045993642c4dfb8f7b09efe91b2d7d09669f6eeb56a948ff22dffd0da0710f8c`; seven rules passed |
| implement | 4 | `wish-0011-authority-implementation-001` | 49 artifacts; `765436b138a2c19f155ecb2c7175d1586c8f30b0d09a65b6e6a98c13ab4ea107`; two rules passed |
| verify | 5 | `wish-0011-authority-verify-001` | 192 artifacts; `08d907364c03ff0f4316a6b88b3589479bc72bc6b7e5028fec837cfceaa34a93`; seven record rules and explicit agent semantic pass |

The design set includes the 26 storage API pages, controller figure, common checker support,
renderer regression, visual review, and durable-root review. The separate design review is
`cache/evidence/wish-0011/authority-design-review.json`. The terminal judge is
`kimi-authority-verification`. The immutable pre-gate matrix is
`cache/evidence/wish-0011/authority-verification-summary.json`; the subsequent completion
and recovery record is `cache/evidence/wish-0011/authority-final-acceptance-run.json`.

Exact replay returned the identical response. After runtime-cache removal, `next` recovered
the same completed head. `authority-final-checkout` verified all 376 revision-5 material
lengths and SHA-256 digests. Checkout and live record replays each passed all seven rules;
`authority-independent-replay-invocation.json` and `authority-live-replay-invocation.json`
retain the exact commands and digests. Another full post-gate render produced zero changes
across all 58 figures, and nine primary figure checks passed. Final, original, and
development stores all reported no full-byte doctor findings.

Final preflight passed 85 Rust tests, 22 SimpleFlow Python checker tests, five focused
CLI-view tests, documentation audit and strict build, SysML parse/model-only/full checks,
and `git diff --check`. The model checks had no violations, 129 held rules, and 26
unevaluated rules. The 69-test renderer suite remains **not green** with the same two
pre-existing failures; the failing documentation-view source, tests, and affected figures
remain unchanged from `HEAD`. The timeline correction is unrelated to those failures.

Inspect the final item with `XDESIGN_STORE_ROOT=.xdesign-final xdesign next wish-0011`.
The original `.xdesign` and `.xdesign-determinism` stores retain their earlier records.
The shared limits above apply to the final run; technical completion is not final human
acceptance.

### Disclosure addendum: intermediate model-check failure during design

A challenge review of the retained evidence found that
`cache/storage-design-checks/model-check.json` and `full-check.json` (written at 00:53,
before the final runs) recorded `ok: false`: `StorageVerification` had duplicate
`ObjectiveMembership` bindings at `verification.sysml:223`. The defect was corrected
before the 01:35 acceptance runs (the final `api-model-check.json` and the preflight
above are clean), but the earlier record text summarized this as "model checks had no
violations" without disclosing the intermediate failure. This addendum closes that
disclosure gap; the failed reports are retained unmodified.

### Review fixes: read consistency, retry recovery, and filesystem-aware validation

A storage acceptance review blocked on two P1 findings and raised six P2 findings;
a companion challenge review added checker-timeout, recovery-disclosure, and durability
findings. All were fixed on the `fix/storage-review-findings` branch, rebased on the
reviewed uncommitted working tree:

1. **Read pinning (P1).** `store read` now reads bytes at the revision it selected
   (`Some(selected.revision)` instead of the head), so a concurrent append can never
   pair one revision's metadata with another revision's bytes
   (`src/storage/cli.rs`).
2. **Retryable stage operations (P1).** Stage attempts stage the engine run record and
   response under `cache/xdesign/WISH_ID/attempts/` and publish them only after the
   store commits; the canonical run record is a projection of the committed head and is
   repaired from store bytes by readers and writers on mismatch. A retry of an
   unreserved operation discards its stale speculative files, `init` reclaims leftover
   directories only when no store item and no pending operation claims them, and
   attempt files of reserved operations are kept as recovery sources
   (`src/stage_cli/persistent.rs`, `src/stage_cli/store.rs`).
3. **Pending recovery reuses published content (P2).** `prepare_materials` checks
   committed/published blobs before reading any source, so an interrupted save whose
   blobs were published completes after its transient sources are lost
   (`src/storage/ops.rs`).
4. **Filesystem-aware manifest names (P2).** Validation rejects a name used as both
   file and directory and distinct names that fold equal under Unicode simple case
   folding (`a`+`a/x`, `File.txt`+`file.txt`, `A/x`+`a/y`), shared with checkout
   materialization (`src/storage/mod.rs`, `src/storage/ops.rs`, `src/storage/blobs.rs`).
5. **Envelope reservation (P2).** Saves reject results/metadata that leave no margin
   (6 levels, 2048 bytes) for the receipt envelope before reservation, so committed
   work is always reportable (`src/storage/ops.rs`).
6. **Numeric precision (P2).** Strict JSON rejects integer tokens beyond u64/i64 and
   decimals whose shortest float64 round-trip changes the value, instead of silently
   rounding them in metadata and receipts (`src/storage/mod.rs`).
7. **Destination ancestry (P2).** Read/checkout destinations require every ancestor to
   be an existing non-symlink directory, not only the immediate parent
   (`src/storage/blobs.rs`).
8. **Containment by identity (P2).** Source containment canonicalizes the longest
   existing prefix, so case-insensitive aliases and symlinks onto the store root are
   refused (`src/storage/ops.rs`).
9. **Checker deadlines (challenge P1).** Checker subprocesses run under a kill deadline
   (default 600 s, `XDESIGN_CHECKER_TIMEOUT_SECS` override) with retained partial logs
   and an explicit `CheckerTimeout` gate error (`src/simple_flow/check.rs`,
   `src/main.rs`).
10. **Durability and disclosure (challenge P2).** macOS flushes use `F_FULLFSYNC`
    (file/dir syncs and the SQLite `fullfsync` pragma), the store's non-recoverable
    identity binding is disclosed with its re-registration recovery path, and the
    recompute-over-cache scaling limit is stated (`docs/design/storage.md`).

Cross-boundary acceptance cases added per the review's matrix: a SQLite write-lock
timeout before reservation followed by an exact retry that commits; an interrupted save
whose sources are deleted after publication, completing from published blobs; threaded
reads pinned to their selected revision while a writer advances the head; abandoned
stage attempts repaired and committed; init retry after abandoned files, blocked by
committed items and by pending operations; and checker kill/output deadline cases
(`tests/storage.rs`, `tests/storage_cli.rs`, `tests/checker_timeout.rs`,
`src/storage/ops.rs` tests). The busy timeout is overridable per process with
`XDESIGN_STORE_BUSY_TIMEOUT_MS` for exactly these cases.

On this branch, 99 Rust tests and 22 SimpleFlow Python checker tests pass. The renderer
suite keeps the same two pre-existing failures disclosed above; they are unchanged by
this branch. These fixes are working-tree evidence, not final human acceptance.
