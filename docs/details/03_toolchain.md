# Toolchain

The pinned compiler, the commands used on this model, where every figure comes from, and
what a clean run does not establish.

<a id="t01"></a>

## T01 The pinned compiler

| Item | Value |
| --- | --- |
| Compiler | `haradama/sysml-v2-rs`, release `v0.1.4` |
| Executable | `tools/sysml_web/bin/sysml` — `sysml 0.1.4 (standard library 2026-05)` |
| SHA-256 | `674fce28512a772d1886ff2397f5f02dda66265299c67c01f5cc36eaf00207f6` |
| Toolchain record | `tools/sysml_web/toolchain.json` |
| Workbench | `tools/sysml_web/server.py` — a local UI that runs the compiler against submitted sources and projects the views the model declares |
| The designed flow's fixed checks | Four checkers and 18 rules are declared in the model and implemented under `tools/checks/`; a SimpleFlow run invokes them through a JSON command report ([D06](simple_flow/02_gates_and_rules.md#d06), [D07](simple_flow/02_gates_and_rules.md#d07)). Earlier prototypes under `cache/research/checks/` are separate research. |
| Figure renderer | `tools/render_docs.py` — projects declared views and model-owned presentation metadata, uses Graphviz for graph layout and SVG, and uses a compact ordered layout for the primary sequence; historical native exports remain supported ([D22–D26](04_diagram_presentation.md#d22)) |
| Documentation audit | `tools/check_documentation.py --json` — checks model documentation targets, Markdown SVG embeds, and adjacent figure explanations ([D31](05_documentation_contract.md#d31)) |
| Documentation website | `tools/docs.py` — builds or previews these Markdown documents with MkDocs Material; configuration lives in `mkdocs.yml` |

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
stay under `cache/`. The existing document filenames and section anchors are preserved;
the site root opens the [documentation entry](../00_index.md). Figure regeneration remains
a separate command described in [T03](#t03).

The compiler resolves the semantic model. The documentation adapter projects its export,
uses presentation metadata owned by the model, and preserves source identities in the
figures. The view selection, rendering, and explanatory prose are separate responsibilities;
each needs evidence appropriate to the claim ([D23](04_diagram_presentation.md#d23)).

<a id="t02"></a>

## T02 Commands and what each establishes

| Command | Establishes | Does not establish |
| --- | --- | --- |
| `sysml parse <files>` | The files are syntactically valid SysML v2 | Anything semantic |
| `sysml check <dirs>` | Every name resolves and the specification's rules hold; reports `held` and `unevaluated` rules, unresolved references, collisions, violations | The model's own `assert constraint` bodies ([T04](#t04)) |
| `sysml export` | The semantic interchange model — the same content the workbench's declared-view projection reads | — |
| `sysml diagram <paths>` | The definitions-and-specializations figure | — |
| `sysml diagram --internal=<name>` | The internal structure of one definition | — |
| `sysml diagram --browser` | The membership tree of the loaded sources | — |
| `sysml diagram --sequence=<name>` | The interaction an occurrence definition declares ([E06](02_records_and_evidence.md#e06)) | — |
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
([D31](05_documentation_contract.md#d31)).

<a id="t03"></a>

## T03 Figure provenance

Every figure in this documentation is rendered from model content. A figure's name is
its path below `docs/`, within the carrying document directory's `assets/` child
(`docs/AGENTS.md`). Regenerate one with
`python3 tools/render_docs.py details/assets/01_controller`, or all of them with
`python3 tools/render_docs.py`.

| Figure | Rendered as | From |
| --- | --- | --- |
| `arch/assets/00_model_definitions.svg` | definitions and specializations | `mbse/model` |
| `arch/assets/00_lifecycle.svg` | declared view `XDesignWorkflow::wishLifecycle` | `Workflow::executeWish` and its actions |
| `arch/assets/03_wish_to_delivery.svg` | native stage/checkpoint projection of `XDesignArchitecture::VModel` | compiler-exported stage successors, exit gates, kinds, criteria, and verification correspondences in [B30](../arch/03_wish_to_delivery.md#b30) |
| `arch/assets/01_composition.svg` | internal structure | `XDesignArchitecture::XDesign` |
| `arch/assets/01_interfaces.svg` | declared view `XDesignWorkflow::workflowInterfaces` | `Workflow` parts and interfaces |
| `details/assets/01_wish_verify_flow.svg` | internal structure | `XDesignArchitecture::WishVerifyFlow` |
| `details/assets/01_controller.svg` | internal structure | `XDesignArchitecture::Controller` |
| `details/assets/02_records.svg` | declared view `XDesignArchitecture::recordStructure` | the record definitions |
| `details/assets/02_journey_rework.svg` | sequence | `Wish0002::Wish0002Journey` |
| `review/assets/01_journey_wish0005.svg` | sequence | `Wish0005::Wish0005Journey` |
| `review/assets/01_journey_wish0006.svg` | sequence | `Wish0006::Wish0006Journey` |
| `review/assets/01_journey_wish0009.svg` | sequence | `Wish0009::wish0009Sequence` and its recorded visits |
| `details/simple_flow/assets/01_designed_flow.svg` | declared view `XDesignDetailedDesign::designedFlowTree` | the designed flow's stages, gates, and declared checker/rule attachments |
| `details/simple_flow/assets/01_design_classes.svg` | declared view `XDesignDetailedDesign::designTree` | the run class, factory and functions with their members |
| `details/simple_flow/assets/01_factory.svg` | declared view `XDesignDetailedDesign::factoryFlow` | the factory's creation steps |
| `details/simple_flow/assets/01_gate_evaluation.svg` | declared view `XDesignDetailedDesign::gateEvaluationFlow` | the gate evaluation action's dispatch and assessment path |
| `details/simple_flow/assets/02_checkpoint_evaluation.svg` | declared view `XDesignDetailedDesign::checkpointEvaluationFlow` | the checkpoint class's evaluate operation, its two paths and the assessment |
| `details/simple_flow/assets/03_timeline.svg` | sequence | `SimpleFlowRunTimeline` |
| `details/simple_flow/assets/03_logical.svg` | definitions of one file, the rest as library | `mbse/model/detailedDesign.sysml` |
| `details/simple_flow/assets/03_interfaces.svg` | declared view `XDesignDetailedDesign::interfaceStructure` | the automation's parts, ports and interfaces |
| `requirements/assets/01_requirements.svg`, `requirements/assets/01_use_cases.svg` | requirement-trace and actor-goal projections | the two SRS views in [D15](../requirements/01_baseline.md#d15) and [D17](../requirements/01_baseline.md#d17) |
| `arch/assets/02_packages.svg` | `XDesignRustUml::projectPackageIntegration` package projection | all nine project packages, including composition and the Rust UML design, in [D21](../arch/02_design.md#d21) |
| `arch/assets/02_simple_flow_packages.svg` | `XDesignDetailedDesign::packageDiagram` package projection | the seven-package SimpleFlow concern retained in [D21](../arch/02_design.md#d21) |
| `details/simple_flow/assets/05_*.svg` (six figures) | module type, interface, authorization, activity, sequence, and state projections | the six module views mapped in [D12](simple_flow/04_sysml_diagrams.md#d12), with presentation rules in [D23](04_diagram_presentation.md#d23) |
| `api/assets/01_crate_layers.svg` | declared view `XDesignRustUml::rustCrateLayers` | the crate's two layers and their one-way dependency, explained in [U68](../api/01_struct.md#u68) |
| `details/simple_flow/assets/06_crate_modules.svg` | declared view `XDesignRustUml::rustModuleLayout` | the crate's module dependencies, explained in [D45](simple_flow/06_rust_uml_design.md#d45) |
| `details/simple_flow/assets/06_class_structure.svg` | declared view `XDesignRustUml::rustClassStructure` | Rust types and their relationships, explained in [D46](simple_flow/06_rust_uml_design.md#d46) |
| `details/simple_flow/assets/06_gate_sequence.svg` | declared view `XDesignRustUml::rustGateSequence` | planned gate interaction, explained in [D47](simple_flow/06_rust_uml_design.md#d47) |
| `details/simple_flow/assets/06_run_states.svg` | declared view `XDesignRustUml::rustRunStates` | run states and transitions, explained in [D48](simple_flow/06_rust_uml_design.md#d48) |
| `prj_manage/assets/06_file_plan.svg` | focused projection of declared view `XDesignDetailedDesign::simpleFlowFilePlanView` | model-owned repository paths and responsibilities for Rust, checker, and verification files in [D53](../prj_manage/06_file_plan_and_verification.md#d53) |

The declared views are model content, not renderer settings: `wishLifecycle`,
`workflowInterfaces`, `recordStructure`, the SimpleFlow run and file-plan views, and
`interfaceStructure` pick what they expose and which rendering they ask for, and the workbench or
`render_docs.py` projects them accordingly. The Rust UML views also select their own
crate content. The nine coordinated SysML views, earlier structure graphs, and Rust UML
views carry `DocFigure` annotations locating their figure and document
([D09](simple_flow/03_structure_graphs.md#d09)); the checkpoint evaluation figure
carries one too ([D11](simple_flow/02_gates_and_rules.md#d11)).

The nine primary views also have matching `DocumentationViewpoint` usages. Each viewpoint
states a stakeholder question and binds its `subject` to the native view; the view's
`expose` declarations select content and `DocFigure` locates the asset. The carrying
design subsection supplies the explanation and limits ([D29](05_documentation_contract.md#d29)).

A declared view selects semantic content; it does not choose all visual positions or make
a generated SVG readable. `DiagramPresentation` supplies concise visible labels and
summaries while full model names, expressions, and IDs remain inspectable. The primary
nine figures use diagram-specific projection for boundaries, ports, parameter terminals,
actors, and control-flow notation. Graphviz places the graph elements and routes edges;
the primary sequence preserves its selected participants and message order in a compact
layout. The historical wish journey figures continue to use the pinned compiler's native
sequence renderer.

Generated layout input, SVGs, previews, and review records must refer to the same final
assets. The presentation review uses per-file hashes and actual built-article screenshots
([D24–D26](04_diagram_presentation.md#d24)). Successful rendering and structural tests
establish only their stated results; they do not replace visual review.

<a id="t04"></a>

## T04 Limits, and what a clean check does not establish

- **Constraint bodies are not evaluated.** `check` reports `assert constraint`s as
  unevaluated; a constraint whose expression is false would still be reported as passing.
  The invariants in [E05](02_records_and_evidence.md#e05), [F02](01_flows_and_checkpoints.md#f02)
  and [B03](../arch/01_system_composition.md#b03) are therefore verified by reading,
  and each verification activity records how.
- **Multi-valued values truncate in the export.** The export keeps only the first element
  of a multi-valued value — found while verifying wish-0002 with a three-element probe.
  Judgment permission is one boolean per authority instead of a list because of it
  ([F02](01_flows_and_checkpoints.md#f02)).
- **Not every rendering kind is projected by the workbench.** The declared-view projection
  draws interconnection, tree and general views. The documentation's nine-view adapter
  projects selected requirement, package, type, binding, use-case, activity, state, and
  sequence relationships from compiler interchange. The nine primary figures apply the
  notation and presentation contract in [D23](04_diagram_presentation.md#d23); native
  sequence rendering remains in use for historical journey figures (`--sequence`,
  [T02](#t02)). These are focused presentations of the SysML v2 model, not a complete
  implementation of every SysML graphical convention. The export omits activity
  succession guard expressions and some initial-node references. Exact guards remain in
  the source model and [activity documentation](simple_flow/05_module_design.md#d18);
  unavailable references must be disclosed, not silently invented.
- **The workbench caps a run's output.** A workbench run refuses output past 64 MiB, and
  this model's export is larger, so `render_docs.py` runs `export` directly and projects
  the declared views itself. The workbench is for submitted sources, not for this model.
