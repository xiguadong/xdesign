# `hash_artifact`

> `xdesign::core::record` · function · implemented xdesign slice · Rust-side addition with no matching UML operation; obtains one file's byte identity

<a id="u56"></a>

## hash_artifact

```rust
pub fn hash_artifact(uri: &str) -> Result<RsArtifactDigest, String>
```

Read a repository-relative file and return its URI with a lowercase SHA-256 of exact
bytes. The CLI uses this when recording an activity; `verify_artifacts` uses it when
checking that bytes remain unchanged.

**Returns** — `Ok(RsArtifactDigest { uri, sha256 })` for a valid readable file;
otherwise `Err(String)` describing an invalid URI or read failure.

**Logic** — reject any path that fails [`valid_relative_path`](valid-relative-path.md#u62);
read the file bytes; hash them through [`sha256_bytes`](sha256-bytes.md#u60).

**See also:** [`RsArtifactDigest`](rs-artifact-digest.md#u51), [`RsFlowRun::record_activity`](rs-flow-run-record-activity.md#u48).
