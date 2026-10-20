# `RsCheckerRuleSpec`

> plan · struct · implemented SimpleFlow slice · realizes `CheckerRule` ([D06](../../details/simple_flow/02_gates_and_rules.md#d06))

<a id="u36"></a>

## U36 RsCheckerRuleSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckerRuleSpec {
    pub rule_id: String,
    pub condition: String,
    pub reads: Vec<String>,
}
```

One fixed rule of a checker: its identity and the documented condition the command
applies. The condition text is the model's declaration of the rule; the command's
output must report one verdict per `rule_id`, and the design's agreement check keeps
the two lists from drifting ([R01](../../requirements/01_baseline.md#r01),
`checkerRulesAreDeclared`).

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `rule_id` | `String` | `pub` | unique within its checker | Matches the report's entries |
| 2 | `condition` | `String` | `pub` | non-empty; exact pass condition per D06 | What the command checks |
| 3 | `reads` | `Vec<String>` | `pub` | non-empty | Input artifacts examined by the rule |

**See also:** [`RsCheckerSpec`](20_checker_spec.md#u35), [`RsRuleReport`](23_rule_report.md#u38), [rules D06](../../details/simple_flow/02_gates_and_rules.md#d06).
