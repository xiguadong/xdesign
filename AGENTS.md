# AGENTS.md

This file defines the project's purpose, core concepts, and Agent collaboration rules. It applies to the entire repository.

## Project Purpose

This project is a tool for managing engineering development with SysML v2. Users express wishes in their own projects, build system models, and define constraints on outputs. Through checks and intelligent judgment within the V-model, each wish progresses toward a delivery outcome supported by evidence.

The tool must answer two questions:

1. What makes up the system being developed, and how do its parts relate and work together?
2. Why did each wish lead to these requirements, designs, implementations, and verification results?

The project MBSE model expresses the first. The operational store expresses the second for new store-backed runs; independent SysML traces remain the authority for legacy and bootstrap runs. The workflow organizes how these models, records, and artifacts are produced and checked, who makes judgments, and when work may proceed.

## Start a task

Read docs/index.md for the document map. Use the matching route and read the owning page before editing:

| Work | Read first | Verify with |
| --- | --- | --- |
| Requirements or acceptance conditions | docs/requirements/requirements.md and docs/verification/requirements.md | compiler checks; docs audit |
| Architecture, composition, lifecycle, or constraints | docs/architecture/index.md and the owning system page | compiler checks; docs build |
| Flow model, run lifecycle, gates, evidence, or adapters | docs/design/flow.md, lifecycle.md, evidence-and-revisions.md, adapters.md | compiler checks; cargo tests; docs audit |
| SimpleFlow or WishVerifyFlow definition, implementation, or views | docs/design/flows/<flow>/index.md, then definition.md, implementation.md, views.md | compiler checks; cargo tests; design checker; docs build |
| Operational storage, revisions, materials, receipts, or recovery | docs/design/storage.md, docs/interfaces/store-format.md, and docs/records/plans/storage.md | cargo tests; storage fault/recovery cases; docs audit |
| An API design contract or source behavior | docs/design/api/index.md and the per-API page, then the owning flow implementation | cargo tests; docs audit when docs change |
| A SysML model or model-derived figure | mbse/AGENTS.md, docs/interfaces/documentation-contract.md, docs/architecture/constraints.md | sysml parse/check; render and verify figures |
| Documentation structure or prose | docs/AGENTS.md, docs/index.md, and the owning page | docs audit; strict docs build |
| Plans, reviews, executions, or wish history | docs/records/index.md and the relevant record | do not rewrite historical results; link current records |

Plans for the executed slices are in docs/records/plans. They record the verified sequence and acceptance evidence; current design claims live in the design pages and current requirements/verification pages.

## Directory Conventions

| Directory | Purpose |
| --- | --- |
| mbse/ | SysML v2 models: project definitions and legacy/bootstrap wish traces. Store-backed runs do not add Git wish traces. |
| src/ | Application source code. |
| tools/ | Scripts and development utilities. |
| docs/ | Authored documentation: requirements, architecture, design (common pages plus flow-specific pages and one document per API), interfaces, verification design, and records. Entry: docs/index.md; rules: docs/AGENTS.md. |
| docs/records/ | Plans, reviews, execution records, evidence summaries, current status, and legacy/bootstrap wish history. |
| .xdesign/ | Durable local operational store: new wish/run revisions, content-addressed materials, evidence, and receipts. Ignored by Git; not a development environment or temporary cache. |
| cache/ | Temporary data, local project environments, regenerable working material, and temporary files. |

Store all local project environments, including development and test environments, under cache/. Configure tools to place all caches, temporary data, and temporary files there as well. The entire cache/ directory is excluded from version control. Durable operational bytes belong in .xdesign/, whose default location can be overridden with XDESIGN_STORE_ROOT.

## Executable Flow Slices

The SimpleFlow plan is docs/records/plans/simple-flow.md; the executable two-stage WishVerifyFlow plan is docs/records/plans/wish-verify.md; the storage redesign is docs/records/plans/storage.md. Their verification design is in docs/verification/design.md, and their executed results are in docs/records/executions/ and the plan records. Read the plan's review basis, source-file plan, verification cases, and acceptance boundary before changing the corresponding flow's model, API, checker commands, storage, or run code. Keep the plan and its linked design/verification documents current when a decision, source path, or verification result changes.

The gated four-stage SimpleFlow and the gated two-stage wish-verify flow are the executed slices. Do not describe them as a complete V-model workflow: the default V-model gate commands, generic controller integration, whole-store backup/migration, and project-level acceptance have separate obligations. A declared verification case or a passing SysML compiler check is not an executed engineering result; retain command logs and evidence under cache/ and state exactly what was tested.

## SysML v2 Compiler

Use the repository-bundled Rust-based haradama/sysml-v2-rs compiler. The selected release was sourced from the local xflow workspace; running this project does not require that workspace.

- Selected version: v0.1.4.
- Executable: tools/sysml_web/bin/sysml.
- Toolchain record: tools/sysml_web/toolchain.json.
- Executable SHA-256: 674fce28512a772d1886ff2397f5f02dda66265299c67c01f5cc36eaf00207f6.

Run parse and check after model changes, including the related source files needed to resolve cross-file references. Keep compiler reports, exports, and temporary files under this project's cache/. Preserve diagnostics and unevaluated rules; passing compiler checks establishes only the checks actually performed, while engineering verification and validation require their own evidence.

## Documentation And Verification Commands

    python3 -B tools/check_documentation.py --json
    python3 -B tools/docs.py build
    tools/sysml_web/bin/sysml parse mbse/model/*.sysml mbse/wishes/*.sysml
    tools/sysml_web/bin/sysml check mbse/model
    tools/sysml_web/bin/sysml check mbse/model mbse/wishes
    cargo test
    python3 -m unittest discover -s tests -p "test_simple_flow_*.py"
    python3 -m unittest discover -s tools/sysml_web/tests -p "test_*.py"

If cargo is not on PATH in this workspace, prepend cache/toolchain/stable-aarch64-apple-darwin/bin, use CARGO_HOME=cache/cargo-home, and pass --offline for the cached dependency set. Run the checks that the changed artifacts actually require. Keep logs under cache/ and state the tested revision and limits. Do not treat a docs audit or compiler pass as engineering acceptance.

## Language

Use English for all project content and Agent communication, including documentation, model names and descriptions, code identifiers and comments, scripts, and commit messages. Translate existing Chinese project content into English when updating it.
