# `RsStageVisit::append_assessment`

> `xdesign::core::record` · method · implemented xdesign slice · realizes the assessment-appending half of `EvaluateSimpleFlowGate` ([D04](../../flows/simple-flow/implementation.md#d04))

<a id="u29"></a>

## append_assessment

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
by [`evaluate_checkpoint`](evaluate-checkpoint.md#u20) or generated as
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

**See also:** [`RsFlowRun::evaluate_gates`](rs-flow-run-evaluate-gates.md#u18), [`RsAssessmentEntry`](rs-assessment-entry.md#u32), [`completion_authorized`](rs-stage-visit-completion-authorized.md#u30).
