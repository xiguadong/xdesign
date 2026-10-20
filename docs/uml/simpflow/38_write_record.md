# `RsSimpleFlowRun::write_record`

> run · method · implemented SimpleFlow slice · Rust-side addition with no matching UML operation; writes the executable checker snapshot

<a id="u54"></a>

## U54 write_record

```rust
pub fn write_record(&self) -> Result<(), String>
```

Write a re-readable JSON record with the run ID, `execution_type = "SimpleFlowRun"`,
frozen plan identity, wish URI, run status, current-visit index, and all visits,
activities, artifact digests, and assessments. The checker commands read this file.

**Returns** — `Ok(())` after the temporary JSON file is renamed to
`run_record_uri`, or an error string for an invalid relative path, serialization,
directory creation, write, or rename failure.

**Logic** — construct a borrowed `RunRecord`; serialize to pretty JSON; create the
parent directory; write a UUID-suffixed temporary file beside the destination; rename
it to the destination. Run methods use an internal `persist` wrapper that records
failures in `last_error()` and prevents failed writes from authorizing advancement.

**Limit** — this is a local JSON snapshot under `cache/` in the CLI example. It is
not durable multi-user persistence.

**See also:** [`RsSimpleFlowRun`](02_simple_flow_run.md#u17), [`evaluate_gates`](03_evaluate_gates.md#u18), [`RsCheckCommand::run_check`](07_run_check.md#u22).
