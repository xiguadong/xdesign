# `python_command`

<a id="u112"></a>

## python_command

`xdesign::simple_flow::resources` · language-side helper under the modeled
`RsCheckerResources` allocation. It realizes the existing `CheckerExecutorBundle` or
`ProjectCheckerExecution` responsibility; no separate SysML action is claimed. Splitting
the resource adapter into these helpers is a Rust implementation choice.

```rust
pub fn python_command(checker: &RsCheckerSpec, attempt: &str, commands: &BTreeMap<String, String>) -> Result<Command, String>
```

Verify retained file digests and construct a Python process with explicit project cwd, interpreter, bundle root, parser and command mapping. Clear inherited Python import settings, disable user-site and bytecode output, and create private temporary/cache directories under the supplied attempt. Return the command without executing it; callers apply the checker deadline and retain outputs. Legacy checkers without a bundle retain their source path.

The owning behavior and limits are [checker execution](../../checker-execution.md#d111);
verification is [ISO-01–09](../../../verification/design.md#d112).
