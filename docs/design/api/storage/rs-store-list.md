# `RsStore::list`

> `xdesign::storage` · method · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::ProjectStore::list` in [D86](../../storage.md#d86); implementation
> projection `src/storage/{ops,db,blobs}.rs`.

<a id="u101"></a>

## list

```rust
impl RsStore {
    pub fn list(
        &self,
        kind: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<RsItemSummary>, RsStoreError>;
}
```

List current item-head summaries in one consistent snapshot, with optional exact-kind and
derived wish-status filters.

**Parameters**

- `kind = Some(value)` restricts results to that exact valid item kind; `None` accepts every
  kind.
- `status = Some(value)` accepts only `received`, `active`, `blocked`, or `delivered` and
  filters the **derived wish projection**, not run-status text or a metadata flag. `None`
  applies no status filter.

The filters are conjunctive. A status filter excludes kinds without a wish projection; it does
not give those kinds a default status.

**Returns**

`Ok(Vec<RsItemSummary>)` contains current-head summaries in deterministic `item_id` order.
Every [`RsItemSummary`](rs-item-summary.md#u88) derives any wish status/current stage from
that head's exact registry, run, and plan materials. An empty selection is a successful empty
vector. The complete result is returned only after every selected row and required projection
succeeds; there is no silent truncation or partial vector.

**Persistence**

The method validates root/database identity and reads a consistent SQLite snapshot plus the
stored materials needed for projection. It does not reserve operations, clean staging, repair
records, materialize cache, update heads, or write an independent wish status.

**Errors**

Invalid kind/status is `InvalidInput`. Unknown items are not an error because this is a
collection query. Malformed records, references, bytes, or projections are `Integrity`;
unsafe stored paths are `UnsafePath`; local filesystem/SQLite access uses `Io`/`Database`.
Status filtering never skips a malformed wish merely to make the query succeed.

**Logic**

1. Validate filters and store identity, then begin one read snapshot.
2. Select current item heads and apply the exact kind filter.
3. Resolve every required same-head wish projection from exact manifest bytes.
4. Apply the derived status filter, excluding non-wish kinds when status is present.
5. Return all summaries in `item_id` order, or return an error instead of partial output.

**Example** (illustrative, not executed):

```rust
let active_wishes = store.list(Some("wish"), Some("active"))?;
for summary in active_wishes {
    assert!(matches!(
        summary.projection.map(|p| p.status),
        Some(RsWishStatus::Active)
    ));
}
```

**See also:** [`RsItemSummary`](rs-item-summary.md#u88),
[`RsWishProjection`](rs-wish-projection.md#u92), and
[query shapes I13](../../../interfaces/store-format.md#i13).
