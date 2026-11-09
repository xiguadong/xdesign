# Operation contract authoring guide

Select this template to specify a command, query, endpoint, or tool operation with request, response, errors, invariants, and compatibility. Do not select it for a data schema with no operation behavior; use data-contract instead.

## Required content

Provide operation boundary and participants, request and response shapes, error behavior and invariants, version and compatibility policy, and model/verification trace. Dependencies are the operation's design responsibility, data contract, and model interaction; orchestration and executable tests consume this contract without rewriting it.

Worked field: `invariants = "At most one terminal outcome per request, no history mutation, and success returns the newest valid value at read time."` This constrains behavior beyond field presence.

## Structure

Use purpose, operation contract, errors/invariants, compatibility, and trace in order. Define each outcome's meaning and ordering prerequisites instead of listing only an operation name and response schema.

## Depth and omission

Right-size to one operation and its observable behavior; link data shapes and interaction sequences rather than copying their owners. Omit schema-only documentation and record data-contract as owner.

## Views and alternatives

Request/response and error tables are normally primary. Add an interaction figure only when ordering or alternate flows need visual explanation and the manifest declares that concern.

## Sources and review

Source IDs: GOOD-DOCS, DPR. Review operation completeness, error and invariant precision, compatibility, and whether examples distinguish planned behavior from executed evidence.
