# Documentation authoring rules

The root `AGENTS.md` applies here. This file defines how documents under `docs/` are
named, identified, linked, and verified. It is a rules file, not a published page.

## One home per fact

- Requirements live in `requirements/`; architecture in `architecture/`; module and API
  design in `design/`; cross-boundary contracts in `interfaces/`; verification design in
  `verification/`; plans, reviews, executions, status, and wish history in `records/`.
- A design page states the design claim and links to the model element, source, verification
  case, or record that supports it. It does not copy signatures, record results, or repeat
  another page's status table.
- `verification/` is verification design only. Executed results, evidence summaries, and
  completion records belong in `records/` and are not rewritten as design claims.
- Historical records keep their dates, command results, findings, and evidence pointers.
  Moving or linking a record does not create a new assessment.

## Paths and filenames

- Use the document classes and paths in the repository's approved target tree. Do not add
  `governance/`, `decisions/`, `reference/`, `start/`, `guides/`, or `process/` trees.
- File names are lowercase kebab-case and semantic. Do not use numeric prefixes or
  underscores. `index.md` is only for a genuine class or layer landing page.
- API design keeps one page per API under `design/api/core/`, `design/api/simple-flow/`,
  or `design/api/wish-verify/`. Each page retains its existing `U` anchor and states the
  design contract, model realization or declared language-side rationale, and implementation
  projection pointer.

## Headings, IDs, and anchors

- One H1 per page. Formal design and specification pages use local decimal headings
  (`1.`, `1.1`, `1.1.1`) that restart on each page; hubs, API pages, and records keep natural
  headings and dates.
- A referenced or figure-carrying section has an explicit HTML anchor on its own line before
  a meaningful H2-H4. Existing `A/B/D/R/F/E/I/V/U` IDs are preserved and never renumbered;
  do not put the ID in the visible heading. New model-located IDs are allocated in the
  approved mapping, remain globally unique, and are not derived from display numbers.
- Fenced examples do not create real anchors or headings. Keep examples out of the
  anchor/target checks.

## Links and figures

- Use file-relative Markdown links and descriptive link text such as "gate authorization",
  not a bare `U19` or `D27` label as the reading interface.
- A model-rendered figure sits in the `assets/` child of its carrying page, has a unique
  semantic basename, and keeps the six ordered fields Question, Model view, Companions,
  Status, Reading, and Limits plus its `Figure <section-id>-<ordinal>` label.
- `DocFigure.figure` names the canonical asset; the renderer registry carries the same
  mapping. Do not move a figure without updating both and the carrying-section embed.

## Verification of documentation changes

- Rewritten design pages are checked against the model, source, verification design, and
  retained records; a claim without a traceable basis is a defect, not a style issue.
- During the documentation stage, validate with the approved mapping and a strict MkDocs
  build. The maintained documentation checker is updated in the link/reference-sync stage;
  after that, run the default canonical audit.
- Keep command logs and temporary output under `cache/`. Do not add this refactor's plan,
  mapping, charters, or evidence to `docs/` or `records/`.
