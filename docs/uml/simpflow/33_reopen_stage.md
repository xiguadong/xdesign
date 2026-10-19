# `RsSimpleFlowRun::reopen_stage`

> run · method · designed-only · realizes `ReopenSimpleFlowStage`
> ([D04](../../details/simple_flow/01_run_design.md#d04))

<a id="u49"></a>

## U49 reopen_stage

```rust
pub fn reopen_stage(&mut self, stage_id: &str) -> bool
```

Reopen the completed terminal stage for new work. This first implementation
supports the `verify` stage of a completed run. It appends a visit to that stage
at its next per-stage `visit_index`, with `artifact_revision == 0`, no activities,
and no assessments. The earlier completed visit and its evidence stay intact;
the run becomes active, so delivery authorization must be established again.
Reopening a nonterminal stage would require downstream invalidation and is
deferred explicitly.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `stage_id` | `&str` | must be the completed terminal stage ID | Stage to reopen |

**Returns**

| Type | Guarantee |
| --- | --- |
| `bool` | `true` iff a completed terminal stage was reopened; `false` leaves the run unchanged |

**Errors** — no exception. A nonterminal stage ID, unknown stage, or a run that
is not completed returns `false` without mutation.

**Logic**

```
1. if self.status != Completed or stage_id != "verify": return false
2. previous := latest visit with previous.stage_id() == stage_id
3. append RsStageVisit::pending(stage_id, previous.visit_index() + 1)
4. new visit.status := Active
   self.current_visit := index of new visit
   self.status := Active
5. return true
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Appends one visit and changes run status; no I/O |
| Complexity | O(visits) to find the latest visit |
| Thread safety | `&mut self`; caller serializes |

**See also:** [`advance`](04_advance.md#u19), [`RsStageVisit`](12_stage_visit.md#u27), [run states D48](../../details/simple_flow/06_rust_uml_design.md#d48).
