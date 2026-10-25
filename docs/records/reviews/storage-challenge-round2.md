# Storage acceptance-challenge round-2 review fixes

> Reviewed: 2026-10-25. Basis: `fix/storage-review-findings` at `ecc1580` and its
> working tree. The independent round-2 challenge review, its six findings, and its
> probe fixtures are retained under `cache/storage-challenge2-20261025/` (`review.md`,
> `lock-probe/`, `mem-probe/`, `env-probe/`). This record owns the finding
> dispositions, the round-2 design decisions, and the executed re-verification; it
> does not rewrite the round-1 record [P40](storage-challenge.md#p40) or the storage
> plan's earlier execution history [P39](../plans/storage.md#p39).

<a id="p42"></a>

## Findings, dispositions, and decisions

The review re-read the nine round-1 fixes and found them sound, with the full suite
passing on `ecc1580`. It then reproduced six new defects against that tree: one broken
design guarantee (P2), two availability or recovery gaps the round-1 review missed
(P2), and three smaller gaps (P3). Every finding was reproduced before source changes
and now has a permanent regression. The fixes are in the commit that contains this record,
on `fix/storage-review-findings` after `ecc1580`; the executed commands and results are in
[P43](#p43).

| ID | Priority | Finding | Disposition |
| --- | --- | --- | --- |
| N1 | P2 | The response-envelope reservation budgeted the manifest echo with raw `uri.len()`, while `valid_uri` permits `"` characters that the pinned JSON encoder escapes to two bytes. A quote-heavy manifest could then commit a revision whose `get` and `history` responses exceed the 16 MiB envelope, permanently. | `reserve_response_capacity` now budgets each manifest entry at its JSON-escaped serialized length. The probe request (9,000 materials, 16,695,161 bytes) is rejected with `invalid_input` before reservation and leaves no operation; the committed baseline binary accepts it and both `get` and `history` then fail with exit 5. Permanent regression: `tests/storage.rs` `escaped_uri_manifest_is_budgeted_by_the_envelope_reservation`. |
| N2 | P2 | Save buffered every new material at once and re-read each carried committed blob fully just to learn its length; finalize also re-read every referenced blob inside the `BEGIN IMMEDIATE` transaction with a full-buffering read. Peak save memory grew with the manifest total while checkout was already bounded. | Each material is now resolved, verified, and published as one unit before the next is read, through one reused material-sized source buffer; committed and same-operation content is verified with bounded streaming length-and-digest passes, and finalize re-verifies with the streaming pass while the write lock is held. The review probe (96 MiB manifest, 24 materials) drops from 113,491,968 to at most 12,271,616 bytes RSS across three runs; the counting-allocator regression (`tests/storage_memory.rs`) bounds save peak-heap growth to about 2.19 MB for 32 new 1 MiB materials. |
| N3 | P2 | A crash's per-wish stage-writer lock under `locks/items/` wedged every later `check` of that wish, while `doctor` inspected only `staging/` and reported zero findings, leaving no supported diagnosis of the residue. | Doctor now inspects `locks/items/` and reports `StaleLock` when the recorded process is no longer running (or no owner is readable); a live-pid lock remains normal concurrency and is never reported. Doctor stays read-only: removing the named lock file is the documented operator recovery, and no automatic takeover was added. The probe shows a dead-pid lock producing exit 5 with a `stale_lock` finding and a live-pid lock producing a clean report. The `RsDoctorFindingKind` API page and the SysML `DoctorFindingKind` enums carry the new variant. |
| N4 | P3 | Any foreign file in the store root, typically Finder's `.DS_Store` on macOS, made the store and every wish in it unopenable with no supported remediation. | A closed allowlist tolerates exactly the OS-generated `.DS_Store` root entry at `init`, `open`, and `doctor`: it is inert to every store semantic, never read or written, and not a finding. Any other foreign root entry still refuses `open`. The probe shows `.DS_Store` keeping `info` at exit 0 and doctor clean while another foreign entry still refuses with exit 5. |
| N5 | P3 | Material URIs could be up to 4,096 bytes and still commit, though checkout can fail with `ENAMETOOLONG` under the smallest supported `PATH_MAX`; the promised materializability guarantee was enforced for name collisions but not path length. | New reservations cap every material URI at 900 UTF-8 bytes, leaving ancestry headroom under a 1,024-byte `PATH_MAX`. The gate runs only when no operation row exists and deliberately stays out of `validate_request`, so operations reserved under the original validation remain readable, retryable, and committable. The probe shows a 1,023-byte URI committed by the baseline binary and rejected with `invalid_input` before reservation by the fix. Permanent regressions: `new_reservations_reject_uris_beyond_the_materialization_budget` and `already_reserved_long_uri_operation_still_resumes_and_commits`. |
| N6 | P3 | The documented recovery after an unknown outcome ("retry the exact request") was impossible for direct CLI users when the request file was gone: the retained request bytes were crate-internal, so a pending operation had no recovery path. | `xdesign store request OPERATION_ID --output FILE` writes the exact retained request bytes under the same destination rules as `read` (outside the store, no overwrite, every existing ancestor a non-symlink directory) and reports the operation ID, request SHA-256, receipt state, and byte length. The probe verifies byte-identical recovery, no-overwrite and containment refusals, and `not_found` for an unknown operation; the pending-operation retry path is covered by the storage-CLI regression `store_request_recovers_exact_pending_bytes_for_a_retry`. |

**Round-2 design decisions.** The envelope invariant is enforced at reservation time
using the escaped serialized length, without changing the versioned envelope limits or
the stored request encoding. Save bounds its working set by the largest single material
rather than by the manifest total: one material resolved, verified, and published at a
time, one reused source buffer, and streaming verification inside the commit window. A
stale lock is reported, never repaired: process liveness is best effort because a reused
pid can mask residue, so removing the named lock file remains an operator action. The
root allowlist stays closed at exactly `.DS_Store`; no generic dotfile allowance was
introduced. The 900-byte materialization budget gates new reservations only, so existing
stored operations keep the validation they were reserved under. Request recovery reuses
the read destination safety rules and adds no new store mutation path.

<a id="p43"></a>

## Executed verification

All commands ran from the repository root with the cached toolchain
(`PATH=cache/toolchain/stable-aarch64-apple-darwin/bin`,
`CARGO_HOME=cache/cargo-home`). Round-2 probe scripts, outputs, and verification logs are
under `cache/storage-challenge2-20261025/final/`; the re-run used isolated store roots and
never touched the project's `.xdesign/`.

| Check | Result |
| --- | --- |
| `cargo test --offline --locked` | 129 passed, 0 failed across 14 test targets plus doc-tests (lib 13, main 6, checker timeout 7, model-plan sync 4, process-check contract 4, run restore 11, simple flow 14, stage CLI 10, storage 29, storage CLI 13, storage memory 2, storage paths 3, storage protocol 5, wish-verify 8). |
| Save/checkout memory regressions | Save peak-heap growth about 2.19 MB over a 1.14 MB baseline for 32 new 1 MiB materials; checkout peak-heap growth about 0.1 MB over a 3.23 MB baseline for 48 aliases of one blob (counting-allocator `storage_memory` regression, retained in `verification/storage-memory-allocator.log`). |
| Round-2 probes on the fixed CLI | N1: 9,000-material escaped manifest rejected before reservation (`invalid_input`, no operation; baseline committed, then `get` and `history` exited 5). N2: 96 MiB save max RSS 113,491,968 → at most 12,271,616 bytes over three runs. N3: dead-pid lock → exit 5 with `stale_lock`; live-pid lock → exit 0, no finding. N4: `.DS_Store` root → `info` exit 0 and doctor clean; other foreign entry → exit 5. N5: 1,023-byte URI rejected before reservation by the fix, committed by the baseline. N6: retained request bytes recovered byte-identically with the matching SHA-256; overwrite, inside-store output, and unknown operation refused with the documented codes. |
| `python3 -m unittest discover -s tests -p "test_simple_flow_*.py"` | 22 passed. |
| `python3 -m unittest discover -s tools/sysml_web/tests -p "test_*.py"` | 69 tests, 2 failures — the same pre-existing documentation-view failures disclosed for earlier runs (requirements edge count `5 != 6`, state-guard count `9 != 6`); their sources and figures are untouched by these fixes. |
| `tools/sysml_web/bin/sysml parse` / `check mbse/model` / full check | Parse: 0 errors; model-only 0 errors, 20,700/20,700 references resolved, 0 violations; full 0 errors, 22,641/22,641 references resolved, 0 violations, 129 rules held, 26 pre-existing unevaluated constraint bodies. |
| `python3 tools/check_documentation.py --json` | Clean: all six rules pass, 0 diagnostics. |
| `python3 tools/docs.py build` | Exit 0; strict site build succeeds (only the pre-existing note that `AGENTS.md` is excluded from the site). |
| `python3 tools/render_docs.py` | All 58 figures rendered, exit 0; only `docs/architecture/assets/model-definitions.svg` changed, adding `enum staleLock` to both doctor-finding-kind enum boxes; renderer registry unchanged. |
| `python3 tools/verify_diagrams.py --node-modules <cache>/diagram-tooling/node/node_modules` | 9/9 primary figure checks PASS; the changed definitions figure was inspected at overview scale and in two focused crops, with the new variants between `stagingArtifact` and `invalidRecord` and no overlap or notation regression. |
| `cargo fmt --check`; `git diff --check` | Both clean. |

**Limits.** These are local executed checks, not physical power-loss certification, a
frozen compatibility baseline, or final human acceptance. The non-Unix checker fallback is
`cfg`-gated and was not compiled on this host. Staging cleanup after a terminal receipt is
best effort, so a cleanup failure leaves doctor-visible residue rather than hiding a
receipt; concurrent retries can still race staging cleanup against an in-flight
publication, a pre-existing window that leaves published blobs and receipts correct.
Stale-lock liveness is best effort: a reused pid can mask crash residue, which is why the
diagnosis stays advisory. The root allowlist is deliberately exactly `.DS_Store`; other OS
metadata is still foreign content. The 900-byte budget gates new reservations only, so an
operation reserved earlier with a longer URI remains readable and retryable but could
still fail checkout explicitly with `ENAMETOOLONG`. The 26 unevaluated SysML constraint
bodies and the two pre-existing renderer test failures remain explicit limits. The store
is not claimed as the sole authority for irreplaceable history (decision in
[P40](storage-challenge.md#p40)); whole-store backup, restore/rebind, migration, deletion,
and remote operation remain out of scope.
