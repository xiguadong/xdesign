# `RsFlowStatus`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `FlowStatus` ([E01](../../details/02_records_and_evidence.md#e01))

<a id="u39"></a>

## U39 RsFlowStatus

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsFlowStatus {
    Pending,
    Active,
    Blocked,
    Completed,
}
```

The run's lifecycle vocabulary. Transitions are made only by
[`create_run`](46_create_run.md#u16), [`evaluate_gates`](41_evaluate_gates.md#u18),
[`advance`](42_advance.md#u19), [`record_activity`](43_run_record_activity.md#u48),
and [`reopen_stage`](44_reopen_stage.md#u49); see the state design
[D48](../../details/simple_flow/06_rust_uml_design.md#d48).

| Variant | Meaning | Entered when |
| --- | --- | --- |
| `Pending` | Declared run state, not entered by the current constructor | No current `create_run` path constructs a pending run; construction returns `Active` |
| `Active` | Running; exactly one visit is active | `create_run` completes; corrective activity resumes; terminal stage reopens |
| `Blocked` | A gate was not authorized | `evaluate_gates` records a non-passed outcome |
| `Completed` | Terminal visit authorized | `advance` returns `CompleteRun` |

**See also:** [`RsStageStatus`](02_stage_status.md#u40), [`RsFlowRun`](40_flow_run.md#u17).
