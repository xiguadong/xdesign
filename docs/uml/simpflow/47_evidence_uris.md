# `RsStageVisit::evidence_uris`

> record · method · implemented SimpleFlow slice · collects current-visit evidence

<a id="u63"></a>

## U63 evidence_uris

```rust
pub fn evidence_uris(&self) -> Vec<String>
```

Collect artifact URIs in the current visit and evidence URIs from assessments
whose evaluated revision and artifact snapshot digest equal the visit's current
values. The verify-stage semantic judge can therefore receive a record-check
report appended earlier in the same gate evaluation call.

**Logic** — start with current artifact URIs; append evidence from matching
assessments in retained order; sort and deduplicate the resulting list. The method
does not itself verify the files; [`evaluate_checkpoint`](05_evaluate_checkpoint.md#u20)
checks cited files before accepting a semantic pass.

**See also:** [`RsStageVisit`](12_stage_visit.md#u27), [`evaluate_gates`](03_evaluate_gates.md#u18), [`RsJudge::review`](09_review.md#u24).
