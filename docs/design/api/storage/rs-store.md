# `RsStore`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes the local
> `XDesignStorage::ProjectStore` boundary in [D86](../../storage.md#d86); implementation
> projection `src/storage/{mod,db}.rs`.

<a id="u81"></a>

## RsStore

```rust
pub struct RsStore {
    // Owned root path, open SQLite handle, and recorded root/database identities.
}
```

`RsStore` owns its root path, SQLite handle, and the device/inode identities needed to refuse
replacement. Its fields are private; callers use typed operations rather than SQL or physical
blob paths. There is no in-memory mutable item head that substitutes for filesystem/SQLite
arbitration.

The API receives its root explicitly and does not read `XDESIGN_STORE_ROOT`; that policy
belongs to the CLI. An application supplies the default as `Path::new(".xdesign")` when
desired. Construction is owned by [`init`](rs-store-init.md#u98) and
[`open`](rs-store-open.md#u99).

**Method map**

| Method | Page | Effect owned by the method |
| --- | --- | --- |
| `init` | [`RsStore::init`](rs-store-init.md#u98) | Create only an absent/empty compatible root, or reopen a valid store with unchanged identity. |
| `open` | [`RsStore::open`](rs-store-open.md#u99) | Validate and open an existing store without creation or repair. |
| `save` | [`RsStore::save`](rs-store-save.md#u82) | Encode and execute one typed exact-operation save. |
| `save_bytes` | [`RsStore::save_bytes`](rs-store-save-bytes.md#u100) | Execute one exact raw-request save or resume pending work. |
| `outcome` | [`RsStore::outcome`](rs-store-outcome.md#u83) | Read the actual pending or terminal receipt. |
| `list` | [`RsStore::list`](rs-store-list.md#u101) | Read current-head summaries with optional filters. |
| `get` | [`RsStore::get`](rs-store-get.md#u102) | Read one complete selected revision. |
| `history` | [`RsStore::history`](rs-store-history.md#u103) | Read every complete revision in ascending order. |
| `read_material` | [`RsStore::read_material`](rs-store-read-material.md#u104) | Return one manifest blob only after full verification. |
| `checkout` | [`RsStore::checkout`](rs-store-checkout.md#u84) | Materialize a complete revision into a new external destination. |
| `doctor` | [`RsStore::doctor`](rs-store-doctor.md#u105) | Complete a read-only inspection and return all findings. |

**Common identity, error, and persistence contract**

Every opened-handle operation revalidates live root/database identity and owner/path
invariants before use; save repeats them inside its commit boundary. Initialization and opening
perform the construction-time checks on their own pages. Identity disagreement is `Integrity`,
unsafe objects are `UnsafePath`, and local access failures use `Io`/`Database`. Each operation's
page owns its specific arguments, return data, errors, and mutation boundary.

Opening or dropping a handle does not rewrite item history. Queries do not repair or clean;
checkout writes only its external destination; doctor never repairs. Opened-handle operations
take `&self`; `init` and `open` are associated functions. The API exposes no mutex, busy, or
transaction handles. Cross-process atomicity is not an in-process threading guarantee: no
`Send`, `Sync`, async, or arbitrary connection thread-safety promise is inferred.

**Example** (illustrative use of an already opened handle):

```rust
fn read_heads(store: &RsStore) -> Result<Vec<RsItemSummary>, RsStoreError> {
    store.list(None, None)
}
```

**See also:** [storage overview](index.md#u79),
[physical format I11–I12](../../../interfaces/store-format.md#i11), and
[store plan](../../../records/plans/storage.md#p35).
