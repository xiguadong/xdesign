# `RsRuleReport`

> `xdesign::core::ports` · struct · implemented xdesign slice · part of the `CheckCommandExchange` report ([D10](../../flows/simple-flow/views.md#d10))

<a id="u38"></a>

## RsRuleReport

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsRuleReport {
    pub rule_id: String,
    pub holds: bool,
    pub diagnostic: Option<String>,
}
```

RsRuleReport is one rule's verdict inside a gate report: whether it held and the diagnostic when it did not. Use it to read or retain per-rule evidence rather than collapsing a checker to a single pass/fail.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `rule_id` | `String` | `pub` | matches an `RsCheckerRuleSpec` | Which rule |
| 2 | `holds` | `bool` | `pub` | — | The verdict |
| 3 | `diagnostic` | `Option<String>` | `pub` | `Some` whenever `holds == false` | Why it failed |

**See also:** [`RsGateReport`](rs-gate-report.md#u37), [`RsCheckerRuleSpec`](rs-checker-rule-spec.md#u36).
