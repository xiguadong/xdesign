# `RsStoreError`

> `xdesign::storage` · enum · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · language-side typed failure channel; the logical
> model defines storage outcomes and defects but no Rust error type. Implementation projection
> `src/storage/mod.rs`.

<a id="u93"></a>

## RsStoreError

```rust
pub enum RsStoreError {
    InvalidInput(String),
    UnsupportedFormat(String),
    NotFound(String),
    OperationMismatch { operation_id: String },
    Integrity(String),
    UnsafePath(String),
    Io { operation_id: Option<String>, message: String },
    Database { operation_id: Option<String>, message: String },
    UnknownOutcome { operation_id: String },
}
```

The typed error channel for expected storage failures. String payloads are diagnostics, not
stable machine-readable codes. Optional operation IDs on local failures aid diagnosis;
`UnknownOutcome` always names the operation that must be reconciled.

**Variants, payload semantics, and CLI projection**

| Variant | Contract | Store CLI mapping |
| --- | --- | --- |
| `InvalidInput(String)` | Request, option, ID, digest, URI, source, or size validation failed. | 2 |
| `UnsupportedFormat(String)` | Root/database/request format is foreign or unsupported; no migration. | 5 for root/database; 2 for request schema |
| `NotFound(String)` | Store, operation, item, revision, or manifest entry does not exist. | 3 |
| `OperationMismatch { operation_id }` | The operation ID is already bound to different exact request bytes. | 4 |
| `Integrity(String)` | Stored records, references, root/database identity, bytes, or projection are defective. | 5 |
| `UnsafePath(String)` | A store/source/destination path fails containment, type, symlink, owner, or permission checks. | 5 |
| `Io { operation_id, message }` | Local filesystem failure outside the unknown-outcome window. | 6 |
| `Database { operation_id, message }` | SQLite failure outside the unknown-outcome window, including an exhausted busy wait. | 6 |
| `UnknownOutcome { operation_id }` | The operation may be pending, committed, or conflict after publication/commit/response uncertainty. | 7 |

The stable JSON envelope codes and exit behavior belong to
[I14](../../../interfaces/store-format.md#i14), not to the diagnostic strings.

**Conflict and persistence rules**

A stale expected head or create/kind conflict is **not** an error: save returns
`Ok(RsOperationReceipt { outcome: Conflict, .. })` after durably recording it. Source failures
after reservation can leave a valid pending receipt. `Err` therefore does not guarantee zero
persistence effects. `UnknownOutcome` requires [`outcome`](rs-store-outcome.md#u83) or exact
retry before new work; callers must not substitute a fresh operation ID.

**Logic**

Operations return expected caller/data, format, integrity, path, filesystem, and database
failures through this enum instead of panicking. Individual doctor findings remain report data
when inspection completes. `outcome` itself is the reconciliation read and has no
`UnknownOutcome` result. Resource exhaustion and process abort remain outside the documented
panic/failure contract.

**Example** (illustrative reconciliation, not executed):

```rust
let receipt = match store.save_bytes(project_root, raw) {
    Ok(receipt) => receipt,
    Err(RsStoreError::UnknownOutcome { operation_id }) => {
        store.outcome(&operation_id)?
    }
    Err(error) => return Err(error),
};
```

**See also:** [`RsOperationReceipt`](rs-operation-receipt.md#u90),
[`RsOperationOutcome`](rs-operation-outcome.md#u89), and
[`RsStore::outcome`](rs-store-outcome.md#u83).
