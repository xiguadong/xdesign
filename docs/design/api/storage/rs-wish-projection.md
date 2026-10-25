# `RsWishProjection`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes
> `XDesignStorage::WishProjection` in [D86](../../storage.md#d86); implementation
> projection `src/storage/mod.rs`.

<a id="u92"></a>

## RsWishProjection

```rust
pub struct RsWishProjection {
    pub revision: u64,
    pub flow_id: String,
    pub registry_uri: String,
    pub run_uri: String,
    pub status: RsWishStatus,
    pub current_stage: Option<String>,
}
```

Output-only flow, registration/run, lifecycle, and current-stage projection at one selected
item revision. It is derived from exact materials, not stored as an independent status record.

**Fields**

| Field | Contract |
| --- | --- |
| `revision` | Exactly the containing item revision, or containing summary's `head_revision`. |
| `flow_id` | Supported flow identifier from wish metadata, equal to the selected registry's `flow_id`. |
| `registry_uri` | Metadata URI resolved through this revision's complete manifest to strict version-1 registry JSON. |
| `run_uri` | Metadata URI equal to registry `run_record_uri`, resolved through this same manifest to the current run JSON. |
| `status` | Derived [`RsWishStatus`](rs-wish-status.md#u91): pending/active/blocked/completed run maps to received/active/blocked/delivered. |
| `current_stage` | For a non-completed run, `Some(visits[current_visit].stage_id)` with a valid index. For a completed run, always `None`. |

Registry wish ID must equal item ID; registry run ID, plan identity/digest, and the run's
corresponding bindings must agree with the retained plan. The projection does not replace the
full [`restore_run`](../core/restore-run.md#u78) validation contract.

**Errors**

Invalid metadata identities or URIs, missing selected materials, malformed registry/run/plan
JSON, an out-of-range visit, unknown run status, or any inconsistent binding is
`RsStoreError::Integrity`. No empty/default projection is substituted. Local byte/database
access uses `Io`/`Database`; unsafe material paths use `UnsafePath`.

**Persistence**

Projection is read-only derivation. Constructing or dropping this value has no effect, and
queries do not write a status/current-stage field. It binds only the selected immutable
revision; later workspace edits and later revisions do not alter it.

**Logic**

1. Select one revision and read its metadata and complete manifest in one snapshot.
2. Resolve `registry_uri` and `run_uri` only through that manifest and read the exact
   registry, run, and retained plan bytes.
3. Validate wish/run/flow/plan identities and digests.
4. Map run status; for non-completed runs select the indexed stage, and for completed runs set
   `current_stage = None`.
5. Return the complete projection or an integrity error.

**Example** (illustrative, not executed):

```rust
let revision = store.get("wish-012345", None)?;
let projection = revision.projection.ok_or_else(|| {
    RsStoreError::Integrity("wish projection is absent".into())
})?;
assert_eq!(projection.revision, revision.revision);
if matches!(projection.status, RsWishStatus::Delivered) {
    assert_eq!(projection.current_stage, None);
}
```

**See also:** [`RsWishStatus`](rs-wish-status.md#u91),
[`RsItemRevision`](rs-item-revision.md#u87), and
[wish metadata/projection I13](../../../interfaces/store-format.md#i13).
