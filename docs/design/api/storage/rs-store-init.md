# `RsStore::init`

> `xdesign::storage` · associated function · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes initialization of the
> local `XDesignStorage::ProjectStore` boundary in [D86](../../storage.md#d86); implementation
> projection `src/storage/{mod,db}.rs`.

<a id="u98"></a>

## init

```rust
use std::path::Path;

impl RsStore {
    pub fn init(root: &Path) -> Result<Self, RsStoreError>;
}
```

Initialize a compatible version-1 store, or idempotently reopen an existing valid store. It
never adopts nonempty foreign content, upgrades a format, or treats partial residue as a valid
store.

**Parameters**

`root` is an absent path, a safe empty directory, or an existing valid version-1 store on a
supported local POSIX filesystem. Any other nonempty content is refused; a closed allowlist of
exactly the OS-generated root entry `.DS_Store` is tolerated because it is inert to every store
semantic, is never read or written, and is not a doctor finding. The effective user
must own every store-controlled object. The function receives the path directly and does not
resolve `XDESIGN_STORE_ROOT` itself.

**Returns and persistence**

`Ok(store)` returns a usable handle with verified root/database identities. Creating a store
creates and synchronizes the version-1 marker, schema, identity, blob, staging, and lock
structures with owner-only permissions required by
[I12](../../../interfaces/store-format.md#i12). Reinitializing a valid existing store returns
it with the **same `store_id` and no record effect**.

`Err` returns no usable handle. It does not assert that every partial filesystem artifact was
removed; residue is not adopted by a later `init` and requires explicit operator handling.

**Errors**

| Variant | Condition |
| --- | --- |
| `InvalidInput` | Unusable root argument. |
| `UnsafePath` | Containment, object type, symlink, owner, or permission failure. |
| `UnsupportedFormat` | Existing foreign or unsupported store content; no migration or adoption. |
| `Integrity` | Live and recorded root/database/store identity disagree. |
| `Io` / `Database` | Local filesystem or SQLite creation/access failure outside an unknown save outcome. |

**Logic**

1. Validate the root and parent without following store-controlled symlinks.
2. If a complete valid store exists, return it through the open validation path with unchanged
   identity; otherwise require an absent or safe empty root, where a root holding only the
   allowlisted OS-generated entry counts as empty.
3. Create owner-only staging structures; write and flush `format.json`.
4. Create SQLite with `XDS1` application ID, `user_version = 1`, WAL, foreign keys, and full
   synchronization; record root/database identity and `store_id`.
5. Flush required directories, reopen, and verify the same identities before returning.

**Example** (illustrative, not executed):

```rust
let store = RsStore::init(Path::new(".xdesign"))?;
drop(store);
let same_store = RsStore::init(Path::new(".xdesign"))?;
// The second call preserves store_id and adds no item, revision, or operation.
drop(same_store);
```

**See also:** [`RsStore`](rs-store.md#u81),
[`RsStore::open`](rs-store-open.md#u99), and
[root initialization I11](../../../interfaces/store-format.md#i11).
