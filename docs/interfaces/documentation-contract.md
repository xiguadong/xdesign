# Documentation and SysML contract

A diagram must answer its question at the size at which readers encounter it. Selecting
correct model elements and writing a valid SVG do not establish that those elements are
readable. wish-0007 introduces a separate presentation contract for the nine primary
SimpleFlow figures mapped from [D12](../design/flows/simple-flow/views.md#d12) to their
requirements, architecture, and module-design sections.

<a id="d22"></a>

## 1. Renderer responsibility and scope

`DiagramDocumentationRenderer` realizes the project requirements
`diagramViewsAreReadable` and `diagramPresentationRetainsSourceIdentity`. It turns a
focused model view into a diagram whose visible vocabulary, notation, and layout suit the
question being asked. It preserves the model's meaning and source identity while using
short labels in the normal reading surface.

The nine primary figures are the block definition, internal block, requirements,
parametric, use case, activity, sequence, state machine, and package diagrams now embedded
in the stage-owned sections listed in [D12](../design/flows/simple-flow/views.md#d12). Each
must be legible at a desktop article width
of 900 CSS pixels, without requiring enlargement to understand its main content. This is
a review condition for these focused figures; whole-model definition exports and membership
trees remain reference material and can need enlargement or a more selective view.

The pinned Rust compiler remains responsible for parsing, supported semantic checks, and
semantic interchange. The presentation adapter is responsible for the selected notation,
visible labels, layout, and SVG output. A compiler pass cannot discharge the adapter's
readability obligation. The source model, the diagram-specific projection, the layout
engine, and the documentation display are checked as separate stages.

<a id="d23"></a>

## 2. From a semantic view to a readable figure

| Stage | Responsibility | Retained information |
| --- | --- | --- |
| SysML view | Select elements and relationships that answer one question. | Model definitions, usages, relationships, and selected source identities. |
| Presentation metadata | Supply a concise `label`, optional `summary`, and optional view `question` through `DiagramPresentation`. | The full model name and exact source expression remain in the model. |
| Compiler export | Resolve the selected model using the pinned Rust compiler. | Interchange identities and the relationships available in the export; omissions remain disclosed. |
| Diagram-specific projection | Express those relationships with the notation appropriate to their question. | Source IDs and full labels accompany visible elements in the SVG. |
| Layout and SVG | Use Graphviz to place graph elements and route relationships; use a compact, ordered layout for the sequence. | Generated SVG, layout input, and rendering provenance. |
| Documentation | Display the final figures in the article and allow full-size inspection. | Explanations, exact guard and binding tables, and figure/source mappings. |

Presentation metadata belongs to the model. A renderer may wrap text, arrange nodes, and
choose visual primitives; it may not invent a different engineering relationship or
replace a source expression with an unrelated illustration. A short caption explains the
same model element; the original name, expression, and identity remain available for
inspection in the source model and generated SVG. SVG element titles retain full details
when the SVG is opened directly. The enlargement dialog provides **Open SVG with source
details** for this inspection; embedded images do not expose their internal SVG titles.
The article supplies the normal reading surface, while prose and tables retain details
that cannot fit there.

The notation must preserve the distinctions that made the reference lesson clear:

| Diagram | Presentation obligation |
| --- | --- |
| Block definition | Show type names, ownership multiplicities, and specialization without expanding every inherited member. Multiplicities stay with their relationships. |
| Internal block | Enclose the automation's parts in its system boundary; attach ports to the appropriate parts and connect interfaces to those ports. |
| Requirements | Keep the requirement, satisfier, and planned verification roles distinct; label satisfaction and verification relationships. |
| Parametric | Place the authorization constraint centrally, expose its parameter terminals, and connect the four bound values to their corresponding terminals. Keep the relation readable and the detailed source calculation available. |
| Use case | Put goals inside the subject boundary and participating actors outside it; connect actors to their goals. |
| Activity | Show ordered actions and alternative guarded paths; keep the branch condition beside its branch and disclose unavailable export details. |
| Sequence | Preserve all six selected lifelines and ten ordered messages, with compact participant and message labels. |
| State machine | Distinguish the four states, recovery paths, and completion conditions; keep concise transition labels and document exact guards. |
| Package | Show the seven selected project packages and dependency direction using package notation. |

These are presentation rules for the existing SysML v2 semantic model. They do not add
unmodeled actors, messages, ports, guards, or engineering requirements. The CanSat diagrams
in the lesson are references for composition and notation; the rendered subject remains
xdesign's SimpleFlow.

<a id="d24"></a>

## 3. Checks at each pipeline boundary

A failed presentation check identifies the stage that needs correction:

| Check | Evidence needed | What a failure means |
| --- | --- | --- |
| Model validity | Pinned compiler parse/check reports, with diagnostics and unevaluated rules retained. | Correct the model or disclose the compiler limitation. |
| Projection fidelity | Expected source identities, relationship direction, multiplicities, bindings, and ordered messages. | Correct selection or diagram-specific projection before adjusting layout. |
| Layout | Final SVG with visible boundaries, ports, branch labels, and source-bound edges; inspect for collisions and clipping. | Correct the layout constraints, notation, or selected visible content. |
| Typography at article width | Display each final primary figure at 900 CSS pixels or its smaller natural size; its smallest visible text must be at least 11 CSS pixels. | Reduce unnecessary content or adjust geometry and font size, then render again. |
| Documentation display | Screenshots of the built article using the final assets, together with the full-size view. | Correct article sizing, image styling, or the asset used by the page. |
| Evidence freshness | Matching final SVG and preview hashes, plus screenshots from the corresponding built page. | Regenerate and review the changed figure; a stale preview cannot authorize it. |

For an SVG whose view-box width is `W` and smallest text size is `f` in the same SVG
coordinate units, a displayed image width of `D` CSS pixels shows approximately
`f × D / W` pixels of text. Use the actual displayed width, capped at 900 pixels or the
figure's smaller natural CSS width, and account for any unit conversion in SVG dimensions.
This calculation detects the previous failure in which a wide figure looked adequately labeled at its native size but
shrank below reading size in the article. It does not detect all visual problems: clear
routing, hierarchy, and diagram meaning still need review of the final image.

Structural tests check information retained by the projection. A successful renderer
command establishes that an asset was produced. Neither is evidence of readable layout by
itself. Compiler checks similarly establish only the rules actually evaluated
([T04](../architecture/constraints.md#t04)).

<a id="d25"></a>

## 4. Presentation verification

`DiagramPresentationReview` is the verification case for the two presentation
requirements. It combines structural checks with review of the final nine figures at the
specified article width. The reviewer asks whether each figure answers the question in
[D12](../design/flows/simple-flow/views.md#d12), whether its notation preserves the selected
relationships, and whether every visible label is readable and associated with its element.

The review checks that no label is clipped, no node or port is obscured, and no edge-label
collision makes the intended relationship ambiguous. Crossings must remain understandable.
The main figure uses concise names and meaningful summaries; raw selection expressions,
long qualified identifiers, and complete inherited-member dumps belong in inspectable
source details rather than the diagram's primary labels.

Acceptance also requires that the explanatory prose, model metadata, and selected diagram
agree. The exact activity and lifecycle guards and the authorization binding table remain
in [D16](../design/flows/simple-flow/views.md#d16), [D18](../design/flows/simple-flow/views.md#d18),
and [D20](../design/flows/simple-flow/views.md#d20). A short visible caption must not change their
meaning. A planned verification case or a satisfaction link does not itself record a pass;
the actual assessment belongs to [wish-0007's trace](../records/wishes/wish-history.md#v06).
Revision 1 passed this review using all nine final figures and the evidence described in
[D26](#d26).

<a id="d26"></a>

## 5. Final-asset evidence and remaining limits

Revision-1 evidence is retained under `cache/evidence/wish-0007/`. The final review binds
the files actually inspected to the generated documentation assets. `figure-manifest.json`
records source-bundle and SVG hashes, native source IDs, preview PNG paths and SHA-256
hashes, rasterization and article-width settings, layout-engine provenance, and automatic
checks. `browser-review.json` records the corresponding built SVG hashes, actual article
screenshots and their hashes, and browser text geometry. `visual-review.md` records the
agent's per-diagram judgment. If an SVG changes after a review, its older preview and
screenshots no longer establish acceptance for the changed file.

All nine primary figures passed the 11-pixel text requirement; the smallest text at the
specified width is about 12.11 CSS pixels. The actual 1440-pixel desktop article displays
its widest figure at about 876 pixels and still meets the minimum. Browser checks found
zero text-to-text overlaps and zero canvas clipping, and visual review confirmed readable
notation and relationships. Repeated PNG rasterizations match, and the inspected SVGs
match the copies served by the built documentation.

The complete set of 27 figures regenerated, 52 tests passed, and the strict documentation
build passed. Compiler parse/check and an independent project-model check passed; their
reports retain the unevaluated rules. Regenerating the complete figure set exposes
regressions in other custom diagrams. Acceptance at 900 pixels applies to the nine primary figures;
large historical journey diagrams and whole-model reference exports are assessed according
to their own scope. Enlargement remains useful for close inspection and smaller screens,
but it does not substitute for the primary desktop reading condition.

The compiler's omissions remain model-tool limitations, including activity guard text and
some initial-node references in interchange. The adapter must disclose unavailable data
and retain exact source conditions in the documentation. A readable diagram and a passing
presentation review do not execute the designed workflow, evaluate SysML constraint bodies,
or establish application performance. Current delivery evidence and any outstanding
presentation work are recorded in [V06](../records/wishes/wish-history.md#v06).

> Project: xdesign | Status: design specification | Applies to: project model, wish
> traces, authored documents, model-rendered figures, and their checks.

This specification states the repository's documentation contract. It keeps the model's
structural claims, the documents' rationale, and the wish records' evidence connected
without treating a picture or a compiler pass as proof of engineering acceptance. Existing [B04](../architecture/system-composition.md#b04) explains the
metadata already present; [D22–D26](#d22) defines the
presentation pipeline established by wish-0007.

<a id="d27"></a>

## 6. DocSection: model element to stable document section

`XDesignCommon::DocSection` has two strings: `document`, a repository-relative Markdown
path under `docs/`, and `section`, the stable section identifier in that file. Apply it
*inside* the SysML element it locates; the application annotates the enclosing element.
Keep the element's engineering definition in its proper `mbse/model/` module and let the
document explain it. A wish file may point to documentation, but does not become the only
copy of a requirement or design product.

```sysml
part def XDesign {
    @XDesignCommon::DocSection {
        document = "docs/architecture/system-composition.md";
        section = "B01";
    }
}
```

The target file must exist and contain exactly one explicit anchor whose ID equals the
lowercase form of `section`. The anchor is followed by a meaningful H2-H4 heading; the
heading text is readable and does not need to repeat the ID. IDs are unique across the
document set and survive renaming, splitting, or heading-text changes; update only the
`document` path if a section moves. Page and directory names follow `docs/AGENTS.md`
(semantic lowercase kebab-case; no numeric prefix). The [index](../index.md) carries the
readable model-to-document map, while the annotations are the precise forward links. A
new module or flow gets its own section IDs allocated in the approved mapping; do not reuse
old IDs or renumber historical anchors.

This metadata locates explanation; it does not assert that the element is implemented,
verified, or current in a wish's run. Those claims come from the model, code, and
revision-bound evidence. The documentation checker in [D31](#d31) audits locator targets,
while the compiler checks SysML syntax and supported name resolution.

<a id="d28"></a>

## 7. DocFigure: declared content to the figure in its section

`XDesignCommon::DocFigure` records the `document` carrying a figure and the repository-
relative `figure` path in that document directory's `assets/` child. Apply it to the
declared view or element
whose selected model content the figure renders. Its `document` must be a valid
`DocSection` target for that same view or element, or an explicitly documented carrying
section when the annotation is on a broader package. Keep `DocSection` immediately before
`DocFigure` on the same element so the audit can check section placement. The figure stays
beside its carrying design page in an `assets/` child and uses a unique semantic basename:
for example, `docs/design/flows/simple-flow/views.md` carries
`docs/design/flows/simple-flow/assets/block-definition.svg`.

The [project overview](../README.md) may reuse a canonical SVG with a compact caption
linking to its owning page. This overview embed does not move the figure, add a second
`DocFigure`, or replace the owning page's full figure block.

```sysml
view blockDefinitionDiagram : StandardViewDefinitions::GeneralView {
    @XDesignCommon::DocSection {
        document = "docs/design/flows/simple-flow/views.md";
        section = "D13";
    }
    @XDesignCommon::DocFigure {
        document = "docs/design/flows/simple-flow/views.md";
        figure = "docs/design/flows/simple-flow/assets/block-definition.svg";
    }
    expose XDesignDetailedDesign::SimpleFlowRun;
}
```

Embed the generated figure in the section it explains, using a relative Markdown image
path. A document may also embed a model-rendered figure whose source is tracked in the
[figure provenance table](../architecture/constraints.md#t03); the same explanatory rule applies to it.
Every embedded model figure has one compact Markdown blockquote immediately after its
image, in the same section. Use `**Figure <section-id>-<ordinal> — <title>.**` followed
in order by the bold field labels `Question:`, `Model view:`, `Companions:`, `Status:`,
`Reading:`, and `Limits:`. The ordinal is local to the section. The following block
belongs in `docs/design/flows/simple-flow/views.md`, where
`assets/block-definition.svg` resolves beside that page:

```markdown
![Run structure](assets/block-definition.svg)

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
[D23](#d23). If the tool cannot render the required view, place
a labeled table in the design subsection and mark it `table-substitute`; do not call it a
generated figure. A changed final SVG needs a matching preview and renewed visual review
before any carried acceptance claim is reused ([D24](#d24)).

<a id="d29"></a>

## 8. Concern, viewpoint, view, and document

The design starts with a stakeholder concern and a specific question. A viewpoint names
the concern and stakeholder; the accompanying document defines the selection and
presentation method and review criteria for that question. A SysML v2 view selects actual
elements through `expose` and, where applicable, a `render` request. The rendered figure
is a projection of that selection, and `DocSection`/`DocFigure` connect the view to its
explanation. The document explains why the selection matters and what it cannot prove.
This chain applies separately to each flow and each design concern.

| Layer | Decision owned there | Example in the current project |
| --- | --- | --- |
| Concern and viewpoint | Question, stakeholder, selection rule, notation, companions, review criterion | `XDesignDetailedDesign::DocumentationViewpoint` and `blockDefinitionConcern`, plus the nine questions in [D12](../design/flows/simple-flow/views.md#d12) and presentation rules in [D23](#d23) |
| Model view | Actual exposed elements and requested rendering | `XDesignDetailedDesign::blockDefinitionDiagram` and its eight exposed definitions |
| Figure | Visible projection with source references | `block-definition.svg` |
| Document section | Interpretation, limits, and related contracts | [D13](../design/flows/simple-flow/views.md#d13) |
| Wish assessment | Whether an artifact revision satisfies a criterion | wish-0007's final presentation review in [V06](../records/wishes/wish-history.md#v06) |

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
security, decision, or risk views are chosen by the concern. The [D12 coverage index](../design/flows/simple-flow/views.md#d12)
links the nine SimpleFlow views to their requirements, architecture, and module-design
sections; their figures are embedded in those sections.

<a id="u09"></a>

## 9. View applicability for a software design

A module's software design states for each view whether the view is required,
recommended, or not applicable, with a reason. An omitted view without a reason is a
defect; a reasoned omission is a design decision. The decision is recorded in the
module design's own applicability table — the SimpleFlow Rust design's is
[D52](../design/flows/simple-flow/views.md#d52) — and reviewed with the design
([U12](../verification/review-criteria.md#u12)).

The vocabulary is the SysML projection vocabulary this repository actually renders:
package projections for module layout and allowed dependency direction,
block-definition projections for type structure, internal-block projections for
connected parts, sequence projections for interactions, and state-machine projections
for lifecycle handling, alongside the activity, use-case, requirements, and parametric
questions of the [D29](#d29) coverage checklist. Each adopted view is declared in the
model, located by `DocSection`/`DocFigure`, and embedded in the design section that
answers its question — never collected into a separate view gallery. The figure's
status vocabulary and block are [D28](#d28)'s; the two rules D28 does not state follow.

<a id="u10"></a>

### 9.1 Figure numbering

A figure is numbered by its document section — `Figure <section-id>-<ordinal>`, for
example `Figure D47-1` — and ordinals stay local to the section. A figure added to one
section never renumbers another section's figures, so a figure number stays stable for
as long as its section exists.

<a id="u11"></a>

### 9.2 Figure reading note and design rationale

Every figure needs a reading note and a design rationale; a bare figure is not
acceptable. The reading note — the figure block's `Reading` field — names the visible
elements, relationships, and the point of the figure in two to four sentences. The
rationale, recorded in the embedding section's prose, says why the boundary, types, or
interaction are divided this way.

<a id="d30"></a>

## 10. Document classes, baselines, and flow extension

The documentation set is organized by document class rather than by numbered directories:

| Class | Home | Owns |
| --- | --- | --- |
| Requirements | `requirements/` | normative requirements, rationale, acceptance conditions |
| Architecture | `architecture/` | context, composition, behavior, lifecycle, constraints |
| Detailed design | `design/` | module behavior, gates, revisions/evidence, and the authored API design set |
| Interface control | `interfaces/` | ICD, command line, record data contract, documentation contract |
| Verification design | `verification/` | methods, cases, criteria, acceptance boundaries, trace; no executed results |
| Records | `records/` | plans, reviews, executions, evidence summaries, status, wish history |

`docs/index.md` is the reader entry; `docs/AGENTS.md` holds authoring rules. Page names are
semantic lowercase kebab-case; `index.md` is used only for a genuine class or layer landing
page. Figures stay beside the design section they explain; work-management charts and
execution evidence belong in `records/`. Decisions and their rejected alternatives stay in
the owning architecture/design page or in a plan/review record; there is no separate
decision directory.

| Baseline role | Required content before claiming it | Review point |
| --- | --- | --- |
| Functional | Requirements with rationale, measurable acceptance conditions, method and phase, and trace to design and verification plan | Requirements review / SRR |
| Allocated | Architecture decomposition and interfaces, chosen concepts and decisions, detailed module design, and interface contracts with requirement allocation | Design reviews / PDR and CDR as applicable |
| Product | Implementation, interface and document revisions that agree, plus executed verification and validation evidence with residual risks | Release review / FRR |

Baseline names describe evidence to collect; this file does not assert that those reviews
have occurred. Each document states its status and revision. A new developed flow adds a
directory under `design/flows/` with index, definition, implementation, and views pages;
shared flow behavior is stated once in the common design pages, and API pages remain one per
API under `design/api/`, without changing another flow's pages or stable IDs. Keep files under 800 lines and headings at depth four or
less. Changes affecting a baseline update the linked requirements, design, interface
contract, model annotations, and review record together; a wish trace records the evaluated
revision rather than silently rewriting a passed assessment.

<a id="d31"></a>

## 11. Executable documentation checks and evidence limits

`tools/check_documentation.py` implements a deterministic directory, locator, embed, and
figure-block audit. It reads the project's `.sysml` files, authored Markdown documents, and
SVGs in each carrying page's `assets/` child from the repository root. Its JSON report gives
an aggregate verdict, counts, a verdict for each rule, and the source location, target, and
reason for every failure. Missing inputs, unreadable files, and annotation forms it cannot
recognize receive an explicit diagnostic; none may be counted as a pass. The compiler
remains responsible for deciding whether the SysML itself is valid. The audit has these
predicates:

| Rule | Exact pass condition | Failure cases |
| --- | --- | --- |
| `directory-layout` | The root has `index.md`; Markdown lives under the approved document classes; page and directory names are lowercase semantic kebab-case; `index.md` is used only for landing pages; asset directories contain no Markdown. | Missing root entry, unrecognized class directory, invalid page name, Markdown in an asset directory. |
| `section-target` | Every `DocSection(document, section)` path stays within `docs/`, names an existing `.md` file, and finds exactly one explicit `id` anchor equal to lowercase `section`. The anchor is followed by a meaningful H2-H4 heading; heading text does not need to contain the ID. IDs are unique across the document set. | Missing file or anchor, duplicate anchor or ID, malformed or absolute/outside path, anchor not attached to a heading. |
| `figure-target` | Every `DocFigure(document, figure)` names an existing document and SVG in that page's `assets/` child, uses a unique semantic asset basename, and its document embeds that exact asset once. When the same element's `DocSection` immediately precedes `DocFigure` and names that document, the embed is in that section. | Missing asset or document, wrong asset directory, no embed, duplicate embed, or an embed outside the adjacent same-document section. |
| `figure-block` | Every model-rendered SVG image embedded in an authored page has exactly one immediately following Markdown blockquote in the same section. Its first line is a unique `Figure <section-id>-<ordinal>` title matching that section; the bold labels appear in the order `Question`, `Model view`, `Companions`, `Status`, `Reading`, `Limits` with nonempty values. The status begins with an allowed value from D28. | Missing or repeated block, absent or out-of-order field, duplicate figure number, wrong section ID, invalid status, block separated from its image or placed under another section. |
| `embed-source` | Every local model figure embed resolves to one SVG in its carrying page's `assets/` child; each figure named by `DocFigure` agrees with its carrying page's actual embed. | Broken path, duplicate or ambiguous target, or annotation/embed disagreement. |

The checker accepts inline Markdown SVG embeds. It reports reference-style images, HTML
image elements, and external image URLs explicitly for review rather than silently skipping
them. It normalizes relative Markdown links against the page's directory and rejects
traversal outside `docs/`. It does not infer a pass from a `DocFigure` annotation alone: the
asset, embed, and block are independent facts. Conversely, model figures without
`DocFigure` still need their embed and block audited. A zero-error audit establishes only
those filesystem, anchor, and block predicates; it cannot establish that a figure's semantic
projection is faithful, that a viewpoint's concern is covered, or that readers can read the
final SVG.

Run the pinned compiler's `parse` and `check` on the changed model and its dependencies;
retain diagnostics and unevaluated rules under `cache/`. Run `tools/render_docs.py` to
regenerate figures and `tools/docs.py build` to check links and anchors. A changed final SVG
needs a matching preview and renewed visual review before any carried acceptance claim is
reused. The checker's mechanical result and a reviewer's engineering judgment are separate:
one can pass while the other fails, and neither substitutes for the other.
