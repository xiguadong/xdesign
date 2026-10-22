# `RsFlowPlan::freeze`

> `xdesign::core::plan` · method · implemented xdesign slice · Rust-side addition with no matching UML operation; binds a plan to exact JSON bytes

<a id="u53"></a>

## freeze

```rust
pub fn freeze(&mut self, uri: impl Into<String>) -> Result<(), RsPlanError>
```

Serialize `plan_id`, `initial_stage_id`, stages, and checkpoints as pretty JSON at a
repository-relative path. Store that path and the SHA-256 of its exact bytes in the
runtime plan, outside the hashed file. A later `validate` rereads the file, so changed
bytes invalidate the plan.

**Parameters** — `uri` must contain only normal relative path components.
**Returns** — `Ok(())` after writing and validating the file, or
`RsPlanError::InvalidPlanFile` for an invalid path, I/O or serialization failure.
The final `validate()` call can return any of the variants listed in
[`U26`](rs-flow-plan-validate.md#u26) for inconsistent plan data. The method may
overwrite an existing path; callers should
choose an output path whose contents they intend to replace.

**Logic** — validate the URI shape; serialize the frozen fields; create the parent
directory; write the bytes; set `plan_uri` and `plan_sha256`; call `validate()`.

**See also:** [`RsFlowPlan`](rs-flow-plan.md#u25), [`validate`](rs-flow-plan-validate.md#u26), [`designed_simple_flow_plan`](../simple-flow/designed-simple-flow-plan.md#u52).
