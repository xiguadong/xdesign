<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=design template=generic-design contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster reading pipeline design

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D100"></a>

## 1. Purpose and allocation

**Design subject:** The reading pipeline from probe value to controller response, including validation and bounded history.

**Requirements links:** The sampling, UTC timestamp, invalid-value rejection, and latest-valid-reading requirements drive this design.

<!-- document-kit:content:purpose:start -->
The design allocates validation, buffering, and export for the fictional Aster hub while preserving the latest valid reading during controller outages.
<!-- document-kit:content:purpose:end -->

<a id="D101"></a>

## 2. Structure and interfaces

**Components:** ProbeValidator, UtcClock, RingBuffer, and ReadLatestExporter form the design's small internal structure.

**Interfaces:** Validator accepts ProbeSample, buffer accepts Reading, and exporter returns Reading or NOT_READY without mutation operations.

<!-- document-kit:content:structure:start -->
A validator returns either a UTC Reading or an error, the buffer appends only valid readings, and the exporter exposes readLatest without exposing buffer mutation.
<!-- document-kit:content:structure:end -->

<a id="D102"></a>

## 3. Behavior and constraints

**Behavior:** Validate then timestamp then append; export newest valid value or NOT_READY, preserving history during controller failure.

**Constraints:** No NaN or infinity crosses validation, buffer capacity is ten, and export never changes reading order or content.

<!-- document-kit:content:behavior:start -->
Invalid samples are dropped before buffering, the eleventh valid sample evicts the oldest, and an empty buffer produces NOT_READY rather than a fabricated value.
<!-- document-kit:content:behavior:end -->

<a id="D103"></a>

## 4. Decisions and rationale

**Chosen design:** Use single-point validation and a ten-entry FIFO ring with a read-only exporter.

**Tradeoffs:** Older history is intentionally lost; this is simpler and bounded but cannot answer long-term trend queries.

<!-- document-kit:content:rationale:start -->
Validation before buffering avoids later error handling in every reader, while a ring buffer favors predictable memory over unlimited history.
<!-- document-kit:content:rationale:end -->

<a id="D104"></a>

## 5. Implementation projection and verification

**Source projection:** ProbeValidator, RingBuffer, and ReadLatestExporter are intended source units; tests target their public contracts.

**Verification links:** Invalid-value rejection, FIFO eviction, empty NOT_READY, and latest-valid outage cases are planned checks.

**Model elements:** Example::Sensor and Example::Controller are the model participants represented by the design boundaries.

<!-- document-kit:content:projection:start -->
The fictional source projection names validator, buffer, and exporter units plus capacity, invalid-input, empty-export, and outage cases.
<!-- document-kit:content:projection:end -->

<a id="D105"></a>

## 6. Status and limits

**Unresolved items:** Thread safety, durable storage, transport security, and deployment configuration need separate design decisions.

<!-- document-kit:content:limits:start -->
Concurrency, persistence after restart, and production API transport are deliberately excluded from this focused design example.
<!-- document-kit:content:limits:end -->
