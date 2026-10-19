# `RsStageVisit::append_assessment`

> record · method · implemented SimpleFlow slice · realizes the assessment-appending half of `EvaluateSimpleFlowGate` ([D04](../../details/simple_flow/01_run_design.md#d04))

<a id="u29"></a>

## U29 append_assessment

```rust
pub(crate) fn append_assessment(&mut self, assessment: RsAssessmentEntry)
```

Append one gate verdict to this visit. Earlier assessments — passed, failed or
unevaluated — stay in place: history is append-only so a later reader can re-examine
every judgment.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `assessment` | `RsAssessmentEntry` | produced by gate evaluation; revision and snapshot digest equal the visit state at evaluation | The verdict to record |

**Returns** — `()`.

**Errors** — none. The run calls this crate-visible method for entries returned
by [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20) or generated as
`Unevaluated` when evaluation or record persistence fails. The public run API
never accepts an arbitrary assessment from a caller.

**Logic**

```
1. debug_assert!(assessment.evaluated_revision == self.artifact_revision)
   debug_assert!(assessment.evaluated_snapshot_digest == self.artifact_snapshot_digest)
2. self.assessments.push(assessment)
3. // never overwrites; duplicates are data —
   // completion_authorized counts distinct gates, not entries
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Mutates the visit: appends one assessment |
| Complexity | O(1) amortized |
| Thread safety | `&mut self` — caller serializes |

**See also:** [`RsSimpleFlowRun::evaluate_gates`](03_evaluate_gates.md#u18), [`RsAssessmentEntry`](17_assessment_entry.md#u32), [`completion_authorized`](15_completion_authorized.md#u30).
