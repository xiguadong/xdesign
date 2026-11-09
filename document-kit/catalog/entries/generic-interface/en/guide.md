# Generic interface authoring guide

Select this template for a small interface that combines participants, operations or data, errors, invariants, compatibility, and trace without needing a dedicated operation- or data-focused treatment. Do not select it for one operation's detailed request/response behavior (use operation-contract) or for schema and versioning alone (use data-contract).

## Required content

Provide the boundary and participants, operations and data shapes, error behavior and invariants, version and compatibility policy, and model/verification trace. Dependencies are the owning design and the exchanged model elements; downstream tests must cover both normal and error behavior.

Worked field: `error_behavior = "Return UNKNOWN_SENSOR for an unknown id and NOT_READY for empty history; neither changes buffered values."` This distinguishes two outcomes and preserves state.

## Structure

Use the five generic chapters in order with explicit anchors. Keep request, response, and error meanings precise, and avoid copying orchestration or schema detail into multiple authorities.

## Depth and omission

Right-size to a small mixed interface. Use operation-contract for command/query semantics and data-contract for field validation and version interoperability; record those links rather than creating another contract owner.

## Views and alternatives

Request/response, error, and schema tables are normally primary. Add an interaction or relationship figure only when ordering or ownership is otherwise unclear.

## Sources and review

Source IDs: DPR. Review participant coverage, error completeness, invariants, compatibility, and whether the generic interface duplicates a focused operation or data contract.
