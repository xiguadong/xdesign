# Verification test plan authoring guide

Select this template to define planned test, demonstration, or analysis cases with inputs, environment, procedure, expected results, and measurable pass criteria. Do not select it to record an executed result or to rely only on inspection; use retained evidence for results and design-review for semantic judgments.

## Required content

Provide requirement/design basis, methods and environment, exact case inputs and procedures, expected results, measurable pass criteria and trace, and unexecuted/residual limits. Dependencies are testable requirements, identified units and interfaces, and any fixtures or drivers; execution logs belong in the evidence record.

Worked field: `expected_results = "The buffer holds 20.2–21.1 and recovery exports 21.1 with its exact original UTC timestamp."` This is a precise expectation, not an observed result.

## Structure

Use scope, methods/environment, cases, acceptance/trace, and limits in order. Give enough input and procedure detail for another runner to reproduce each planned case without inventing setup.

## Depth and omission

Right-size to executable cases and measurable outcomes; link detailed model/design inspection to design-review and actual transcripts to evidence. Omit executed-result claims and label every case planned until run.

## Views and alternatives

No figure is required by default; trace and case matrices are primary. Include a figure only if it clarifies setup or sequence and retain its six-field block and fidelity review.

## Sources and review

Source IDs: DPR, REQ42. Review case reproducibility, exact pass criteria, requirement trace, and whether unexecuted status and residual risks are honest. A declared case is never an executed result.
