# Storage acceptance-challenge review fixes

> Reviewed: 2026-10-25. Basis: storage-slice commit `5801897` and its working tree. The
> independent challenge review, its nine findings, and its reproduction evidence are in
> `cache/storage-challenge-20261025/review.json`. This record owns the finding dispositions,
> the recovery-boundary decision, and the executed verification; it does not rewrite the
> storage plan's earlier execution history.

<a id="p40"></a>

## Findings, dispositions, and decision

The review blocked storage acceptance on three P1 defects and six P2 gaps. All nine were
reproduced against the reviewed tree before changing source, then fixed with permanent
regression tests. The fixes are in the commit that contains this record, on
`fix/storage-review-findings` after `5801897`; the executed commands and results are in
[P41](#p41).

| ID | Priority | Finding | Disposition |
| --- | --- | --- | --- |
| S1 | P1 | Validation used Rust `f64` parsing/formatting while storage used `serde_json`, so accepted numeric tokens were silently altered and committed unreadable revisions/receipts (`unknown_outcome`, integrity failures on `get`/`outcome`/retry). | Strict validation now uses the same pinned `serde_json` parser/serializer as storage. Each open numeric token is accepted only when its canonical stored form has the same decimal value, and `strict_json` requires the whole decoded document to be an exact decode/encode fixed point before any reservation. `9764.519195169985` and result `1.0468000232875951e-07` are now rejected with `invalid_input` and no receipt; accepted tokens read back identically through `get`, `outcome`, and exact retry. |
| S2 | P1 | Exact stage replay of a committed check needed its response in attempt scratch or canonical cache; deleting cache made the identical check fail even though the committed revision held the bytes, and `next` did not repair retry. | Committed replay now publishes the canonical run record and response from the committed revision's verified materials (`store.read_material`), cross-checking the committed request digest, with no engine rerun and no dependence on attempt scratch or canonical cache. The review probe is a permanent storage-CLI regression: delete `attempts/`, `run.json`, and `responses/`, repeat the identical check, and the stored response, receipt revision, canonical projections, and `next` all remain correct. |
| S3 | P1 | The checker deadline killed only the immediate child; pipe-reader threads waited for descendants holding stdout/stderr, so a 150 ms deadline took about 2 s and a stuck descendant could hang a check while the writer lock was held. | On Unix the checker now runs in its own process group; stdout/stderr are drained non-blockingly inside the deadline loop; expiry sends `SIGKILL` to the whole group, reaps the direct child, and returns an explicit timeout without waiting on descendant-held pipes. The probe scenario now returns in about 150 ms with `timed_out = true`; the 7-test checker-timeout suite is stable across repeated runs. |
| S4 | P2 | Lowercasing did not implement the promised filesystem name equivalence; `Σ.txt`/`ς.txt` and composed/decomposed `é.txt` manifests committed but checkout failed with `EEXIST`, and doctor reported nothing. | A generated, pure-Rust Unicode module supplies full canonical decomposition (NFD, including algorithmic Hangul), canonical ordering, and full case folding. Save validation folds every URI entry and every implied directory entry and rejects canonical/case-equivalent duplicates; checkout materialization repeats the same check before writing. A single Unicode name still saves and checks out; the review's colliding manifests are rejected before reservation. |
| S5 | P2 | Destination validation stopped at the first existing directory, so `read --output alias/nested/...` and `checkout --destination alias/nested/...` succeeded through an earlier symlink ancestor (`alias -> actual`). | The destination ancestor walk now continues up to the filesystem root, requiring every existing ancestor to be a non-symlink directory, while missing parents remain `not_found`. Read output and checkout both refuse the review setup and write nothing. |
| S6 | P2 | Filesystem validation ran before terminal receipt lookup, so replacing a source parent with a file or deleting the project root made an identical committed retry fail with `ENOTDIR`/`not_found` while `get`/`outcome` reported committed. | After request validation, the store now inspects the exact operation row and returns an existing terminal committed/conflict receipt before canonicalizing the project root or resolving any source path; only new/pending operations resolve the project tree and reserve. The review probes now return the identical committed receipt with no new revision; a new operation with an invalid root still leaves no reservation. |
| S7 | P2 | Checkout expanded every manifest URI into RAM (about 6.4 MB RSS for one 1 MiB blob and 73 MB for 64 aliases), so larger valid manifests could exhaust memory during recovery. | Checkout verifies distinct digests with a bounded streaming pass and materializes through bounded buffers; peak memory is independent of manifest size and alias count. The 64-alias review probe drops from 73,039,872 bytes to 6,029,312 bytes RSS, and an in-process allocator regression holds checkout peak-heap growth to tens of kilobytes. |
| S8 | P2 | Extreme decimal input (`0.0e-2147483648`) panicked in debug builds at exponent subtraction instead of returning a typed validation result. | Exponent arithmetic is checked `i64`, unsupported exponent magnitudes are rejected explicitly, and the same extreme token now returns `invalid_input` with exit 2 and no panic. |
| S9 | P2 | Unrelated operations could adopt a published orphan blob without a source, contradicting the documented source-omission contract. | Source-less reuse now requires a committed blob row or verified attribution that an earlier attempt of the same operation published the digest (`staging/<operation-id>/<attempt>/<digest>`). Publication cleanup is ordered after a durable terminal receipt so recovery attribution survives; an unrelated orphan is refused with no item, revision, or blob row, while own-pending recovery and committed carried digests still work. |

**Recovery-boundary decision (design challenge).** The review asked whether this slice may
claim the store as the sole authority for irreplaceable operational history while a restored
or moved store is refused at open and no supported rebind exists. Decision: **it may not.**
The design now states that technical-acceptance evidence and the review facts that support
acceptance remain retained in Git-tracked records independent of any single store directory;
the store remains the operational authority for new wish/run facts, but re-registering wishes
after store loss cannot recover original judgments, receipts, or run history. A supported
verified restore/rebind path is recorded as open work and must exist before any
sole-authority claim. The identity and ownership trust boundary is unchanged: a copied or
moved store is still refused at open, and this decision adds no implicit adoption or
migration. This was an explicit scope decision in the review, not a hidden defect.

Two additional review observations are covered by the same changes: the debug-build decimal
panic (S8) and the cross-operation orphan adoption (S9). The review's note that all 39
targeted regression tests passed on the reviewed tree is consistent with the fixes being
additions of missing cases rather than unrelated breakage.

<a id="p41"></a>

## Executed verification

All commands ran from the repository root against the fix commit with the cached toolchain
(`PATH=cache/toolchain/stable-aarch64-apple-darwin/bin`,
`CARGO_HOME=cache/cargo-home`). Logs and probe output are under
`cache/lead-probes/final/` and `cache/lead-probes/after/`; per-teammate evidence is under
`cache/storage-review-fixes/`, `cache/implementation/storage-*-tests/` (fixtures), and
`cache/task5-evidence/`.

| Check | Result |
| --- | --- |
| `cargo test --offline --locked` | 122 passed, 0 failed across 15 test targets (includes storage 27, storage CLI 10, storage paths 3, storage protocol 4, storage memory 1, checker timeout 7, model-plan sync 4, plus the pre-existing run/stage/flow suites). |
| `cargo test --offline --lib` | 13 passed, including the new blobs, ops, and Unicode unit cases. |
| Review probes against the fixed CLI | S1 numeric tokens → `invalid_input`, no operation/revision, doctor clean; S8 underflow → typed error exit 2 (was panic 101); S4 colliding Unicode manifests → `invalid_input` pre-reservation, single Unicode name saves and checks out; S5 earlier-symlink read/checkout → `unsafe_path` exit 5 with nothing written; S6 destroyed root/source-parent retry → identical committed receipt revision 1, exit 0; S9 unrelated-orphan adoption → `not_found`, no item; S7 64-alias checkout → 6,029,312 bytes RSS (baseline 73,039,872); S3 150 ms deadline with descendants → 150–151 ms `timed_out` (baseline ~2045 ms). |
| S2 cache-loss replay | The new storage-CLI regression commits a fixed-operation wish-verify check, deletes attempt scratch plus canonical run/response files, replays the identical check, and verifies stored response/exit, committed receipt revision, unchanged history, byte-identical restored projections, and a working `next`. |
| Numeric adversarial run | 400 syntactically valid random/repro numbers: accepted tokens all read back decimal-equal through `get`/`outcome`; rejected tokens all `invalid_input` before a receipt; no panics, no unreadable committed revisions, doctor clean. |
| Checkout memory | External CLI RSS probe and the counting-allocator integration test both show checkout working set independent of alias count; all materialized bytes verified. |
| Checker deadline suite | 7 passed, stable across three sequential runs; the review's timeout probe returns 150–151 ms in both parent-waits and parent-exits modes. |
| `python3 -m unittest discover -s tests -p "test_simple_flow_*.py"` | 22 passed. |
| `python3 -m unittest discover -s tools/sysml_web/tests -p "test_*.py"` | 69 tests, 2 failures — the same pre-existing documentation-view failures disclosed for earlier runs (`requirements` edge count `5 != 6`, state-guard count `9 != 6`); their sources and affected figures are untouched by these fixes. |
| `tools/sysml_web/bin/sysml parse` / `check mbse/model` / full check | Parse: 0 errors. Model and full checks: 0 unresolved references, 0 collisions, 0 violations, 129 rules held, 26 pre-existing unevaluated constraint bodies. |
| `python3 tools/check_documentation.py --json` | Clean: 0 diagnostics. |
| `python3 tools/docs.py build` | Exit 0; strict site build succeeds (only the pre-existing note that `AGENTS.md` is excluded from the site). |
| `python3 tools/render_docs.py` | All 58 figures rendered, exit 0. Four figures changed deterministically: `model-definitions.svg` and the SimpleFlow `class-structure.svg` gain the model's `RsGateError::checkerTimeout` channel, `simple-flow-file-plan.svg` reflects the new source/test inventory, and `storage-save.svg` corrects a stale activity label (`Read exact sources` → the model's current `Reuse published, read sources`). |
| `python3 tools/verify_diagrams.py --node-modules <cache>/diagram-tooling/node/node_modules` | 9/9 primary figure checks PASS (measurable presentation properties). The four changed figures were also rasterized as PNG previews and inspected for layout or notation regressions: `storage-save.svg` now shows the model's `Reuse published, read sources` label, the file-plan figure lists the new source and test inventory, and the Rust-types figure shows the added `RsGateError` channel without overlap; the very large `model-definitions.svg` was checked at overview scale, with its added `checkerTimeout` channel also covered by the model checks. |
| `cargo fmt --check`; `git diff --check` | Both clean (the tree was normalized with the pinned rustfmt; only formatting changed in `src/main.rs` and `src/stage_cli/store.rs`, which are otherwise untouched). |

**Limits.** These are local executed checks, not physical power-loss certification, a frozen
compatibility baseline, or final human acceptance. The non-Unix checker fallback is
`cfg`-gated and was not compiled on this host. Staging cleanup after a terminal receipt is
best effort, so a cleanup failure leaves doctor-visible residue rather than hiding a receipt;
concurrent retries can still race staging cleanup against an in-flight publication, a
pre-existing window that leaves published blobs and receipts correct. The 26 unevaluated
SysML constraint bodies and the two pre-existing renderer test failures remain explicit
limits. The store is not claimed as the sole authority for irreplaceable history (decision
in [P40](#p40)); a supported restore/rebind remains open work.
