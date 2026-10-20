# `RsFlowRun`

> `xdesign::core::run` · struct · implemented xdesign slice · realizes `XDesignArchitecture::FlowExecution` ([E01](../../details/02_records_and_evidence.md#e01))

<a id="u17"></a>

## U17 RsFlowRun

```rust
pub struct RsFlowRun {
    pub(crate) run_id: String,
    pub(crate) plan_id: String,
    pub(crate) plan_uri: String,
    pub(crate) plan_sha256: String,
    pub(crate) wish_model_uri: String,
    pub(crate) run_record_uri: String,
    pub(crate) plan: RsFlowPlan,
    pub(crate) status: RsFlowStatus,
    pub(crate) visits: Vec<RsStageVisit>,
    pub(crate) current_visit: usize,
    pub(crate) check: Box<dyn RsCheckCommand>,
    pub(crate) judgment: Box<dyn RsJudge>,
    pub(crate) persistence_error: bool,
    pub(crate) last_error: Option<String>,
}
```

The generic run engine: one run of a structurally valid flow plan. Owns the plan as
data, the visit history in order, the independent wish trace path, the runtime JSON
record path, and the two trait objects it is automated through. A blocked visit is
corrected in place. Reopening a completed visit of a stage whose `completes_flow`
flag is set appends a new visit while preserving the completed one. A run whose plan
is the frozen designedSimpleFlow blueprint plays the `SimpleFlowRun` role
([D02](../../details/simple_flow/01_run_design.md#d02)); the JSON record it writes
carries `execution_type == "FlowRun"` ([`write_record`](45_write_record.md#u54)).

**Fields** (UML attributes compartment)

| # | Field | Type | Visibility | Constraint / invariant | Description |
| --- | --- | --- | --- | --- | --- |
| 1 | `run_id` | `String` | `pub(crate)` | non-empty, unique per crate instance | Run identifier |
| 2 | `plan_id` | `String` | `pub(crate)` | copied from the validated frozen plan; immutable | Plan identity copied from frozen plan metadata |
| 3 | `plan_uri` | `String` | `pub(crate)` | immutable repository-relative JSON path | Frozen plan file |
| 4 | `plan_sha256` | `String` | `pub(crate)` | immutable SHA-256 of exact plan bytes | Plan integrity anchor |
| 5 | `wish_model_uri` | `String` | `pub(crate)` | non-empty, immutable after construction | Independent SysML trace identity |
| 6 | `run_record_uri` | `String` | `pub(crate)` | non-empty, immutable after construction | Runtime JSON record read by fixed checkers |
| 7 | `plan` | `RsFlowPlan` | `pub(crate)` | immutable after construction | The flow as data |
| 8 | `status` | `RsFlowStatus` | `pub(crate)`; read via `status()` | changed only by factory and run methods | Run lifecycle state |
| 9 | `visits` | `Vec<RsStageVisit>` | `pub(crate)` | `len() >= plan.stages.len()`; history append-only | Every visit in order |
| 10 | `current_visit` | `usize` | `pub(crate)` | always `< visits.len()`; indexes the current active, blocked, or completed visit | Current visit index |
| 11 | `check` | `Box<dyn RsCheckCommand>` | `pub(crate)` | `Send` | Syntactic gate invocation boundary |
| 12 | `judgment` | `Box<dyn RsJudge>` | `pub(crate)` | `Send` | Semantic gate judgment boundary |
| 13 | `persistence_error` | `bool` | `pub(crate)` | blocks advancement while a failed record write remains unresolved | Record write failure latch |
| 14 | `last_error` | `Option<String>` | `pub(crate)`; read via `last_error()` | set when a record write fails | Latest persistence diagnostic |

**Class invariants**

- Exactly one visit is `Active` while the run is active; the current visit is
  `Blocked` while the run is blocked and `Completed` after terminal completion.
- `status == Completed` implies the terminal stage's visit is `Completed` and was
  authorized at its visited revision.
- A reopened run keeps every earlier visit and assessment ([D48](../../details/simple_flow/06_rust_uml_design.md#d48)).

**Methods**

| Method | Kind | Page |
| --- | --- | --- |
| [`evaluate_gates`](41_evaluate_gates.md#u18) | public, mutating | full entry |
| [`advance`](42_advance.md#u19) | public, mutating | full entry |
| [`record_activity`](43_run_record_activity.md#u48) | public, mutating | full entry |
| [`reopen_stage`](44_reopen_stage.md#u49) | public, mutating | full entry |
| [`write_record`](45_write_record.md#u54) | public, writes JSON | full entry |
| `status() -> RsFlowStatus`, `current() -> &RsStageVisit`, `visits() -> &[RsStageVisit]`, `plan() -> &RsFlowPlan`, `run_id() -> &str`, `run_record_uri() -> &str`, `current_visit_index() -> usize`, `last_error() -> Option<&str>` | public getters | trivial ([U15](../00_overview.md#u15)) |

**See also:** [`create_run`](46_create_run.md#u16), [`RsStageVisit`](17_stage_visit.md#u27), [run states D48](../../details/simple_flow/06_rust_uml_design.md#d48).
