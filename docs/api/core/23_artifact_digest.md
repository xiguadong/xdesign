# `RsArtifactDigest`

> `xdesign::core::record` · struct · implemented xdesign slice · Rust-side evidence identity for the artifact revision carried by `StageRecord` ([E03](../../details/02_records_and_evidence.md#e03))

<a id="u51"></a>

## U51 RsArtifactDigest

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsArtifactDigest {
    pub uri: String,
    pub sha256: String,
}
```

One artifact's immutable byte identity. `uri` locates a repository artifact;
`sha256` is the lowercase 64-digit SHA-256 of that file's exact bytes. The
controller computes the hash when recording a changed artifact, and a fixed
checker recomputes it from the file it reads. A URI alone is insufficient
because a file can change in place.

| Field | Constraint | Meaning |
| --- | --- | --- |
| `uri` | non-empty repository-relative path, unique within one snapshot | Artifact location |
| `sha256` | lowercase 64-digit hexadecimal SHA-256, verified against bytes | Artifact content identity |

A visit's snapshot digest is SHA-256 of the UTF-8 encoding of its artifact
digests, sorted lexicographically by URI, one record per line encoded as
`uri`, NUL, `sha256`, LF. The empty snapshot hashes the empty byte string.
Every assessment cites the snapshot digest it evaluated. A checker report
must cite the same digest after checking the file bytes.

**See also:** [`RsActivityEntry`](21_activity_entry.md#u31),
[`RsStageVisit`](17_stage_visit.md#u27),
[`RsAssessmentEntry`](22_assessment_entry.md#u32),
[`RsGateReport`](33_gate_report.md#u37).
