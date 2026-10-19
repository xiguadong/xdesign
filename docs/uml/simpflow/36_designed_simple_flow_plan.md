# `designed_simple_flow_plan`

> factory · function · implemented SimpleFlow slice · maps `designedSimpleFlow` ([D01](../../details/simple_flow/01_run_design.md#d01))

<a id="u52"></a>

## U52 designed_simple_flow_plan

```rust
pub fn designed_simple_flow_plan() -> RsFlowPlan
```

Construct the fixed four-stage plan data: requirements, design, implement, and verify,
with their five exit checkpoints and four declared checker rule sets. The returned plan
has no file URI or SHA-256 yet and cannot create a run until it is frozen.

**Parameters** — none. **Returns** — an owned `RsFlowPlan` with `plan_id ==
"designedSimpleFlow"`, the initial requirements stage, and rule IDs matching [D06](../../details/simple_flow/02_gates_and_rules.md#d06).

**Logic** — construct each checker and rule in declaration order; attach the four
syntactic gates and final semantic gate; set the linear stage successors and verify
terminal flag; leave `plan_uri` and `plan_sha256` empty. No file is read or written.

**Example**

```rust
let mut plan = designed_simple_flow_plan();
plan.freeze("cache/example/plan.json")?;
```

**See also:** [`RsFlowPlan::freeze`](37_freeze_plan.md#u53), [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26).
