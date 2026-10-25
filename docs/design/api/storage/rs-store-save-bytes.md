# `RsStore::save_bytes`

> `xdesign::storage` · method · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes the exact raw-request form of
> `XDesignStorage::SaveOperation` in [D86](../../storage.md#d86); implementation projection
> `src/storage/{ops,db,blobs}.rs`.

<a id="u100"></a>

## save_bytes

```rust
use std::path::Path;

impl RsStore {
    pub fn save_bytes(
        &self,
        project_root: &Path,
        raw: &[u8],
    ) -> Result<RsOperationReceipt, RsStoreError>;
}
```

Validate and execute one complete raw JSON request without canonicalizing it, or resume its
exact pending operation. This is the byte-preserving counterpart of
[`RsStore::save`](rs-store-save.md#u82).

**Parameters and exact identity**

- `project_root` is the safe local containment root for all declared sources; it does not
  reinterpret the opened store root. Sources must be project-relative regular files without
  symlink components and must not enter the store.
- `raw` is one complete strict UTF-8 JSON request of at most 16 MiB, including all required
  version-1 fields. Trailing bytes/content, duplicate or unknown protocol fields, excess
  depth, and invalid values are rejected.

`request_sha256` covers the **exact supplied bytes**, which are also retained for comparison.
No formatting or member sorting is applied. Raw requests that differ only in whitespace or
member order are different operations for idempotency; reusing their operation ID produces
`OperationMismatch`. Typed save instead obtains deterministic identity from its deterministic
compact JSON encoding, not JCS.

Number tokens in `metadata` and `result` are validated with the same pinned `serde_json`
numeric parser/serializer that stores them, and are accepted only when the persisted
representation is an exact decode/re-encode fixed point. Integers beyond the u64/i64 range,
decimals whose canonical stored form differs from the supplied token, and unsupported
extreme exponent ranges are `InvalidInput`; they are never rounded silently and never panic.
Pure request/path-syntax validation precedes reservation, and an identical exact request
with a terminal receipt returns that receipt before the project root is canonicalized or any
source path is resolved.

**Returns and persistence**

| Case | Return and durable effect |
| --- | --- |
| New operation at expected head | `Ok` committed receipt with `revision = Some(expected + 1)`; revision, complete manifest, blob references, receipt, and head advance commit together. |
| Stale head, create collision, or kind conflict | `Ok` conflict receipt with `revision = None`; only that receipt commits, with no item/revision/manifest/head effect. |
| Exact terminal retry | Return the stored committed/conflict receipt before project-root canonicalization, source-path resolution, or publication; no new effect. |
| Exact pending retry | Resume the same reservation and return a final receipt or an error/unknown outcome; never create another operation. |
| Same ID, different valid bytes | `Err(OperationMismatch)` with no new effect and no disclosure of the other result as this request's result. |
| Uncertain publication/commit/response | `Err(UnknownOutcome { operation_id })`; pending, committed, and conflict remain possible until reconciliation. |

Successful return is never `Pending`. Conflict preserves only the proposed result for
diagnosis. Errors after reservation can leave durable pending work; use
[`outcome`](rs-store-outcome.md#u83) or the **same raw slice**, never a fresh or prettified
request.

**Errors**

Strict JSON, field, source, digest, and size validation failures are `InvalidInput`;
unsupported request schema is `UnsupportedFormat`. A never-stored digest is `NotFound`;
missing/altered committed bytes and root replacement are `Integrity`; path violations are
`UnsafePath`; local failures outside uncertainty are `Io`/`Database`. Publication/commit/
response uncertainty is `UnknownOutcome`. Conflict is not an error variant.

**Logic**

1. Validate root identity and the unchanged raw request/schema, including the complete
   manifest, exact-number fixed points, material-name equivalence, and source declarations;
   hash and retain exact `raw`. This pure validation does not resolve the project root or
   sources.
2. In a short transaction, reject different bytes for an existing ID, return an exact terminal
   receipt before project-root canonicalization or source-path resolution, resume exact
   pending work, or reserve pending receipt/request/digests.
3. After reservation, canonicalize `project_root` and read supplied sources through EOF with
   the 256 MiB cap and verify digest/length; for omitted sources require a committed blob row
   or a digest verifiably published by this exact operation's earlier attempt, never an
   unrelated published or orphan blob.
4. Stage, fsync, and no-replace publish required blobs, verifying existing final bytes before
   deduplication and flushing containing directories.
5. In `BEGIN IMMEDIATE`, recheck exact operation and item/head/kind. Commit only a conflict
   receipt on arbitration failure; otherwise atomically create/append the complete revision,
   blob metadata, committed receipt, and head.
6. Commit, synchronize, read back, and return the terminal receipt. Uncertain steps around
   publication, commit, or response produce `UnknownOutcome` for reconciliation.

**Example** (illustrative, not executed):

```rust
let first = store.save_bytes(project_root, &raw)?;
let retry = store.save_bytes(project_root, &raw)?;
assert_eq!(first.revision, retry.revision); // no second revision or terminal source read
```

**See also:** [`RsStore::save`](rs-store-save.md#u82),
[`RsSaveRequest`](rs-save-request.md#u80), and
[exact JSON identity I13](../../../interfaces/store-format.md#i13).
