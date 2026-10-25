# `RsOperationOutcome`

> `xdesign::storage` · enum · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::OperationOutcome` in [D86](../../storage.md#d86); implementation
> projection `src/storage/mod.rs`.

<a id="u89"></a>

## RsOperationOutcome

```rust
pub enum RsOperationOutcome {
    Pending,
    Committed,
    Conflict,
}
```

The durable state vocabulary of one exact save operation. JSON uses lowercase `pending`,
`committed`, and `conflict`.

**Variants**

| Variant | Meaning | Receipt invariant |
| --- | --- | --- |
| `Pending` | The exact request is reserved but has no terminal outcome. It is recoverable, not committed authority. | `revision == None`; exact retry may resume it. |
| `Committed` | The complete revision and head advance committed atomically with the receipt. | `revision == Some(positive_revision)`; terminal and immutable. |
| `Conflict` | Expected-head, create, or immutable-kind arbitration rejected the proposed revision. | `revision == None`; terminal and immutable; no item/revision/manifest/head effect from that operation. |

A conflict is data in an `Ok(RsOperationReceipt)`, not an `RsStoreError`. Its preserved result
is proposed diagnostic data, not evidence that the response was applied. `Pending` likewise
never authorizes use of proposed run facts.

**Errors**

The enum itself has no parameters or fallible method. An unknown or inconsistent stored
outcome is `RsStoreError::Integrity`. An unknown operation ID is `NotFound` from
[`RsStore::outcome`](rs-store-outcome.md#u83). `RsStoreError::UnknownOutcome` instead reports
uncertainty around a mutation/response; it is not a fourth receipt outcome.

**Persistence**

The outcome is persisted as part of an [`RsOperationReceipt`](rs-operation-receipt.md#u90).
Constructing or matching the enum alone has no effect. Pending may be finalized only for the
same exact request; committed and conflict receipts never change. Terminal exact retries add
no revision or publication.

**Logic**

Reservation creates or resumes `Pending`. The commit transaction chooses exactly one terminal
state: `Committed` with the complete revision, or `Conflict` with only the durable conflict
receipt. `outcome` returns the actual stored state, including pending, rather than converting
it into an error or success-with-commit.

**Example** (illustrative, not executed):

```rust
match receipt.outcome {
    RsOperationOutcome::Committed => assert!(receipt.revision.is_some()),
    RsOperationOutcome::Pending | RsOperationOutcome::Conflict => {
        assert_eq!(receipt.revision, None);
    }
}
```

**See also:** [`RsOperationReceipt`](rs-operation-receipt.md#u90),
[`RsStore::save`](rs-store-save.md#u82), and
[`RsStore::outcome`](rs-store-outcome.md#u83).
