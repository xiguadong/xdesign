# `RsCheckpointSpec`

> `xdesign::core::plan` · struct · implemented xdesign slice · realizes `SimpleFlowCheckpoint` and its two adaptations ([D11](../../flows/simple-flow/definition.md#d11))

<a id="u34"></a>

## RsCheckpointSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

RsCheckpointSpec is one gate expressed as plan data: its kind, criteria, checker bindings, and judgment permissions. Use it when validating, freezing, or inspecting a designed flow plan; the run engine consults it during evaluation.

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

**Invariants** — enforced by [`RsFlowPlan::validate`](rs-flow-plan-validate.md#u26):
`(Syntactic, None)` and `(Semantic, Some(_))` combinations are rejected;
every gate has criteria; a semantic gate permits an agent or human. The two
SysML checkpoint adaptations are represented by the `kind` and checker/judge
fields of this one Rust type, with explicit realization dependencies to both.

**See also:** [`evaluate_checkpoint`](evaluate-checkpoint.md#u20), [`RsCheckerSpec`](rs-checker-spec.md#u35), [`RsCheckKind`](rs-check-kind.md#u41), [checkpoint class D11](../../flows/simple-flow/definition.md#d11).
