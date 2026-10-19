# `RsFlowPlan::validate`

> plan · method · implemented SimpleFlow slice · Rust-side addition with no SysML counterpart — the model assumes a well-formed flow definition; the crate makes plan validation typed and explicit

<a id="u26"></a>

## U26 validate

```rust
pub fn validate(&self) -> Result<(), RsPlanError>
```

Check the plan's internal consistency and agreement with the built-in
`designed_simple_flow_plan()` blueprint before any run is created from it. Called by
[`create_run`](01_create_run.md#u16) as its first step; may also be called standalone
when a plan is loaded.

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
| `RsPlanError::DuplicateStageId` | two stages share a `stage_id` |
| `RsPlanError::UnknownStageReference` | any `next_stage_ids` or gate `stage_id` matches no stage |
| `RsPlanError::SyntacticSpecWithoutChecker` | a gate has `kind == Syntactic` and `checker == None` |
| `RsPlanError::SemanticSpecWithChecker` | a gate has `kind == Semantic` and `checker.is_some()` |
| `RsPlanError::MissingCriteria` | any gate has no criterion |
| `RsPlanError::NoPermittedJudge` | a semantic gate permits neither agent nor human |
| `RsPlanError::InvalidChecker` | a syntactic checker lacks a command or rules, or rule IDs repeat |
| `RsPlanError::InvalidDesignedFlow` | the four-stage order, successor links, terminal flag, exit-gate references, or unique checkpoint identities disagree with `designedSimpleFlow` |
| `RsPlanError::InvalidPlanFile` | plan ID, frozen JSON file, SHA-256, or decoded plan disagree with this value |

**Logic**

```
1. if stages is empty or initial_stage_id not in stages → MissingInitialStage
2. if any stage_id occurs twice → DuplicateStageId
3. for each stage: for each id in next_stage_ids:
       if stage(id) is None → UnknownStageReference
4. for each gate in checkpoints:
5.     if stage(gate.stage_id) is None → UnknownStageReference
6.     match (gate.kind, gate.checker):
           (Syntactic, None)    → SyntacticSpecWithoutChecker
           (Semantic,  Some(_)) → SemanticSpecWithChecker
           _                    → continue
7.     if gate.criteria.is_empty() → MissingCriteria
       if gate.kind == Semantic and neither judge flag is true → NoPermittedJudge
       if gate.kind == Syntactic and (checker.command_uri is empty or
           checker.rules is empty or rule IDs repeat or a rule has no reads):
           → InvalidChecker
8. compare the complete stage and checkpoint specifications to the built-in
   designed_simple_flow_plan() blueprint: stage order, edges, terminal flag,
   exit gates, gate IDs/kinds, criteria, judge flags, checker identities,
   commands, conformance classes, and each rule's ID, condition, reads, and
   order. Any difference → InvalidDesignedFlow or InvalidChecker
9. if plan_id != "designedSimpleFlow" or plan_uri is invalid or
   plan_sha256 is not lowercase 64-digit hex → InvalidPlanFile
   read exact bytes at plan_uri; require sha256(bytes) == plan_sha256;
   decode frozen JSON and require its plan_id, initial_stage_id,
   stages and checkpoints equal this value → InvalidPlanFile
10. return Ok(())
```

The exact blueprint comparison prevents a caller from weakening a rule while
retaining its identifier. The explicit gate and stage checks enforce the modeled plan even though the
SysML compiler does not evaluate its constraint bodies. `MissingRunRecordUri` is
checked by `create_run`, because the wish path is not a plan field.

**Other**

| Item | Value |
| --- | --- |
| Side effects | Reads the frozen plan file; does not mutate it |
| Complexity | O(stages + references + gates + plan file bytes) |
| Thread safety | `&self`; re-entrant |

**See also:** [`RsFlowPlan`](10_flow_plan.md#u25), [`RsPlanError`](30_plan_error.md#u45), [`create_run`](01_create_run.md#u16).
