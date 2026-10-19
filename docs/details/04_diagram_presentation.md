# Diagram presentation

A diagram must answer its question at the size at which readers encounter it. Selecting
correct model elements and writing a valid SVG do not establish that those elements are
readable. wish-0007 introduces a separate presentation contract for the nine primary
SimpleFlow figures mapped from [D12](simple_flow/04_sysml_diagrams.md#d12) to their
requirements, architecture, and module-design sections.

<a id="d22"></a>

## D22 Renderer responsibility and scope

`DiagramDocumentationRenderer` realizes the project requirements
`diagramViewsAreReadable` and `diagramPresentationRetainsSourceIdentity`. It turns a
focused model view into a diagram whose visible vocabulary, notation, and layout suit the
question being asked. It preserves the model's meaning and source identity while using
short labels in the normal reading surface.

The nine primary figures are the block definition, internal block, requirements,
parametric, use case, activity, sequence, state machine, and package diagrams now embedded
in the stage-owned sections listed in [D12](simple_flow/04_sysml_diagrams.md#d12). Each
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

## D23 From a semantic view to a readable figure

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

## D24 Checks at each pipeline boundary

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
([T04](03_toolchain.md#t04)).

<a id="d25"></a>

## D25 Presentation verification

`DiagramPresentationReview` is the verification case for the two presentation
requirements. It combines structural checks with review of the final nine figures at the
specified article width. The reviewer asks whether each figure answers the question in
[D12](simple_flow/04_sysml_diagrams.md#d12), whether its notation preserves the selected
relationships, and whether every visible label is readable and associated with its element.

The review checks that no label is clipped, no node or port is obscured, and no edge-label
collision makes the intended relationship ambiguous. Crossings must remain understandable.
The main figure uses concise names and meaningful summaries; raw selection expressions,
long qualified identifiers, and complete inherited-member dumps belong in inspectable
source details rather than the diagram's primary labels.

Acceptance also requires that the explanatory prose, model metadata, and selected diagram
agree. The exact activity and lifecycle guards and the authorization binding table remain
in [D16](simple_flow/05_module_design.md#d16), [D18](simple_flow/05_module_design.md#d18),
and [D20](simple_flow/05_module_design.md#d20). A short visible caption must not change their
meaning. A planned verification case or a satisfaction link does not itself record a pass;
the actual assessment belongs to [wish-0007's trace](../review/01_wish_status.md#v06).
Revision 1 passed this review using all nine final figures and the evidence described in
[D26](#d26).

<a id="d26"></a>

## D26 Final-asset evidence and remaining limits

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
presentation work are recorded in [V06](../review/01_wish_status.md#v06).
