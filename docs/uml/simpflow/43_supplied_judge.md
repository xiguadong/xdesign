# `RsSuppliedJudge`

> ports · struct and trait implementation · implemented SimpleFlow slice · carries a caller's semantic verdict

<a id="u59"></a>

## U59 RsSuppliedJudge

```rust
pub struct RsSuppliedJudge {
    pub authority: RsJudgeAuthority,
    pub identity: String,
    pub outcome: RsCheckpointOutcome,
    pub rationale: String,
}

impl RsJudge for RsSuppliedJudge { /* review, authority, identity */ }
```

The local CLI and tests use this adapter to supply an explicit agent or human
verdict. `review` returns the stored outcome and rationale; the getters return
the stored authority and identity. It does not inspect the criteria or evidence
it receives and therefore does not independently verify the developed system.

**Contract** — [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20) rejects an
unpermitted authority, empty identity, empty rationale, refusal, and a pass without
an existing current verification artifact and retained evidence files. The caller
is responsible for a real reasoned judgment.

**Logic** — return the stored fields unchanged; leave validation to the gate
evaluator. No I/O or mutation occurs in this adapter.

**See also:** [`RsJudge`](08_judge.md#u23), [`RsJudge::review`](09_review.md#u24), [verification gate D05](../../details/simple_flow/02_gates_and_rules.md#d05).
