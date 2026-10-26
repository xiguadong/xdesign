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
5. **Run** — `next` and `check` execute the frozen definition. Editing, renaming, or
   deleting the library entry later changes nothing for that run; a changed definition
   applies only to a new registration.

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
| Syntactic | `checkpoint.library = <library id>`; `checker.checker_id = entry.checker_id` when declared, otherwise the library id; `checker.rules = entry.rules`; `checker.command_uri = entry.script command` (script mode) or the default command URI (CLI mode); `checker.custom` names `process` or `cli` + CLI/model | The stage work freezes `entry.inputs` as the stage's declared inputs instead of invoking `--list-inputs` |
| Semantic | `checkpoint.library = <library id>`; `criteria = entry.criteria`; `agent_may_judge`/`human_may_judge` from `entry.judges`; `checker: null` | No work-input change; the caller's permitted judgment evaluates the frozen criteria as today |

Kind mismatch, a missing entry, an unknown library id, or a `library` binding on a gate
the flow does not support is an initialization error: no run is created and no store
revision is committed. `wish-verify` keeps its two semantic gates and has no library
binding in this round.

Everything resolved is frozen, so the library is not re-read later:

- `next`, `check`, restore, the record checker, and the operator page reconstruct the
  binding from the frozen plan; a later library edit, rename, or delete changes nothing for
  an existing run.
- A default or non-library run is byte-identical to before; library mode adds frozen fields
  only to the run that chose it.
- The frozen checker id for a script binding is the entry's declared `checker_id`, or the
  library id when it declares none, so an assessment names the executor that answered the
  frozen rules rather than the built-in checker it replaced.
- To use a changed definition, register a new wish; this round exposes no rebind action.

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
- It adds no SysML element in this round. The design claim for the library rests on this
  page, the interface pages, and the source; the modeled flow declarations are unchanged.
- A library entry is not evidence. A passing bound check creates an assessment with
  retained evidence; a declared case in the verification design, a green scaffold, or a
  validated entry alone is not an executed engineering result.

The verification cases for this contract are [LIB-01 to LIB-10](../verification/design.md#d103);
they are verification design, not executed acceptance.
