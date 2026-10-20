# `hash_artifact`

> record · function · implemented SimpleFlow slice · Rust-side addition with no matching UML operation; obtains one file's byte identity

<a id="u56"></a>

## U56 hash_artifact

```rust
pub fn hash_artifact(uri: &str) -> Result<RsArtifactDigest, String>
```

Read a repository-relative file and return its URI with a lowercase SHA-256 of exact
bytes. The CLI uses this when recording an activity; `verify_artifacts` uses it when
checking that bytes remain unchanged.

**Returns** — `Ok(RsArtifactDigest { uri, sha256 })` for a valid readable file;
otherwise `Err(String)` describing an invalid URI or read failure.

**Logic** — reject any path that fails [`valid_relative_path`](46_valid_relative_path.md#u62);
read the file bytes; hash them through [`sha256_bytes`](44_sha256_bytes.md#u60).

**See also:** [`RsArtifactDigest`](35_artifact_digest.md#u51), [`RsSimpleFlowRun::record_activity`](32_run_record_activity.md#u48).
