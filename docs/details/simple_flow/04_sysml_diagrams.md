# SimpleFlow view coverage and stage handoffs

The nine selected SysML questions form a coverage index for the SimpleFlow design.
Their figures and explanations are embedded in the current requirements, architecture,
and module-design sections that answer each question. D12 records applicability and the
engineering stage handoffs; the linked D13–D21 sections carry the actual figures.

<a id="d12"></a>

## D12 Scope, sources, and stage handoffs

`02_modeling_and_design.pdf`, page 14, names the nine SysML diagram types; pages 15–23
explain them individually. Page 24 explains the corresponding SysML v2 approach: select
elements from one semantic model through views and viewpoints. This design keeps the nine
familiar question views and expresses their content with SysML v2 definitions, usages,
relationships, constraints, and behavior. Each view has a `DocumentationViewpoint` usage
whose `concernText` and stakeholder role explain why the view selects its elements. The
viewpoint's `subject` binds it to the native view; the view's `DocFigure` locates the
rendered asset and its `DocSection` locates the explanation
([D29](../05_documentation_contract.md#d29)). These are views of xdesign's SimpleFlow;
the lesson's CanSat example supplies the explanations of the diagram types.

| Question | Viewpoint → native view in `XDesignDetailedDesign` | Applicability and reason for SimpleFlow | Owning stage | Figure and explanation | Lesson page |
| --- | --- | --- | --- | --- | --- |
| What types make up the flow and its run? | `blockDefinitionConcern` → `blockDefinitionDiagram` | Required to show run and gate type ownership | Detailed design | [D13](05_module_design.md#d13) | 15 |
| How do a run's internal parts and ports connect? | `internalBlockConcern` → `internalBlockDiagram` | Required to distinguish the checker and judge interfaces | Detailed design | [D14](05_module_design.md#d14) | 16 |
| Which requirements are satisfied and how will they be verified? | `requirementsConcern` → `requirementsDiagram` | Required to show selected requirement allocation and planned review | Requirements | [D15](../../requirements/01_baseline.md#d15) | 17 |
| Which bound values authorize completion? | `parametricConcern` → `parametricDiagram` | Required to expose current-revision gate authorization | Detailed design | [D16](05_module_design.md#d16) | 18 |
| Who uses the flow and for which goals? | `useCaseConcern` → `useCaseDiagram` | Required to show requester and reviewer goals at the system boundary | Requirements | [D17](../../requirements/01_baseline.md#d17) | 19 |
| In which order are checkpoint rules applied? | `activityConcern` → `activityDiagram` | Required to distinguish syntactic and semantic branch conditions | Detailed design | [D18](05_module_design.md#d18) | 20 |
| Which participants interact in which order? | `sequenceConcern` → `sequenceDiagram` | Required to show normal run message order | Detailed design | [D19](05_module_design.md#d19) | 21 |
| In which state is a run, and what changes it? | `stateMachineConcern` → `stateMachineDiagram` | Required to show blocking, recovery, and completion conditions | Detailed design | [D20](05_module_design.md#d20) | 22 |
| How is the model organized and which packages depend on others? | `packageConcern` → `packageDiagram` | Required to show project-model dependency direction used by this flow | Architecture | [D21](../../arch/02_design.md#d21) | 23 |

“Required” is a scope decision for this SimpleFlow design: wish-0006 requested nine-view
coverage, and each row states the design question that makes that view applicable. The
package view selects project-model packages because this flow depends on their boundaries;
it does not depict a flow-specific deployment. Each concern has a declared native view
and a figure in the linked owning-stage section. A future flow records each concern as required,
recommended, or inapplicable with a reason in its own design, as specified in
[D29](../05_documentation_contract.md#d29).

The stages refine one model rather than producing unrelated drawings:

| Stage | Questions and model work | Target handoff, not an accepted baseline |
| --- | --- | --- |
| Requirements | Use cases establish participants, goals, and the system boundary. Requirements state outcomes, rationales, and verification methods; the requirements view retains their traceability. | An SRR requirements baseline that is measurable, verifiable, and traceable. |
| Architecture and concept selection | System composition sets boundaries and responsibilities; package dependencies organize the model; interface and runtime contracts identify decisions that need refinement. | A PDR concept choice with reasons, quantitative analysis, and risk evidence. |
| Detailed design | Refine types, internal ports, activities, interactions, states, and bound authorization values until the run and its checkpoint operations can be implemented. Link the design to planned verification and documentation. | A CDR design baseline sufficient for implementation, integration, and testing. |
| Verification and feedback | Execute the applicable checks and review the evidence against the requirements. A failing or stale assessment prevents advancement and can open rework. | Evidence and a recorded judgment; changes return to the affected requirements and design. |

These are the lesson's engineering stage handoffs. The project's four-stage `SimpleFlow`
uses its own declared checkpoints ([D05](02_gates_and_rules.md#d05)); this document does
not add SRR, PDR, or CDR as executable gates to that flow. wish-0006 remains a design
delivery; wish-0007 changes the diagram presentation tooling and its documentation. The
four SimpleFlow checker commands and local Rust run now execute
([D07](02_gates_and_rules.md#d07), [P08](../../prj_manage/03_simple_flow_execution_record.md#p08));
the complete workflow application and VModel gate commands remain open.

The source PDFs are local reference material, identified here by their original paths and
one-based PDF page numbers:

| Source | Original local path | Pages used |
| --- | --- | --- |
| Overview | `/Users/xigua/life/SE/docs/lesson/slides/pdf/00_overview.pdf` | 8–10: V-model stages; 12: SRR/PDR/CDR and evidence; 13: feedback and rework. |
| Requirements | `/Users/xigua/life/SE/docs/lesson/slides/pdf/01_requirements.pdf` | 2–3: requirements baseline and outcomes versus implementation decisions; 6–7: measurable requirements, verification, and traceability. |
| Modeling and design | `/Users/xigua/life/SE/docs/lesson/slides/pdf/02_modeling_and_design.pdf` | 4: models deepen across stages; 9–12: architecture, concept selection, detailed design, and verification; 14–24: nine diagrams and the v2 view approach; 56–58: question-driven selection. |

The source material describes an example and teaching guidance. The simple-flow model and
its gate criteria remain the project's own design decisions.
