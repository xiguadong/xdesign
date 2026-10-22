# `RsCheckerRuleSpec`

> `xdesign::core::plan` · struct · implemented xdesign slice · realizes `CheckerRule` ([D06](../../flows/simple-flow/definition.md#d06))

<a id="u36"></a>

## RsCheckerRuleSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckerRuleSpec {
    pub rule_id: String,
    pub condition: String,
    pub reads: Vec<String>,
}
```

RsCheckerRuleSpec is one fixed rule of a checker: its identity and the condition its command reports. Use it when defining or reading what a syntactic gate actually establishes; the checker's verdict is the conjunction of its declared rules.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `rule_id` | `String` | `pub` | unique within its checker | Matches the report's entries |
| 2 | `condition` | `String` | `pub` | non-empty; exact pass condition per D06 | What the command checks |
| 3 | `reads` | `Vec<String>` | `pub` | non-empty | Input artifacts examined by the rule |

**See also:** [`RsCheckerSpec`](rs-checker-spec.md#u35), [`RsRuleReport`](rs-rule-report.md#u38), [rules D06](../../flows/simple-flow/definition.md#d06).
