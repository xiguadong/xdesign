<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=interface template=generic-interface contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster latest-reading interface

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="I100"></a>

## 1. Purpose and participants

**Boundary:** The boundary exposes read-only access to buffered readings and hides validation, storage, and sampling implementation.

**Participants:** The external controller is the caller; the Aster export adapter is the provider.

<!-- document-kit:content:purpose:start -->
The interface lets the external controller read the newest valid Aster sample without exposing internal buffer mutation or probe control.
<!-- document-kit:content:purpose:end -->

<a id="I101"></a>

## 2. Interface contract

**Operations:** getLatestReading(sensorId) is synchronous, read-only, and returns Reading, NOT_READY, or UNKNOWN_SENSOR.

**Data shapes:** Reading contains sensorId, utcTimestamp, Celsius value, and quality; no internal buffer index or mutable reference is returned.

<!-- document-kit:content:contract:start -->
getLatestReading(sensorId) returns a Reading for a known sensor or NOT_READY when no valid sample exists; unknown identifiers fail explicitly.
<!-- document-kit:content:contract:end -->

<a id="I102"></a>

## 3. Errors and invariants

**Error behavior:** Return UNKNOWN_SENSOR for unrecognized identifiers and NOT_READY for an empty buffer; transport failures are outside this contract.

**Invariants:** The operation never creates, modifies, reorders, or deletes a reading and never invents a timestamp or value.

<!-- document-kit:content:errors:start -->
An empty history is NOT_READY rather than an error, while an unknown sensor is UNKNOWN_SENSOR; every response leaves buffered history unchanged.
<!-- document-kit:content:errors:end -->

<a id="I103"></a>

## 4. Versioning and compatibility

**Version policy:** Increment the major version only for removed fields, changed meanings, or new error behavior; optional additions remain minor changes.

**Compatibility policy:** Version 1 readers may ignore quality, while version 2 readers must handle its absence for older exporters.

<!-- document-kit:content:compatibility:start -->
Version 1 clients require the original three reading fields; version 2 adds optional quality without changing operation name or error meanings.
<!-- document-kit:content:compatibility:end -->

<a id="I104"></a>

## 5. Model and verification trace

**Model elements:** Example::Controller is the external participant in the fictional read operation.

**Verification links:** Planned interface cases cover NOT_READY, UNKNOWN_SENSOR, immutable history, and version 1/2 field handling.

<!-- document-kit:content:trace:start -->
The contract traces to Example::Controller and planned unknown-sensor, empty-history, valid-reading, and optional-quality compatibility checks.
<!-- document-kit:content:trace:end -->
