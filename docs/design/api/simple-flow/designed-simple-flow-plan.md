# `designed_simple_flow_plan`

> `xdesign::simple_flow::blueprint` · function · implemented SimpleFlow slice · constructs the modeled `designedSimpleFlow` as plan data (model `RsDesignedSimpleFlowPlan`, [D01](../../flows/simple-flow/definition.md#d01))

<a id="u52"></a>

## designed_simple_flow_plan

```rust
pub fn designed_simple_flow_plan() -> RsFlowPlan
```

Construct the fixed four-stage plan data: requirements, design, implement, and verify,
with their five exit checkpoints and four declared checker rule sets. The returned plan
has no file URI or SHA-256 yet and cannot create a run until it is frozen.

**Parameters** — none. **Returns** — an owned `RsFlowPlan` with `plan_id ==
"designedSimpleFlow"`, the initial requirements stage, and rule IDs matching [D06](../../flows/simple-flow/definition.md#d06).
The returned plan is the blueprint itself: once frozen it must still pass
[`validate_designed_plan`](validate-designed-plan.md#u69), the blueprint
conformance check that [`create_designed_run`](create-designed-run.md#u70)
applies before constructing a run.

**Logic** — construct each checker and rule in declaration order; attach the four
syntactic gates and final semantic gate; set the linear stage successors and verify
terminal flag; leave `plan_uri` and `plan_sha256` empty. No file is read or written.

**Example**

```rust
let mut plan = designed_simple_flow_plan();
plan.freeze("cache/example/plan.json")?;
```

**See also:** [`RsFlowPlan::freeze`](../core/rs-flow-plan-freeze.md#u53), [`RsFlowPlan::validate`](../core/rs-flow-plan-validate.md#u26), [`validate_designed_plan`](validate-designed-plan.md#u69), [`create_designed_run`](create-designed-run.md#u70).
