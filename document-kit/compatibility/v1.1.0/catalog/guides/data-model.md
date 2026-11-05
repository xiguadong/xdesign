# Data model authoring guide

Select this template to define entities, fields, relationships, ownership, invariants, and persistence or exchange meaning. Do not select it for interaction sequencing or interface error behavior; use interaction-flow and operation-contract for those concerns.

## Required content

Provide entity and field definitions, cardinalities and ownership, lifecycle and invariants, and source/verification projection. Dependencies are the domain requirements, model elements, and persistence/exchange responsibilities; serialized data contracts and operation flows are downstream contracts, not duplicate owners.

Worked field: `invariants = "Reading.celsius is finite, timestamps are UTC and nondecreasing per sensor, exported values are immutable, and eviction is FIFO."` These are checkable data rules.

## Structure

Use purpose, entities/fields, relationships/ownership, invariants/lifecycle, and projection in order. State required versus optional fields and explain value ownership and mutation rules independently of any transport.

## Depth and omission

Right-size to meaning and invariants that affect behavior or persistence; link request order and error outcomes instead of copying them. Omit orchestration and record interaction-flow or operation-contract as owner.

## Views and alternatives

A structure view is conditional on complex relationships. A schema and invariant table is often sufficient and requires no figure when cardinalities and ownership are explicit.

## Sources and review

Source IDs: SPECD, DPR. Review semantic field definitions, ownership, invariants, persistence meaning, and whether schema checks test actual rules rather than merely file existence.
