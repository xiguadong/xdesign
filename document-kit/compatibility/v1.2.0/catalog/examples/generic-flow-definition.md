<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=flow-definition template=generic-flow-definition contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Fictional flow definition example

> Status: designed-only | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D2020"></a>

## 1. Stages and gates

**Stages:** Prepare collects the request; review assesses its completeness.

**Transitions:** Prepare proceeds to review after the request is complete.

**Exit gates:** Prepare requires a readable request; review requires an explicit caller judgment.

<!-- document-kit:content:stages:start -->
| Stage | Next stage |
| --- | --- |
| Prepare | Review |
| Review | Complete after judgment |
<!-- document-kit:content:stages:end -->

<a id="D2021"></a>

## 2. Behavior and constraints

**Behavior:** Validate the plan, then construct the initial run.

**Invariants:** An invalid plan never produces a run.

<!-- document-kit:content:behavior:start -->
The stated invariant constrains the fictional transition.
<!-- document-kit:content:behavior:end -->

<a id="D2022"></a>

## 3. Trace and verification

**Model elements:** ExampleContext is the context boundary; no flow realization is asserted.

**Requirements links:** Requirement allocation remains an author review item; this example claims no approved requirement.

**Verification links:** Planned cases cover empty plans, repeated stages, and first-stage initialization; none has executed.

<!-- document-kit:content:trace:start -->
The context model and planned cases are review inputs, not proof of implementation.
<!-- document-kit:content:trace:end -->
