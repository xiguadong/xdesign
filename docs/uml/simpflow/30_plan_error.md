# `RsPlanError`

> status · enum · designed-only · Rust-side addition with no SysML counterpart — the model assumes a well-formed flow definition; the crate makes plan validation typed

<a id="u45"></a>

## U45 RsPlanError

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsPlanError {
    MissingInitialStage,
    DuplicateStageId,
    UnknownStageReference,
    SyntacticSpecWithoutChecker,
    SemanticSpecWithChecker,
    MissingWishModelUri,
    MissingRunRecordUri,
    MissingCriteria,
    NoPermittedJudge,
    InvalidDesignedFlow,
    InvalidChecker,
    InvalidPlanFile,
}
```

How [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26) rejects an inconsistent
plan before any run exists. Each variant names one check, in the order `validate`
performs them.

| Variant | Meaning | Raised when |
| --- | --- | --- |
| `MissingInitialStage` | No entry point | `initial_stage_id` matches no stage |
| `DuplicateStageId` | Ambiguous stages | Two stages share an id |
| `UnknownStageReference` | Dangling reference | A `next_stage_ids` or gate `stage_id` matches no stage |
| `SyntacticSpecWithoutChecker` | Undecidable gate | Syntactic kind with `checker == None` |
| `SemanticSpecWithChecker` | Contradictory gate | Semantic kind with `checker.is_some()` |
| `MissingWishModelUri` | Unidentified independent trace | `create_run` received an empty wish model path |
| `MissingRunRecordUri` | Unidentified runtime record | `create_run` received an empty JSON record path |
| `MissingCriteria` | Unstated gate basis | Any gate has no criterion |
| `NoPermittedJudge` | Unjudgeable semantic gate | Neither agent nor human is permitted |
| `InvalidDesignedFlow` | Plan differs from the modeled usage | Four-stage sequence, terminal flag, gate references or identities differ from `designedSimpleFlow` |
| `InvalidChecker` | Fixed check is not executable as specified | Missing command URI, missing rule, duplicate rule ID, or missing rule input |
| `InvalidPlanFile` | Frozen plan is absent or inconsistent | Wrong plan ID, unreadable JSON, hash mismatch, or decoded content differs from the supplied plan |

**See also:** [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26), [`create_run`](01_create_run.md#u16), [`RsGateError`](31_gate_error.md#u46).
