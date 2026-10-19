# `RsStageStatus`

> status · enum · designed-only · mirrors `StageStatus` ([E02](../../details/02_records_and_evidence.md#e02))

<a id="u40"></a>

## U40 RsStageStatus

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsStageStatus {
    Pending,
    Active,
    Blocked,
    Completed,
}
```

One visit's lifecycle vocabulary. Exactly one visit is `Active` while its run is
active; the current visit is blocked or completed in the corresponding run state.

| Variant | Meaning | Entered when |
| --- | --- | --- |
| `Pending` | Planned, not yet entered | `create_run` step 3 |
| `Active` | The run's current visit | `create_run` step 4; `advance` enters the next stage; corrective activity resumes; terminal stage reopens |
| `Blocked` | Gate not authorized; retained for correction | `evaluate_gates` or `advance` finds a nonpassing latest gate assessment |
| `Completed` | Authorized and left | `advance` enter-next or complete paths |

**See also:** [`RsStageVisit`](12_stage_visit.md#u27), [`RsSimpleFlowRun::advance`](04_advance.md#u19).
