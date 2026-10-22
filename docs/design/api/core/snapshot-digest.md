# `snapshot_digest`

> `xdesign::core::record` · function · implemented xdesign slice · Rust-side addition with no matching UML operation; canonicalizes an artifact set

<a id="u57"></a>

## snapshot_digest

```rust
pub fn snapshot_digest(artifacts: &[RsArtifactDigest]) -> String
```

Return the SHA-256 of a visit's artifact identity list. The encoding is URI-sort
order, one record per artifact: UTF-8 URI, NUL, lowercase file SHA-256, LF. An empty
list hashes the empty byte string.

**Parameters** — `artifacts` is the caller's list; the function sorts a temporary
view and does not mutate it. **Returns** — a lowercase SHA-256 string. This function
does not validate duplicate URIs or file bytes; [`verify_artifacts`](rs-stage-visit-verify-artifacts.md#u55)
performs those checks.

**Logic** — sort references lexicographically by URI; append each encoded record to
bytes; pass the bytes to [`sha256_bytes`](sha256-bytes.md#u60).

**See also:** [`RsArtifactDigest`](rs-artifact-digest.md#u51), [`RsStageVisit`](rs-stage-visit.md#u27).
