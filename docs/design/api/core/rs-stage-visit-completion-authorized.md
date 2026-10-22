# `RsStageVisit::completion_authorized`

> `xdesign::core::record` · method · implemented xdesign slice · realizes the authorization predicate bound in `SimpleFlowAuthorization` ([D16](../../flows/simple-flow/views.md#d16))

<a id="u30"></a>

## completion_authorized

```rust
pub fn completion_authorized(&self, plan: &RsFlowPlan) -> bool
```

Decide whether this visit authorizes leaving its stage: the **latest** assessment
of every exit gate must be `Passed` at this visit's current
`artifact_revision` and current artifact snapshot digest. Selecting the latest
entry first means a failed re-evaluation supersedes an earlier pass at the same
revision. Duplicate passes cannot compensate for a missing gate.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `plan` | `&RsFlowPlan` | the run's plan | Supplies the stage's exit gates |

**Returns**

| Type | Guarantee |
| --- | --- |
| `bool` | `true` iff the stage has at least one required gate and every required gate's latest assessment has `outcome == Passed`, `evaluated_revision == self.artifact_revision`, and `evaluated_snapshot_digest == self.artifact_snapshot_digest` |

**Errors** — none; the predicate is total.

**Logic**

```
1. required := plan.checkpoints_of(self.stage_id)
2. if required.is_empty(): return false
3. latest := map of checkpoint_id → last matching entry in self.assessments
4. return required.iter().all(|g| {
       let Some(a) = latest.get(g.checkpoint_id) else { return false };
       a.outcome == Passed &&
       a.evaluated_revision == self.artifact_revision &&
       a.evaluated_snapshot_digest == self.artifact_snapshot_digest
   })
```

Note: the model's predicate also requires a nested subflow, when present, to have
completed ([D16](../../flows/simple-flow/views.md#d16)). The designed
simple flow has no subflow, so this method covers the full condition for this crate;
a flow with subflows would need that clause — stated here, not silently dropped.

**Other**

| Item | Value |
| --- | --- |
| Side effects | None |
| Complexity | O(gates + assessments) |
| Thread safety | `&self`; re-entrant |

**Example** (illustrative)

```rust
let authorized = visit.completion_authorized(plan);
assert!(!authorized || visit.status() != RsStageStatus::Pending);
```

**See also:** [`RsFlowRun::advance`](rs-flow-run-advance.md#u19), [`record_activity`](rs-stage-visit-record-activity.md#u28), [authorization D16](../../flows/simple-flow/views.md#d16).
