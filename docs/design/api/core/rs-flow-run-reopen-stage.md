# `RsFlowRun::reopen_stage`

> `xdesign::core::run` · method · implemented xdesign slice · realizes `ReopenSimpleFlowStage` ([D04](../../flows/simple-flow/implementation.md#d04))

<a id="u49"></a>

## reopen_stage

```rust
pub fn reopen_stage(&mut self, stage_id: &str) -> bool
```

Reopen a completed visit of a flow-completing stage for new work. The gate is
generic: the run must be `Completed` and the named stage's `completes_flow` flag
must be set — no stage name is hardcoded. It appends a visit to that stage
at its next per-stage `visit_index`, with `artifact_revision == 0`, no activities,
and no assessments. The earlier completed visit and its evidence stay intact;
the run becomes active, so delivery authorization must be established again.
Reopening a stage that does not complete the flow would require downstream
invalidation and is rejected.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `stage_id` | `&str` | must name a completed stage whose `completes_flow` flag is set | Stage to reopen |

**Returns**

| Type | Guarantee |
| --- | --- |
| `bool` | `true` iff a completed flow-completing stage was reopened; `false` leaves the run unchanged |

**Errors** — no exception. A stage without `completes_flow`, an unknown stage, or
a run that is not completed returns `false` without mutation.

**Logic**

```
1. if self.status != Completed: return false
2. spec := plan.stage(stage_id)
   if spec is None or !spec.completes_flow: return false
3. previous := latest visit with previous.stage_id() == stage_id
4. append RsStageVisit::pending(stage_id, previous.visit_index() + 1)
5. new visit.status := Active
   self.current_visit := index of new visit
   self.status := Active
6. persist the appended visit and active status; return true only if written,
   otherwise restore the completed in-memory run and return false
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Appends one visit, changes run status, and writes the JSON record; a write failure rolls back the in-memory reopen |
| Complexity | O(visits) to find the latest visit |
| Thread safety | `&mut self`; caller serializes |

**See also:** [`advance`](rs-flow-run-advance.md#u19), [`RsStageVisit`](rs-stage-visit.md#u27), [run states D48](../../flows/simple-flow/implementation.md#d48).
