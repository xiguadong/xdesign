# `RsAssessmentEntry`

> `xdesign::core::record` · struct · implemented xdesign slice · realizes `CheckpointAssessment` ([E04](../../evidence-and-revisions.md#e04))

<a id="u32"></a>

## RsAssessmentEntry

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

RsAssessmentEntry is one recorded gate verdict for a checkpoint at a revision. Use it to append or read the judgments a visit accumulates; the latest assessment at the visited revision governs authorization.

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
| `unevaluated(id: String, revision: u32, digest: String, error: RsGateError, authority: RsJudgeAuthority) -> Self` | constructor | trivial: builds the entry [`evaluate_gates`](rs-flow-run-evaluate-gates.md#u18) records when a gate cannot conclude; `outcome == Unevaluated`, `judged_by` names the channel, and `rationale` names the error |

**See also:** [`append_assessment`](rs-stage-visit-append-assessment.md#u29), [`completion_authorized`](rs-stage-visit-completion-authorized.md#u30), [`RsGateError`](rs-gate-error.md#u46), [assessments E04](../../evidence-and-revisions.md#e04).
