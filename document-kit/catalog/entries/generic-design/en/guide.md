# Generic design authoring guide

Select this template for a balanced design that needs purpose, structure, behavior, rationale, implementation projection, and limits in one document. Do not select it when the main question is interaction ordering, lifecycle states, or data meaning; use interaction-flow, state-machine, or data-model respectively.

## Required content

Provide requirements and model trace, component and interface responsibilities, behavior and constraints, chosen design and tradeoffs, intended source units and verification links, and unresolved limits. Dependencies are approved requirements and architecture; implementation and execution evidence must remain separate from the design projection.

Worked field: `tradeoffs = "A ten-entry FIFO bounds memory but cannot answer long-term trend queries."` This states a real consequence rather than saying the design is simply best.

## Structure

Use the six generic chapters with stable anchors. Name intended source units without claiming they exist, and keep residual constraints visible instead of hiding them behind a complete-sounding structure.

## Depth and omission

Right-size to one coherent design. Move detailed sequence diagrams to interaction-flow, transition semantics to state-machine, and field/cardinality analysis to data-model; record those links instead of reproducing their content.

## Views and alternatives

Use interaction, state, or structure views only for a named concern. Prose or a table can answer simple behavior and data relationships with a documented reason.

## Sources and review

Source IDs: DPR. Review whether the allocation is technically coherent, tradeoffs are real, source projection is honest, and focused scenarios have been linked or omitted deliberately.
