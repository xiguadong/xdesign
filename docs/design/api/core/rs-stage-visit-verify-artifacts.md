# `RsStageVisit::verify_artifacts`

> `xdesign::core::record` · method · implemented xdesign slice · Rust-side addition with no matching UML operation; checks the current byte snapshot

<a id="u55"></a>

## verify_artifacts

```rust
pub fn verify_artifacts(&self) -> bool
```

Check that a visit's stored canonical snapshot digest matches its artifact list and
that each resolved artifact file still has its recorded SHA-256. Completed
snapshot-enabled visits resolve through their retained URI map; current and legacy
visits resolve live URIs. This prevents a
current gate or advancement from relying on an out-of-band file edit.

**Returns** — `true` only if the list has unique URIs, every digest is lowercase
64-digit SHA-256, the canonical list hash equals `artifact_snapshot_digest`, and
every file can be read with matching bytes. A missing file or invalid value returns
`false`.

**Logic** — recompute [`snapshot_digest`](snapshot-digest.md#u57); reject a
mismatch; track seen URIs; validate each digest; call [`hash_artifact`](hash-artifact.md#u56)
for each resolved URI and compare hashes. Detailed failures are available through
[`artifact_diagnostics`](rs-stage-visit-artifact-diagnostics.md#u115).

**See also:** [`RsStageVisit`](rs-stage-visit.md#u27), [`evaluate_checkpoint`](evaluate-checkpoint.md#u20).
