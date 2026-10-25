# Operator web program

The operator web program is the local browser surface over the operational `.xdesign`
store and the two-command stage CLI. Its page, server module, and embedded assets are
compiled into the single `xdesign` executable, so an operator can inspect wishes, follow
the flow lifecycle, create a wish, request current work, run a check, and read retained
materials without hand-editing JSON, a separate web server, or a Node.js toolchain. This
page states the program's position, request flow, read/write boundaries, and release
model. The local HTTP contract is in the [operator web API](../interfaces/web-api.md#i16);
the command line is in the [stage CLI](stage-cli.md#d85); and the verification cases are
in [the verification design](../verification/design.md#d96).

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
| [Documentation site](../index.md) | Authored design, interface, and verification pages rendered by MkDocs | Not imported, not embedded, and not read: the web page is not the documentation site and has no documentation route |
| Generic workflow controller | The modeled `WorkflowController` that receives wishes, selects flows, and owns the run ([B01](../architecture/system-composition.md#b01)) | Not this program: the modeled controller is an architecture specification, not an implemented application ([F03](flow.md#f03), [B05](../architecture/system-context.md#b05)) |

The product constraint is a single released executable plus the repository content it
operates on. The web choice therefore adds an interface, not a deployment: the page
assets are part of the binary, the server binds loopback, and nothing in the page or the
server fetches the network.

<a id="d92"></a>

## 2. Request flow and data path

A browser loads the embedded page, which uses hash routing and calls the local JSON API.
The server handles one request per connection and routes it to one of three paths:

| Path | Server work | Durable effect |
| --- | --- | --- |
| Static | Serves a page asset from the embedded table; unknown extensionless paths fall back to `index.html` for hash routing | None |
| Store read | Assembles the response from store queries and the flow declarations: health, flow catalog, wish list, wish detail, events, history, frozen materials | None; readers validate the committed head and never advance it |
| Lifecycle action | Spawns `xdesign init`, `xdesign next`, or `xdesign check` as a subprocess of the same executable in the project root and returns its `stdout`/`stderr`/exit code and parsed response | Exactly what the invoked command commits: a registration, or one stage revision |

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
an assessment, and never writes a store revision itself. Its writes are limited to three
lifecycle actions — create a wish, get next work, run a check — and each one is a
subprocess of the same executable, so argument validation, the writer lock, the atomic
stage save, and the duplicate/retry behavior are the ones described in the
[stage CLI](stage-cli.md#d85). Cancelling a request does not require a server-side
recovery path: an interrupted check is the same crash boundary the CLI already defines.

| Boundary | Rule |
| --- | --- |
| Listener | Loopback by default (`127.0.0.1`); one request per connection with `Connection: close` |
| Request size and time | Request bodies are capped at 1 MiB and reads time out; oversized bodies are refused as a request error |
| Writes | Only the three lifecycle actions; the server's own attempt and input files stay under disposable `cache/web/`; no Git write, no documentation write, no source edit |
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

## 5. Software realization and limits

| Source | Responsibility |
| --- | --- |
| `web/server/mod.rs` | `serve()` entry, option parsing, project-root change, listener, module root |
| `web/server/http.rs` | Minimal HTTP/1.1 request and response handling, limits, JSON envelopes |
| `web/server/api.rs` | Endpoint data assembly, store reads, lifecycle subprocesses, material serving |
| `web/server/agents.rs` | Static codex/kimi catalog and best-effort availability detection |
| `web/server/build.rs` | Compile-time embedding of `web/assets/` |
| `web/assets/` | The single-page operator interface: entry document, application script, styles, local SVG assets |
| `src/lib.rs` | `#[path = "../web/server/mod.rs"] pub mod web;` — declares the module while keeping all web source under `web/` |
| `src/main.rs` | Dispatches `xdesign web` to the embedded server |

Limits of this design:

- It is a local operator convenience, not a modeled interface: IF-001 to IF-004 in the
  [interface control document](../interfaces/contracts.md#i01) are unchanged, and no SysML
  element was added for this program. The interface claim rests on this page, the
  [API page](../interfaces/web-api.md#i16), and the source.
- The program displays and drives the existing lifecycle; it does not add a new stage
  action, a rebind operation, whole-store backup, remote operation, or project acceptance.
- The page's status, event, and evidence views are projections of the stored record. They
  do not create a second record and do not establish an engineering acceptance.
- The verification design states the cases; executed acceptance and its retained evidence
  belong to the branch's execution record, not to this page.
