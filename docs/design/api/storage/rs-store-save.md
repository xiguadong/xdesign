# `RsStore::save`

> `xdesign::storage` · method · implemented; STORE-01–STORE-11 evidence executed within disclosed limits · realizes
> `XDesignStorage::SaveOperation` in [D86](../../storage.md#d86); implementation projection
> `src/storage/{ops,db,blobs}.rs`.

<a id="u82"></a>

## save

```rust
use std::path::Path;

impl RsStore {
    pub fn save(
        &self,
        project_root: &Path,
        request: &RsSaveRequest,
    ) -> Result<RsOperationReceipt, RsStoreError>;
}
```

Validate and deterministically encode one typed request, then execute the exact-operation
save protocol. A successful return never has `Pending` outcome.

**Parameters**

- `project_root` is the safe local containment root for every
  [`RsSaveMaterial::source`](rs-save-material.md#u85). It does not reinterpret the already
  opened store root. Sources are project-relative regular files without symlink components
  and must not resolve into the store.
- `request` is the complete typed version-1 [`RsSaveRequest`](rs-save-request.md#u80),
  including its expected head, complete manifest, metadata, and result.

The method recursively sorts object members and emits no insignificant whitespace. Primitive
values use the pinned `serde_json` serializer's forms. Strict numeric validation and the
stored metadata/result representation use that same pinned numeric parser/serializer: a
number is accepted only when the representation persisted for it is an exact decode/re-encode
fixed point, so silent rounding is never committed. This deterministic compact JSON is
**not RFC 8785/JCS** and claims no interoperability across arbitrary JSON numbers. The
resulting exact bytes are hashed and retained; save then follows the same protocol as
[`save_bytes`](rs-store-save-bytes.md#u100), which instead preserves caller-supplied raw bytes.
Pure request and path-syntax validation precedes reservation and does not resolve the project
root or any source path. An identical exact request that already holds a terminal receipt
returns that receipt before the project root is canonicalized or any source is resolved.

**Returns and persistence**

| Case | Return | Durable effect |
| --- | --- | --- |
| New operation at expected head | `Ok(receipt)` with `Committed` and `revision = Some(expected + 1)` | Complete revision, manifest, blob references, receipt, and head advance commit together. |
| Existing item head differs, create collides, or immutable kind differs | `Ok(receipt)` with `Conflict` and `revision = None` | Conflict receipt commits; no revision/manifest/head effect; unreferenced published bytes remain doctor-visible. |
| Exact committed or conflict retry | Stored `Ok(receipt)` | None; returned before project-root canonicalization, source-path resolution, or new publication. |
| Exact pending retry | Final `Ok(receipt)`, or an error/unknown outcome | Resumes the reserved request without another operation. |
| Same ID, different valid request bytes | `Err(OperationMismatch)` | No new effect and no return of the other operation's result. |
| Uncertain publication/commit/response | `Err(UnknownOutcome { operation_id })` | Outcome may be pending, committed, or conflict; reconcile before new work. |

A conflict receipt's `result` is the proposed request result, not proof it was applied. Errors
after reservation can leave pending state, so callers must not infer "error means no effect."
Use [`outcome`](rs-store-outcome.md#u83) or retry the exact typed request before new work.

**Errors**

Malformed request/source declarations and source digest or size mismatch are `InvalidInput`;
unsupported request schema is `UnsupportedFormat`. A digest never stored is `NotFound`, while
a missing/altered committed blob is `Integrity`. Path violations are `UnsafePath`; replaced
root/database identity is `Integrity`. Local failures outside the uncertainty window are
`Io`/`Database`; uncertainty uses `UnknownOutcome`. Conflict remains receipt data, not an
error variant.

**Logic**

1. Encode `request` as deterministic compact JSON; hash those exact bytes.
2. Validate root identity and strict request/schema, all IDs/digests/URIs, metadata,
   complete-manifest uniqueness, exact-number fixed points, and material name equivalence.
   This pure request/path-syntax validation does not resolve the project root or sources.
3. In one short reservation transaction, inspect `operation_id`: reject different bytes,
   return an exact terminal receipt (before project-root canonicalization or source-path
   resolution), resume exact pending work, or insert pending receipt, exact request bytes,
   and digest reservations.
4. After reservation, canonicalize `project_root` and read every supplied source through EOF
   with the 256 MiB cap and require the declared digest/length. For `source = None`, accept
   only a committed blob row or a digest verifiably published by this exact operation's
   earlier attempt; an unrelated published or orphan blob is never adopted.
5. Stage, fsync, and no-replace publish all required new blobs; verify existing final bytes
   before deduplication and fsync containing directories.
6. `BEGIN IMMEDIATE`; recheck operation bytes and current item/head/kind. On conflict finalize
   only the conflict receipt. Otherwise create the item if needed and atomically insert blob
   metadata, the complete revision/manifest, committed receipt, and `head = expected + 1`.
7. Commit and synchronize, then read back the terminal receipt. Any uncertain step 5–7 failure
   becomes `UnknownOutcome(operation_id)`, never a fresh operation or a claim that no commit
   occurred.

No revision row is written before every referenced blob is published.

**Example** (illustrative, not executed):

```rust
let receipt = store.save(Path::new("."), &request)?;
match receipt.outcome {
    RsOperationOutcome::Committed => {
        assert_eq!(receipt.revision, Some(request.expected_revision + 1));
    }
    RsOperationOutcome::Conflict => assert_eq!(receipt.revision, None),
    RsOperationOutcome::Pending => {
        return Err(RsStoreError::Integrity("save returned pending".into()));
    }
}
```

**See also:** [`RsStore::save_bytes`](rs-store-save-bytes.md#u100),
[`RsOperationReceipt`](rs-operation-receipt.md#u90), and
[crash boundaries I12](../../../interfaces/store-format.md#i12).
