# xdesign documentation index

> Project: xdesign | Status: working design documentation | Updated: 2026-10-20 | Audience: requesters, model
> authors, implementers, and reviewers.

This is the entry to the project's engineering documentation. The project SysML v2 model
describes the system and workflow; one independent model per wish records why its work was
done and the evidence used to judge it. The documents explain the model, design choices,
limits, and review results. Start with a reading path below, then follow stable section
links rather than relying on page order.

<a id="a07"></a>

## A07 What is documented

xdesign manages wishes through selected engineering flows. Its controller records work and
evidence by stage and permits progress only when the applicable checkpoints authorize it.
The [architecture overview](arch/00_overview.md#a01) introduces the two questions the
system answers; [system composition](arch/01_system_composition.md#b01) shows its parts and
interfaces. The [wish-to-delivery graph](arch/03_wish_to_delivery.md#b30) shows the
default route through requirements, design, implementation, verification, and delivery.
A flow may be the full V-model, a four-stage SimpleFlow, or a shorter design or
two-stage wish or design flow. The model's definitions and recorded runs must be read separately.

The present repository contains model definitions, documentation, a figure renderer, the
documentation site, and executable four-stage SimpleFlow and two-stage wish-verify
modules in the xdesign crate, with four gate commands for the SimpleFlow slice.
It does not yet contain the complete workflow application or the three VModel gate
commands ([T01](details/03_toolchain.md#t01),
[V04](review/01_wish_status.md#v04)). A clean compiler result establishes only its
supported syntax and semantic checks, not the model's constraint bodies or engineering
acceptance ([T04](details/03_toolchain.md#t04)).

<a id="a08"></a>

## A08 Reading paths

| Reader | Start here | Then read |
| --- | --- | --- |
| Requester or reviewer | [wish-to-delivery graph](arch/03_wish_to_delivery.md#b30) and [wish status](review/01_wish_status.md#v02) | [requirements specification](requirements/01_baseline.md#r04), the [stage review checklist](review/02_checklist.md#v08), and the relevant wish evidence |
| Model author or architect | [requirements](requirements/01_baseline.md#r01) and [composition](arch/01_system_composition.md#b01) | [architecture design](arch/02_design.md#b05), [flows](details/01_flows_and_checkpoints.md#f01), and [documentation contract](details/05_documentation_contract.md#d27) |
| Module implementer | [SimpleFlow module design](details/simple_flow/05_module_design.md#d32) | [run design](details/simple_flow/01_run_design.md#d01), [gates and rules](details/simple_flow/02_gates_and_rules.md#d05), [Rust UML design](details/simple_flow/06_rust_uml_design.md#d44), [file plan and verification](prj_manage/06_file_plan_and_verification.md#d53), [interface contract](icd/01_interfaces.md#i01), and [toolchain](details/03_toolchain.md#t01) |
| Integrator or verifier | [interface contract](icd/01_interfaces.md#i01) and [records and evidence](details/02_records_and_evidence.md#e01) | [requirements trace](requirements/01_baseline.md#r02), [stage checklist](review/02_checklist.md#v08), and [wish review](review/01_wish_status.md#v01) |
| New agent | This index and the repository `AGENTS.md` files | [model map](#a11), [documentation contract](details/05_documentation_contract.md#d27), then the relevant flow and wish |

The [project management area](prj_manage/00_overview.md) records the combined design
review, the SimpleFlow implementation plan, its acceptance checks, and remaining work.

<a id="a09"></a>

## A09 Document set and its growth

| Concern | Current document or location | State |
| --- | --- | --- |
| Entry and system context | This index; [architecture overview](arch/00_overview.md#a01) | Written |
| Requirements and traceability | [requirements overview](requirements/00_overview.md#r15), [specification](requirements/01_baseline.md#r04), [inventory](requirements/01_baseline.md#r01), and [trace](requirements/01_baseline.md#r02) | Stage topics written; missing acceptance methods and targets remain explicit |
| Architecture and concept selection | [wish-to-delivery route](arch/03_wish_to_delivery.md#b30), [architecture design](arch/02_design.md#b05), [composition](arch/01_system_composition.md#b01), and [flows](details/01_flows_and_checkpoints.md#f01) | Stage topics written; deployment, concept comparison, and PDR evidence remain incomplete |
| Detailed module design | [shared design overview](details/00_overview.md#d42), [SimpleFlow overview](details/simple_flow/00_overview.md#d43), [module design](details/simple_flow/05_module_design.md#d32), [run](details/simple_flow/01_run_design.md#d01), and [gates](details/simple_flow/02_gates_and_rules.md#d05) | Figures embedded with module topics; a four-stage Rust and checker slice runs, while the full application remains open |
| WishVerifyFlow module design | [WishVerifyFlow overview](details/wish_verify_flow/00_overview.md#d56), [run design](details/wish_verify_flow/01_run_design.md#d57), [gates and judgment](details/wish_verify_flow/02_gates_and_judgment.md#d61), [module design](details/wish_verify_flow/04_module_design.md#d65), and [Rust UML design](details/wish_verify_flow/05_rust_uml_design.md#d75) | Two-stage judged slice designed with figures embedded; `xdesign::wish_verify` exists, executed verification pending in the plan |
| Interface control and decisions | [interface overview](icd/00_overview.md#i10), [ICD](icd/01_interfaces.md#i01), [ADR overview and register](adr/00_overview.md#j01) | Model-level exchanges documented; transport/versioning undecided and no ADR accepted |
| Views and figure provenance | [nine-question coverage](details/simple_flow/04_sysml_diagrams.md#d12), [presentation](details/04_diagram_presentation.md#d22), and [toolchain](details/03_toolchain.md#t03) | Figures are embedded in their requirements, architecture, or module section; coverage and provenance are indexed separately |
| API reference and UML software design | [xdesign API reference](api/00_overview.md#u01), [crate structure](api/01_struct.md#u68), [SimpleFlow Rust UML design](details/simple_flow/06_rust_uml_design.md#d44), [wish-verify Rust UML design](details/wish_verify_flow/05_rust_uml_design.md#d75) | Three-layer inventory with one page per API; the core + simple_flow slice has a first executable implementation, and the wish_verify slice follows it |
| Project management | [Review synthesis, implementation plan, file plan, acceptance cases, and execution record](prj_manage/00_overview.md#p01) | Review findings, implementation sequence, observed test results, and limits |
| Reviews and wish outcomes | [review overview](review/00_overview.md#v15), [stage checklist](review/02_checklist.md#v08), and [wish status](review/01_wish_status.md#v02) | Stage gate gaps and historical wish assessments are distinguished |

The actual directory map is:

| Directory | Primary content |
| --- | --- |
| `docs/requirements/` | Requirements specification and traceability |
| `docs/arch/` | Architecture design and system composition |
| `docs/details/` | Cross-flow contracts, toolchain, and detailed designs under `simple_flow/` and `wish_verify_flow/` |
| `docs/icd/`, `docs/adr/`, `docs/review/`, `docs/prj_manage/` | Interface control, decision records, reviews, and project planning |
| `docs/api/` | The xdesign crate's class-and-function API reference: [inventory and project boundary](api/00_overview.md#u65), [crate structure map](api/01_struct.md#u68), and one page per API under `core/`, `simple_flow/`, and `wish_verify_flow/`; module-level UML design stays with its module |
| Each document directory's `assets/` | Model-rendered figures beside their carrying documents, for example `docs/arch/assets/` and `docs/details/simple_flow/assets/` |
| `mbse/model/`, `mbse/wishes/` | Reusable project model and independent per-wish traces |

The package under `cache/templates/` supplied the stage topics now carried by the
requirements, architecture, module design, ICD, ADR index, and review checklist above.
The templates are not project baselines; each current document states its actual maturity.
The root entry is `00_index.md`; each prose subdirectory starts with `00_overview.md`
and then has contiguous numbered topics. A topic with several documents can have its own
subdirectory under its current module; new flows can get a `docs/details/<flow>/` area.
After the revision-3 migration, preserve existing section IDs and links when documents
are split or expanded
([D30](details/05_documentation_contract.md#d30)).

<a id="a10"></a>

## A10 Flows and their records

`XDesignComposition::xdesign` owns every selectable flow as a specialization of its
abstract `xdesign.'flow' : Flow`. `xdesign.vModel` is the default selection. Each block
gives the flow's purpose and current execution status; [F03](details/01_flows_and_checkpoints.md#f03)
contains the exact modeled stages and gates. A generic controller and `FlowExecution`
record apply where no specialized run or factory has been designed.

**`xdesign.vModel : VModel` — default full lifecycle**

| Item | Current design |
| --- | --- |
| Purpose and stages | Eleven stages from `wishIntake` through stakeholder needs, requirements, architecture, detailed design, implementation, component/integration/system verification, acceptance validation, and `delivery` ([F03](details/01_flows_and_checkpoints.md#f03)). |
| Exit checkpoints | One per stage: syntactic `requirementsAccepted`, `implementationReady`, `deliveryAccepted` (script judged when their commands exist); semantic `intakeAccepted`, `needsAccepted`, `architectureAccepted`, `designAccepted`, `componentsVerified`, `integrationVerified`, `systemVerified`, `wishValidated` (agent or human according to each gate's flags). |
| Run and factory | Generic `FlowExecution` records and `Controller::receiveWish`; no VModel-specific run class or factory is declared. |
| Rules and checks | The three syntactic checkpoints name checkers, but their commands and rule lists are pending. The semantic criteria are declared in `mbse/model/architecture.sysml` (`VModel`); [F02–F03](details/01_flows_and_checkpoints.md#f02) explain their judge and flow context. Open items are in [V04](review/01_wish_status.md#v04). |
| Figures | The [wish-to-delivery overview](arch/03_wish_to_delivery.md#b30) shows the eleven-stage route, exit gates, and verification links. |
| Status | Flow modeled; complete VModel checker execution is not implemented. |

**`xdesign.workflowFlow : WorkflowFlow` — historical four-stage usage**

| Item | Current design |
| --- | --- |
| Purpose and stages | Requirements → design → implement → verify for wishes that derive requirements and implementation ([F03](details/01_flows_and_checkpoints.md#f03)). |
| Exit checkpoints | Only terminal `verificationAccepted`, a semantic gate judged by an agent or human; the first three stages have no exit gates. |
| Run and factory | Generic `FlowExecution` records and `Controller::receiveWish`; no specialized factory for this usage. |
| Rules and checks | The semantic criterion asks whether verification evidence demonstrates implementation against requirements ([F03](details/01_flows_and_checkpoints.md#f03)); there is no fixed checker for this gate. |
| Figures | [Lifecycle](arch/00_overview.md#a03) and [controller](details/01_flows_and_checkpoints.md#f04) explain shared operation. |
| Status | Flow modeled and historical wish runs recorded, including [wish-0007](review/01_wish_status.md#v06); these records do not establish a complete running workflow application. |

**`xdesign.designedSimpleFlow : DesignedSimpleFlow` — gated four-stage usage**

| Item | Current design |
| --- | --- |
| Purpose and stages | The same four stages as SimpleFlow, with a designed checkpoint at each stage exit ([D01](details/simple_flow/01_run_design.md#d01)). |
| Exit checkpoints | Syntactic `simpleRequirementsAccepted`, `simpleDesignAccepted`, `simpleImplementationAccepted`, `simpleRecordAccepted` (script judgment); terminal semantic `verificationAccepted` (agent or human judgment, [D05](details/simple_flow/02_gates_and_rules.md#d05)). |
| Run and factory | `SimpleFlowRun` and `SimpleFlowRunFactory::createRun` are modeled in [D02–D03](details/simple_flow/01_run_design.md#d02) and realized by `xdesign::simple_flow` ([P08](prj_manage/03_simple_flow_execution_record.md#p08)). |
| Rules and checks | All 18 fixed rules, inputs, diagnostics, and report fields are in [D06](details/simple_flow/02_gates_and_rules.md#d06); the four implemented command contracts are in [D07](details/simple_flow/02_gates_and_rules.md#d07). |
| Figures | [Designed plan](details/simple_flow/01_run_design.md#d01), [structure](details/simple_flow/03_structure_graphs.md#d08), and the [nine-question coverage index](details/simple_flow/04_sysml_diagrams.md#d12), whose figures sit in stage-owned sections. |
| Status | Detailed design and generated figures exist; the `xdesign::simple_flow` module, factory, checkers, and local run execute ([P08](prj_manage/03_simple_flow_execution_record.md#p08), [P12](prj_manage/04_simple_flow_review_fix_record.md#p12)). Generic controller integration and durable persistence remain open. |

**`xdesign.designedWishVerifyFlow : DesignedWishVerifyFlow` — gated two-stage judged usage**

| Item | Current design |
| --- | --- |
| Purpose and stages | Wish → verify for a judged artifact, with a designed semantic gate at each stage exit ([D57](details/wish_verify_flow/01_run_design.md#d57)). |
| Exit checkpoints | Two semantic gates, both judged by an agent or a human: `wishAccepted` at the wish stage (this design's addition) and the inherited terminal `outcomeAccepted` at verify ([D61](details/wish_verify_flow/02_gates_and_judgment.md#d61)). |
| Run and factory | `WishVerifyFlowRun` and `WishVerifyFlowRunFactory::createRun` are modeled in [D58–D59](details/wish_verify_flow/01_run_design.md#d58) and realized by `xdesign::wish_verify` ([U73](api/wish_verify_flow/00_overview.md#u73)). |
| Rules and checks | No syntactic checkpoint and no checker command exists for this flow; the judgment contract — who may judge, what a judgment must carry, what never becomes a pass — is [D62](details/wish_verify_flow/02_gates_and_judgment.md#d62). |
| Figures | [Nine-question coverage index](details/wish_verify_flow/03_sysml_diagrams.md#d64), whose figures sit in the sections that answer each question. |
| Status | Designed and implemented slice per the [wish-verify plan](prj_manage/09_wish_verify_flow_implementation_plan.md#p25); executed cases WV-01–WV-10 are recorded in [P28](prj_manage/09_wish_verify_flow_implementation_plan.md#p28) as they run. Generic controller integration and durable persistence remain open. |

**`xdesign.wishVerifyFlow : WishVerifyFlow` — judged artifact**

| Item | Current design |
| --- | --- |
| Purpose and stages | Wish → verify for an artifact whose outcome requires judgment, without separate requirements or implementation stages ([F03](details/01_flows_and_checkpoints.md#f03)). |
| Exit checkpoints | Terminal semantic `outcomeAccepted`, judged by an agent or human. |
| Run and factory | Generic `FlowExecution` records and `Controller::receiveWish`; no flow-specific run class or factory. |
| Rules and checks | The two `outcomeAccepted` criteria are declared in `mbse/model/architecture.sysml` (`WishVerifyFlow`); [F03](details/01_flows_and_checkpoints.md#f03) explains the flow context. No fixed checker is declared for this semantic gate. |
| Figures | [Two-stage structure](details/01_flows_and_checkpoints.md#f03) and [wish-0005 journey](review/01_wish_status.md#v03). |
| Status | Flow modeled; wish-0005 has a delivered, evidence-backed historical run. |

**`xdesign.designFlow : DesignFlow` — SysML design deliverable**

| Item | Current design |
| --- | --- |
| Purpose and stages | Wish → verify for a SysML model and its documentation, with no implementation stage ([F03](details/01_flows_and_checkpoints.md#f03)). |
| Exit checkpoints | `sysmlDeliverableAccepted` is syntactic and names the pinned compiler; inherited `outcomeAccepted` is semantic and needs an agent or human judgment. |
| Run and factory | Generic `FlowExecution` records and `Controller::receiveWish`; no DesignFlow-specific run class or factory. |
| Rules and checks | Pinned compiler `parse`/`check` results establish supported SysML checks ([T02](details/03_toolchain.md#t02)); the inherited semantic outcome criteria are declared in `mbse/model/architecture.sysml` (`WishVerifyFlow`). |
| Figures | The [wish-0006 design journey](review/01_wish_status.md#v05) shows one recorded run; no dedicated DesignFlow definition figure is embedded. |
| Status | Flow modeled and used for design deliveries; the wish-0008 record and its current assessment are in [V07](review/01_wish_status.md#v07). |

**`xdesign.nestedFlowExample : NestedFlowExample` — subflow example**

The outer four-stage path contains a design-stage subflow. It is modeled to show
that the child must finish before the parent stage may advance ([F05](details/01_flows_and_checkpoints.md#f05)); no executable run is claimed.

Each wish has its own `mbse/wishes/wish-<id>.sysml`. That file records the selected flow,
stage visits, revisions, assessments, and evidence URIs. It may reference products in the
project model, but does not hold their only copy ([E01](details/02_records_and_evidence.md#e01)).
For another flow, add a row here and links to its own design and verification documents;
the document structure does not presume only SimpleFlow.

<a id="a11"></a>

## A11 Model-to-document map

| SysML source | Main document sections | Figure or record role |
| --- | --- | --- |
| `mbse/model/common.sysml` | [checkpoint types](details/01_flows_and_checkpoints.md#f02), [DocSection](details/05_documentation_contract.md#d27), [DocFigure](details/05_documentation_contract.md#d28) | Shared metadata and statuses |
| `mbse/model/requirements.sysml`, `traceability.sysml` | [inventory](requirements/01_baseline.md#r01), [trace](requirements/01_baseline.md#r02) | Requirements and their model relationships |
| `mbse/model/architecture.sysml`, `workflow.sysml`, `composition.sysml` | [composition](arch/01_system_composition.md#b01), [flows](details/01_flows_and_checkpoints.md#f03), [records](details/02_records_and_evidence.md#e01) | Structure and lifecycle figures |
| `mbse/model/detailedDesign.sysml`, `verification.sysml` | [run design](details/simple_flow/01_run_design.md#d01), [file plan and cases](prj_manage/06_file_plan_and_verification.md#d53), [requirement view](requirements/01_baseline.md#d15), [package view](arch/02_design.md#d21), [viewpoint contract](details/05_documentation_contract.md#d29) | Declared viewpoints and stage-owned figures with planned verification |
| `mbse/model/detailedDesign.sysml` (wishVerify elements), `verification.sysml` (wish-verify cases) | [run design](details/wish_verify_flow/01_run_design.md#d57), [gates and judgment](details/wish_verify_flow/02_gates_and_judgment.md#d61), [module design](details/wish_verify_flow/04_module_design.md#d65), [view coverage](details/wish_verify_flow/03_sysml_diagrams.md#d64), [wish-verify plan](prj_manage/09_wish_verify_flow_implementation_plan.md#p25) | Declared wish-verify viewpoints and figures with planned verification |
| `mbse/model/uml.sysml` (`crate::wishVerify` and realization dependencies) | [wish-verify Rust UML design](details/wish_verify_flow/05_rust_uml_design.md#d75), [wish-verify API layer](api/wish_verify_flow/00_overview.md#u73) | wishVerify module layout, three flow-level functions, and the realization mapping to the wish-verify design |
| `mbse/model/uml.sysml` | [Rust UML design](details/simple_flow/06_rust_uml_design.md#d44) | xdesign crate boundary, core + simple_flow layer layout, Rust types, gate interaction, run states, and realization mapping; the [API inventory and boundary](api/00_overview.md#u65) and [crate structure](api/01_struct.md#u68) expand the design |
| `mbse/wishes/wish-0001.sysml` through `wish-0003.sysml` | [wish status](review/01_wish_status.md#v02) | Independent per-wish trace and assessment evidence |
| `mbse/wishes/wish-0004.sysml` through `wish-0009.sysml` | [wish-0004](review/01_wish_status.md#v04), [wish-0005](review/01_wish_status.md#v03), [wish-0006](review/01_wish_status.md#v05), [wish-0007](review/01_wish_status.md#v06), [wish-0008](review/01_wish_status.md#v07), [wish-0009](review/01_wish_status.md#v16) | Each independent trace links to its own review section |

The model's `DocSection` and `DocFigure` metadata give the forward direction from an
element or view to a document section and figure. This table is a human reverse index;
it does not replace auditing each annotation target ([D27](details/05_documentation_contract.md#d27),
[D28](details/05_documentation_contract.md#d28)).

<a id="a12"></a>

## A12 Status, baselines, and evidence

The requirements, allocated design, and delivered product are distinct engineering
baselines. A document's presence does not freeze one. [D30](details/05_documentation_contract.md#d30)
defines the review and change record expected before claiming a baseline. The current wish
status and open implementation items are maintained in [V02](review/01_wish_status.md#v02)
and [V04](review/01_wish_status.md#v04); wish-0008's own outcome must be read from
[V07](review/01_wish_status.md#v07) and `mbse/wishes/wish-0008.sysml`.

| Baseline or artifact | Current state | Review or evidence |
| --- | --- | --- |
| Functional requirements | Working; no SRR freeze | [R13](requirements/01_baseline.md#r13), [V09](review/02_checklist.md#v09) |
| Allocated architecture and module design | Working; no PDR or CDR freeze | [B28](arch/02_design.md#b28), [D40](details/simple_flow/05_module_design.md#d40) |
| Product | Four-stage SimpleFlow slice and two-stage wish-verify slice executed; complete workflow application and VModel gate commands incomplete; no FRR | [P08](prj_manage/03_simple_flow_execution_record.md#p08), [P28](prj_manage/09_wish_verify_flow_implementation_plan.md#p28), [V08](review/02_checklist.md#v08) |
| SysML model and rendered figures | Revision-4 parse/check, render, audit, and figure verification passed for their stated scopes | [V07](review/01_wish_status.md#v07) |

Run the pinned compiler and inspect its diagnostics as described in [T02](details/03_toolchain.md#t02).
The renderer writes model figures in each document directory's `assets/` and the site builder writes its
output under `cache/docs-site/` ([T03](details/03_toolchain.md#t03)). Review evidence for
each wish stays under `cache/evidence/<wish>/`, with the assessed artifact revision recorded
in that wish's model. Wish-0007 established the nine-figure presentation review
([V06](review/01_wish_status.md#v06)); wish-0008 reran final-asset checks after its
revision-2 stage-document and figure relocation changes, then reran them for revision-3
directory overviews and path migration, then for revision-4 local asset placement
([V07](review/01_wish_status.md#v07)). Later figure changes need
their own corresponding evidence.

<a id="a13"></a>

## A13 Open documentation and design work

| Open item | Why it remains open | Responsible role |
| --- | --- | --- |
| Complete per-requirement method, phase, and acceptance criteria | An SRR cannot freeze an incomplete verification plan ([R11](requirements/01_baseline.md#r11)) | Requirements owner, unassigned |
| Choose and justify architecture and interface baselines | PDR lacks concept comparison and the draft ICD has open contracts ([B28](arch/02_design.md#b28), [I09](icd/01_interfaces.md#i09)) | Architect and integrator, unassigned |
| Integrate the workflow application and VModel commands | The four-stage SimpleFlow run and commands execute, but generic controller integration, durable persistence, and three VModel gate commands remain open ([D40](details/simple_flow/05_module_design.md#d40), [P12](prj_manage/04_simple_flow_review_fix_record.md#p12)) | Module owner, unassigned |
| Hold SRR, PDR, CDR, and FRR with revision-bound evidence | Stage documents and individual wish assessments do not constitute those gate decisions ([V14](review/02_checklist.md#v14)) | Review authorities, unassigned |
