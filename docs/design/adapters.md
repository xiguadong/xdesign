# Ports and adapters



The common checker-command and judge boundaries, their reports, and the authority rules for syntactic and semantic gates. Flow-specific commands and rules belong in each flow's definition page.



<a id="f02"></a>

## 1. Checkpoint kinds and judges

`CheckKind` separates criteria a fixed check can establish from criteria that need
judgment; `JudgeAuthority` names who judges an assessment. `ConformanceKind` describes
what a checker establishes: `artifactPresence` checks that a required artifact exists,
`identifierConformance` checks declared identities, `definitionCoverage` checks that
required definitions are present, and `traceConformance` checks required links.
These are coverage categories, not verdicts about engineering adequacy.

| Kind | Names checkers | Judge flags | Judged by | Example here |
| --- | --- | --- | --- | --- |
| `CheckKind::syntactic` | yes, at least one | none set | a script (`JudgeAuthority::script`) | `requirementsAccepted` — "requirements and their verification criteria trace to the wish", fixed by `requirementTraceCheck` |
| `CheckKind::semantic` | no | `agentMayJudge` and/or `humanMayJudge` | an agent or a human | `verificationAccepted` — "verification evidence demonstrates that the implementation satisfies the requirements" |

This is the generic modeled contract. The implemented custom CLI binding permits agent
authority on a gate still labeled syntactic and therefore deviates from the third
assertion below. The [as-built authority review](checkpoint-library.md#d107) records the
deviation and policy alternatives; the compiler does not evaluate assertion bodies.

Three assertions specify the split:

- `Checkpoint::syntacticCriteriaAreFixedByCheckers` — a syntactic checkpoint has at least
  one checker and sets no judge flag.
- `Checkpoint::semanticCriteriaNeedJudgment` — a semantic checkpoint has no checker and
  names at least one judge.
- `CheckpointAssessment::judgmentMatchesCheckpointKind` — an assessment judged by a script
  may only concern a syntactic checkpoint, and vice versa.

Judgment permission is one boolean per authority rather than a list, because the
compiler's export keeps only the first element of a multi-valued value — found in
wish-0002 and recorded in its trace ([T04](../architecture/constraints.md#t04)).
