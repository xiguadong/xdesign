<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=design template=interaction-flow language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
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

**Interfaces:** ReadRequest carries sensorId. ReadingResponse carries exactly one of Reading, NOT_READY, UNKNOWN_SENSOR or TIMEOUT; an error carries no Reading value.

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

**Alternate flows:** UNKNOWN_SENSOR skips reading, NOT_READY denotes empty history, and TIMEOUT is the single error ReadingResponse emitted at the deadline; later provider responses are suppressed.

**Error behavior:** Errors are explicit outcomes and never mutate history or turn an old value into a newly timestamped reading.

<!-- document-kit:content:alternates:start -->
UNKNOWN_SENSOR returns before buffer access; empty history returns NOT_READY. At the deadline the operation boundary emits one ReadingResponse(TIMEOUT) without a Reading and discards any later provider result. It never retimestamps an old value.
<!-- document-kit:content:alternates:end -->

<a id="D104"></a>

## 5. Behavior and constraints

**Behavior:** One request produces at most one terminal outcome and never changes buffer contents or reading timestamps.

**Constraints:** No response after a terminal outcome, history mutation, implicit retry or substitute measurement; the deadline TIMEOUT error itself is permitted.

<!-- document-kit:content:constraints:start -->
This read-only, single-request design excludes concurrent retries, authentication and retransmission. Each request has at most one terminal ReadingResponse, including the deadline TIMEOUT error; after that outcome no late provider response or fallback value may be forwarded.
<!-- document-kit:content:constraints:end -->

<a id="D105"></a>

## 6. Projection and verification

**Source projection:** RequestHandler, RequestValidator, ReadLatestExporter, and a controllable timeout are intended design units.

**Verification links:** Ordered success, unknown-sensor, empty-history, and timeout event-sequence cases are planned checks.

<!-- document-kit:content:projection:start -->
Planned tests compare event order and unique terminal outcomes against the fictional language-side interaction prose on this page. Example::InteractionFlow is only an empty named view declaration, not behavior evidence. Source unit names remain proposed allocations.
<!-- document-kit:content:projection:end -->
