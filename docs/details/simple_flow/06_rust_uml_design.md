# SimpleFlow Rust UML design

> Module: simpflow crate (SimpleFlow workflow automation) | Version: v0.1 | Status:
> draft, designed-only — no Rust source exists in this repository.
> Audience: module implementers, integrators, verifiers, and reviewers.
> Relation: [module design](05_module_design.md#d32) | requirements
> [R01](../../requirements/01_baseline.md#r01) (`rustUmlDesignMapsEveryDesignElement`,
> `umlViewsAreSelectedWithReasons`, `softwareDesignClaimsNoImplementation`) | interface
> contract [I01](../../icd/01_interfaces.md#i01) | notation guide
> [U06](../../uml/01_c4_and_uml_guide.md#u06). Model source: `mbse/model/uml.sysml`
> (package `SimpleFlowRustUml`).

## Revision history

| Version | Date | Author | Change summary | Reviewed by |
| --- | --- | --- | --- | --- |
| v0.1 | 2026-10-19 | kimi-agent | First draft: crate layout, types, gate interaction, run states | wish-0009 `outcomeAccepted` assessment ([V16](../../review/01_wish_status.md#v16)) |

<a id="d44"></a>

## D44 Purpose, scope and reading order

This document is the UML software design of the designed simple flow
([D01](01_run_design.md#d01)) for a future Rust implementation, the `simpflow` crate.
The SysML detailed design fixes what a run is made of; this design decides what a Rust
program would declare to realize it — modules, structs, enums, traits and functions —
and records which SysML design element each Rust element realizes. The design model is
`SimpleFlowRustUml` in `mbse/model/uml.sysml`; its UML views are SysML projections
rendered by the pinned toolchain ([U10](../../uml/01_c4_and_uml_guide.md#u10)).

Out of scope: Rust code, crate manifests, async runtimes, persistence, and any build or
test execution. Nothing in this document claims an implementation; per
`softwareDesignClaimsNoImplementation` every element here is designed-only
([R01](../../requirements/01_baseline.md#r01)).

Reading order: [crate layout](#d45) -> [types](#d46) -> [interactions](#d47) ->
[states](#d48) -> [interfaces](#d49) -> [errors and performance](#d50) ->
[tests and traceability](#d51) -> [view applicability](#d52). The class-and-function
level of this design — every public API with its full Rust signature, contracts and
stepwise logic — lives in the [simpflow API reference](../../uml/simpflow/00_overview.md#u13),
one page per API in the style of the PyTorch API reference.

<a id="d45"></a>

## D45 Crate and module structure

The `simpflow` crate is one component with seven modules. One module owns one concern,
and dependencies point toward the data definitions: `status` depends on nothing;
`record` and `plan` depend on `status`; `ports` adds the trait boundaries; `gate`
combines records, plan and ports; `run` owns behavior; `factory` builds runs. No module
reaches upward, so the run behavior never depends on its caller.

![Module layout of the simpflow crate and its allowed dependencies](assets/06_crate_modules.svg)

> **Figure D45-1 — Crate module layout.**  
> **Question:** How is the simpflow crate organized, and which module dependencies are allowed?  
> **Model view:** Package projection of `SimpleFlowRustUml::rustModuleLayout`.  
> **Companions:** [class structure](#d46), [package rules U11](../../uml/01_c4_and_uml_guide.md#u11), [module design D32](05_module_design.md#d32).  
> **Status:** renderable-now; designed-only.  
> **Reading:** Each box is one Rust module; an arrow is an allowed `use` dependency. `status` sits at the bottom with no dependencies, `factory` and `run` at the top.  
> **Limits:** Shows design-time module dependencies only; not a deployment view, and no crate exists to compile.

| Module | Responsibility | Depends on |
| --- | --- | --- |
| `status` | Flow, stage, check-kind, outcome and judge-authority enums | — |
| `record` | Visit, activity and assessment entries | `status` |
| `plan` | Stage, checkpoint, checker and rule specifications | `status` |
| `ports` | `CheckCommand` and `Judge` trait boundaries | `status`, `plan` |
| `gate` | Gate evaluation and visit authorization | `record`, `plan`, `ports` |
| `run` | The run type, its state handling and advancement | `record`, `plan`, `gate`, `ports` |
| `factory` | Run construction | `run`, `plan` |

The crate component `SimpflowCrate` exposes the two automation boundaries as ports typed
by the design's existing exchanges ([D49](#d49)), so the crate honors the modeled
interfaces instead of inventing new ones.
The project package view [D21](../../arch/02_design.md#d21) includes
`SimpleFlowRustUml` beside architecture, detailed design, verification, and
traceability. Each Rust model element's `DocSection` points to its specific
[API page](../../uml/simpflow/00_overview.md#u14); the views and package retain
their module-level sections here. This keeps the software design in the same
model-to-document chain as the rest of the system.

<a id="d46"></a>

## D46 Types: structs, enums and traits

The crate's types fall in three groups. **Status enums** mirror the model's lifecycle
vocabulary one-to-one (`RsFlowStatus`, `RsStageStatus`, `RsCheckKind`,
`RsCheckpointOutcome`, `RsJudgeAuthority`, `RsConformanceKind`), plus
`RsAdvanceOutcome`, which names the successor outcomes. **Record and plan structs**
carry data: a visit owns activities, assessments and artifact digests; completed
visits remain in history while blocked visits receive corrective work. The plan makes the
designed flow — stages, checkpoints, checkers, rules — data the run consults rather than
code it hard-codes. **Traits** are the two automation boundaries, `CheckCommand` and
`Judge`; the run holds trait objects and knows no concrete checker or reviewer.

![Structs, enums and traits of the simpflow crate](assets/06_class_structure.svg)

> **Figure D46-1 — Rust type structure.**  
> **Question:** Which structs, enums and traits would the crate declare, and what does each realize?  
> **Model view:** Block-definition projection of `SimpleFlowRustUml::rustClassStructure`.  
> **Companions:** [gate interaction](#d47), [run states](#d48), [block definition D13](05_module_design.md#d13).  
> **Status:** renderable-now; designed-only.  
> **Reading:** Each card is one Rust type with its fields; the `Rs` prefix marks the crate's own vocabulary. Composition (`Vec`-owned entries) runs from visit to activity and assessment entries and from plan to its specifications.  
> **Limits:** Field types are design-level (strings, integers, enums); concrete Rust generics, lifetimes and serialization derives are implementation decisions.

Core data structures:

| Name | Shape / type | Range / units | Invariant |
| --- | --- | --- | --- |
| `RsStageVisit` | struct; `visit_index`, `artifact_revision`, current artifact digest | revision starts at 0 | Latest assessments cite both current revision and exact artifact snapshot digest ([D16](05_module_design.md#d16)) |
| `RsAssessmentEntry` | struct; `outcome`, `judged_by`, evaluated snapshot digest | SHA-256 | `checkpoint_id` resolves against the plan; a syntactic gate's entry is `judged_by = script` |
| `RsArtifactDigest` | struct; artifact URI and SHA-256 | lowercase 64-digit hash | Changes at the same URI invalidate the former byte identity ([U51](../../uml/simpflow/35_artifact_digest.md#u51)) |
| `RsCheckpointSpec` | struct; `kind` enum, `checker` option | — | `checker` is present exactly when `kind = syntactic`; a semantic spec names judge flags and no checker |
| `RsFlowPlan` | struct; 4 stage specs and frozen plan URI/SHA-256 | exact plan file bytes | Stage order and exit gates equal `designedSimpleFlow` ([D01](01_run_design.md#d01)) |
| `RsSimpleFlowRun` | struct; `visits: Vec<RsStageVisit>` | ≥ 4 visits | One pending visit per stage at creation; current visit indexes active, blocked, or completed state |

Every type realizes a named SysML design element, recorded as a `dependency` in
`SimpleFlowRustUml` (for example `rsSimpleFlowRunRealizes` from `RsSimpleFlowRun` to
`SimpleFlowRun`); `rustUmlDesignMapsEveryDesignElement` makes that mapping the coverage
obligation the design review reads ([D52](#d52)). Each type's fields, methods,
invariants and per-function contracts are specified entry by entry in the
[simpflow API reference](../../uml/simpflow/00_overview.md#u14); the two error enums
`RsPlanError` and `RsGateError` are Rust-side additions with their own pages
([U45](../../uml/simpflow/30_plan_error.md#u45), [U46](../../uml/simpflow/31_gate_error.md#u46)).

<a id="d47"></a>

## D47 Interaction design

The key scenario is one gate evaluation. The run asks the current visit's checkpoint
spec to evaluate; the spec reads its own rules and branches on `kind`: a syntactic spec
invokes its checker through the `CheckCommand` trait and holds only when every rule
holds; a semantic spec presents its criteria and the visit's artifact and evidence URIs
through the `Judge` trait and records the returned verdict. Either way the run appends
the assessment to the current visit. The caller later invokes `advance()` for the
transition decision. The branch is exclusive for one spec; the figure draws both
paths because it specifies the operation, not one execution.

![Gate evaluation interaction in the Rust design](assets/06_gate_sequence.svg)

> **Figure D47-1 — Gate evaluation interaction.**  
> **Question:** Which Rust objects exchange which calls when one gate is evaluated?  
> **Model view:** Sequence projection of `SimpleFlowRustUml::RsGateEvaluationInteraction`.  
> **Companions:** [type structure](#d46), [interface implementation](#d49), [activity view D18](05_module_design.md#d18).  
> **Status:** renderable-now; designed-only.  
> **Reading:** Lifelines are the run, the checkpoint spec, the check command and the judge. The middle four messages are the kind branch: check invocation and report for a syntactic spec, judgment request and verdict for a semantic one.  
> **Limits:** One gate only; `evaluateGates` repeats it per exit checkpoint. No timeouts, retries or concurrency are designed — see [D50](#d50).

The four functions of the design map onto the model's operations (full signatures,
contracts and stepwise logic: the linked API pages):

| Rust function | Realizes | Decides | API entry |
| --- | --- | --- | --- |
| `factory::create_run(...) -> Result<RsSimpleFlowRun, RsPlanError>` | `SimpleFlowRunFactory::createRun` ([D03](01_run_design.md#d03)) | Validates frozen plan and both record URIs; initial stage active | [U16](../../uml/simpflow/01_create_run.md#u16) |
| `run.evaluate_gates()` | `EvaluateSimpleFlowGate` ([D04](01_run_design.md#d04)) | One assessment per exit gate of the current visit | [U18](../../uml/simpflow/03_evaluate_gates.md#u18) |
| `evaluate_checkpoint(...) -> Result<RsAssessmentEntry, RsGateError>` | `EvaluateSimpleFlowCheckpoint` ([D11](02_gates_and_rules.md#d11)) | Scripted report or permitted judgment, by `kind` | [U20](../../uml/simpflow/05_evaluate_checkpoint.md#u20) |
| `run.advance() -> RsAdvanceOutcome` | `AdvanceSimpleFlowRun` ([D04](01_run_design.md#d04)) | Enter next stage, complete, or retain a blocked visit | [U19](../../uml/simpflow/04_advance.md#u19) |
| `run.record_activity(activity)` | `Controller::recordActivity` ([F04](../01_flows_and_checkpoints.md#f04)) | Correct the same blocked visit | [U48](../../uml/simpflow/32_run_record_activity.md#u48) |
| `run.reopen_stage(stage_id) -> bool` | `ReopenSimpleFlowStage` ([D04](01_run_design.md#d04)) | Append a new terminal-stage visit after completion | [U49](../../uml/simpflow/33_reopen_stage.md#u49) |

<a id="d48"></a>

## D48 State design

The run's state handling mirrors `SimpleFlowRunLifecycle`
([D20](05_module_design.md#d20)) with `RsFlowStatus` as the state variable the methods
maintain: `create_run` moves pending to active; a gate that is not authorized blocks the
run and its current visit; corrective work resumes that same visit and increments
its revision when artifact bytes change; an authorized nonterminal
visit advances within active; an authorized terminal visit completes it; and a delivered
run can reopen its terminal stage as a new revision-zero visit without erasing history.

![Run state machine of the Rust design](assets/06_run_states.svg)

> **Figure D48-1 — Run state machine.**  
> **Question:** When does the Rust run activate, block, resume, complete or reopen?  
> **Model view:** State-machine projection of `SimpleFlowRustUml::RsRunLifecycle`.  
> **Companions:** [gate interaction](#d47), [lifecycle D20](05_module_design.md#d20).  
> **Status:** renderable-now; designed-only.  
> **Reading:** Four states with transitions keyed on `run.status`, which `create_run`, `evaluate_gates`, `advance`, `record_activity` and `reopen_stage` set. Blocked correction keeps the current visit; terminal reopening appends one.  
> **Limits:** This view's guards are status projections weakened for compiler support; they do not encode the full authorization and current-visit conditions in [D16](05_module_design.md#d16) and the API methods.

<a id="d49"></a>

## D49 Interface implementation

The crate honors the design's two modeled exchanges through its traits; the ports on
`SimpflowCrate` are typed by those exchanges, so no new interface is invented here.
Field-level contracts stay in the interface control document
([I01](../../icd/01_interfaces.md#i01)).

| Interface | Role | Signature or message | Timeout / retry / idempotency | Interface contract |
| --- | --- | --- | --- | --- |
| `CheckCommand` trait ↔ `CheckCommandExchange` | consumer (run invokes a checker) | `run_check(checker, plan identity/digest, wish model URI, run record URI, stage, revision, snapshot digest) -> Result<RsGateReport, RsGateError>` | One synchronous call per gate evaluation; timeout and retry policy open | [U22](../../uml/simpflow/07_run_check.md#u22), [I04](../../icd/01_interfaces.md#i04) |
| `Judge` trait ↔ `JudgmentExchange` | consumer (run presents to a judge) | `review(criteria, evidence_uris) -> (outcome, rationale)` after authority check | A human judge answers out of band; timeout and retry policy open | [U24](../../uml/simpflow/09_review.md#u24), [I05](../../icd/01_interfaces.md#i05) |

Ordering assumption: a gate's assessment is appended only after its command report or
judgment returns; `advance()` reads the latest appended assessment per gate.
The checker verifies the frozen plan file and artifact SHA-256 values, then
returns the byte snapshot digest the assessment records. Delivery and failure of
the command process itself are deployment concerns no view in this document covers
([D52](#d52)).

<a id="d50"></a>

## D50 Errors, logging and performance

- Error taxonomy, designed at the type level: a failed rule is data (`RsRuleReport`
  with `holds = false` and a diagnostic), not an exception; a missing checker for a
  syntactic spec is a plan error the factory must reject; a judge's refusal is an
  `unevaluated` outcome, never a fabricated pass.
- Retry, degradation and recovery: none designed. A blocked run resumes through
  corrective work and a new assessment at the new revision ([D48](#d48)); the design
  adds no automatic retry of commands or judgments.
- Logging: the record is the log. Activities and assessments with timestamps and
  evidence URIs are the audit trail ([E03](../02_records_and_evidence.md#e03)); no
  separate telemetry is designed.
- Performance: no budget is set. Gate evaluation is linear in the stage's exit
  checkpoints and their rules; authorization counting is linear in a visit's
  assessments. No measurement exists, and none is claimed.

<a id="d51"></a>

## D51 Test points and traceability

Test focus for a later implementation, stated now so the design's obligations are
checkable; all cases are planned, none exists.

| Requirement | UML element(s) | Test focus | Case id |
| --- | --- | --- | --- |
| `rustUmlDesignMapsEveryDesignElement` | `RsSimpleFlowRun`, `RsCheckpointSpec`, `RsFlowPlan` | property: every plan checkpoint of the designed flow resolves to a spec | TC-UML-01 (planned) |
| `rustUmlDesignMapsEveryDesignElement` | `create_run` | unit: four pending visits in stage order, initial visit active | TC-UML-02 (planned) |
| `umlViewsAreSelectedWithReasons` | this document's [D52](#d52) table | review: every view has applicability and a reason | TC-UML-03 (planned, review) |
| `softwareDesignClaimsNoImplementation` | whole document | review: no code, build or test claim without a repository artifact | TC-UML-04 (planned, review) |
| Gate authorization ([D16](05_module_design.md#d16)) | `RsEvaluateCheckpoint`, `RsAdvanceOutcome` | unit: the latest verdict of every gate must pass at the current revision and artifact snapshot; missing, failed, stale and duplicate passes do not authorize | TC-UML-05 (planned) |
| Lifecycle ([D48](#d48)) | `RsRunLifecycle`, `RsReopenStage` | unit: failed gates retain the blocked visit, corrective content changes advance its revision, and reopening a completed terminal stage appends a revision-zero visit while retaining history | TC-UML-06 (planned) |
| Frozen plan and evidence ([D47](#d47)) | `RsFlowPlan`, `RsArtifactDigest`, `RsGateReport` | integration: the exact plan file hash and every artifact hash match the run snapshot; changed bytes at an unchanged URI invalidate an earlier pass | TC-UML-07 (planned) |
| Gate authority and report validity ([D49](#d49)) | `RsJudge`, `RsCheckCommand`, `RsEvaluateCheckpoint` | unit: forbidden judge identities, omitted criteria, mismatched rule sets, revision, or snapshot digest cannot produce a pass | TC-UML-08 (planned) |

<a id="d52"></a>

## D52 UML view applicability and design limits

Applicability follows the guide's selection rule ([U09](../../uml/01_c4_and_uml_guide.md#u09)):
each UML 2.5 view is required, recommended, or not applicable with a reason.

| UML view | Applicability | Figure / status |
| --- | --- | --- |
| class | required — the crate's types are the design's core | [Figure D46-1](#d46), renderable-now |
| object | not applicable — no instance snapshot answers a question the class view cannot; run instances are recorded runs, documented in [E01](../02_records_and_evidence.md#e01) | — |
| component | required — the crate boundary and its two trait ports | table in [D45](#d45); figure planned |
| composite structure | recommended — port wiring already shown by [D14](05_module_design.md#d14) | companion, not redrawn |
| deployment | not applicable — no node or protocol decision has been made for the crate | — |
| package | required — module layout and allowed dependencies | [Figure D45-1](#d45), renderable-now |
| profile | not applicable — no domain stereotypes extend the notation | — |
| use case | not applicable — actor goals are the automation's, already in [D17](../../requirements/01_baseline.md#d17) | companion |
| activity | recommended — checkpoint evaluation flow already in [D18](05_module_design.md#d18) | companion, not redrawn |
| state machine | required — run state handling | [Figure D48-1](#d48), renderable-now |
| sequence | required — gate evaluation interaction | [Figure D47-1](#d47), renderable-now |
| communication | not applicable — the sequence view answers the collaboration question for one scenario | — |
| interaction overview | not applicable — one scenario; no fragment composition to show | — |
| timing | not applicable — no time constraint is set on gates or judgments ([D50](#d50)) | — |

Design limits, stated once for the whole document: the crate, its modules, types,
traits and functions are designed-only; the verification case
`XDesignVerification::RustUmlDesignReview` declares the acceptance reading but is not
executed; and this design neither selects async or persistence technology nor produces
a buildable crate. A CDR for this module would still need the evidence package the
checklist lists ([V08](../../review/02_checklist.md#v08)).
