# Cross-layer review reconciliation

> Review basis: `cache/kimi_review/REPORT.md` and its five detailed audits, dated
> 2026-10-20. That directory is local and ignored by Git; this page retains the
> dispositions in the project record. The earlier execution results in
> [P08](../executions/simple-flow.md#p08) and
> [P12](simple-flow-fixes.md#p12) remain revision-specific history.

<a id="p14"></a>

## Findings and changes

The review found three kinds of drift: statements that predated the running
SimpleFlow slice, declarations that no longer matched the source, and historical
wish evidence that is no longer available at its recorded URI.

| Review findings | Disposition |
| --- | --- |
| C1–C11, L2–L4, L6: stale implementation status and weak section descriptions | Scope status statements to the running four-stage SimpleFlow crate and checker commands. The default V-model commands and complete application remain open. Name `StageWork` and the annotated status enums in their target sections. |
| C12/S3: 17 declared slice files versus 21 actual files | Extend the model file plan and D53 inventory to include the four previously omitted source/test files, then require SF-11 to check both directions of the inventory. |
| S2 and the open P11 requirements-checker decision | Align declared reads and fail-closed cardinality with the commands and compare the frozen factory declarations to the model. Map the `retained plan serialization` read to the frozen plan URI in the process adapter; the first integrated rerun exposed that missing mapping. |
| S5–S8 and L1/L5: lifecycle, action, module, and API drift | Reconcile completed-run revalidation, action signatures, module dependencies, persistence fields, and documented Rust-side additions. Correct the API reference and inventory its crate-visible evidence query. |
| S4: wish-0001's unsupported production and missing files | Correct the activity's unsupported requirement-production statement. Label the wish model as a historical illustrative trace and retain the missing-evidence limitation. |
| S9: documentation URIs in earlier passed assessments | Keep the assessments append-only and disclose the historical-path limitation. A later assessment needs fresh evidence at a new revision if these wishes are reaccepted. |
| S10: modeled cardinality and unasserted behavior | Model the validated plan's five checkpoints and each checker's nonempty rules. Add cases for duplicate checkpoint identities, initially pending noninitial visits, and completed-visit artifact revalidation after reopening. |

<a id="p15"></a>

## Verification of this reconciliation

All command outputs below are retained under `cache/kimi_review/fixes/`.
The commands ran after the source, model, and documentation edits, except that
the figure and compiler checks do not depend on the later process-adapter fix.

| Check | Observed result | Local evidence |
| --- | --- | --- |
| Bundled SysML `parse` and model-only/full `check` | All 17 model and wish files parsed; model-only and full checks resolved 19,494/19,494 and 21,093/21,093 references, with zero violations. Both reported 129 held and 26 unevaluated rules. | `model_parse.json`, `model_only_check.json`, `model_check.json` |
| `cargo test --offline` and `cargo fmt --check` using the cache toolchain | 21 Rust tests passed: one CLI case, two model/sync cases, four process-adapter cases, and 14 run cases. Formatting passed. The new adapter case covers the frozen plan read. | `rust_tests_final.log`, `rust_fmt_final.log`, `process_check_regression.log` |
| Python `unittest` discovery under `tests/` | 22 tests passed. | `python_tests.log` |
| `cargo run --offline --quiet -- demo` with an explicit simulated human fixture | Requirements, design, implementation, and verification completed; all five gates passed. The first run blocked at the record gate because the adapter had not recognized the new plan read; the final run passed after that mapping was fixed. This is an interface exercise, not engineering acceptance. | `demo_final.log`, `demo_final/plan.json`, `demo_final/run.json`, `demo_final/reports/`; initial failure in `demo_success.log` |
| Documentation audit, figure render, nine-diagram verification, and strict site build | The audit found zero diagnostics across 221 `DocSection`, 26 `DocFigure`, and 235 anchors. All 34 figures rendered, all nine primary diagrams passed verification, and the strict site build completed. | `documentation_audit_final.json`, `figure_render.log`, `diagram_verification.log`, `docs_build.log` |

<a id="p16"></a>

## Acceptance boundary

The recorded `delivered` state and passed judgment in `wish-0001` are historical
values, not independently re-openable engineering evidence: `src/report.rs`,
`cache/build/report.xml`, and the revision-3 and revision-4 check logs are absent.
No substitute files were invented and no passed assessment was edited. Some passed
assessments in wishes 0005–0008 also cite document paths renamed by later work;
their cited paths are not re-openable from the current checkout. The SimpleFlow JSON
run checker does not audit these earlier SysML wish histories.

The four-stage implementation and its checks establish only the executable slice
described in [P03](../plans/simple-flow.md#p03). The remaining
evidence-chain and V-model obligations in [P13](simple-flow-fixes.md#p13)
still apply.
