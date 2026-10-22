# `RsSuppliedJudge`

> `xdesign::core::ports` · struct and trait implementation · implemented xdesign slice · Rust-side addition with no matching UML class; carries a caller's semantic verdict

<a id="u59"></a>

## RsSuppliedJudge

```rust
pub struct RsSuppliedJudge {
    pub authority: RsJudgeAuthority,
    pub identity: String,
    pub outcome: RsCheckpointOutcome,
    pub rationale: String,
}

impl RsJudge for RsSuppliedJudge { /* review, authority, identity */ }
```

RsSuppliedJudge is a judge adapter for supplying an explicit agent or human judgment from the CLI and tests. Use it for local runs and controlled fixtures; production judgment integrations supply their own RsJudge.

**Contract** — [`evaluate_checkpoint`](evaluate-checkpoint.md#u20) rejects an
unpermitted authority, empty identity, empty rationale, refusal, and a pass without
an existing current verification artifact and retained evidence files. The caller
is responsible for a real reasoned judgment.

**Logic** — return the stored fields unchanged; leave validation to the gate
evaluator. No I/O or mutation occurs in this adapter.

**See also:** [`RsJudge`](rs-judge.md#u23), [`RsJudge::review`](rs-judge-review.md#u24), [verification gate D05](../../flows/simple-flow/definition.md#d05).
