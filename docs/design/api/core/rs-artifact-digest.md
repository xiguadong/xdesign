# `RsArtifactDigest`

> `xdesign::core::record` · struct · implemented xdesign slice · Rust-side evidence identity for the artifact revision carried by `StageRecord` ([E03](../../evidence-and-revisions.md#e03))

<a id="u51"></a>

## RsArtifactDigest

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsArtifactDigest {
    pub uri: String,
    pub sha256: String,
}
```

RsArtifactDigest is one artifact's immutable byte identity within a run. Use it to record which exact bytes a visit assessed, pairing the repository-relative URI with the lowercase SHA-256 of those bytes.

| Field | Constraint | Meaning |
| --- | --- | --- |
| `uri` | non-empty repository-relative path, unique within one snapshot | Artifact location |
| `sha256` | lowercase 64-digit hexadecimal SHA-256, verified against bytes | Artifact content identity |

A visit's snapshot digest is SHA-256 of the UTF-8 encoding of its artifact
digests, sorted lexicographically by URI, one record per line encoded as
`uri`, NUL, `sha256`, LF. The empty snapshot hashes the empty byte string.
Every assessment cites the snapshot digest it evaluated. A checker report
must cite the same digest after checking the file bytes.

**See also:** [`RsActivityEntry`](rs-activity-entry.md#u31),
[`RsStageVisit`](rs-stage-visit.md#u27),
[`RsAssessmentEntry`](rs-assessment-entry.md#u32),
[`RsGateReport`](rs-gate-report.md#u37).
