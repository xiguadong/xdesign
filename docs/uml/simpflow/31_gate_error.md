# `RsGateError`

> status · enum · designed-only · Rust-side addition with no SysML counterpart — the model leaves failure channels to the exchanges; the crate makes them typed

<a id="u46"></a>

## U46 RsGateError

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsGateError {
    CheckerMissing,
    CommandFailed,
    JudgmentRefused,
    JudgeNotPermitted,
    MissingEvidence,
}
```

How gate evaluation fails **to conclude** — returned by
[`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20) and recorded by
[`evaluate_gates`](03_evaluate_gates.md#u18) as an `Unevaluated` assessment. A rule
that does not hold is **not** here: it is data (`RsRuleReport { holds: false }`).
This split keeps "the gate says no" and "the gate cannot say" distinct.

| Variant | Meaning | Raised when |
| --- | --- | --- |
| `CheckerMissing` | Plan defect reaches runtime | Syntactic spec with `checker == None` (normally caught by `validate`) |
| `CommandFailed` | No trustworthy report | The command errored, timed out, or its report violates the report invariants |
| `JudgmentRefused` | No verdict | The judge returned `Unevaluated` |
| `JudgeNotPermitted` | Invalid authority | The judge's authority is not allowed by this semantic gate |
| `MissingEvidence` | Unsupported pass | A semantic judge passed without citing any current artifact or evidence URI |

**See also:** [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20), [`RsPlanError`](30_plan_error.md#u45), [`RsCheckpointOutcome`](27_checkpoint_outcome.md#u42).
