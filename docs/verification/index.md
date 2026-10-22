# Verification design

> Project: xdesign | Status: verification design | Review position: SRR, PDR, CDR, and
> FRR have not been held. This directory contains verification design, not executed
> results or evidence summaries.

<a id="v15"></a>

## Scope, reading order, and decision boundaries

This directory helps a reader distinguish an individual wish's assessed outcome from a
project stage review. Begin with [wish status](../records/wishes/wish-history.md#v01) for the assessment
method, current wish table, retained evidence, and unresolved workflow limitations.
Then use the [engineering review checklist](review-criteria.md#v08) to prepare SRR, PDR,
CDR, or FRR and to see which stage-specific evidence remains open.

Each wish's independent SysML trace under `mbse/wishes/` is the record of its visits,
artifact revisions, judgments, and evidence. The status document summarizes those
records; it does not create new gate outcomes. The checklist follows the stage topics expected for SRR, PDR, CDR, and FRR and links
requirements, architecture, module design, interfaces, and the evidence required for each
decision. Executed results and evidence summaries remain in `records/`.

Some design wishes have passed their own recorded assessments. No project-level SRR,
PDR, CDR, or FRR decision has been recorded, and no functional, allocated, detailed
design, or release baseline is frozen. A review becomes a passed gate only when its
judge, date, assessed revision, evidence, outcome, and conditions are recorded.
