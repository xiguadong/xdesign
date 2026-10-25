# `RsItemSummary`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · language-side current-head projection for
> `ProjectStore::list`; the logical model has no separate summary authority. Implementation
> projection `src/storage/mod.rs`.

<a id="u88"></a>

## RsItemSummary

```rust
pub struct RsItemSummary {
    pub item_id: String,
    pub kind: String,
    pub head_revision: u64,
    pub projection: Option<RsWishProjection>,
}
```

The head-only list form of an item. It complements, and never replaces, complete
[`RsItemRevision`](rs-item-revision.md#u87) history.

**Fields**

| Field | Contract |
| --- | --- |
| `item_id` | Stable identity of the listed item. |
| `kind` | Immutable item kind used by exact-kind filtering. |
| `head_revision` | Current committed head at the list snapshot's start. |
| `projection` | Same-head derived wish projection when defined, otherwise `None`; its revision equals `head_revision`. |

The summary intentionally omits metadata, manifest entries, and producing operation ID. Use
[`RsStore::get`](rs-store-get.md#u102) or
[`RsStore::history`](rs-store-history.md#u103) when those complete facts are required.

**Errors**

The summary has no independent fallible constructor. Invalid list filters are `InvalidInput`;
malformed selected records, materials, or wish projections are `Integrity`; local snapshot
access uses `Io`/`Database`. A malformed wish is not silently skipped merely to make a status
filter succeed.

**Persistence**

A summary is derived read output, not a second mutable head or status record. Constructing,
cloning, or dropping it has no effect. Listing does not write records, update heads, clean
staging, or repair defects.

**Logic**

[`RsStore::list`](rs-store-list.md#u101) selects current heads in one consistent snapshot,
derives every required same-revision wish projection, applies conjunctive kind/status filters,
and returns summaries in deterministic `item_id` order. An empty selection is a successful
empty vector, not `NotFound`.

**Example** (illustrative, not executed):

```rust
let summaries = store.list(Some("wish"), Some("active"))?;
for summary in summaries {
    assert_eq!(summary.kind, "wish");
    assert_eq!(summary.projection.as_ref().map(|p| p.revision),
               Some(summary.head_revision));
}
```

**See also:** [`RsStore::list`](rs-store-list.md#u101),
[`RsWishProjection`](rs-wish-projection.md#u92), and
[`RsItemRevision`](rs-item-revision.md#u87).
