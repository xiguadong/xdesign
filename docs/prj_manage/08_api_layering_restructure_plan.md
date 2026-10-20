# API layering restructure plan

> Status: completed. Model, documentation, and code steps each verified as
> executed; the record is [P23](#p23). Scope: split the xdesign software design,
> API reference, and Rust source into a generic `core` layer and the
> `simple_flow` layer built on it; rename `docs/uml/` to `docs/api/`; dissolve
> the C4/UML guide into its load-bearing parts. Model first, documentation
> second, code third — code is never changed before its model and API contract.

<a id="p20"></a>

## P20 Task and acceptance goal

Today every software-design element, API page, and Rust source file lives under
a SimpleFlow-only umbrella (`SimpleFlowRustUml`, `docs/uml/xdesign/simple_flow/`,
`src/simple_flow/`), although most of it — plan data, records, ports, reports,
status vocabulary, the run engine — is generic xdesign infrastructure. This plan
splits all three planes into `core` (flow-agnostic infrastructure, including the
run engine) and `simple_flow` (only the designed four-stage flow: its plan data,
blueprint conformance, factory, and fixed-checker adapter).

Acceptance: the SysML model expresses the two layers with one-way dependencies
(`simpleFlow` → `core`) and compiles clean; `docs/api/` routes every API from
one inventory and one crate-level structure figure, with one page per API under
`core/` or `simple_flow/`; the crate builds as `xdesign::core` plus
`xdesign::simple_flow`; every pre-existing behavior and checker test passes
unchanged in intent; all logs and reports are retained under `cache/`.

<a id="p21"></a>

## P21 Basis and decisions before the change

Decisions fixed with the requester:

1. **Layer allocation.** `core` owns: status/error vocabulary, plan
   specifications and structural validation, records and evidence hashing, the
   `RsCheckCommand`/`RsJudge` trait boundaries and report types, the generic run
   engine (renamed `RsFlowRun`), gate evaluation, and `create_run`.
   `simple_flow` owns: `designed_simple_flow_plan` (the blueprint as data),
   `validate_designed_plan` (blueprint conformance), `create_designed_run`
   (conformance plus generic construction), and `RsProcessCheck` (the adapter
   bound to the four fixed checker commands; its read-coverage policy encodes
   this flow's contract and is not generic).
2. **Engine generalization.** `RsSimpleFlowRun` becomes `RsFlowRun` realizing
   `XDesignArchitecture::FlowExecution`; `reopen_stage` keys on the plan's
   `completes_flow` flag instead of the hardcoded `"verify"` stage;
   `RsFlowPlan::validate` keeps only structural checks — the blueprint equality
   and `plan_id` checks move to `validate_designed_plan`; the run record's
   `execution_type` becomes `"FlowRun"` (the record checker rule
   `designed-runs-are-typed-by-the-design-class` is reworded accordingly in
   model, plan data, and checker together).
3. **Documentation shape.** `docs/uml/` is renamed to `docs/api/`. Its
   `00_overview.md` is the single entry: routing, the project boundary sections
   (U65–U67 folded in), the complete crate inventory grouped `core` first
   (U13–U15 moved up), and the API-reference construction rules (U47 moved
   here). `01_struct.md` carries exactly one crate-level package/dependency
   projection rendered from the model (new section U68); class-level relations
   stay in the module design document. Pages live under `core/` and
   `simple_flow/`; section identifiers (U16–U64) move with their pages and never
   change; new pages take new identifiers from U69.
4. **Guide dissolution.** `docs/uml/01_c4_and_uml_guide.md` is deleted. No C4 or
   UML-notation figure exists in the repository — every figure is a SysML
   projection — so the C4/UML view catalogs (U06–U08) and drawing rules (U11)
   are removed as unused. U47 moves to `docs/api/00_overview.md`; the
   view-applicability obligation (U09) and the figure rules (U10) fold into
   `docs/details/05_documentation_contract.md`; the surviving review items of
   U12 fold into `docs/review/02_checklist.md`. The `U` section prefix is
   retained for moved sections per the identifier-stability rule.
5. **Order.** Model (`mbse/model/uml.sysml` and its two importing files) before
   documentation (`docs/api/`, conventions, navigation, links) before code
   (`src/`, `tests/`, `tools/checks/`). Each plane is verified before the next
   begins; documentation checks validate model↔document consistency after the
   second step.

<a id="p22"></a>

## P22 Implementation sequence and file ownership

| Step | Work product | Completion check |
| --- | --- | --- |
| 1. Restructure the model | `mbse/model/uml.sysml`: package `SimpleFlowRustUml` renamed `XDesignRustUml`; `crate::core::{status,plan,record,ports,gate,run,factory}` and `crate::simpleFlow::{blueprint,factory,check}` layout with one-way dependencies; `RsFlowRun` realizes `FlowExecution`; `RsCreateDesignedRun` realizes `SimpleFlowRunFactory::createRun`; new `RsDesignedSimpleFlowPlan` and `RsValidateDesignedPlan` action defs; all `DocSection` paths point at `docs/api/core/` or `docs/api/simple_flow/`; new `rustCrateLayers` view carrying the `docs/api/01_struct.md` locator; `rustModuleLayout`/`rustClassStructure` expose the new layout. Import updates in `traceability.sysml` and `verification.sysml`. | Pinned compiler parse and check of `mbse/model` and `mbse/wishes` pass; reports preserved under `cache/sysml/`. |
| 2. Rebuild the documentation | `docs/api/{00_overview.md,01_struct.md,core/,simple_flow/}`; 46 pages re-homed under `core/` (01–46) and 4 under `simple_flow/` (01–04, two of them new: U69 `validate_designed_plan`, U70 `create_designed_run`); guide dissolved per P21.4; `docs/AGENTS.md`, `docs/00_index.md`, `mkdocs.yml`, and every cross-link updated; `docs/details/simple_flow/06_rust_uml_design.md` reworded to the two-layer crate; figures regenerated. | `python3 tools/check_documentation.py --json` clean; `tools/render_docs.py` and `tools/verify_diagrams.py` clean; strict site build passes; logs under `cache/`. |
| 3. Restructure the code | `src/core/{status,plan,record,ports,gate,run,factory}.rs` and `src/simple_flow/{blueprint,factory,check}.rs`; `src/lib.rs` exports both modules; `src/main.rs`, `src/main_wish_tests.rs`, and `tests/` updated to the new paths and names; `tools/checks/simple_record.py` reworded rule. | `cargo build` and the full Rust + Python test suite pass with build directories under `cache/`; behavior unchanged apart from the generalized `reopen_stage` and the `execution_type` value. |
| 4. Record the result | This plan's P23 filled with the executed commands, exit codes, and remaining limits. | Every claimed check names its retained log. |

<a id="p23"></a>

## P23 Verification and record of completion

Executed on 20 October 2026, in the plan's order. Every row names its retained
log; an exit code of 0 is the recorded run, not an inference.

**Step 1 — model.** Baseline captured first (`cache/sysml/parse.baseline.json`,
`cache/sysml/check.baseline.json`). After the restructure:
`parse mbse/model/*.sysml mbse/wishes/*.sysml` exit 0, 0 errors
(`cache/sysml/parse.json`, `parse.stderr.log`); `check mbse/model mbse/wishes`
exit 0, 0 unresolved/collisions/violations, rules 129 held / 26 unevaluated —
identical counts to baseline (`cache/sysml/check.json`, `check.stderr.log`).
The new `rustCrateLayers` view was probe-rendered to
`cache/sysml/rustCrateLayers.probe.svg` (crate/core/simpleFlow nodes and the
one-way dependency edge present). A follow-up edit generalized `RsFlowPlan`'s
stage/checkpoint multiplicities and repointed its realization to
`XDesignArchitecture::Flow`; check re-run, exit 0.

**Step 2 — documentation.** `python3 tools/check_documentation.py --json` exit
0, `ok: true`, 0 diagnostics (`cache/docs-render/documentation-check.json`).
`python3 tools/render_docs.py` exit 0, 37 figures regenerated including the new
`docs/api/assets/01_crate_layers.svg` (`cache/docs-render/render_docs.log`).
`python3 tools/verify_diagrams.py` 9/9 PASS
(`cache/docs-render/verify_diagrams.log`, manifest
`cache/docs-render/review/figure-manifest.json`; its `sharp` runtime was
reinstalled under `cache/diagram-tooling/node/node_modules/`). Strict site
build `python3 tools/docs.py build` exit 0 (`cache/docs-render/docs_build.log`).
Reference sweep proof: no `docs/uml` / `uml/xdesign` reference remains outside
historical wish records and this plan.

**Step 3 — code.** The coupled model/doc rule-text edit
(`designed-runs-are-typed-by-the-design-class` → `FlowRun` typing) and the
`simpleFlowFilePlan` source-plane entries landed in
`mbse/model/detailedDesign.sysml` first; sysml check, figure render, and
documentation check all re-run with exit 0. Then: `cargo build` clean, no
warnings (`cache/implementation/cargo-build.log`); `cargo test` 21 passed, 0
failed — same count as the pre-change baseline
(`cache/implementation/cargo-test.log`); Python suite 22 tests OK via
`python3 -m unittest discover -s tests -p "test_simple_flow_*.py"`
(`cache/implementation/pytest-checkers.log`); `cargo fmt --check` clean
(`cache/implementation/rust-fmt.log`). One end-to-end demo run completed all
five gates, exit 0 (`cache/implementation/demo-run.log`):
`xdesign simple-flow demo --wish-model mbse/wishes/wish-0009.sysml --plan
cache/implementation/demo/plan.json --record cache/implementation/demo/run.json
--verify-evidence cache/implementation/demo/verification-evidence.txt
--judge-authority human --judge-identity "demo reviewer" --judge-outcome passed
--judge-rationale "…"`. Final sweep after the last doc edits:
`check_documentation.py` exit 0
(`cache/docs-render/documentation-check-final.json`), strict site build exit 0
(`cache/docs-render/docs_build_final.log`). Toolchain: the repo-bundled Rust
toolchain under `cache/toolchain/`, with `CARGO_HOME`/`CARGO_TARGET_DIR` under
`cache/`.

**Remaining limits.** The `core` layer is exercised only through the designed
simple flow; no second flow proves its generality yet. `RsProcessCheck` remains
deliberately flow-bound; a second flow supplies its own `RsCheckCommand`
adapter. The generic controller, wish/trace ownership, and project-level API
remain modeled-only ([U66](../api/00_overview.md#u65)). The compiler still does
not evaluate the 26 unevaluated constraint bodies; executable tests carry those
invariants. This restructure changes organization and the six documented
behaviors (P22 step 3); it does not extend the delivered V-model scope.

<a id="p24"></a>

## P24 Risks and recovery

| Risk | Control or recovery |
| --- | --- |
| The rename `RsSimpleFlowRun` → `RsFlowRun` and the page moves break a missed reference | Compiler check, documentation check, and test suite run after each plane; grep for the old paths before claiming completion. |
| Blueprint conformance silently weakens when it leaves `validate` | `validate_designed_plan` keeps every blueprint assertion byte-for-byte; `tests/simple_flow.rs` counterexamples still exercise them through `create_designed_run`. |
| Historical wish records reference old names | `mbse/wishes/` prose is historical and stays; only resolving references (imports, element names) are updated, and the compiler check over `mbse/model mbse/wishes` proves resolution. |
| The restructure is reversible only as a whole | Land model, docs, and code in one commit sequence on a branch; each step's verification runs before the next starts. |
