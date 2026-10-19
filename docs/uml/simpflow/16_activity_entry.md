# `RsActivityEntry`

> record · struct · designed-only · realizes `ActivityRecord` ([E02](../../details/02_records_and_evidence.md#e02))

<a id="u31"></a>

## U31 RsActivityEntry

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsActivityEntry {
    pub description: String,
    pub performed_by: String,
    pub occurred_at: String,
    pub artifact_uris: Vec<String>,
    pub artifacts_changed: bool,
    pub artifact_digests: Vec<RsArtifactDigest>,
    pub removed_artifact_uris: Vec<String>,
}
```

One recorded piece of work in a visit: what was done, by whom, when, and which
artifacts it touched. Plain data; the visit owns entries in append order.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `description` | `String` | `pub` | non-empty | What was done |
| 2 | `performed_by` | `String` | `pub` | non-empty | Agent, script or person identity |
| 3 | `occurred_at` | `String` | `pub` | ISO-8601 UTC | When it happened |
| 4 | `artifact_uris` | `Vec<String>` | `pub` | repository-relative paths | Artifacts produced or changed |
| 5 | `artifacts_changed` | `bool` | `pub` | true for any content change, including in-place edits | Explicit revision-bump signal |
| 6 | `artifact_digests` | `Vec<RsArtifactDigest>` | `pub` | exact bytes of created or updated artifacts | New content identities |
| 7 | `removed_artifact_uris` | `Vec<String>` | `pub` | only paths present in the prior snapshot | Deleted artifacts |

**Invariants** — the caller sets `artifacts_changed` from the actual write or
content comparison, independent of whether a URI changed. The run records this
declaration; artifact integrity checks must verify digests against the bytes.
If either digest or removal list is non-empty, `artifacts_changed` must be true.
If `artifacts_changed` is true, at least one digest or removal must be supplied.

**See also:** [`RsStageVisit`](12_stage_visit.md#u27), [`RsAssessmentEntry`](17_assessment_entry.md#u32), [records and evidence E02](../../details/02_records_and_evidence.md#e02).
