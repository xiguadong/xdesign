# `validate_designed_plan`

> `xdesign::simple_flow::blueprint` · function · implemented SimpleFlow slice · realizes model `RsValidateDesignedPlan` — blueprint conformance to `DesignedSimpleFlow` ([D01](../../details/simple_flow/01_run_design.md#d01))

<a id="u69"></a>

## U69 validate_designed_plan

```rust
pub fn validate_designed_plan(plan: &RsFlowPlan) -> Result<(), RsPlanError>
```

Blueprint conformance for the designed four-stage flow: reject any plan whose
identity, stages, checkpoints, checker identities, or rule sets differ from the
`designedSimpleFlow` blueprint before a designed run is created from it. Structural
validity is [`RsFlowPlan::validate`](../core/11_flow_plan_validate.md#u26)'s separate
concern; this function compares against
[`designed_simple_flow_plan()`](01_designed_simple_flow_plan.md#u52) so a caller
cannot weaken a rule while retaining its identifier. Called by
[`create_designed_run`](03_create_designed_run.md#u70) before construction.

**Parameters**

| Name | Type | Constraint | Description |
| --- | --- | --- | --- |
| `plan` | `&RsFlowPlan` | structurally valid plan, frozen or not | The plan to check against the blueprint |

**Returns**

| Case | Type | Guarantee |
| --- | --- | --- |
| `Ok(())` | `()` | The plan's stages and checkpoints equal the blueprint |
| `Err(e)` | `RsPlanError` | The first conformance failure, in the order below |

**Errors**

| Variant | Raised when |
| --- | --- |
| `RsPlanError::InvalidPlanFile` | `plan_id != "designedSimpleFlow"` |
| `RsPlanError::InvalidDesignedFlow` | stage sequence, successor chains, exit-gate id lists, the terminal flag, checkpoint ids, kinds, criteria, or judge flags differ from the blueprint |
| `RsPlanError::InvalidChecker` | a checker identity, command URI, conformance class set, or rule-id set — with each rule's condition, reads and order — differs from the blueprint |

Panics: none.

**Logic**

```
1. if plan.plan_id != "designedSimpleFlow" → InvalidPlanFile
2. blueprint := designed_simple_flow_plan()
3. stages: require exactly [requirements, design, implement, verify] in order;
   the exact successor chains (requirements → design → implement → verify,
   verify with none); the exact exit-gate id lists per stage; and
   completes_flow set only on verify; any difference → InvalidDesignedFlow
4. checkpoints: require exactly five, with the blueprint's ids and kinds — four
   syntactic (simpleRequirementsAccepted, simpleDesignAccepted,
   simpleImplementationAccepted, simpleRecordAccepted) and the final semantic
   verificationAccepted with both agent and human judge flags — plus equal
   criteria and judge flags; any difference → InvalidDesignedFlow
5. checkers: require exactly the four blueprint identities
   (simpleRequirementsCheck, simpleDesignCheck, simpleImplementationCheck,
   simpleRecordCheck) with command URIs under tools/checks/simple_*.py and the
   exact rule-id sets per checker, each rule with its declared condition, reads
   and order; any difference → InvalidChecker
6. require plan.stages == blueprint.stages and
   plan.checkpoints == blueprint.checkpoints (full equality)
7. return Ok(())
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | None; a pure data comparison — no file is read |
| Complexity | O(stages + checkpoints + rules) |
| Thread safety | Shared reference only; re-entrant |

**Example** (illustrative)

```rust
let plan = designed_simple_flow_plan();
validate_designed_plan(&plan)?; // the blueprint conforms to itself
```

**See also:** [`designed_simple_flow_plan`](01_designed_simple_flow_plan.md#u52), [`create_designed_run`](03_create_designed_run.md#u70), [`RsFlowPlan::validate`](../core/11_flow_plan_validate.md#u26), [`RsPlanError`](../core/08_plan_error.md#u45).
