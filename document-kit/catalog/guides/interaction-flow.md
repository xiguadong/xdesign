# Interaction flow authoring guide

Select this template to explain architecturally significant participants, message ordering, main interactions, and alternate flows. Do not select it for static data alone or a simple pure calculation; use data-model for meaning and schema, or concise prose for trivial logic.

## Required content

Provide the interaction purpose and requirements, participants and interfaces, ordered main flow, alternate/error flows, behavior constraints, and source/verification projection. Dependencies are the operation and architecture responsibilities plus model participants; data contracts and test cases own their own detailed shapes and inputs.

Worked field: `ordering = "Validate sensorId before buffer access, read before response, and terminate on timeout without a value response."` This defines sequence and a prohibited outcome.

## Structure

Use purpose, participants, main flow, alternates/errors, constraints, and projection in order. Distinguish messages from data definitions and make terminal outcomes explicit for every alternate path.

## Depth and omission

Right-size to meaningful ordering and failure behavior; link static entities and operation signatures instead of duplicating them. Omit pure calculations and record that no sequence view adds information.

## Views and alternatives

A sequence or activity view is conditional on meaningful interaction. For one or two simple branches, a branch table or ordered prose is sufficient; record the decision and rationale.

## Sources and review

Source IDs: ARC42, DPR, GITHUB-DOCS. Review participant completeness, event order, terminal outcomes, and whether numbered arrows and prose describe the same interaction. Diagram review is separate from structural checks.
