# `RsCheckpointOutcome`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `CheckpointOutcome` ([E04](../../evidence-and-revisions.md#e04))

<a id="u42"></a>

## RsCheckpointOutcome

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsCheckpointOutcome {
    Unevaluated,
    Failed,
    Passed,
}
```

RsCheckpointOutcome is the vocabulary for one checkpoint assessment. Use it to record or read whether a gate passed, failed, or remained unevaluated at a revision; only Passed at the visited revision counts toward completion.

| Variant | Meaning | Produced when |
| --- | --- | --- |
| `Unevaluated` | No conclusion | The channel failed (`RsGateError`) or the judge declined |
| `Failed` | Judged, does not hold | A rule failed, or a judge returned failure |
| `Passed` | Judged, holds | Every rule held, or a judge passed the criteria |

**See also:** [`RsAssessmentEntry`](rs-assessment-entry.md#u32), [`RsJudge::review`](rs-judge-review.md#u24).
