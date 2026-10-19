# SimpleFlow implementation plan

> Status: first executable slice completed; results in [P08](03_simple_flow_execution_record.md#p08).
> Scope: one executable run of the gated four-stage
> `designedSimpleFlow`, with fixed syntactic checks, a semantic judge boundary,
> revision-bound records, and tests. The complete V-model application is later work.

<a id="p03"></a>

## P03 Task and acceptance goal

Implement the design from `mbse/model/detailedDesign.sysml` and
`mbse/model/uml.sysml` as a runnable SimpleFlow core. A caller must be able to
construct a validated plan, record work, evaluate the current stage's exits through
the fixed checker or a permitted judge, inspect the retained assessments, and
advance only when the latest verdict for every required exit gate passed at the
current artifact revision. The four documented fixed checker commands must report
their declared rules with diagnostics and evidence.

Acceptance requires a runnable library and command entry point, one successful
four-stage example with both verify-stage gates, counterexamples for stale evidence,
later failures, missing gates, invalid reports, unauthorized judges, and blocked
correction, and a machine-readable record whose plan and evidence identities can be
re-examined. A declared SysML verification case is a plan until a test has run and
its result is recorded.

<a id="p04"></a>

## P04 Basis and decisions before code

The [joint review](01_review_synthesis.md#p02) determines the risk order. The source
file plan and verification matrix are in [D53](../details/simple_flow/07_file_plan_and_verification.md#d53).
The root `AGENTS.md`, `docs/AGENTS.md`, and `mbse/AGENTS.md` control layout,
language, source/model separation, and cache placement. The Rust API reference in
`docs/uml/simpflow/` is the function contract; its model links and behavior must be
reconciled with the generic record model before implementation. The exact checker
conditions are [D06](../details/simple_flow/02_gates_and_rules.md#d06).

Use these decisions consistently:

1. The run owns an immutable validated flow-plan snapshot. A change to the plan
   starts a different run or an explicitly recorded reopen; it does not silently
   rewrite prior decisions.
2. An untouched visit starts at `artifactRevision = 0`. Every recorded content
   change increments it, including a same-URI edit. Assessment order is append-only;
   the latest assessment for each gate determines current authorization.
3. Failed evaluation blocks the current visit. Recording corrective work and
   reevaluating its gates may resume it. A new visit is for an explicit revisit of a
   completed stage, with earlier history preserved.
4. A syntactic gate runs only its declared checker and exact rule set. A semantic
   gate has no checker and accepts only an agent or human permitted by its flags.
   A missing input, command error, refusal, or malformed report never becomes a pass.
5. The CLI reports what it checked and writes its local run output under `cache/`;
   tests and examples may use fixtures under `tests/`. A user-facing product and
   default V-model automation are outside this slice.
6. Build and review the first checker implementation through the existing DesignFlow
   and project checks. Select `designedSimpleFlow` for a new run only after its four
   commands and rule declarations exist; do not let its requirements or design gates
   self-authorize the work that creates those commands.

<a id="p05"></a>

## P05 Implementation sequence and file ownership

| Step | Work product | Completion check |
| --- | --- | --- |
| 1. Reconcile design | Correct SysML gate selection/control and blocked/rework semantics; connect Rust definitions to API pages; repair documented contracts and semantic links. | Model-only and full parse/check; documentation audit; independent reading of latest-per-gate and revision rules. |
| 2. Freeze the source plane | Add the model file-plan view, source path table, and verification cases. | Figure regenerates from the declared view; every planned source file has a responsibility and case. |
| 3. Implement run core | `Cargo.toml`, `src/lib.rs`, `src/main.rs`, and `src/{status,record,plan,ports,gate,run,factory}.rs`. Make construction, plan validation, activity revision, evaluation, and advancement observable. | Rust unit/integration cases cover all transitions and malformed data without a false pass. |
| 4. Implement fixed checks | `tools/checks/simple_{requirements,design,implementation,record}.py` with shared report schema and command dispatch. | Each checker emits the documented rule IDs, diagnostics, and a failed outcome on bad input; rule-set equality is tested. |
| 5. Connect and demonstrate | Wire a command adapter and a permitted semantic judge to the engine; retain a JSON run report and plan identity. | One four-stage example and failure recovery exercise the real checker interface and retain re-readable evidence. |
| 6. Reconcile delivery docs | Update source/API status, figures, review limits, and this plan's results. | Strict site build, diagram verification, compiler diagnostics, Rust/Python test logs all retained under `cache/`. |

These steps are dependency-ordered. A documented semantic judgment is still a
judgment supplied by a permitted actor; a passing script does not manufacture it.

<a id="p06"></a>

## P06 Verification and record of completion

The [case matrix](../details/simple_flow/07_file_plan_and_verification.md#d54)
defines inputs, expected outcomes, and the model behavior each case exercises.
The [execution record](03_simple_flow_execution_record.md#p08) distinguishes
observed results from these planned expectations.
Run the pinned compiler on `mbse/model/` alone and with `mbse/wishes/`; keep the
parse/check JSON, stderr, and counts of unevaluated constraints. Run
`python3 tools/check_documentation.py --json`, regenerate and verify diagrams,
build the strict documentation site, and run Rust and Python behavior tests with
their build and temporary directories under `cache/`. Preserve exit codes and
logs. Mark a case passed only for a recorded run; do not infer a result from its
SysML declaration or from a diagram.

The review report's historical evidence caveat remains even when this implementation
passes. Completion of this plan establishes the gated SimpleFlow slice, not the
default V-model, durable multi-user persistence, project-level baseline approvals,
or complete engineering verification of a user's developed system.

<a id="p07"></a>

## P07 Risks and recovery

| Risk | Control or recovery |
| --- | --- |
| Toolchain or dependency unavailable on this host | Put any local Rust/Python environment and downloads under `cache/`; keep source and test evidence, report the unrun check precisely. |
| Model compiler accepts an unevaluated invariant | Use executable cases for authorization, plan shape, and lifecycle; report unevaluated counts. |
| Checker and model disagree on a rule | Compare exact rule IDs and inputs before accepting a report; reject unknown or contradictory results. |
| Evidence is absent after a run | Keep its URI and digest in the record, fail the record check, and identify the missing item; never replace the absent evidence with an inferred pass. |

Source changes are reversible through Git commits. Preserve the prior model and
assessment history; a correction creates a new recorded event rather than erasing
the observation that failed.
