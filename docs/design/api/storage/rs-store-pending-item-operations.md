# `RsStore::pending_item_operations`

> `xdesign::storage` · method · implemented; STORE-01–STORE-10 evidence executed within
> disclosed limits · realizes `XDesignStorage::ProjectStore` queries in
> [D86](../../storage.md#d86); implementation projection `src/storage/ops.rs`.

<a id="u106"></a>

## pending_item_operations

```rust
impl RsStore {
    pub fn pending_item_operations(
        &self,
        item_id: &str,
    ) -> Result<Vec<String>, RsStoreError>;
}
```

Return the IDs of operations that reference an item and are still `pending`, in ascending
ID order. This is the ownership probe for speculative local files: it lets a caller tell
a leftover directory that no reservation claims from files that a pending operation may
still recover.

**Parameters**

`item_id` follows the common item ID rule. The method takes no request bytes and performs
no retry or reconciliation itself.

**Returns**

`Ok(Vec<String>)` is empty when every operation for the item is terminal (committed or
conflict) or the item has no operations at all. The presence of an item head is answered
by [`RsStore::get`](rs-store-get.md#u102), not by this method.

**Persistence**

The method revalidates root/database identity and reads one consistent SQLite snapshot.
It does not reserve, finalize, retry, or abandon any operation, read material sources,
clean staging, or update heads.

**Errors**

| Variant | Condition |
| --- | --- |
| `InvalidInput` | Malformed item ID. |
| `UnsafePath` | Live store path/identity objects cannot be inspected safely. |
| `Integrity` | Stored operation state violates pending or terminal invariants. |
| `Io` / `Database` | Local filesystem or SQLite access fails. |

**Logic**

1. Validate `item_id` and live store identity.
2. Select `pending` operations for the item in one snapshot.
3. Return their IDs without changing pending state.

**Example** (illustrative, not executed):

```rust
let pending = store.pending_item_operations("wish-012345")?;
if pending.is_empty() {
    // No reservation claims the wish's files; leftover speculative
    // directories may be reclaimed by a fresh initialization.
}
```

**See also:** [`RsStore::outcome`](rs-store-outcome.md#u83),
[`RsStore::save_bytes`](rs-store-save-bytes.md#u100), and
[crash recovery I12](../../../interfaces/store-format.md#i12).
