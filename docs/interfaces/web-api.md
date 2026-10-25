# Operator web API

> Interface: local operator web API | Transport: loopback HTTP/1.1 with JSON | Status:
> draft branch-local contract; no frozen interface baseline and no remote-deployment
> claim.

This document owns the HTTP contract between the embedded operator page and the
`xdesign web` server: routes, envelopes, status codes, request and response fields, and
the lifecycle actions that drive the stage CLI. The command line and server options are in
[the stage CLI](../design/stage-cli.md#d90); the program design, read/write boundaries and
embedded-asset release model are in [the operator web design](../design/web-operator.md#d91).
The API is local and read/write only through the CLI; it is not the documentation site's
interface and does not serve or read the docs tree.

<a id="i16"></a>

## 1. Transport, envelopes, status codes, and errors

The server is single-process and operator-local. HTTP/1.1 connections carry one request
and close (`Connection: close`); `GET` and `POST` are the only accepted methods and any
other method returns `405`. Requests and responses are UTF-8 JSON unless a static asset or
raw material is served.

The JSON envelope has exactly one of two shapes. Success carries `data`; failure carries
`error` with a stable code and a human-readable message:

```json
{"ok": true, "data": {}}
{"ok": false, "error": {"code": "<stable_code>", "message": "<human text>"}}
```

| HTTP status | Meaning | Envelope |
| --- | --- | --- |
| 200 | Success, including a blocked stage check that returned the CLI's blocked response | `ok:true` |
| 400 | Invalid request: bad JSON, missing or invalid field, unknown artifact file, or invalid judgment/checks payload | `ok:false` |
| 404 | Unknown route, unknown wish, requested material not present in the revision, or material not retained | `ok:false` |
| 405 | Method other than `GET`/`POST` on a known route | `ok:false` |
| 409 | State conflict: the wish already exists or is already delivered | `ok:false` |
| 413 | Request body or requested material above its size limit | `ok:false` |
| 500 | A lifecycle command failed, or an unexpected server condition | `ok:false` |

The stable error codes are the handler codes in the tables below: `invalid_json`,
`invalid_id`, `invalid_title`, `invalid_description`, `invalid_flow`, `invalid_checks`,
`invalid_artifact`, `invalid_judgment`, `missing_uri`, and `invalid_uri` for requests;
`not_found`, `unknown_wish`, and `material_not_found` for missing routes, wishes, and
revisions; `method_not_allowed`; `already_exists` and `already_delivered` for conflicts;
`payload_too_large` and `material_too_large`; and `work_failed`, `check_failed`,
`init_failed`, and `internal_error` for server-side failures. The message is human text;
callers branch on the code. Failed lifecycle actions may also carry an `error.detail`
object with the command's `exit_code`, `stdout`, and `stderr`.

The response always carries an explicit `Content-Length`. JSON responses use
`Content-Type: application/json; charset=utf-8`, forbid caching (`Cache-Control:
no-store`), and set `X-Content-Type-Options: nosniff`. Request bodies are limited to
1 MiB and reads time out after 10 seconds. `wish_id` path segments match
`[A-Za-z0-9][A-Za-z0-9_-]{0,127}` and are treated as read-only identifiers; an unknown
path under `/api/` returns the JSON `404`, not the page.

<a id="i17"></a>

## 2. Read endpoints and field tables

All payloads below are the `data` member of the success envelope.

| Method and path | Purpose |
| --- | --- |
| `GET /api/health` | Build version, project and store roots, store presence, agent availability, server time |
| `GET /api/flows` | Flow catalog with stages, checkpoints, checker rules, supported binding modes, and the agent catalog |
| `GET /api/wishes` | Wish list with flow, status, current stage, head revision, and source |
| `GET /api/wishes/{wish_id}` | Wish detail: plan with frozen bindings, run visits and assessments, revision history, and derived events |
| `GET /api/wishes/{wish_id}/work` | Current work order from `xdesign next` |
| `GET /api/wishes/{wish_id}/material?uri=URI` | Raw frozen material bytes of the requested head-revision URI |
| `GET /` and `GET /<asset>` | Embedded page and static assets |

### 2.1 `GET /api/health`

| Field | Type | Meaning |
| --- | --- | --- |
| `version` | string | `xdesign` crate version |
| `project_root` | string | Absolute project root the server changed into |
| `store_root` | string | Absolute store root in use |
| `store_present` | boolean | Whether the store exists at `store_root` |
| `agents` | array | `{cli, available, version}` entries from runtime detection |
| `now` | string | Server time, RFC 3339 |

### 2.2 `GET /api/flows`

`flows` is an array of `flow_id`, `title`, `summary`, `custom_checks_supported`, `stages`,
and `checkpoints`. A stage has `stage_id`, `title`, `goal`, `instructions`, and
`expected_outputs`. A checkpoint has `checkpoint_id`, `stage_id`, `kind`
(`syntactic`/`semantic`), `title`, `criteria`, `agent_may_judge`, `human_may_judge`, and,
for syntactic checkpoints, a `checker` object:

| Checker field | Meaning |
| --- | --- |
| `checker_id` | Declared checker id |
| `command_uri` | Built-in command path |
| `establishes` | Conformance kinds the checker states |
| `rules` | Array of `{rule_id, condition, reads}` |
| `modes` | Binding modes the page can offer: `["default", "cli"]` on a syntactic checker; a semantic checkpoint has `checker: null` |

`custom_checks_supported` is `true` only for `simple-flow`, the flow whose four syntactic
checkpoints can be bound ([D88](../design/flows/simple-flow/definition.md#d88)).

`agents` follows the static codex/kimi catalog plus runtime detection: each entry has
`cli`, `label`, `command`, `available`, `version`, `default_model`, and `models` as
`{id, label}` pairs. Detection runs `<command> --version` with a three-second deadline and
reports the first non-empty line; a detected default model from the user's CLI
configuration replaces or precedes the static default. Unknown extra models are accepted
in create requests, so the page offers the listed values plus a free-text model.

### 2.3 `GET /api/wishes`

`wishes` is an array sorted by status priority (`blocked`, `active`, `received`,
`delivered`) and then wish id. Each entry has:

| Field | Meaning |
| --- | --- |
| `wish_id` | Item identifier |
| `title`, `description` | Display text; from the wish source marker, the first SysML description, or the wish id |
| `flow_id` | `simple-flow` or `wish-verify` |
| `status` | `received`, `active`, `blocked`, or `delivered` |
| `current_stage` | Current stage id, or `null` when completed |
| `head_revision` | Store head revision; `null` for a legacy registration |
| `source` | `store` for store items, `legacy` for cache registrations |
| `updated_at` | Currently always `null`; the field is reserved for a later source |

### 2.4 `GET /api/wishes/{wish_id}`

An unknown id is `404 unknown_wish`. The detail payload repeats the list fields and adds
`plan`, `run`, `history`, and `events`. `plan` is the frozen plan: `plan_id`, `plan_sha256`, `stages`, and
`checkpoints`. A plan stage has `stage_id`, `next_stage_ids`, `exit_checkpoint_ids`, and
`completes_flow`. A plan checkpoint has `checkpoint_id`, `stage_id`, `kind`, `title`,
`criteria`, `agent_may_judge`, `human_may_judge`, and `checker`, whose `binding` is
derived from the frozen plan:

| `binding.mode` | Other binding fields | Meaning |
| --- | --- | --- |
| `default` | none (the object is just `{"mode": "default"}`) | Built-in deterministic command |
| `cli` | `cli` and `model` | Agent CLI executor with the named model |
| `command` | `command_uri` (the frozen checker's command path) | Custom project-relative checker command |

A syntactic checkpoint's `title` is its checker id; a semantic checkpoint's title is the
checkpoint id ([D88](../design/flows/simple-flow/definition.md#d88)).

`run` is the stored run projection: `run_id`, `execution_type`, `plan_id`, `status`,
`current_visit`, and `visits`. A visit has `stage_id`, `visit_index`, `status`,
`artifact_revision`, `artifact_snapshot_digest`, `artifact_digests`, `activities`, and
`assessments`. An activity has `description`, `performed_by`, `occurred_at`,
`artifact_uris`, `artifacts_changed`, `artifact_digests`, and `removed_artifact_uris`. An
assessment has `checkpoint_id`, `evaluated_revision`, `evaluated_snapshot_digest`,
`outcome`, `judged_by`, `assessor`, `rationale`, and `evidence_uris`; the executor chosen
by the plan binding is visible in `judged_by`/`assessor`, so a custom run is not presented
as a default check.

`history` is the store revision history, newest last, as
`{revision, operation_id, material_count}`; a legacy wish has an empty history. `events`
is derived in visit order: one `init` event, then each visit's activities (their
`occurred_at` time), its assessments in recorded order (`actor` is the assessor), and an
`advance` event when the visit is completed and a next stage exists (`Entered <stage>`) or
the run is completed (`Run completed`). Each event has `kind`, `stage_id`, `visit_index`,
`title`, `detail`, `at`, `actor`, `outcome`, and `evidence_uris`; `detail` may be `null`.

### 2.5 `GET /api/wishes/{wish_id}/work`

The server runs `xdesign next WISH_ID` in the project root and returns
`exit_code`, `response` (the parsed CLI response), `stdout`, and `stderr`. The response
carries `schema_version`, `wish_id`, `run_id`, `flow_id`, `status`, `completed`,
`state_token`, `store_revision`, and `work`; `work` is `null` when the run is completed
and otherwise names the same fields as the [stage CLI exchange](../design/stage-cli.md#d85):
stage, visit, revision, snapshot digest, goal, instructions, inputs, outputs, exit
checkpoints, current assessments, correction reasons, collection diagnostics, and
candidate digests. A non-zero exit code is an HTTP 500 error with code `work_failed` and
the command's streams in `error.detail`.

### 2.6 `GET /api/wishes/{wish_id}/material?uri=URI`

Returns the raw bytes of a head-revision material at the project-relative `uri`, with
`Content-Type` chosen by extension (`json`, `md`/`txt`, and `sysml` as text, otherwise
`application/octet-stream`). A missing query parameter is `400 missing_uri` and a
non-normalized path is `400 invalid_uri`. A URI absent from the head revision is
`404 material_not_found`; a body above 512 KiB is `413 material_too_large`. For a legacy wish the allowlist is the legacy run record's
evidence/artifact URIs and the paths beginning `cache/xdesign/<wish_id>/`. The route never
serves an arbitrary filesystem path.

### 2.7 Static assets

`GET /` serves the embedded `index.html`. `GET /<asset>` serves an embedded asset with its
generated content type. An unknown extensionless path falls back to `index.html` so hash
routing works; an unknown asset-like path returns `404`. The assets are compiled into the
binary ([D94](../design/web-operator.md#d94)); no source-tree or documentation-site file
is read to serve them.

<a id="i18"></a>

## 3. Lifecycle actions, create fields, and write boundary

Two routes write, and both do so only by running the same executable's stage CLI. The
blocked/completed semantics and exit codes are the CLI's; the API does not re-evaluate a
gate.

| Action | Server sequence | Durable effect |
| --- | --- | --- |
| `POST /api/wishes` | Validate the request; write the wish source and optional binding file under `cache/web/`; run `xdesign init WISH_ID --flow FLOW --wish SOURCE [--checks FILE]` | The CLI's registration revision |
| `POST /api/wishes/{wish_id}/check` | Run `next`; if completed return `409 already_delivered`; write the result JSON under `cache/web/attempts/`; run `xdesign check WISH_ID --result FILE` | The CLI's stage revision, or its recorded blocked result |

Any other route is read-only. The server holds no writer path of its own: it starts no
second writer, bypasses no expected-head guard, and performs no Git, documentation, or
source write. Its own prepared inputs are disposable cache files. The full read/write
boundary and local-only safety claims are in [the operator web design](../design/web-operator.md#d93).

### 3.1 `POST /api/wishes` — create a wish

Request fields:

| Field | Rule |
| --- | --- |
| `wish_id` | Required; matches `[A-Za-z0-9][A-Za-z0-9_-]{0,127}` |
| `title` | Required; 1–200 characters |
| `description` | Optional; at most 4000 characters |
| `flow_id` | Required; `simple-flow` or `wish-verify` |
| `checks` | Optional array; a non-empty array is allowed only with `simple-flow` and binds the four syntactic checkpoints |

A `checks` entry has `checkpoint_id`, `mode`, and the mode's fields:

| Field | Rule |
| --- | --- |
| `checkpoint_id` | One of `simpleRequirementsAccepted`, `simpleDesignAccepted`, `simpleImplementationAccepted`, `simpleRecordAccepted`; no duplicates |
| `mode` | `default` or `cli`; `command` is rejected here because a custom command is configured through `xdesign init --checks` |
| `cli`, `model` | Required for `cli`; `cli` is `codex` or `kimi` and `model` is a non-empty identifier of 1–200 characters |

A checkpoint omitted from `checks` defaults to `default`; bindings are not required to be
complete. An existing store item or legacy directory is `409 already_exists`.

The server writes `cache/web/wishes/<wish_id>.sysml` with the marker comment
`// xdesign-wish: {"title": "...", "description": "...", "flow_id": "..."}` and a package
whose documentation comment carries the display text, and, when at least one binding is
non-default, `cache/web/checks/<wish_id>.json` with
`{"schema_version": 1, "checks": [...]}`. It then invokes `init` with `--wish` and
`--checks` as applicable and returns `wish_id`, `flow_id`, `init` (the parsed CLI
response), `stdout`, and `stderr`. A non-zero `init` exit is an HTTP 500 error with code
`init_failed`, the CLI's stderr as the message, and the raw streams in `error.detail`.

### 3.2 `POST /api/wishes/{wish_id}/check` — record and check

Request fields:

| Field | Rule |
| --- | --- |
| `description` | Optional non-empty string; default `Web check, stage <stage>` |
| `performed_by` | Optional non-empty string; default `xdesign web operator` |
| `artifacts` | Optional array of `{uri}`; the server hashes each project file and rejects a missing file as `400` before invoking the CLI |
| `judgment` | Optional; meaningful only for a semantic exit checkpoint |

`judgment` has `authority` (`agent` or `human`), `identity`, `outcome`
(`passed`/`failed`/`unevaluated`), and `rationale`; all supplied values must be non-empty,
unknown fields are rejected, and a judgment supplied for a stage with no semantic exit
checkpoint is `400 invalid_judgment`.
The server fills `artifact_snapshot_digest` from the `candidate_snapshot_digest` of the
preceding `next` response, so the judgment binds the bytes the page displayed and cannot
approve older edits. Omitting the judgment records an unevaluated semantic assessment,
never an inferred pass.

The check response is `exit_code`, `response` (the parsed CLI check response with
`assessed` and `advance`), `stdout`, and `stderr`. To reach it the server first runs
`next`; a failed `next` is an HTTP 500 error with code `work_failed`, and a semantic
judgment for a stage without a semantic exit checkpoint is `400 invalid_judgment`. A
`check` subprocess with `exit_code` `0` or `1` is a successful envelope; exit code `1`
means the stage remained blocked and is returned with `exit_code: 1` so the page can show
the correction reasons. Any other exit code is an HTTP 500 error with code
`check_failed`, its message from stderr, and the raw streams under `error.detail`.

## 4. Control position and limits

This is a local operator interface, not a service contract. It binds loopback, has no
authentication or authorization layer, no TLS, no remote or multi-user deployment, and no
compatibility promise beyond this branch; the interface has no frozen baseline. The page
is not the documentation site, does not use its interface, and does not read the docs
tree. The API is not a second record authority: every durable write is the stage CLI's,
and the recorded plan, assessments, and evidence remain the authority for what happened.
Verification design for this contract is [WEB-01 to WEB-05](../verification/design.md#d96);
executed acceptance and retained command evidence belong to the branch's execution
record, not to this page.
