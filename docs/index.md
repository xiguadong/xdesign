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
| How durable revisions, materials, saves and recovery work | [Storage design](design/storage.md#d86) and [store format](interfaces/store-format.md#i11) |
| The simplest agent interaction, shown as a timeline and swimlane | [WishVerifyFlow CLI interaction](design/flows/wish-verify/views.md#d84) |
| The class-and-function design of the crate | [design/api/index.md](design/api/index.md) |
| Cross-boundary contracts, CLI, and record format | [interfaces/index.md](interfaces/index.md) |
| How the design is verified and accepted | [verification/index.md](verification/index.md) |
| What currently runs, what is open, and what evidence exists | [records/status.md](records/status.md) |
| Plans, reviews, executions, and wish history | [records/index.md](records/index.md) |
| How to name, anchor, cite, and figure a document | [AGENTS.md](AGENTS.md) |

<a id="a08"></a>

## Reading paths

Start with this page, then follow the class page for the task. Use the linked section
when a rule matters, and use [current status](records/status.md) when capability or open
work matters. Page order is not the document set.

<a id="a09"></a>

## Document classes

| Class | Home | It owns |
| --- | --- | --- |
| Requirements | `requirements/` | normative requirements, rationale, acceptance conditions |
| Architecture | `architecture/` | context, composition, behavior, lifecycle, constraints |
| Detailed design | `design/` | module behavior, gates, revisions, evidence, and API design |
| Interface control | `interfaces/` | ICD, command line, and record data contracts |
| Verification design | `verification/` | methods, cases, criteria, acceptance boundaries, trace |
| Records | `records/` | plans, reviews, executions, evidence summaries, status, wish history |

`AGENTS.md` is the authoring and naming rules file and is not part of the published
reading set.

<a id="a10"></a>

## Flows and records

The selectable flows and their gates are described in [flow execution](design/flow.md#f03);
their designs are in [SimpleFlow](design/flows/simple-flow/index.md) and
[WishVerifyFlow](design/flows/wish-verify/index.md). Plans, reviews, executions, and wish
history are in [records](records/index.md); this entry does not repeat their status.

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
| `mbse/model/verification.sysml` | `verification/` |
| `mbse/model/uml.sysml` | `design/api/` and the module designs |
| `mbse/model/storage.sysml` | `design/storage.md`, `interfaces/store-format.md`, `design/api/storage/` |
| `mbse/model/common.sysml` | `design/evidence-and-revisions.md`, `interfaces/documentation-contract.md` |
| `mbse/wishes/wish-*.sysml` | `records/wishes/wish-history.md` (legacy/bootstrap traces) |
| `.xdesign/` item revisions and materials | `design/storage.md`, `interfaces/store-format.md`; local operational facts, not authored documentation |

## Status and evidence

Current capability status is maintained once in `records/status.md`. Historical
outcomes, plans, and reviews live in `records/` and keep their original dates and evidence;
a document's presence does not freeze a baseline or establish an engineering acceptance.
