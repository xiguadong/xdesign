# Requirements overview

> Project: xdesign | Status: working requirements | Review position: no SRR decision or
> frozen functional baseline.

<a id="r15"></a>

## R15 Scope, reading order, and baseline position

This directory owns the system requirements specification: the behaviors and constraints
the project must satisfy, their rationale, their links to the SysML model, and the
verification planned for each. The requirement definitions are in
`mbse/model/requirements.sysml`; satisfaction links are in
`mbse/model/traceability.sysml`. This prose makes the current set readable and exposes
missing acceptance detail; it does not replace those model elements.

Read the [requirements specification](01_baseline.md#r04) from scope and user goals
through the inventory, interfaces, data, verification plan, and traceability matrix.
The [use-case view](01_baseline.md#d17) sits beside the user scenarios it explains,
and the [requirements view](01_baseline.md#d15) sits beside the traceability discussion.
For allocations and interface detail, continue to the [architecture design](../arch/02_design.md#b05)
and [interface control document](../icd/01_interfaces.md#i01). The
[review checklist](../review/02_checklist.md#v09) states what an SRR must still decide.

The current specification is a working requirements set. Some requirements lack an
individual measurable acceptance criterion, verification method, phase, or executed
case. No SRR outcome or configuration-controlled functional baseline is recorded.
Per-wish passed assessments apply to the revisions they name; they do not close these
project-level requirements gaps.
