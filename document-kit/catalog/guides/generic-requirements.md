# Generic requirements authoring guide

Select this template for a self-contained requirement set whose scope, rationale, measurable acceptance, and trace fit the generic chapters without a current/proposed delta or a new-capability narrative. Do not select it for a behavior delta (use change-requirements) or a greenfield capability (use system-requirements).

## Required content

Provide the requirement owners and model sources, system boundary, normative statements with rationale, measurable acceptance and method, model/design trace, and unresolved limits. Upstream dependencies are the wish or product goals and any model elements that own the behavior; downstream verification links must identify planned cases, not results.

Worked field: `measurable_conditions = "Ten valid samples arrive at 60±0.1-second intervals and NaN never reaches the buffer."` This demonstrates a measurable input and outcome rather than prose such as "works correctly."

## Structure

Use the five generic chapters in order and give each a stable anchor and substantive labeled fields. Keep acceptance separate from intended verification method and do not turn a code or model pointer into evidence of implementation.

## Depth and omission

Right-size to the decisions and evidence readers need. Move detailed current/proposed comparisons to change-requirements and greenfield scope to system-requirements; record that omission reason instead of duplicating them.

## Views and alternatives

No view is required by default. Add a context or behavior view only if it answers a stated question; a requirement and acceptance table is often sufficient and needs no diagram decision field.

## Sources and review

Source IDs: DPR. Use sources for artifact purpose and information needs only; no prose is copied. Review requirement testability, model trace, scenario fitness, and acceptance quality separately from structural checks.
