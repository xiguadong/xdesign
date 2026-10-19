# SimpleFlow detailed design overview

> Project: xdesign | Status: working module design with an executable four-stage
> slice | Audience: module implementers, integrators, verifiers, and reviewers.
> No CDR or full workflow acceptance is claimed.

<a id="d43"></a>

## D43 Module documents and boundaries

The developed SimpleFlow design refines the generic flow and record contracts into a run,
factory, checkpoint specializations, and guarded advancement. It describes the intended
automation. The specialized run and four checker commands now execute as a Rust
library and local CLI; the complete workflow application remains open. The
[requirements baseline](../../requirements/01_baseline.md#r01)
states the obligations, the [architecture design](../../arch/02_design.md#b13)
allocates the logical parts, and the [draft ICD](../../icd/01_interfaces.md#i01)
records their modeled exchanges.

| Document | Design responsibility | Evidence limit |
| --- | --- | --- |
| [Run design](01_run_design.md#d01) | Stage plan, run type, factory, and advancement | Rust core and example run execute this slice; full workflow is separate |
| [Gates and rules](02_gates_and_rules.md#d05) | Checkpoint kinds, criteria, rule inputs, diagnostics, and command reports | Four fixed commands execute; semantic adequacy remains a supplied judgment |
| [Structure graphs](03_structure_graphs.md#d08) | Timeline, logical tree, and interface overview figures | Model projections; no runtime endpoint proof |
| [Nine-view coverage](04_sysml_diagrams.md#d12) | Applicability reasons and links to views embedded by stage | Navigation and coverage record, not a figure gallery |
| [Module design](05_module_design.md#d32) | Types, behavior, authorization, interfaces, errors, performance, tests, and CDR gaps | Working design; no frozen module baseline |
| [Rust UML design](06_rust_uml_design.md#d44) | Crate modules, Rust types, gate interaction, run states, and UML view applicability for `simpflow` | Core code exists; the API reference records its implemented boundary |
| [File plan and verification](07_file_plan_and_verification.md#d53) | Source, checker and test files, with executable gate, recovery and file-conformance cases | Case definitions stay separate from executed results in [P08](../../prj_manage/03_simple_flow_execution_record.md#p08) |

The two requirements views are embedded in the
[requirements specification](../../requirements/01_baseline.md#d15), the package view
in the [architecture design](../../arch/02_design.md#d21), and six module views beside
their detailed explanations in [D13–D20](05_module_design.md#d13). The Rust UML views
are SysML projections of `SimpleFlowRustUml` embedded in
[D45–D48](06_rust_uml_design.md#d45), selected per the
[UML guide](../../uml/01_c4_and_uml_guide.md#u06). The
[stage checklist](../../review/02_checklist.md#v11) states what would still be needed
for a CDR decision.
