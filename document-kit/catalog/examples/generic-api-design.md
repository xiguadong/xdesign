<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=api-design template=generic-api-design contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Fictional api design example

> Status: designed-only | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="U2000"></a>

## Purpose and boundary

**Api identity:** Example::create_run

**Api kind:** function

**Layer:** core

**Purpose:** Construct an in-memory run for a validated plan.

<!-- document-kit:content:purpose:start -->
This function example defines an in-memory boundary with no execution claim.
<!-- document-kit:content:purpose:end -->

<a id="U2001"></a>

## Declaration and contract

**Declaration:** `create_run(plan: Plan) -> Result<Run, InvalidPlan>`

**Inputs:** One caller-owned plan with a nonempty ordered stage list.

**Outputs:** A new run at its first stage; ownership transfers to the caller.

**Errors:** InvalidPlan when the stage list is empty or repeats an identifier.

**Side effects:** Not applicable: this fictional function does not persist data or invoke callbacks.

<!-- document-kit:content:contract:start -->
Invalid inputs are refused before a run is constructed.
<!-- document-kit:content:contract:end -->

<a id="U2002"></a>

## Behavior and constraints

**Behavior:** Validate the plan, then construct the initial run.

**Invariants:** An invalid plan never produces a run.

**Constraints:** The caller serializes access; concurrency is outside this example.

<!-- document-kit:content:behavior:start -->
The stated invariant constrains the fictional transition.
<!-- document-kit:content:behavior:end -->

<a id="U2003"></a>

## Trace and verification

**Model realization or reason:** Language-side helper; ExampleContext supplies boundary context but does not claim a realization of this function.

**Source projection:** src/example/factory.rs is a proposed implementation location, not an existing artifact.

**Requirements links:** Requirement allocation remains an author review item; this example claims no approved requirement.

**Design links:** The declaration and behavior sections on this page own the fictional contract.

**Verification links:** Planned cases cover empty plans, repeated stages, and first-stage initialization; none has executed.

<!-- document-kit:content:trace:start -->
The context model and planned cases are review inputs, not proof of implementation.
<!-- document-kit:content:trace:end -->

<a id="U2004"></a>

## Status and limits

**Unresolved items:** Concurrency and durable recovery remain outside the fictional example.

<!-- document-kit:content:limits:start -->
The status is designed-only; no acceptance result is supplied.
<!-- document-kit:content:limits:end -->
