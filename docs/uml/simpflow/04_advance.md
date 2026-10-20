# `RsSimpleFlowRun::advance`

> run · method · implemented SimpleFlow slice · realizes `AdvanceSimpleFlowRun` ([D04](../../details/simple_flow/01_run_design.md#d04))

<a id="u19"></a>

## U19 advance

```rust
pub fn advance(&mut self) -> RsAdvanceOutcome
```

Decide where the run goes after gate evaluation. When the current visit is
authorized — the latest assessment of every exit gate passed at the visited revision — a
nonterminal stage hands the run to the next stage's pending visit; the terminal
(verify) stage completes the run. A visit that is not authorized remains blocked
and current so corrective activity can be recorded on that visit. Reopening a
previously completed terminal stage is a separate operation
([`reopen_stage`](33_reopen_stage.md#u49)).

**Parameters** — none beyond `&mut self`.

**Returns**

| Variant | Meaning | Post-state |
| --- | --- | --- |
| `RsAdvanceOutcome::EnterNextStage` | The next stage's pending visit became current | `current_visit` moved; that visit `Active`, previous `Completed`; run stays `Active` |
| `RsAdvanceOutcome::CompleteRun` | The terminal visit was authorized | terminal visit `Completed`; `status == Completed` |
| `RsAdvanceOutcome::RemainBlocked` | The current visit is not authorized | current visit and run are `Blocked`; no new visit is created |

**Errors** — none; the decision is total over the run's states.
Calling it again after completion returns `CompleteRun` while the plan, current
and completed artifact snapshots, latest gate assessments, and retained passed-assessment
evidence remain valid. A failed plan validation, changed artifact bytes, absent
evidence on a current or completed visit, unresolved record-write failure, or
missing latest pass returns `RemainBlocked` and marks the run and current visit
`Blocked`.

**Logic**

```
1. if self.status == Completed: recheck plan, current and completed artifacts, latest
   assessments, and all passed evidence on current/completed visits;
   return CompleteRun only while they remain valid
2. require plan.validate(), current and completed artifact snapshots, all passed evidence
   on current/completed visits, and a durable current record;
   otherwise retain the current visit as Blocked
3. if visit.completion_authorized(&self.plan):
4.     if self.plan.stage(visit.stage_id()).completes_flow:
4.         current_mut().status := Completed
5.         self.status         := Completed
6.         persist the new state; return CompleteRun only if it was written
7.     next := plan.next_stage(visit.stage_id())
8.     current_mut().status := Completed
9.     self.current_visit  := index_of_pending(next)
10.    current_mut().status := Active        // artifact_revision unchanged
11.    persist the new state; return EnterNextStage only if it was written
12. else:
13.    current_mut().status := Blocked       // same visit retained ([D48])
14.    self.status := Blocked
15.    persist blocked state when possible; return RemainBlocked
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Rehashes current and completed artifacts, checks retained passed evidence, mutates visit statuses and `current_visit`, writes the JSON record; a failed transition write rolls back the transition and marks the current visit blocked |
| Complexity | O(gates + visits) via `completion_authorized` |
| Thread safety | `&mut self` — caller serializes |

**Example** (illustrative)

```rust
match run.advance() {
    RsAdvanceOutcome::EnterNextStage => println!("entered {}", run.current().stage_id()),
    RsAdvanceOutcome::CompleteRun => println!("run completed"),
    RsAdvanceOutcome::RemainBlocked => println!("blocked at visit {}", run.current().visit_index()),
}
```

**See also:** [`RsStageVisit::completion_authorized`](15_completion_authorized.md#u30), [`evaluate_gates`](03_evaluate_gates.md#u18), [`RsAdvanceOutcome`](29_advance_outcome.md#u44), [run states D48](../../details/simple_flow/06_rust_uml_design.md#d48).
