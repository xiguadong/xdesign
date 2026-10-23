# Stage CLI

<a id="d85"></a>

## 1. Local implementation contract

`LocalStageCliRuntime` realizes the [common stage interaction](lifecycle.md#f04) for
the two existing flat plans: `simple-flow` and `wish-verify`. The implementation is
developed through [wish-0010's SimpleFlow plan](../records/plans/stage-cli.md).
That record owns execution results; this page specifies behavior.

### 1.1 Commands

Run from the project root. Registration uses an existing independent model at
`mbse/wishes/WISH_ID.sysml`:

```text
xdesign init WISH_ID --flow simple-flow
xdesign next WISH_ID
xdesign check WISH_ID [--result FILE]
```

`init` is the one-time boundary before the repeated `next` / `check` loop. It creates
`cache/xdesign/WISH_ID/` exclusively, freezes the chosen built-in plan and work definitions,
creates the initial visits, and publishes the registration. It never overwrites a run. The wish path identifies the run; initialization checks
that the file exists but does not parse or rewrite its SysML trace. Runtime JSON records
are reconciled into the independent SysML history separately.
`--flow wish-verify` selects the two-stage alternative. IDs contain only ASCII letters,
digits, hyphens and underscores. Unsupported flows, unknown options, duplicate options,
and extra positional arguments are errors.

`next` reads the existing registration and validates its frozen files and run history.
It returns the current work without creating a directory, lock, report or record. It
does not assess, record, initialize, advance or reopen anything. Current workspace edits
are allowed: the command reports candidate artifact digests without recording them.

`check` validates the result envelope, collects outputs, records actual digest changes,
evaluates the current stage's gates, and calls the existing engine's advancement logic.
It returns assessments for the visit it checked separately from the resulting current
work. A blocked visit retains history and can be corrected by another `check`. Unchanged
artifact content does not increment the revision. A completed `next` or `check` returns
completion without writes. Root completion is available for human review; it is not a
record of the human's acceptance.

Every successful command emits one JSON object to stdout. Exit 0 means usable work or
completion; exit 1 means a checked visit remains blocked; exit 2 means invalid input,
an unknown wish, corrupt state, a stale result, a busy writer, or another command error.
Diagnostics go to stderr for command errors. `--help` describes the public commands.
The existing demo commands remain available for earlier examples.

### 1.2 Work and result exchange

Responses carry `schema_version`, `wish_id`, `run_id`, `flow_id`, `status`, `completed`,
`state_token`, and `work` (null when completed). Work names `stage_id`, `visit_index`,
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

The state token hashes the registration, plan, work and current run bytes. Together with
wish, run, stage, visit and base revision it prevents a result prepared for another state
from changing this run. The artifact snapshot digest separately prevents a judgment
prepared for older workspace bytes from approving edits. A caller may query `next` again
after doing the work to obtain the current candidate digests before writing the result.

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
| WishVerifyFlow wish | The independent wish model, with result-supplied answering artifacts |
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

The run uses the existing atomic temporary-file/rename record writer. Initialization
reserves the wish directory without replacing previous files. `check` takes an exclusive
per-wish lock for the complete restore/record/assess/advance operation; a second writer
fails instead of overwriting its predecessor. The lock is removed on normal exit. After a
crash, inspect the run and confirm no writer remains before manually removing a stale lock;
the CLI does not guess that a lock is abandoned.

The [restore API](api/core/restore-run.md#u78) checks serialized identity, visit shape,
replayed revision snapshots, assessment ownership and authority, and completed-stage
authorization. It does not treat the local record as cryptographically authenticated.
Live file changes are checked during collection and advancement, so inspection remains
possible after the agent edits the workspace. Earlier completed snapshots must still
match their files before further advancement can succeed.

This is a local single-writer implementation for the existing repository-bound plans.
It does not implement arbitrary user-defined flows, nested executions, a database,
remote judge authentication, migration, or final human acceptance. Cache files survive
ordinary process restarts but remain subject to the project's cache retention policy.

### 1.5 Software realization and verification

| Source | Responsibility |
| --- | --- |
| `src/main.rs` | Routes the top-level commands and retains the demo entry points |
| `src/stage_cli/mod.rs` | Validates arguments, result exchange, and stage orchestration |
| `src/stage_cli/work.rs` | Freezes modeled stage work and collects artifact snapshots |
| `src/stage_cli/store.rs` | Resolves wishes, verifies frozen files and serializes writers |
| `src/core/restore.rs` | Reconstructs a validated run with its existing adapters |
| `tests/stage_cli.rs`, `tests/run_restore.rs` | Process-level interaction and corruption/recovery cases |

`RsStageCliApplication` realizes `LocalStageCliRuntime`; `RsRestoreRun` realizes its
restore operation. The binary depends on the three existing library layers. The
restore function has its own [API contract](api/core/restore-run.md#u78).

`LocalStageCliRuntimeVerification` covers both flows across process restarts, read-only
queries and completed calls, blocked correction, unchanged revisions, stale envelopes,
changed judgment artifacts, corrupt state, plan/work drift, concurrent writers and unsafe
paths. The existing package, timeline and swimlane projections remain applicable to the
common interaction; they are design views, not screenshots of a run. Detailed executed
results belong to the [wish plan](../records/plans/stage-cli.md).
