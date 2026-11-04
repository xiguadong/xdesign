<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=design template=data-model contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster reading and buffer data model

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D100"></a>

## 1. Purpose and allocation

**Design subject:** Reading values, their fields and invariants, buffer ownership, and retention meaning.

**Requirements links:** UTC timestamps, finite Celsius values, latest-valid retention, and immutable history requirements drive the model.

<!-- document-kit:content:purpose:start -->
The data model defines what a Reading means, who owns it, and how the bounded buffer preserves value meaning without describing request sequencing.
<!-- document-kit:content:purpose:end -->

<a id="D101"></a>

## 2. Entities and fields

**Entities:** Reading is the only entity; Sensor is an identified source, and RingBuffer is an owner of retained Reading values.

**Fields:** sensorId is nonempty text, utcTimestamp is UTC ISO 8601, celsius is finite, and quality is valid, suspect, or missing.

<!-- document-kit:content:entities:start -->
Reading is a value object with sensorId, utcTimestamp, celsius, and quality; Sensor identity is stable across all readings.
<!-- document-kit:content:entities:end -->

<a id="D102"></a>

## 3. Relationships and ownership

**Relationships:** Sensor 1→many Reading and RingBuffer 1→0..10 Reading; no relationship exists between Reading and ForecastRecord.

**Ownership:** AsterHub owns values after validation, buffer owns retention order, and exporter receives immutable copies.

**Diagram decisions:** [{"concern":"data-relationships","applies":true,"choice":"table","rationale":"Two cardinalities and one ownership rule are clearer in a schema table.","asset":null,"model_view":null}]

<!-- document-kit:content:relationships:start -->
Each Reading belongs to exactly one Sensor and is retained by at most one hub buffer; export copies values rather than sharing mutable objects.
<!-- document-kit:content:relationships:end -->

<a id="D103"></a>

## 4. Invariants and lifecycle

**Invariants:** Finite celsius, UTC timestamps, stable sensorId, nondecreasing time, immutable exported values, and FIFO eviction are mandatory.

**Lifecycle:** A Reading is created by validation, retained up to eviction, copied for export, and never edited in place.

<!-- document-kit:content:invariants:start -->
Timestamps never move backward within one sensor, values must be finite, and buffer eviction removes only the oldest valid entry.
<!-- document-kit:content:invariants:end -->

<a id="D104"></a>

## 5. Projection and verification

**Source projection:** Reading, ReadingQuality, and RingBuffer are intended data types; no ORM or database schema is implied.

**Verification links:** Field validation, timestamp ordering, cardinality, immutability, and FIFO cases are planned checks.

<!-- document-kit:content:projection:start -->
The projection uses an immutable Reading record and a small invariant suite for finite values, time order, cardinality, and copy-on-export behavior.
<!-- document-kit:content:projection:end -->
