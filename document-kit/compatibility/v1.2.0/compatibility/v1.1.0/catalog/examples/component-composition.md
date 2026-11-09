<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=architecture template=component-composition contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster collector buffer and exporter composition

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="B110"></a>

## 1. Context and boundary

**System boundary:** Only Collector, RingBuffer, and Exporter are internal components in this example.

**External actors:** Probe and Operator appear only as interface endpoints so component responsibilities remain clear.

<!-- document-kit:content:context:start -->
Within the AsterHub boundary, three components divide acquisition, retention, and query work; external actors are intentionally omitted from this composition view.
<!-- document-kit:content:context:end -->

<a id="B111"></a>

## 2. Parts and responsibilities

**Parts:** Collector, UtcClock, RingBuffer, and Exporter are the purposeful parts; no further decomposition is needed.

**Responsibilities:** Collector rejects invalid values, clock supplies UTC, buffer evicts oldest-first, and exporter returns newest or NOT_READY.

**Diagram decisions:** [{"concern":"component-structure","applies":true,"choice":"table","rationale":"Three responsibilities and two relationships are clearer in a table than another diagram.","asset":null,"model_view":null}]

<!-- document-kit:content:parts:start -->
Collector validates and timestamps, RingBuffer keeps ten valid readings, and Exporter answers queries without mutation. A table is sufficient for these three responsibilities.
<!-- document-kit:content:parts:end -->

<a id="B112"></a>

## 3. Relationships and interfaces

**Relationships:** Collector→UtcClock for time, Collector→RingBuffer for append, and Exporter→RingBuffer for readLatest are the only internal dependencies.

**Interfaces:** RingBuffer exposes append(Reading) to Collector and readLatest() to Exporter; mutation is unavailable to Exporter.

<!-- document-kit:content:relationships:start -->
Collector depends on UtcClock and writes RingBuffer; Exporter depends only on RingBuffer's read interface and cannot call append or evict.
<!-- document-kit:content:relationships:end -->

<a id="B113"></a>

## 4. Behavior and lifecycle

**Interactions:** Validation precedes append, readLatest returns a copy, and export failure never propagates back into collection.

**States:** Empty, partially full, and full buffer states distinguish NOT_READY, append, and FIFO eviction behavior.

<!-- document-kit:content:behavior:start -->
The collector continues appending while the exporter is unavailable, and the eleventh valid sample evicts only the oldest retained sample.
<!-- document-kit:content:behavior:end -->

<a id="B114"></a>

## 5. Constraints and decisions

**Constraints:** One writer per buffer, read-only export, ten-entry capacity, and no controller callback into collection.

**Design rationale:** Single-writer FIFO and narrow interfaces make ordering, ownership, and failure isolation easy to inspect.

<!-- document-kit:content:constraints:start -->
The composition intentionally avoids a shared database or bidirectional controller dependency so each internal responsibility remains independently testable.
<!-- document-kit:content:constraints:end -->

<a id="B115"></a>

## 6. Trace and verification

**Model elements:** Example::Sensor provides the owning part; Collector, RingBuffer, and Exporter are language-side design responsibilities.

**Verification links:** Planned capacity, read-only export, FIFO eviction, and controller-outage cases verify composition behavior.

<!-- document-kit:content:trace:start -->
The composition traces to Example::Sensor and the interaction design; buffer eviction and exporter-isolation checks cover the important relationships.
<!-- document-kit:content:trace:end -->
