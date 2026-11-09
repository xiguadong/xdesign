# SysML documentation authoring contract

> Contract: `xdesign-sysml-docs` 1.3.0. The adjacent `profile.json` is the
> machine-readable route and template contract. Rule IDs below are stable within
> this version. A project pins the version it uses and reviews any upgrade.

This pack makes SysML-backed documentation the design source that code must answer.
It governs authored documents in a selected project root. It does not prescribe a
particular agent, model provider, file writer, diagram renderer, or execution flow.
MCP, CLI, and a human author must apply the same rules to the same files.

Schema 1.1 adds a catalog of generic defaults and scenario entries. Read the routing
and entry guides, select an explicit `template_id` or accept the class's generic
default, and keep that effective identity in the managed marker and registry. Later
updates and checks use the persisted entry rather than reselecting a changed default.
Examples are fictional and never become selected contracts or acceptance evidence.
The byte-exact schema 1.0 compatibility pack retains its original class behavior.

## Reusable rules

### DOC-001 — One home and a safe document path

Choose the document class that owns the claim: requirements, architecture, design,
interface, or verification. Resolve its path from `profile.json` relative to the
selected project root. The `{name}` stem is lowercase semantic kebab-case. Reject
absolute paths, `..`, unexpected separators, and symlinks that escape the project
root. Do not create a second authoritative copy of a requirement, decision, API
contract, or result in another class. Link to its owner instead.

### DOC-002 — Required structure and identification

A managed page has one H1, a status and revision, a model-source reference, and the
class's exact ordered H2 headings and required labeled fields in `profile.json`.
Formal sections use local decimal heading numbers, restarting on each page. A
referenced or figure-carrying section has an explicit globally unique ID on a
standalone `<a id="..."></a>` line immediately before its H2–H4 heading. IDs are
allocated explicitly, not derived from heading numbers, and existing IDs are never
renumbered. The profile's preferred prefixes guide new primary sections; all
listed prefixes remain valid for a cross-cutting section. Extra subsections may
refine, but not replace, required sections.

The exact heading and field checks apply to pages carrying this pack's managed
marker. Existing project pages are not silently rewritten to a new template.
Project-level path, anchor, link, and model-locator checks still apply to them.

### DOC-003 — Engineering claims and acceptance conditions

State each requirement in testable terms with its rationale, measurable acceptance
conditions, and planned verification method. Explain architecture and design
choices, their boundaries, alternatives or tradeoffs, and unresolved constraints.
Point from design to its SysML elements and intended source projection. A code
pointer indicates an implementation target; it does not prove that implementation
or verification occurred. State status and limits accurately. Do not turn a
declared SysML verification case, a compiler pass, or a documentation check into
an executed engineering result.

### DOC-004 — Model-to-document trace

Give each substantive design claim a precise model locator or an explicit reason
it is language-side only. Keep `DocSection` targets at the current document path
and stable section ID. For a model figure, keep `DocFigure`, the renderer mapping,
the canonical asset, and the page embed in agreement. The SysML compiler checks
supported syntax and references; a documentation audit checks paths and targets.
Neither establishes the claim's semantic adequacy.

### DOC-005 — Figure as a documented answer

Put a model-rendered SVG in the carrying page's `assets/` child with a semantic
basename. Embed it once in its owning section. Immediately after the embed, give
it a unique `Figure <section-id>-<ordinal>` label and the six ordered, nonempty
fields `Question`, `Model view`, `Companions`, `Status`, `Reading`, and `Limits`.
Use a status from `profile.json`. The question says what the view answers; the
reading explains the visible elements and relationships; the limits name what
the figure cannot establish. Review the final SVG for readability and source
fidelity after rendering.
The exact embed and blockquote syntax is in the selected language's `profile.json` `figure_block.templates` declaration.
Its primary status is one of `renderable-now`, `planned`,
`table-substitute`, or `external-tool`; `designed-only` and
`implemented` are separate software-claim qualifiers.

### DOC-006 — Navigable links and source identity

Use project-relative model locators and file-relative Markdown links with
descriptive text. A document section links to its requirements, related design,
interface contract, verification design, or retained record rather than copying
their content. Every local target must resolve in the built documentation. A
display label or abbreviated diagram caption must preserve a route back to the
native SysML identity.

### DOC-007 — Verification plan and execution evidence

Verification documents state methods, environment, case inputs, procedure,
expected results, measurable pass criteria, and trace. An unexecuted case remains
planned. Executed commands, observations, judgments, residual risks, and the
exact assessed artifact revision belong in the project's evidence record or
store. Keep historical records immutable; append a new revision to correct or
supersede a result.

### DOC-008 — No placeholder acceptance

Unresolved `{{...}}` tokens, `TODO`, `TBD`, or `FIXME` in authored prose prevent a
managed document from passing. An authoring tool may create a draft skeleton,
but must report it as incomplete until every required section and field has
substantive content. A filled field can still be semantically wrong; mechanical
completeness is not engineering acceptance. Do not fabricate claims or evidence
to make a check pass.

### DOC-009 — Separate mechanical and semantic results

Report deterministic findings with rule ID, project-relative file, line, and
reason. A structural pass means only that checked paths, headings, fields,
anchors, figure blocks, and links satisfy their predicates. The author or reviewer
must separately judge model coverage, rationale, acceptance quality, figure
fidelity, and engineering correctness. Preserve failed and unevaluated results;
never return a single "accepted" verdict merely because syntax passed.

### DOC-010 — Design before implementation

Before implementation, establish the requirements, SysML model, relevant
architecture/design/interface pages, and planned verification cases at an
identified revision. During implementation, point code and tests back to that
design. After implementation, run the applicable model, documentation, build,
and product checks, then record evidence against the assessed revision. A
design change after a pass requires a new assessment for the changed revision.

## xdesign project additions

The reusable rules above are the minimum contract. When this pack is used in the
xdesign repository, the root `AGENTS.md`, `docs/AGENTS.md`, and
`docs/interfaces/documentation-contract.md` add these stricter obligations:

- `docs/README.md` is a special overview exception. `docs/records/` is a frozen
  archive; maintained records belong in `.xdesign` record items. No authoring
  operation may rewrite that archive.
- `index.md` is only a genuine class or layer landing page. API design has one
  page per API in the approved `docs/design/api/` branches. A new developed flow
  uses `docs/design/flows/<flow>/` with index, definition, implementation, and
  views pages. The bundled structured classes resolve those routes directly;
  the five flat classes retain their earlier paths and defaults.
- xdesign's explicit `DocSection` and `DocFigure` annotation syntax, renderer
  registry, canonical SVG ownership, and figure status vocabulary are normative.
  The nine diagram questions form an applicability checklist, and final figures
  are reviewed at the documented 900-pixel article width. A diagram is not
  required for an inapplicable question, but the applicability reason is stated.
- The repository's `tools/check_documentation.py` implements five deterministic
  rule families: `directory-layout`, `section-target`, `figure-target`,
  `figure-block`, and `embed-source`. Its clean report covers those predicates
  only. The pinned SysML compiler, figure renderer, strict docs build, and
  semantic review remain separate checks.

For another project, keep the pack versioned in Git and supply a project-specific
profile for its routes, model roots, anchor allocation, and added review criteria.
Do not assume xdesign's frozen archive, flow subdirectories, or diagram count
apply to that project.

## Structured release

Read [API.md](API.md) for the exact operations and publication outcomes. API pages
require allocated U anchors and a layer field matching their route; index is
reserved as an API basename. Flow index is a genuine landing page with natural
headings; definition, implementation and views have distinct decimal sections.
Every page supplies its own metadata and readable model sources. Optional figures
still require supplied assets and complete figure blocks. Neither a failed
publication nor a selected-document check implies whole-project acceptance.
The original 1.0, 1.1 and 1.2 closures remain selectable without migration.


## Bilingual resource and identity rules

Select one declared `en` or `zh-CN` variant. All 21 entries retain their engineering
class, ordered section/field keys and checks in both languages. The manifest owns
localized headings and labels; stable field keys, routes, anchors, API layers,
model names and status tokens keep their meanings. Chinese templates guide
connected, objective reasoning about the problem and boundary, rationale,
method/design, evidence/verification and limits as appropriate to the class.
Neither language may add, omit or weaken engineering obligations. Unsupported
citations and invented model, implementation, test or review results are forbidden.
Fictional examples and planned cases must retain that status.

Plan/create omission means `en`; exact `zh-CN` selects Chinese. Prose and locale do
not select or convert a variant. A new marker and registry include language and
digest version 3. Updates preserve effective identity and registry bytes; an
optional language assertion must agree. Missing/unsupported/disagreeing persisted
language is a registry defect. An occupied route never converts language.
Historical packs remain immutable and language-less. Select the current pack for
mixed old/new checks and updates; original packs may reject a foreign registry.

Read `catalog/shared/routing.md`, the selected entry guide and `API.md` before
writing. The complete resource tree and original nested compatibility packs are
part of the selected contract digest. No prose detector, fallback translation or
automatic migration supplies missing resources or identity. Structural checks
leave scholarly quality and English equivalence explicitly unevaluated.
