# `sha256_bytes`

> `xdesign::core::plan` · utility function · implemented xdesign slice · Rust-side addition with no matching UML operation; hashes exact bytes

<a id="u60"></a>

## U60 sha256_bytes

```rust
pub fn sha256_bytes(bytes: &[u8]) -> String
```

Return a lowercase 64-digit SHA-256 hexadecimal string for exactly the supplied
bytes. There is no path lookup, newline normalization, or serialization inside
this function.

**Logic** — call `Sha256::digest(bytes)` and format the digest in lowercase hex.

**See also:** [`RsFlowPlan::freeze`](12_freeze_plan.md#u53), [`hash_artifact`](25_hash_artifact.md#u56), [`snapshot_digest`](26_snapshot_digest.md#u57).
