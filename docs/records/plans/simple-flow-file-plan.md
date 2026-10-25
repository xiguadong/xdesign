# SimpleFlow implementation file plan and verification cases

> Scope: first executable `designedSimpleFlow` slice | Status: implemented source
> layout and verification design. The D53–D55 section identifiers remain stable after
> relocation from detailed design. A declared case is not an executed result;
> [P08](../executions/simple-flow.md#p08) records actual runs.

This project-management record connects the project requirements and [run design](../../design/flows/simple-flow/definition.md#d01) to
the Rust [module/API design](../../design/flows/simple-flow/implementation.md#d44), the [fixed gate rules](../../design/flows/simple-flow/definition.md#d06),
the [record contract](../../design/evidence-and-revisions.md#e03), and the
[draft checker and judge interfaces](../../interfaces/contracts.md#i04). The executable
slice is the four-stage designed usage; the eleven-stage V-model has a separate default
flow and command set ([A10](../../index.md#a10)).

<a id="d53"></a>

## Repository file plan

![SimpleFlow implementation file plan](assets/simple-flow-file-plan.svg)

> **Figure D53-1 — SimpleFlow file plan.**  
> **Question:** Which repository files carry the run, its fixed checks, and its verification?  
> **Model view:** Focused file-plan projection of `XDesignDetailedDesign::simpleFlowFilePlanView`, exposing the native `simpleFlowFilePlan` item and its `repositoryPath` and `responsibility` values.  
> **Companions:** [D45](../../design/flows/simple-flow/implementation.md#d45) for module dependencies, [D06](../../design/flows/simple-flow/definition.md#d06) for the four checker rule sets, and [D54](../../verification/design.md#d54) for verification.  
> **Status:** renderable-now; model-declared file plan, with source and executable results checked separately.  
> **Reading:** Application sources, fixed checker commands, and tests are separate groups. The `repositoryPath` values below give exact filenames; labels in the figure stay short.  
> **Limits:** A planned path does not prove that a file exists, compiles, or fulfills its contract.

| Group | Repository files | Responsibility |
| --- | --- | --- |
| xdesign entry | `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `src/main_wish_tests.rs` | Build and expose the application library; provide a local command entry point and its wish-validation cases. |
| Core module | `src/core/mod.rs`, `src/core/status.rs`, `src/core/plan.rs`, `src/core/record.rs`, `src/core/ports.rs`, `src/core/gate.rs`, `src/core/run.rs`, `src/core/factory.rs` | Implement the flow-agnostic layer selected by the Rust UML design: vocabulary, plan data and structural validation, records, ports, gate evaluation, the run engine, and generic run construction. |
| SimpleFlow module | `src/simple_flow/mod.rs`, `src/simple_flow/blueprint.rs`, `src/simple_flow/factory.rs`, `src/simple_flow/check.rs` | Implement the designed four-stage flow on the core layer: the blueprint plan data and conformance, the designed-run factory, and the fixed-checker adapter. |
| Stage persistence | `src/core/restore.rs`, `src/stage_cli/mod.rs`, `src/stage_cli/store.rs`, `src/stage_cli/work.rs`, `src/stage_cli/persistent.rs` | Restore validated runs, collect work, and adapt new stage registrations and checks to stored-head revisions while preserving legacy runs. |
| Operational storage | `src/storage/mod.rs`, `src/storage/db.rs`, `src/storage/blobs.rs`, `src/storage/ops.rs`, `src/storage/cli.rs`, `src/storage/unicode.rs` | Implement SQLite format and queries, content-addressed materials, atomic idempotent revisions, recovery, doctor, the store command surface, and generated Unicode equivalence tables for manifest names. |
| Fixed gates | `tools/checks/common.py`, `tools/checks/simple_requirements.py`, `tools/checks/simple_design.py`, `tools/checks/simple_implementation.py`, `tools/checks/simple_record.py` | Share a JSON report contract and report the 2, 7, 2, and 7 fixed rules in [D06](../../design/flows/simple-flow/definition.md#d06). |
| Executable cases | `tests/simple_flow.rs`, `tests/plan_model_sync.rs`, `tests/process_check_contract.rs`, `tests/run_restore.rs`, `tests/stage_cli.rs`, `tests/storage.rs`, `tests/storage_cli.rs`, `tests/storage_paths.rs`, `tests/storage_protocol.rs`, `tests/storage_memory.rs`, `tests/checker_timeout.rs`, `tests/test_simple_flow_checkers.py`, `tests/test_simple_flow_design_layout.py` | Exercise run behavior, model-plan agreement, process and stage interaction, storage faults/recovery, terminal-receipt retry, destination ancestry, bounded checkout memory, checker process-tree deadlines, and checker results, including negative cases. |

The file-plan audit compares in both directions: every declared path must exist, and
every source or test file in `src/`, `tests/`, and `tools/checks/` that belongs to this
slice must have one declaration. The original entries cover the first source plane. The
[stage CLI plan](stage-cli.md) added restoration and the initial stage adapter; the
[storage plan](storage.md#p35) adds the persistent adapter, storage module (including the
generated Unicode table module) and the storage and checker-deadline test files to the same
model inventory. The Unicode tables themselves are generated at development time by
`tools/generate_unicode_tables.py`; generated commit content is not a slice source entry.
Planned paths become implementation evidence only when the files and their executed cases
exist.

The existing project model in `mbse/model/` defines stages, gates, records, views, and
planned verification. The existing documentation in `docs/` explains their contracts;
the compiler at `tools/sysml_web/bin/sysml`, `tools/check_documentation.py`, and
`tools/render_docs.py` check or render them. These are inputs and supporting tools for
the file plan, not replacements for its source files. Reports, previews, dependency
caches, and local development environments belong under ignored `cache/` as the root
`AGENTS.md` requires. [D86](../../design/storage.md#d86) allocates durable product evidence
to store revisions and content-addressed materials; a cache path by itself is not durable.

<a id="d55"></a>

## Evidence and limits

Retain each executed case's inputs, output report, source revision, and verdict under
`cache/` while developing; new store-backed runs also archive the required bytes in their
revisions. The bundled compiler checks syntax, name resolution, and its supported
model rules; it does not evaluate the 26 currently unevaluated constraint bodies or run
these cases ([T04](../../architecture/constraints.md#t04)). The fixed gates check declared structure,
record integrity, and artifact presence. Engineering adequacy remains a semantic
judgment at the applicable checkpoint, supported by retained evidence.
