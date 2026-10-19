# `RsCheckpointOutcome`

> status · enum · designed-only · mirrors `CheckpointOutcome` ([E04](../../details/02_records_and_evidence.md#e04))

<a id="u42"></a>

## U42 RsCheckpointOutcome

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsCheckpointOutcome {
    Unevaluated,
    Failed,
    Passed,
}
```

A verdict's vocabulary. Only `Passed` at the visited revision counts toward
authorization ([U30](15_completion_authorized.md#u30)).

| Variant | Meaning | Produced when |
| --- | --- | --- |
| `Unevaluated` | No conclusion | The channel failed (`RsGateError`) or the judge declined |
| `Failed` | Judged, does not hold | A rule failed, or a judge returned failure |
| `Passed` | Judged, holds | Every rule held, or a judge passed the criteria |

**See also:** [`RsAssessmentEntry`](17_assessment_entry.md#u32), [`RsJudge::review`](09_review.md#u24).
