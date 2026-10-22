# `RsCheckKind`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `CheckKind` ([F02](../../adapters.md#f02))

<a id="u41"></a>

## RsCheckKind

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsCheckKind {
    Syntactic,
    Semantic,
}
```

How a gate's criteria are established — the single bit that selects the evaluation
path in [`evaluate_checkpoint`](evaluate-checkpoint.md#u20).

| Variant | Meaning | Consequence |
| --- | --- | --- |
| `Syntactic` | A fixed command decides | `checker` must be `Some`; the verdict is `judged_by == Script` |
| `Semantic` | A permitted judge decides | `checker` must be `None`; at least one judge flag must be `true` |

**See also:** [`RsCheckpointSpec`](rs-checkpoint-spec.md#u34), [`RsFlowPlan::validate`](rs-flow-plan-validate.md#u26).
