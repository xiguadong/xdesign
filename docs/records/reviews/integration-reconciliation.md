# xdesign flow and SimpleFlow integration reconciliation

> Status: implementation and local checks completed on 20 October 2026.
> Evidence is under ignored `cache/validation/`; these results do not constitute
> project-level engineering acceptance.

<a id="p17"></a>

## Decisions and source ownership

The [wish-to-delivery overview](../../architecture/wish-lifecycle.md#b30) now presents the
default eleven-stage route, its exit gates, and verification links in one model-derived
graph. It shows a dependency and authorization sequence; no dates or durations have
been assigned. The [flow portfolio](../../design/flow.md#f03)
defines the selectable alternatives.

`mbse/model/composition.sysml` owns the single `xdesign` usage. Its abstract
`xdesign.'flow' : Flow` is the common origin of six concrete usages, including
`xdesign.vModel` as the default and `xdesign.designedSimpleFlow` as the executable
four-stage selection. `DesignedSimpleFlow` is a reusable definition in
`mbse/model/detailedDesign.sysml`; it is not a separately owned flow. The wish models
now point structurally to portfolio members. Their recorded activities and assessments
were preserved. The SimpleFlow design checker reads the detailed design and the new
composition file as separate inputs.

The Rust package is `xdesign`. Its first implemented library module is
`xdesign::simple_flow` in `src/simple_flow/`, exposed by `src/lib.rs`; the command
entry is `xdesign simple-flow demo`. The [xdesign API boundary](../../design/api/index.md#u65)
owns the crate inventory, and the [SimpleFlow API pages](../../design/api/simple-flow/index.md#u72)
are nested under it. `XDesignRustUml::XDesignCrate` models this containment.

The [source file plan and acceptance cases](../plans/simple-flow-file-plan.md#d53)
are in project management, with the generated file-plan figure beside that record.
The plan declares 22 source, checker, and test paths. Existing D53–D55 anchors were
retained so model locators and review links remain stable.

<a id="p18"></a>

## Executed local checks

| Check | Observed result | Retained local log or output |
| --- | --- | --- |
| Pinned SysML compiler | Parse passed for 18 model and wish files. Model-only check resolved 19,535/19,535 references; full check resolved 21,143/21,143. Both reported 129 held and 26 unevaluated rules. | `cache/validation/sysml/parse.json`, `check_model.json`, `check_full.json` and paired stderr logs |
| Rust behavior and file-plan tests | 21 tests passed with the bundled local Rust toolchain; the source inventory matched the 22 declared file-plan paths. | `cache/validation/cargo_test.log` |
| Python checker and layout tests | 22 tests passed, including the design checker reading the composition file. | `cache/validation/python_tests.log` |
| Local four-stage command | Requirements, design, implementation, and verification advanced in order; five gates passed. The semantic pass was explicitly supplied by a fixture reviewer. | `cache/validation/demo/cli.log`, `plan.json`, `run.json`, `reports/` |
| Documentation and figures | Locator/layout audit passed; 35 figures regenerated; nine primary diagrams passed presentation checks; strict documentation site build passed. | `cache/validation/docs_check.json`, `render_docs.log`, `verify_diagrams.log`, `figure-review/figure-manifest.json`, `docs_build.log` |

The pinned compiler does not evaluate the reported constraint bodies. The local CLI
run exercised wiring with a simulated judgment and does not establish engineering
verification or validation of a user's product.

<a id="p19"></a>

## Remaining integration boundary

The xdesign crate contains an executable SimpleFlow module, while the generic
controller API, durable persistence, and the three fixed VModel gate commands remain
unimplemented. The [API overview](../../design/api/index.md#u66) records that
boundary. No default VModel end-to-end run or project-level acceptance is claimed.
The model-derived overview shows the intended route, and each wish trace supplies
the actual evidence for its selected flow.
