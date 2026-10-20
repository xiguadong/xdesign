# `RsAdvanceOutcome`

> status · enum · implemented SimpleFlow slice · makes the three guarded paths of `AdvanceSimpleFlowRun` explicit ([D04](../../details/simple_flow/01_run_design.md#d04))

<a id="u44"></a>

## U44 RsAdvanceOutcome

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsAdvanceOutcome {
    EnterNextStage,
    CompleteRun,
    RemainBlocked,
}
```

What [`advance`](04_advance.md#u19) decided for one visit. A Rust-side enum with no
SysML counterpart: the model leaves the decision implicit in guarded paths;
the crate names it so callers handle every case.

| Variant | Meaning | Post-state |
| --- | --- | --- |
| `EnterNextStage` | Nonterminal visit authorized | Next stage's pending visit is current |
| `CompleteRun` | Terminal visit authorized | `status == RsFlowStatus::Completed` |
| `RemainBlocked` | Visit not authorized | The same visit remains current and blocked |

**See also:** [`RsSimpleFlowRun::advance`](04_advance.md#u19), [`RsFlowStatus`](24_flow_status.md#u39).
