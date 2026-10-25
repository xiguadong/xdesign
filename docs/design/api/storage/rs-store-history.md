# `RsStore::history`

> `xdesign::storage` · method · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes
> `XDesignStorage::ProjectStore::history` in [D86](../../storage.md#d86); implementation
> projection `src/storage/{ops,db,blobs}.rs`.

<a id="u103"></a>

## history

```rust
impl RsStore {
    pub fn history(
        &self,
        item_id: &str,
    ) -> Result<Vec<RsItemRevision>, RsStoreError>;
}
```

Return every complete immutable revision of one item in ascending revision order.

**Parameters**

`item_id` follows the common item ID rule and selects one item. History has no revision range
or head selector: it selects the item's entire committed history in one snapshot.

**Returns**

`Ok(Vec<RsItemRevision>)` contains each complete revision from oldest to newest. Every entry
includes exact metadata, all manifest entries, producing operation ID, and any projection
derived from that same revision. The vector is complete or an error is returned; there is no
silent truncation, partial history, or reconstruction from current workspace paths.

**Persistence**

The method revalidates root/database identity and reads a consistent SQLite snapshot and any
stored bytes needed for projections. It performs no operation reservation, head update,
staging cleanup, record repair, or cache materialization.

**Errors**

A malformed item ID is `InvalidInput`; a missing item is `NotFound`. Noncontiguous history,
invalid head/revision or operation links, malformed manifests, and any selected projection
failure are `Integrity`. Unsafe stored paths use `UnsafePath`; local filesystem/SQLite access
uses `Io`/`Database`.

**Logic**

1. Validate the ID and live store identity, then begin one snapshot.
2. Resolve the item and select all revision numbers in ascending order.
3. For every selected revision, read its exact metadata and complete manifest.
4. Validate contiguous revisions, current head, committed producing receipts, and manifests.
5. Derive each wish projection from that revision's own exact bytes and return the complete
   ordered vector, or fail without partial output.

**Example** (illustrative, not executed):

```rust
let history = store.history("wish-012345")?;
for adjacent in history.windows(2) {
    assert_eq!(adjacent[0].revision + 1, adjacent[1].revision);
}
```

**See also:** [`RsItemRevision`](rs-item-revision.md#u87),
[`RsStore::get`](rs-store-get.md#u102), and
[complete-history JSON I13](../../../interfaces/store-format.md#i13).
