# `RsFlowRun::record_activity`

> `xdesign::core::run` · method · implemented xdesign slice · realizes the run's corrective activity path ([F04](../../details/01_flows_and_checkpoints.md#f04))

<a id="u48"></a>

## U48 record_activity

```rust
pub fn record_activity(&mut self, activity: RsActivityEntry) -> Result<(), String>
```

Record work on the current visit. A blocked visit remains the current visit;
recording corrective activity resumes that same visit. The visit's
[`record_activity`](18_record_activity.md#u28) increments its revision when
`activity.artifacts_changed` is true, including an edit at an unchanged URI.
The caller supplies the content digests of changed artifacts for the new
snapshot. The run does not create a new visit on gate failure.
For a changed activity, `artifact_uris` must name exactly the files in
`artifact_digests` and `removed_artifact_uris`. For an unchanged activity, any
named file must exist when the activity is recorded.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `activity` | `RsActivityEntry` | identifies the work and any changed artifact bytes | Work to retain in the current visit |

**Returns** — `Ok(())` after the updated JSON record is written.

**Errors** — `Err(String)` when the run is completed, required activity metadata
is empty, a change declaration conflicts with its digest or removal list, the
named artifact URIs differ from the changed and removed URI set, a
digest does not match actual file bytes, a removal names an absent prior artifact,
an artifact path is invalid, an unchanged activity names a missing file,
`artifacts_changed` is true when `artifact_revision == u32::MAX`, or JSON
record persistence fails. The controller
must call [`reopen_stage`](44_reopen_stage.md#u49) before work on a completed run.
An unsuccessful write is retained in `last_error()` and blocks advancement.
The activity may already have changed the in-memory visit when a later JSON
write fails; the caller must resolve persistence before treating that event as
retained evidence.

**Logic**

```
1. reject completed run or invalid activity metadata, paths, content digests,
   change declaration, URI set, or removals
2. current_mut().record_activity(activity)
3. if self.status == Blocked:
       current_mut().status := Active
       self.status := Active
4. persist the updated JSON record; return Ok only when written
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Reads changed artifact bytes for digest validation; appends activity, may update the artifact snapshot and revision, may resume the run and visit, and writes the JSON record |
| Complexity | O(artifacts in the current snapshot) when digests change |
| Thread safety | `&mut self`; caller serializes |

**See also:** [`RsStageVisit::record_activity`](18_record_activity.md#u28), [`evaluate_gates`](41_evaluate_gates.md#u18), [`reopen_stage`](44_reopen_stage.md#u49).
