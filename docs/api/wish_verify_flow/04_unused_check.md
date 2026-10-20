# `RsUnusedCheck`

> `xdesign::wish_verify::factory` · struct and trait implementation · implemented wish-verify slice · Rust-side addition with no matching UML class; satisfies the generic engine's check boundary for an all-semantic plan

<a id="u77"></a>

## U77 RsUnusedCheck

```rust
pub struct RsUnusedCheck;

impl RsCheckCommand for RsUnusedCheck { /* run_check */ }
```

The check-command stub [`create_wish_verify_run`](03_create_wish_verify_run.md#u76)
hands to the generic [`create_run`](../core/46_create_run.md#u16). The designed
wish-verify flow declares no syntactic checkpoint
([D62](../../details/wish_verify_flow/02_gates_and_judgment.md#d62)), so a conforming
plan contains no checker spec and the engine can never select a check command — yet
`create_run` requires a boundary value. The stub makes that boundary honest rather than
absent: every call returns `RsGateError::CommandFailed`, so an accidental invocation
surfaces as an error instead of a silent or fabricated pass (plan decisions 3 and 7,
[P26](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p26)).

**Why this is a Rust-side addition** — the model's wish-verify design has no checker
element to realize; the stub exists only because the generic Rust engine's construction
signature carries the check boundary for flows that do use it. It is the inverse of
[`RsProcessCheck`](../simple_flow/04_process_check.md#u58): that adapter invokes real
commands; this one must never run. The behavior tests assert it is never called (case
WV-01 and the happy path WV-02 of
[P28](../../prj_manage/09_wish_verify_flow_implementation_plan.md#p28)).

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

**See also:** [`RsCheckCommand`](../core/29_check_command.md#u21), [`RsGateError`](../core/09_gate_error.md#u46), [`create_wish_verify_run`](03_create_wish_verify_run.md#u76), [`RsProcessCheck`](../simple_flow/04_process_check.md#u58).
