# Operator web program

The operator web program is the local browser surface over the operational `.xdesign`
store and the two-command stage CLI. Its page, server module, and embedded assets are
compiled into the single `xdesign` executable, so an operator can inspect wishes, follow
the flow lifecycle, create a wish, request current work, run a check, and read retained
materials without hand-editing JSON, a separate web server, or a Node.js toolchain. This
page states the program's position, request flow, read/write boundaries, and release
model. The local HTTP contract is in the [operator web API](../interfaces/web-api.md#i16);
the command line is in the [stage CLI](stage-cli.md#d85); the checkpoint library it
authors through is [the checkpoint library design](checkpoint-library.md#d97); and the
verification cases are in [the verification design](../verification/design.md#d96) and
[the round-2 cases](../verification/design.md#d104).

<a id="d91"></a>

## 1. Purpose and position

The program is an *operator surface*, not a new workflow authority. It presents what the
store and the stage CLI already decided, and its write actions are the existing
`init`/`next`/`check` commands run inside the same executable. The program is a Rust module
of the `xdesign` library, declared from `src/lib.rs` with
`#[path = "../web/server/mod.rs"]`, and its page is embedded at compile time; it has no
runtime process or service of its own besides the loopback HTTP listener.

| Neighbor | What it owns | How the web program relates to it |
| --- | --- | --- |
| [Stage CLI](stage-cli.md#d85) | Argument validation, the result exchange, gate evaluation, advancement, and store commits | Invokes the same three commands as subprocesses and reports their JSON responses and exit codes |
| [Operational store](storage.md#d86) | Durable revisions, materials, receipts, and projections | Reads through the storage queries; writes only through the stage CLI |
| [Checkpoint library](checkpoint-library.md#d97) | Git-tracked reusable check definitions in `checkpoints/` | Reads entries through `xdesign checkpoint list/show`; creates entries only through `xdesign checkpoint save`; never edits the library files itself |
| [Documentation site](../index.md) | Authored design, interface, and verification pages rendered by MkDocs | Not imported, not embedded, and not read: the web page is not the documentation site and has no documentation route |
| Generic workflow controller | The modeled `WorkflowController` that receives wishes, selects flows, and owns the run ([B01](../architecture/system-composition.md#b01)) | Not this program: the modeled controller is an architecture specification, not an implemented application ([F03](flow.md#f03), [B05](../architecture/system-context.md#b05)) |

The product constraint is a single released executable plus the repository content it
operates on. The web choice therefore adds an interface, not a deployment: the page
assets are part of the binary, the server binds loopback, and nothing in the page or the
server fetches the network.

<a id="d92"></a>

## 2. Request flow and data path

A browser loads the embedded page, which uses hash routing and calls the local JSON API.
The server handles one request per connection and routes it to one of four paths:

| Path | Server work | Durable effect |
| --- | --- | --- |
| Static | Serves a page asset from the embedded table; unknown extensionless paths fall back to `index.html` for hash routing | None |
| Store read | Assembles the response from store queries and the flow declarations: health, flow catalog, wish list, wish detail, events, history, frozen materials | None; readers validate the committed head and never advance it |
| Lifecycle action | Spawns `xdesign init`, `xdesign next`, or `xdesign check` as a subprocess of the same executable in the project root and returns its `stdout`/`stderr`/exit code and parsed response | Exactly what the invoked command commits: a registration, or one stage revision |
| Checkpoint library | Spawns `xdesign checkpoint list`/`show` for the Checkpoints view and plan binding labels; for a save, writes the request under `cache/web/checkpoint-requests/` and spawns `xdesign checkpoint save --request FILE` | The CLI's authoring update to `checkpoints/library.json` and, for a scaffold, its generated script; never a run-plan change |

The wish list combines two sources: store items of kind `wish`, whose status is the store
projection (`received`, `active`, `blocked`, `delivered`), and legacy writable
`cache/xdesign/<wish_id>/` registrations, which are listed read-only. Wish detail derives
its plan, run, visits, assessments, and event timeline from the stored run record and the
frozen plan; the plan's per-checkpoint binding is shown from the frozen plan, not from the
create request or any workspace configuration file. The flow catalog joins the built-in
plan declarations with static display metadata, and the agent catalog is static metadata
plus a best-effort runtime version check with a three-second deadline.

The create-wish action is the one place where the server prepares input for a lifecycle
command. It validates the request, writes the wish source and the optional check-binding
file under `cache/web/`, and then invokes `xdesign init` with `--wish` and, when a
non-default binding was chosen, `--checks`. The binding modes and their plan-freezing
semantics are the design in [D88](flows/simple-flow/definition.md#d88); the request and
response fields are in [the operator web API](../interfaces/web-api.md#i18).

<a id="d93"></a>

## 3. Read/write boundaries and safety

The program keeps the CLI's authority boundary: it never evaluates a gate, never appends
an assessment, and never writes a store revision itself. Its writes are limited to the
three lifecycle actions — create a wish, get next work, run a check — plus checkpoint saves,
and each one is a subprocess of the same executable, so argument validation, the writer
lock, the atomic stage save, and the duplicate/retry behavior are the ones described in the
[stage CLI](stage-cli.md#d85). A checkpoint save is an authoring write to project source,
not a run write, and the server never touches `checkpoints/` except through that
subprocess. Cancelling a request does not require a server-side recovery path: an
interrupted check is the same crash boundary the CLI already defines.

| Boundary | Rule |
| --- | --- |
| Listener | Loopback by default (`127.0.0.1`); one request per connection with `Connection: close` |
| Request size and time | Request bodies are capped at 1 MiB and reads time out; oversized bodies are refused as a request error |
| Writes | Only the three lifecycle actions and checkpoint saves; the server's own attempt, input, and save-request files stay under disposable `cache/web/`; no direct Git, documentation, or source edit |
| Paths | `wish_id` is an identifier matching a fixed pattern; the material endpoint serves only frozen head-revision bytes or legacy record evidence, never an arbitrary file path |
| Evidence | A check request's artifacts are hashed by the server before the CLI runs; a semantic judgment's snapshot digest is filled from the preceding work response so a judgment cannot bind older bytes |
| Page | All page assets are local; the page makes no external request and has no external dependency |
| Failure | A failed lifecycle subprocess is an error envelope with the command's stderr; a blocked stage is not a transport failure and still returns the recorded blocked response |

The page is a local single-operator surface. This slice has no authentication,
authorization, remote or multi-user deployment, TLS, or interface baseline; those remain
outside the web program's claim. The [HTTP conventions and error codes](../interfaces/web-api.md#i16)
state exactly what the program returns, and the API is not a second record authority.

<a id="d94"></a>

## 4. Embedded assets and release model

The build script `web/server/build.rs` walks `web/assets/` at compile time and generates a
table of `(path, include_bytes!, content type)` entries; the server serves that table from
memory and never depends on the source tree at runtime. Adding, splitting, or removing a
page file is picked up by the next build — there is no checked-in asset manifest and no
separate packaging step.

| Concern | Design |
| --- | --- |
| Shipping unit | The single `xdesign` executable; the page and the server are compiled in |
| Page source root | Repository `web/`, with the page under `web/assets/` and the Rust module under `web/server/` |
| Build dependency | The existing Cargo build only; no Node.js, npm, bundler, minifier, or CDN, and no Python web server at runtime |
| Asset naming | Embedded paths are root-relative (`/index.html`, `/app.js`, `/styles.css`, and any split assets); content types are generated from the file extension |
| Release behavior | A page change requires an executable rebuild; a binary built from an earlier revision keeps serving its embedded assets |
| Documentation independence | The MkDocs site and its generated `cache/docs-site/` output are not inputs to the build, not embedded, and not read by the server; the web page is not the docs site |

`web/SPEC.md` is the branch's working server/page contract, and `web/README.md` records the
operator and developer workflow. They are source-tree notes, not published documentation:
this page and the [operator web API](../interfaces/web-api.md#i16) are the published
design and interface claims.

<a id="d105"></a>

## 5. Operator views and check-executor wording

The round-2 page adds a **Checkpoints** view and a library choice in the create-wish
wizard. The view lists the effective library from `GET /api/checkpoints` and shows each
entry's title, id, kind chip, built-in versus user source, executor summary, rule or
criteria count, version, updated time, and tags. A detail drawer shows the full description,
the rule table, inputs, executor, criteria and judges, and the retained version history. A
New checkpoint form selects the kind explicitly, requires a description, edits rules or
criteria, inputs and executor, and surfaces `invalid_checkpoint`/`builtin_checkpoint`
errors inline; a built-in entry is read-only and can pre-fill a new user entry.

The wizard's check cards keep the resource-selection analogy rather than adding a second
workflow: a `Check executor` choice with `Built-in script`, `Agent CLI`, and `Library`
(the `Library checkpoint` form is used in chips and detail rows). The distinction is
necessary because the whole flow's stage work is performed by the working agent regardless
of this choice; the choice selects only who verifies the checkpoint:

| Choice | Meaning |
| --- | --- |
| Built-in script | The repository's deterministic checker, recorded with script authority |
| Agent CLI | A separate `codex`/`kimi` run judges the same declared rules, recorded with agent authority `<cli>:<model>` |
| Library checkpoint | The frozen library entry's rules or criteria are used with the executor that entry declares ([D100](checkpoint-library.md#d100)) |

The four syntactic cards share one stable grid, so the executor control and its dependent
columns (CLI, model, or library selector) stay aligned when the mode changes and at the
narrow responsive widths; the choice never overlaps or reflows another control. Each check
node states the working-agent distinction in plain language, and every chip, wizard review
line, lifecycle node, and assessment row uses the same vocabulary. An assessment always
shows `judged_by` (`script`/`agent`/`human`) and the assessor, so a library script, a
built-in script, and an agent run are not presented as the same executor.

The Flows view lists library availability alongside `custom_checks_supported` so an
operator can see which library entries a flow's gates can accept. The wizard's review step
lists each library binding with the entry title and kind; the wish detail's lifecycle check
nodes show the frozen library id and, when it resolves, its current title and kind.

<a id="d106"></a>

## 6. Start-work handoff

Creating a wish — including freezing its flow plan and library bindings — never starts an
agent. The run waits at its first stage. The only way stage work starts is a caller handing
the current work order to a working agent and submitting the stage result:

```sh
xdesign next WISH_ID                     # the work order for the current stage
xdesign check WISH_ID --result FILE      # submit the stage outputs and let the gates decide
```

The web program is an operator console, not a worker: it can run `next` and `check` at the
operator's request and display the resulting work order and assessments, but it never
launches the stage work and never bypasses the check. The handoff is explicit in the page:

- a one-line explanation on every non-delivered wish that the flow does not auto-run;
- a copyable handoff prompt built from the wish detail — wish id and title, flow and
  current stage, goal, instructions and expected outputs, the commands to run, and the
  requirement that the result file carry the current `state_token`, the batch fields
  (activities, artifacts, and digests), and a permitted judgment when the stage has a
  semantic exit checkpoint;
- copyable `xdesign next WISH_ID` and `xdesign check WISH_ID --result FILE` commands, and a
  "Load current work" action that runs the read-only `next` so the prompt can include the
  loaded work order and `state_token`;
- after a successful check the page refreshes the detail (current stage, assessments,
  events) and shows the next handoff state; a blocked check shows the correction reasons
  and keeps the same handoff with those corrections included.

The prompt is composed in the page from `GET /api/wishes/{wish_id}` and the work response;
no handoff endpoint is required. The normal Get next work and Run check controls stay
available below the handoff card, but they are operator actions on the same CLI, not an
automatic stage worker. The work order still comes from `next` and the submission still
goes through `check`, so the handoff adds no new lifecycle command and no server-side
automation.

## 7. Software realization and limits

| Source | Responsibility |
| --- | --- |
| `web/server/mod.rs` | `serve()` entry, option parsing, project-root change, listener, module root |
| `web/server/http.rs` | Minimal HTTP/1.1 request and response handling, limits, JSON envelopes |
| `web/server/api.rs` | Endpoint data assembly, store reads, lifecycle subprocesses, material serving |
| `web/server/agents.rs` | Static codex/kimi catalog and best-effort availability detection |
| `src/checkpoint/` | Checkpoint library model, validation, built-in seeds, version/history, scaffold generation, and the `checkpoint` command replies |
| `checkpoints/` | Project-level authoring source: `library.json`, `README.md`, and generated scripts under `scripts/` |
| `web/server/build.rs` | Compile-time embedding of `web/assets/` |
| `web/assets/` | The single-page operator interface: entry document, application script, styles, local SVG assets |
| `src/lib.rs` | `#[path = "../web/server/mod.rs"] pub mod web;` — declares the module while keeping all web source under `web/` |
| `src/main.rs` | Dispatches `xdesign web` to the embedded server |

Limits of this design:

- `XDesignArchitecture::OperatorWebProgram` now allocates this local boundary, and
  `xdesign.webOperator` binds it to the project context and stage CLI. The as-built
  [model/source map](checkpoint-library.md#d107) was added after the original delivery;
  it does not backdate a design-stage approval. IF-001 to IF-004 in the
  [interface control document](../interfaces/contracts.md#i01) remain unchanged; the
  local HTTP contract is the [operator API](../interfaces/web-api.md#i16).
- The program displays and drives the existing lifecycle; it does not add a new stage
  action, a rebind operation, whole-store backup, remote operation, or project acceptance.
- The page's status, event, and evidence views are projections of the stored record. They
  do not create a second record and do not establish an engineering acceptance.
- The verification design states the cases; executed acceptance and its retained evidence
  belong to the branch's execution record, not to this page.

The server selects one project root and one store root. Checkpoint edits are made in that
project's Git-owned source, while stage attempts and evidence belong to its store. Its
history view is not a multi-project history service. The current ownership, portability
limits and proposed shared-definition/history boundary are in the
[checkpoint project design](checkpoint-library.md#d107).
