# `validate_wish_verify_plan`

> `xdesign::wish_verify::blueprint` · function · implemented wish-verify slice · realizes model `RsValidateWishVerifyPlan` — blueprint conformance to `DesignedWishVerifyFlow` ([D57](../../flows/wish-verify/definition.md#d57), [D61](../../flows/wish-verify/definition.md#d61))

<a id="u75"></a>

## validate_wish_verify_plan

```rust
pub fn validate_wish_verify_plan(plan: &RsFlowPlan) -> Result<(), RsPlanError>
```

Blueprint conformance for the designed two-stage judged flow: reject any plan whose
identity, stages, checkpoints, kinds, criteria, or judge flags differ from the
`designedWishVerifyFlow` blueprint before a designed run is created from it. Structural
validity is [`RsFlowPlan::validate`](../core/rs-flow-plan-validate.md#u26)'s separate
concern; this function compares against
[`wish_verify_flow_plan()`](wish-verify-flow-plan.md#u74) so a caller cannot weaken
a criterion while retaining its identifier — or smuggle in the syntactic gate the
design deliberately does not declare ([D62](../../flows/wish-verify/definition.md#d62)).
Called by [`create_wish_verify_run`](create-wish-verify-run.md#u76) before
construction.

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
| `RsPlanError::InvalidPlanFile` | `plan_id != "designedWishVerifyFlow"` |
| `RsPlanError::InvalidDesignedFlow` | stage sequence, successor chain, exit-gate id lists, the terminal flag, checkpoint ids, kinds, criteria, or judge flags differ from the blueprint — including a checkpoint declared syntactic or naming a checker |
| `RsPlanError::InvalidChecker` | the plan declares any checker, where the blueprint declares none |

Panics: none.

**Logic**

```
1. if plan.plan_id != "designedWishVerifyFlow" → InvalidPlanFile
2. blueprint := wish_verify_flow_plan()
3. stages: require exactly [wish, verify] in order; the exact successor chain
   (wish → verify, verify with none); the exact exit-gate id lists per stage
   ([wishAccepted], [outcomeAccepted]); and completes_flow set only on verify;
   any difference → InvalidDesignedFlow
4. checkpoints: require exactly two, with the blueprint's ids and the semantic
   kind — wishAccepted and outcomeAccepted — with equal criteria and both agent
   and human judge flags set; a syntactic kind or a named checker →
   InvalidDesignedFlow
5. checkers: require none, matching the blueprint's empty set; any declared
   checker → InvalidChecker
6. require plan.stages == blueprint.stages and
   plan.checkpoints == blueprint.checkpoints (full equality)
7. return Ok(())
```

**Other**

| Item | Value |
| --- | --- |
| Side effects | None; a pure data comparison — no file is read |
| Complexity | O(stages + checkpoints) |
| Thread safety | Shared reference only; re-entrant |

**Example** (illustrative)

```rust
let plan = wish_verify_flow_plan();
validate_wish_verify_plan(&plan)?; // the blueprint conforms to itself
```

**See also:** [`wish_verify_flow_plan`](wish-verify-flow-plan.md#u74), [`create_wish_verify_run`](create-wish-verify-run.md#u76), [`RsFlowPlan::validate`](../core/rs-flow-plan-validate.md#u26), [`RsPlanError`](../core/rs-plan-error.md#u45).
