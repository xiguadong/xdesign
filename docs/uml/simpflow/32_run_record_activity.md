# `RsSimpleFlowRun::record_activity`

> run · method · designed-only · realizes the run's corrective activity path
> ([F04](../../details/01_flows_and_checkpoints.md#f04))

<a id="u48"></a>

## U48 record_activity

```rust
pub fn record_activity(&mut self, activity: RsActivityEntry)
```

Record work on the current visit. A blocked visit remains the current visit;
recording corrective activity resumes that same visit. The visit's
[`record_activity`](13_record_activity.md#u28) increments its revision when
`activity.artifacts_changed` is true, including an edit at an unchanged URI.
The caller supplies the content digests of changed artifacts for the new
snapshot. The run does not create a new visit on gate failure.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `activity` | `RsActivityEntry` | identifies the work and any changed artifact bytes | Work to retain in the current visit |

**Returns** — `()`; activity and any revision change are retained in the visit.

**Errors** — none for an active or blocked run. The controller does not call
this on a completed run; it first calls [`reopen_stage`](33_reopen_stage.md#u49).

**Logic**

```
1. current_mut().record_activity(activity)
2. if self.status == Blocked:
       current_mut().status := Active
       self.status := Active
3. return
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Appends activity, may update the artifact snapshot and revision, may resume the run and visit; no I/O |
| Complexity | O(artifacts in the current snapshot) when digests change |
| Thread safety | `&mut self`; caller serializes |

**See also:** [`RsStageVisit::record_activity`](13_record_activity.md#u28), [`evaluate_gates`](03_evaluate_gates.md#u18), [`reopen_stage`](33_reopen_stage.md#u49).
