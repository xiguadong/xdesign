# `RsCheckKind`

> status · enum · implemented SimpleFlow slice · mirrors `CheckKind` ([F02](../../details/01_flows_and_checkpoints.md#f02))

<a id="u41"></a>

## U41 RsCheckKind

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsCheckKind {
    Syntactic,
    Semantic,
}
```

How a gate's criteria are established — the single bit that selects the evaluation
path in [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20).

| Variant | Meaning | Consequence |
| --- | --- | --- |
| `Syntactic` | A fixed command decides | `checker` must be `Some`; the verdict is `judged_by == Script` |
| `Semantic` | A permitted judge decides | `checker` must be `None`; at least one judge flag must be `true` |

**See also:** [`RsCheckpointSpec`](19_checkpoint_spec.md#u34), [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26).
