# `RsOperationReceipt`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::OperationReceipt` in [D86](../../storage.md#d86); implementation
> projection `src/storage/mod.rs`.

<a id="u90"></a>

## RsOperationReceipt

```rust
use serde_json::Value;

pub struct RsOperationReceipt {
    pub operation_id: String,
    pub request_sha256: String,
    pub outcome: RsOperationOutcome,
    pub revision: Option<u64>,
    pub result: Value,
}
```

The durable reconciliation record for one exact save request. It is returned by save and
outcome operations; its `outcome` must be checked before treating `result` as applied work.

**Fields**

| Field | Contract |
| --- | --- |
| `operation_id` | ID reserving the exact operation. |
| `request_sha256` | Lowercase SHA-256 of the exact retained request bytes, not of a later reserialization. Equality checks also compare the retained bytes. |
| `outcome` | Actual [`Pending`, `Committed`, or `Conflict`](rs-operation-outcome.md#u89) state. |
| `revision` | `Some(positive_revision)` exactly for `Committed`; `None` for `Pending` and `Conflict`. |
| `result` | Exact JSON result value from the request. A conflict preserves the proposed result only for diagnosis; pending has no committed authority. |

The receipt schema and null rules are in [I13](../../../interfaces/store-format.md#i13).
A conflict receipt guarantees no item/revision/manifest/head effect from its operation.

**Errors**

An inconsistent stored receipt is `RsStoreError::Integrity`. An unknown operation is
`NotFound`; reusing an operation ID for different valid request bytes is `OperationMismatch`.
A stale head or create/kind conflict is not an error variant. The receipt type itself has no
fallible constructor or independent return channel.

**Persistence**

Constructing or dropping the value has no effect. The stored pending receipt and exact request
are durable before source reads. A committed receipt commits with its complete revision and
head advance; a conflict receipt commits alone. Terminal receipts are immutable. Errors after
reservation can leave pending state, so an error never implies that no receipt was written.

**Logic**

Save inspects the receipt before source reads: an exact terminal receipt returns immediately;
an exact pending request resumes; different bytes fail without returning the other operation's
result. [`RsStore::outcome`](rs-store-outcome.md#u83) reads and validates the actual receipt
without comparing material sources. After uncertain publication/commit/response, callers use
`outcome` or exact retry, never a fresh operation ID.

**Example** (illustrative, not executed):

```rust
let receipt = store.outcome("check-00000042")?;
match receipt.outcome {
    RsOperationOutcome::Committed => {
        assert!(receipt.revision.is_some());
        // Only here may result be treated as the committed operation's response.
    }
    RsOperationOutcome::Pending | RsOperationOutcome::Conflict => {
        assert_eq!(receipt.revision, None);
    }
}
```

**See also:** [`RsOperationOutcome`](rs-operation-outcome.md#u89),
[`RsStore::save`](rs-store-save.md#u82), and
[receipt JSON I13](../../../interfaces/store-format.md#i13).
