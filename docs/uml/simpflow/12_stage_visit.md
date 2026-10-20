# `RsStageVisit`

> record · struct · implemented SimpleFlow slice · realizes `StageRecord` ([E03](../../details/02_records_and_evidence.md#e03))

<a id="u27"></a>

## U27 RsStageVisit

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsStageVisit {
    stage_id: String,
    visit_index: u32,
    pub(crate) status: RsStageStatus,
    pub(crate) artifact_revision: u32,
    pub(crate) artifact_snapshot_digest: String,
    pub(crate) artifact_digests: Vec<RsArtifactDigest>,
    activities: Vec<RsActivityEntry>,
    assessments: Vec<RsAssessmentEntry>,
}
```

One visit of one stage. Owns its activities and assessments as `Vec` entries; earlier
visits remain in order. Corrective activity stays on the blocked current visit;
reopening a completed terminal stage appends a new visit at the next index.

**Fields** (UML attributes compartment)

| # | Field | Type | Visibility | Constraint / invariant | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `stage_id` | `String` | private; read via getters | matches a plan stage | Which stage this visit is for |
| 2 | `visit_index` | `u32` | private | 0 for the first visit of a stage, +1 per rework | Visit ordinal |
| 3 | `status` | `RsStageStatus` | `pub(crate)`; mutated by run methods | current visit agrees with run state | Visit lifecycle |
| 4 | `artifact_revision` | `u32` | `pub(crate)` | starts at 0; bumped by `record_activity` on declared artifact change | The revision assessments cite |
| 5 | `artifact_snapshot_digest` | `String` | `pub(crate)` | SHA-256 of canonical artifact digest list; empty-set digest at creation | Current byte snapshot identity |
| 6 | `artifact_digests` | `Vec<RsArtifactDigest>` | `pub(crate)` | unique URI entries, sorted when hashed | Current artifact bytes |
| 7 | `activities` | `Vec<RsActivityEntry>` | private; read via `activities()` | append-only | Work done in this visit |
| 8 | `assessments` | `Vec<RsAssessmentEntry>` | private; read via `assessments()` | append-only | Gate verdicts on this visit |

**Methods**

| Method | Kind | Page |
| --- | --- | --- |
| [`record_activity`](13_record_activity.md#u28) | crate-visible, mutating | full entry |
| [`append_assessment`](14_append_assessment.md#u29) | crate-visible, mutating | full entry |
| [`completion_authorized`](15_completion_authorized.md#u30) | public query | full entry |
| [`verify_artifacts`](39_verify_artifacts.md#u55) | public query and file read | full entry |
| [`evidence_uris`](47_evidence_uris.md#u63) | public query | full entry |
| [`verify_passed_evidence`](48_verify_passed_evidence.md#u64) | crate-visible query and file read | full entry |
| `pending(stage_id: String, visit_index: u32) -> Self` | crate-visible constructor | trivial: starts at revision 0 with empty artifact set; used by `create_run` and terminal reopen |
| `stage_id()`, `visit_index()`, `status()`, `artifact_revision()`, `artifact_snapshot_digest()`, `artifact_digests()`, `activities()`, `assessments()` | getters | trivial |

**See also:** [`RsSimpleFlowRun`](02_simple_flow_run.md#u17), [`RsActivityEntry`](16_activity_entry.md#u31), [`RsAssessmentEntry`](17_assessment_entry.md#u32), [records E03](../../details/02_records_and_evidence.md#e03).
