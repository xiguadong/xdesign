# `RsCheckerBundle`

<a id="u110"></a>

## RsCheckerBundle

`xdesign::core::plan::RsCheckerBundle` is the optional executor identity in
`RsCheckerSpec::bundle`. It realizes `CheckerExecutorBundle` through the UML dependency
`rsCheckerBundleRealizes`; the flow resource adapter realizes `ProjectCheckerExecution`.

```rust
pub struct RsCheckerBundle {
    pub source: String,
    pub entry_uri: String,
    pub files: Vec<RsArtifactDigest>,
    pub python: Option<String>,
}
```

`source` is `builtin-v1` or `project`; `entry_uri` names a retained Python entry among
`files`. Each file has a normalized project-relative URI and a lowercase SHA-256 digest.
The optional `python` names a project-local environment executable under `cache/`.
Validate shape, unique paths, entry membership and digests before freezing a plan.
The stage adapter additionally requires run-confined executor paths and verified store
materials. Core owns data validation, without depending on resource extraction or SQLite.

Absent `bundle` preserves old serialized plans. New store-backed registrations freeze
exact resources and retain them from revision 1. Interpreter bytes and installed packages
are not part of the bundle; execution evidence records the interpreter actually used.
Source changes require a new run. See [execution design](../../checker-execution.md#d111)
and [acceptance cases](../../../verification/design.md#d112).
