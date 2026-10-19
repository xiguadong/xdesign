# UML decomposition overview

> Project: xdesign | Status: adopted guide, first module design drafted | Audience:
> architects, software module owners, integrators, and reviewers.
> This directory is the UML decomposition level of the design documentation:
> architecture selects the system structure and interfaces, detailed design fixes the
> module behavior and interfaces, and UML expresses the software design and its
> notation. It does not replace the architecture or detailed-design documents; it owns
> the UML-specific selection rules and notation guidance.

<a id="u01"></a>

## U01 What this directory owns

| Document | Question it answers |
| --- | --- |
| [01_c4_and_uml_guide.md](01_c4_and_uml_guide.md#u06) | Which C4/UML view answers which design question, which notation and figure-block rules apply, and how a module's class-and-function-level API reference is structured ([U47](01_c4_and_uml_guide.md#u47))? |
| [simpflow/00_overview.md](simpflow/00_overview.md#u13) | What public and crate-visible API does the designed `simpflow` crate offer — every struct, enum, trait and nontrivial operation, each with its own page? |

This directory does not own system structure, system context, or module behavior
contracts: those belong to the architecture design and the detailed design, which embed
their own views. A module's UML design lives with that module — the first is the
[SimpleFlow Rust UML design](../details/simple_flow/06_rust_uml_design.md#d44) — and
embeds its figures in the design subsection each one explains. This directory never
collects figures into a separate gallery.

<a id="u02"></a>

## U02 Reading order

1. [01_c4_and_uml_guide.md](01_c4_and_uml_guide.md#u06): selection by stage, notation
   rules, figure-block contract.
2. The module's own UML design document, used with its architecture design and
   interface contract; today that is
   [06_rust_uml_design.md](../details/simple_flow/06_rust_uml_design.md#d44).
3. The class-and-function-level reference: [simpflow/00_overview.md](simpflow/00_overview.md#u13)
   gives the complete API inventory, and each page under `simpflow/` documents
   exactly one API with its signature, contracts, logic and example, in the style of
   the PyTorch API reference.

<a id="u03"></a>

## U03 Relation to the decomposition levels

| Level | Owns | Where |
| --- | --- | --- |
| architecture | system boundary, structure, interfaces, concept selection | [architecture design](../arch/02_design.md#b05) |
| details | module behavior, data, interfaces, tests | [SimpleFlow module design](../details/simple_flow/05_module_design.md#d32) |
| UML | software design notation and UML-based module design | this directory plus the module's UML document |

The whole-project package view includes the Rust UML package
([D21](../arch/02_design.md#d21)); model elements link to their own API pages
through `DocSection` annotations. The levels are complementary, not competing documents. A design concern chooses the
level that answers it and embeds the figure there; the UML level supplies notation,
selection guidance and the applicability contract
([U09](01_c4_and_uml_guide.md#u09)).

<a id="u04"></a>

## U04 Current state and limits

- The pinned toolchain renders SysML views; UML figures in this project are SysML
  projections carrying a UML question (renderable-now), labelled tables
  (table-substitute), planned views, or explicitly sourced external-tool figures
  ([U10](01_c4_and_uml_guide.md#u10)). No dedicated UML renderer exists.
- One module UML design exists: the SimpleFlow Rust design, which is designed-only —
  no Rust code, build or test exists ([D44](../details/simple_flow/06_rust_uml_design.md#d44)).
  Its class-and-function level is the [simpflow API reference](simpflow/00_overview.md#u13):
  a complete grouped inventory with one page per API, each carrying its full Rust
  signature, parameter/return/error contracts, stepwise logic and example.
- The guide derives from the template package under `cache/templates/docs/uml/`,
  mapped onto this repository's directory contract (this overview, numbered topics,
  figure assets beside their documents).

<a id="u05"></a>

## U05 Review

The UML level is reviewable when: every chosen UML view answers a stated question and
is embedded in its section; every figure has the contract figure block; applicability
and status are stated for all fourteen UML views; names, numbering and notation are
consistent; and interface, traceability and test references are linked. The checklist
is [U12](01_c4_and_uml_guide.md#u12); a module design records its own applicability
table, as in [D52](../details/simple_flow/06_rust_uml_design.md#d52).
