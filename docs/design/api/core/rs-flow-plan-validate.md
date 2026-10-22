# `RsFlowPlan::validate`

> `xdesign::core::plan` · method · implemented xdesign slice · realizes `RsValidatePlan` in `mbse/model/uml.sysml`; the crate makes plan validation typed and explicit

<a id="u26"></a>

## validate

```rust
pub fn validate(&self) -> Result<(), RsPlanError>
```

Check the plan's internal structural consistency before any run is created from
it. Called by [`create_run`](create-run.md#u16) as its first step; may also be
called standalone when a plan is loaded.

The checks are structural only: duplicate or empty stage ids, missing or unknown
stage references, kind↔checker consistency, nonempty criteria, semantic judge
flags, checker sanity (unique nonempty checker and rule ids, a valid command path,
nonempty `establishes` and `rules`), and the frozen plan file binding (uri,
sha256, exact bytes). Blueprint conformance — equality with a designed flow's
exact stage, gate, checker and rule shape — is the flow module's separate concern;
for the designed simple flow it lives in
[`validate_designed_plan`](../simple-flow/validate-designed-plan.md#u69).

**Parameters** — none beyond `&self`.

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(())` | `()` | All checks below hold |
| `Err(e)` | `RsPlanError` | The first failed check, in the order below |

**Errors**

| Variant | Raised when |
| --- | --- |
| `RsPlanError::MissingInitialStage` | `initial_stage_id` matches no stage |
| `RsPlanError::DuplicateStageId` | two stages share a `stage_id`, or a stage ID is empty |
| `RsPlanError::UnknownStageReference` | any `next_stage_ids` or gate `stage_id` matches no stage |
| `RsPlanError::SyntacticSpecWithoutChecker` | a gate has `kind == Syntactic` and `checker == None` |
| `RsPlanError::SemanticSpecWithChecker` | a gate has `kind == Semantic` and `checker.is_some()` |
| `RsPlanError::MissingCriteria` | any gate has no criterion or a whitespace-only criterion |
| `RsPlanError::NoPermittedJudge` | a semantic gate permits neither agent nor human |
| `RsPlanError::InvalidChecker` | a checker-carrying gate permits a judge; checker IDs repeat or are blank; the command URI is not a valid relative path; `establishes` or `rules` is empty; a rule ID repeats or is blank; a rule condition is blank; or `reads` is empty or contains a blank entry |
| `RsPlanError::InvalidPlanFile` | `plan_uri` is invalid, `plan_sha256` is not lowercase 64-digit hex, or the frozen file's bytes or decoded content disagree with this value |

Every `RsPlanError` variant except `InvalidDesignedFlow` may surface from this
method in principle — `InvalidDesignedFlow` is conformance-only and raised solely
by [`validate_designed_plan`](../simple-flow/validate-designed-plan.md#u69).
(`MissingRunRecordUri` and `MissingWishModelUri` are in fact checked by
`create_run`, because the record and wish paths are not plan fields.)
`InvalidChecker` and `InvalidPlanFile` can also come from
[`validate_designed_plan`](../simple-flow/validate-designed-plan.md#u69).

**Logic**

```
1. if any stage_id occurs twice or is empty → DuplicateStageId
2. if stages is empty or initial_stage_id not in stages → MissingInitialStage
3. for each stage: for each id in next_stage_ids:
       if stage(id) is None → UnknownStageReference
4. for each gate in checkpoints:
5.     if stage(gate.stage_id) is None → UnknownStageReference
6.     match (gate.kind, gate.checker):
           (Syntactic, None)    → SyntacticSpecWithoutChecker
           (Semantic,  Some(_)) → SemanticSpecWithChecker
           _                    → continue
7.     if gate.criteria is empty or contains a whitespace-only entry → MissingCriteria
       if gate.kind == Semantic and neither judge flag is true → NoPermittedJudge
       if checker exists and (either judge flag is true, checker_id repeats
           or is blank, command_uri is not a valid relative path,
           establishes or rules is empty, or any rule ID repeats or is blank,
           rule condition is blank, reads is empty or has a blank entry):
           → InvalidChecker
8. if plan_uri is invalid or plan_sha256 is not lowercase 64-digit hex
       → InvalidPlanFile
   read exact bytes at plan_uri; require sha256(bytes) == plan_sha256;
   decode frozen JSON and require its plan_id, initial_stage_id,
   stages and checkpoints equal this value → InvalidPlanFile
9. return Ok(())
```

The exact four-stage/five-gate blueprint assertions that once lived in step 8
moved to [`validate_designed_plan`](../simple-flow/validate-designed-plan.md#u69):
they compare against the `designedSimpleFlow` shape, and no structural rule may
name one designed flow. The explicit gate and stage checks above enforce the
modeled structural plan even though the SysML compiler does not evaluate its
constraint bodies.

**Other**

| Item | Value |
| --- | --- |
| Side effects | Reads the frozen plan file; does not mutate it |
| Complexity | O(stages + references + gates + plan file bytes) |
| Thread safety | `&self`; re-entrant |

**See also:** [`RsFlowPlan`](rs-flow-plan.md#u25), [`RsPlanError`](rs-plan-error.md#u45), [`create_run`](create-run.md#u16), [`validate_designed_plan`](../simple-flow/validate-designed-plan.md#u69).
