# `RsUnusedCheck`

> `xdesign::wish_verify::factory` · struct and trait implementation · implemented wish-verify slice · Rust-side addition with no matching UML class; satisfies the generic engine's check boundary for an all-semantic plan

<a id="u77"></a>

## RsUnusedCheck

```rust
pub struct RsUnusedCheck;

impl RsCheckCommand for RsUnusedCheck { /* run_check */ }
```

RsUnusedCheck is a never-invoked check-command stub required by the generic construction signature for all-semantic WishVerifyFlow plans. Use it only where that boundary is structurally required; reaching it at runtime is a defect.

**Why this is a Rust-side addition** — the model's wish-verify design has no checker
element to realize; the stub exists only because the generic Rust engine's construction
signature carries the check boundary for flows that do use it. It is the inverse of
[`RsProcessCheck`](../simple-flow/rs-process-check.md#u58): that adapter invokes real
commands; this one must never run. The behavior tests assert it is never called (case
WV-01 and the happy path WV-02 of
[P28](../../../records/plans/wish-verify.md#p28)).

**Contract** — `run_check` ignores its arguments and always returns
`Err(RsGateError::CommandFailed)`. It reads and writes nothing and retains no report.

**Logic**

```
1. return Err(RsGateError::CommandFailed)   — unconditionally; an all-semantic
   plan never reaches this, and reaching it is a defect to surface
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | None |
| Complexity | O(1) |
| Thread safety | Stateless; `Send` by construction |

**See also:** [`RsCheckCommand`](../core/rs-check-command.md#u21), [`RsGateError`](../core/rs-gate-error.md#u46), [`create_wish_verify_run`](create-wish-verify-run.md#u76), [`RsProcessCheck`](../simple-flow/rs-process-check.md#u58).
