# `RsCheckoutReport`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::CheckoutReport` in [D86](../../storage.md#d86); implementation
> projection `src/storage/mod.rs`.

<a id="u94"></a>

## RsCheckoutReport

```rust
use std::path::PathBuf;

pub struct RsCheckoutReport {
    pub item_id: String,
    pub revision: u64,
    pub destination: PathBuf,
    pub material_count: usize,
}
```

The successful result of materializing one complete selected revision. It identifies only the
stored bytes covered by checkout; it does not claim Git working-file verification, replay, or
a checker run.

**Fields**

| Field | Contract |
| --- | --- |
| `item_id` | Exact selected item identity. |
| `revision` | Exact selected positive revision, including when `None` selected the snapshot head. |
| `destination` | Previously absent external root beneath which the validated URI tree was created. |
| `material_count` | Number of URI entries in the selected complete manifest. An empty generic manifest yields zero and an empty destination. |

**Errors and return boundary**

The report has no independent fallible constructor. [`RsStore::checkout`](rs-store-checkout.md#u84)
returns it only after every selected blob is verified and every destination write succeeds and
is flushed. Invalid selection/destination, missing or altered bytes, unsafe paths, and local
read/write failures return `RsStoreError` and no report. A partial external destination after
failure is a failed checkout, never a successful partial report.

**Persistence**

Constructing, cloning, or dropping the report has no effect. The producing checkout is
read-only with respect to store records and staging; it creates a new external destination
with no-replace semantics and owner-only access. It never merges, overwrites, or automatically
cleans a destination.

**Logic**

Checkout selects the complete revision, verifies every manifest entry and blob before creating
the destination, then publishes each URI beneath it without replacement. Only after all files
and required directories succeed does it construct this report with the actual selected
revision and manifest count.

**Example** (illustrative, not executed):

```rust
let report = store.checkout(
    "wish-012345",
    Some(3),
    Path::new("cache/replays/wish-012345-r3"),
)?;
assert_eq!(report.item_id, "wish-012345");
assert_eq!(report.revision, 3);
println!("{} verified materials", report.material_count);
```

**See also:** [`RsStore::checkout`](rs-store-checkout.md#u84),
[`RsItemRevision`](rs-item-revision.md#u87), and
[checkout CLI contract I14](../../../interfaces/store-format.md#i14).
