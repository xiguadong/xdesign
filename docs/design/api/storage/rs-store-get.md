# `RsStore::get`

> `xdesign::storage` · method · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes
> `XDesignStorage::ProjectStore::get` in [D86](../../storage.md#d86); implementation
> projection `src/storage/{ops,db,blobs}.rs`.

<a id="u102"></a>

## get

```rust
impl RsStore {
    pub fn get(
        &self,
        item_id: &str,
        revision: Option<u64>,
    ) -> Result<RsItemRevision, RsStoreError>;
}
```

Return one complete immutable revision selected from a consistent committed snapshot.

**Parameters**

- `item_id` follows the common item ID rule.
- `revision = Some(n)` selects exactly revision `n`; `Some(0)` is invalid because committed
  revisions start at 1.
- `revision = None` selects the item head at snapshot start. Later commits do not change which
  revision this call selected.

**Returns**

`Ok(RsItemRevision)` includes item identity/kind, selected positive revision, exact metadata,
every manifest entry in deterministic URI order, producing operation ID, and any same-revision
wish projection. It never synthesizes historical metadata or manifests from current workspace
paths. No partial revision is returned if a selected row or projection fails.

**Persistence**

The method validates root/database identity and reads records and required projection bytes
without writes. It does not update heads, reserve operations, clean staging, repair defects,
materialize cache, or modify live sources.

**Errors**

Malformed ID or revision zero is `InvalidInput`; missing item/revision is `NotFound`.
Noncontiguous revision/head data, invalid producing-operation links, malformed manifests, and
projection failure are `Integrity`. Unsafe stored paths use `UnsafePath`; local filesystem or
SQLite access uses `Io`/`Database`.

**Logic**

1. Validate arguments and store identity, then begin one snapshot.
2. Resolve the item and selected revision (or snapshot head).
3. Read exact metadata and every complete manifest row.
4. Validate revision/head, producing-receipt, and manifest invariants.
5. Derive any wish projection from that same selected revision's exact registry/run/plan bytes,
   then return the complete revision.

**Example** (illustrative, not executed):

```rust
let head = store.get("wish-012345", None)?;
let selected = store.get("wish-012345", Some(head.revision))?;
assert_eq!(selected.revision, head.revision);
assert_eq!(selected.operation_id, head.operation_id);
```

**See also:** [`RsItemRevision`](rs-item-revision.md#u87),
[`RsStore::history`](rs-store-history.md#u103), and
[`RsStore::read_material`](rs-store-read-material.md#u104).
