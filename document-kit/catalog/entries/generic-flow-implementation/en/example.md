<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=flow-implementation template=generic-flow-implementation language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# Fictional flow implementation example

> Status: designed-only | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D2030"></a>

## 1. Allocation

**Components:** Controller initiates work and maintains flow state and gates. Reviewer supplies a judgment and cannot mutate the request.

**Interfaces:** Controller passes a request identifier to Reviewer only in Review after readable, complete Prepare input; Reviewer returns a judgment that Controller records before completion. The interface grants no request-mutation access.

<!-- document-kit:content:allocation:start -->
Controller owns state and gate enforcement; Reviewer supplies a judgment without request-mutation access. Separating these permissions prevents review logic from bypassing stage conditions.
<!-- document-kit:content:allocation:end -->

<a id="D2031"></a>

## 2. Behavior and source projection

**Behavior:** Validate Plan and construct initial Run; Controller then checks Prepare input, advances to Review only after its gate, invokes Reviewer in Review, records the designated judgment and completes only after that gate.

**Source projection:** src/example/factory.rs is a proposed implementation location, not an existing artifact.

**Constraints:** The caller serializes access; concurrency is outside this example.

<!-- document-kit:content:projection:start -->
Source allocation is planned and subject to implementation review.
<!-- document-kit:content:projection:end -->

<a id="D2032"></a>

## 3. Verification and limits

**Model elements:** Package Example supplies participant context only; it declares no sample-flow realization.

**Verification links:** Planned checks cover empty plans, repeated stages, first-stage initialization and invalid judgment-interface calls outside Review or without complete Prepare; verify no request mutation and no completion without a recorded judgment. None has executed.

**Unresolved items:** Concurrency and durable recovery remain outside the fictional example.

<!-- document-kit:content:trace:start -->
The context model and planned cases are review inputs, not proof of implementation.
<!-- document-kit:content:trace:end -->
