# Review criteria

> Project: xdesign | Revision: 0.1 | Status: working checklist | Audience: requesters,
> architects, module owners, integrators, and reviewers. This is the stage review plan
> and current gap record, not evidence that SRR, PDR, CDR, or FRR passed.

<a id="v08"></a>

## 1. Gate review position

Each gate requires a named judge, date, assessed artifact revision, evidence, and an
explicit outcome. The [wish status](../records/wishes/wish-history.md#v02) records outcomes for individual
wishes; it does not freeze the project's functional, allocated, or product baseline.

| Gate | Decision question | Stage documents and views | Current position |
| --- | --- | --- | --- |
| SRR | Are requirements complete, traceable, and verifiable? | [Requirements](../requirements/requirements.md#r01), requirement trace and use-case views, verification methods | **Not held.** Requirement model and trace exist; a full requirements review record and acceptance of the functional baseline are missing. |
| PDR | Can the architecture and chosen concept meet the requirements? | [Architecture design](../architecture/system-context.md#b05), composition, trade studies/ADRs, interface inventory, view applicability, risks and budgets | **Not held.** Architecture structure is modeled; concept comparison, quantified analysis, accepted ADRs, and PDR package are missing. |
| CDR | Is detailed design mature for build and integration? | [SimpleFlow module design](../design/flows/simple-flow/implementation.md#d32), [gate rules](../design/flows/simple-flow/definition.md#d06), [ICD](../interfaces/contracts.md#i01), test points, implementation maturity | **Not held.** The four SimpleFlow commands and local run execute; field-level contracts, full verification coverage, and a CDR package remain incomplete. |
| FRR | Do verification and validation support delivery with accepted residual risk? | Requirements trace, executed tests/analysis/inspection/demonstration, interface results, risk and waiver record | **Not held.** The complete workflow application and three VModel gate commands are absent; the executed SimpleFlow slice and per-wish design assessments are narrower than a release review. |

<a id="v09"></a>

## 2. Requirements and SRR checks

| Check | Current evidence or gap |
| --- | --- |
| Each requirement is singular, unambiguous, feasible, and objectively verifiable | [R01](../requirements/requirements.md#r01) lists project requirements and rationales; formal quality review against the condition/action/target/method/phase fields is pending. |
| The set is complete, consistent, and free of duplicate or conflicting targets | Model trace is in [R02](../requirements/requirements.md#r02); set-level review and approved scope are pending. |
| Every requirement names verification method, phase, and acceptance criterion | Planned verification cases exist for some requirements; a complete requirement-to-case acceptance table is pending. |
| Functional baseline and change control are recorded | No SRR result or frozen functional baseline is recorded. |

<a id="v10"></a>

## 3. Architecture and PDR checks

| Check | Current evidence or gap |
| --- | --- |
| Context, logical structure, runtime, deployment, data, and security concerns are explained or reasoned not applicable | [B05–B17](../architecture/system-context.md#b05) organize the architecture-stage concerns; complete deployment, data, and security allocations remain to be reviewed. |
| Function → Concept → Form, variables, values, and derived requirements are separated | The model gives parts and stages; a recorded concept comparison and allocation decision are pending. |
| Candidate selection, quantified quality targets, budgets, risks, and trade-offs support the architecture | No accepted [ADR](../records/status.md) or complete PDR trade and budget package is recorded. |
| Interface baseline has IDs, owners, and review status | [I01](../interfaces/contracts.md#i01) inventories four modeled interfaces; [I09](../interfaces/contracts.md#i09) identifies unresolved field-level choices. No interface baseline is frozen. |
| Required SysML views are embedded where their design claims are explained | [D12](../design/flows/simple-flow/views.md#d12) maps the nine views to requirements, architecture, and module sections; their applicability and stage placement should be assessed at PDR/CDR. |

<a id="v11"></a>

## 4. Detailed design and CDR checks

| Check | Current evidence or gap |
| --- | --- |
| Module responsibility, requirements, classes, and data structures are identified | [D32–D33](../design/flows/simple-flow/implementation.md#d32) organize the module boundary and types; each realized requirement and data contract still needs implementation trace. |
| Scenarios, states, control flow, algorithms, and failure paths are specific enough to implement | [D34–D37](../design/flows/simple-flow/implementation.md#d34) and [D11](../design/flows/simple-flow/definition.md#d11) show modeled behavior; the local commands and recovery tests execute ([P08](../records/executions/simple-flow.md#p08), [P12](../records/reviews/simple-flow-fixes.md#p12)), while full application error behavior remains open. |
| Interface implementation covers fields, timeout, retry, and idempotency | [I02–I05](../interfaces/contracts.md#i02) records typed fields; transport, timing, error, and compatibility policies are open in [I09](../interfaces/contracts.md#i09). |
| Performance budgets, logs, boundary tests, and requirement-to-case links are recorded | SimpleFlow boundary and integration tests are recorded in [P08](../records/executions/simple-flow.md#p08) and [P12](../records/reviews/simple-flow-fixes.md#p12); a complete CDR evidence package, performance budgets, and requirement-to-case matrix are still missing. |
| Design baseline, waivers, and residual risks are reviewed | No CDR result or frozen detailed design baseline is recorded. |

<a id="v12"></a>

## 5. Documentation and SysML consistency checks

| Check | Method | Current limit |
| --- | --- | --- |
| Directory overviews and numbering, stable `DocSection` and `DocFigure` targets, valid assets, local figure blocks | `python3 tools/check_documentation.py --json` ([D31](../interfaces/documentation-contract.md#d31)) | Checks directory layout, paths, anchors, embeds, and fields; it does not judge design meaning. Rerun after documentation or model changes. |
| SysML syntax, names, and supported constraints | Bundled compiler `parse` and `check` over `mbse/model` and `mbse/wishes`, then independent model check ([T02](../architecture/constraints.md#t02)) | Compiler leaves some model constraint bodies unevaluated; reports must be read. |
| Rendered view identity and article-width readability | `tools/render_docs.py`, `tools/verify_diagrams.py`, and asset review ([V06](../records/wishes/wish-history.md#v06)) | Prior visual evidence binds to exact SVG revisions; rerendered assets require fresh inspection. |
| Links, anchor navigation, and reader paths | Strict `tools/docs.py build`, documentation audit, and human navigation | A build cannot determine whether the document set follows the document classes or whether prose is correct. |
| Requirement and document meaning | Agent or human reviews the actual claims against the current model and wishes | [wish-0008](../records/wishes/wish-history.md#v07) has separate revision-2 stage-content, revision-3 directory-layout, and revision-4 asset-placement assessments; later changes require a new assessment. |

Every figure needs the question, source view, companions, status, reading note, and limits
in its own subsection ([D28](../interfaces/documentation-contract.md#d28)). Model
projections cannot replace an ICD field contract or an engineering judgment.

<a id="u12"></a>

### 5.1 Design-view and API-reference checklist

- [ ] each view answers a stated question and is embedded in the subsection that
  explains it;
- [ ] required views are present, and every omitted view carries a reason
  ([U09](../interfaces/documentation-contract.md#u09));
- [ ] every figure has the contract figure block with a valid status and stays within
  its section's numbering ([D28](../interfaces/documentation-contract.md#d28),
  [U10](../interfaces/documentation-contract.md#u10));
- [ ] model projections agree with the model elements; no hand-drawn engineering
  diagram substitutes for a projection;
- [ ] names, numbering, and notation are consistent across figures;
- [ ] interface, traceability, and test references are linked;
- [ ] each figure's stated limits do not overclaim implementation;
- [ ] a module's software design has an API design page per
  [U47](../design/api/index.md#u47): complete inventory, one page per API, contracts
  and logic on every behavioral entry.

<a id="v13"></a>

## 6. Syntactic and semantic evidence to collect

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
is accepted ([V01](../records/wishes/wish-history.md#v01)).
