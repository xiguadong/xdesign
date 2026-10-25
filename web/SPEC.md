# xdesign web contract (working specification)

This file is the implementation contract between the web server module
(`web/server/`) and the operator page (`web/assets/`). It is a working
specification for the feature branch; the published design lives under `docs/`.

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
    agents.rs             CLI/model catalog and availability detection
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
  "agents": [
    {"cli": "codex", "available": true, "version": "codex-cli 0.162.1"},
    {"cli": "kimi", "available": true, "version": "2.2.0"}
  ],
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
            "modes": ["default", "cli"]
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
  ],
  "agents": [
    {
      "cli": "codex",
      "label": "Codex CLI",
      "command": "codex",
      "available": true,
      "version": "codex-cli 0.162.1",
      "default_model": "gpt-6-astra",
      "models": [
        {"id": "gpt-6-astra", "label": "GPT-6 Astra"},
        {"id": "gpt-5.6-terra", "label": "GPT-5.6 Terra"},
        {"id": "gpt-5.1-codex", "label": "GPT-5.1 Codex"},
        {"id": "gpt-5-codex", "label": "GPT-5 Codex"},
        {"id": "o4-mini", "label": "o4-mini"}
      ]
    },
    {
      "cli": "kimi",
      "label": "Kimi Code CLI",
      "command": "kimi",
      "available": true,
      "version": "2.2.0",
      "default_model": "kimi-code/k3.1-pro",
      "models": [
        {"id": "kimi-code/k3.1-pro", "label": "K3.1 Pro"},
        {"id": "kimi-code/k3.1-pro-highspeed", "label": "K3.1 Pro Highspeed"},
        {"id": "kimi-code/k3", "label": "K3"},
        {"id": "kimi-code/k3-256k", "label": "K3 256k"},
        {"id": "kimi-code/kimi-for-coding", "label": "K2.8 Preview"},
        {"id": "kimi-code/kimi-for-coding-highspeed", "label": "K2.7 Code Highspeed"}
      ]
    }
  ]
}
```

The catalog is static display metadata plus runtime detection. Detection runs
`<command> --version` with a 3 s deadline and reports the first non-empty line.
A detected default model from the user's CLI configuration replaces/precedes the
static default when found (`~/.codex/config.toml` `model =` line,
`~/.kimi-code/config.toml` `default_model =` line; best effort, missing files are
not an error). Unknown extra models are allowed in creation requests; the page
offers the listed ones plus a free-text value.

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
          "binding": {"mode": "default", "cli": null, "model": null, "command_uri": null}
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
assessments in recorded order (actor is the assessor; for syntactic checker
bindings the plan binding tells whether the executor was default or CLI),
then an `advance` event when the visit status is `completed` and a next stage
exists (`"Entered <next stage>"`) or the run is `completed`
(`"Run completed"`). `detail` is a short human string and may be null.

`title` for a checkpoint: syntactic → checker id; semantic → checkpoint id.
`binding` is derived from the frozen plan checkpoint: no custom runner means
`{"mode":"default"}`, runner `cli` means `{"mode":"cli","cli":...,"model":...}`,
runner `process` means `{"mode":"command","command_uri":<checker command_uri>}`.
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
    {"checkpoint_id": "simpleDesignAccepted", "mode": "cli",
     "cli": "codex", "model": "gpt-6-astra"},
    {"checkpoint_id": "simpleImplementationAccepted", "mode": "default"},
    {"checkpoint_id": "simpleRecordAccepted", "mode": "default"}
  ]
}
```

- Validation: id pattern above; title 1–200 chars; description ≤ 4000; flow in
  `simple-flow|wish-verify`; `checks` only allowed (and required for no reason to
  be complete) with `simple-flow`; each `checkpoint_id` must be one of the four
  syntactic simple-flow checkpoints, no duplicates; `mode` `default` or `cli`;
  `cli` in `codex|kimi`; `model` non-empty ≤ 200. Bindings not needed for a
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
    {"checkpoint_id": "simpleDesignAccepted", "mode": "cli",
     "cli": "codex", "model": "gpt-6-astra"},
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
- `mode: cli`: execute the checkpoint with an agent CLI. `cli` is `codex` or
  `kimi`; `model` is any non-empty model identifier the CLI accepts. The agent
  answers exactly the checkpoint's declared checker rules and its verdict is
  retained as an `agent`-authority assessment whose assessor is
  `<cli>:<model>`.
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
- Agent CLI execution environment:
  - `codex`: `codex exec --skip-git-repo-check --sandbox read-only -m <model> <prompt>`
    (the prompt is the final positional argument; it is a single bounded argv entry,
    never shell-interpreted).
  - `kimi`: `kimi -m <model> -p <prompt>`.
  - Programs can be overridden for development and tests with
    `XDESIGN_AGENT_CODEX_BIN` / `XDESIGN_AGENT_KIMI_BIN` (path to an
    executable). Tests use stub executables and never call a real agent.
  - The whole invocation is bounded by `XDESIGN_CHECKER_TIMEOUT_SECS`
    (default 600 s), the same deadline as process checkers.
  - The prompt instructs a read-only review and requires one JSON verdict
    object; the runtime validates the reported rule set, writes the retained
    report plus `.stdout.log`/`.stderr.log`, and records the assessment.
- Agent runs are non-deterministic. The page and CLI surface the executor in the
  binding and the assessment authority/assessor, so a custom run is never
  presented as a deterministic default check.

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
     recent wishes, agent availability.
  2. **Wishes**: filter chips (all / active / blocked / delivered / received,
     flow filter), searchable table/cards with a mini stage pipeline, flow chip,
     status chip, head revision, age; row click opens the wish.
  3. **New wish wizard**: steps Identity → Flow → Checks → Review with a stepper;
     flow cards (SimpleFlow four stages/five gates, WishVerifyFlow two judged
     stages); for SimpleFlow each of the four check nodes gets a card with the
     checker name, rule count, live "default / agent CLI" segmented control, CLI
     selector (codex/kimi with availability badge), model selector from
     `/api/flows`, and a free-text model override; validation inline; successful
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
