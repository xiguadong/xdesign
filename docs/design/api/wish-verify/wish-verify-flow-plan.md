# `wish_verify_flow_plan`

> `xdesign::wish_verify::blueprint` · function · implemented wish-verify slice · constructs the modeled `designedWishVerifyFlow` as plan data (model `RsWishVerifyFlowPlan`, [D57](../../flows/wish-verify/definition.md#d57))

<a id="u74"></a>

## wish_verify_flow_plan

```rust
pub fn wish_verify_flow_plan() -> RsFlowPlan
```

Construct the fixed two-stage plan data: `wish` and `verify`, with their two semantic
exit checkpoints — `wishAccepted` at the wish stage and the inherited terminal
`outcomeAccepted` at the verify stage — both permitting an agent or a human judge and
naming no checker. The returned plan has no file URI or SHA-256 yet and cannot create a
run until it is frozen.

**Parameters** — none. **Returns** — an owned `RsFlowPlan` with `plan_id ==
"designedWishVerifyFlow"`, the initial wish stage, checkpoint kinds, criteria, and
judge flags matching [D61](../../flows/wish-verify/definition.md#d61).
The returned plan is the blueprint itself: once frozen it must still pass
[`validate_wish_verify_plan`](validate-wish-verify-plan.md#u75), the blueprint
conformance check that [`create_wish_verify_run`](create-wish-verify-run.md#u76)
applies before constructing a run.

**Logic** — construct the two stages in order with the linear successor chain and the
verify terminal flag; attach the two semantic checkpoints with their verbatim criteria
and both judge flags set; declare no checker; leave `plan_uri` and `plan_sha256` empty.
No file is read or written.

**Example**

```rust
let mut plan = wish_verify_flow_plan();
plan.freeze("cache/example/wish_verify_plan.json")?;
```

**See also:** [`RsFlowPlan::freeze`](../core/rs-flow-plan-freeze.md#u53), [`RsFlowPlan::validate`](../core/rs-flow-plan-validate.md#u26), [`validate_wish_verify_plan`](validate-wish-verify-plan.md#u75), [`create_wish_verify_run`](create-wish-verify-run.md#u76).
