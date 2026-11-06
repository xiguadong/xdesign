# Toolchain and environment constraints

<a id="b08"></a>

### 0.1 Architecture constraints

| Constraint | Architectural effect | Source |
| --- | --- | --- |
| The project model must resolve without wish files | Reusable definitions may be referenced by wishes, but may not depend on their identities | [A02](index.md#a02), `modelStandsWithoutWishes` |
| Wish products must remain in the project model | A wish trace records provenance but cannot be the only copy of a requirement or design change | [A02](index.md#a02), `wishProductsAreRetained` |
| Completion requires current evidence | Every exit checkpoint must pass at the visit's artifact revision; a subflow must finish before its parent stage | [B03](system-composition.md#b03), [E03](../design/evidence-and-revisions.md#e03) |
| Syntactic and semantic judgments have different authorities | The former uses declared checkers and script evidence; the latter names permitted judges | [F02](../design/adapters.md#f02) |
| SysML checking uses the pinned local compiler | Model edits receive `parse` and `check` reports; unevaluated constraint bodies remain engineering obligations | [T01](#t01), [T04](#t04) |

No architecture-level cost, schedule, latency, availability, or hosting constraint has
been baselined. The repository's documentation and `cache/` conventions constrain this
design's artifacts; they do not select a runtime platform.

The pinned compiler, the commands used on this model, where every figure comes from, and
what a clean run does not establish.

<a id="t01"></a>

## 1. The pinned compiler

| Item | Value |
| --- | --- |
| Compiler | `haradama/sysml-v2-rs`, release `v0.1.4` |
| Executable | `tools/sysml_web/bin/sysml` — `sysml 0.1.4 (standard library 2026-05)` |
| SHA-256 | `674fce28512a772d1886ff2397f5f02dda66265299c67c01f5cc36eaf00207f6` |
| Toolchain record | `tools/sysml_web/toolchain.json` |
| Workbench | `tools/sysml_web/server.py` — a local UI that runs the compiler against submitted sources and projects the views the model declares |
| The designed flow's fixed checks | Four checkers and 18 rules are declared in the model and implemented under `tools/checks/`; a SimpleFlow run invokes them through a JSON command report ([D06](../design/flows/simple-flow/definition.md#d06), [D07](../design/flows/simple-flow/definition.md#d07)). Earlier prototypes under `cache/research/checks/` are separate research. |
| Figure renderer | `tools/render_docs.py` — projects declared views and model-owned presentation metadata, uses Graphviz for graph layout and SVG, and uses a compact ordered layout for the primary sequence; historical native exports remain supported ([D22–D26](../interfaces/documentation-contract.md#d22)) |
| Documentation audit | `tools/check_documentation.py --json` — checks model documentation targets, Markdown SVG embeds, and adjacent figure explanations ([D31](../interfaces/documentation-contract.md#d31)) |
| Documentation website | `tools/docs.py` — builds or previews these Markdown documents with MkDocs Material; configuration lives in `mkdocs.yml` |
| Operational storage | `xdesign::storage` and `xdesign store` — local SQLite plus content-addressed materials under `.xdesign/`; `XDESIGN_STORE_ROOT` selects an explicit alternate root ([D86](../design/storage.md#d86)) |

Preview the documentation website from the repository root:

```sh
python3 tools/docs.py serve
```

Open <http://127.0.0.1:8766>. The site includes document navigation, full-text search,
section links, light and dark themes, and enlarged diagram views with an **Open SVG with
source details** link. Markdown and asset
changes reload automatically. Use `--port 8767` to choose another local port.

Build static HTML with `python3 tools/docs.py build`. The generated site is written to
`cache/docs-site/html/`; serve that directory with any static HTTP server. The first run
creates `cache/docs-site/venv/` and installs the pinned dependencies from
`tools/docs_site/requirements.txt`, which requires internet access. Subsequent runs reuse
that environment. Environments, build output, dependency caches, and temporary files
stay under `cache/`. Durable operational revisions and their blobs instead stay under
`.xdesign/`; neither location is a development environment, and only cache is disposable.
The existing document filenames and section anchors are preserved; the site root opens the
[documentation entry](../index.md). Figure regeneration remains a separate command described
in [T03](#t03).

The compiler resolves the semantic model. The documentation adapter projects its export,
uses presentation metadata owned by the model, and preserves source identities in the
figures. The view selection, rendering, and explanatory prose are separate responsibilities;
each needs evidence appropriate to the claim ([D23](../interfaces/documentation-contract.md#d23)).

<a id="t02"></a>

## 2. Commands and what each establishes

| Command | Establishes | Does not establish |
| --- | --- | --- |
| `sysml parse <files>` | The files are syntactically valid SysML v2 | Anything semantic |
| `sysml check <dirs>` | Every name resolves and the specification's rules hold; reports `held` and `unevaluated` rules, unresolved references, collisions, violations | The model's own `assert constraint` bodies ([T04](#t04)) |
| `sysml export` | The semantic interchange model — the same content the workbench's declared-view projection reads | — |
| `sysml diagram <paths>` | The definitions-and-specializations figure | — |
| `sysml diagram --internal=<name>` | The internal structure of one definition | — |
| `sysml diagram --browser` | The membership tree of the loaded sources | — |
| `sysml diagram --sequence=<name>` | The interaction an occurrence definition declares ([E06](../records/executions/wish-journeys.md#e06)) | — |
| `python3 tools/check_documentation.py --json` | `DocSection` paths and anchors, `DocFigure` assets and carrying embeds, local SVG embeds, duplicate targets, and seven-field adjacent figure blocks | Whether a figure answers its viewpoint's question or its prose agrees with the model |

Run `check mbse/model mbse/wishes` for the current model and retain the complete report
under `cache/sysml/` or the current wish's evidence directory. Element counts and rule
results depend on the source revision; historical counts in the wish records describe
those revisions. A clean report does not establish layout quality or evaluate the model's
constraint bodies.

Run the documentation audit after changing model annotations, section anchors, figures,
or embeds. Its JSON report names each failed target; retain any report used for an
assessment under `cache/evidence/<wish>/`. The audit checks paths and presentation fields,
while the pinned compiler checks SysML syntax and supported name resolution. A reviewer
must still compare selected model content with its explanation
([D31](../interfaces/documentation-contract.md#d31)).

<a id="t03"></a>

## 3. Figure provenance

Every figure in this documentation is rendered from model content. A figure's name is
its path below `docs/`, within the carrying document directory's `assets/` child
(`docs/AGENTS.md`). Regenerate one with
`python3 tools/render_docs.py design/assets/controller`, or all of them with
`python3 tools/render_docs.py`.

| Figure | Rendered as | From |
| --- | --- | --- |
| `architecture/assets/model-definitions.svg` | definitions and specializations | `mbse/model` |
| `architecture/assets/lifecycle.svg` | declared view `XDesignWorkflow::wishLifecycle` | `Workflow::executeWish` and its actions |
| `architecture/assets/wish-to-delivery.svg` | native stage/checkpoint projection of `XDesignArchitecture::VModel` | compiler-exported stage successors, exit gates, kinds, criteria, and verification correspondences in [B30](wish-lifecycle.md#b30) |
| `architecture/assets/composition.svg` | internal structure | `XDesignArchitecture::XDesign` |
| `architecture/assets/stage-cli-packages.svg` | declared view `XDesignComposition::stageCliPackageView` | selected CLI types, controller actions, records, and the concrete `xdesign.cli` binding in [B01](system-composition.md#b01) |
| `architecture/assets/interfaces.svg` | declared view `XDesignWorkflow::workflowInterfaces` | `Workflow` parts and interfaces |
| `design/assets/wish-verify-flow.svg` | internal structure | `XDesignArchitecture::WishVerifyFlow` |
| `design/assets/controller.svg` | internal structure | `XDesignArchitecture::Controller` |
| `interfaces/assets/records.svg` | declared view `XDesignArchitecture::recordStructure` | the record definitions |
| `records/executions/assets/journey-rework.svg` | sequence | `Wish0002::Wish0002Journey` |
| `records/wishes/assets/journey-wish0005.svg` | sequence | `Wish0005::Wish0005Journey` |
| `records/wishes/assets/journey-wish0006.svg` | sequence | `Wish0006::Wish0006Journey` |
| `records/wishes/assets/journey-wish0009.svg` | sequence | `Wish0009::wish0009Sequence` and its recorded visits |
| `design/flows/simple-flow/assets/designed-flow.svg` | declared view `XDesignDetailedDesign::designedFlowTree` | the designed flow's stages, gates, and declared checker/rule attachments |
| `design/flows/simple-flow/assets/design-classes.svg` | declared view `XDesignDetailedDesign::designTree` | the run class, factory and functions with their members |
| `design/flows/simple-flow/assets/factory.svg` | declared view `XDesignDetailedDesign::factoryFlow` | the factory's creation steps |
| `design/flows/simple-flow/assets/gate-evaluation.svg` | declared view `XDesignDetailedDesign::gateEvaluationFlow` | the gate evaluation action's dispatch and assessment path |
| `design/flows/simple-flow/assets/checkpoint-evaluation.svg` | declared view `XDesignDetailedDesign::checkpointEvaluationFlow` | the checkpoint class's evaluate operation, its two paths and the assessment |
| `design/flows/simple-flow/assets/timeline.svg` | sequence | `SimpleFlowRunTimeline` |
| `design/flows/wish-verify/assets/cli-timeline.svg` | declared view `XDesignDetailedDesign::wishVerifyCliTimelineView`; action-order projection | one `AgentStageWork` iteration expanded through `StageCli` and `Controller`, scoped to the simplest gated two-stage WishVerifyFlow in [D84](../design/flows/wish-verify/views.md#d84) |
| `design/flows/wish-verify/assets/cli-swimlane.svg` | declared view `XDesignDetailedDesign::wishVerifyCliSwimlaneView`; responsibility projection | the same action selection and authorization branches; lanes are documented responsibilities and the repeat path is documented intent in [D84](../design/flows/wish-verify/views.md#d84) |
| `design/flows/simple-flow/assets/logical.svg` | definitions of one file, the rest as library | `mbse/model/detailedDesign.sysml` |
| `design/flows/simple-flow/assets/interfaces.svg` | declared view `XDesignDetailedDesign::interfaceStructure` | the automation's parts, ports and interfaces |
| `verification/assets/requirements.svg`, `requirements/assets/use-cases.svg` | requirement-trace and actor-goal projections | the two SRS views in [D15](../verification/requirements.md#d15) and [D17](../requirements/requirements.md#d17) |
| `architecture/assets/packages.svg` | `XDesignRustUml::projectPackageIntegration` package projection | all nine project packages, including composition and the Rust UML design, in [D21](system-composition.md#d21) |
| `architecture/assets/simple-flow-packages.svg` | `XDesignDetailedDesign::packageDiagram` package projection | the seven-package SimpleFlow concern retained in [D21](system-composition.md#d21) |
| `six files under design/flows/simple-flow/assets/ (block-definition, internal-block, parametric, activity, sequence, state-machine)` (six figures) | module type, interface, authorization, activity, sequence, and state projections | the six module views mapped in [D12](../design/flows/simple-flow/views.md#d12), with presentation rules in [D23](../interfaces/documentation-contract.md#d23) |
| `design/api/assets/crate-layers.svg` | declared view `XDesignRustUml::rustCrateLayers` | the crate's two layers and their one-way dependency, explained in [U68](../design/api/index.md#u68) |
| `design/flows/simple-flow/assets/crate-modules.svg` | declared view `XDesignRustUml::rustModuleLayout` | the crate's module dependencies, explained in [D45](../design/flows/simple-flow/implementation.md#d45) |
| `design/flows/simple-flow/assets/class-structure.svg` | declared view `XDesignRustUml::rustClassStructure` | Rust types and their relationships, explained in [D46](../design/flows/simple-flow/implementation.md#d46) |
| `design/flows/simple-flow/assets/gate-sequence.svg` | declared view `XDesignRustUml::rustGateSequence` | planned gate interaction, explained in [D47](../design/flows/simple-flow/implementation.md#d47) |
| `design/flows/simple-flow/assets/run-states.svg` | declared view `XDesignRustUml::rustRunStates` | run states and transitions, explained in [D48](../design/flows/simple-flow/implementation.md#d48) |
| `records/plans/assets/simple-flow-file-plan.svg` | focused projection of declared view `XDesignDetailedDesign::simpleFlowFilePlanView` | model-owned repository paths and responsibilities for Rust, checker, and verification files in [D53](../records/plans/simple-flow-file-plan.md#d53) |

The declared views are model content, not renderer settings: `wishLifecycle`,
`workflowInterfaces`, `recordStructure`, the SimpleFlow run and file-plan views, and
`interfaceStructure` pick what they expose and which rendering they ask for, and the workbench or
`render_docs.py` projects them accordingly. The Rust UML views also select their own
crate content. The nine coordinated SysML views, earlier structure graphs, and Rust UML
views carry `DocFigure` annotations locating their figure and document
([D09](../design/flows/simple-flow/views.md#d09)); the checkpoint evaluation figure
carries one too ([D11](../design/flows/simple-flow/definition.md#d11)).

The nine primary views also have matching `DocumentationViewpoint` usages. Each viewpoint
states a stakeholder question and binds its `subject` to the native view; the view's
`expose` declarations select content and `DocFigure` locates the asset. The carrying
design subsection supplies the explanation and limits ([D29](../interfaces/documentation-contract.md#d29)).

A declared view selects semantic content; it does not choose all visual positions or make
a generated SVG readable. `DiagramPresentation` supplies concise visible labels and
summaries while full model names, expressions, and IDs remain inspectable. The primary
nine figures use diagram-specific projection for boundaries, ports, parameter terminals,
actors, and control-flow notation. Graphviz places the graph elements and routes edges;
the primary sequence preserves its selected participants and message order in a compact
layout. The historical wish journey figures continue to use the pinned compiler's native
sequence renderer.

The WishVerifyFlow CLI timeline and swimlane project common action order and
responsibilities in the simplest two-stage flow context. Their declared views select
`AgentStageWork`, `StageCli`, `StageWorkOrder`, `Controller`, and the base and gated
wish-verify flow definitions. They are separate from the native run timeline:
no sequence messages, allocations, or loop back-edge are added by the projection.
Source identities and projection limits remain available in the SVG details and the
[carrying section](../design/flows/wish-verify/views.md#d84).

Generated layout input, SVGs, previews, and review records must refer to the same final
assets. The presentation review uses per-file hashes and actual built-article screenshots
([D24–D26](../interfaces/documentation-contract.md#d24)). Successful rendering and structural tests
establish only their stated results; they do not replace visual review.

<a id="t04"></a>

## 4. Limits, and what a clean check does not establish

- **Constraint bodies are not evaluated.** `check` reports `assert constraint`s as
  unevaluated; a constraint whose expression is false would still be reported as passing.
  The invariants in [E05](../design/evidence-and-revisions.md#e05), [F02](../design/adapters.md#f02)
  and [B03](system-composition.md#b03) are therefore verified by reading,
  and each verification activity records how.
- **Multi-valued values truncate in the export.** The export keeps only the first element
  of a multi-valued value — found while verifying wish-0002 with a three-element probe.
  Judgment permission is one boolean per authority instead of a list because of it
  ([F02](../design/adapters.md#f02)).
- **Not every rendering kind is projected by the workbench.** The declared-view projection
  draws interconnection, tree and general views. The documentation's nine-view adapter
  projects selected requirement, package, type, binding, use-case, activity, state, and
  sequence relationships from compiler interchange. The nine primary figures apply the
  notation and presentation contract in [D23](../interfaces/documentation-contract.md#d23); native
  sequence rendering remains in use for historical journey figures (`--sequence`,
  [T02](#t02)). These are focused presentations of the SysML v2 model, not a complete
  implementation of every SysML graphical convention. The export omits activity
  succession guard expressions and some initial-node references. Exact guards remain in
  the source model and [activity documentation](../design/flows/simple-flow/views.md#d18);
  unavailable references must be disclosed, not silently invented.
- **The workbench caps a run's output.** A workbench run refuses output past 64 MiB, and
  this model's export is larger, so `render_docs.py` runs `export` directly and projects
  the declared views itself. The workbench is for submitted sources, not for this model.

The focused [worktree store view](../design/storage.md#d116) maps
`XDesignStorage::worktreeStoreBoundaries` to `design/assets/worktree-store-boundaries.svg`
through the existing block-definition projection in `tools/render_docs.py`. Native node and
ownership relationship IDs remain in the SVG; its four exposed definitions are designed-only.
