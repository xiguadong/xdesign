# SimpleFlow review fixes and follow-up verification

> Scope: the gated four-stage SimpleFlow slice. The consolidated local review is
> `cache/review-simpleflow/implementation_review.md`; this page retains the
> decisions and executed results in version control. The earlier acceptance
> record [P08](03_simple_flow_execution_record.md#p08) remains a historical
> snapshot of its original run.

<a id="p11"></a>

## P11 Review findings and disposition

The original review identified H1–H4 and M1–M5. A follow-up reproduced seven
specific defects, F1–F7, before changing source. The fixes below were applied
to the implementation and checked with regression cases.

| Findings | Change | Executed check |
| --- | --- | --- |
| H1, F2: forged run records bypass the record gate | Validate run, visit, and assessment fields and lifecycle order before the seven record rules; use the current-visit index when selecting evidence. | Checker mutations reject malformed statuses, revisions, outcomes, pending work, and a completed run with pending visits; valid completion and reopen histories pass. |
| H2, F4: declared requirement and satisfy forms are missed | Inspect requirement usages including specialization, abstract, and nested package forms; inspect every satisfy link in the declared traceability file. | Python regressions reject missing rationale and dangling links outside `XDesignTraceability`, and resolve qualified nested targets. |
| H3, F3: top-level design declarations evade `DocSection` | Scan immediate package members by brace depth after the pinned parser validates syntax; reject unclassified forms and require directly owned annotations. | Two-space, tab, multiline, abstract, attribute, and calc declaration regressions fail without their own section. |
| H4: `RemainBlocked` can leave the run active | Put the run and current visit in `Blocked` on every blocked advancement path, including record-write failures. | Rust persistence-failure cases assert the returned outcome and both statuses. |
| F1 and the historical artifact timing gap: completion can cite missing or changed earlier evidence | Recheck passed assessment evidence URIs and current/completed artifact bytes before advancement, including repeated advancement of a completed run. | Rust cases remove current, prior, and verify evidence reports, and edit a prior artifact after gate evaluation. |
| F5: activity URI list can disagree with its snapshot | Require changed activity URIs to equal the digest and removal URI set; check named unchanged files exist. | Rust cases reject mismatches, duplicates, and missing unchanged files. |
| F6, part of M1/M3: a syntactic report can omit its declared inputs | Hash the wish, plan, run record, and every passed rule input; the adapter rehashes them, checks declared reads, and requires all four command paths for design command rules. | Real requirements and design process reports pass; forged reports with unrelated, incomplete, or omitted command inputs fail. |
| F7, part of M2: frozen rule conditions differ from the model | Copy the four design rule conditions from SysML and compare every factory rule condition with the model in a Rust regression. | All 18 factory rule conditions match the current model declarations. |
| M4: the demo freezes an evolving wish into a completed artifact snapshot | Keep the wish URI as the run's independent identity; snapshot only the requirements checker's declared model inputs. | A Rust demo-path case edits the wish after requirements advancement and verifies the completed visit snapshot still matches. |
| M5: original acceptance claims exceed retained evidence | Correct the P08/P09 page count, log references, site-build provenance, and case coverage statements. | The new verification below records commands and results separately from the original run. |

The requirements checker retains its fail-closed requirement for at least one
requirement and one satisfy link. The literal universal wording in D06 does not
state that cardinality; aligning the model condition with this safeguard is a
separate model decision.

<a id="p12"></a>

## P12 Executed verification

The local commands and outputs are retained under
`cache/review-simpleflow/fixes/`. The positive command run used
`mbse/wishes/wish-0009.sysml` and an explicit simulated human fixture judgment;
it demonstrates the command and record path, not engineering acceptance.

| Check | Result | Local evidence |
| --- | --- | --- |
| `cargo test --offline` with the bundled cache toolchain | 19 Rust tests passed: 14 run cases, three process-adapter cases, one model-condition case, and one demo wish-snapshot case. | `rust_tests_final.log` |
| `cargo fmt --check` | Passed. | `rust_fmt_final.log` |
| Python `unittest` discovery under `tests/` | 22 tests passed. | `python_tests_final.log` |
| `cargo run --offline --quiet -- demo` with explicit fixture judgment | Requirements, design, implement, and verify completed; all five gates passed; the requirements visit contains the two declared model sources and excludes the wish URI. | `demo_success.log`, `demo/plan.json`, `demo/run.json`, and `demo/reports/` |
| Bundled SysML compiler `parse` and `check` | All 17 model/wish files parsed; model-only and full checks resolved 19,451/19,451 and 21,050/21,050 references, with no violations. Both checks report 129 held and 26 unevaluated rules. | `sysml/full_parse.json`, `sysml/model_check.json`, `sysml/full_check.json`, and stderr logs |
| Documentation audit, figure render/verification, and strict site build | Audit returned `ok: true` with zero diagnostics; all nine primary views passed verification; `tools/docs.py build` completed with `--strict`. | `docs_audit.json`, `render_docs.log`, `diagram_verification.log`, `docs_build.log`, `verification_commands.md` |

<a id="p13"></a>

## P13 Remaining boundary

The report adapter authenticates reported input bytes and coverage; it does not
bind the checker command, shared harness, compiler executable, or Git source
revision into the retained report. For model-derived directory reads such as
`docs/`, a replaced checker could still misreport which files it inspected.
Passed assessment evidence is required to exist, but its content has no separate
immutable digest in the run record. These are the remaining M1 evidence-chain
obligations, not claims established by the tests above.

The four-stage demo remains a local command exercise with a simulated judgment;
the automated suite exercises the real process adapter for requirements and
design but does not run all four stages in one test.
Default V-model checkers, historical SysML wish-trace auditing, durable multi-user
retention, and project-level engineering acceptance remain outside this slice.
