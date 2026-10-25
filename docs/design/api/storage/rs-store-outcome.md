# `RsStore::outcome`

> `xdesign::storage` · method · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::ProjectStore::outcome` in [D86](../../storage.md#d86); implementation
> projection `src/storage/{ops,db}.rs`.

<a id="u83"></a>

## outcome

```rust
impl RsStore {
    pub fn outcome(
        &self,
        operation_id: &str,
    ) -> Result<RsOperationReceipt, RsStoreError>;
}
```

Return the actual stored pending or terminal receipt for one operation. This is the
reconciliation read after save uncertainty, not a committed-only query.

**Parameters**

`operation_id` follows the common operation ID rule and selects the exact operation. The method
takes no request or source paths and does not compare a new request against the stored one.

**Returns**

`Ok(RsOperationReceipt)` contains the stored exact request digest,
[`Pending`, `Committed`, or `Conflict`](rs-operation-outcome.md#u89), optional committed
revision, and preserved JSON result. `Pending` is explicit recoverable state, not an error or
committed authority. `Conflict` is terminal receipt data; its proposed result was not applied.

**Persistence**

The method revalidates root/database identity and reads one consistent SQLite snapshot. It
does not reserve/finalize an operation, read material sources, clean staging, repair records,
or update heads. Constructing the returned receipt has no additional persistence effect.

**Errors**

| Variant | Condition |
| --- | --- |
| `InvalidInput` | Malformed operation ID. |
| `NotFound` | No operation with that ID exists. |
| `Integrity` | Stored request/receipt state violates pending or terminal invariants. |
| `UnsafePath` | Live store path/identity objects cannot be inspected safely. |
| `Io` / `Database` | Local filesystem or SQLite access fails. |

`outcome` has no `UnknownOutcome` result: it is the read used to resolve that uncertainty.
An empty or fabricated receipt is never substituted for an unknown ID or access failure.

**Logic**

1. Validate `operation_id` and live store identity.
2. Read the exact operation and receipt in one snapshot.
3. Validate request-digest, outcome, revision, and result invariants.
4. Return the stored receipt without reading sources or changing pending state.

**Example** (illustrative, not executed):

```rust
let receipt = store.outcome("check-00000042")?;
match receipt.outcome {
    RsOperationOutcome::Pending => {
        // Resume only with the original exact request bytes.
    }
    RsOperationOutcome::Committed | RsOperationOutcome::Conflict => {
        // Terminal receipt: exact retry returns this same outcome.
    }
}
```

**See also:** [`RsOperationReceipt`](rs-operation-receipt.md#u90),
[`RsStore::save_bytes`](rs-store-save-bytes.md#u100), and
[crash recovery I12](../../../interfaces/store-format.md#i12).
