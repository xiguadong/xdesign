# `RsJudge`

> ports · trait · designed-only · realizes `Judge` ([D10](../../details/simple_flow/03_structure_graphs.md#d10))

<a id="u23"></a>

## U23 RsJudge

```rust
pub trait RsJudge: Send {
    fn review(&self, criteria: &[String], evidence_uris: &[String])
        -> (RsCheckpointOutcome, String);

    fn authority(&self) -> RsJudgeAuthority;
    fn identity(&self) -> &str;
}
```

The boundary a semantic gate is judged through. An agent or a human stands behind
it; the run knows only the trait. What a script cannot settle crosses this interface
and comes back as an outcome with a rationale.

**Required methods**

| Method | Kind | Page |
| --- | --- | --- |
| [`review`](09_review.md#u24) | required | full entry |
| `authority() -> RsJudgeAuthority` | required getter | trivial: `Agent` or `Human`, matching the gate's flags |
| `identity() -> &str` | required getter | trivial: recorded as the assessment's `assessor` |

**Implementor contract**

1. **Permitted authority:** `authority()` returns `Agent` or `Human`; the gate
   evaluator checks that identity against its `agent_may_judge` / `human_may_judge`
   flags before presenting criteria.
2. **Reasoned verdicts:** a `Passed` or `Failed` outcome always comes with a
   non-empty rationale a later reader can re-examine.
3. **Refusal is honest:** when the evidence is insufficient to judge, return
   `Unevaluated` rather than guessing — the run records it as
   [`RsGateError::JudgmentRefused`](31_gate_error.md#u46).

**See also:** [`RsJudge::review`](09_review.md#u24), [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20), [judgment exchange D10](../../details/simple_flow/03_structure_graphs.md#d10).
