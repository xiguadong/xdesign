# SysML v2 Workbench

A local web interface for loading, editing, checking, and rendering SysML v2
and KerML files. It uses the project's selected Rust-based
`haradama/sysml-v2-rs` v0.1.4 compiler. The server uses Python's standard
library, and the browser interface has no third-party dependencies.

Graph presentations use the Graphviz `dot` executable for measured labels and routed
connections. Install Graphviz and put `dot` on `PATH`, or set `SYSML_GRAPHVIZ_DOT` to its
executable. Tree views and the compiler's native sequence drawings retain their existing
renderers. A missing graph layout engine is reported as an error.

The nine primary documentation figures add diagram-specific composition over the native
model: short labels from `DiagramPresentation`, real system boundaries and port terminals,
and a compact event-anchored sequence. Run `python3 tools/render_docs.py` to regenerate
them. `python3 tools/verify_diagrams.py --help` describes the final-asset typography,
source-identity, preview and SHA-256 manifest checks. Visual review of the final built
article remains required.

## Run

From the repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/sysml_web/server.py
```

Open <http://127.0.0.1:8765>. The workbench loads the project's `mbse/` files.
You can also upload multiple `.sysml` and `.kerml` files, edit their source,
or create a new file. Keep dependent files loaded: the compiler checks and
resolves the entire workspace together.

Use **Check** to inspect diagnostics and the compiled model, or **Render** to
check and generate a diagram. Internal and sequence views require a definition
name, such as `XDesignArchitecture::XDesign`. Click a diagnostic to select its
file and source location. The interface offers SVG, model JSON, and report
downloads.

Source and Diagram use separate page views. The source editor occupies the
workspace width; **Diagram** opens the viewer across the whole browser page,
even before rendering. **Render** switches to that page immediately. Zoom,
pan, fit, view selection, and downloads remain available. Use **Source** or
**Escape** to return to the editor with your files and edits preserved. Diagrams
initially fit the page width and scroll vertically to keep labels readable;
**Fit all** shows the entire diagram at once.

## Model-defined views

Choose **SysML view** in the Diagram page and select a view declared in your
loaded model. The tool uses the compiler's resolved `expose` memberships and
rendering declarations, including rendering inherited from a local view
definition. A usage of `StandardViewDefinitions::GeneralView` can use its
standard graph presentation without an explicit `render` clause:

```sysml
view wishLifecycle : StandardViewDefinitions::GeneralView {
    expose Lifecycle::**;
}
```

Load [examples/declared_views.sysml](examples/declared_views.sysml) for a
standalone lifecycle model with that view and a declared tree rendering. Then
select `DeclaredViewExample::wishLifecycle` or
`DeclaredViewExample::lifecycleTree`.

Supported presentations are `Views::asTreeDiagram`,
`Views::asInterconnectionDiagram`, and the defaults for the standard
`GeneralView`, `BrowserView`, and `InterconnectionView` types. A view definition
is a reusable template; choose a concrete view usage with `expose` selections.
The view catalog shows why a declaration cannot be rendered.

Custom renderings, tabular/textual renderings, ordinary imports inside views,
and view filters are currently unsupported. The pinned compiler omits some
filter details from its semantic export and does not apply inherited filters
consistently. The tool therefore also retains the Rust parser's concrete syntax
tree to detect unsupported view selection, and reports it explicitly. It never
substitutes a whole-model diagram for an unsupported declared view.

Edits belong to the browser workspace and retained compiler snapshots. They do
not overwrite repository models. Download edited source to save your changes.

To use a different port or project directory:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/sysml_web/server.py --port 8766 --project-root /path/to/project
```

The compiler is bundled in this repository:

```text
tools/sysml_web/bin/sysml
```

The default is resolved relative to the tool's own directory, so no external
workspace is needed. The bundled executable is the selected macOS Apple Silicon
release. Its version, checksum, source archive, and license records are stored
in [toolchain.json](toolchain.json), `vendor/`, and `licenses/`.

The tool verifies its SHA-256 against the project pin and retains a working
copy under `cache/sysml-web/compiler/`. You may set `SYSML_COMPILER` or pass
`--compiler` explicitly. For the selected v0.1.4 binary on a different platform,
also pass its independently verified `--compiler-sha256`. Source builds can use
the bundled upstream source archive; extract and build it under the project's
`cache/`, with Cargo's build and dependency caches there too.

## Compilation and rendering

1. Freeze all loaded source files in a new run directory.
2. Run the Rust compiler's `parse` and `check` commands against those files.
3. If checks pass, run `export` to produce the compiler's native semantic model
   as standard SysML v2 interchange JSON.
4. Render the exported element graph into SVG, joining elements by native
   `@id` references. No second SysML parser or source-text pattern matching is
   used for the graph renderer.

The exported model is the compiler's semantic representation of the parsed
source, with typed elements, ownership, specializations, and relationships. It
is not the compiler's concrete token-tree debug dump. The model explorer
projects named declarations; the downloaded JSON retains the complete export,
including unnamed expressions and memberships. Native element IDs are scoped
to their compilation run.

| View | Rendering input |
| --- | --- |
| Definitions | Exported definitions, owned features, and native relationships |
| Internal structure | Exported parts and connectors of a selected definition |
| Model tree | Exported ownership hierarchy |
| SysML view | Exported view selection, declared rendering, or recognized standard view type |
| Sequence | Rust compiler's native sequence renderer on the same frozen files |

SVG rendering is a visualization of compiler-supported model facts. Missing
or unsupported relationship details are not reconstructed from source text.
Sequence rendering has the coverage provided by the pinned compiler.

## Reports and limits

Every run remains under `cache/sysml-web/runs/<run-id>/`, including source
snapshots, parse/check reports, raw model JSON, diagram SVG, command stderr,
and an aggregate `report.json`. Temporary compiler data also stays under
`cache/`. An invalid model retains diagnostics and does not produce a success
diagram. Reports preserve the number of held and unevaluated compiler rules.
Passing compiler checks does not establish engineering verification or
validation, or evaluate every modeled checkpoint.

The tool binds to localhost. It accepts up to 128 files, 2 MiB per file and
10 MiB per workspace. Compiler output is bounded to 64 MiB per command; the
default timeout is 30 seconds per command and can be changed with `--timeout`.
Uploaded paths must remain relative to the run's source directory. Reports
are retained until you remove them from `cache/`.

## API

| Endpoint | Result |
| --- | --- |
| `GET /api/project` | Project model source and compiler identity |
| `POST /api/analyze` | Check/render a frozen multi-file workspace |
| `GET /api/runs/<run-id>/<artifact>` | Download a retained report, export, or SVG |

An analysis request has this shape:

```json
{
  "files": [{"name": "example.sysml", "content": "package Example { part def System; }"}],
  "operation": "render",
  "view": "definitions",
  "target": ""
}
```

Compiler diagnostics return HTTP 200 with `ok: false`; malformed requests
return HTTP 400. Each response includes the compiler identity, rule counts,
diagnostics, compact model projection, and retained artifact URLs.

## Tests

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/sysml_web/tests -v
```

The integration tests invoke the actual selected Rust compiler and exercise
cross-file resolution, invalid source, diagnostics, SVG output, artifact
retention, and the HTTP API. Test workspaces remain under the project's
`cache/` directory.
