# Stage CLI

<a id="d85"></a>

## 1. Local implementation contract

`LocalStageCliRuntime` realizes the [common stage interaction](lifecycle.md#f04) for
the two existing flat plans: `simple-flow` and `wish-verify`. The implementation is
developed through [wish-0010's SimpleFlow plan](../records/plans/stage-cli.md).
That record owns execution results; this page specifies behavior.

### 1.1 Commands

Run from the project root. New registration defaults to the legacy wish source at
`mbse/wishes/WISH_ID.sysml`; `--wish FILE` imports a different existing SysML source:

```text
xdesign init WISH_ID --flow simple-flow [--wish FILE]
xdesign next WISH_ID
xdesign check WISH_ID [--result FILE] [--operation-id ID]
xdesign store init
xdesign store list [--kind KIND] [--status STATUS]
xdesign store get WISH_ID [--revision N]
xdesign store history WISH_ID
xdesign store checkout WISH_ID [--revision N] --to DIR
xdesign store outcome OPERATION_ID
xdesign store doctor [--verify]
```

`init` is the one-time boundary before the repeated `next` / `check` loop. New runs create
a store item and initial revision exclusively, freeze the chosen built-in plan and work,
create initial visits, and publish registration last. They never overwrite a run. The
source identifies the wish input; initialization does not execute its SysML or reconcile a
second Git trace. `--flow wish-verify` selects the two-stage alternative. IDs contain only
ASCII letters, digits, hyphens and underscores. Unsupported flows, unknown options,
duplicate options, and extra positional arguments are errors. Pre-existing cache-backed
registrations remain readable on the legacy path and are not implicitly migrated.

`next` validates the stored registration, frozen files and run history. It does not assess,
record, initialize, advance or reopen anything, and it does not change the stored head.
Missing regenerable cache copies may be materialized; current workspace edits are otherwise
left alone and reported as candidate digests without recording them. Legacy `next` retains
its stricter no-file-creation behavior.

`check` validates the result envelope, collects outputs, records actual digest changes,
evaluates the current stage's gates, calls the existing advancement logic, and commits a
new store revision for new runs. It returns assessments separately from resulting work.
A blocked visit retains history and can be corrected. Unchanged content does not increment
the revision. A completed `next` or `check` returns the stored completion without a new
assessment. Root completion is not final human acceptance.

Stage commands retain their existing JSON responses and exit 0/1/2 meanings. Store commands
use the versioned envelope and detailed exit codes in [store format](../interfaces/store-format.md#i11).
Diagnostics go to stderr for command errors. `--help` describes public commands; existing
demo commands remain available for earlier examples.

### 1.2 Work and result exchange

Responses carry `schema_version`, `wish_id`, `run_id`, `flow_id`, `status`, `completed`,
`state_token`, and `work` (null when completed). Store-backed responses also carry
`store_revision`; a check response names its `operation_id`. Work names `stage_id`, `visit_index`,
`artifact_revision`, `artifact_snapshot_digest`, goal, instructions, inputs, expected
outputs, exit checkpoint definitions, current assessments and correction reasons.
`candidate_artifacts` and `candidate_snapshot_digest` describe currently collectible
workspace bytes; diagnostics identify missing declared outputs.

A check response also carries `assessed` with the assessed stage, visit index, revision,
snapshot digest and newly appended assessments, plus `advance`. Successor work is never
misrepresented as the visit whose assessments just ran.

The optional result file is strict JSON:

```json
{
  "wish_id": "wish-0010",
  "run_id": "<from work response>",
  "stage_id": "verify",
  "visit_index": 0,
  "base_revision": 0,
  "state_token": "<from work response>",
  "description": "Run the acceptance cases and retain their evidence.",
  "performed_by": "agent",
  "artifacts": [{"uri": "cache/evidence/wish-0010/verification.json", "sha256": "<file SHA-256>"}],
  "judgment": {
    "authority": "agent",
    "identity": "reviewing-agent",
    "outcome": "passed",
    "rationale": "The retained results meet the stage criteria.",
    "artifact_snapshot_digest": "<digest of the complete collected artifact set>"
  }
}
```

The judgment is optional and applies only to a permitted semantic gate. Omitting it
records an unevaluated semantic assessment, never an inferred pass. A caller can supply
`failed` or `unevaluated` as well as `passed`. Fixed gates always use their frozen checker
declarations; the result cannot supply a syntactic verdict. Every supplied file digest
must match the file. A semantic judgment's snapshot digest must match the entire collected
set, including automatically located outputs. The canonical digest is SHA-256 over sorted
`URI`, NUL, lowercase file SHA-256, LF entries, as defined by `snapshot_digest`.

The state token hashes the registration, plan, work and current stored run bytes. Together
with wish, run, stage, visit and base revision it prevents a result prepared for another
state from changing this run. The artifact snapshot digest separately prevents a judgment
prepared for older workspace bytes from approving edits. A caller may query `next` again
after doing the work to obtain current candidate digests. `--operation-id` names the complete
stage save; on uncertain outcome, repeat it through `store outcome` or retry with the exact
same result and state rather than creating a fresh operation.

### 1.3 Work definitions and collection

The frozen work companion supplies every stage's nonempty goal, instructions, input
declarations and output selectors. The built-in strings realize the stage attributes in
`DesignedSimpleFlow` and `DesignedWishVerifyFlow`. Registration binds the plan and work
hashes; later calls reject drift rather than regenerating a changed definition.

| Stage | Declared output collection |
| --- | --- |
| SimpleFlow requirements | Freeze the paths reported by `simple_requirements.py --list-inputs` |
| SimpleFlow design | Freeze the paths reported by `simple_design.py --list-inputs` |
| SimpleFlow implement | `Cargo.toml` and recursively discovered `src/**/*.rs` |
| WishVerifyFlow wish | The working wish source, with result-supplied answering artifacts |
| Either verify stage | Result-supplied verification evidence; an empty artifact set cannot pass |

Result artifacts supplement automatically collected outputs. Prior supplemental URIs remain
part of the current visit and are rehashed on later calls; no removal command is exposed
in this slice. Missing supplemental files must be restored, and remain blockers on
repeated checks. New evidence can first be recorded without a judgment; the returned
candidate digest then binds a later explicit judgment. Missing expected files
remain diagnostics and block authorization. The runtime must not collect its own mutable
registry, plan, work, lock, run record or checker reports as stage outputs; checker exchange
files already have their separate integrity bindings. The CLI retains an immutable
JSON record copy at `reports/<stage>_<revision>_<uuid>.record.json` before each
fixed checker call and passes that copy as the checker
input, so the report's input URI and hash stay re-openable after the live run advances. Artifact paths stay inside the
project, including after symlink resolution. A result file cannot choose commands to run.

### 1.4 Storage, recovery and limits

The storage redesign gives new registrations a durable backend through
[`XDesignStorage`](storage.md#d86); its delivery is tracked by the
[storage plan](../records/plans/storage.md#p35). Runs initialized before that boundary stay
on the legacy cache backend and are not implicitly migrated. The legacy backend retains
the atomic temporary-file/rename writer and cache retention limits.

For a new run, `init` accepts an existing SysML source at the default
`mbse/wishes/WISH_ID.sysml` or an explicit `--wish FILE`, imports its bytes into the store,
and materializes `cache/wishes/WISH_ID.sysml` as the working source. The original import is
not reread as an implicit authority change. Later edits to the working source are collected
and saved as new revisions. There is no second Git trace for a store-backed run.

Readers validate an immutable stored head rather than a mixture of live files being replaced
by a writer. Missing runtime plans, work, run records, checker exchanges, and wish working
copies are regenerated from stored materials. Read-only inspection does not overwrite edited
workspace artifacts or claim that their current bytes still match an earlier assessment.

`check` holds the per-wish writer lock for restore/record/assess/advance/save. Engine
output is speculative until the store commits: the engine's working run record and the
stage response are staged under `cache/xdesign/WISH_ID/attempts/` while the canonical run
record stays at the committed head, and publication replaces the canonical files only
after the store commits. After the engine produces a result, one expected-head store
operation commits the registration, frozen plan and work, run record, wish source,
immutable checker inputs and reports, current artifacts, evidence, and the returned
response. A failed or unevaluated gate commits its factual blocked record. Invalid
results and stale-head conflicts do not advance the stored head. Callers can supply an
operation ID and reconcile a lost response through `store outcome`. Operation lookup
precedes stale-result validation, so a completed head cannot hide the original assessed
response. Replaying the exact committed operation does not rerun checkers; a pending
operation retains its final request so the same stage command can resume it without
another invocation — including when its transient sources are gone, because published
blobs are reused first. A retry of an operation with no receipt discards that operation's
stale speculative files and repairs the on-disk run record from the committed head, so a
crash mid-attempt or a database timeout before reservation never wedges the wish on
immutable-record drift; the same projection repair applies to readers. The writer lock is
removed on normal exit. After a crash, inspect the store and active process before
manually removing a stale lock; the CLI does not guess that a writer has abandoned it.

Checker subprocesses run under a hard deadline (default 600 seconds, overridden by
`XDESIGN_CHECKER_TIMEOUT_SECS`): a checker that exceeds it is killed, its partial logs are
retained with a timeout note, and the gate reports an explicit checker timeout instead of
blocking the CLI indefinitely.

`store list --kind wish` derives its projection from the stored run: `pending` maps to
`received`, `active` to `active`, `blocked` to `blocked`, and `completed` to `delivered`.
For a non-completed run, the current stage is the visit at `current_visit`; a completed run
has no current stage. `--status` filters this projection. There is no separately editable
wish-status field.

The [restore API](api/core/restore-run.md#u78) still checks serialized identity, visit shape,
replayed revision snapshots, assessment ownership and authority, and completed-stage
authorization. It does not authenticate the record's author. Live artifact changes are
checked during collection and advancement, and earlier completed snapshots must match
before further advancement. Stored revisions additionally retain the corresponding bytes,
so cache deletion does not make the operational record unreadable.

The scope remains local, repository-bound flat flows with guarded writers and concurrent
readers. It excludes arbitrary or nested flows, whole-store backup/migration, remote judge
authentication, and final human acceptance. Technical completion is not human acceptance.

### 1.5 Software realization and verification

| Source | Responsibility |
| --- | --- |
| `src/main.rs` | Routes the top-level commands and retains the demo entry points |
| `src/stage_cli/mod.rs` | Validates arguments, result exchange, and stage orchestration |
| `src/stage_cli/work.rs` | Freezes modeled stage work and collects artifact snapshots |
| `src/stage_cli/store.rs` | Resolves wishes, verifies frozen files and serializes writers |
| `src/stage_cli/persistent.rs` | Adapts stored heads, materialization and atomic stage revisions |
| `src/storage/` | Implements store records, SQLite, blobs, operations, queries and commands |
| `src/core/restore.rs` | Reconstructs a validated run with its existing adapters |
| `tests/stage_cli.rs`, `tests/run_restore.rs` | Process-level interaction and corruption/recovery cases |
| `tests/storage.rs`, `tests/storage_cli.rs` | Store protocol, fault, query, checkout and cache-loss cases |

`RsStageCliApplication` realizes `LocalStageCliRuntime`; `RsRestoreRun` realizes its
restore operation. `RsStore` and the storage API realize `XDesignStorage`; their contracts
are in [storage API design](api/storage/index.md). The binary uses the existing flow engine
and process adapters without moving SQLite into `core`.

`LocalStageCliRuntimeVerification` covers both flows across process restarts, read-only
queries and completed calls, blocked correction, unchanged revisions, stale envelopes,
changed judgment artifacts, corrupt state, plan/work drift, concurrent writers and unsafe
paths. `StorageVerification` adds exact retry, crash, durability and checkout/replay cases
in [verification design](../verification/design.md#d87). Figures are design views, not
screenshots. Executed results belong to the [storage plan](../records/plans/storage.md#p39).
