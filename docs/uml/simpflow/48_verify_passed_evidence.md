# `RsStageVisit::verify_passed_evidence`

> record · crate-visible method · implemented SimpleFlow slice · Rust-side addition with no SysML operation counterpart

<a id="u64"></a>

## U64 verify_passed_evidence

```rust
pub(crate) fn verify_passed_evidence(&self) -> bool
```

Check retention of evidence cited by every `Passed` assessment on this visit.
[`RsSimpleFlowRun::advance`](04_advance.md#u19) calls this for the current and
completed visits before authorizing a transition, including a repeated call after
completion.

**Parameters** — none beyond `&self`.

**Returns** — `true` only when each passed assessment has a nonempty
`evidence_uris` list and every URI is a valid repository-relative path to an
existing file. Assessments with other outcomes do not impose this check. An empty
set of passed assessments satisfies this method; authorization separately requires
current passed verdicts through
[`completion_authorized`](15_completion_authorized.md#u30).

**Errors** — none; invalid or missing evidence yields `false`.

**Logic**

```
1. for assessment in self.assessments where outcome == Passed:
2.     if assessment.evidence_uris is empty: return false
3.     for uri in assessment.evidence_uris:
           if !valid_relative_path(uri) or !Path::new(uri).is_file():
               return false
4. return true
```

**Other** — reads file metadata for every cited evidence URI; does not mutate the
visit. Time is linear in the number of passed assessments and their evidence URIs.

**Limit** — this method checks presence and path form. Assessment evidence is
retained by URI without a content digest, so it does not prove that an existing
evidence file still has the bytes it had when the assessment was made.

**Example** (illustrative, inside the crate)

```rust
let uri = "cache/examples/passed-evidence.json";
std::fs::create_dir_all("cache/examples").unwrap();
std::fs::write(uri, b"retained report").unwrap();
let mut visit = RsStageVisit::pending("verify".into(), 0);
let snapshot = visit.artifact_snapshot_digest().to_owned();
visit.append_assessment(RsAssessmentEntry {
    checkpoint_id: "simpleRecordAccepted".into(),
    evaluated_revision: 0,
    evaluated_snapshot_digest: snapshot,
    outcome: RsCheckpointOutcome::Passed,
    judged_by: RsJudgeAuthority::Script,
    assessor: "simpleRecordCheck".into(),
    rationale: "passed".into(),
    evidence_uris: vec![uri.into()],
});
assert!(visit.verify_passed_evidence());
std::fs::remove_file(uri).unwrap();
assert!(!visit.verify_passed_evidence());
```

**See also:** [`RsStageVisit`](12_stage_visit.md#u27),
[`verify_artifacts`](39_verify_artifacts.md#u55),
[`RsSimpleFlowRun::advance`](04_advance.md#u19).
