# Documentation and SysML v2 contract

> Project: xdesign | Status: design specification | Applies to: project model, wish
> traces, numbered documents, model-rendered figures, and their checks.

This specification turns the draft patterns in `cache/templates/` into rules for this
repository. It keeps the model's structural claims, the documents' rationale, and the
wish records' evidence connected without treating a picture or a compiler pass as proof of
engineering acceptance. Existing [B04](../arch/01_system_composition.md#b04) explains the
metadata already present; [D22–D26](04_diagram_presentation.md#d22) defines the
presentation pipeline established by wish-0007.

<a id="d27"></a>

## D27 DocSection: model element to stable document section

`XDesignCommon::DocSection` has two strings: `document`, a repository-relative Markdown
path under `docs/`, and `section`, the stable section identifier in that file. Apply it
*inside* the SysML element it locates; the application annotates the enclosing element.
Keep the element's engineering definition in its proper `mbse/model/` module and let the
document explain it. A wish file may point to documentation, but does not become the only
copy of a requirement or design product.

```sysml
part def XDesign {
    @XDesignCommon::DocSection {
        document = "docs/arch/01_system_composition.md";
        section = "B01";
    }
}
```

The target file must exist, contain exactly one explicit anchor whose ID equals the
lowercase form of `section`, and place a heading with that section identifier after the
anchor. IDs are unique across the document set and survive renaming or splitting a file;
update only the `document` path if a section moves. Number files `NN_subject.md` within
each directory as `docs/AGENTS.md` requires. The root
[model-to-document map](../00_index.md#a11) is a readable reverse index, while the
annotations are the precise forward links. A new module or flow gets its own section IDs;
do not reuse D01–D26 or renumber historical anchors.

This metadata locates explanation; it does not assert that the element is implemented,
verified, or current in a wish's run. Those claims come from the model, code, and
revision-bound evidence. The documentation checker in [D31](#d31) audits locator targets,
while the compiler checks SysML syntax and supported name resolution.

<a id="d28"></a>

## D28 DocFigure: declared content to the figure in its section

`XDesignCommon::DocFigure` records the `document` carrying a figure and the repository-
relative `figure` path in that document directory's `assets/` child. Apply it to the
declared view or element
whose selected model content the figure renders. Its `document` must be a valid
`DocSection` target for that same view or element, or an explicitly documented carrying
section when the annotation is on a broader package. Keep `DocSection` immediately before
`DocFigure` on the same element so the audit can check section placement. The figure stays
beside its document in an `assets/` child and repeats the document's numbered filename
prefix: for example,
`docs/details/simple_flow/05_module_design.md` carries
`docs/details/simple_flow/assets/05_block_definition.svg`.

```sysml
view blockDefinitionDiagram : StandardViewDefinitions::GeneralView {
    @XDesignCommon::DocSection {
        document = "docs/details/simple_flow/05_module_design.md";
        section = "D13";
    }
    @XDesignCommon::DocFigure {
        document = "docs/details/simple_flow/05_module_design.md";
        figure = "docs/details/simple_flow/assets/05_block_definition.svg";
    }
    expose XDesignDetailedDesign::SimpleFlowRun;
}
```

Embed the generated figure in the section it explains, using a relative Markdown image
path. A document may also embed a model-rendered figure whose source is tracked in the
[figure provenance table](03_toolchain.md#t03); the same explanatory rule applies to it.
Every embedded model figure has one compact Markdown blockquote immediately after its
image, in the same section. Use `**Figure <section-id>-<ordinal> — <title>.**` followed
in order by the bold field labels `Question:`, `Model view:`, `Companions:`, `Status:`,
`Reading:`, and `Limits:`. The ordinal is local to the section. The following block
belongs in `docs/details/simple_flow/05_module_design.md`, where
`assets/05_block_definition.svg` resolves beside that document:

```markdown
![Run structure](assets/05_block_definition.svg)

> **Figure D13-1 — Block definition.**
> **Question:** What types make up the run?
> **Model view:** `XDesignDetailedDesign::blockDefinitionDiagram`.
> **Companions:** D14 for connected parts; D15 for requirements.
> **Status:** renderable-now; design reviewed.
> **Reading:** The types and ownership edges show the run's structure. Multiplicities
> remain attached to their modeled relationships.
> **Limits:** This figure does not show runtime messages.
```

Each field has a defined meaning:

| Field | Required content |
| --- | --- |
| Title and number | A stable figure label local to the document section |
| Question | The engineering question it answers |
| Model view or projection | Qualified declared view, native diagram target, or an explicit `table-substitute` |
| Companions | Other views, tables, or interface/trace sections needed to read it |
| Status | `renderable-now`, `planned`, `table-substitute`, or `external-tool`; state `designed-only` or `implemented` separately when making a software claim |
| Reading note | Two to four sentences naming the visible elements, relationships, and the point of the figure |
| Limits | What the figure cannot establish or omits |

Render from the model with `tools/render_docs.py`; do not draw an independent engineering
diagram or substitute a screenshot. The diagram and its block must agree with the model's
source identities, relationship directions, guards, and selected values. A short visible
caption may improve legibility, but the full model identity must remain available, as in
[D23](04_diagram_presentation.md#d23). If the tool cannot render the required view, place
a labeled table in the design subsection and mark it `table-substitute`; do not call it a
generated figure. A changed final SVG needs a matching preview and renewed visual review
before any carried acceptance claim is reused ([D24](04_diagram_presentation.md#d24)).

<a id="d29"></a>

## D29 Concern, viewpoint, view, and document

The design starts with a stakeholder concern and a specific question. A viewpoint names
the concern and stakeholder; the accompanying document defines the selection and
presentation method and review criteria for that question. A SysML v2 view selects actual
elements through `expose` and, where applicable, a `render` request. The rendered figure
is a projection of that selection, and `DocSection`/`DocFigure` connect the view to its
explanation. The document explains why the selection matters and what it cannot prove.
This chain applies separately to each flow and each design concern.

| Layer | Decision owned there | Example in the current project |
| --- | --- | --- |
| Concern and viewpoint | Question, stakeholder, selection rule, notation, companions, review criterion | `XDesignDetailedDesign::DocumentationViewpoint` and `blockDefinitionConcern`, plus the nine questions in [D12](simple_flow/04_sysml_diagrams.md#d12) and presentation rules in [D23](04_diagram_presentation.md#d23) |
| Model view | Actual exposed elements and requested rendering | `XDesignDetailedDesign::blockDefinitionDiagram` and its eight exposed definitions |
| Figure | Visible projection with source references | `05_block_definition.svg` |
| Document section | Interpretation, limits, and related contracts | [D13](simple_flow/05_module_design.md#d13) |
| Wish assessment | Whether an artifact revision satisfies a criterion | wish-0007's final presentation review in [V06](../review/01_wish_status.md#v06) |

The current model declares `DocumentationViewpoint` with `concernText` and
`stakeholderRole`. Nine viewpoint usages, from `blockDefinitionConcern` through
`packageConcern`, each bind a `subject` view in `XDesignDetailedDesign` and carry a
`DocSection` link to its design subsection. Each subject view is typed by
`StandardViewDefinitions`, selects native model elements, carries
`DiagramPresentation.question`, and has its own `DocSection` and `DocFigure` links. The
viewpoint's `concernText` equals its subject view's presentation question in the current
model. Maintain that equality when either changes; the compiler's successful name
resolution does not evaluate this project-specific text equality or judge whether the
selection answers the concern. The nine viewpoint usages are design metadata for the
existing figures, not evidence that the workflow application executes their review.

Use the familiar nine SysML diagram questions as a **coverage checklist**, not as nine
unrelated drawings: block definition, internal block, package, activity, sequence, state
machine, use case, requirements, and parametric. For each applicable design or flow, record
`required`, `recommended`, or `not applicable` with a reason; required views are declared
and embedded where they answer their question. Additional context, deployment, data,
security, decision, or risk views are chosen by the concern. The [D12 coverage index](simple_flow/04_sysml_diagrams.md#d12)
links the nine SimpleFlow views to their requirements, architecture, and module-design
sections; their figures are embedded in those sections.

<a id="d30"></a>

## D30 Document roles, baselines, and per-flow extension

The [index](../00_index.md#a09) lists the documents currently carried by this project.
The draft `cache/templates/` set supplied the stage topics now used by the requirements
specification, architecture design, SimpleFlow module design, ICD, ADR index, and review
checklist. The template files are not a frozen baseline and do not replace the numbered
project paths or stable section anchors.

| Baseline role | Required content before claiming it | Review point |
| --- | --- | --- |
| Functional | Requirements with rationale, measurable acceptance conditions, method and phase, and trace to design and verification plan | Requirements review / SRR |
| Allocated | Architecture decomposition and interfaces, chosen concepts and decisions, detailed module design, and interface contracts with requirement allocation | Design reviews / PDR and CDR as applicable |
| Product | Implementation, interface and document revisions that agree, plus executed verification and validation evidence with residual risks | Release review / FRR |

Baseline names describe evidence to collect; this file does not assert that those reviews
have occurred. Each document states its status and revision. The requirement baseline
belongs to `docs/requirements/`; architecture and composition to `docs/arch/`; shared
mechanisms to `docs/details/`; flow-specific design to a subdirectory such as
`docs/details/simple_flow/`; outcomes to `docs/review/`. New modules may split further
under their module directory with `00_overview.md` followed by numbered topics. For each new flow, add its
purpose, stages, gates, design documents, figures, and implementation status to the root
index rather than changing the site's structure around SimpleFlow. An interface control
document or architecture decision record should be added when the actual design needs
that contract or decision, then linked from the owning architecture or module section.

Figures stay inside the design subsection they explain. Work-management charts belong in
management material, not the architecture or detailed design. Keep files under 800 lines,
headings at depth four or less, and ADRs under 100 lines as the template package advises.
Changes affecting a baseline update the linked requirements, design, interface contract,
model annotations, and review record together; the wish trace records the evaluated
revision rather than silently rewriting a passed assessment.

<a id="d31"></a>

## D31 Executable documentation checks and evidence limits

`tools/check_documentation.py` implements a deterministic directory-layout, locator, embed,
and figure-block audit. It reads the project's `.sysml` files, numbered Markdown documents,
and SVGs in each document directory's `assets/` child from the repository root. Its JSON
report gives an aggregate verdict,
counts, a verdict for each rule, and the source location, target and reason for every
failure. Missing inputs, unreadable files, and annotation forms it cannot
recognize receive an explicit diagnostic; none
may be counted as a pass. The compiler remains responsible for deciding whether the
SysML itself is valid. The audit has these predicates:

| Rule | Exact pass condition | Failure cases |
| --- | --- | --- |
| `directory-layout` | The root has `00_index.md`; every prose-document subdirectory under `docs/` has one `00_overview.md`; other Markdown topics use unique, contiguous numeric prefixes starting at `01`. An asset-only `assets/` child at any depth is exempt and contains no Markdown. | Missing root entry or overview, Markdown in `assets/`, another file reserved at `00`, malformed or duplicate prefix, or a numbering gap. |
| `section-target` | Every `DocSection(document, section)` path stays within `docs/`, names an existing `.md` file, and finds exactly one explicit `id` anchor equal to lowercase `section` followed by a matching section heading. IDs are unique across all numbered documents. | Missing file or anchor, duplicate anchor or ID, mismatched heading, malformed or absolute/outside path. |
| `figure-target` | Every `DocFigure(document, figure)` names an existing document and SVG in that document directory's `assets/` child; its filename starts with the document's numeric prefix plus `_`. Its document embeds that exact asset once. When the same element's `DocSection` immediately precedes `DocFigure` and names that document, the embed is in that section. | Missing asset or document, wrong directory/prefix, no embed, duplicate embed, or an embed outside the adjacent same-document `DocSection` target. |
| `figure-block` | Every model-rendered SVG image embedded in a numbered document has exactly one immediately following Markdown blockquote in the same section. Its first line is a unique `Figure <section-id>-<ordinal>` title matching that section; subsequent bold labels appear in the order `Question`, `Model view`, `Companions`, `Status`, `Reading`, `Limits` with nonempty values. The status begins with an allowed value from D28. | Missing or repeated block, absent or out-of-order field, duplicate figure number, wrong section ID, invalid status, block separated from its image or placed under another section. |
| `embed-source` | Every local model figure embed resolves to one SVG in its carrying document directory's `assets/` child; each figure named by `DocFigure` agrees with its carrying document's actual embed. | Broken path, duplicate or ambiguous target, or annotation/embed disagreement. |

The checker accepts inline Markdown SVG embeds. It reports reference-style Markdown
images, HTML image elements, and external image URLs explicitly for review rather than
silently skipping them. It must normalize relative Markdown links against the document's
directory and reject traversal outside `docs/`. It must not infer a pass from a
`DocFigure` annotation alone: the asset, embed, and block are independent facts.
Conversely, model figures without `DocFigure` still need their embed and block audited.
The checker's output should separate failures from any unsupported syntax or external
figure links, naming the latter
for review instead of counting them as passes. A zero-error audit establishes only those
filesystem, anchor, and block predicates; it cannot establish that the figure's semantic
projection is faithful, that a viewpoint's concern is covered, or that readers can read
the final SVG.

Run the pinned compiler's `parse` and `check` on the changed model and its dependencies;
retain diagnostics and unevaluated rules under `cache/`. Run `tools/render_docs.py` to
regenerate figures and `tools/docs.py build` to check links and anchors. For the nine
primary SimpleFlow views, use the final SVG, matching preview, source hashes, and visual
review process in [D24–D26](04_diagram_presentation.md#d24). An agent or human judges
engineering meaning, completeness, and acceptance against the affected wish's current
artifact revision. That semantic review checks each required viewpoint's stakeholder and
concern, its `subject` link, exact equality of `concernText` and the view's
`DiagramPresentation.question`, suitability of its `expose` selection, faithful
projection, readable presentation, and the block's stated limits. `check_documentation.py`
audits locators and figures, while `sysml check` validates supported SysML relationships;
neither check substitutes for this review. Record the judgment and its evidence URIs in
the wish trace. The documentation audit and four SimpleFlow gate commands execute;
their rule-by-rule report contract is documented in
[D06](simple_flow/02_gates_and_rules.md#d06) and [D07](simple_flow/02_gates_and_rules.md#d07),
with execution evidence in [P08](../prj_manage/03_simple_flow_execution_record.md#p08).
The rendering utility is already implemented ([T01](03_toolchain.md#t01)).
