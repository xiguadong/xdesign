# Generic verification authoring guide

Select this template for a general verification plan spanning methods, environment, inputs, procedure, expected results, pass criteria, and trace. Do not select it for a detailed executable test plan (use verification-test-plan) or for judgments that require model/design inspection rather than execution (use design-review).

## Required content

Provide requirement/design basis, methods and environment, case inputs and procedures, expected results, measurable pass criteria and trace, and unexecuted/residual limits. Dependencies are identified requirements and testable designs; execution transcripts and reviewer judgments belong in later evidence, not this plan.

Worked field: `expected_results = "The buffer holds 20.2–21.1 and recovery exports 21.1 with its original UTC timestamp."` This gives exact expected values and ordering.

## Structure

Use the five generic chapters in order with stable anchors. Distinguish method from environment and expected result from pass criterion so a planned case can be executed without inventing missing inputs.

## Depth and omission

Right-size to the general plan. Use verification-test-plan for detailed executable steps and design-review for semantic judgments; record that separation instead of labeling every activity a test.

## Views and alternatives

Trace and case matrices are primary and no figure is required. Retain figures under review with their own blocks when they help evaluate a planned case.

## Sources and review

Source IDs: DPR. Review case executability, exact pass criteria, trace coverage, and the honesty of unexecuted and residual-risk statements; structural completion is not acceptance.
