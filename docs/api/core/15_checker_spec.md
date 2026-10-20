# `RsCheckerSpec`

> `xdesign::core::plan` · struct · implemented xdesign slice · realizes `Checker` ([F02](../../details/01_flows_and_checkpoints.md#f02))

<a id="u35"></a>

## U35 RsCheckerSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckerSpec {
    pub checker_id: String,
    pub command_uri: String,
    pub establishes: Vec<RsConformanceKind>,
    pub rules: Vec<RsCheckerRuleSpec>,
}
```

The command a syntactic gate runs, and the rules its report must answer. The command
runs unchanged for every assessment; its per-rule report for the visited revision is
the assessment's evidence.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `checker_id` | `String` | `pub` | unique in the plan | Recorded as the assessment's assessor |
| 2 | `command_uri` | `String` | `pub` | repository-relative path of an existing command | What is invoked |
| 3 | `establishes` | `Vec<RsConformanceKind>` | `pub` | non-empty | Which conformance classes this checker establishes |
| 4 | `rules` | `Vec<RsCheckerRuleSpec>` | `pub` | non-empty | What decides the verdict |

**See also:** [`RsCheckerRuleSpec`](16_checker_rule_spec.md#u36), [`RsConformanceKind`](07_conformance_kind.md#u50), [`RsCheckCommand::run_check`](30_run_check.md#u22), [gates and rules D06](../../details/simple_flow/02_gates_and_rules.md#d06).
