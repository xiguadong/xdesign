<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=flow-views template=generic-flow-views language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# Fictional flow views example

> Status: designed-only | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D2040"></a>

## 1. Coverage and applicability

**View coverage:** The stage table in definition.md#d2020 (Chinese: definition.md#d13820) owns stage order and exit conditions; responsibility prose in implementation.md#d2030 (Chinese: implementation.md#d13930) owns Controller/Reviewer allocation.

**Applicability reasons:** A sequence figure is optional when interaction timing needs discussion; this example has no timing claim.

<!-- document-kit:content:coverage:start -->
View applicability is decided by the author and remains subject to review.
<!-- document-kit:content:coverage:end -->

<a id="D2041"></a>

## 2. Views and references

**Model views:** Not applicable: package Example supplies participant context but declares no rendered sample-flow view.

**Figure references:** Not applicable: this example uses prose and a stage table, without an SVG.

**Verification links:** Planned review compares the referenced definition stage table with its stage/transition/gate fields for order and coverage, then compares implementation responsibility prose with Controller state/gate ownership and Reviewer permissions. Check that neither completion before judgment nor Reviewer mutation is allowed. SVG fidelity is inapplicable because no SVG is selected; no review has executed.

<!-- document-kit:content:views:start -->
There is no figure to validate in this fictional input.
<!-- document-kit:content:views:end -->

<a id="D2042"></a>

## 3. Status and limits

**Unresolved items:** Concurrency and durable recovery remain outside the fictional example.

<!-- document-kit:content:limits:start -->
The status is designed-only; no acceptance result is supplied.
<!-- document-kit:content:limits:end -->
