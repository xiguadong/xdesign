# Review overview

> Project: xdesign | Status: working review record and checklist | Review position:
> SRR, PDR, CDR, and FRR have not been held.

<a id="v15"></a>

## V15 Scope, reading order, and decision boundaries

This directory helps a reader distinguish an individual wish's assessed outcome from a
project stage review. Begin with [wish status](01_wish_status.md#v01) for the assessment
method, current wish table, retained evidence, and unresolved workflow limitations.
Then use the [engineering review checklist](02_checklist.md#v08) to prepare SRR, PDR,
CDR, or FRR and to see which stage-specific evidence remains open.

Each wish's independent SysML trace under `mbse/wishes/` is the record of its visits,
artifact revisions, judgments, and evidence. The status document summarizes those
records; it does not create new gate outcomes. The checklist follows the stage topics
in `cache/templates/docs/review/00_checklist.md` and links requirements, architecture,
module design, interfaces, and verification evidence to the decisions they support.

Some design wishes have passed their own recorded assessments. No project-level SRR,
PDR, CDR, or FRR decision has been recorded, and no functional, allocated, detailed
design, or release baseline is frozen. A review becomes a passed gate only when its
judge, date, assessed revision, evidence, outcome, and conditions are recorded.
