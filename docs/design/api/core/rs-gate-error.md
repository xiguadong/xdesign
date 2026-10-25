# `RsGateError`

> `xdesign::core::status` · enum · implemented xdesign slice · Rust enum matching the model's `XDesignRustUml::RsGateError` failure-channel enum

<a id="u46"></a>

## RsGateError

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsGateError {
    CheckerMissing,
    CommandFailed,
    CheckerTimeout,
    JudgmentRefused,
    JudgeNotPermitted,
    MissingEvidence,
}
```

How gate evaluation fails **to conclude** — returned by
[`evaluate_checkpoint`](evaluate-checkpoint.md#u20) and recorded by
[`evaluate_gates`](rs-flow-run-evaluate-gates.md#u18) as an `Unevaluated` assessment. A rule
that does not hold is **not** here: it is data (`RsRuleReport { holds: false }`).
This split keeps "the gate says no" and "the gate cannot say" distinct.

| Variant | Meaning | Raised when |
| --- | --- | --- |
| `CheckerMissing` | Plan defect reaches runtime | Syntactic spec with `checker == None` (normally caught by `validate`) |
| `CommandFailed` | No trustworthy report | The command errored, inputs were unreadable, or its report violates the report invariants |
| `CheckerTimeout` | Deadline exceeded; no trustworthy report | The checker process tree outlived the checker deadline (default 600 s, overridden by `XDESIGN_CHECKER_TIMEOUT_SECS`) and was killed; the timeout is reported explicitly rather than hanging or being invented as a rule failure |
| `JudgmentRefused` | No verdict | The judge returned `Unevaluated` or an empty rationale |
| `JudgeNotPermitted` | Invalid authority | The judge's authority is not allowed by this semantic gate or its identity is blank |
| `MissingEvidence` | Unsupported pass | A semantic judge passed without a current artifact or evidence URI, or cited an invalid or missing file |

**See also:** [`evaluate_checkpoint`](evaluate-checkpoint.md#u20), [`RsPlanError`](rs-plan-error.md#u45), [`RsCheckpointOutcome`](rs-checkpoint-outcome.md#u42).
