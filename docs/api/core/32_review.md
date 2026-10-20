# `RsJudge::review`

> `xdesign::core::ports` · trait method · implemented xdesign slice · realizes the `JudgmentExchange` invocation ([D10](../../details/simple_flow/03_structure_graphs.md#d10))

<a id="u24"></a>

## U24 review

```rust
fn review(&self, criteria: &[String], evidence_uris: &[String])
    -> (RsCheckpointOutcome, String)
```

Present a semantic gate's criteria and the visit's artifact and evidence URIs to a
permitted judge; return the verdict and its rationale. The trait is synchronous in
this slice. The local [`RsSuppliedJudge`](35_supplied_judge.md#u59) returns a caller's
supplied verdict without independently reviewing those inputs.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `criteria` | `&[String]` | non-empty, from the gate's spec | What must be judged |
| `evidence_uris` | `&[String]` | artifact and prior evidence URIs from the current visit | What the judge reads |

**Returns**

| Field | Type | Guarantee |
| --- | --- | --- |
| outcome | `RsCheckpointOutcome` | `Passed` or `Failed` is a judgment; `Unevaluated` means the judge declined and the caller records `JudgmentRefused` |
| rationale | `String` | non-empty whenever outcome is `Passed` or `Failed`; states what was read and why it holds or fails |

**Errors** — returned in-band as `Unevaluated`, never as an exception; see the
implementor contract on [`RsJudge`](31_judge.md#u23).

**Logic** (caller side, for context)

```
1. caller verifies judge.authority() is permitted by the gate's flags
2. (outcome, rationale) := judge.review(spec.criteria, visit.evidence_uris())
3. if outcome == Unevaluated → Err(GateError::JudgmentRefused)
4. otherwise build the AssessmentEntry with judged_by = judge.authority()
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Whatever the judge does to reach a verdict; the call itself mutates nothing |
| Complexity | Out of band; no timeout is designed ([D50](../../details/simple_flow/06_rust_uml_design.md#d50)) |
| Thread safety | `&self`; implementations must be `Send` |

**See also:** [`RsJudge`](31_judge.md#u23), [`evaluate_checkpoint`](39_evaluate_checkpoint.md#u20), [`RsCheckpointOutcome`](04_checkpoint_outcome.md#u42).
