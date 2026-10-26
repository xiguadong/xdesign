# Checkpoint library

The checkpoint library is a project-level, Git-tracked authoring source for reusable
checks: a developer declares a syntactic rule set with its executor or a semantic
criterion set with its judges once, and any wish may bind that definition to a supported
SimpleFlow gate at initialization. The library is *authoring source*, not operational
state — the frozen run records what it bound, and later library edits never rewrite an
existing run. This page states the library model, file format, lifecycle, built-in
entries, versioning, script and scaffold behavior, runtime binding and freezing, and its
limits. The command surface is in the [stage CLI](stage-cli.md#d85); the binding contract
in the flow definition is [D88](flows/simple-flow/definition.md#d88); the checkpoint HTTP
endpoints are in the [operator web API](../interfaces/web-api.md#i19); and the verification
cases are in [the verification design](../verification/design.md#d103).

<a id="d97"></a>

## 1. Purpose, lifecycle, and ownership

Development repeatedly answers the same verification questions: an energy budget must be
non-negative, requirements must carry rationale, evidence must answer every requirement.
Writing the same rule set again for each wish invites drift, and a flow-level declaration
alone cannot say which of several available rule sets a particular wish should use. The
library separates the *reusable check definition* from the *gate binding*:

1. **Author** — a developer adds or updates an entry in `checkpoints/library.json`,
   either by editing the file directly (it is Git-tracked project source) or through
   `xdesign checkpoint save` and the operator page's New checkpoint form. A syntactic
   entry points at an existing script or asks the runtime to generate a scaffold script.
2. **Validate on load** — every library use reads the project library and validates it.
   A missing file means "built-ins only"; malformed JSON or an invalid entry is an
   explicit error, never a guess.
3. **Bind at initialization** — `xdesign init --checks FILE` accepts a `library` mode for
   the four syntactic SimpleFlow gates and the semantic `verificationAccepted` gate. The
   project library is read from the project root at that moment.
4. **Freeze** — the resolved rules or criteria, the executor, the judges, and the declared
   work inputs are frozen into the plan and work definition. The run no longer reads the
   library entry.
5. **Run** — `next` and `check` use the frozen rules, criteria and executor selection.
   Library edits do not replace those fields. A script executor still reads its current
   project file: script bytes and dependencies are not pinned at initialization.

The library is project content, not a service or an operational record. `checkpoints/` is
Git-tracked authoring source like the SysML model and the checker commands, reviewed and
committed like any other design product. The `.xdesign` store remains the operational
record of what each wish's frozen plan actually bound. The library is therefore not a
second record authority: the store keeps the run, and the library keeps only the reusable
definitions from which a run may be initialized.

| Artifact | Role | Authority |
| --- | --- | --- |
| `checkpoints/library.json` and `checkpoints/scripts/` | Reusable authoring source | Git-tracked project content; edited by developers |
| Built-in entries | Binary-provided seed definitions | Read-only; shipped with the executable |
| Frozen plan and work definition | The resolved definition a wish bound | Operational record in the store; never rewritten by library edits |
| `.xdesign` store revisions, materials, receipts | The wish's run facts | Operational authority for the run |

<a id="d98"></a>

## 2. The library file and entry model

The library lives at the project root:

```text
checkpoints/
  README.md              short usage note
  library.json           user-maintained entries
  scripts/<slug>.py      checker scripts, including generated scaffolds
```

`checkpoints/library.json` is strict JSON with a schema version and an entry array:

```json
{
  "schema_version": 1,
  "checkpoints": [
    {
      "checkpoint_id": "energy-budget-positive",
      "title": "Energy budget stays non-negative",
      "description": "Deterministic check that no declared energy budget is negative.",
      "kind": "syntactic",
      "tags": ["energy", "design"],
      "rules": [
        {"rule_id": "budget-declared",
         "condition": "Every actuator declares a numeric energyBudget.",
         "reads": ["mbse/model/design.sysml"]},
        {"rule_id": "budget-non-negative",
         "condition": "Every declared energyBudget is >= 0.",
         "reads": ["mbse/model/design.sysml"]}
      ],
      "inputs": ["mbse/model/design.sysml"],
      "executor": {"mode": "script",
                   "command_uri": "checkpoints/scripts/energy_budget_positive.py"},
      "version": 1,
      "created_at": "2026-10-26T00:00:00Z",
      "updated_at": "2026-10-26T00:00:00Z",
      "source": "generated",
      "history": []
    },
    {
      "checkpoint_id": "verification-evidence-complete",
      "title": "Verification evidence answers every requirement",
      "description": "A permitted judge can re-examine the retained evidence and see each requirement answered.",
      "kind": "semantic",
      "tags": ["verification"],
      "criteria": [
        "Every requirement has retained evidence that a reader can re-examine.",
        "The evidence and the record stand on their own without re-asking the author."
      ],
      "judges": {"agent": true, "human": true},
      "version": 1,
      "created_at": "2026-10-26T00:00:00Z",
      "updated_at": "2026-10-26T00:00:00Z",
      "source": "manual",
      "history": []
    }
  ]
}
```

Every entry has a `checkpoint_id`, a title, a description, a `kind`, tags, version and
timestamps, a `source`, and a history. The kind decides which of the two shapes the rest
of the entry takes:

| Field | Rule |
| --- | --- |
| `checkpoint_id` | Slug `[a-z0-9][a-z0-9_-]{0,63}`; unique among user entries, and no built-in id may be reused |
| `title` | 1–120 characters, required |
| `description` | 1–1000 characters, required |
| `kind` | `syntactic` or `semantic`, required |
| `tags` | At most 8 slug tags |
| `version`, `created_at`, `updated_at` | Server-managed: computed by the save path. A save request may echo them (for example when posting a full `show` payload back) but the save path ignores the echoed values |
| `source` | `builtin`, `generated`, or `manual`; set by the save path |
| `history` | Previous entry snapshots, newest first, at most 10 (see [D99](#d99)) |

A **syntactic** entry is a rule set that a deterministic or agent executor can answer:

| Syntactic field | Rule |
| --- | --- |
| `rules` | At least one `{rule_id, condition, reads[1..]}`; `reads` uses the same vocabulary as checker reads |
| `inputs` | At least one normalized project-relative file that exists; frozen into the stage's work at initialization |
| `executor` | `{"mode": "script", "command_uri": "<project-relative .py>"}` or `{"mode": "cli", "cli": "codex"\|"kimi", "model": "<non-empty>"}` |
| `checker_id` | Optional checker identity `[A-Za-z0-9][A-Za-z0-9_-]{0,127}` used when the executor is a shipped checker script that reports its own hardcoded identity. When present it becomes the frozen `checker.checker_id` and the assessment assessor; otherwise the library id is the identity. Not allowed on a semantic entry |

A **semantic** entry is a criterion set that a permitted judge reads:

| Semantic field | Rule |
| --- | --- |
| `criteria` | At least one non-empty criterion |
| `judges` | `{"agent": <bool>, "human": <bool>}` with at least one true |
| rules, inputs, executor | Not allowed on a semantic entry |

An entry that mixes the two shapes, omits a required kind-specific field, or carries an
unknown field is invalid. The runtime never guesses an entry's kind and never coerces a
semantic definition into a syntactic executor. A missing `checkpoints/library.json` is not
an error: the effective library is the built-in entries plus zero user entries. A malformed
library file or an invalid user entry on load is a hard error — listing fails explicitly
rather than showing a partial or guessed library. The `issues` list in a listing reports
only non-fatal concerns, currently a script executor whose `command_uri` no longer names an
existing project file (see [D100](#d100)), so a reader sees a missing script without losing
the readable definitions.

<a id="d99"></a>

## 3. Built-in entries, versions, and history

The binary provides seed entries so a project can see what a library entry looks like and
so built-in checks remain listable without a user library file. The seeds are:

| Built-in id | Kind | Definition source |
| --- | --- | --- |
| `builtin-simple-requirements` | syntactic | The `simpleRequirementsCheck` rules, command URI, and checker id from the default SimpleFlow plan |
| `builtin-simple-design` | syntactic | The `simpleDesignCheck` rules, command URI, and checker id from the default plan |
| `builtin-simple-implementation` | syntactic | The `simpleImplementationCheck` rules, command URI, and checker id from the default plan |
| `builtin-simple-record` | syntactic | The `simpleRecordCheck` rules, command URI, and checker id from the default plan |
| `builtin-simple-verification` | semantic | The `verificationAccepted` criteria copied from the default plan |

Built-in entries appear in every listing with `"builtin": true`. They cannot be saved,
overwritten, or deleted through the library commands or the operator API: a save that
names a built-in id is rejected, not silently ignored. They serve as read-only templates —
the operator page can pre-fill a new user entry from a built-in definition, but the new
entry gets its own user slug and its own version history.

A user save behaves according to the id:

- a save with a new id creates the entry at `version: 1` with empty history;
- a save with an existing user id is an **update**: `version` increments by one and the
  previous snapshot is pushed into `history` (newest first, at most ten kept);
- a save that names a built-in id is an error.

A history item keeps `{version, updated_at, description, kind, rules, inputs, executor,
criteria, judges, tags}` so a reader can see what a rule set looked like at an earlier
version without reconstructing it from Git. History lives in the library file itself; the
`.xdesign` store is not involved in authoring revisions. Beyond the ten retained snapshots,
Git history is the complete record of the authoring source.

<a id="d100"></a>

## 4. Executors, script reuse, and scaffold generation

A syntactic library entry declares its executor. Semantic entries have no executor: their
criteria are judged by a permitted agent or human through the stage interaction.

| Executor | Meaning | Recorded authority of an assessment |
| --- | --- | --- |
| Script, existing file | Run the entry's `command_uri` as a deterministic checker under the frozen checker id | `script`, assessor = frozen checker id (the entry's `checker_id` when declared, otherwise the library id) |
| Script, generated scaffold | Generate a self-contained checker at `checkpoints/scripts/<id-with-underscore>.py`, then behave as the existing-file case | `script`, assessor = frozen checker id |
| Agent CLI | Invoke `codex` or `kimi` with the named model; it answers the entry's frozen rules | `agent`, assessor = `<cli>:<model>` |
| Semantic entry | No executor; the caller supplies a permitted agent or human judgment over the frozen criteria | the caller's permitted authority |

Script reuse is the normal path: set `executor.mode = "script"` with a `command_uri` that
already exists and implements the checker invocation and report interface. The generated
script is Git-tracked like any other source file and may be edited by its author.

Scaffold generation is requested with `"create_script": "scaffold"` on a save request. It
is allowed only with a syntactic script executor whose `command_uri` is
`checkpoints/scripts/<id-with-underscores>.py`. The runtime writes a deterministic,
self-contained Python checker that:

- supports `--list-inputs` and the full checker invocation flag set
  (`--wish-model --record --checker-id --plan-id --plan-uri --plan-sha256 --stage
  --artifact-revision --snapshot-digest --report`);
- verifies the plan, run-record, and visit binding like the shipped checkers before
  reporting;
- writes the raw report shape the runtime requires — `checker_id`, `stage`,
  `artifact_revision`, `snapshot_digest`, `report_uri`, `overall_verdict`, `rules[]`
  (`rule_id`, `verdict`, `inputs`, `observed`, `failures`), and `input_sha256` covering
  the wish model, record snapshot, plan, declared inputs, and the script's own bytes;
- ships one TODO rule function per declared rule that returns a failed verdict with
  `"rule not implemented: <rule_id>"`, with the rule condition and reads as comments.

The scaffold is honest and safe by construction: it can run, but it cannot silently pass a
rule its author has not implemented. The entry's `source` records `generated` for a scaffold
and `manual` for an entry created without scaffold generation; reusing an existing script
needs no `create_script` flag.

**Script identity.** A shipped `tools/checks/*.py` checker rejects an invocation whose
`--checker-id` differs from its hardcoded id. When a syntactic entry reuses one of those
scripts, it may set the optional `checker_id` field to that identity (for example
`simpleRequirementsCheck`); the frozen plan then records it as the assessment assessor
while `checkpoint.library` keeps the library provenance ([D101](#d101)). A generated
scaffold echoes whatever `--checker-id` the runtime passes, so it needs no `checker_id`.

<a id="d101"></a>

## 5. Runtime binding and plan freezing

A wish binds a library entry through the `library` mode of the check-binding file:

```json
{"schema_version": 1, "checks": [
  {"checkpoint_id": "simpleRequirementsAccepted", "mode": "library",
   "library_id": "energy-budget-positive"},
  {"checkpoint_id": "verificationAccepted", "mode": "library",
   "library_id": "verification-evidence-complete"}
]}
```

The supported gates are the four syntactic SimpleFlow gates and the semantic
`verificationAccepted` gate. Resolution happens once, at `init`, from the project root:

| Gate kind | Frozen plan fields | Frozen work fields |
| --- | --- | --- |
| Syntactic | `checkpoint.library = <library id>`; `checker.checker_id = entry.checker_id` when declared, otherwise the library id; `checker.rules = entry.rules`; `checker.command_uri = entry.script command` (script mode) or the default command URI (CLI mode); `checker.custom` names `process` or `cli` + CLI/model | Requirements/design freeze `entry.inputs` instead of invoking `--list-inputs`; implement/verify keep their built-in selectors |
| Semantic | `checkpoint.library = <library id>`; `criteria = entry.criteria`; `agent_may_judge`/`human_may_judge` from `entry.judges`; `checker: null` | No work-input change; the caller's permitted judgment evaluates the frozen criteria as today |

Kind mismatch, a missing entry, an unknown library id, or a `library` binding on a gate
the flow does not support is an initialization error: no run is created and no store
revision is committed. `wish-verify` keeps its two semantic gates and has no library
binding in this round.

The resolved plan fields are frozen, so the library is not re-read for evaluation:

- `next`, `check`, restore, the record checker, and the operator page reconstruct the
  binding from the frozen plan; a later library edit, rename, or delete changes nothing for
  an existing run.
- A default or non-library run is byte-identical to before; library mode adds frozen fields
  only to the run that chose it.
- The frozen checker id for a script binding is the entry's declared `checker_id`, or the
  library id when it declares none, so an assessment names the executor that answered the
  frozen rules rather than the built-in checker it replaced.
- To use a changed definition, register a new wish; this round exposes no rebind action.

The frozen provenance is only the library ID. It does not include the entry version,
definition digest, upstream origin or script digest. The script path is frozen, while
`run_process_check` invokes the current file at that path. A report's `input_sha256`
materials retain the bytes that the report actually declares; generated scaffolds declare
their own script bytes. This is evidence of a particular execution, not an initialization
lock on the executor or its imported dependencies. Library-selected input paths replace
the built-in selectors only for requirements and design; implement and verify retain
their existing output selection (`src/stage_cli/work.rs::freeze_with_inputs`).

The operator page shows the frozen plan's `library_id` and frozen rules or criteria first.
It resolves the current library only for a display summary (`title`, `version`, `builtin`)
and reports a warning when a frozen library id can no longer be resolved in the current
project library; that warning never changes the run.

<a id="d102"></a>

## 6. Limits and what the library does not do

- It is project-local: one `checkpoints/library.json` at the project root. There is no
  remote or shared registry, no cross-project resolution, and no version constraint
  language.
- It is authoring source, not a workflow: list/show/save and load never execute a checker
  or a judgment; execution happens only when a bound run's gate is evaluated.
- It does not rebind history: library edits never change a registered run, and deleting an
  entry does not invalidate a run that frozen it ([D101](#d101)).
- Its history records the last ten authoring snapshots in the library file; longer history
  is Git history, not store history, and the `.xdesign` store is not asked to version the
  library.
- The original delivery added no SysML elements. The current as-built allocation is
  [below](#d107); it does not establish that the earlier implementation followed a flow.
- A library entry is not evidence. A passing bound check creates an assessment with
  retained evidence; a declared case in the verification design, a green scaffold, or a
  validated entry alone is not an executed engineering result.

The verification cases for this contract are [LIB-01 to LIB-10](../verification/design.md#d103);
they are verification design, not executed acceptance.

<a id="d107"></a>

## 7. As-built model and real-project boundary

This allocation describes the existing implementation. It was added after the original
feature delivery and does not backdate its requirements, reviews or flow history.

| Model element | Responsibility | Existing source |
| --- | --- | --- |
| `XDesignArchitecture::ProjectCheckContext` | One selected project root, library and store | `web/server/mod.rs` project-root selection; `web/server/api.rs::AppState` |
| `XDesignArchitecture::CheckpointLibrarySource` | Project-owned definitions and scripts | `src/checkpoint/library.rs::LIBRARY_URI`, `SCRIPTS_DIRECTORY`, `RsCheckpointLibrary::load` |
| `XDesignArchitecture::ReusableCheckpointDefinition` | Kind-specific rules/criteria, inputs, executor, local version and judge permissions | `src/checkpoint/library.rs::RsCheckpointEntry` |
| `XDesignArchitecture::OperatorWebProgram` | Embedded local operator, delegated lifecycle, source authoring | `web/server/`, `web/assets/` |
| `XDesignDetailedDesign::CheckpointLibraryService` | Validation, save/history, scaffold and binding resolution | `src/checkpoint/library.rs`, `src/simple_flow/blueprint.rs` |
| `XDesignDetailedDesign::FrozenCheckBinding` | Resolved fields across frozen plan and work | `src/core/plan.rs`, `src/stage_cli/work.rs`, `src/stage_cli/persistent.rs` |
| `XDesignRustUml::RsCheckpointLibrary`, `RsCustomChecker`, `RsOperatorAppState` | Software realization projections | Existing library service, custom-runner struct and private HTTP application state |

The concrete `xdesign.projectChecks` references the existing `xdesign.store`;
`xdesign.webOperator` references that project context and `xdesign.cli`. This is an
operator integration into the composition, not an implementation of the generic controller.
The requirements and their verification allocation are in
[project checkpoint requirements](../requirements/requirements.md#r16) and
[design review cases](../verification/design.md#d110).

### 7.1 Using the current binary in another project

`xdesign web --project-root /path/to/customer-project` reads and writes
`/path/to/customer-project/checkpoints/library.json`, resolves scripts and input paths
under that project, and defaults to `/path/to/customer-project/.xdesign`. The installation
directory of the executable is not the customer's project root. CLI checkpoint and stage
commands likewise run from the selected project directory. Registration initializes the
selected store when needed (`src/stage_cli/persistent.rs::init`).

| Content | Current location in the selected project | Preservation rule |
| --- | --- | --- |
| Reusable rules/criteria and scripts | `checkpoints/library.json`, `checkpoints/scripts/` or another declared project-relative script path | Commit engineering source to that project's Git repository |
| Optional init binding request | Caller-supplied `--checks FILE`; the web program uses `cache/web/checks/` | Disposable after resolution; it is not the run authority |
| Resolved rules, criteria and executor selection | Frozen plan/work materials in the project's store | Preserve immutable store revisions |
| Check attempts and judgments | Wish run visits and assessments in the store, with report/input/log/evidence materials | Preserve failed and unevaluated attempts as well as passes |
| Working wish, plan, reports and exchanges | `cache/wishes/`, `cache/xdesign/` and other declared cache paths | Regenerable working copies when the corresponding materials were retained |

The built-in SimpleFlow checks assume this repository's model and source layout, and the
binary does not ship the default `tools/checks/*.py` files. A fresh customer project can
list the built-in definitions without being able to execute them. It needs compatible
project scripts and input mappings; copying only `library.json` is insufficient. Library
input selection currently changes requirements/design only. General project templates and
arbitrary stage output mapping remain unimplemented.

### 7.2 Three meanings of history

**Definition history** answers how a reusable check changed. The JSON keeps at most ten
partial prior snapshots: title, `checker_id`, source metadata and script bytes are absent
from those snapshots. Direct file edits do not automatically append them. Git retains
committed definitions and scripts; uncommitted changes are not a complete archive.

**Binding history** answers which definition a run selected. Its frozen plan keeps the
local library ID and resolved rules/criteria/executor fields. It does not identify the
library version or upstream commit. The operator's current-library title/version summary
is a live lookup and must not be read as the historical version used by the run.

**Execution history** answers what ran, over which revision, with what verdict, authority
and evidence. It belongs to each project's immutable wish revisions and assessments.
Neither the ten definition snapshots nor a current library listing can replace it.

### 7.3 Authority deviation and preservation limit

The generic requirements `syntacticCheckIsFixed` and `syntacticAssessmentIsScripted`, and
`CheckpointAssessment::judgmentMatchesCheckpointKind`, specify deterministic script
assessment of syntactic checkpoints. The current CLI binding retains `kind = syntactic`
while recording `judged_by = agent`. `RsCustomChecker` models that implemented selection;
it does not make the generic constraint true. The compiler does not evaluate that
constraint body. The policy needs a separate design decision: retain the scripted-only
contract and reclassify agent gates, or explicitly model rule kind and permitted executor
authority as separate dimensions. No generic authority requirement is weakened here.

An external `--store-root` or `XDESIGN_STORE_ROOT` changes a local storage location; it
does not establish project identity or a multi-project namespace. Sharing one store among
unrelated projects can collide on wish IDs and gives the web server one project root for
all artifact paths. It is not a supported federation mechanism. Also, the current store
is bound to filesystem identity and refuses a moved or copied root. Its
[recovery boundary](storage.md#d86) prevents treating a raw directory copy as a supported
backup. Real deployment needs verified backup/restore before operational history can be
entrusted solely to this store.

<a id="d108"></a>

## 8. Proposed cross-project ownership and reuse

The selected scope is shared checkpoint definitions plus a unified history view, with
each project retaining its own authoritative records. The requester confirmed this
direction on 2026-10-26; the decision is retained in store record
`record-review-checkpoint-sharing-direction`, revision 1. Selecting this scope does not
approve a serialized format or establish implemented cross-project behavior.

The package, identity and storage contracts below remain design proposals.
`ProposedProjectChecks` models the identities needed to develop that direction. It
introduces no runnable flow or storage schema.

### 8.1 Ownership and distribution

| Concern | Proposed owner and location | Change behavior |
| --- | --- | --- |
| Project-specific definitions and adapters | Consumer project Git source under `checkpoints/` | Reviewed changes affect new bindings |
| Shared reusable definitions and executor dependencies | Dedicated versioned Git package owned by a team or organization | Publish immutable revisions; upgrades are explicit |
| Imported package bytes | Consumer-project vendor directory, for example `checkpoints/vendor/<namespace>/<revision>/` | Keep executable dependencies inside the selected project; no reference into another project's workspace |
| Selected imports and project input mapping | Proposed Git-tracked `checkpoints/lock.json` | Pin namespace, ID, origin revision, definition and executor-bundle digests; reject ambiguity or digest mismatch |
| Run binding and each evaluation | Consumer project's store | Retain the resolved definition and exact declared executor bundle, inputs, environment identity and evidence |
| Cross-project history view | Rebuildable index of registered project stores | Link to authoritative revisions; expose unavailable sources and last-observed revision |

The vendor and lock paths above are proposed examples. The current strict library schema
does not accept these import fields. A minimal reuse workflow today is an explicit Git
copy of the definition, script and its dependencies into each consumer project, followed
by project-specific input adaptation and review. That does not create automatic upstream
provenance, update propagation or a shared registry.

Resolution must name a local definition or a qualified imported definition explicitly.
Two packages containing `requirements-complete` must not silently shadow one another.
Built-in IDs stay reserved. A package's human version is for navigation; exact content
digests and an immutable upstream revision establish the binding. Updating an imported
package creates a new binding for a new run, or a future explicit rebind with its own
record. It never rewrites an earlier assessment.

Pinning the script entry file alone is insufficient: declare and retain imported helper
modules, rule data and required tool/environment versions. Input selectors need an
explicit mapping to the consumer project's layout. A historical evaluation records the
bundle actually used. Re-execution is a new attempt and does not promise identical agent
answers or erase the old result.

### 8.2 History identity and project lifecycle

A proposed reference includes `(project_id, store_id, item_id, store_revision)` and the
run, stage, visit, artifact revision and assessment index. A checkpoint reference also
names its qualified definition and content digest. Local slugs, absolute paths, human
version numbers and timestamps are not globally unique identities.

`project_id` would be stable project configuration in Git; local workspace and store paths
would be machine-local registrations. These are proposed fields, not existing config.
Repositories forked as new projects need explicit new identity; worktrees of the same
project retain project identity but name their selected store. Stores are distinct
execution authorities, so a project ID alone cannot identify an assessment. Concurrent
worktrees also need an explicit workspace/materialization isolation contract before they
share writable execution state.

| Situation | Required behavior of the proposed design |
| --- | --- |
| A and B use the same check | Each store keeps its own binding, attempts and evidence; the shared package keeps definition releases |
| A and B use the same wish ID | Composite project/store identity prevents merging their histories |
| Package updated or deleted upstream | Retained local bundle and frozen binding still identify prior evaluations |
| New clone or another machine | Git reconstructs authoring source; operational history requires explicit verified transfer/restore |
| Workspace moved or removed | Registration can change, but the current store cannot be moved/adopted implicitly; the future restore contract must preserve logical provenance |
| Store unreachable | The index reports an unavailable source; it must not infer success, empty history or current status |
| Project archived | Retain its evidence-bearing store through a verified archival/restore path before removing workspace or storage |

An optional central service could later become the authority, but that is a different
storage design requiring project namespaces, identity, concurrency, retention and restore.
Pointing today's single-project runtime at one global `.xdesign` directory does not
implement it. The initial proposal keeps the cross-project view read-only and derived.

<a id="d109"></a>

## 9. Remediation sequence and acceptance boundary

The earlier delivery's plan/execution records remain historical facts. A new remediation
must register its actual work, attach products and verification evidence, and distinguish
its review from the earlier implementation. Adding this design does not retroactively
make that implementation pass requirements or design gates.

The supporting plan is store record `record-plan-checkpoint-project-design`, revision 1
(`python3 tools/records.py show record-plan-checkpoint-project-design --revision 1`).
Its new run is `wish-checkpoint-project-design-20261026`; the store retains the answering
artifacts and judgments, separately from the earlier feature execution records.

For the current documentation/model reconciliation, use the executable `wish-verify`
flow: record the request and design artifacts at `wish`, then judge the design and its
retained compiler/documentation results at `verify`. Both gates remain semantic. Compiler
checks are supporting evidence, not an automatically installed `DesignFlow` gate; the
modeled `DesignFlow` has no executable stage-CLI registration. The authoring agent's
judgment is not an independent review or final human acceptance.

Runtime extensions should follow a new gated `simple-flow` run after the design policy
is decided: requirements and acceptance criteria, architecture/detailed design and
interface impact, implementation, then executed verification. Select existing applicable
checks before implementing new checks; a new checker cannot authorize its own creation.

The dependency order is: resolve the gate-authority policy; define real-project input
mapping and frozen executor/provenance contracts; design store backup/restore and project
identity; add pinned package import; then add cross-project history projection. The
[verification matrix](../verification/design.md#d110) states the current design review
and future executable obligations separately. No cross-project execution, package import
or portable store recovery is delivered by this reconciliation.
