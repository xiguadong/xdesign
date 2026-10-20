# `RsStageVisit::record_activity`

> record · method · implemented SimpleFlow slice · realizes the activity-recording half of `StageWork` and `Controller::recordActivity` ([F04](../../details/01_flows_and_checkpoints.md#f04))

<a id="u28"></a>

## U28 record_activity

```rust
pub(crate) fn record_activity(&mut self, activity: RsActivityEntry) -> bool
```

Append one piece of work to this visit. The activity's explicit
`artifacts_changed` field records whether artifact content changed, including
in-place edits at an existing URI. A true value increments `artifact_revision`,
so a prior pass cannot authorize the new content. The public run wrapper
([U48](32_run_record_activity.md#u48)) resumes a blocked visit after this method.
Changed files carry their exact SHA-256 values; removals name the deleted URIs.
The visit recomputes its canonical artifact snapshot digest from the resulting set.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `activity` | `RsActivityEntry` | `occurred_at` non-empty; `artifacts_changed` accurately identifies content changes | The work to record |

**Returns** — `bool`, equal to `activity.artifacts_changed`; the visit carries
the recorded effect.

**Errors** — none; recording never fails. (Persisting the record is the controller's
concern, not this type's.)

**Logic**

```
1. artifacts_changed := activity.artifacts_changed
2. if artifacts_changed:
       remove activity.removed_artifact_uris from self.artifact_digests
       replace/add every activity.artifact_digests by URI
       self.artifact_snapshot_digest := sha256(canonical artifact digests)
       self.artifact_revision += 1
3. self.activities.push(activity)
4. // assessments already recorded stay untouched;
   // they cite their own evaluated_revision
5. return artifacts_changed
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | Mutates the visit: appends an activity, may bump `artifact_revision` |
| Complexity | O(artifact count × log artifact count) for canonical sorting on a change |
| Thread safety | `&mut self` — caller serializes |

**Example** (illustrative)

```rust
let rev = visit.artifact_revision();
visit.record_activity(activity);
if visit.artifact_revision() > rev {
    // earlier passed assessments no longer authorize this visit
}
```

**See also:** [`append_assessment`](14_append_assessment.md#u29), [`completion_authorized`](15_completion_authorized.md#u30), [`RsActivityEntry`](16_activity_entry.md#u31).
