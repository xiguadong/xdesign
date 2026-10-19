# `RsAssessmentEntry`

> record · struct · implemented SimpleFlow slice · realizes `CheckpointAssessment` ([E04](../../details/02_records_and_evidence.md#e04))

<a id="u32"></a>

## U32 RsAssessmentEntry

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsAssessmentEntry {
    pub checkpoint_id: String,
    pub evaluated_revision: u32,
    pub evaluated_snapshot_digest: String,
    pub outcome: RsCheckpointOutcome,
    pub judged_by: RsJudgeAuthority,
    pub assessor: String,
    pub rationale: String,
    pub evidence_uris: Vec<String>,
}
```

One recorded gate verdict. The checkpoint is named by identifier rather than held by
reference, so an entry stays serializable; the run resolves it against the plan when
authorization is counted.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `checkpoint_id` | `String` | `pub` | resolves against the plan | Which gate was judged |
| 2 | `evaluated_revision` | `u32` | `pub` | starts at 0; equals the revision judged | Freshness anchor for authorization |
| 3 | `evaluated_snapshot_digest` | `String` | `pub` | SHA-256 of exact artifact bytes judged | Freshness anchor beyond the revision counter |
| 4 | `outcome` | `RsCheckpointOutcome` | `pub` | — | The verdict |
| 5 | `judged_by` | `RsJudgeAuthority` | `pub` | `Script` for syntactic gates; `Agent`/`Human` for semantic | Judging authority |
| 6 | `assessor` | `String` | `pub` | non-empty | Who or what judged |
| 7 | `rationale` | `String` | `pub` | non-empty for `Passed`/`Failed` by a judge | Why the verdict holds |
| 8 | `evidence_uris` | `Vec<String>` | `pub` | repository-relative paths | What the judgment rests on |

**Associated functions**

| Function | Kind | Page |
| --- | --- | --- |
| `unevaluated(id: String, revision: u32, digest: String, error: RsGateError, authority: RsJudgeAuthority) -> Self` | constructor | trivial: builds the entry [`evaluate_gates`](03_evaluate_gates.md#u18) records when a gate cannot conclude; `outcome == Unevaluated`, `judged_by` names the channel, and `rationale` names the error |

**See also:** [`append_assessment`](14_append_assessment.md#u29), [`completion_authorized`](15_completion_authorized.md#u30), [`RsGateError`](31_gate_error.md#u46), [assessments E04](../../details/02_records_and_evidence.md#e04).
