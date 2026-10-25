# `RsStore::open`

> `xdesign::storage` · associated function · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes opening of the local
> `XDesignStorage::ProjectStore` boundary in [D86](../../storage.md#d86); implementation
> projection `src/storage/{mod,db}.rs`.

<a id="u99"></a>

## open

```rust
use std::path::Path;

impl RsStore {
    pub fn open(root: &Path) -> Result<Self, RsStoreError>;
}
```

Open and validate an existing initialized version-1 store. Open performs no creation, cleanup,
repair, adoption, fallback, downgrade, or migration.

**Parameters**

`root` names an existing valid store on a supported local POSIX filesystem. The path is
explicit; this function does not read the environment or choose `.xdesign` on its own. All
store-controlled objects must satisfy ownership, permission, type, and containment rules. A
closed allowlist of exactly the OS-generated root entry `.DS_Store` is tolerated: it is inert to
every store semantic and is not a doctor finding, while any other foreign root entry refuses
`open`.

**Returns and persistence**

`Ok(store)` has validated the root marker, database application and schema IDs, store ID,
owner-only permissions, and root/database device/inode pairs. The returned handle retains
those identities for checks before every operation and commit. Opening and later read
snapshots do not mutate store authority. `Err` returns no handle and creates nothing.

**Errors**

| Variant | Condition |
| --- | --- |
| `NotFound` | The store is absent. |
| `UnsupportedFormat` | Foreign, truncated, unsupported, or newer root/database format. |
| `UnsafePath` | Symlink, containment, owner, permission, or object-type defect prevents safe opening. |
| `Integrity` | Live and recorded root/database/store identities disagree. |
| `Io` / `Database` | Local filesystem or SQLite access fails. |

The store CLI's doctor uses a separate private degraded inspection path to report an invalid
root rather than opening it as a store. That does not relax this public method's checks.

**Logic**

1. Inspect root objects without following store-controlled symlinks.
2. Read strict `format.json`; open SQLite without creating it.
3. Check application/schema identity, store ID, and foreign keys.
4. Compare live and recorded root/database device/inode identities and validate owner-only
   permissions and path types.
5. Retain the verified identities for subsequent operation and commit checks, then return the
   handle.

**Example** (illustrative, not executed):

```rust
let store = RsStore::open(Path::new(".xdesign"))?;
let heads = store.list(None, None)?;
drop(heads);
drop(store);
```

Opening a handle is not a promise that arbitrary SQLite connection configurations are
thread-safe; use the common [`RsStore`](rs-store.md#u81) boundary.

**See also:** [`RsStore::init`](rs-store-init.md#u98),
[`RsStore::doctor`](rs-store-doctor.md#u105), and
[physical identity I12](../../../interfaces/store-format.md#i12).
