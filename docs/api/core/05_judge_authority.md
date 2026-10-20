# `RsJudgeAuthority`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `JudgeAuthority` ([F02](../../details/01_flows_and_checkpoints.md#f02))

<a id="u43"></a>

## U43 RsJudgeAuthority

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsJudgeAuthority {
    Script,
    Agent,
    Human,
}
```

Who may judge a gate — recorded on every assessment so a later reader knows what
kind of judgment they are re-examining.

| Variant | Meaning | Required pairing |
| --- | --- | --- |
| `Script` | A fixed command judged | Syntactic gates only; evidence is the command's report |
| `Agent` | An agent judged | Gate's `agent_may_judge == true`; rationale required |
| `Human` | A human judged | Gate's `human_may_judge == true`; rationale required |

**See also:** [`RsJudge`](31_judge.md#u23), [`RsAssessmentEntry`](22_assessment_entry.md#u32).
