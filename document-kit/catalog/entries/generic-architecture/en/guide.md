# Generic architecture authoring guide

Select this template when one document must cover context, composition, behavior, constraints, and trace at a similar level of detail. Do not select it for an external-only boundary question (use system-context) or an internal-part responsibility question (use component-composition).

## Required content

Provide model sources, external actors and flows, internal parts and relationships, lifecycle behavior, constraints and rationale, and requirement/verification trace. Dependencies are the requirements being allocated and the model's parts and exchanges; downstream interface and verification documents need links, not copied contracts.

Worked field: `relationships = "Collector appends validated readings to RingBuffer; Exporter uses readLatest and cannot mutate history."` This names both dependency and restriction.

## Structure

Use the five generic chapters in order with explicit anchors. Keep external context and internal composition present but concise; if one dominates and the other adds no information, choose the focused scenario instead.

## Depth and omission

Allocate only meaningful responsibilities and behavior. Move detailed boundary ownership to system-context and detailed internal dependency analysis to component-composition; record the omission rather than filling generic chapters with repetition.

## Views and alternatives

Choose a view only when it answers a context, composition, or behavior question. A responsibility or exchange table may replace a figure with a rationale; no generic architecture-wide figure mandate exists.

## Sources and review

Source IDs: DPR. Sources guide scenario depth, not engineering claims. Review boundary correctness, responsibility cohesion, behavior completeness, and whether any omitted scenario has been recorded.
