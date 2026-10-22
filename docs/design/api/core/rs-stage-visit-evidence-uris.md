# `RsStageVisit::evidence_uris`

> `xdesign::core::record` · method · implemented xdesign slice · Rust-side addition with no matching UML operation; collects current-visit evidence

<a id="u63"></a>

## evidence_uris

```rust
pub fn evidence_uris(&self) -> Vec<String>
```

Collect artifact URIs in the current visit and evidence URIs from assessments
whose evaluated revision and artifact snapshot digest equal the visit's current
values. The verify-stage semantic judge can therefore receive a record-check
report appended earlier in the same gate evaluation call.

**Logic** — start with current artifact URIs; append evidence from matching
assessments in retained order; sort and deduplicate the resulting list. The method
does not itself verify the files; [`evaluate_checkpoint`](evaluate-checkpoint.md#u20)
checks cited files before accepting a semantic pass.

**See also:** [`RsStageVisit`](rs-stage-visit.md#u27), [`evaluate_gates`](rs-flow-run-evaluate-gates.md#u18), [`RsJudge::review`](rs-judge-review.md#u24).
