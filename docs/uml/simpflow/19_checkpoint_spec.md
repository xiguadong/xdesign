# `RsCheckpointSpec`

> plan · struct · designed-only · realizes `SimpleFlowCheckpoint` and its two adaptations ([D11](../../details/simple_flow/02_gates_and_rules.md#d11))

<a id="u34"></a>

## U34 RsCheckpointSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsCheckpointSpec {
    pub checkpoint_id: String,
    pub stage_id: String,
    pub kind: RsCheckKind,
    pub criteria: Vec<String>,
    pub checker: Option<RsCheckerSpec>,
    pub agent_may_judge: bool,
    pub human_may_judge: bool,
}
```

One gate as data. `kind` selects the evaluation path: a syntactic spec carries its
checker and no judge flags; a semantic spec carries criteria and judge flags and no
checker. This is the type-level realization of the model's checkpoint class and its
two adaptations.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `checkpoint_id` | `String` | `pub` | unique in the plan | Gate identifier |
| 2 | `stage_id` | `String` | `pub` | resolves to a stage (validated) | Stage this gate guards |
| 3 | `kind` | `RsCheckKind` | `pub` | — | Scripted or judged path |
| 4 | `criteria` | `Vec<String>` | `pub` | non-empty for every gate | Stated basis of the check or judgment |
| 5 | `checker` | `Option<RsCheckerSpec>` | `pub` | `Some` iff `kind == Syntactic` (validated) | The fixed check |
| 6 | `agent_may_judge` | `bool` | `pub` | at least one judge flag `true` for semantic gates | Agent authority |
| 7 | `human_may_judge` | `bool` | `pub` | see above | Human authority |

**Invariants** — enforced by [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26):
`(Syntactic, None)` and `(Semantic, Some(_))` combinations are rejected;
every gate has criteria; a semantic gate permits an agent or human. The two
SysML checkpoint adaptations are represented by the `kind` and checker/judge
fields of this one Rust type, with explicit realization dependencies to both.

**See also:** [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20), [`RsCheckerSpec`](20_checker_spec.md#u35), [`RsCheckKind`](26_check_kind.md#u41), [checkpoint class D11](../../details/simple_flow/02_gates_and_rules.md#d11).
