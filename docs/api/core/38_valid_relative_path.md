# `valid_relative_path`

> `xdesign::core::plan` · utility function · implemented xdesign slice · Rust-side addition with no matching UML operation; checks repository path shape

<a id="u62"></a>

## U62 valid_relative_path

```rust
pub fn valid_relative_path(s: &str) -> bool
```

Return `true` for a nonempty relative path whose components, as parsed by
`std::path::Path`, are all normal. Absolute paths and parent components fail;
the path parser may normalize an interior `.` before this check. The function checks path shape;
it does not establish that a file exists or that the path stays inside a repository
after resolving symlinks.

**Logic** — construct a `Path`, reject empty or absolute values, then require
every component to be `Component::Normal`.

**See also:** [`create_run`](46_create_run.md#u16), [`hash_artifact`](25_hash_artifact.md#u56), [`RsFlowPlan::freeze`](12_freeze_plan.md#u53).
