# `command_map`

<a id="u113"></a>

## command_map

`xdesign::simple_flow::resources` · language-side helper under the modeled
`RsCheckerResources` allocation. It realizes the existing `CheckerExecutorBundle` or
`ProjectCheckerExecution` responsibility; no separate SysML action is claimed. Splitting
the resource adapter into these helpers is a Rust implementation choice.

```rust
pub fn command_map(plan: &RsFlowPlan) -> BTreeMap<String, String>
```

Project each logical command source identifier to its retained entry URI, falling back to the original command only for legacy plans without a bundle. This map lets built-in design checks inspect exact retained commands in the selected project; it performs no filesystem search or execution.

The owning behavior and limits are [checker execution](../../checker-execution.md#d111);
verification is [ISO-01–09](../../../verification/design.md#d112).
