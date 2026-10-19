# C4 and UML guide for design documents

> Audience: architects, module owners, integrators, and reviewers writing or reading
> software and system design documents.
> Basis: the template C4/UML guide under `cache/templates/docs/uml/` (C4 model, UML 2.5
> selection by stage, drawing rules) adapted to this project's documentation contract
> ([D27–D31](../details/05_documentation_contract.md#d27)). SysML remains the primary
> modelling language for system structure, behavior, requirements, and parametrics;
> C4 and UML are additional views for the questions where they are clearer.

<a id="u06"></a>

## U06 Where C4 and UML belong

C4 and UML are notation choices for a **view of the design**, not separate
documentation sets. A view answers one question and is embedded in the design
subsection it explains. Never collect figures into a separate "UML chapter" or a
standalone view gallery.

- Prefer the SysML nine views for system structure and behavior (block definition,
  internal block, requirements, activity, sequence, state machine, use case,
  parametric, package).
- Use C4 for communicating software architecture to different audiences.
- Use UML for software detailed design (classes, objects, components, interactions,
  states, deployment, packages).
- A design may skip a notation only with a written reason; record applicability as
  required, recommended, or not applicable ([U09](#u09)).

<a id="u07"></a>

## U07 C4 model (architecture communication)

| Level | Content | Audience | Typical companions |
| --- | --- | --- | --- |
| L1 System context | system, users, external systems | everyone | context/use-case view, interface list |
| L2 Container | deployable units, data stores | engineers and stakeholders | deployment, interface contract |
| L3 Component | modules inside a container | engineers | class/sequence views, package view |
| L4 Code (optional) | classes and interactions; usually UML | core developers | class, sequence, state views |

Every C4 figure carries a 2–4 sentence rationale: why the boundary, containers or
components are divided this way. Container and component names use the same names in
every figure.

<a id="u08"></a>

## U08 UML 2.5 views (14 diagrams)

Structure views:

| View | Question it answers | Typical stage | Companions |
| --- | --- | --- | --- |
| class | What types exist, with which attributes and relations? | detailed design | sequence, state, package |
| object | What instances and links exist at one moment? | detailed design, fault analysis | class, test data |
| component | What deployable software parts exist, with which provided/required interfaces? | architecture, detailed design | composite, deployment, interface contract |
| composite structure | What parts, ports and connectors are inside a component? | detailed design | component, sequence |
| deployment | Which software runs on which nodes, over which protocols? | architecture, detailed design | deployment topology, interface contract |
| package | How is the model or code organized, and which packages depend on which? | architecture, detailed design | component, module dependency |
| profile | Which domain stereotypes extend the language? | modelling convention | class, tooling rules |

Behavior views:

| View | Question it answers | Typical stage | Companions |
| --- | --- | --- | --- |
| use case | Who uses the system and which services cross its boundary? | requirements, architecture | context, activity |
| activity | What is the control flow, with decisions and concurrency? | requirements, detailed design | sequence, state |
| state machine | Which states and transitions govern behavior? | detailed design | sequence, activity |
| sequence | Which parts exchange which messages, in what order? | detailed design | component, interface, timing |
| communication | Which objects collaborate, and what links them? | detailed design | sequence, class |
| interaction overview | How do interaction fragments compose? | detailed design | sequence |
| timing | What are the time constraints on state changes and messages? | detailed design | sequence, performance |

<a id="u09"></a>

## U09 Selection by stage

| Stage | Recommended C4/UML views | SysML counterparts |
| --- | --- | --- |
| requirements / SRR | use case; activity for complex flows; context diagram | use case, activity, requirements |
| architecture / PDR | C4 L1–L2, component, composite, deployment, package | block definition, internal block, package, requirements, sequence, state |
| detailed design / CDR | class, object, component, composite, sequence, communication, state machine, activity, deployment, package, timing | block definition, internal block, activity, sequence, state machine, parametric |
| verification / FRR | test-case mapping, deployment/operations views where relevant | requirements trace, verification matrix |

A design states for each C4/UML view whether it is required, recommended, or not
applicable (with a reason). The decision is recorded in the module design's own
applicability table; the SimpleFlow Rust design's is
[D52](../details/simple_flow/06_rust_uml_design.md#d52). An omitted view without a
reason is a defect; a reasoned omission is a design decision.

<a id="u10"></a>

## U10 Relating UML to SysML and the documentation contract

- The SysML model remains the structural source of truth. A UML or C4 figure in this
  project is a projection of model elements declared as a SysML view (preferred), a
  labelled table marked table-substitute, or an explicitly sourced external-tool
  figure; it is never an unanchored drawing.
- Every model figure is located by `DocSection`/`DocFigure` and followed by the
  figure block of the documentation contract ([D28](../details/05_documentation_contract.md#d28)):
  title and number, Question, Model view, Companions, Status, Reading, Limits.
- The status is one of renderable-now, planned, table-substitute, or external-tool.
  State designed-only or implemented separately when making a software claim — a UML
  design without code is designed-only
  ([R01](../requirements/01_baseline.md#r01), `softwareDesignClaimsNoImplementation`).
- When the toolchain cannot render a UML view, put a labelled table or a short textual
  description in the design subsection and mark it table-substitute. Do not draw a
  separate engineering diagram or paste a screenshot into the document.

Required figure block:

```markdown
![<figure title>](<assets/NN_figure.svg>)

> **Figure <section-id>-<ordinal> — <title>.**  
> **Question:** <the one design question the figure answers>.  
> **Model view:** <declared view/projection, or table-substitute>.  
> **Companions:** <other views, interface contract, trace section>.  
> **Status:** <renderable-now / planned / table-substitute / external-tool>.  
> **Reading:** <two to four sentences naming the visible elements and the point of the figure>.  
> **Limits:** <what the figure does not establish or omits>.
```

<a id="u11"></a>

## U11 Drawing and writing rules

- Naming: one object has one name in every figure; use consistent case and avoid
  aliases. Rust types in the SimpleFlow design carry the `Rs` prefix so a figure never
  confuses them with the SysML design elements they realize.
- Numbering: number figures by their document section, for example
  `Figure D47-1`; keep ordinals local to the section.
- Colors: actors/blue, internal elements/green, synchronous messages solid,
  asynchronous messages dashed, key notes red; keep the palette consistent across the
  document set.
- Size: one sequence view has at most 8–10 participants; one figure fits one A4
  landscape page; split larger scenarios.
- Figure plus text: every figure has a reading note and a design rationale; a bare
  figure is not acceptable.
- Source: prefer model-rendered figures. If an external UML tool (PlantUML, Mermaid
  or a drawing tool) is explicitly accepted, keep its source in the document's assets
  child, mark the figure external-tool, and review the rendered result; never paste a
  screenshot without a source.
- Symbols: decisions as diamonds, processing as rectangles, synchronous messages as
  solid arrows, returns as dashed arrows, states as rounded rectangles; use the
  selected notation consistently within a document.

<a id="u12"></a>

## U12 Review checklist

- [ ] each C4/UML view answers a stated question and is embedded in that subsection;
- [ ] required views are present; omitted views have a reason;
- [ ] every figure has the contract figure block and a valid status;
- [ ] model projections agree with the model elements; no hand-drawn engineering diagram;
- [ ] names, numbering, and notation are consistent across figures;
- [ ] interface, traceability, and test references are linked;
- [ ] the figure's limits are stated and do not overclaim implementation;
- [ ] a module's software design has an API reference per [U47](#u47): complete
  inventory, one page per API, contracts and logic on every behavioral entry.

<a id="u47"></a>

## U47 API-reference structure (class-and-function level)

A module's UML design has two granularities, and both are required for a software
module. The module level — component structure, type groups, key interactions,
states, applicability — lives in the module's own document (for example
[D44–D52](../details/simple_flow/06_rust_uml_design.md#d44)). The class-and-function
level lives in an **API reference** organized after the PyTorch API reference:

- **One inventory page.** Every public or crate-visible nontrivial API of the module
  — structs, enums, traits, functions, methods, constants — appears exactly once, grouped by role, with its
  signature and a one-line summary. A reader learns what the module offers from this
  page alone. Prose-only mentions of an API are defects.
- **One page per API.** Each entry documents exactly one API: full signature
  (generics, lifetimes, trait bounds, `Result` error channels), parameter contract,
  return contract, error contract variant by variant, stepwise logic (≤ 50
  numbered steps; split the function otherwise), side effects, complexity, thread
  safety, and a small example. A struct entry carries its field table with
  visibility and invariants; an enum entry its variant table; a trait entry its
  implementor contract plus one entry per required method.
- **Conventions.** Trivial getters and constructors are declared on the owning type's page and marked
  `trivial` — no page of their own, but still listed. One object has one name across
  signatures, inventory, figures and model. Every entry names the model element it
  realizes, or states why it is a language-side addition. Every entry is marked
  designed-only until code exists in the repository.

The reference sits in its own directory under `docs/uml/<module>/` with the
repository's overview and numbering rules; the first instance is the
[simpflow API reference](simpflow/00_overview.md#u13). The template with the full
entry formats and rules is `cache/kimi_template/docs/detailed-design/DD-API参考级设计模板.md`.
