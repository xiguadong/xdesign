# Engineering design review checklist

> Project: xdesign | Revision: 0.1 | Status: working checklist | Audience: requesters,
> architects, module owners, integrators, and reviewers. This is the stage review plan
> and current gap record, not evidence that SRR, PDR, CDR, or FRR passed.

<a id="v08"></a>

## V08 Gate review position

Each gate requires a named judge, date, assessed artifact revision, evidence, and an
explicit outcome. The [wish status](01_wish_status.md#v02) records outcomes for individual
wishes; it does not freeze the project's functional, allocated, or product baseline.

| Gate | Decision question | Stage documents and views | Current position |
| --- | --- | --- | --- |
| SRR | Are requirements complete, traceable, and verifiable? | [Requirements](../requirements/01_baseline.md#r01), requirement trace and use-case views, verification methods | **Not held.** Requirement model and trace exist; a full requirements review record and acceptance of the functional baseline are missing. |
| PDR | Can the architecture and chosen concept meet the requirements? | [Architecture design](../arch/02_design.md#b05), composition, trade studies/ADRs, interface inventory, view applicability, risks and budgets | **Not held.** Architecture structure is modeled; concept comparison, quantified analysis, accepted ADRs, and PDR package are missing. |
| CDR | Is detailed design mature for build and integration? | [SimpleFlow module design](../details/simple_flow/05_module_design.md#d32), [gate rules](../details/simple_flow/02_gates_and_rules.md#d06), [ICD](../icd/01_interfaces.md#i01), test points, implementation maturity | **Not held.** SimpleFlow interfaces and rules are designed; endpoint implementations, field-level contracts, cases, and CDR package are missing. |
| FRR | Do verification and validation support delivery with accepted residual risk? | Requirements trace, executed tests/analysis/inspection/demonstration, interface results, risk and waiver record | **Not held.** A working workflow application and planned gate commands are absent; per-wish design assessments are narrower than a release review. |

<a id="v09"></a>

## V09 Requirements and SRR checks

| Check | Current evidence or gap |
| --- | --- |
| Each requirement is singular, unambiguous, feasible, and objectively verifiable | [R01](../requirements/01_baseline.md#r01) lists project requirements and rationales; formal quality review against the template's condition/action/target/method/phase fields is pending. |
| The set is complete, consistent, and free of duplicate or conflicting targets | Model trace is in [R02](../requirements/01_baseline.md#r02); set-level review and approved scope are pending. |
| Every requirement names verification method, phase, and acceptance criterion | Planned verification cases exist for some requirements; a complete requirement-to-case acceptance table is pending. |
| Functional baseline and change control are recorded | No SRR result or frozen functional baseline is recorded. |

<a id="v10"></a>

## V10 Architecture and PDR checks

| Check | Current evidence or gap |
| --- | --- |
| Context, logical structure, runtime, deployment, data, and security concerns are explained or reasoned not applicable | [B05–B17](../arch/02_design.md#b05) organize the architecture-stage concerns; complete deployment, data, and security allocations remain to be reviewed. |
| Function → Concept → Form, variables, values, and derived requirements are separated | The model gives parts and stages; a recorded concept comparison and allocation decision are pending. |
| Candidate selection, quantified quality targets, budgets, risks, and trade-offs support the architecture | No accepted [ADR](../adr/00_overview.md#j01) or complete PDR trade and budget package is recorded. |
| Interface baseline has IDs, owners, and review status | [I01](../icd/01_interfaces.md#i01) inventories four modeled interfaces; [I09](../icd/01_interfaces.md#i09) identifies unresolved field-level choices. No interface baseline is frozen. |
| Required SysML views are embedded where their design claims are explained | [D12](../details/simple_flow/04_sysml_diagrams.md#d12) maps the nine views to requirements, architecture, and module sections; their applicability and stage placement should be assessed at PDR/CDR. |

<a id="v11"></a>

## V11 Detailed design and CDR checks

| Check | Current evidence or gap |
| --- | --- |
| Module responsibility, requirements, classes, and data structures are identified | [D32–D33](../details/simple_flow/05_module_design.md#d32) organize the module boundary and types; each realized requirement and data contract still needs implementation trace. |
| Scenarios, states, control flow, algorithms, and failure paths are specific enough to implement | [D34–D37](../details/simple_flow/05_module_design.md#d34) and [D11](../details/simple_flow/02_gates_and_rules.md#d11) show modeled behavior; command implementation and runtime error behavior remain open. |
| Interface implementation covers fields, timeout, retry, and idempotency | [I02–I05](../icd/01_interfaces.md#i02) records typed fields; transport, timing, error, and compatibility policies are open in [I09](../icd/01_interfaces.md#i09). |
| Performance budgets, logs, boundary tests, and requirement-to-case links are recorded | No complete CDR evidence package or executed tests are recorded. |
| Design baseline, waivers, and residual risks are reviewed | No CDR result or frozen detailed design baseline is recorded. |

<a id="v12"></a>

## V12 Documentation and SysML consistency checks

| Check | Method | Current limit |
| --- | --- | --- |
| Directory overviews and numbering, stable `DocSection` and `DocFigure` targets, valid assets, local figure blocks | `python3 tools/check_documentation.py --json` ([D31](../details/05_documentation_contract.md#d31)) | Checks directory layout, paths, anchors, embeds, and fields; it does not judge design meaning. Rerun after documentation or model changes. |
| SysML syntax, names, and supported constraints | Bundled compiler `parse` and `check` over `mbse/model` and `mbse/wishes`, then independent model check ([T02](../details/03_toolchain.md#t02)) | Compiler leaves some model constraint bodies unevaluated; reports must be read. |
| Rendered view identity and article-width readability | `tools/render_docs.py`, `tools/verify_diagrams.py`, and asset review ([V06](01_wish_status.md#v06)) | Prior visual evidence binds to exact SVG revisions; rerendered assets require fresh inspection. |
| Links, anchor navigation, and reader paths | Strict `tools/docs.py build`, documentation audit, and human navigation | A build cannot determine whether the document set follows each stage template or whether prose is correct. |
| Requirement and document meaning | Agent or human reviews the actual claims against the current model and wishes | [wish-0008](01_wish_status.md#v07) has separate revision-2 stage-content, revision-3 directory-layout, and revision-4 asset-placement assessments; later changes require a new assessment. |

Every figure needs the question, source view, companions, status, reading note, and limits
in its own subsection ([D28](../details/05_documentation_contract.md#d28)). Model
projections cannot replace an ICD field contract or an engineering judgment.

<a id="v13"></a>

## V13 Syntactic and semantic evidence to collect

| Evidence | Judge | Pass condition and record |
| --- | --- | --- |
| SysML parse/check and model-only check | Script/compiler | No parse errors, unresolved names, collisions, or violations; preserve all unevaluated-rule diagnostics in the gate record. |
| Documentation and figure audit | Script plus visual reviewer | All locators and blocks resolve; required figures are readable and match their selected source. Preserve final asset hashes. |
| Requirements quality and trace completeness | Requirements reviewer | All applicable [V09](#v09) checks are answered and an SRR decision records evidence and conditions. |
| Architecture adequacy and interface allocation | Architect and integrator | [V10](#v10) is reviewed with concept, risk, interface, and PDR evidence. |
| Build maturity, interface contracts, and test points | Module owner and verifier | [V11](#v11) is reviewed with implementation, CDR, and requirement-to-case evidence. |
| Release verification and validation | Authorized reviewers | Executed evidence covers acceptance criteria, interfaces, residual risks, and FRR disposition. |

The scripted checks and semantic review answer different questions. A compiler success
does not mean a checkpoint's design rationale, stakeholder outcome, or interface behavior
is accepted ([V01](01_wish_status.md#v01)).

<a id="v14"></a>

## V14 Review record and unresolved conditions

| Review | Judge | Date | Revision | Evidence | Outcome |
| --- | --- | --- | --- | --- | --- |
| SRR | Unassigned | Not held | — | [V09](#v09) gaps | Open |
| PDR | Unassigned | Not held | — | [V10](#v10) gaps and [ADR index](../adr/00_overview.md#j01) | Open |
| CDR | Unassigned | Not held | — | [V11](#v11) gaps and [ICD](../icd/01_interfaces.md#i09) | Open |
| FRR | Unassigned | Not held | — | [V08](#v08) release prerequisites | Open |

Individual historical wish outcomes remain in [V02–V07](01_wish_status.md#v02).
This table needs an assessed revision, named judge, date, evidence files, conditions,
and disposition before any stage review can be reported as passed.
