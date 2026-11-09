# Change requirements authoring guide

Select this template to specify current-versus-proposed behavior, affected requirements, compatibility, migration, and acceptance for an existing system. Do not select it for greenfield scope with no current behavior; use system-requirements instead.

## Required content

Provide the current behavior and baseline, proposed delta, affected requirements and rationale, compatibility constraints and migration notes, measurable acceptance, model trace, and unresolved impacts. Dependencies are the existing requirements, current model, consumer contracts, and release/migration policy; downstream regression and new-behavior cases must be identified.

Worked field: `compatibility_constraints = "Heat-alert cadence may change, but Reading fields, query operation, UTC format, and error meanings remain unchanged."` This bounds the delta for consumers.

## Structure

Use change summary, affected requirements, compatibility/migration, acceptance, and trace/limits in order. Explicitly name what does not change so reviewers can distinguish intentional compatibility from accidental omission.

## Depth and omission

Right-size to the delta and its blast radius; link unchanged contracts rather than rewriting greenfield requirements. Omit full system scope and record that system-requirements owns any genuinely new capability.

## Views and alternatives

A behavior or dependency view is conditional on a complicated delta; a current/proposed/impact table is normally sufficient and should include compatibility and acceptance columns.

## Sources and review

Source IDs: REQ42, OPENSPEC. Review baseline accuracy, affected-item completeness, migration safety, and whether regression acceptance proves both the new behavior and retained behavior.
