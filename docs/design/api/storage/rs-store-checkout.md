# `RsStore::checkout`

> `xdesign::storage` · method · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::CheckoutOperation` in [D86](../../storage.md#d86); implementation
> projection `src/storage/{ops,blobs}.rs`.

<a id="u84"></a>

## checkout

```rust
use std::path::Path;

impl RsStore {
    pub fn checkout(
        &self,
        item_id: &str,
        revision: Option<u64>,
        destination: &Path,
    ) -> Result<RsCheckoutReport, RsStoreError>;
}
```

Verify and materialize every entry of one complete selected revision into a previously absent
external destination without replacement.

**Parameters**

- `item_id` follows the common item ID rule.
- `revision = Some(n)` selects that positive revision; `None` selects the head at snapshot
  start; `Some(0)` is invalid.
- `destination` must be previously absent, with an existing parent that is a safe local
  directory without symlink escape. It is an external materialization root, not permission to
  replace project or store content. Every selected manifest URI is joined beneath it using
  validated logical path components.

**Returns**

`Ok(RsCheckoutReport)` identifies the exact item/revision, destination, and number of manifest
URI entries only after every entry is materialized. An empty generic manifest produces an
empty destination and count zero. The report covers selected stored bytes only; it does not
claim Git working-file verification, replay, or a checker run.

**Persistence**

The store remains read-only. Checkout reads and verifies every selected blob through EOF
before creating the destination, then creates required directories and publishes files with
no-replace semantics and owner-only access. It never overwrites or merges an existing path.
Store staging, records, heads, and receipts are unchanged.

If destination creation or a later write fails, the method returns an error and no report.
Any partial external destination is explicitly a failed checkout, not usable success; it is
not automatically cleaned or presented as a partial successful checkout.

**Errors**

Invalid item/revision is `InvalidInput`; missing item/revision is `NotFound`; an unsafe or
existing destination, traversal, or symlink is `UnsafePath`. Missing/altered bytes and invalid
manifests are `Integrity`; destination and local read/write failures use `Io`/`Database`.
No unverified or partial file set is returned as success.

**Logic**

1. Validate arguments/store identity and select the complete revision in one snapshot.
2. Verify all manifest structure and every selected blob's length and SHA-256 through EOF.
3. Revalidate destination parent safety and destination absence.
4. Create directories/files without replacement while revalidating each path component.
5. Flush written files and required directories; return the report only after all entries
   succeed.

**Example** (illustrative, not executed):

```rust
let report = store.checkout(
    "wish-012345",
    Some(3),
    Path::new("cache/replays/wish-012345-r3"),
)?;
println!("{} materials at revision {}", report.material_count, report.revision);
```

The destination must not exist before the call; checkout itself does not perform replay, a
checker pass, or cleanup.

**See also:** [`RsCheckoutReport`](rs-checkout-report.md#u94),
[`RsStore::read_material`](rs-store-read-material.md#u104), and
[executed STORE-07/09 evidence](../../../records/plans/storage.md#p38).
