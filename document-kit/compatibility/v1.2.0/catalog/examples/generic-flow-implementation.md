<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=flow-implementation template=generic-flow-implementation contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Fictional flow implementation example

> Status: designed-only | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D2030"></a>

## 1. Allocation

**Components:** A controller requests work and a reviewer supplies a decision.

**Interfaces:** The controller passes a request identifier and receives a recorded judgment.

<!-- document-kit:content:allocation:start -->
The controller and reviewer have separate responsibilities.
<!-- document-kit:content:allocation:end -->

<a id="D2031"></a>

## 2. Behavior and source projection

**Behavior:** Validate the plan, then construct the initial run.

**Source projection:** src/example/factory.rs is a proposed implementation location, not an existing artifact.

**Constraints:** The caller serializes access; concurrency is outside this example.

<!-- document-kit:content:projection:start -->
Source allocation is planned and subject to implementation review.
<!-- document-kit:content:projection:end -->

<a id="D2032"></a>

## 3. Verification and limits

**Model elements:** ExampleContext is the context boundary; no flow realization is asserted.

**Verification links:** Planned cases cover empty plans, repeated stages, and first-stage initialization; none has executed.

**Unresolved items:** Concurrency and durable recovery remain outside the fictional example.

<!-- document-kit:content:trace:start -->
The context model and planned cases are review inputs, not proof of implementation.
<!-- document-kit:content:trace:end -->
