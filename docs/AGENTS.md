# AGENTS.md

This file defines documentation rules under `docs/`. The repository's root `AGENTS.md` also applies here.

## Filename Convention

The repository entry is `docs/00_index.md`. Every documentation subdirectory begins
with `00_overview.md`; it tells a reader what that directory owns, gives the reading
order, and states the current design and review status. This applies recursively, for
example to `docs/details/simple_flow/`. Each document directory may have an `assets/`
child for its generated figures. These asset-only directories contain no Markdown
documents and are exempt from the overview and numbering rules.

Name the other documentation files using this pattern:

```text
NN_ref.md
```

| Field | Meaning | Rules |
| --- | --- | --- |
| `NN` | Document index, local to its directory. | Reserve `00` for `00_overview.md`; number the next document `01`, then continue in reading order. Use at least two digits, with leading zeros where needed. |
| `ref` | A short description of the document's subject. | Concise, meaningful English in lowercase snake_case. Do not repeat the directory's name; the path already says which module the document belongs to. |

## Numbering and Directory Rules

- Numbering is **per directory**, not across `docs/`. Every documentation directory — `arch/`, `requirements/`, `details/`, `review/`, and any nested design directory — has exactly one `00_overview.md`. One directory's documents never renumber because another's changed.
- A module may be **decomposed into subdirectories**. A topic with several documents gets its own directory under its module, for example `docs/details/simple_flow/`; its overview is `00_overview.md` and its first topic document is `01_*.md`. Keep one file per numeric prefix and continue the local sequence without gaps.
- When bringing an existing directory into this convention, rename its `00_index.md` to `00_overview.md` if present; otherwise add `00_overview.md` and move existing numbered topic files one position later. Update all links, model annotations, figure paths, navigation, and renderer outputs in the same change. After that migration, keep document numbers and section IDs stable when editing content. Each document has a unique index **within its directory**.
- After `99`, continue with `100`, `101`, and so on.
- If a document covers multiple stages, use its primary stage's directory and describe the other stages in its content.
- `AGENTS.md` is exempt from the filename pattern and index sequence.
- Architecture decision records use four digits from `0001_<subject>.md` onward; `00_overview.md` remains the directory overview and index.

Run `python3 tools/check_documentation.py --json` after changing a documentation
directory. Its directory-layout rule checks the overview, numeric sequence, and unique
prefixes alongside model locators and figure blocks.

## Section Identifiers

Give every section a stable identifier and an anchor, so a model element or another
document can refer to one section instead of a whole document:

```markdown
<a id="r01"></a>

## R01 The requirements
```

Prefixes in use: `A` overview, `B` architecture, `D` design, `R` requirements, `F` flows,
`E` records, `T` toolchain, `I` interface control, `J` decisions, `V` review, and
`U` UML, and `P` project management. Identifiers are unique across all documents, so a
section keeps its identifier wherever its document is renamed or split. A reference names
the document and the identifier, for example
`docs/details/01_flows_and_checkpoints.md#f01`.

## UML documentation

`docs/uml/` owns the C4/UML notation guide and the view-applicability contract; a
module's UML design lives with that module (for example
`docs/details/simple_flow/06_rust_uml_design.md`) and follows the guide's selection,
figure-block, and status rules. A software module's class-and-function level is an
API reference under `docs/uml/<module>/`: one inventory page listing every public
API, then one page per API with its signature, contracts and stepwise logic
(`docs/uml/01_c4_and_uml_guide.md#U47`). UML figures are SysML projections rendered
by the pinned toolchain, labelled tables marked table-substitute, or explicitly
sourced external-tool figures — never unanchored drawings. A software design states
its implementation status honestly: designed-only until code exists in the repository.

## Project management documents

`docs/prj_manage/` owns the durable review synthesis and the current implementation
plan. Its `00_overview.md` gives the reading order, and numbered documents record the
task, design decisions, source-file plane, implementation sequence, acceptance cases,
actual verification, and limits. Keep plan status and code status synchronized. Local
reports under ignored `cache/` may support a decision, but copy the durable finding
and its disposition into a numbered project-management document.

## Figures

A figure lives in the `assets/` child of the directory containing its document, and
is named for the document it belongs to:

| Document | Its figures |
| --- | --- |
| `docs/arch/01_system_composition.md` | `docs/arch/assets/01_*.svg` |
| `docs/details/01_flows_and_checkpoints.md` | `docs/details/assets/01_*.svg` |
| `docs/details/simple_flow/03_structure_graphs.md` | `docs/details/simple_flow/assets/03_*.svg` |

Because each document directory owns its assets, equal numeric prefixes in different
directories do not make their figures collide. Figures are rendered from the model by
`python3 tools/render_docs.py`; figures are assets, not documents, and carry no document
index of their own.

## Examples

```text
docs/arch/00_overview.md
docs/details/00_overview.md
docs/details/simple_flow/00_overview.md
docs/details/simple_flow/02_gates_and_rules.md
```

Use English for all filenames and document content.
