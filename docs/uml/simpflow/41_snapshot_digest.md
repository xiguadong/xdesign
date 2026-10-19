# `snapshot_digest`

> record · function · implemented SimpleFlow slice · canonicalizes an artifact set

<a id="u57"></a>

## U57 snapshot_digest

```rust
pub fn snapshot_digest(artifacts: &[RsArtifactDigest]) -> String
```

Return the SHA-256 of a visit's artifact identity list. The encoding is URI-sort
order, one record per artifact: UTF-8 URI, NUL, lowercase file SHA-256, LF. An empty
list hashes the empty byte string.

**Parameters** — `artifacts` is the caller's list; the function sorts a temporary
view and does not mutate it. **Returns** — a lowercase SHA-256 string. This function
does not validate duplicate URIs or file bytes; [`verify_artifacts`](39_verify_artifacts.md#u55)
performs those checks.

**Logic** — sort references lexicographically by URI; append each encoded record to
bytes; pass the bytes to [`sha256_bytes`](44_sha256_bytes.md#u60).

**See also:** [`RsArtifactDigest`](35_artifact_digest.md#u51), [`RsStageVisit`](12_stage_visit.md#u27).
