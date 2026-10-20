# AGENTS.md

This file defines the project's purpose, core concepts, and Agent collaboration rules. It applies to the entire repository.

## Project Purpose

This project is a tool for managing engineering development with SysML v2. Users express wishes in their own projects, build system models, and define constraints on outputs. Through checks and intelligent judgment within the V-model, each wish progresses toward a delivery outcome supported by evidence.

The tool must answer two questions:

1. What makes up the system being developed, and how do its parts relate and work together?
2. Why did each wish lead to these requirements, designs, implementations, and verification results?

The project MBSE model expresses the first. An independent SysML traceability model for each wish expresses the second. The workflow organizes how these models and artifacts are produced and checked, who makes judgments, and when work may proceed.

## Directory Conventions

| Directory | Purpose |
| --- | --- |
| `mbse/` | SysML v2 models, including the project MBSE model and wish traceability models. |
| `src/` | Application source code. |
| `tools/` | Scripts and development utilities. |
| `docs/` | Project documentation. |
| `docs/prj_manage/` | Review synthesis, active implementation plans, and acceptance records. |
| `cache/` | Temporary data, local project environments, and temporary files. |

Store all local project environments, including development and test environments, under `cache/`. Configure tools to place all caches, temporary data, and temporary files there as well. The entire `cache/` directory is excluded from version control.

## Executable Flow Slices

The SimpleFlow implementation sequence is recorded in
`docs/prj_manage/02_simple_flow_implementation_plan.md`, and the executable
two-stage wish-verify slice that followed it in
`docs/prj_manage/09_wish_verify_flow_implementation_plan.md`. Read a plan's review basis,
source-file plan, verification cases, and acceptance boundary before changing the
corresponding flow's model, API, checker commands, or run code. Keep the plan and its linked
design documents current when a decision, source path, or verification result changes.
The local `cn_todos` template supplied a useful task/analysis/acceptance structure;
this repository uses the English plan in `docs/prj_manage/` as its project record.

The gated four-stage SimpleFlow and the gated two-stage wish-verify flow are the
executed executable slices.
Do not describe those slices as a complete V-model workflow: the default V-model gate
commands, generic controller integration, persistence, and project-level acceptance
have separate obligations. A declared verification case or a passing SysML compiler
check is not an executed engineering result; retain command logs and evidence under
`cache/` and state exactly what was tested.

## SysML v2 Compiler

Use the repository-bundled Rust-based `haradama/sysml-v2-rs` compiler. The selected release was sourced from the local xflow workspace; running this project does not require that workspace.

- Selected version: `v0.1.4`.
- Executable: `tools/sysml_web/bin/sysml`.
- Toolchain record: `tools/sysml_web/toolchain.json`.
- Executable SHA-256: `674fce28512a772d1886ff2397f5f02dda66265299c67c01f5cc36eaf00207f6`.

Run `parse` and `check` after model changes, including the related source files needed to resolve cross-file references. Keep compiler reports, exports, and temporary files under this project's `cache/`. Preserve diagnostics and unevaluated rules; passing compiler checks establishes only the checks actually performed, while engineering verification and validation require their own evidence.

## Language

Use English for all project content and Agent communication, including documentation, model names and descriptions, code identifiers and comments, scripts, and commit messages. Translate existing Chinese project content into English when updating it.
