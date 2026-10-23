# `restore_run`

> `xdesign::core::restore` · function · local run restoration · model `XDesignRustUml::RsRestoreRun` realizes the read-only restoration needed by `XDesignWorkflow::LocalStageCliRuntime`

<a id="u78"></a>

## restore_run

```rust
pub fn restore_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsFlowRun, String>
```

Restore the local JSON snapshot written by
[`RsFlowRun::write_record`](rs-flow-run-write-record.md#u54), validate its internal
history, and attach the caller's checker and judge boundaries. Restoration reads
the frozen plan, existing wish path, and run record; it does not write a file,
evaluate a gate, initialize a visit, advance, or reopen the run. This makes it
suitable for the read-only [`next` interaction](../../lifecycle.md#f04).

The flow layer or application must validate its selected blueprint before calling
this generic function. Restoration does not infer a different flow from a record
or replace the supplied frozen plan.

**Parameters**

| Name | Constraint |
| --- | --- |
| `plan` | Passes structural validation, including its exact frozen file digest; its identity must match the record |
| `wish_model_uri` | Existing repository-relative `.sysml` file; exactly matches the recorded wish binding |
| `run_record_uri` | Existing repository-relative local JSON snapshot |
| `check` | Caller-selected fixed-check adapter for the validated plan; never invoked while restoring |
| `judgment` | Caller-selected semantic judgment boundary; never invoked while restoring |

**Validation**

1. Validate the supplied plan and paths, then decode the current writer's exact
   schema. Unknown fields, missing fields, duplicate fields, invalid enum values,
   and invalid field types are errors. The local reader has no automatic migration
   from a different record schema.
2. Require a canonical nonzero UUID run identity, `execution_type = "FlowRun"`,
   matching plan ID, path and SHA-256, and the same wish-model URI.
3. Require the original visit for every plan stage in plan order. Additional
   visits may only be contiguous explicit reopens of the reached terminal stage.
   The current visit, run status, completed predecessors, and untouched pending
   visits must describe a state reachable through the run engine's first-successor
   progression and terminal-stage reopening.
4. Replay each visit's activity digest updates and removals in memory. Validate
   activity metadata, relative paths, digest syntax, unique URI sets, valid
   removals, and the change declaration. Reconstructed revisions, artifact sets,
   and snapshot digests must equal the stored values.
5. Bind every assessment to its owning stage's gate and the reconstructed
   historical revision and snapshot. Require evaluation order, assessor identity,
   evidence shape, and the gate's checker or permitted judge authority. A refused
   or unavailable evaluation remains a nonauthorizing `unevaluated` entry with
   assessor `unavailable` and no evidence, including a refusal caused by an
   unpermitted judge.
6. Require every completed visit to have the latest passing assessment for each
   exit gate at that visit's current revision and snapshot. A completed run must
   end at its authorized terminal visit.
7. Construct the owned run with the retained IDs, visits, current index, and
   status, attach the supplied boundaries, and clear transient persistence-error
   state. No new activity or assessment is appended.

**Returns** — `Ok(run)` preserves the recorded execution state. `Err(message)`
identifies an invalid file, identity, history, assessment, or lifecycle condition;
the function has not mutated the record or workspace.

**Artifact drift** — restoration validates recorded digests against the activity
history, without reading current artifact or evidence bytes. The agent may have
edited the current stage's outputs before calling `check`; those edits must remain
inspectable through `next`. Changed or missing files from completed visits likewise
remain visible. Existing gate evaluation and
[`advance`](rs-flow-run-advance.md#u19) retain their live-artifact and evidence
checks and prevent such drift from authorizing progression.

**Limits** — this is consistency validation of local process state. It does not
authenticate a file author, establish that a supplied semantic judgment is true,
or provide durable multi-user persistence. Concurrency exclusion belongs to the
calling application. Validation uses memory proportional to the retained record
and reconstructed revision history.

**Verification** — `tests/run_restore.rs` covers read-only restoration and
continuation, terminal reopening, live drift, corrupted identities and schemas,
inconsistent activities and revisions, foreign or unauthorized assessments, and
forged completion. Executed outcomes belong to the implementing wish's records.

**See also:** [`create_run`](create-run.md#u16),
[`RsFlowRun`](rs-flow-run.md#u17),
[`RsStageVisit::completion_authorized`](rs-stage-visit-completion-authorized.md#u30),
[evidence and revisions](../../evidence-and-revisions.md#e03).
