<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=design template=interaction-flow contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster current-reading interaction flow

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D100"></a>

## 1. Purpose and allocation

**Design subject:** Synchronous request/validation/response flow between Controller and Sensor with one timeout alternate.

**Requirements links:** Latest-reading query, explicit invalid-input rejection, and no-fabricated-value requirements drive the flow.

<!-- document-kit:content:purpose:start -->
The design explains the ordering for one current-reading request, including validation and timeout behavior rather than static schema details.
<!-- document-kit:content:purpose:end -->

<a id="D101"></a>

## 2. Participants and interfaces

**Participants:** Controller is the requester; Sensor is the provider; RingBuffer is an internal read-only participant, not an external actor.

**Interfaces:** ReadRequest carries sensorId; ReadingResponse carries Reading, NOT_READY, UNKNOWN_SENSOR, or TIMEOUT.

<!-- document-kit:content:participants:start -->
Controller originates ReadRequest, Sensor checks sensorId and buffer state, and the operation boundary returns ReadingResponse or an explicit outcome.
<!-- document-kit:content:participants:end -->

<a id="D102"></a>

## 3. Main interaction flow

**Interactions:** Controller sends ReadRequest, Sensor validates sensorId, reads newest valid Reading, and sends ReadingResponse.

**Ordering:** Validation precedes buffer access, buffer access precedes response, and timeout terminates the attempt without a value response.

**Diagram decisions:** [{"concern":"interaction-ordering","applies":true,"choice":"figure","rationale":"Numbered arrows and lifelines make request, validation, response, and timeout order explicit.","asset":"assets/interaction-flow.svg","model_view":"Example::InteractionFlow"}]

<!-- document-kit:content:flow:start -->
The numbered flow is request, validation, buffer read, then response; timeout cannot produce a ReadingResponse after the deadline.

![Aster request and response ordering](assets/interaction-flow.svg)

> **Figure D102-1 — Aster request and response ordering.**  
> **Question:** In what order do request, validation, response, and timeout occur?  
> **Model view:** Example::InteractionFlow  
> **Companions:** The operation-contract and alternate-flow tables.  
> **Status:** renderable-now; designed-only  
> **Reading:** Controller sends ReadRequest, Sensor validates the request, then returns ReadingResponse; the dashed timeout path returns no reading.  
> **Limits:** The view is fictional and omits transport encoding, authentication, and precise timeout duration.
<!-- document-kit:content:flow:end -->

<a id="D103"></a>

## 4. Alternate flows and errors

**Alternate flows:** UNKNOWN_SENSOR skips reading, NOT_READY follows an empty read, and TIMEOUT ends the operation without ReadingResponse.

**Error behavior:** Errors are explicit outcomes and never mutate history or turn an old value into a newly timestamped reading.

<!-- document-kit:content:alternates:start -->
Unknown sensorId returns UNKNOWN_SENSOR before buffer access, empty history returns NOT_READY, and timeout produces no reading rather than a stale fabricated response.
<!-- document-kit:content:alternates:end -->

<a id="D104"></a>

## 5. Behavior and constraints

**Behavior:** One request produces at most one terminal outcome and never changes buffer contents or reading timestamps.

**Constraints:** No response after timeout, no buffer mutation, and no implicit retry or fallback value.

<!-- document-kit:content:constraints:start -->
The flow is read-only and single-request; concurrent retries, authentication, and transport retransmission are deliberately outside this example.
<!-- document-kit:content:constraints:end -->

<a id="D105"></a>

## 6. Projection and verification

**Source projection:** RequestHandler, RequestValidator, ReadLatestExporter, and a controllable timeout are intended design units.

**Verification links:** Ordered success, unknown-sensor, empty-history, and timeout event-sequence cases are planned checks.

<!-- document-kit:content:projection:start -->
Tests record event order for success, unknown sensor, empty history, and timeout paths and inspect the corresponding Example::InteractionFlow view.
<!-- document-kit:content:projection:end -->
