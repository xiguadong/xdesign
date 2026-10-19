# `RsStageSpec`

> plan · struct · designed-only · realizes the SimpleFlow subset of `Stage` ([F01](../../details/01_flows_and_checkpoints.md#f01))

<a id="u33"></a>

## U33 RsStageSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsStageSpec {
    pub stage_id: String,
    pub next_stage_ids: Vec<String>,
    pub exit_checkpoint_ids: Vec<String>,
    pub completes_flow: bool,
}
```

One stage of the designed flow as data: its successors, its exit gates, and whether
its completion finishes the flow.
The generic SysML `Stage` also permits nested subflows and `verifiesStage` links;
the designed four-stage flow uses neither, so this crate does not carry them.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `stage_id` | `String` | `pub` | unique in the plan | Stage identifier |
| 2 | `next_stage_ids` | `Vec<String>` | `pub` | each resolves to a stage (validated) | Successors in flow order |
| 3 | `exit_checkpoint_ids` | `Vec<String>` | `pub` | each resolves to a gate (validated) | Gates that must pass to leave |
| 4 | `completes_flow` | `bool` | `pub` | exactly one stage has `true` (verify) | Terminal flag |

**See also:** [`RsFlowPlan`](10_flow_plan.md#u25), [`RsCheckpointSpec`](19_checkpoint_spec.md#u34), [`RsSimpleFlowRun::advance`](04_advance.md#u19).
