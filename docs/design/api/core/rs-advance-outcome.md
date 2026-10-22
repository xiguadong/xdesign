# `RsAdvanceOutcome`

> `xdesign::core::status` · enum · implemented xdesign slice · makes the three guarded paths of `AdvanceSimpleFlowRun` explicit ([D04](../../flows/simple-flow/implementation.md#d04))

<a id="u44"></a>

## RsAdvanceOutcome

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsAdvanceOutcome {
    EnterNextStage,
    CompleteRun,
    RemainBlocked,
}
```

What [`advance`](rs-flow-run-advance.md#u19) decided for one visit. A Rust-side enum with no
SysML counterpart: the model leaves the decision implicit in guarded paths;
the crate names it so callers handle every case.

| Variant | Meaning | Post-state |
| --- | --- | --- |
| `EnterNextStage` | Nonterminal visit authorized | Next stage's pending visit is current |
| `CompleteRun` | Terminal visit authorized | `status == RsFlowStatus::Completed` |
| `RemainBlocked` | Visit not authorized | The same visit remains current and blocked |

**See also:** [`RsFlowRun::advance`](rs-flow-run-advance.md#u19), [`RsFlowStatus`](rs-flow-status.md#u39).
