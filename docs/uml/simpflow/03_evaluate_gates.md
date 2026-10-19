# `RsSimpleFlowRun::evaluate_gates`

> run · method · designed-only · realizes `EvaluateSimpleFlowGate` ([D04](../../details/simple_flow/01_run_design.md#d04))

<a id="u18"></a>

## U18 evaluate_gates

```rust
pub fn evaluate_gates(&mut self) -> Vec<RsAssessmentEntry>
```

Evaluate every exit gate of the current visit's stage, each by its own `kind`, and
append the produced assessments to the current visit. A gate with no assessment
authorizes nothing; earlier assessments and their evidence stay untouched.

**Parameters** — none beyond `&mut self`. The gates are read from the plan
(`self.plan.checkpoints_of(self.current().stage_id())`), never from the visit.
The plan's gate specs and the current visit snapshot are cloned before mutating
the visit, so the evaluation loop does not hold a borrow of the run's plan while
appending assessments.

**Returns**

| Type | Guarantee |
| --- | --- |
| `Vec<RsAssessmentEntry>` | One entry per exit gate of the current stage, in plan order; every entry is also appended to the current visit before the next gate is evaluated |

**Errors** — no `Result`. A gate that cannot conclude records an assessment with
`outcome == Unevaluated` and `judged_by` naming the failed channel
([`RsGateError`](31_gate_error.md#u46) is carried in its `rationale`); evaluation of
the remaining gates continues. A blocked run is a state, not an exception.

**Logic**

```
1. gates := plan.checkpoints_of(current().stage_id()).cloned().collect()
   visit_snapshot := current().clone()
2. out := []
3. for spec in gates:
4.     result := evaluate_checkpoint(&spec, &visit_snapshot,
           &plan_id, &plan_uri, &plan_sha256,
           &wish_model_uri, &run_record_uri, &*check, &*judgment)
5.     entry := match result {
           Ok(a)  => a,
           Err(e) => AssessmentEntry::unevaluated(spec.checkpoint_id,
                          current().artifact_revision(),
                          current().artifact_snapshot_digest().to_owned(), e)
       }
6.     current_mut().append_assessment(entry.clone())
7.     out.push(entry)
8. if not current().completion_authorized(&plan):
       current_mut().status := Blocked
       self.status := Blocked            // gate not authorized ([D48])
   else:
       current_mut().status := Active
       self.status := Active
9. return out
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Mutates the current visit (appends assessments) and possibly `self.status`; the syntactic path invokes an external command, the semantic path an external judge |
| Complexity | O(gates × rules) for the scripted path; the judged path is out of band |
| Thread safety | `&mut self` — caller serializes; no interior mutability |

**Example** (designed-only)

```rust
let entries = run.evaluate_gates();
assert!(!entries.is_empty());
```

**See also:** [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20), [`advance`](04_advance.md#u19), [`RsStageVisit::append_assessment`](14_append_assessment.md#u29), [gate evaluation D04](../../details/simple_flow/01_run_design.md#d04).
