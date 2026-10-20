# `RsSimpleFlowRun::evaluate_gates`

> run · method · implemented SimpleFlow slice · realizes `EvaluateSimpleFlowGate` ([D04](../../details/simple_flow/01_run_design.md#d04))

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
The plan's gate specs are cloned before mutating the visit. The current visit is
cloned afresh for each gate, so the verify-stage semantic judge can inspect the
record check's report appended immediately before it. The visit's artifact
revision and digest stay fixed throughout this evaluation call.

**Returns**

| Type | Guarantee |
| --- | --- |
| `Vec<RsAssessmentEntry>` | On an active or blocked run, one returned entry per exit gate of the current stage in plan order; every returned entry is appended to the current visit. A completed run returns an empty vector |

**Errors** — no `Result`. A gate that cannot conclude records an assessment with
`outcome == Unevaluated` and `judged_by` naming the failed channel
([`RsGateError`](31_gate_error.md#u46) is carried in its `rationale`); evaluation of
the remaining gates continues. A completed run returns an empty list. If the plan
is invalid or the current JSON record cannot be persisted, the affected gate is
`Unevaluated`; persistence failure prevents advancement. A blocked run is a state,
not an exception.

**Logic**

```
1. gates := plan.checkpoints_of(current().stage_id()).cloned().collect()
2. out := []
3. for spec in gates:
4.     visit_snapshot := current().clone()
       authority := if spec.kind == Syntactic { Script }
                    else { judgment.authority() }
       require plan.validate() and persist current JSON record
       result := evaluate_checkpoint(&spec, &visit_snapshot,
           &plan_id, &plan_uri, &plan_sha256,
           &wish_model_uri, &run_record_uri, &*check, &*judgment)
           or CommandFailed if validation/persistence failed
5.     entry := match result {
           Ok(a)  => a,
           Err(e) => AssessmentEntry::unevaluated(spec.checkpoint_id,
                          current().artifact_revision(),
                          current().artifact_snapshot_digest().to_owned(),
                          e, authority)
       }
6.     current_mut().append_assessment(entry.clone())
7.     persist updated JSON record; if it fails, append and return a later
       Unevaluated assessment for this gate, so the failed write cannot authorize it
       out.push(the latest entry for this gate)
8. if not current().completion_authorized(&plan):
       current_mut().status := Blocked
       self.status := Blocked            // gate not authorized ([D48])
   else:
       current_mut().status := Active
       self.status := Active
9. persist status change; if it fails, block the run and append Unevaluated
   assessments for each returned gate
10. return out
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Mutates the current visit and status; writes the JSON record before each gate, after assessments and after the status change; the syntactic path invokes an external command, the semantic path an external judge |
| Complexity | O(gates × rules) for the scripted path; the judged path is out of band |
| Thread safety | `&mut self` — caller serializes; no interior mutability |

**Example** (illustrative)

```rust
let entries = run.evaluate_gates();
assert!(!entries.is_empty());
```

**See also:** [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20), [`advance`](04_advance.md#u19), [`RsStageVisit::append_assessment`](14_append_assessment.md#u29), [gate evaluation D04](../../details/simple_flow/01_run_design.md#d04).
