# Detailed design overview

> Status checked: 2026-10-20. The four-stage SimpleFlow slice executes; project-level design reviews remain open.

> Project: xdesign | Status: working design documentation | Audience: model authors,
> implementers, verifiers, and reviewers. The detailed design is not a frozen CDR baseline.

<a id="d42"></a>

## D42 Scope and reading path

This directory explains the workflow contracts shared by the project's flows and points
to the developed design for each flow. The project model remains the structural source;
these documents explain behavior, evidence, tooling, and implementation limits. Start
with the [requirements specification](../requirements/01_baseline.md#r01) and
[architecture design](../arch/02_design.md#b05) before using a module design to build or
verify an implementation.

| Document | Question it answers | Current state |
| --- | --- | --- |
| [Flows and checkpoints](01_flows_and_checkpoints.md#f01) | Which flows, stages, gates, and judge authorities are declared? | Model design; the default V-model's planned checkers lack commands |
| [Records and evidence](02_records_and_evidence.md#e01) | How do visits, revisions, assessments, and evidence retain a wish's history? | Model design; `simpleRecordCheck` executes for the new SimpleFlow JSON run, while historical SysML records remain outside it |
| [Toolchain](03_toolchain.md#t01) | Which local compiler, renderer, audits, and site commands produce evidence? | Some standalone tools implemented; compiler limitations disclosed |
| [Diagram presentation](04_diagram_presentation.md#d22) | How are model views rendered and checked for readability? | Nine primary views have presentation evidence |
| [Documentation contract](05_documentation_contract.md#d27) | How do `DocSection`, `DocFigure`, views, and documents stay linked? | Locator audit implemented; engineering meaning still needs review |
| [SimpleFlow overview](simple_flow/00_overview.md#d43) | Where is the developed SimpleFlow run and module design? | Four-stage crate and commands execute; complete application and VModel commands remain open |
| [WishVerifyFlow overview](wish_verify_flow/00_overview.md#d56) | Where is the developed wish-verify run and module design? | Two-stage judged slice designed and implemented on the shared core engine; executed verification pending in its plan |

Other modeled flows have overview blocks in [A10](../00_index.md#a10). A new developed
flow should get its own numbered subdirectory with `00_overview.md`, then its module
documents. A wish's actual progress and current assessment belong to its independent
SysML trace and the [wish status](../review/01_wish_status.md#v02), not to this design
overview.
