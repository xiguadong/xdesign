# `RsCheckerSpec`

> `xdesign::core::plan` · struct · implemented xdesign slice · realizes `Checker` ([F02](../../adapters.md#f02))

<a id="u35"></a>

## RsCheckerSpec

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckerSpec {
    pub checker_id: String,
    pub command_uri: String,
    pub establishes: Vec<RsConformanceKind>,
    pub rules: Vec<RsCheckerRuleSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<RsCustomChecker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<RsCheckerBundle>,
}
```

RsCheckerSpec binds a syntactic gate to the command it runs and the rules its report must answer. Use it when reading a flow's gate data or when adding a fixed checker; judging is a separate boundary.

**Fields**

| # | Field | Type | Visibility | Constraint | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `checker_id` | `String` | `pub` | unique in the plan | Recorded as the assessment's assessor |
| 2 | `command_uri` | `String` | `pub` | logical built-in source identifier or project-relative script | Bundle entry is invoked for retained executors |
| 3 | `establishes` | `Vec<RsConformanceKind>` | `pub` | non-empty | Which conformance classes this checker establishes |
| 4 | `rules` | `Vec<RsCheckerRuleSpec>` | `pub` | non-empty | What decides the verdict |
| 5 | `custom` | `Option<RsCustomChecker>` | `pub` | absent for default; new bindings use `process` | Optional executor selection frozen in the plan |
| 6 | `bundle` | `Option<RsCheckerBundle>` | `pub` | valid entry and unique digest-bound files | Exact retained script resources for new registrations |

`RsCustomChecker` contains `runner: RsCustomRunner` (`Process` or compatibility-only `Cli`),
`cli: Option<String>` and `model: Option<String>`. The two optional strings use serde
default/omit-when-none attributes and remain only to decode historical plans. New bindings
use the process runner and `command_uri`; `Cli` is rejected for new registrations and is
never launched by a later check. The default omits `custom` to preserve prior frozen-plan bytes.
New store-backed registrations also carry a [retained executor bundle](rs-checker-bundle.md#u110); legacy plans without it retain their earlier path-based contract. The
[as-built model](../../checkpoint-library.md#d107) states the script-only current policy
and historical compatibility boundary.

**See also:** [`RsCheckerRuleSpec`](rs-checker-rule-spec.md#u36), [`RsConformanceKind`](rs-conformance-kind.md#u50), [`RsCheckCommand::run_check`](rs-check-command-run-check.md#u22), [gates and rules D06](../../flows/simple-flow/definition.md#d06).
