# Component composition authoring guide

Select this template to explain internal parts, responsibilities, connections, and meaningful levels of decomposition. Do not select it for external context alone or runtime node allocation without components; use system-context for boundary ownership and a deployment-specific profile for nodes.

## Required content

Provide internal part inventory, responsibility allocation, relationships and interfaces, lifecycle behavior, constraints and rationale, and model/verification trace. Dependencies are system context, requirements, and internal model parts; interaction and verification artifacts should consume the allocation without duplicating it.

Worked field: `interfaces = "RingBuffer exposes append(Reading) only to Collector and readLatest() to Exporter; Exporter cannot evict or append."` This states both allowed and forbidden dependencies.

## Structure

Use context, parts/responsibilities, relationships/interfaces, behavior, constraints, and trace in order. Decompose only where responsibility or dependency clarity improves, and keep external actors as endpoints rather than internal parts.

## Depth and omission

Right-size to important parts and relationships; avoid arbitrary layers and external-context repetition. Omit external-only details and record system-context as their owner.

## Views and alternatives

A structure/composition view is expected, but a responsibility/dependency table is a valid reasoned substitute for a small number of parts. Do not add a diagram solely because this template is selected.

## Sources and review

Source IDs: ARC42, DPR, C4. Review cohesion, dependency direction, interface ownership, purposeful decomposition, and whether any figure or table accurately represents responsibilities.
