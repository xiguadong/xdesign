# `RsPlanError`

> `xdesign::core::status` · enum · implemented xdesign slice · Rust-side addition with no SysML counterpart — the model assumes a well-formed flow definition; the crate makes plan validation typed

<a id="u45"></a>

## RsPlanError

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

How [`RsFlowPlan::validate`](rs-flow-plan-validate.md#u26) rejects a structurally
inconsistent plan, and how the flow layer's
[`validate_designed_plan`](../simple-flow/validate-designed-plan.md#u69) rejects a
plan that differs from its designed blueprint, before any run exists. The table
follows enum declaration order and names which function raises each variant:
`validate` (structural), `validate_designed_plan` (conformance), or both.
Validation checks duplicate stage IDs before it checks the initial stage, and
`create_run` checks wish and record paths after plan validation.

| Variant | Meaning | Raised when | Raised by |
| --- | --- | --- | --- |
| `MissingInitialStage` | No entry point | `initial_stage_id` matches no stage | `validate` (structural) |
| `DuplicateStageId` | Ambiguous stages | Two stages share an id, or a stage ID is empty | `validate` (structural) |
| `UnknownStageReference` | Dangling reference | A `next_stage_ids` or gate `stage_id` matches no stage | `validate` (structural) |
| `SyntacticSpecWithoutChecker` | Undecidable gate | Syntactic kind with `checker == None` | `validate` (structural) |
| `SemanticSpecWithChecker` | Contradictory gate | Semantic kind with `checker.is_some()` | `validate` (structural) |
| `MissingWishModelUri` | Unidentified independent trace | `create_run` received an invalid path, a nonexistent file, or a path not ending in `.sysml` | `create_run` / `create_designed_run` (construction, after validation) |
| `MissingRunRecordUri` | Unidentified runtime record | `create_run` received an invalid relative JSON record path | `create_run` / `create_designed_run` (construction, after validation) |
| `MissingCriteria` | Unstated gate basis | Any gate has no criterion or contains a whitespace-only criterion | `validate` (structural) |
| `NoPermittedJudge` | Unjudgeable semantic gate | Neither agent nor human is permitted | `validate` (structural) |
| `InvalidDesignedFlow` | Plan differs from the modeled usage | Stage sequence, gate identities, criteria, judge flags, or other checkpoint fields differ from the built-in `designedSimpleFlow` blueprint | `validate_designed_plan` (conformance only) |
| `InvalidChecker` | Fixed check specification is invalid or differs from the blueprint | Missing or changed command URI, checker identity, conformance class, rule ID, condition, input, or order; file existence is checked at invocation | both — sanity by `validate`, blueprint identity/rule match by `validate_designed_plan` |
| `InvalidPlanFile` | Frozen plan is absent or inconsistent | Wrong plan ID, unreadable JSON, hash mismatch, or decoded content differs from the supplied plan | both — byte binding by `validate`, wrong `plan_id` by `validate_designed_plan` |

**See also:** [`RsFlowPlan::validate`](rs-flow-plan-validate.md#u26), [`validate_designed_plan`](../simple-flow/validate-designed-plan.md#u69), [`create_run`](create-run.md#u16), [`RsGateError`](rs-gate-error.md#u46).
