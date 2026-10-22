# `RsJudge`

> `xdesign::core::ports` · trait · implemented xdesign slice · realizes `Judge` ([D10](../../flows/simple-flow/views.md#d10))

<a id="u23"></a>

## RsJudge

```rust
pub trait RsJudge: Send {
    fn review(&self, criteria: &[String], evidence_uris: &[String])
        -> (RsCheckpointOutcome, String);

    fn authority(&self) -> RsJudgeAuthority;
    fn identity(&self) -> &str;
}
```

RsJudge is the trait boundary for semantic gate judgment. Implement or supply it when a gate requires an agent or human judgment rather than a fixed checker; fixed-checker logic must not be hidden behind it.

**Required methods**

| Method | Kind | Page |
| --- | --- | --- |
| [`review`](rs-judge-review.md#u24) | required | full entry |
| `authority() -> RsJudgeAuthority` | required getter | trivial: `Agent` or `Human`, matching the gate's flags |
| `identity() -> &str` | required getter | trivial: recorded as the assessment's `assessor` |

**Implementor contract**

1. **Permitted authority:** `authority()` returns `Agent` or `Human`; the gate
   evaluator checks that identity against its `agent_may_judge` / `human_may_judge`
   flags before presenting criteria.
2. **Reasoned verdicts:** a `Passed` or `Failed` outcome always comes with a
   non-empty rationale and judge identity a later reader can re-examine. A
   semantic pass cites existing evidence files and at least one current
   verify-stage artifact; the record-check report alone does not meet this rule.
3. **Refusal is honest:** when the evidence is insufficient to judge, return
   `Unevaluated` rather than guessing — the run records it as
   [`RsGateError::JudgmentRefused`](rs-gate-error.md#u46).

**See also:** [`RsJudge::review`](rs-judge-review.md#u24), [`evaluate_checkpoint`](evaluate-checkpoint.md#u20), [judgment exchange D10](../../flows/simple-flow/views.md#d10).
