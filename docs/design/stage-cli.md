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
xdesign init WISH_ID --flow simple-flow|wish-verify [--wish FILE] [--checks FILE]
xdesign next WISH_ID
xdesign check WISH_ID [--result FILE] [--operation-id ID]
xdesign store init
xdesign store list [--kind KIND] [--status STATUS]
xdesign store get WISH_ID [--revision N]
xdesign store history WISH_ID
xdesign store checkout WISH_ID [--revision N] --to DIR
xdesign store outcome OPERATION_ID
xdesign store doctor [--verify]
xdesign checkpoint list
xdesign checkpoint show CHECKPOINT_ID
xdesign checkpoint save --request FILE
xdesign web [--host HOST] [--port PORT] [--open]
            [--project-root PATH] [--store-root PATH]
```

`--checks FILE` selects the check executor for the four SimpleFlow syntactic checkpoints
and the semantic `verificationAccepted` gate for the new run, and is meaningful only with
`--flow simple-flow`; a non-default binding for another flow is rejected, and a
default-only file is a no-op. The file, its validation errors and its frozen-plan effect
are in [§1.5](#d89); library bindings also resolve project library entries. `xdesign
checkpoint` reads and updates the project checkpoint library (see §1.6 and the
[checkpoint library design](checkpoint-library.md#d97)); `xdesign web`
starts the embedded operator page, whose command line, loopback and lifecycle boundaries
are in [§1.7](#d90).

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

Work includes `review_contract.syntactic` and `review_contract.semantic`, containing
compact projections of the frozen checkpoint criteria, rule conditions/inputs,
checker identity and judge permissions, without duplicating executor bundles.
`exit_checkpoints` retains the full declarations. An empty semantic list means
there is no formal semantic exit gate at that stage. Independent worker review is a
coordinator policy and cannot replace a script result or authorize advancement.
`correction_reasons` identifies retained artifact failures with stage/visit, original
URI, retained URI, expected digest and observed digest or read error.

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

Supplemental read context does not freeze its live source forever. New persistent
runs report `artifact_retention = "completed-stage-snapshots"`: on completion, each
visit retains assessed bytes under `cache/xdesign/WISH_ID/artifact-snapshots/SHA256`.
The run's optional `artifact_snapshot_directory` and each completed visit's
`artifact_snapshots` map bind these locations to the original URI/digest list.
Current-stage checks still read live files. Completed-stage checks use the retained
versions, including when different stages assessed different versions of one URI.
Missing cache snapshots are reconstructed from verified store materials; conflicting
bytes or links are refused. Historical runs without this field retain
`artifact_retention = "legacy-workspace"` and their original live-file checks.

`workspace_changes` reports live product changes relative to the latest completed
artifact per URI; it does not revoke or rewrite historical judgments. Before a new
semantic PASS, the changed live files must be included in `result.artifacts` and the
judgment must bind that complete current snapshot. A missing changed file cannot
silently satisfy this requirement. Completion concerns the assessed snapshot, not
arbitrary later workspace edits. Non-Rust products must supply their actual source,
template and test artifacts explicitly; the built-in Rust selector is not a claim
of automatic whole-product coverage.

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
response. Replaying an exact committed operation does not rerun the engine or any checker:
it publishes the canonical run record and response from the committed revision's verified
materials, so replay still succeeds after the attempt scratch and canonical cache copies
have been lost. A pending operation retains its final request so the same stage command
can resume it without another invocation — including when its transient sources are gone,
because published blobs are reused first. A retry of an operation with no receipt discards
that operation's stale speculative files and repairs the on-disk run record from the
committed head, so a crash mid-attempt or a database timeout before reservation never
wedges the wish on immutable-record drift; the same projection repair applies to readers.
The writer lock is removed on normal exit. After a crash, inspect the store and active
process before manually removing a stale lock; the CLI does not guess that a writer has
abandoned it.

Checker subprocesses run under a hard deadline (default 600 seconds, overridden by
`XDESIGN_CHECKER_TIMEOUT_SECS`). On the supported Unix-like hosts the deadline bounds the
entire process tree: each checker runs in an isolated process group, the group is killed
when the deadline expires, and its stdout/stderr pipes are drained through a bounded drain
that cannot block completion. A descendant that inherits those pipes therefore cannot extend
execution past the deadline, whether the direct child has exited or not. Partial logs are
retained with a timeout note, and the gate reports an explicit `CheckerTimeout` instead of
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

<a id="d89"></a>

### 1.5 Check binding configuration

`xdesign init` accepts `--checks FILE` for `--flow simple-flow` only. The file selects the
check executor of the four syntactic SimpleFlow checkpoints and the semantic
`verificationAccepted` gate; the design contract for the modes, plan freezing and recorded
authority is in [D88](flows/simple-flow/definition.md#d88), and the library format is in
[the checkpoint library design](checkpoint-library.md#d97). The file is strict JSON:

```json
{
  "schema_version": 1,
  "checks": [
    {"checkpoint_id": "simpleRequirementsAccepted", "mode": "default"},
    {"checkpoint_id": "simpleDesignAccepted", "mode": "default"},
    {"checkpoint_id": "simpleImplementationAccepted", "mode": "command",
     "command_uri": "tools/checks/custom_implementation.py"},
    {"checkpoint_id": "simpleRecordAccepted", "mode": "library",
     "library_id": "builtin-simple-record"},
    {"checkpoint_id": "verificationAccepted", "mode": "library",
     "library_id": "verification-evidence-complete"}
  ]
}
```

| Field | Rule |
| --- | --- |
| `schema_version` | required, exactly `1` |
| `checks` | required array; an empty array is equivalent to an all-default binding |
| `checkpoint_id` | required; one of the four syntactic SimpleFlow checkpoint ids, or the semantic id `verificationAccepted` (which accepts only `library` mode); no duplicates |
| `mode` | required; `default`, `command` or `library`; `cli` is rejected for new bindings |
| `command_uri` | required for `command`; a normalized project-relative `.py` path that exists in the repository and implements the invocation and report interface |
| `files`, `python` | optional for `command`; declared helper/data paths and a project-local Python under `cache/`, as specified in [D111](checker-execution.md#d111) |
| `library_id` | required for `library`; a checkpoint id resolvable in the project library; the entry's kind must match the gate (`command` is not accepted on the semantic gate) |

A checkpoint omitted from the file keeps the `default` binding. Unknown fields, unknown
checkpoint ids, duplicate checkpoints, unknown modes, missing required members, empty
`command_uri` values, a missing or unknown `library_id`, and a library entry
whose kind does not match the gate are invalid requests: `init` exits 2 with the error on
stderr before it creates the run or commits a store revision. A non-default binding with
`--flow wish-verify` is rejected because that flow has no library binding in this round; a
default-only file for that flow is accepted as a no-op.
The validated binding is frozen into the plan; `next` and `check` accept no binding
option, and editing the file or the library later does not rebind a registered run
([D88](flows/simple-flow/definition.md#d88), [D101](checkpoint-library.md#d101)).
Historical frozen plans with a `cli` runner remain readable, but `check` cannot invoke
that runner; it returns an unevaluated/blocked result rather than launching an agent.

### 1.6 Checkpoint library commands

The `checkpoint` commands read and update the project library at
`checkpoints/library.json` ([D97](checkpoint-library.md#d97)). Each command prints one JSON
object on stdout: a success reply with exit `0`, or a failure object with exit `2` for an
invalid request or definition and `1` for an internal failure:

```json
{"schema_version": 1, "ok": false, "error": {"code": "invalid_checkpoint", "message": "..."}}
```

The stable failure codes are `invalid_request` (bad arguments or unreadable request file),
`invalid_checkpoint` (malformed library, invalid entry, or malformed save request),
`builtin_checkpoint` (an attempted built-in mutation), `unknown_checkpoint` (a show of an
unknown id), and `internal_error` (I/O failure).

```text
xdesign checkpoint list
xdesign checkpoint show CHECKPOINT_ID
xdesign checkpoint save --request FILE
```

`list` prints the effective library — built-ins plus user entries — in the reply shape
owned by the [checkpoint library interface](../interfaces/web-api.md#i19):

```json
{"schema_version": 1, "library_uri": "checkpoints/library.json", "issues": [],
 "checkpoints": [{"checkpoint_id": "...", "title": "...", "kind": "syntactic",
   "builtin": false, "version": 2, "updated_at": "...", "tags": ["..."],
   "executor": {"mode": "script", "command_uri": "..."},
   "checker_id": "simpleRequirementsCheck",
   "rule_count": 2, "criteria_count": 0}]}
```

The summary's `executor` is `null` for a semantic entry, and `checker_id` is `null`
unless the entry declares a reused-script identity.

`show CHECKPOINT_ID` prints the full entry with its `builtin` flag and whether its script
exists; an unknown id is an invalid request. `save --request FILE` takes the full entry
fields plus an optional `"create_script": "scaffold"` (see [D100](checkpoint-library.md#d100)):

```json
{"checkpoint_id": "energy-budget-positive",
 "title": "Energy budget stays non-negative",
 "description": "Deterministic check that no declared energy budget is negative.",
 "kind": "syntactic",
 "tags": ["energy", "design"],
 "rules": [{"rule_id": "budget-declared",
            "condition": "Every actuator declares a numeric energyBudget.",
            "reads": ["mbse/model/design.sysml"]}],
 "inputs": ["mbse/model/design.sysml"],
 "executor": {"mode": "script",
              "command_uri": "checkpoints/scripts/energy_budget_positive.py"},
 "create_script": "scaffold"}
```

The request may also carry the optional syntactic `checker_id` identity when it reuses a
shipped checker script that reports its own hardcoded id; without it the library id becomes
the frozen checker id and assessor ([D100](checkpoint-library.md#d100)). Saving an existing
user id is an update (`version` + 1, previous snapshot pushed into `history`); saving a
built-in id or a new entry that reuses a built-in id is an invalid request; saving a new id
creates version 1. The reply names the saved entry, its script URI
when one was generated, and whether the save created an entry. `list` and `show` never
execute a checker. An unreadable library is reported as an issue or an invalid-request
error; a missing library file is the built-ins-only library, not an error
([D98](checkpoint-library.md#d98)).

<a id="d90"></a>

### 1.7 Operator web command

`xdesign web` starts the embedded operator page and its local HTTP API:

```text
xdesign web [--host 127.0.0.1] [--port 7788] [--open]
            [--project-root PATH] [--store-root PATH]
```

The server listens on loopback by default (`127.0.0.1`), serves page assets embedded in
the binary, and prints one machine-readable startup line to stdout:
`xdesign web listening on http://HOST:PORT`. `--port 0` binds an ephemeral port and
reports the actual one. `--project-root` defaults to the current directory; the server
changes into it before serving so lifecycle subprocesses and relative paths match the CLI
contract. `--store-root` defaults to `XDESIGN_STORE_ROOT` when set, else
`<project>/.xdesign`. `--open` launches the platform browser opener best-effort and never
fails the command. `SIGINT` terminates the process with default behavior.

The command adds no runtime environment: no Node.js, npm, build step, Python web server,
CDN, separately installed static server, or rendered documentation site is required, and
the server performs no network fetch. The page reads the operational store and drives the
same `init`/`next`/`check` commands as subprocesses of the same executable, so its
lifecycle actions write exactly where the local CLI writes. The HTTP endpoints,
envelopes, status codes, and action behavior are in the
[operator web API](../interfaces/web-api.md#i16); the program design, read/write
boundaries and embedded-asset release model are in
[the operator web design](web-operator.md#d91).

### 1.8 Software realization and verification

| Source | Responsibility |
| --- | --- |
| `src/main.rs` | Routes the top-level commands, including `checkpoint` and `web`, and retains the demo entry points |
| `src/checkpoint/mod.rs`, `library.rs`, `cli.rs` | Library model and validation, built-in seeds, version/history, scaffold generation, and the `checkpoint` command replies |
| `checkpoints/library.json`, `checkpoints/scripts/` | Project authoring source; see the [checkpoint library design](checkpoint-library.md#d98) |
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

New store-backed script bindings retain exact executor bundles. Command bindings accept optional `files` and `python`; library script executors accept the same fields in their definition. See [checker execution](checker-execution.md#d111) for release resources, dependency authoring, interpreter selection, attempt context and legacy limits.

The planned worktree extension classifies legacy candidates before persistent root enrollment and uses one command lease across classification, cache restoration and commit, and adds read-only backend inspection without altering plan/run schemas. See [D119](worktree-store-migration.md#d119) and the exact [I22 interaction contract](../interfaces/worktree-store.md#i22); current lifecycle command support is unchanged by the design alone.
