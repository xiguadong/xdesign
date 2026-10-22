# `RsFlowStatus`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `FlowStatus` ([E01](../../../interfaces/record-format.md#e01))

<a id="u39"></a>

## RsFlowStatus

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsFlowStatus {
    Pending,
    Active,
    Blocked,
    Completed,
}
```

RsFlowStatus is the lifecycle-state vocabulary for a run. Use it to inspect or compare run state; transitions are made only by the run engine, never by assigning the field directly.

| Variant | Meaning | Entered when |
| --- | --- | --- |
| `Pending` | Declared run state, not entered by the current constructor | No current `create_run` path constructs a pending run; construction returns `Active` |
| `Active` | Running; exactly one visit is active | `create_run` completes; corrective activity resumes; terminal stage reopens |
| `Blocked` | A gate was not authorized | `evaluate_gates` records a non-passed outcome |
| `Completed` | Terminal visit authorized | `advance` returns `CompleteRun` |

**See also:** [`RsStageStatus`](rs-stage-status.md#u40), [`RsFlowRun`](rs-flow-run.md#u17).
