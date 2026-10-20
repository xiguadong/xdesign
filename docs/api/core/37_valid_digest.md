# `valid_digest`

> `xdesign::core::plan` · utility function · implemented xdesign slice · Rust-side addition with no matching UML operation; checks digest syntax

<a id="u61"></a>

## U61 valid_digest

```rust
pub fn valid_digest(s: &str) -> bool
```

Return `true` if `s` is exactly 64 ASCII characters, each a digit or lowercase
`a` through `f`. This validates syntax only; it does not compare the digest with
file bytes.

**Logic** — test length and every byte's allowed range. The plan validator and
visit artifact verifier use it before comparing actual hashes.

**See also:** [`RsFlowPlan::validate`](11_flow_plan_validate.md#u26), [`RsStageVisit::verify_artifacts`](24_verify_artifacts.md#u55).
