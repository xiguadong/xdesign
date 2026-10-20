# SimpleFlow execution record

> Scope: executable four-stage `designedSimpleFlow` slice. This record accompanies
> the implementation commit that contains the Rust crate, four Python checkers,
> model rule correction, and their documentation. Local command output is kept
> under ignored `cache/implementation/`.

<a id="p08"></a>

## P08 Executed acceptance

The [implementation plan](02_simple_flow_implementation_plan.md#p03) was exercised
against the repository source, the frozen plan serialization, and the current
design/model files. The actual run used `mbse/wishes/wish-0009.sysml` as its wish
identity and an explicit **simulated fixture judgment** for the semantic verify
gate. That judgment demonstrates the authority and evidence exchange; it is not
an engineering acceptance of a user's developed system.

| Check | Executed result | Local evidence |
| --- | --- | --- |
| Rust crate and run behavior | `cargo test --offline`: 10 tests passed; `cargo fmt --check` passed | `cache/implementation/rust_tests_final.log`, `cache/implementation/rust_fmt_final.log` |
| Four fixed commands and bad-input behavior | 13 Python tests passed; exact 2 + 7 + 2 + 7 rule IDs tested, plus targeted false rules, missing checker bindings, wrong requirement targets, reused figures, and unevaluated inputs | `cache/implementation/python_checker_tests.log` |
| Integrated four-stage command | Requirements, design, implement, and verify completed in order; all five gate assessments passed, including record then supplied semantic judgment; the retained record audit reports matching current artifact bytes and snapshot digests, but its five-line output has no retained audit script | `cache/implementation/demo/final_run.json`, `final_plan.json`, `final_demo.log`, `final_record_audit.log`, and the referenced reports |
| File plan and rule mapping | All 17 model-declared source paths exist; the seven Rust modules are exported; all 18 rule IDs match the four command files; the API directory contains one inventory page and 47 per-API pages, and the file-plan figure exists | `cache/implementation/file_plan_audit.log` |
| Pinned SysML compiler | Parse succeeded for 17 source files; model-only check resolved 19,451/19,451 references and full check 21,050/21,050, with no violations, 129 held and 26 unevaluated rules | `cache/implementation/sysml/` JSON and stderr logs |
| Documentation and figures | Documentation locator audit, figure generation, and nine-primary-view verification passed. The retained site build output reports success, but does not record its command and exit code; a new strict build is recorded in [P12](04_simple_flow_review_fix_record.md#p12). | `cache/implementation/` audit, render, diagram, and build logs |

The CLI example is a local command boundary. It requires an existing wish SysML
path, a verification evidence file, and explicit judge authority, identity,
outcome, and rationale. It freezes the plan in JSON, records input file digests
for each visit, retains one report per syntactic gate, and writes a JSON run
record. The verify stage presents its own evidence and the preceding record
report to the supplied judge.

<a id="p09"></a>

## P09 Case disposition

The case IDs and expected behavior are defined in [D54](06_file_plan_and_verification.md#d54).

| Cases | Executed evidence | Result |
| --- | --- | --- |
| SF-01–SF-04 | Rust malformed-plan and initial/stage-scope/mixed-verify tests; integrated run | Selected cases passed. Duplicate checkpoint IDs, only-requirements-active, and per-stage assessment isolation were not each asserted directly. |
| SF-05–SF-06 | Rust malformed-report, authority, and latest-verdict tests; Python bad-input and rule mutation tests | Passed, including pass→fail and pass→unevaluated revocation |
| SF-07–SF-09 | Rust content-revision, blocked-correction, terminal-reopen, plan-drift, and persistence-failure tests | Selected cases passed. The original tests did not assert retained completed-visit evidence after reopen or a second completion. |
| SF-10 | Thirteen Python command tests plus four real reports from the integrated run | Passed for the declared 18-rule set |
| SF-11 | Source-path mapping, bundled compiler parse/check, documentation audit, figure render/verification, site build output | Source, compiler, audit, and figure checks passed for the checked tree. The original site build lacks a retained invocation and exit code; compiler constraint bodies remain unevaluated. |

<a id="p10"></a>

## P10 Boundary of the result

This result establishes a runnable SimpleFlow **slice** with fixed checks,
revision-bound visits, strict command reports, and a supplied semantic judgment
boundary. The default eleven-stage V-model still lacks its three checker
commands. The new JSON record does not audit historical `mbse/wishes/*.sysml`
traces internally, and the example judgment is a fixture assertion rather than
independent engineering verification. The ICD has no owner-approved remote
protocol or identity proof. Visit artifacts have byte digests, but assessment
reports and other evidence are recorded by URI without a separate immutable
evidence digest; a later edit to one of those files is not detected by this slice.
Evidence under ignored `cache/` is local development evidence; durable
multi-user retention and delivery approval remain open.

The bundled SysML compiler resolved names and checked supported model rules, but
did not evaluate 26 constraint bodies. The passed code tests establish their
specified cases only; they do not establish every constraint or the adequacy of
requirements, design, and implementation for a real user's wish.
