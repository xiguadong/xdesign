# xdesign web contract (working specification)

This file is the implementation contract between the web server module
(`web/server/`) and the operator page (`web/assets/`). The published design
lives under `docs/`. The operator does not launch stage workers or agent CLI
checkpoint checkers. Existing CLI-bound plans remain readable as historical
records, but cannot run that checker again.

## 1. Product constraints

- The released product is the single `xdesign` executable plus the repository
  content it operates on (project model, checkers, docs). The web program ships
  **inside** that executable: page assets are embedded at compile time by
  `web/server/build.rs` and served from memory. No Node.js, npm, Python web
  server, CDN, or separately installed static server is required at runtime.
- The page must not reuse the documentation site or its API. It reads the
  operational `.xdesign` store through the Rust storage API and drives the
  existing local stage CLI (`xdesign init/next/check`) as a subprocess of the
  same executable.
- The page code lives in the repository root `web/` directory. The server is a
  Rust module of the `xdesign` library, referenced from `src/lib.rs` with
  `#[path = "../web/server/mod.rs"]`, so all web code stays under `web/`.
- Default bind address is loopback. The server performs no network fetch.
- No third-party Rust crates, no third-party JavaScript. Standard library and
  the existing `serde`/`serde_json`/`uuid`/`sha2` dependency set only.

## 2. Repository layout

```
web/
  SPEC.md                 this contract (authoritative for the branch)
  README.md               operator and developer notes
  assets/                 embedded page assets (frontend ownership)
    index.html            required entry document
    app.js                required single-page application
    styles.css            required stylesheet
    favicon.svg           optional
  server/                 Rust server module (backend ownership)
    build.rs              asset embedding (Lead-owned plumbing)
    mod.rs                serve() entry, option parsing, module root
    http.rs               minimal HTTP/1.1 request/response layer
    api.rs                API data assembly
```

New files under `web/assets/` are embedded automatically; the frontend may split
JavaScript into `web/assets/js/*.js` and reference it with root-relative URLs
(`/js/foo.js`). Server module files must be added to `mod.rs` by the backend.

## 3. Server command

```
xdesign web [--host 127.0.0.1] [--port 7788] [--open]
            [--project-root PATH] [--store-root PATH]
```

- `--host` default `127.0.0.1`; `--port` default `7788`; `--port 0` binds an
  ephemeral port and reports the actual one.
- `--project-root` default: current working directory. The server `chdir`s into
  the project root before serving so lifecycle subprocesses and relative paths
  match the CLI contract.
- `--store-root` default: `XDESIGN_STORE_ROOT` if set, else `<project>/.xdesign`.
- `--open` launches the platform browser opener best-effort (never fatal).
- Startup prints exactly one machine-readable line to stdout:
  `xdesign web listening on http://127.0.0.1:7788`
  followed by human-readable hints. The test harness polls this line.
- SIGINT terminates the process with default behavior.

## 4. HTTP conventions

- HTTP/1.1, one request per connection (`Connection: close`).
- Methods: `GET`, `POST`; anything else `405`.
- JSON responses use the envelope:
  - success `{"ok":true,"data":<payload>}` HTTP 200
  - failure `{"ok":false,"error":{"code":"<stable_code>","message":"<human text>"}}`
    with HTTP 400 (invalid request), 404 (unknown wish or route), 409 (state
    conflict), 413 (oversized body), 405, or 500.
- `Content-Type: application/json; charset=utf-8` for JSON; `no-store` caching;
  `X-Content-Type-Options: nosniff`.
- Body limit 1 MiB, read timeout 10 s, response bodies always have an explicit
  `Content-Length`.
- Path segments are read-only identifiers: `wish_id` matches
  `[A-Za-z0-9][A-Za-z0-9_-]{0,127}`. Unknown paths under `/api/` are 404 JSON.

## 5. API endpoints

All payloads below are the `data` member of the envelope.

### 5.1 `GET /api/health`

```json
{
  "version": "0.1.0",
  "project_root": "/abs/path",
  "store_root": "/abs/path/.xdesign",
  "store_present": true,
  "now": "2026-10-25T23:59:00Z"
}
```

### 5.2 `GET /api/flows`

```json
{
  "flows": [
    {
      "flow_id": "simple-flow",
      "title": "SimpleFlow",
      "summary": "Four gated stages: requirements, design, implement, verify.",
      "custom_checks_supported": true,
      "stages": [
        {"stage_id": "requirements", "title": "Requirements",
         "goal": "Define requirements and their rationale.",
         "instructions": "State the requirements, rationale and satisfaction relationships in the project model.",
         "expected_outputs": ["checker:simpleRequirementsCheck"]},
        {"stage_id": "design", "title": "Design", "...": "..."},
        {"stage_id": "implement", "title": "Implementation", "...": "..."},
        {"stage_id": "verify", "title": "Verification", "...": "..."}
      ],
      "checkpoints": [
        {
          "checkpoint_id": "simpleRequirementsAccepted",
          "stage_id": "requirements",
          "kind": "syntactic",
          "title": "simpleRequirementsCheck",
          "criteria": ["Every requirement declared in the project model carries its rationale, ..."],
          "agent_may_judge": false,
          "human_may_judge": false,
          "checker": {
            "checker_id": "simpleRequirementsCheck",
            "command_uri": "tools/checks/simple_requirements.py",
            "establishes": ["identifier_conformance", "trace_conformance"],
            "rules": [
              {"rule_id": "requirements-carry-rationale",
               "condition": "XDesignRequirements declares at least one requirement, ...",
               "reads": ["mbse/model/requirements.sysml"]},
              {"rule_id": "satisfy-links-resolve", "...": "..."}
            ],
            "modes": ["default", "library"]
          }
        },
        {
          "checkpoint_id": "verificationAccepted",
          "stage_id": "verify",
          "kind": "semantic",
          "title": "verificationAccepted",
          "criteria": ["Verification evidence demonstrates that the implementation satisfies the requirements.", "..."],
          "agent_may_judge": true,
          "human_may_judge": true,
          "checker": null
        }
      ]
    },
    {
      "flow_id": "wish-verify",
      "title": "WishVerifyFlow",
      "summary": "Two judged stages: wish, verify.",
      "custom_checks_supported": false,
      "stages": [{"stage_id": "wish", "...": "..."}, {"stage_id": "verify", "...": "..."}],
      "checkpoints": [
        {"checkpoint_id": "wishAccepted", "kind": "semantic", "checker": null, "...": "..."},
        {"checkpoint_id": "outcomeAccepted", "kind": "semantic", "checker": null, "...": "..."}
      ]
    }
  ]
}
```

The flow catalog exposes the built-in and library binding choices. It does not
probe local agent executables or return an agent catalog.

### 5.3 `GET /api/wishes`

```json
{
  "wishes": [
    {
      "wish_id": "wish-0011",
      "title": "Storage slice",
      "description": null,
      "flow_id": "simple-flow",
      "status": "delivered",
      "current_stage": null,
      "head_revision": 5,
      "source": "store",
      "updated_at": null
    },
    {
      "wish_id": "wish-0005",
      "title": "wish-0005",
      "description": "Produce documentation under docs/ ...",
      "flow_id": "wish-verify",
      "status": "delivered",
      "current_stage": null,
      "head_revision": null,
      "source": "legacy",
      "updated_at": null
    }
  ]
}
```

Sources:

- `store`: `kind = "wish"` items in the operational store, ordered by item id.
  `status` is the store projection (`received`, `active`, `blocked`,
  `delivered`). `head_revision` is the store head revision.
- `legacy`: writable `cache/xdesign/<wish_id>/registry.json` directories that
  are not store items, listed read-only. `status` maps run record status
  `pending|active|blocked|completed` to `received|active|blocked|delivered`.
- `title`/`description` come from the page-created wish source marker comment
  `// xdesign-wish: {"title": "...", "description": "..."}` when present, else
  from the first SysML `attribute redefines description = "..."` occurrence in
  the wish source, else `title` is the wish id and `description` is null.
- Results are sorted by status priority (`blocked`, `active`, `received`,
  `delivered`) then wish id.

### 5.4 `GET /api/wishes/{wish_id}`

```json
{
  "wish_id": "wish-0011",
  "title": "Storage slice",
  "description": "...",
  "flow_id": "simple-flow",
  "status": "delivered",
  "current_stage": null,
  "head_revision": 5,
  "source": "store",
  "plan": {
    "plan_id": "designedSimpleFlow",
    "plan_sha256": "…64 hex…",
    "stages": [
      {"stage_id": "requirements", "next_stage_ids": ["design"],
       "exit_checkpoint_ids": ["simpleRequirementsAccepted"], "completes_flow": false}
    ],
    "checkpoints": [
      {
        "checkpoint_id": "simpleRequirementsAccepted",
        "stage_id": "requirements",
        "kind": "syntactic",
        "title": "simpleRequirementsCheck",
        "criteria": ["..."],
        "agent_may_judge": false,
        "human_may_judge": false,
        "checker": {
          "checker_id": "simpleRequirementsCheck",
          "command_uri": "tools/checks/simple_requirements.py",
          "establishes": ["identifier_conformance"],
          "rules": [{"rule_id": "...", "condition": "...", "reads": ["..."]}],
          "binding": {"mode": "default"}
        }
      },
      {
        "checkpoint_id": "verificationAccepted",
        "stage_id": "verify",
        "kind": "semantic",
        "title": "verificationAccepted",
        "criteria": ["..."],
        "agent_may_judge": true,
        "human_may_judge": true,
        "checker": null
      }
    ]
  },
  "run": {
    "run_id": "…",
    "execution_type": "FlowRun",
    "plan_id": "designedSimpleFlow",
    "status": "completed",
    "current_visit": 3,
    "visits": [
      {
        "stage_id": "requirements",
        "visit_index": 0,
        "status": "completed",
        "artifact_revision": 1,
        "artifact_snapshot_digest": "…64 hex…",
        "artifact_digests": [{"uri": "mbse/model/requirements.sysml", "sha256": "…"}],
        "activities": [
          {"description": "Record requirements stage input artifacts",
           "performed_by": "xdesign simple-flow demo caller",
           "occurred_at": "2026-10-20T10:00:00Z",
           "artifact_uris": ["mbse/model/requirements.sysml"],
           "artifacts_changed": true,
           "artifact_digests": [{"uri": "mbse/model/requirements.sysml", "sha256": "…"}],
           "removed_artifact_uris": []}
        ],
        "assessments": [
          {"checkpoint_id": "simpleRequirementsAccepted",
           "evaluated_revision": 1,
           "evaluated_snapshot_digest": "…",
           "outcome": "passed",
           "judged_by": "script",
           "assessor": "simpleRequirementsCheck",
           "rationale": "All declared checker rules passed",
           "evidence_uris": ["cache/xdesign/wish-0011/reports/requirements_1_<uuid>.json"]}
        ]
      }
    ]
  },
  "history": [{"revision": 5, "operation_id": "…", "material_count": 24}],
  "events": [
    {"kind": "init", "stage_id": null, "visit_index": null,
     "title": "Wish initialized", "detail": "flow simple-flow",
     "at": null, "actor": "xdesign init", "outcome": null, "evidence_uris": []},
    {"kind": "activity", "stage_id": "requirements", "visit_index": 0,
     "title": "Record requirements stage input artifacts",
     "detail": "1 artifact", "at": "2026-10-20T10:00:00Z",
     "actor": "xdesign check caller", "outcome": null, "evidence_uris": []},
    {"kind": "assessment", "stage_id": "requirements", "visit_index": 0,
     "title": "simpleRequirementsCheck",
     "detail": "All declared checker rules passed", "at": null,
     "actor": "simpleRequirementsCheck", "outcome": "passed",
     "evidence_uris": ["cache/xdesign/wish-0011/reports/requirements_1_<uuid>.json"]},
    {"kind": "advance", "stage_id": "requirements", "visit_index": 0,
     "title": "Entered design", "detail": null, "at": null,
     "actor": "xdesign", "outcome": null, "evidence_uris": []}
  ]
}
```

`history` for store wishes uses the store revision history (newest last);
legacy wishes have `"history": []`. `events` are derived in visit order:
one `init` event, then per visit its activities (`at` = activity time), its
assessments in recorded order (actor is the assessor; the frozen plan identifies
the checker binding),
then an `advance` event when the visit status is `completed` and a next stage
exists (`"Entered <next stage>"`) or the run is `completed`
(`"Run completed"`). `detail` is a short human string and may be null.

`title` for a checkpoint: syntactic → checker id; semantic → checkpoint id.
`binding` is derived from the frozen plan checkpoint: no custom runner means
`{"mode":"default"}`, runner `process` means
`{"mode":"command","command_uri":<checker command_uri>}`. Historical runner
`cli` is projected read-only as `{"mode":"cli","cli":...,"model":...}`.
Only the fields relevant to the mode are emitted; clients must treat the others
as absent/null.

### 5.5 `GET /api/wishes/{wish_id}/work`

Runs `xdesign next WISH_ID` in the project root and returns:

```json
{
  "exit_code": 0,
  "response": {"schema_version": 1, "wish_id": "...", "run_id": "...",
               "flow_id": "simple-flow", "status": "active", "completed": false,
               "state_token": "...", "store_revision": 2,
               "work": {"stage_id": "design", "visit_index": 0,
                        "artifact_revision": 0, "artifact_snapshot_digest": "...",
                        "goal": "...", "instructions": "...", "inputs": [...],
                        "input_artifacts": [...], "expected_outputs": [...],
                        "frozen_output_paths": [...], "exit_checkpoints": [...],
                        "current_assessments": [...], "correction_reasons": [...],
                        "collection_diagnostics": [...], "candidate_artifacts": [...],
                        "candidate_snapshot_digest": "..."}},
  "stdout": "<raw JSON text>",
  "stderr": ""
}
```

When `response.completed` is true, the page shows the delivered state instead of
work. Exit code differs from 0 → `ok:false`, code `work_failed`.

### 5.6 `POST /api/wishes/{wish_id}/check`

Request:

```json
{
  "description": "Web check run for design",
  "performed_by": "xdesign web operator",
  "artifacts": [{"uri": "docs/design/flows/simple-flow/definition.md"}],
  "judgment": {
    "authority": "agent",
    "identity": "operator",
    "outcome": "passed",
    "rationale": "Reviewed the retained evidence set."
  }
}
```

- `description`, `performed_by` are optional non-empty strings; defaults are
  `"Web check, stage <stage>"` and `"xdesign web operator"`.
- `artifacts` entries need only `uri`; the server computes SHA-256 from the
  project file and rejects missing files with 400 before invoking the CLI.
- `judgment` is only meaningful for a semantic exit checkpoint; the server fills
  `artifact_snapshot_digest` from the `candidate_snapshot_digest` of the
  preceding `next` response. All supplied fields must be non-empty and
  `outcome` in `passed|failed|unevaluated`; `authority` in `agent|human`.
- Server sequence: run `next`; if `completed` return 409 `already_delivered`;
  write the result JSON to `cache/web/attempts/<wish_id>-<uuid>.json`; run
  `xdesign check WISH_ID --result FILE`; return:

```json
{
  "exit_code": 0,
  "response": {"assessed": {...}, "advance": "enter_next_stage", "...": "..."},
  "stdout": "<raw JSON text>",
  "stderr": ""
}
```

Exit code 1 means the stage remained blocked; it is still a valid response and
is returned as `ok:true` with `exit_code: 1` so the page can show correction
reasons. Other non-zero exit codes (2, engine error) → `ok:false`, code
`check_failed`, message from stderr, plus `stdout`/`stderr` in `error.detail`.

### 5.7 `POST /api/wishes`

Request:

```json
{
  "wish_id": "wish-web-0001",
  "title": "Add energy budget check",
  "description": "Introduce a deterministic energy budget gate.",
  "flow_id": "simple-flow",
  "checks": [
    {"checkpoint_id": "simpleRequirementsAccepted", "mode": "default"},
    {"checkpoint_id": "simpleDesignAccepted", "mode": "library",
     "library_id": "energy-budget-positive"},
    {"checkpoint_id": "simpleImplementationAccepted", "mode": "default"},
    {"checkpoint_id": "simpleRecordAccepted", "mode": "default"}
  ]
}
```

- Validation: id pattern above; title 1–200 chars; description ≤ 4000; flow in
  `simple-flow|wish-verify`; `checks` only allowed (and required for no reason to
  be complete) with `simple-flow`; each `checkpoint_id` must be one of the four
  syntactic simple-flow checkpoints plus `verificationAccepted` for semantic
  library bindings, no duplicates; `mode` `default` or `library`. Bindings not needed for a
  checkpoint default to `default`. `mode: "command"` is rejected here: custom
  command checkers are configured through `xdesign init --checks` only.
- Existing store item or legacy directory → 409 `already_exists`.
- Server writes the wish source and binding config:
  - `cache/web/wishes/<wish_id>.sysml`:
    ```
    // xdesign-wish: {"title":"...","description":"...","flow_id":"simple-flow"}
    package Wish_<id-with-nonalphanumeric-replaced-by-underscore> {
        doc /* <title>

    <description> */
    }
    ```
  - `cache/web/checks/<wish_id>.json` (only when at least one non-default
    binding): `{"schema_version":1,"checks":[{...as requested...}]}`
- Server runs `xdesign init <wish_id> --flow <flow> --wish <source>
  [--checks <config>]` in the project root and returns:

```json
{
  "wish_id": "wish-web-0001",
  "flow_id": "simple-flow",
  "init": {"schema_version": 1, "wish_id": "...", "store_revision": 1, "...": "..."},
  "stdout": "<raw JSON text>",
  "stderr": ""
}
```

Failure of `init` → `ok:false` code `init_failed` with stderr message.

### 5.8 `GET /api/wishes/{wish_id}/material?uri=<project-relative URI>`

Returns the raw material bytes of the requested head revision URI with
`Content-Type` by extension (`json`, `md`/`txt`, `sysml` → text, else
`application/octet-stream`), or 404 if the head revision has no such material.
Used to open frozen plans, reports, retained evidence, and the wish source.
Limit 512 KiB, 413 above that. For legacy wishes, allow URIs that appear in the
legacy run record evidence/artifacts or that start with `cache/xdesign/<id>/`.

### 5.9 Static assets

- `GET /` serves embedded `/index.html`.
- `GET /<asset>` serves an embedded asset with its generated content type.
- Unknown asset paths without an extension fall back to `/index.html` so the
  page's hash routing works; unknown asset-like paths return 404.

## 6. Custom check bindings (shared with the CLI feature)

The CLI accepts an optional check-binding configuration at initialization:

```
xdesign init WISH_ID --flow simple-flow --checks FILE
xdesign init WISH_ID --flow simple-flow --wish FILE --checks FILE
```

`FILE` is strict JSON:

```json
{
  "schema_version": 1,
  "checks": [
    {"checkpoint_id": "simpleRequirementsAccepted", "mode": "default"},
    {"checkpoint_id": "simpleDesignAccepted", "mode": "library",
     "library_id": "energy-budget-positive"},
    {"checkpoint_id": "simpleImplementationAccepted", "mode": "command",
     "command_uri": "tools/checks/custom_implementation.py"},
    {"checkpoint_id": "simpleRecordAccepted", "mode": "default"}
  ]
}
```

- `checkpoint_id`: one of the four syntactic simple-flow gates
  (`simpleRequirementsAccepted`, `simpleDesignAccepted`,
  `simpleImplementationAccepted`, `simpleRecordAccepted`).
- `mode: default`: run the flow's built-in deterministic checker. Bindings
  omitted from the file default to this mode.
- `mode: library`: use a project checkpoint definition whose syntactic executor
  is a script. A semantic library entry may bind `verificationAccepted`.
- `mode: command`: run a project-relative checker command that implements the
  existing checker interface (`--wish-model --record --checker-id --plan-id
  --plan-uri --plan-sha256 --stage --artifact-revision --snapshot-digest
  --report --list-inputs` with the same retained-report schema). It answers the
  same declared rule ids as the default checker.
- Non-default bindings are rejected for `wish-verify` in this slice; the
  two-stage flow keeps its semantic judgments only.
- Default bindings freeze the plan bytes exactly as before this feature. A run
  with a custom binding freezes the binding into its plan, so
  `xdesign next`, `xdesign check` and the web page can reconstruct the choice
  from the store without a separate config file.
- New CLI executor bindings are rejected. A frozen historical CLI binding remains
  visible in detail responses but its checker fails closed without launching an agent.

## 7. Frontend direction (visual and interaction)

- Product name in the page: **xdesign · WishFlow Studio**.
- Dark engineering-console aesthetic: deep navy canvas, layered radial
  gradients, fine grid texture, glass cards, monospace identifiers, status
  palette: delivered `#34d399`, active `#22d3ee`, blocked `#fb7185`,
  received `#94a3b8`, warning `#fbbf24`, accent violet `#a78bfa`.
- Layout: fixed left rail (brand, Overview, Wishes, New wish, Flows; store
  status at the bottom), sticky top bar (breadcrumb, project path, refresh,
  "New wish" primary button), content column max-width 1280 px.
- Views (hash routing: `#/overview`, `#/wishes`, `#/new`, `#/wishes/<id>`,
  `#/flows`):
  1. **Overview**: hero row (title + short explanation), four stat cards
     (total, active, blocked, delivered), the flow pipeline visualization,
     recent wishes and the local environment.
  2. **Wishes**: filter chips (all / active / blocked / delivered / received,
     flow filter), searchable table/cards with a mini stage pipeline, flow chip,
     status chip, head revision, age; row click opens the wish.
  3. **New wish wizard**: steps Identity → Flow → Checks → Review with a stepper;
     flow cards (SimpleFlow four stages/five gates, WishVerifyFlow two judged
     stages); for SimpleFlow each of the four check nodes gets a card with the
     checker name, rule count, built-in script / library checkpoint control,
     library selector and validation inline; successful
     creation navigates to the new wish.
  4. **Wish detail**: header with status, flow, head revision, actions; lifecycle
     board (stages as connected nodes with the current stage highlighted and
     check nodes as diamonds underneath, coloured by latest assessment outcome);
     stage cards with per-checkpoint assessment rows (outcome chip, rationale,
     judged-by, evidence material links opening a viewer modal); vertical event
     timeline; side panel with current work, "Get next work", and "Run check"
     forms (description, artifact URI lines, semantic judgment fields when the
     stage has semantic gates); material modal renders JSON pretty-printed.
  5. **Flows**: flow catalog cards showing stages, checkpoints, checker rules,
     and default command URIs.
- Interaction details: polling refresh every 4 s on overview/list/detail; manual
  refresh button; toasts for actions; skeleton loading; empty states with a
  short invitation to create the first wish; `Esc` closes modals; responsive
  down to 980 px (rail collapses to icons) and 720 px (single column).
- No external requests of any kind; all assets local; system font stack with
  `ui-monospace` for IDs; respect `prefers-reduced-motion`.
- A stable module-level `window.XDESIGN_WEB` debug flag advertises the build so
  the acceptance screenshot can assert the page loaded (`{version, ready,
  sample}`).
- Empty-versus-sample rule: sample data is used only when the API is
  unreachable or when `?demo=1` is requested, and in that mode write actions are
  disabled and the state is labelled. A reachable store with no wishes is a
  real empty state: the page shows the empty-state call to action and the New
  wish wizard is fully enabled, so the first wish can be created from the UI.

## 8. Ownership and acceptance

- Frontend owns `web/assets/**`; backend owns `web/server/**` (except
  `build.rs`, which the Lead owns); the Lead owns `web/SPEC.md`, `README.md`,
  `src/lib.rs`, `src/main.rs`, `Cargo.toml`, and integration.
- Acceptance: `cargo build`, `cargo test`, real `xdesign web` serving the
  embedded page, API smoke tests for health/flows/wishes/create/work/check,
  screenshots of overview / wizard / detail at 1600×1000, and no runtime
  dependency beyond the shipped binary.

## 9. Round 2: checkpoint library, rule authoring, and start-work handoff

This section extends the contract after operator feedback. Where it conflicts with an
earlier section, this section wins; otherwise the earlier rules still apply.

### 9.1 Why this round exists

Operator feedback on the first web delivery:

1. The check-node control row must stay aligned when the selected binding changes.
2. The check executor choice must be distinct from the external stage worker.
3. Development produces many syntactic and semantic check rules; most repeat. The product
   needs a reusable **checkpoint library** that records the tools/rules and offers them in
   flows, with an explicit statement of whether a checkpoint is a syntactic rule set or a
   semantic rule set, and for syntactic ones the ability to reuse an existing script or
   generate a new script.
4. Saving a flow or creating a wish does not launch an agent; the product must make the
   start-work handoff explicit instead of implying automation.

### 9.2 Checkpoint library: storage and model

The library is project-level, Git-tracked authoring source (not operational state):

```
checkpoints/
  README.md                     short usage note
  library.json                  user-maintained entries
  scripts/<slug>.py             scripts generated by the scaffold action
```

`checkpoints/library.json`:

```json
{
  "schema_version": 1,
  "checkpoints": [
    {
      "checkpoint_id": "energy-budget-positive",
      "title": "Energy budget stays non-negative",
      "description": "Deterministic check that no declared energy budget is negative and every actuator declares one.",
      "kind": "syntactic",
      "tags": ["energy", "design"],
      "rules": [
        {"rule_id": "budget-declared", "condition": "Every actuator declares a numeric energyBudget.",
         "reads": ["mbse/model/design.sysml"]},
        {"rule_id": "budget-non-negative", "condition": "Every declared energyBudget is >= 0.",
         "reads": ["mbse/model/design.sysml"]}
      ],
      "inputs": ["mbse/model/design.sysml"],
      "executor": {"mode": "script", "command_uri": "checkpoints/scripts/energy_budget_positive.py"},
      "checker_id": "energyBudgetCheck",
      "version": 1,
      "created_at": "2026-10-26T00:00:00Z",
      "updated_at": "2026-10-26T00:00:00Z",
      "source": "generated",
      "history": []
    },
    {
      "checkpoint_id": "verification-evidence-complete",
      "title": "Verification evidence answers every requirement",
      "description": "Semantic checkpoint: a permitted judge must be able to re-examine the retained evidence and see each requirement answered.",
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

Rules:

- `checkpoint_id` is a slug `[a-z0-9][a-z0-9_-]{0,63}` and unique among user entries;
  built-ins below are reserved.
- `kind` is `syntactic` or `semantic` and required. A syntactic entry must carry at least
  one rule, at least one input, and an executor. A semantic entry must carry at least one
  criterion, a `judges` object with at least one of `agent`/`human` true, and no rules,
  inputs, or executor. `title` 1–120 chars, `description` 1–1000 chars, `tags` ≤ 8 slugs.
- `inputs` are normalized project-relative files that exist and are frozen into work at
  initialization (they replace the `--list-inputs` invocation for that gate). `rules[].reads`
  uses the same vocabulary as checker reads.
- `checker_id` (optional, syntactic only) names the identity the checker script reports
  when it is reused: the shipped `tools/checks/*.py` scripts reject an invocation whose
  `--checker-id` differs from their hardcoded id (for example `simpleRequirementsCheck`).
  When set, the frozen plan's `checker.checker_id` is this value, so the assessment
  assessor is the script identity while `checkpoint.library` keeps the library provenance.
  When absent it defaults to `checkpoint_id`. Generated scaffold scripts echo whatever
  `--checker-id` the runtime passes, so a default is fine for them. The value matches
  `[A-Za-z0-9][A-Za-z0-9_-]{0,127}`.
- Syntactic executor: `{"mode":"script","command_uri":<project-relative .py>}`.
  Existing CLI executor entries remain readable, but cannot be saved or bound
  to a new run.
- `source` is `builtin`, `generated`, or `manual`.
- `version` increments on each successful update of a user entry; `history` keeps the
  previous snapshot objects (at most 10, newest first) in the library file itself. The
  `history` item shape is `{version, updated_at, description, kind, rules, inputs,
  executor, criteria, judges, tags}`.
- Built-in entries are provided by the binary and shown in every listing with
  `"builtin": true`; they cannot be saved, overwritten, or deleted through the API. Seed
  built-ins: the four SimpleFlow syntactic checkers (`builtin-simple-requirements`,
  `builtin-simple-design`, `builtin-simple-implementation`, `builtin-simple-record`, rules
  and command URIs copied from the default plan) and one semantic SimpleFlow gate entry
  (`builtin-simple-verification`, criteria copied from `verificationAccepted`).
- `checkpoints/library.json` missing is not an error: the effective library is the built-ins
  plus zero user entries. Malformed JSON or an invalid entry is an explicit error; the
  product never guesses.

Scaffold generation: a create request may set `create_script: "scaffold"` only together
with a syntactic script executor whose `command_uri` is `checkpoints/scripts/<id-with-_.py>`.
The runtime writes a deterministic, self-contained Python checker at that URI that:

- supports `--list-inputs` (prints the declared inputs as JSON) and the full checker
  invocation flag set used by the runtime (`--wish-model --record --checker-id --plan-id
  --plan-uri --plan-sha256 --stage --artifact-revision --snapshot-digest --report`);
- verifies the record/plan/visit binding like the shipped checkers, writes a raw report
  with `checker_id`, `stage`, `artifact_revision`, `snapshot_digest`, `report_uri`,
  `overall_verdict`, `rules[]` (`rule_id`, `verdict`, `inputs`, `observed`, `failures`) and
  `input_sha256` covering the wish model, record snapshot, plan, declared inputs, and its
  own script bytes;
- ships one TODO rule function per declared rule returning a failed verdict with
  `"rule not implemented: <rule_id>"` so the scaffold is honest and safe until the author
  edits it; the file includes the rule condition/reads as comments.
An existing script must be reused through `executor.mode == "script"` without
`create_script`; generated scripts are Git-tracked like any other source file.

CLI surface (all commands print exactly one compact JSON object on stdout and nothing on
stderr; exit 0 success, 2 invalid request/unknown/built-in mutation, 1 internal failure.
A failure object is `{"schema_version":1,"ok":false,"error":{"code":"<stable code>",
"message":"..."}}`; success objects carry `schema_version` and the payload and never an
`ok` member):

```
xdesign checkpoint list
xdesign checkpoint show CHECKPOINT_ID
xdesign checkpoint save --request FILE
```

`list` reply:

```json
{"schema_version":1,"library_uri":"checkpoints/library.json",
 "issues":[],
 "checkpoints":[{"checkpoint_id":"...","title":"...","kind":"syntactic",
   "builtin":false,"version":2,"updated_at":"...","tags":["..."],
   "executor":{"mode":"script","command_uri":"..."},
   "rule_count":2,"criteria_count":0}]}
```

`show` reply: `{"schema_version":1,"checkpoint":{...full entry...,"builtin":false,"script_exists":true}}`.

`save` request: the full entry fields plus optional `"create_script": "scaffold"`; a save
with an existing user `checkpoint_id` is an update (version + 1, previous snapshot pushed
into history); saving a built-in id or a new entry with a duplicate built-in id is an
error. Reply: `{"schema_version":1,"checkpoint":{...},"script_uri":"..."|null,
"created":true|false,"version":2}`.

### 9.3 Runtime binding to library checkpoints

`xdesign init --checks FILE` accepts a fourth binding mode for the four syntactic gates
and the SimpleFlow semantic gate `verificationAccepted`:

```json
{"schema_version":1,"checks":[
  {"checkpoint_id":"simpleRequirementsAccepted","mode":"library",
   "library_id":"energy-budget-positive"},
  {"checkpoint_id":"simpleDesignAccepted","mode":"default"},
  {"checkpoint_id":"simpleImplementationAccepted","mode":"default"},
  {"checkpoint_id":"simpleRecordAccepted","mode":"default"},
  {"checkpoint_id":"verificationAccepted","mode":"library",
   "library_id":"verification-evidence-complete"}
]}
```

Resolution at `init` (the project library is read from the project root):

- A syntactic gate requires a syntactic entry. The frozen plan records
  `checkpoint.library = <checkpoint_id>`; the gate's checker becomes the library entry:
  `checker_id = entry.checker_id` (or `<checkpoint_id>` when absent), `rules = entry.rules`,
  `command_uri = entry.executor.command_uri`, and `custom.runner = "process"`.
  The work definition freezes `entry.inputs` for that stage instead of invoking
  `--list-inputs`.
- The semantic gate requires a semantic entry: `checkpoint.library = <checkpoint_id>`,
  `criteria = entry.criteria`, `agent_may_judge`/`human_may_judge` from `entry.judges`;
  the gate keeps `checker: null`.
- Kind mismatch, missing entry, unknown id, or library-mode binding on an unsupported gate
  is an initialization error.
- Everything frozen is durable: later library edits never change an existing run. The
  default plan bytes and all non-library runs stay exactly as before.
- `wish-verify` keeps its two semantic gates and has no library binding in this round.

Assessment semantics follow the existing rules: a script executor records script authority
named by the frozen checker id; semantic library gates are judged by the caller's
permitted judgment (agent/human) as today. Historical CLI assessments remain readable.

### 9.4 HTTP API additions

- `GET /api/checkpoints` → the `checkpoint list` reply passed through as `data`.
- `GET /api/checkpoints/{checkpoint_id}` → the `checkpoint show` reply as `data`; unknown
  id → 404.
- `POST /api/checkpoints` → body is the `checkpoint save` request object (full entry
  fields plus optional `create_script`). The server writes it to
  `cache/web/checkpoint-requests/<uuid>.json` and runs `xdesign checkpoint save --request
  <file>`; success returns the save reply as `data`; invalid definitions map to 400
  `invalid_checkpoint`, built-in mutation to 409 `builtin_checkpoint`, other CLI failures
  to 500 `checkpoint_failed` with `error.detail` carrying exit code/stdout/stderr.
- `POST /api/wishes` accepts `{"checkpoint_id":..., "mode":"library", "library_id":...}`
  entries for the four syntactic gates and the semantic gate as in 9.3. The server passes
  them through unchanged; `init` performs the authoritative validation.
- `GET /api/wishes/{id}`: plan checkpoint objects gain `library_id` (string or null) and
  the derived `binding` gains `{"mode":"library","library_id":...}`; the detail response
  also includes `library` next to each checkpoint when the id can be resolved in the
  current project library: `{"checkpoint_id","title","kind","version","builtin"}`; null
  when the library entry was removed or the library is unreadable (a `warnings` array
  names the missing ids). Frontend must match on the frozen plan fields first and use the
  summary only for display.
- No new work/check/material endpoint is required: the library modifies plan binding and
  display only. `GET /api/wishes/{id}/material` continues to serve the frozen plan and
  retained reports.

### 9.5 Start-work handoff (explicit, not automatic)

Creating a wish (including freezing a flow and its library bindings) never starts an
agent. The run waits at its first stage. The only way work starts is a caller handing the
current work order to a working agent and submitting the stage result:

```sh
xdesign next WISH_ID      # the work order: stage, goal, instructions, expected outputs,
                          # frozen output paths, current assessments, correction reasons
xdesign check WISH_ID --result FILE  # submit stage outputs; evaluates gates and advances
```

- The web program is an operator console: it can run `next` and `check` for the operator
  and display the resulting work order/assessment, but it never launches stage work.
- The wish detail page must present a "Start work" handoff card for non-delivered wishes
  containing: a one-line explanation that the flow does not auto-run, a copyable agent
  handoff prompt built from the detail (wish id/title/stage/goal/instructions, the
  commands to run, and the note that the resulting `result.json` must include the current
  `state_token`, batch fields and a judgment when the stage has a semantic gate), and
  copyable `xdesign next`/`xdesign check` commands. The prompt may be composed client-side
  from `GET /api/wishes/{id}` and the work response; no new endpoint is required.
- After a successful `check` the page refreshes the detail (current stage, assessments,
  events) and shows the next handoff state. A blocked check shows correction reasons and
  keeps the same handoff with the corrections included.

### 9.6 UI corrections and additions

1. **Check-node control alignment.** The check-card control row is a stable grid, not a
   flex row whose track widths depend on the selected mode. In all four cards the
   executor control column and the dependent library columns
   selector) use the same tracks and vertical baseline; no control may overlap another.
   The row must stay aligned when switching Built-in script → Library and at 980/720
   responsive widths.
2. **Unambiguous executor language.** The option group is labelled "Check executor" with
   options `Built-in script` and `Library checkpoint` (or `Library`).
   Every check node explains that stage work is performed by the working agent regardless
   of this choice, and the choice selects only who verifies the checkpoint:
   - Built-in script: the repository's deterministic checker, recorded with script
     authority.
   - Library checkpoint: the frozen library definition's rules/criteria are used, with
     the executor the library entry declares.
   Chips and detail rows use the same vocabulary; an assessment shows judged-by
   (script/agent/human) and the assessor for retained assessments.
3. **Checkpoints view** (`#/checkpoints`): watch/reuse/filter the library. Cards/rows show
   title, id, kind chip (syntactic/semantic), source (built-in/user), executor summary,
   rule/criteria count, version, updated time, tags. A detail drawer shows the full
   description, rule table (rule id/condition/reads), inputs, executor, criteria/judges,
   and the version history. A "New checkpoint" form selects kind explicitly, requires a
   description, edits rules/criteria, inputs and script executor (reuse script path or
   generate scaffold script), and surfaces backend validation errors inline.
   Built-in entries are read-only and are marked as templates from which a new user entry
   can be pre-filled/duplicated.
4. **Wizard library binding.** Each syntactic check card gets the `Library` mode with a
   selector of syntactic library entries (title, id, rule count, executor) and a compact
   preview of the selected entry's rules. The SimpleFlow semantic gate gets a separate
   small card with `Default criteria` vs a semantic library checkpoint selector. The
   review step lists library bindings with the entry title/kind.
5. **Wish detail**: lifecycle check nodes show the binding (built-in script or
   library `<id>` + title), the frozen rules/criteria of a library gate,
   and the assessment authority. The "Start work" handoff card from 9.5 sits above the
   work/check actions; Get next work and Run check stay available for the operator.
6. **Flows view** lists library availability per flow (`custom_checks_supported`, library
   entries usable for syntactic and semantic gates).
7. Sample mode and empty/sample rules from section 7 remain unchanged; the checkpoint
   view must handle an unreachable API with the same labelled sample/offline behavior.

### 9.7 Acceptance for this round

- `cargo test` green; library model, CLI, plan binding, work freezing and restore
  paths have unit and CLI-level tests using temporary project roots and script stubs.
- Lead acceptance: `xdesign checkpoint list/show/save` round-trip; `init --checks` with a
  syntactic library binding and a semantic library binding; frozen plan carries
  `library`/rules/criteria; check executes the bound script and records the expected
  authority; default plan bytes unchanged; CLI rejects kind mismatch, missing library,
  builtin mutation, and bad entries.
- Web acceptance: `/api/checkpoints` list/show/save, create wish with library bindings,
  detail join and warnings, UI layout fix screenshot, library view/create screenshots,
  wizard library binding screenshot, handoff card screenshot; zero console errors and
  zero external requests as before.
- Docs: checkpoint library design + interface updates, stage CLI/library mode,
  start-work handoff semantics, verification cases; docs audit and build clean.

Script executor objects also accept optional `files` (additional normalized project-relative helper/data paths) and `python` (a project-relative environment executable under `cache/`). The script editor exposes and preserves both values. New runs freeze and retain executor bundles; see `docs/design/checker-execution.md#d111`. Built-in availability refers to embedded resources and does not require a project `tools/` file.

The planned worktree-location extension is owned by [I21](../docs/interfaces/worktree-store.md#i21) and [D117](../docs/design/worktree-store-migration.md#d117). It requires explicit persistent root propagation to lifecycle children, legacy-only sessions that bypass unrelated enrollment, location diagnostics and read-only legacy/store collision inspection; these are designed-only additions to this implemented interface.
