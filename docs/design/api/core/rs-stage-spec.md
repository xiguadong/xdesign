# `RsStageSpec`

> `xdesign::core::plan` · struct · implemented xdesign slice · realizes the SimpleFlow subset of `Stage` ([F01](../../flow.md#f01))

<a id="u33"></a>

## RsStageSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsStageSpec {
    pub stage_id: String,
    pub next_stage_ids: Vec<String>,
    pub exit_checkpoint_ids: Vec<String>,
    pub completes_flow: bool,
}
```

RsStageSpec is one stage of a designed flow as data: its successors, exit gates, and whether it completes the flow. Use it when validating or inspecting a plan and when mapping a stage to its visit.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `stage_id` | `String` | `pub` | unique in the plan | Stage identifier |
| 2 | `next_stage_ids` | `Vec<String>` | `pub` | each resolves to a stage (validated) | Successors in flow order |
| 3 | `exit_checkpoint_ids` | `Vec<String>` | `pub` | each resolves to a gate (validated) | Gates that must pass to leave |
| 4 | `completes_flow` | `bool` | `pub` | exactly one stage has `true` (verify) | Terminal flag |

**See also:** [`RsFlowPlan`](rs-flow-plan.md#u25), [`RsCheckpointSpec`](rs-checkpoint-spec.md#u34), [`RsFlowRun::advance`](rs-flow-run-advance.md#u19).
