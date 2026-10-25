# `RsStore::read_material`

> `xdesign::storage` · method · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes
> `XDesignStorage::ReadMaterialOperation` in [D86](../../storage.md#d86); implementation
> projection `src/storage/{ops,db,blobs}.rs`.

<a id="u104"></a>

## read_material

```rust
impl RsStore {
    pub fn read_material(
        &self,
        item_id: &str,
        revision: Option<u64>,
        uri: &str,
    ) -> Result<Vec<u8>, RsStoreError>;
}
```

Return exact bytes for one entry of a selected committed manifest only after complete length
and SHA-256 verification.

**Parameters**

- `item_id` and `revision` use [`get`](rs-store-get.md#u102) selection: `None` selects the head
  at snapshot start, `Some(n)` selects that positive revision, and `Some(0)` is invalid.
- `uri` must be a valid logical URI and an exact member of that selected complete manifest.
  A digest-identical orphan is not eligible without this manifest membership.

**Returns**

`Ok(Vec<u8>)` contains the exact selected bytes after the recorded length and SHA-256 have
been checked through end of file. The vector is bounded by the 256 MiB material cap. On error,
no unverified prefix is returned as content.

**Persistence**

The method revalidates root/database identity, reads the selected revision/manifest in one
consistent snapshot, and streams the stored blob read-only. It does not update heads, reserve
operations, clean staging, repair bytes, write cache, or trust a live source path.

**Errors**

Malformed ID/URI or revision zero is `InvalidInput`; missing item, revision, or manifest URI
is `NotFound`. Missing, truncated, altered, extra-long, swapped, or otherwise inconsistent
stored bytes are `Integrity`; symlink/unsafe path components are `UnsafePath`. Local read or
database access failures use `Io`/`Database`.

**Logic**

1. Validate arguments and store identity; select revision and manifest in one snapshot.
2. Resolve the exact URI to its committed manifest entry and digest-derived blob path.
3. Open only that path with store type/containment/symlink checks.
4. Stream through EOF while counting bytes and computing SHA-256; reject excess length.
5. Require exact recorded length and digest, then return the verified byte vector.

**Example** (illustrative, not executed):

```rust
let head = store.get("wish-012345", None)?;
let uri = "cache/wishes/wish-012345.sysml";
let bytes = store.read_material("wish-012345", Some(head.revision), uri)?;
let manifest = head.materials.iter()
    .find(|entry| entry.uri == uri)
    .ok_or(RsStoreError::NotFound("wish source".into()))?;
assert_eq!(bytes.len() as u64, manifest.byte_length);
```

The example assumes the named entry exists. Production code should retain the selected
manifest entry rather than repeat a string literal lookup.

**See also:** [`RsMaterial`](rs-material.md#u86),
[`RsStore::get`](rs-store-get.md#u102), and
[verified read I12](../../../interfaces/store-format.md#i12).
