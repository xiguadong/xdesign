<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=flow-definition template=generic-flow-definition language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# Fictional flow definition example

> Status: designed-only | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D2020"></a>

## 1. Stages and gates

**Stages:** Prepare collects and validates the request; Review assesses completeness and records a decision.

**Transitions:** Prepare enters Review only after a readable, complete request; Review completes only after the designated caller judgment is recorded.

**Exit gates:** Prepare requires a readable, complete request; Review requires an explicit judgment by the designated caller.

<!-- document-kit:content:stages:start -->
| Stage | Next stage and exit condition |
| --- | --- |
| Prepare | Review only after the request is readable and complete |
| Review | Complete only after the designated caller judgment is recorded |

Prepare collects and validates the request; Review assesses completeness and records the explicit decision. Input existence alone cannot establish review completion.
<!-- document-kit:content:stages:end -->

<a id="D2021"></a>

## 2. Behavior and constraints

**Behavior:** Validate Plan and construct the first-stage Run, then execute stage input checks and guarded transitions in order.

**Invariants:** Invalid Plan produces no Run; incomplete Prepare cannot enter Review; absent recorded judgment cannot complete the flow.

<!-- document-kit:content:behavior:start -->
Factory initialization is a prerequisite. The flow then advances only through the declared stages when their exit gates are satisfied; initialization alone is not completion.
<!-- document-kit:content:behavior:end -->

<a id="D2022"></a>

## 3. Trace and verification

**Model elements:** Package Example supplies participant context only; it declares no sample-flow realization.

**Requirements links:** Requirement allocation remains an author review item; this example claims no approved requirement.

**Verification links:** Planned checks cover empty plans, repeated stages, first-stage initialization, rejection of incomplete Prepare and rejection of completion without recorded gate judgment; none has executed.

<!-- document-kit:content:trace:start -->
The context model and planned cases are review inputs, not proof of implementation.
<!-- document-kit:content:trace:end -->
