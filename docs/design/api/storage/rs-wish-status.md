# `RsWishStatus`

> `xdesign::storage` · enum · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · mirrors `XDesignCommon::WishStatus` for the
> derived projection in [D86](../../storage.md#d86); implementation projection
> `src/storage/mod.rs`.

<a id="u91"></a>

## RsWishStatus

```rust
pub enum RsWishStatus {
    Received,
    Active,
    Blocked,
    Delivered,
}
```

Output-only wish lifecycle vocabulary derived from a selected revision's run record. JSON uses
lowercase `received`, `active`, `blocked`, and `delivered`. It is never an independently
editable store field.

**Variant logic**

| Exact run fact | Derived variant |
| --- | --- |
| `status == "pending"` | `Received` |
| `status == "active"` | `Active` |
| `status == "blocked"` | `Blocked` |
| `status == "completed"` | `Delivered`; the containing projection has `current_stage = None` |

For every non-completed valid run, the containing projection instead takes `current_stage`
from `visits[current_visit].stage_id`. No metadata flag or arbitrary run-status string is an
alternative authority.

**Errors**

The enum has no parameters or fallible method of its own. An unknown run status, invalid
current-visit index, or inconsistent registry/run/plan binding is `RsStoreError::Integrity`,
not a guessed/default `RsWishStatus`. Query argument and local access failures retain their
own typed errors.

**Persistence**

The status is computed while reading a selected revision; it is not separately mutated or
persisted as wish authority. Constructing or matching the enum has no store effect. Queries
that derive it do not update the run, item head, or metadata.

**Logic**

The projection first validates the selected manifest and exact registry/run/plan bindings,
then applies the table above. Only a completed run maps to `Delivered` and clears the current
stage. Invalid source records abort projection with an integrity error rather than excluding
or repairing the wish.

**Example** (illustrative helper, not an additional store API):

```rust
fn wish_status(run_status: &str) -> Result<RsWishStatus, RsStoreError> {
    match run_status {
        "pending" => Ok(RsWishStatus::Received),
        "active" => Ok(RsWishStatus::Active),
        "blocked" => Ok(RsWishStatus::Blocked),
        "completed" => Ok(RsWishStatus::Delivered),
        other => Err(RsStoreError::Integrity(format!("invalid run status: {other}"))),
    }
}
```

**See also:** [`RsWishProjection`](rs-wish-projection.md#u92),
[`RsStore::list`](rs-store-list.md#u101), and
[wish projection D86](../../storage.md#d86).
