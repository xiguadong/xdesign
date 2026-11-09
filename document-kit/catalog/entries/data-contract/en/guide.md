# Data contract authoring guide

Select this template to specify exchanged or persisted data shapes, validation, required fields, versioning, and compatibility. Do not select it for operation orchestration or domain behavior; use operation-contract or interaction-flow for those concerns.

## Required content

Provide schema and required fields, validation and invariants, version and compatibility policy, participants and boundary, and model/verification trace. Dependencies are the semantic data model, serialization format, and consumer versions; operation contracts and interoperability tests consume but do not redefine the schema.

Worked field: `required_fields = "sensorId, utcTimestamp, and celsius are required; quality is optional and null never replaces a required field."` This makes presence and nullability explicit.

## Structure

Use purpose, data contract, validation/invariants, compatibility, and trace in order. Define types, units, required/optional status, and treatment of unknown fields separately from operation errors.

## Depth and omission

Right-size to exchanged meaning and interoperability; link sequencing and domain invariants that belong elsewhere. Omit orchestration and record operation-contract or interaction-flow as owner.

## Views and alternatives

Schema and compatibility tables are normally primary. Add a relationship view only when ownership or cardinality is unclear; no diagram is required by this template alone.

## Sources and review

Source IDs: SPECD, GOOD-DOCS. Review field semantics, validation, backward/forward compatibility, and whether version rules are demonstrated with real valid and invalid values.
