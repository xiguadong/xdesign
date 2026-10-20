# xdesign API reference

> Project: xdesign | Status: the `xdesign::core` + `xdesign::simple_flow` slice is
> implemented; the generic project controller remains at the SysML design level |
> Audience: application integrators, module owners, and reviewers.
> This directory is the class-and-function API reference of the `xdesign` crate,
> organized after the PyTorch API reference: one inventory (this page), one crate
> structure figure, then one page per API under its layer directory.

<a id="u01"></a>

## U01 What this directory owns

The class-and-function level of the `xdesign` crate's software design, in two
layers: [`core/`](core/00_overview.md#u71), the flow-agnostic infrastructure —
status vocabulary, plan, record, ports, gate evaluation, the run engine, and
construction — and [`simple_flow/`](simple_flow/00_overview.md#u72), the designed
four-stage flow: blueprint data, blueprint conformance, the designed-run factory,
and the fixed-checker adapter. The crate's layer map is
[01_struct.md](01_struct.md#u68).

| Document | Question it answers |
| --- | --- |
| this overview | What does the crate offer, what is the project boundary, and what are the conventions? |
| [01_struct.md](01_struct.md#u68) | How is the crate divided into layers, and which direction may dependencies point? |
| [core/00_overview.md](core/00_overview.md#u71) | What does the core layer own, and in which order should its groups be read? |
| [simple_flow/00_overview.md](simple_flow/00_overview.md#u72) | What does the flow layer own, and which core APIs does it build on? |

This directory does not own system structure, module behavior contracts, or test
evidence: those belong to the architecture design and the detailed design, which
embed their own views. A module's UML design lives with that module — today that
is the [SimpleFlow Rust UML design](../details/simple_flow/06_rust_uml_design.md#d44).

<a id="u65"></a>

## U65 API boundary and reading order

`xdesign` is the Rust package for this project. Its library entry point is
`src/lib.rs`, which exposes the `core` and `simple_flow` modules. The
[crate inventory](#u14) below and the one-API-per-page reference under the two
layer directories describe that implemented slice. `src/main.rs` is a local
four-stage demonstration command; it is not the public controller for every
selectable flow.

Read the [crate structure](01_struct.md#u68), the
[xdesign architecture](../arch/02_design.md#b13), the
[SimpleFlow Rust design](../details/simple_flow/06_rust_uml_design.md#d44), and
then the [implemented API inventory](#u14). The
[implementation plan](../prj_manage/02_simple_flow_implementation_plan.md#p03)
sets the first executable slice's acceptance boundary.

<a id="u66"></a>

## U66 Project responsibilities and software status

The SysML `XDesign` composition owns the selected `flow`, controller, wishes, and
traces. These are project-level responsibilities. The Rust crate beneath this
directory implements one selected flow on a generic run engine; it does not
replace that composition.

| Modeled responsibility | Rust location or intended boundary | Status |
| --- | --- | --- |
| `XDesign` composition and flow selection | xdesign application boundary | Modeled; no generic Rust API yet |
| `Controller::receiveWish`, `recordActivity`, `progressWish` | xdesign controller API | Modeled; no generic Rust signatures or implementation yet |
| `Trace` and `FlowExecution` for a wish | xdesign trace and run API | Modeled; `xdesign::core`'s run engine realizes `FlowExecution` with a local, specialized run record |
| `designedSimpleFlow` construction, gate evaluation, and advancement | `xdesign::core` + `xdesign::simple_flow` | Implemented four-stage slice; [API inventory](#u14) |
| Fixed checker commands | `tools/checks/simple_*.py` used by `xdesign::simple_flow` | Implemented for that slice |

The intended integration is for the project controller to select a flow and create
its execution, then delegate a `designedSimpleFlow` execution to
`xdesign::simple_flow`. The controller remains responsible for wish registration,
trace ownership, generic flow selection, and the full V-model. This allocation
follows the [modeled controller actions](../details/01_flows_and_checkpoints.md#f04)
and [record ownership](../details/02_records_and_evidence.md#e01); no Rust
controller API is claimed until its contracts and code exist.

<a id="u67"></a>

## U67 Integration rule

The API pages under this directory document only exported and crate-visible
operations of the `xdesign` crate. A module's API pages live under
`docs/api/<layer>/` — one layer directory per crate layer, with the inventory
above routing to them. A new project-level Rust API must be added to this
inventory when it is designed, with a dedicated contract page and a link to the
SysML element it realizes. The generic controller must consume a selected flow
through the modeled `FlowInterface` and retain the independent wish trace; the
current local CLI does neither for the full workflow. The
[interface contract](../icd/01_interfaces.md#i01) and
[architecture runtime behavior](../arch/02_design.md#b14) remain the source for
those future integration decisions.

<a id="u13"></a>

## U13 What this reference covers

The public and crate-visible APIs of the `xdesign` crate — structs, enums, traits,
functions and methods — grouped by layer and role, core first. The code implements
the gated four-stage slice described here. The modeled UML and the executable tests
have different evidence roles: consult the
[implementation plan](../prj_manage/02_simple_flow_implementation_plan.md#p06)
for executed results and remaining delivery limits. Each layer's overview gives
its module-to-file map: [core](core/00_overview.md#u71),
[simple_flow](simple_flow/00_overview.md#u72).

Reading order for a newcomer: the inventory below top to bottom, then the pages of
`create_run`, `RsFlowRun`, `evaluate_gates`, `advance`, and the flow layer's
`designed_simple_flow_plan` → `validate_designed_plan` → `create_designed_run`
path.

<a id="u14"></a>

## U14 API inventory

### Core — status and error enums

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsFlowStatus` | enum | Run lifecycle vocabulary | [core/01](core/01_flow_status.md#u39) |
| `RsStageStatus` | enum | Visit lifecycle vocabulary | [core/02](core/02_stage_status.md#u40) |
| `RsCheckKind` | enum | How a gate's criteria are established | [core/03](core/03_check_kind.md#u41) |
| `RsCheckpointOutcome` | enum | A verdict's vocabulary | [core/04](core/04_checkpoint_outcome.md#u42) |
| `RsJudgeAuthority` | enum | Who may judge | [core/05](core/05_judge_authority.md#u43) |
| `RsAdvanceOutcome` | enum | What `advance()` decided | [core/06](core/06_advance_outcome.md#u44) |
| `RsConformanceKind` | enum | What class of conformance a checker establishes | [core/07](core/07_conformance_kind.md#u50) |
| `RsPlanError` | enum | How plan validation fails | [core/08](core/08_plan_error.md#u45) |
| `RsGateError` | enum | How gate evaluation fails to conclude | [core/09](core/09_gate_error.md#u46) |

### Core — plan types

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsFlowPlan` | struct | The flow as data: stages and checkpoints | [core/10](core/10_flow_plan.md#u25) |
| `RsFlowPlan::validate` | `fn validate(&self) -> Result<(), RsPlanError>` | Reject a structurally inconsistent plan before any run exists | [core/11](core/11_flow_plan_validate.md#u26) |
| `RsFlowPlan::freeze` | `fn freeze(&mut self, uri: impl Into<String>) -> Result<(), RsPlanError>` | Write a frozen plan and bind its exact byte hash | [core/12](core/12_freeze_plan.md#u53) |
| `RsStageSpec` | struct | One stage: successors, exit gates, terminal flag | [core/13](core/13_stage_spec.md#u33) |
| `RsCheckpointSpec` | struct | One gate: kind, criteria, optional checker, judge flags | [core/14](core/14_checkpoint_spec.md#u34) |
| `RsCheckerSpec` | struct | The command a syntactic gate runs and its rules | [core/15](core/15_checker_spec.md#u35) |
| `RsCheckerRuleSpec` | struct | One fixed rule's identity and condition | [core/16](core/16_checker_rule_spec.md#u36) |

### Core — record types

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsStageVisit` | struct | One visit of one stage; owns activities and assessments | [core/17](core/17_stage_visit.md#u27) |
| `RsStageVisit::record_activity` | `pub(crate) fn record_activity(&mut self, activity: RsActivityEntry) -> bool` | Append work done; bump revision when artifacts changed | [core/18](core/18_record_activity.md#u28) |
| `RsStageVisit::append_assessment` | `pub(crate) fn append_assessment(&mut self, assessment: RsAssessmentEntry)` | Append a gate verdict; earlier assessments stay | [core/19](core/19_append_assessment.md#u29) |
| `RsStageVisit::completion_authorized` | `fn completion_authorized(&self, plan: &RsFlowPlan) -> bool` | Latest verdict of every exit gate passed at the current revision and snapshot digest | [core/20](core/20_completion_authorized.md#u30) |
| `RsActivityEntry` | struct | One recorded piece of work | [core/21](core/21_activity_entry.md#u31) |
| `RsAssessmentEntry` | struct | One recorded gate verdict with evidence | [core/22](core/22_assessment_entry.md#u32) |
| `RsArtifactDigest` | struct | URI and SHA-256 of exact artifact bytes | [core/23](core/23_artifact_digest.md#u51) |
| `RsStageVisit::verify_artifacts` | `fn verify_artifacts(&self) -> bool` | Rehash each current artifact and its canonical set | [core/24](core/24_verify_artifacts.md#u55) |
| `hash_artifact` | `fn hash_artifact(uri: &str) -> Result<RsArtifactDigest, String>` | Read a file and obtain its exact byte digest | [core/25](core/25_hash_artifact.md#u56) |
| `snapshot_digest` | `fn snapshot_digest(artifacts: &[RsArtifactDigest]) -> String` | Hash the canonical artifact digest list | [core/26](core/26_snapshot_digest.md#u57) |
| `RsStageVisit::evidence_uris` | `fn evidence_uris(&self) -> Vec<String>` | Collect current artifacts and same-revision assessment evidence | [core/27](core/27_evidence_uris.md#u63) |
| `RsStageVisit::verify_passed_evidence` | `pub(crate) fn verify_passed_evidence(&self) -> bool` | Confirm every passed assessment retains at least one existing evidence file | [core/28](core/28_verify_passed_evidence.md#u64) |

### Core — trait boundaries and reports

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsCheckCommand` | trait | Boundary every fixed check is invoked through | [core/29](core/29_check_command.md#u21) |
| `RsCheckCommand::run_check` | `fn run_check(&self, checker, plan identity/digest, wish/run URIs, stage, revision, snapshot digest) -> Result<RsGateReport, RsGateError>` | Invoke the selected command; return a byte-bound rule report | [core/30](core/30_run_check.md#u22) |
| `RsJudge` | trait | Boundary a semantic gate is judged through | [core/31](core/31_judge.md#u23) |
| `RsJudge::review` | `fn review(&self, criteria: &[String], evidence_uris: &[String]) -> (RsCheckpointOutcome, String)` | Present criteria and evidence; return verdict and rationale | [core/32](core/32_review.md#u24) |
| `RsGateReport` | struct | The check command's answer: verdict plus per-rule reports | [core/33](core/33_gate_report.md#u37) |
| `RsRuleReport` | struct | One rule's verdict and diagnostic | [core/34](core/34_rule_report.md#u38) |
| `RsSuppliedJudge` | struct, implements `RsJudge` | Carry a caller's explicit semantic verdict; performs no independent review | [core/35](core/35_supplied_judge.md#u59) |

### Core — validation utilities

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `sha256_bytes` | `fn sha256_bytes(bytes: &[u8]) -> String` | Hash exact bytes | [core/36](core/36_sha256_bytes.md#u60) |
| `valid_digest` | `fn valid_digest(s: &str) -> bool` | Check lowercase SHA-256 syntax | [core/37](core/37_valid_digest.md#u61) |
| `valid_relative_path` | `fn valid_relative_path(s: &str) -> bool` | Check repository-relative path shape | [core/38](core/38_valid_relative_path.md#u62) |

### Core — gate evaluation

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `evaluate_checkpoint` | `fn evaluate_checkpoint(spec, visit, plan identity/digest, wish/run URIs, check, judge) -> Result<RsAssessmentEntry, RsGateError>` | One gate: scripted check or permitted judgment, by `kind` | [core/39](core/39_evaluate_checkpoint.md#u20) |

### Core — run engine

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `RsFlowRun` | struct | The generic run engine: plan, visits, current visit, two trait objects | [core/40](core/40_flow_run.md#u17) |
| `RsFlowRun::evaluate_gates` | `fn evaluate_gates(&mut self) -> Vec<RsAssessmentEntry>` | Evaluate every exit gate of the current visit, append assessments | [core/41](core/41_evaluate_gates.md#u18) |
| `RsFlowRun::advance` | `fn advance(&mut self) -> RsAdvanceOutcome` | Enter the next stage or complete only after authorization; otherwise remain blocked | [core/42](core/42_advance.md#u19) |
| `RsFlowRun::record_activity` | `fn record_activity(&mut self, activity: RsActivityEntry) -> Result<(), String>` | Validate and record correction on the same blocked visit | [core/43](core/43_run_record_activity.md#u48) |
| `RsFlowRun::reopen_stage` | `fn reopen_stage(&mut self, stage_id: &str) -> bool` | Reopen a completed flow-completing stage as a new visit | [core/44](core/44_reopen_stage.md#u49) |
| `RsFlowRun::write_record` | `fn write_record(&self) -> Result<(), String>` | Write the re-readable JSON run snapshot | [core/45](core/45_write_record.md#u54) |

### Core — construction

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `create_run` | `fn create_run(plan, wish_model_uri, run_record_uri, check, judgment) -> Result<RsFlowRun, RsPlanError>` | Structurally validate the frozen plan, then build a run and its planned visits | [core/46](core/46_create_run.md#u16) |

### Simple flow — the designed flow

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `designed_simple_flow_plan` | `fn designed_simple_flow_plan() -> RsFlowPlan` | Construct the modeled four-stage plan before freezing its bytes | [simple_flow/01](simple_flow/01_designed_simple_flow_plan.md#u52) |
| `validate_designed_plan` | `fn validate_designed_plan(plan: &RsFlowPlan) -> Result<(), RsPlanError>` | Reject any plan that differs from the `designedSimpleFlow` blueprint | [simple_flow/02](simple_flow/02_validate_designed_plan.md#u69) |
| `create_designed_run` | `fn create_designed_run(plan, wish_model_uri, run_record_uri, check, judgment) -> Result<RsFlowRun, RsPlanError>` | Check blueprint conformance, then construct through `create_run` | [simple_flow/03](simple_flow/03_create_designed_run.md#u70) |
| `RsProcessCheck` | struct, implements `RsCheckCommand` | Invoke and validate a fixed Python checker command | [simple_flow/04](simple_flow/04_process_check.md#u58) |

<a id="u15"></a>

## U15 Conventions

- **Completeness rule.** Every documented type and nontrivial operation, including
  crate-visible operations, appears in [U14](#u14) exactly once and has exactly
  one page. A page documents one API and nothing else.
- **Trivial getters.** `RsFlowRun::status()`, `RsFlowRun::current()` and
  field getters on data structs have no logic; they are declared on the owning type's
  page (marked `trivial`) and have no page of their own. They appear in the type's
  method list rather than as separate inventory rows.
- **Naming.** Types `UpperCamelCase` with the crate's `Rs` prefix, functions and
  variables `snake_case`, enum variants `UpperCamelCase`, error enums end in `Error`.
  A name in a signature is the same name in the model, the figures and this reference.
- **Signature fidelity.** Pages give full Rust signatures including `Result` error
  channels and trait bounds; `GateError` is reserved for "no conclusion possible" —
  a rule that does not hold is data (`RsRuleReport`), never an error.
- **Status.** The crate's implemented slice has executable behavior tests. This
  does not establish the default V-model, durable evidence retention, or acceptance
  of any developed system; those obligations remain in the project plan.

<a id="u47"></a>

## U47 API-reference structure (class-and-function level)

A module's UML design has two granularities, and both are required for a software
module. The module level — component structure, type groups, key interactions,
states, applicability — lives in the module's own document (for example
[D44–D52](../details/simple_flow/06_rust_uml_design.md#d44)). The class-and-function
level lives in this **API reference**, organized after the PyTorch API reference:

- **One inventory page.** Every public or crate-visible nontrivial API of the crate
  — structs, enums, traits, functions, methods, constants — appears exactly once in
  [U14](#u14), grouped by layer and role, with its signature and a one-line summary.
  A reader learns what the crate offers from this page alone. Prose-only mentions of
  an API are defects.
- **One page per API.** Each entry documents exactly one API: full signature
  (generics, lifetimes, trait bounds, `Result` error channels), parameter contract,
  return contract, error contract variant by variant, stepwise logic (≤ 50
  numbered steps; split the function otherwise), side effects, complexity, thread
  safety, and a small example. A struct entry carries its field table with
  visibility and invariants; an enum entry its variant table; a trait entry its
  implementor contract plus one entry per required method.
- **Conventions.** Trivial getters and constructors are declared on the owning type's page and marked
  `trivial` — no page of their own, but still listed. One object has one name across
  signatures, inventory, figures and model. Every entry names the model element it
  realizes, or states why it is a language-side addition. Every entry is marked
  designed-only until code exists in the repository.

The reference sits under `docs/api/<layer>/` with the repository's overview and
numbering rules; this overview is the inventory. The project boundary and
implemented layers are [U65](#u65)–[U66](#u66). The template with the full entry
formats and rules is `cache/kimi_template/docs/detailed-design/DD-API参考级设计模板.md`.

<a id="u02"></a>

## U02 Reading order

1. [01_struct.md](01_struct.md#u68): the crate's layer map — dependencies point one
   way, `simple_flow` → `core`.
2. The module's own UML design document, used with its architecture design and
   interface contract; today that is
   [06_rust_uml_design.md](../details/simple_flow/06_rust_uml_design.md#d44).
3. The layer overviews — [core/00_overview.md](core/00_overview.md#u71), then
   [simple_flow/00_overview.md](simple_flow/00_overview.md#u72) — then the
   [inventory](#u14) and the pages of the APIs being used.

<a id="u03"></a>

## U03 Relation to the decomposition levels

| Level | Owns | Where |
| --- | --- | --- |
| architecture | system boundary, structure, interfaces, concept selection | [architecture design](../arch/02_design.md#b05) |
| details | module behavior, data, interfaces, tests | [SimpleFlow module design](../details/simple_flow/05_module_design.md#d32) |
| API | class-and-function reference of the crate, layer structure | this directory |

The levels are complementary, not competing documents: architecture selects the
structure, detailed design fixes module behavior, and this reference fixes the
crate's API. The module UML design lives in
[06_rust_uml_design.md](../details/simple_flow/06_rust_uml_design.md#d44); the
whole-project package view includes the Rust UML package
([D21](../arch/02_design.md#d21)); model elements link to their own API pages
through `DocSection` annotations.

<a id="u04"></a>

## U04 Current state and limits

- The implemented slice is `xdesign::core` + `xdesign::simple_flow`: the gated
  four-stage SimpleFlow with executable behavior and checker tests. This is not
  the complete V-model workflow — the generic controller, persistence, and
  project-level acceptance remain modeled-only ([U66](#u66)).
- The layered split documented here is realized in the SysML model
  (`XDesignRustUml`), in this reference, and in the source: the crate has been
  `src/core/` + `src/simple_flow/` since step 3 of the
  [restructure plan](../prj_manage/08_api_layering_restructure_plan.md#p22).
- The crate-layer figure in [01_struct.md](01_struct.md#u68) is a model projection
  rendered by the pinned toolchain; its render step follows this documentation
  step. The inventory above is the load-bearing text.

<a id="u05"></a>

## U05 Review

This level is reviewable when: the inventory lists every public or crate-visible
API exactly once and each has exactly one page; every page carries its full
signature, contracts, and stepwise logic; section identifiers stayed stable across
the layer split; and links resolve to existing sections. The documentation review
checklist is [V12](../review/02_checklist.md#v12).
