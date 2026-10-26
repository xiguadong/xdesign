# xdesign documentation

xdesign is a SysML v2 engineering tool: the project model explains the system, and each
wish's operational trace explains why its requirements, design, implementation, and
verification followed. New runs retain that trace in the store; legacy and bootstrap runs
retain independent SysML traces. This documentation set states the design, exact interfaces,
verification design, and records behind what has actually run.

Start with the [project overview](README.md) for project composition, usage, and the
package, timeline, and swimlane diagrams together on one page.

<a id="a07"></a>

## Read by task

| I want to understand... | Start here |
| --- | --- |
| The project, its parts, and how they collaborate | [architecture/index.md](architecture/index.md) |
| What the system must do and why | [requirements/index.md](requirements/index.md) |
| How the flows and runs are designed | [design/index.md](design/index.md) |
| How a wish's four SimpleFlow checks are bound to a default check, an agent CLI, a custom command, or a reusable library checkpoint | [Configurable check bindings](design/flows/simple-flow/definition.md#d88) |
| How reusable checks are authored, versioned, and shared across wishes | [Checkpoint library](design/checkpoint-library.md#d97) |
| Where checks and history belong in a real project, and the proposed cross-project boundary | [Project ownership and history](design/checkpoint-library.md#d107); [cross-project proposal](design/checkpoint-library.md#d108) |
| The embedded operator web page and its local HTTP contract | [Operator web program](design/web-operator.md#d91) and [operator web API](interfaces/web-api.md#i16) |
| How durable revisions, materials, saves and recovery work | [Storage design](design/storage.md#d86) and [store format](interfaces/store-format.md#i11) |
| The simplest agent interaction, shown as a timeline and swimlane | [WishVerifyFlow CLI interaction](design/flows/wish-verify/views.md#d84) |
| The class-and-function design of the crate | [design/api/index.md](design/api/index.md) |
| Cross-boundary contracts, CLI, and record format | [interfaces/index.md](interfaces/index.md) |
| How the design is verified and accepted | [verification/index.md](verification/index.md) |
| What currently runs, what is open, and what evidence exists | `python3 tools/records.py show record-status`; frozen 2026-10-25 summary in [records/status.md](records/status.md) |
| Plans, reviews, executions, and wish history | `python3 tools/records.py list` for maintained records; frozen history in [records/index.md](records/index.md) |
| How to name, anchor, cite, and figure a document | [AGENTS.md](AGENTS.md) |

<a id="a08"></a>

## Reading paths

Start with this page, then follow the class page for the task. Use the linked section
when a rule matters, and use the store `record-status` item
(`python3 tools/records.py show record-status`) when capability or open work
matters; the frozen 2026-10-25 page remains as historical evidence. Page order is not the
document set.

<a id="a09"></a>

## Document classes

| Class | Home | It owns |
| --- | --- | --- |
| Requirements | `requirements/` | normative requirements, rationale, acceptance conditions |
| Architecture | `architecture/` | context, composition, behavior, lifecycle, constraints |
| Detailed design | `design/` | module behavior, gates, revisions, evidence, and API design |
| Interface control | `interfaces/` | ICD, command line, and record data contracts |
| Verification design | `verification/` | methods, cases, criteria, acceptance boundaries, trace |
| Records | `.xdesign` record items; `records/` frozen archive | maintained plans, reviews, executions, status, wish history; the frozen archive holds their history and acceptance evidence |

`AGENTS.md` is the authoring and naming rules file and is not part of the published
reading set.

<a id="a10"></a>

## Flows and records

The selectable flows and their gates are described in [flow execution](design/flow.md#f03);
their designs are in [SimpleFlow](design/flows/simple-flow/index.md) and
[WishVerifyFlow](design/flows/wish-verify/index.md). Maintained plans, reviews,
executions, status, and wish-history updates are `record` items in the local `.xdesign`
store (conventions in the [storage design](design/storage.md#d86));
`python3 tools/records.py list` and `show <item>` read them compactly;
[records](records/index.md) is their frozen historical archive, and this entry does not
repeat either one's status.

<a id="a11"></a>

## Model-to-document map

The SysML model is the structural source; the annotations `DocSection` and `DocFigure`
point from model elements and views to the sections and figures that explain them. The
interfaces and meaning of those annotations are defined in
`interfaces/documentation-contract.md`.

| Model source | Primary explanation |
| --- | --- |
| `mbse/model/requirements.sysml`, `traceability.sysml` | `requirements/requirements.md`, `verification/requirements.md` |
| `mbse/model/architecture.sysml`, `composition.sysml`, `workflow.sysml` | `architecture/`, `design/flow.md`, `design/lifecycle.md` |
| `mbse/model/detailedDesign.sysml` | `design/flows/simple-flow/`, `design/flows/wish-verify/` |
| Project/operator additions in `architecture.sysml`, `composition.sysml`, `detailedDesign.sysml`, `uml.sysml` | `design/checkpoint-library.md#d107`, `design/web-operator.md`; proposed cross-project definitions are explicitly designed-only |
| `mbse/model/verification.sysml` | `verification/` |
| `mbse/model/uml.sysml` | `design/api/` and the module designs |
| `mbse/model/storage.sysml` | `design/storage.md`, `interfaces/store-format.md`, `design/api/storage/` |
| `mbse/model/common.sysml` | `design/evidence-and-revisions.md`, `interfaces/documentation-contract.md` |
| `mbse/wishes/wish-*.sysml` | `records/wishes/wish-history.md` (frozen legacy/bootstrap traces) |
| `.xdesign/` item revisions and materials | `design/storage.md`, `interfaces/store-format.md`; local operational facts, not authored documentation |

## Status and evidence

Current capability status is maintained once in the store `record-status` item
(`python3 tools/records.py show record-status`); the frozen
[records/status.md](records/status.md) is the 2026-10-25 historical snapshot.
Historical outcomes, plans, and reviews are the frozen `records/` archive and keep their
original dates and evidence; a document's presence does not freeze a baseline or establish
an engineering acceptance.
