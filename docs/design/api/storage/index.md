# xdesign storage API design

> Layer: `xdesign::storage` | Status: exported and implemented; STORE-01 through STORE-10
> evidence executed within disclosed limits | Audience: application integrators, module owners, and reviewers.

<a id="u79"></a>

## Storage API map

The exported `xdesign::storage` layer realizes the logical `XDesignStorage` package in
[storage design D86](../../storage.md#d86). This page is only the map to the single owning
contract page for each storage type and nontrivial operation; it does not repeat their
signatures or behavior.

| Anchor | API | Owning contract page |
| --- | --- | --- |
| U80 | `RsSaveRequest` | [`RsSaveRequest`](rs-save-request.md#u80) |
| U81 | `RsStore` | [`RsStore`](rs-store.md#u81) |
| U82 | `RsStore::save` | [`RsStore::save`](rs-store-save.md#u82) |
| U83 | `RsStore::outcome` | [`RsStore::outcome`](rs-store-outcome.md#u83) |
| U84 | `RsStore::checkout` | [`RsStore::checkout`](rs-store-checkout.md#u84) |
| U85 | `RsSaveMaterial` | [`RsSaveMaterial`](rs-save-material.md#u85) |
| U86 | `RsMaterial` | [`RsMaterial`](rs-material.md#u86) |
| U87 | `RsItemRevision` | [`RsItemRevision`](rs-item-revision.md#u87) |
| U88 | `RsItemSummary` | [`RsItemSummary`](rs-item-summary.md#u88) |
| U89 | `RsOperationOutcome` | [`RsOperationOutcome`](rs-operation-outcome.md#u89) |
| U90 | `RsOperationReceipt` | [`RsOperationReceipt`](rs-operation-receipt.md#u90) |
| U91 | `RsWishStatus` | [`RsWishStatus`](rs-wish-status.md#u91) |
| U92 | `RsWishProjection` | [`RsWishProjection`](rs-wish-projection.md#u92) |
| U93 | `RsStoreError` | [`RsStoreError`](rs-store-error.md#u93) |
| U94 | `RsCheckoutReport` | [`RsCheckoutReport`](rs-checkout-report.md#u94) |
| U95 | `RsDoctorFindingKind` | [`RsDoctorFindingKind`](rs-doctor-finding-kind.md#u95) |
| U96 | `RsDoctorFinding` | [`RsDoctorFinding`](rs-doctor-finding.md#u96) |
| U97 | `RsDoctorReport` | [`RsDoctorReport`](rs-doctor-report.md#u97) |
| U98 | `RsStore::init` | [`RsStore::init`](rs-store-init.md#u98) |
| U99 | `RsStore::open` | [`RsStore::open`](rs-store-open.md#u99) |
| U100 | `RsStore::save_bytes` | [`RsStore::save_bytes`](rs-store-save-bytes.md#u100) |
| U101 | `RsStore::list` | [`RsStore::list`](rs-store-list.md#u101) |
| U102 | `RsStore::get` | [`RsStore::get`](rs-store-get.md#u102) |
| U103 | `RsStore::history` | [`RsStore::history`](rs-store-history.md#u103) |
| U104 | `RsStore::read_material` | [`RsStore::read_material`](rs-store-read-material.md#u104) |
| U105 | `RsStore::doctor` | [`RsStore::doctor`](rs-store-doctor.md#u105) |
| U106 | `RsStore::pending_item_operations` | [`RsStore::pending_item_operations`](rs-store-pending-item-operations.md#u106) |

The [store format](../../../interfaces/store-format.md#i11) owns cross-process JSON, physical
paths, CLI envelopes, and exit codes. The linked API pages own their Rust contracts and cite
the applicable format sections rather than duplicating a second field-by-field wire contract.

<a id="planned-module-projection"></a>

### Module implementation

| Module | Source | Responsibility |
| --- | --- | --- |
| `storage` | `src/storage/mod.rs` | Public types and method exports |
| `db` | `src/storage/db.rs` | Versioned SQLite schema, consistent snapshots, identity checks, and transactions |
| `blobs` | `src/storage/blobs.rs` | Safe source reads, staging, fsync, no-replace publication, and verified streaming reads |
| `ops` | `src/storage/ops.rs` | Save/retry/outcome, queries, checkout, and doctor |
| `cli` | `src/storage/cli.rs` | `xdesign store` argument/envelope/exit adapter |

These source files implement the exported module. The stage integration is
`src/stage_cli/{mod,store,work,persistent}.rs` under the
[source plan](../../../records/plans/storage.md#p37). `src/lib.rs` exports `xdesign::storage`.
The separately integrated `XDesignRustUml` package projection covers the module boundary,
while these pages own the individual Rust contracts.
[Executed STORE-01 through STORE-10 evidence](../../../records/plans/storage.md#p38) covers the
local item/revision store, verified queries, material reads, checkout and doctor, exact-request
recovery, and two store-backed flow demonstrations within disclosed limits. Two unrelated
pre-existing renderer failures remain. This evidence does not establish physical power-loss
certification, a frozen compatibility baseline, a complete V-model, or final human acceptance.
