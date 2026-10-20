# `RsRuleReport`

> `xdesign::core::ports` · struct · implemented xdesign slice · part of the `CheckCommandExchange` report ([D10](../../details/simple_flow/03_structure_graphs.md#d10))

<a id="u38"></a>

## U38 RsRuleReport

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsRuleReport {
    pub rule_id: String,
    pub holds: bool,
    pub diagnostic: Option<String>,
}
```

One rule's verdict inside a gate report: whether it held, and the diagnostic when it
did not. A rule that does not hold is data, never an error
([`RsGateError`](09_gate_error.md#u46) is reserved for "no conclusion possible").

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `rule_id` | `String` | `pub` | matches an `RsCheckerRuleSpec` | Which rule |
| 2 | `holds` | `bool` | `pub` | — | The verdict |
| 3 | `diagnostic` | `Option<String>` | `pub` | `Some` whenever `holds == false` | Why it failed |

**See also:** [`RsGateReport`](33_gate_report.md#u37), [`RsCheckerRuleSpec`](16_checker_rule_spec.md#u36).
