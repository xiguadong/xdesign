# System requirements authoring guide

Select this template to define goals, scope, functional and quality requirements, and acceptance for a new system or major capability. Do not select it for a narrow delta to existing behavior; use change-requirements so current behavior, impact, and migration stay explicit.

## Required content

Provide capability goals and stakeholders, system boundary, functional and quality requirements with rationale, measurable acceptance and methods, model/design trace, and unresolved constraints. Dependencies are the wish, stakeholder goals, and existing model context; downstream context, architecture, and verification artifacts must own their own details.

Worked field: `normative_statements = "Sample every 60 seconds, retain ten valid readings during outage, and expose the latest value to an authorized controller."` This covers capability function, quality, and access constraint.

## Structure

Use purpose, requirements, acceptance, trace, and limits in order with stable anchors. Separate capability-level outcomes from design solutions and do not turn a major capability into a disguised current/proposed change table.

## Depth and omission

Cover enough goals and quality constraints to scope the capability, then link detailed boundary, composition, behavior, and test artifacts. Omit the narrow delta analysis and record why change-requirements is not the owner.

## Views and alternatives

Context or relationship views are conditional on stakeholder and boundary complexity; requirement and acceptance tables are sufficient when clearer and need no figure.

## Sources and review

Source IDs: REQ42, DPR. Review goal-to-acceptance coherence, scope boundaries, quality measures, and whether a narrow behavior change has been misclassified as a new capability.
