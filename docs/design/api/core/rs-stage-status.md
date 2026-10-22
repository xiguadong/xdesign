# `RsStageStatus`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `StageStatus` ([E02](../../evidence-and-revisions.md#e02))

<a id="u40"></a>

## RsStageStatus

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsStageStatus {
    Pending,
    Active,
    Blocked,
    Completed,
}
```

RsStageStatus is the lifecycle vocabulary for one stage visit. Use it to record or read whether a visit is active, blocked, completed, or reopened; exactly one visit is active for a running flow.

| Variant | Meaning | Entered when |
| --- | --- | --- |
| `Pending` | Planned, not yet entered | `create_run` step 3 |
| `Active` | The run's current visit | `create_run` step 4; `advance` enters the next stage; corrective activity resumes; terminal stage reopens |
| `Blocked` | Gate not authorized; retained for correction | `evaluate_gates` or `advance` finds a nonpassing latest gate assessment |
| `Completed` | Authorized and left | `advance` enter-next or complete paths |

**See also:** [`RsStageVisit`](rs-stage-visit.md#u27), [`RsFlowRun::advance`](rs-flow-run-advance.md#u19).
