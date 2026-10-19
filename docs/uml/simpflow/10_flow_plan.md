# `RsFlowPlan`

> plan · struct · designed-only · realizes `designedSimpleFlow` ([D01](../../details/simple_flow/01_run_design.md#d01))

<a id="u25"></a>

## U25 RsFlowPlan

```rust
pub struct RsFlowPlan {
    pub plan_id: String,
    pub plan_uri: String,
    pub plan_sha256: String,
    pub initial_stage_id: String,
    pub stages: Vec<RsStageSpec>,
    pub checkpoints: Vec<RsCheckpointSpec>,
}
```

The designed flow as data: the stage and checkpoint specifications a run consults
instead of hard-coding the plan. The caller builds the value from the selected flow
definition; `create_run` validates it and takes ownership. The plan's metadata
identifies a frozen JSON file containing its stage and checkpoint data.
`plan_sha256` hashes that file's exact UTF-8 bytes; it is stored outside the
hashed file, in this runtime value and the run record. The plan is validated by
[`validate`](11_flow_plan_validate.md#u26), immutable thereafter.

**Fields** (UML attributes compartment)

| # | Field | Type | Visibility | Constraint / invariant | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `plan_id` | `String` | `pub` | exactly `designedSimpleFlow` | Modeled usage identity |
| 2 | `plan_uri` | `String` | `pub` | non-empty repository-relative JSON path | Frozen plan file |
| 3 | `plan_sha256` | `String` | `pub` | lowercase 64-digit hex of exact plan file bytes | Plan integrity anchor |
| 4 | `initial_stage_id` | `String` | `pub`; read via `initial_stage()` | must match a stage after validation | Where every run starts |
| 5 | `stages` | `Vec<RsStageSpec>` | `pub`; read via `stages()` | exactly 4 for the designed flow, in flow order | Stage specifications |
| 6 | `checkpoints` | `Vec<RsCheckpointSpec>` | `pub`; read via `checkpoints_of()` | each `stage_id` matches a stage after validation | Gate specifications |

**Methods**

| Method | Kind | Page |
| --- | --- | --- |
| [`validate`](11_flow_plan_validate.md#u26) | public | full entry |
| `initial_stage() -> &RsStageSpec` | public getter | trivial |
| `stages() -> &[RsStageSpec]` | public getter | trivial |
| `stage(id: &str) -> Option<&RsStageSpec>` | public lookup | trivial: first match or `None` |
| `next_stage(id: &str) -> Option<&RsStageSpec>` | public lookup | trivial: first entry of `next_stage_ids` |
| `checkpoints_of(stage_id: &str) -> Vec<&RsCheckpointSpec>` | public lookup | trivial: plan order, filtered by stage |

**See also:** [`create_run`](01_create_run.md#u16), [`RsStageSpec`](18_stage_spec.md#u33), [`RsCheckpointSpec`](19_checkpoint_spec.md#u34), [designed flow D01](../../details/simple_flow/01_run_design.md#d01).
