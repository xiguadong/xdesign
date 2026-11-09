<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=architecture template=generic-architecture contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster sensor hub architecture

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="B100"></a>

## 1. Context and boundary

**System boundary:** The hub owns sampling, validation, short-term buffering, and export; probe hardware and controller decisions are outside.

**External actors:** The local probe supplies readings and the external controller requests and receives validated readings.

<!-- document-kit:content:context:start -->
The fictional Aster hub sits between one local probe and the external controller. It validates samples but does not choose irrigation actions.
<!-- document-kit:content:context:end -->

<a id="B101"></a>

## 2. Composition and responsibilities

**Parts:** Collector, UTC clock, ten-entry ring buffer, and export adapter are the only parts in this worked architecture.

**Relationships:** Collector writes validated readings to the buffer; exporter reads but never mutates buffered history; the clock supplies UTC timestamps.

<!-- document-kit:content:composition:start -->
A collector validates each sample, a ring buffer retains the latest ten valid readings, and an exporter answers controller requests.
<!-- document-kit:content:composition:end -->

<a id="B102"></a>

## 3. Behavior and lifecycle

**Interactions:** Probe samples enter the collector, valid readings move to the buffer, and controller queries trigger export of the newest reading.

**States:** Empty, collecting, and degraded-export states explain startup, normal sampling, and temporary controller unavailability.

<!-- document-kit:content:behavior:start -->
Startup initializes an empty buffer, periodic operation appends valid samples, and an export outage leaves collection running without discarding history.
<!-- document-kit:content:behavior:end -->

<a id="B103"></a>

## 4. Constraints and decisions

**Constraints:** Retain exactly ten valid readings, use UTC at every boundary, reject non-finite values, and never block collection on export.

**Design rationale:** A small read-only history is sufficient for the example and isolates sampling reliability from controller behavior.

<!-- document-kit:content:constraints:start -->
The buffer is deliberately bounded and export is read-only to prevent a slow controller from mutating sensor history or exhausting memory.
<!-- document-kit:content:constraints:end -->

<a id="B104"></a>

## 5. Trace and verification

**Model elements:** Example::Sensor and Example::Controller are the fictional model participants for composition and exchange.

**Requirements links:** The 60-second sampling and latest-valid-reading requirements own the observable behavior.

**Verification links:** Planned buffer-capacity and export-outage cases cover the main architectural constraints.

<!-- document-kit:content:trace:start -->
The architecture traces to Example::Sensor, Example::Controller, the stated sampling requirements, and a planned buffer/export verification case.
<!-- document-kit:content:trace:end -->
