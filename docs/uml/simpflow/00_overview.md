# simpflow API reference

> Crate: simpflow (SimpleFlow workflow automation) | Version: v0.1 | Status:
> implemented SimpleFlow slice in `src/`, with executable behavior cases in
> `tests/simple_flow.rs` and fixed-command cases in `tests/test_simple_flow_checkers.py`.
> Organization follows the PyTorch API reference: this page is the complete,
> grouped inventory of every public or crate-visible API; each item links to its own page, which
> documents exactly one API — signature, contracts, logic, example.
> Model source: `mbse/model/uml.sysml` (package `SimpleFlowRustUml`). Module-level
> UML views and view applicability: [D44](../../details/simple_flow/06_rust_uml_design.md#d44).
> Notation and selection rules: [U06](../01_c4_and_uml_guide.md#u06).

<a id="u13"></a>

## U13 What this reference covers

The public and crate-visible APIs of the `simpflow` crate — structs, enums, traits,
functions and methods — are grouped by role. The code implements the gated
four-stage slice described here. The modeled UML and the executable tests have
different evidence roles: consult the [implementation plan](../../prj_manage/02_simple_flow_implementation_plan.md#p06)
for executed results and remaining delivery limits.
Reading order for a newcomer: the inventory below top to bottom, then the pages of
`create_run`, `SimpleFlowRun`, `evaluate_gates`, `advance`.

Implemented module-to-file mapping (one file per module, `src/<module>.rs`; tests under
`tests/`): `status.rs` (enums), `record.rs` (visit/activity/assessment), `plan.rs`
(specifications), `ports.rs` (traits and reports), `gate.rs` (evaluation),
`run.rs` (the run), `factory.rs` (construction). Allowed module dependencies are
shown in [Figure D45-1](../../details/simple_flow/06_rust_uml_design.md#d45).

<a id="u14"></a>

## U14 API inventory

### Construction and factory

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `create_run` | `fn create_run(plan, wish_model_uri, run_record_uri, check, judgment) -> Result<RsSimpleFlowRun, RsPlanError>` | Validate the frozen plan, then build a run and its planned visits | [01](01_create_run.md#u16) |
| `designed_simple_flow_plan` | `fn designed_simple_flow_plan() -> RsFlowPlan` | Construct the modeled four-stage plan before freezing its bytes | [36](36_designed_simple_flow_plan.md#u52) |

### Run behavior

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `RsSimpleFlowRun` | `struct` | One run: plan, visits, current visit, two trait objects | [02](02_simple_flow_run.md#u17) |
| `RsSimpleFlowRun::evaluate_gates` | `fn evaluate_gates(&mut self) -> Vec<RsAssessmentEntry>` | Evaluate every exit gate of the current visit, append assessments | [03](03_evaluate_gates.md#u18) |
| `RsSimpleFlowRun::advance` | `fn advance(&mut self) -> RsAdvanceOutcome` | Enter the next stage or complete only after authorization; otherwise remain blocked | [04](04_advance.md#u19) |
| `RsSimpleFlowRun::record_activity` | `fn record_activity(&mut self, activity: RsActivityEntry) -> Result<(), String>` | Validate and record correction on the same blocked visit | [32](32_run_record_activity.md#u48) |
| `RsSimpleFlowRun::reopen_stage` | `fn reopen_stage(&mut self, stage_id: &str) -> bool` | Reopen a completed terminal stage as a new visit | [33](33_reopen_stage.md#u49) |
| `RsSimpleFlowRun::write_record` | `fn write_record(&self) -> Result<(), String>` | Write the re-readable JSON run snapshot | [38](38_write_record.md#u54) |

### Gate evaluation

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `evaluate_checkpoint` | `fn evaluate_checkpoint(spec, visit, plan identity/digest, wish/run URIs, check, judge) -> Result<RsAssessmentEntry, RsGateError>` | One gate: scripted check or permitted judgment, by `kind` | [05](05_evaluate_checkpoint.md#u20) |

### Trait boundaries

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsCheckCommand` | trait | Boundary every fixed check is invoked through | [06](06_check_command.md#u21) |
| `RsCheckCommand::run_check` | `fn run_check(&self, checker, plan identity/digest, wish/run URIs, stage, revision, snapshot digest) -> Result<RsGateReport, RsGateError>` | Invoke the selected command; return a byte-bound rule report | [07](07_run_check.md#u22) |
| `RsProcessCheck` | `struct`, implements `RsCheckCommand` | Invoke and validate a fixed Python checker command | [42](42_process_check.md#u58) |
| `RsJudge` | trait | Boundary a semantic gate is judged through | [08](08_judge.md#u23) |
| `RsJudge::review` | `fn review(&self, criteria: &[String], evidence_uris: &[String]) -> (RsCheckpointOutcome, String)` | Present criteria and evidence; return verdict and rationale | [09](09_review.md#u24) |
| `RsSuppliedJudge` | `struct`, implements `RsJudge` | Carry a caller's explicit semantic verdict; performs no independent review | [43](43_supplied_judge.md#u59) |

### Plan types

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsFlowPlan` | struct | The designed flow as data: stages and checkpoints | [10](10_flow_plan.md#u25) |
| `RsFlowPlan::freeze` | `fn freeze(&mut self, uri: impl Into<String>) -> Result<(), RsPlanError>` | Write a frozen plan and bind its exact byte hash | [37](37_freeze_plan.md#u53) |
| `RsFlowPlan::validate` | `fn validate(&self) -> Result<(), RsPlanError>` | Reject an inconsistent plan before any run exists | [11](11_flow_plan_validate.md#u26) |
| `RsStageSpec` | struct | One stage: successors, exit gates, terminal flag | [18](18_stage_spec.md#u33) |
| `RsCheckpointSpec` | struct | One gate: kind, criteria, optional checker, judge flags | [19](19_checkpoint_spec.md#u34) |
| `RsCheckerSpec` | struct | The command a syntactic gate runs and its rules | [20](20_checker_spec.md#u35) |
| `RsCheckerRuleSpec` | struct | One fixed rule's identity and condition | [21](21_checker_rule_spec.md#u36) |
| `RsConformanceKind` | enum | What class of conformance a checker establishes | [34](34_conformance_kind.md#u50) |

### Record types

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsStageVisit` | struct | One visit of one stage; owns activities and assessments | [12](12_stage_visit.md#u27) |
| `RsStageVisit::record_activity` | `pub(crate) fn record_activity(&mut self, activity: RsActivityEntry) -> bool` | Append work done; bump revision when artifacts changed | [13](13_record_activity.md#u28) |
| `RsStageVisit::append_assessment` | `pub(crate) fn append_assessment(&mut self, assessment: RsAssessmentEntry)` | Append a gate verdict; earlier assessments stay | [14](14_append_assessment.md#u29) |
| `RsStageVisit::completion_authorized` | `fn completion_authorized(&self, plan: &RsFlowPlan) -> bool` | Latest verdict of every exit gate passed at the current revision and snapshot digest | [15](15_completion_authorized.md#u30) |
| `RsStageVisit::verify_artifacts` | `fn verify_artifacts(&self) -> bool` | Rehash each current artifact and its canonical set | [39](39_verify_artifacts.md#u55) |
| `RsStageVisit::evidence_uris` | `fn evidence_uris(&self) -> Vec<String>` | Collect current artifacts and same-revision assessment evidence | [47](47_evidence_uris.md#u63) |
| `RsActivityEntry` | struct | One recorded piece of work | [16](16_activity_entry.md#u31) |
| `RsAssessmentEntry` | struct | One recorded gate verdict with evidence | [17](17_assessment_entry.md#u32) |
| `RsArtifactDigest` | struct | URI and SHA-256 of exact artifact bytes | [35](35_artifact_digest.md#u51) |
| `hash_artifact` | `fn hash_artifact(uri: &str) -> Result<RsArtifactDigest, String>` | Read a file and obtain its exact byte digest | [40](40_hash_artifact.md#u56) |
| `snapshot_digest` | `fn snapshot_digest(artifacts: &[RsArtifactDigest]) -> String` | Hash the canonical artifact digest list | [41](41_snapshot_digest.md#u57) |

### Public validation utilities

| API | Signature | One-liner | Page |
| --- | --- | --- | --- |
| `sha256_bytes` | `fn sha256_bytes(bytes: &[u8]) -> String` | Hash exact bytes | [44](44_sha256_bytes.md#u60) |
| `valid_digest` | `fn valid_digest(s: &str) -> bool` | Check lowercase SHA-256 syntax | [45](45_valid_digest.md#u61) |
| `valid_relative_path` | `fn valid_relative_path(s: &str) -> bool` | Check repository-relative path shape | [46](46_valid_relative_path.md#u62) |

### Reports

| API | Kind | One-liner | Page |
| --- | --- | --- | --- |
| `RsGateReport` | struct | The check command's answer: verdict plus per-rule reports | [22](22_gate_report.md#u37) |
| `RsRuleReport` | struct | One rule's verdict and diagnostic | [23](23_rule_report.md#u38) |

### Status and error enums

| API | One-liner | Page |
| --- | --- | --- |
| `RsFlowStatus` | Run lifecycle vocabulary | [24](24_flow_status.md#u39) |
| `RsStageStatus` | Visit lifecycle vocabulary | [25](25_stage_status.md#u40) |
| `RsCheckKind` | How a gate's criteria are established | [26](26_check_kind.md#u41) |
| `RsCheckpointOutcome` | A verdict's vocabulary | [27](27_checkpoint_outcome.md#u42) |
| `RsJudgeAuthority` | Who may judge | [28](28_judge_authority.md#u43) |
| `RsAdvanceOutcome` | What `advance()` decided | [29](29_advance_outcome.md#u44) |
| `RsPlanError` | How plan validation fails | [30](30_plan_error.md#u45) |
| `RsGateError` | How gate evaluation fails to conclude | [31](31_gate_error.md#u46) |

<a id="u15"></a>

## U15 Conventions

- **Completeness rule.** Every documented type and nontrivial operation, including
  crate-visible operations, appears in [U14](#u14) exactly once and has exactly
  one page. A page documents one API and nothing else.
- **Trivial getters.** `RsSimpleFlowRun::status()`, `RsSimpleFlowRun::current()` and
  field getters on data structs have no logic; they are declared on the owning type's
  page (marked `trivial`) and have no page of their own. They appear in the type's
  method list rather than as separate inventory rows.
- **Naming.** Types `UpperCamelCase` with the crate-wide `Rs` prefix, functions and
  variables `snake_case`, enum variants `UpperCamelCase`, error enums end in `Error`.
  A name in a signature is the same name in the model, the figures and this reference.
- **Signature fidelity.** Pages give full Rust signatures including `Result` error
  channels and trait bounds; `GateError` is reserved for "no conclusion possible" —
  a rule that does not hold is data (`RsRuleReport`), never an error.
- **Status.** The modeled API has an executable crate and behavior tests. This
  does not establish the default V-model, durable evidence retention, or acceptance
  of any developed system; those obligations remain in the project plan.
