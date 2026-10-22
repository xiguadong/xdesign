# `RsFlowPlan`

> `xdesign::core::plan` · struct · implemented xdesign slice · realizes `designedSimpleFlow` ([D01](../../flows/simple-flow/definition.md#d01))

<a id="u25"></a>

## RsFlowPlan

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsFlowPlan {
    pub plan_id: String,
    pub plan_uri: String,
    pub plan_sha256: String,
    pub initial_stage_id: String,
    pub stages: Vec<RsStageSpec>,
    pub checkpoints: Vec<RsCheckpointSpec>,
}
```

RsFlowPlan is a designed flow expressed as data: its stages, checkpoints, checkers, rules, and initial stage. Use it to validate, freeze, or execute a plan; the built-in flows construct it through their blueprint functions.

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
| [`freeze`](rs-flow-plan-freeze.md#u53) | public, writes the frozen plan | full entry |
| [`validate`](rs-flow-plan-validate.md#u26) | public | full entry |
| `initial_stage() -> &RsStageSpec` | public getter | trivial |
| `stages() -> &[RsStageSpec]` | public getter | trivial |
| `stage(id: &str) -> Option<&RsStageSpec>` | public lookup | trivial: first match or `None` |
| `next_stage(id: &str) -> Option<&RsStageSpec>` | public lookup | trivial: first entry of `next_stage_ids` |
| `checkpoints_of(stage_id: &str) -> Vec<&RsCheckpointSpec>` | public lookup | trivial: plan order, filtered by stage |

**See also:** [`create_run`](create-run.md#u16), [`RsStageSpec`](rs-stage-spec.md#u33), [`RsCheckpointSpec`](rs-checkpoint-spec.md#u34), [designed flow D01](../../flows/simple-flow/definition.md#d01).
