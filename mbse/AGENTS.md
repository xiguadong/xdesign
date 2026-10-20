# AGENTS.md

This file defines the model layout under `mbse/`. The repository's root `AGENTS.md` also applies here.

## Model File Structure

Store this project's MBSE model under `mbse/`. Definitions live in `mbse/model/`, and each wish's independent traceability model lives in `mbse/wishes/`:

```text
model/
├── common.sysml          → Units, types, and metadata.
├── requirements.sysml    → All requirements; nested packages may organize them.
├── architecture.sysml    → All architecture: logical, physical, and decisions.
├── detailedDesign.sysml  → All detailed design.
├── uml.sysml             → Software (UML) designs of designed flows, including
│                           the xdesign crate with its SimpleFlow and wishVerify
│                           modules.
├── composition.sysml     → The xdesign instance and its selectable flow portfolio.
├── verification.sysml    → All verification cases.
└── traceability.sysml    → All traceability relationships in the project model.

wishes/
└── wish-0001.sysml       → One wish: its Wish, Trace, stage records, activities,
                            assessments, journey and views. Only local journey
                            rendering definitions are allowed here.
```

## Model Organization

- Keep shared units, types, and metadata in `common.sysml` and reference them from the other model files.
- Organize requirements into nested packages inside `requirements.sysml` as needed.
- Define model elements in their respective files and reference them from `traceability.sysml` when expressing traceability relationships.
- Keep software designs in `uml.sysml`: Rust or other language-targeted types, traits, functions and module layouts that realize detailed-design elements, each recording the element it realizes as a `dependency`. A software design claims no implementation that does not exist in the repository.
- Keep reusable flow definitions in their architecture and detailed-design files; instantiate selectable flows under the `xdesign` portfolio in `composition.sysml`. The abstract `xdesign.'flow'` is their common origin, while `xdesign.selectedFlow` names the default portfolio member.
- Classify every checkpoint with `kind`. A syntactic checkpoint names the `Checker`s that fix its check and sets no judge flag, because a script judges it. A semantic checkpoint names no checker and sets `agentMayJudge` or `humanMayJudge`, because its criteria need judgment. Record the judging authority of every assessment in `judgedBy`; a syntactic checkpoint is judged by a script and a semantic one by an agent or a human.
- Write requirements into `requirements.sysml`, each carrying its `rationale`, outwards from what the tool must do and why. The wish that asked for a requirement belongs to that wish's own record; a requirement never names one.
- Retain the products of a wish — the requirements it produces and the model modifications it causes — in `mbse/model/`. A wish's record may describe them, but it must never hold the only copy.
- Keep the dependency one way. No model element carries a wish's identity, and no file under `mbse/model/` references `mbse/wishes/`; a wish references model elements instead. Record a wish's derivation as a `dependency` from the wish to the requirements it produced. `check mbse/model` on its own must pass, which is how this stays true.
- Maintain an independent SysML traceability model for each wish, as required by the root `AGENTS.md`. These models reference relevant project model elements.
- Record each wish in `mbse/wishes/wish-<id>.sysml` as its own package. Declare usages and values there, referencing the product and workflow definitions in `mbse/model/` instead of restating them. A local `occurrence def` may bind lifelines for a wish journey view; it must not define a project product.
- A wish model records the run: one `StageRecord` per visited stage with its `visitIndex` and `artifactRevision`, the `ActivityRecord`s of that visit, and the `CheckpointAssessment`s with their evidence URIs. An assessment's `evaluatedRevision` must equal the recorded `artifactRevision` it judged, and a stage already marked completed must stay authorized, so record new evidence as a further assessment rather than editing a passed one.
- A historical illustrative trace with missing evidence, such as wish-0001, cannot establish acceptance even if its retained status says `delivered`. Identify its missing evidence in the trace and review record. New passed assessments must name retained, re-openable evidence.
- Present a wish as a journey by binding `ref part` lifelines to the real participants (`xdesign.controller`, `xdesign.<selectedFlow>.<stage>`) and anchoring `message` usages on their `entering`/`entered`, `assessing`/`assessed` and `delivered` event occurrences.
- Use English for all model names, descriptions, and comments.

## Compiler Checks

Use the Rust SysML v2 compiler selected in the root `AGENTS.md`. Once the model files exist, run the following commands from the repository root:

```sh
sysml_compiler="$PWD/tools/sysml_web/bin/sysml"
mkdir -p cache/sysml/tmp
TMPDIR="$PWD/cache/sysml/tmp" "$sysml_compiler" --format json --color never parse mbse/model/*.sysml mbse/wishes/*.sysml > cache/sysml/parse.json 2> cache/sysml/parse.stderr.log
TMPDIR="$PWD/cache/sysml/tmp" "$sysml_compiler" --format json --color never check mbse/model mbse/wishes > cache/sysml/check.json 2> cache/sysml/check.stderr.log
```

Include any additional `.sysml` or `.kerml` dependencies and relevant wish models in the same check invocation when required. Inspect exit codes and report contents, including unevaluated rules, before recording a check result.

`check` resolves names and applies the specification's rules, but it does not evaluate `assert constraint` bodies. A constraint whose expression is false is still reported as passing, so a clean check does not establish this model's own invariants — an evidence-backed passed assessment, subflow completion before its parent stage, one trace per wish, and so on. Treat those as obligations to verify separately and record how they were verified.

Renderings that work with this compiler:

```sh
"$sysml_compiler" --color never diagram --internal=<name> mbse/model mbse/wishes   # flow, trace or controller structure
"$sysml_compiler" --color never diagram --browser mbse/model mbse/wishes           # membership tree, including record values
"$sysml_compiler" --color never diagram --sequence=<name> mbse/model mbse/wishes    # a journey's lifelines and messages
```

A sequence rendering needs the target definition to declare `ref part` lifelines whose types own the `event occurrence`s that the `message` usages anchor on; a definition without them reports no interaction to draw. Nested features of a usage are not addressable as render targets.
