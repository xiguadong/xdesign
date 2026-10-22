# SimpleFlow design



SimpleFlow is the four-stage requirements -> design -> implement -> verify slice. Read the [definition](definition.md) for its stages, gates, rules, and checker commands; the [implementation](implementation.md) for its run, factory, adapter, Rust design, and API mapping; and the [views](views.md) for its structure and coverage. The common [flow model](../../flow.md), [lifecycle](../../lifecycle.md), [evidence](../../evidence-and-revisions.md), and [adapter](../../adapters.md) semantics are not repeated here.



<a id="d43"></a>

## Module documents and boundaries

The developed SimpleFlow design refines the generic flow and record contracts into a run,
factory, checkpoint specializations, and guarded advancement. It describes the intended
automation. The specialized run and four checker commands now execute as a Rust
library and local CLI; the complete workflow application remains open. The
[requirements baseline](../../../requirements/requirements.md#r01)
states the obligations, the [architecture design](../../../architecture/system-behavior.md#b13)
allocates the logical parts, and the [draft ICD](../../../interfaces/contracts.md#i01)
records their modeled exchanges.

| Document | Design responsibility | Evidence limit |
| --- | --- | --- |
| [Run design](definition.md#d01) | Stage plan, run type, factory, and advancement | Rust core and example run execute this slice; full workflow is separate |
| [Gates and rules](definition.md#d05) | Checkpoint kinds, criteria, rule inputs, diagnostics, and command reports | Four fixed commands execute; semantic adequacy remains a supplied judgment |
| [Structure graphs](views.md#d08) | Timeline, logical tree, and interface overview figures | Model projections; no runtime endpoint proof |
| [Nine-view coverage](views.md#d12) | Applicability reasons and links to views embedded by stage | Navigation and coverage record, not a figure gallery |
| [Module design](implementation.md#d32) | Types, behavior, authorization, interfaces, errors, performance, tests, and CDR gaps | Working design; no frozen module baseline |
| [Rust UML design](implementation.md#d44) | `xdesign::core` + `xdesign::simple_flow` modules, Rust types, gate interaction, run states, and view applicability | Core code exists; the API design set records its design boundary and implementation projection |

The two requirements views are embedded in the
[requirements specification](../../../verification/requirements.md#d15), the package view
in the [architecture design](../../../architecture/system-composition.md#d21), and six module views beside
their detailed explanations in [D13–D20](views.md#d13). The Rust UML views
are SysML projections of `XDesignRustUml` embedded in
[D45–D48](implementation.md#d45), selected per the
[documentation contract's applicability rule](../../../interfaces/documentation-contract.md#u09). The
[stage checklist](../../../verification/review-criteria.md#v11) states what would still be needed
for a CDR decision.

The [implementation file plan and acceptance cases](../../../records/plans/simple-flow-file-plan.md#d53)
are managed with the plan and execution records in `records/`.
