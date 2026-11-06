# `RsFlowRun`

> `xdesign::core::run` · struct · implemented xdesign slice · realizes `XDesignArchitecture::FlowExecution` ([E01](../../../interfaces/record-format.md#e01))

<a id="u17"></a>

## RsFlowRun

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
    pub(crate) artifact_snapshot_directory: Option<String>,
}
```

RsFlowRun is the generic run engine for one structurally valid plan. Use it to drive a run through visits, gate evaluation, advancement, reopening, and blocked correction; flow factories construct it and [restore_run](restore-run.md#u78) reloads a validated saved record and the sibling API pages describe its methods.

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
| 15 | `artifact_snapshot_directory` | `Option<String>` | `pub(crate)` | configured before work; normalized path | Optional completed-artifact retention namespace; absent on legacy runs |

**Class invariants**

- Exactly one visit is `Active` while the run is active; the current visit is
  `Blocked` while the run is blocked and `Completed` after terminal completion.
- `status == Completed` implies the terminal stage's visit is `Completed` and was
  authorized at its visited revision.
- A reopened run keeps every earlier visit and assessment ([D48](../../flows/simple-flow/implementation.md#d48)).

**Methods**

| Method | Kind | Page |
| --- | --- | --- |
| [`evaluate_gates`](rs-flow-run-evaluate-gates.md#u18) | public, mutating | full entry |
| [`advance`](rs-flow-run-advance.md#u19) | public, mutating | full entry |
| [`record_activity`](rs-flow-run-record-activity.md#u48) | public, mutating | full entry |
| [`reopen_stage`](rs-flow-run-reopen-stage.md#u49) | public, mutating | full entry |
| [`write_record`](rs-flow-run-write-record.md#u54) | public, writes JSON | full entry |
| [`retain_completed_artifacts`](rs-flow-run-retain-completed-artifacts.md#u114) | public, configures retention before work | full entry |
| `status() -> RsFlowStatus`, `current() -> &RsStageVisit`, `visits() -> &[RsStageVisit]`, `plan() -> &RsFlowPlan`, `run_id() -> &str`, `run_record_uri() -> &str`, `current_visit_index() -> usize`, `last_error() -> Option<&str>` | public getters | trivial ([U15](../index.md#u15)) |

`artifact_snapshot_directory() -> Option<&str>` is a trivial getter. Retention
configuration is Rust-side storage-adapter bookkeeping; it adds no SysML gate or
judgment authority.

**See also:** [`create_run`](create-run.md#u16), [`RsStageVisit`](rs-stage-visit.md#u27), [run states D48](../../flows/simple-flow/implementation.md#d48).
