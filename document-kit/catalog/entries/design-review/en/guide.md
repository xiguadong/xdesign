# Design review authoring guide

Select this template to plan inspection or review of model meaning, design adequacy, source fidelity, and other judgments not established by execution alone. Do not select it to claim a review has passed before the named independent reviewer has judged the identified artifacts.

## Required content

Provide requirement/design basis, review questions and methods, exact inputs and procedure, expected judgments and measurable acceptance, and unexecuted/residual limits. Dependencies are identified model/document hashes, source records, figures, and coverage list; the independent review verdict belongs in later evidence.

Worked field: `review_questions = "Do figure readings match model views and visible geometry, are invariants testable, and do examples distinguish fiction from evidence?"` These require judgment rather than file checks.

## Structure

Use scope, questions/methods, inputs/procedure, acceptance/trace, and limits in order. Identify reviewer authority and assessed hashes, and separate structural findings from unresolved semantic concerns.

## Depth and omission

Right-size to judgments execution cannot make; link executable cases to verification-test-plan and actual findings to the independent report. Omit any premature pass and record the review as planned until performed.

## Views and alternatives

No new figure is required by default; figures under review retain their own blocks. Compare each reading with model views and visible geometry, and record table/prose alternatives where no new figure is useful.

## Sources and review

Source IDs: ARC42, GITHUB-DOCS. Review semantic adequacy, source and figure fidelity, editorial quality, and whether the independent judgment is separate from mechanical reports and runner evidence review.
