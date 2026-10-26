# `freeze_executors`

<a id="u111"></a>

## freeze_executors

`xdesign::simple_flow::resources` · language-side helper under the modeled
`RsCheckerResources` allocation. It realizes the existing `CheckerExecutorBundle` or
`ProjectCheckerExecution` responsibility; no separate SysML action is claimed. Splitting
the resource adapter into these helpers is a Rust implementation choice.

```rust
pub fn freeze_executors(plan: &mut RsFlowPlan, run_directory: &str) -> Result<Vec<RsArtifactDigest>, String>
```

Resolve embedded resources or explicitly selected project files, preserve the declared relative tree, materialize verified copies below the run executor directory, and replace each checker bundle with its retained entry and exact file digests. Return unique files for the registration manifest. Fail on missing files, source drift, unsafe paths or unavailable selected Python; registration commits no wish revision on failure.

The owning behavior and limits are [checker execution](../../checker-execution.md#d111);
verification is [ISO-01–09](../../../verification/design.md#d112).
