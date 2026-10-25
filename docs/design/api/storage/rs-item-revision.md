# `RsItemRevision`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes `XDesignStorage::ItemRevision` in
> [D86](../../storage.md#d86); implementation projection `src/storage/mod.rs`.

<a id="u87"></a>

## RsItemRevision

```rust
use serde_json::{Map, Value};

pub struct RsItemRevision {
    pub item_id: String,
    pub kind: String,
    pub revision: u64,
    pub metadata: Map<String, Value>,
    pub materials: Vec<RsMaterial>,
    pub operation_id: String,
    pub projection: Option<RsWishProjection>,
}
```

One complete immutable item revision, returned with the projection derived from that same
snapshot. It never represents a delta assembled from current workspace files.

**Fields**

| Field | Contract |
| --- | --- |
| `item_id` | Identity of the containing generic item. |
| `kind` | Immutable kind of that item. |
| `revision` | Positive, contiguous revision number within the item; revision 1 is the first commit. |
| `metadata` | Exact typed JSON object retained for this revision. |
| `materials` | Every [`RsMaterial`](rs-material.md#u86) URI retained at this revision, in deterministic URI order. Unchanged entries are restated. |
| `operation_id` | ID of the committed operation that produced this revision; its receipt must satisfy committed-receipt invariants. |
| `projection` | `Some` only for a kind with a defined projection, currently a valid wish; otherwise `None`. A wish projection's revision equals this revision. |

**Errors**

Constructing the value is not a validated store operation. Query argument failures are
`InvalidInput`, and a missing selected item/revision is `NotFound`. Noncontiguous history,
invalid producing-operation links, malformed metadata/manifests, and projection failure in
stored data are `Integrity`; filesystem and SQLite access failures use `Io`/`Database`.

**Persistence**

The represented revision, metadata, complete manifest, and producing-operation link are
immutable after commit. Constructing or dropping the Rust value performs no write. Queries
read a consistent snapshot and never update heads, clean staging, repair rows, or synthesize
old bytes from live source paths.

**Logic**

[`RsStore::get`](rs-store-get.md#u102) selects one revision or the snapshot head;
[`RsStore::history`](rs-store-history.md#u103) selects every revision in ascending order.
The query reads metadata and every manifest row, validates revision/receipt invariants, and
derives any [`RsWishProjection`](rs-wish-projection.md#u92) from the same selected registry,
run, and plan bytes before returning the complete value.

**Example** (illustrative, not executed):

```rust
let selected = store.get("wish-012345", Some(3))?;
assert_eq!(selected.revision, 3);
if let Some(projection) = &selected.projection {
    assert_eq!(projection.revision, selected.revision);
}
```

**See also:** [`RsItemSummary`](rs-item-summary.md#u88),
[`RsOperationReceipt`](rs-operation-receipt.md#u90), and
[revision JSON I13](../../../interfaces/store-format.md#i13).
