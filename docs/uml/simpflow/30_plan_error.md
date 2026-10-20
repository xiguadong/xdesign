# `RsPlanError`

> status · enum · implemented SimpleFlow slice · Rust-side addition with no SysML counterpart — the model assumes a well-formed flow definition; the crate makes plan validation typed

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
plan before any run exists. The table follows enum declaration order; validation
checks duplicate stage IDs before it checks the initial stage, and `create_run`
checks wish and record paths after plan validation.

| Variant | Meaning | Raised when |
| --- | --- | --- |
| `MissingInitialStage` | No entry point | `initial_stage_id` matches no stage |
| `DuplicateStageId` | Ambiguous stages | Two stages share an id, or a stage ID is empty |
| `UnknownStageReference` | Dangling reference | A `next_stage_ids` or gate `stage_id` matches no stage |
| `SyntacticSpecWithoutChecker` | Undecidable gate | Syntactic kind with `checker == None` |
| `SemanticSpecWithChecker` | Contradictory gate | Semantic kind with `checker.is_some()` |
| `MissingWishModelUri` | Unidentified independent trace | `create_run` received an invalid path, a nonexistent file, or a path not ending in `.sysml` |
| `MissingRunRecordUri` | Unidentified runtime record | `create_run` received an invalid relative JSON record path |
| `MissingCriteria` | Unstated gate basis | Any gate has no criterion or contains a whitespace-only criterion |
| `NoPermittedJudge` | Unjudgeable semantic gate | Neither agent nor human is permitted |
| `InvalidDesignedFlow` | Plan differs from the modeled usage | Stage sequence, gate identities, criteria, judge flags, or other checkpoint fields differ from the built-in `designedSimpleFlow` blueprint |
| `InvalidChecker` | Fixed check specification is invalid or differs from the blueprint | Missing or changed command URI, checker identity, conformance class, rule ID, condition, input, or order; file existence is checked at invocation |
| `InvalidPlanFile` | Frozen plan is absent or inconsistent | Wrong plan ID, unreadable JSON, hash mismatch, or decoded content differs from the supplied plan |

**See also:** [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26), [`create_run`](01_create_run.md#u16), [`RsGateError`](31_gate_error.md#u46).
