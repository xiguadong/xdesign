<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=interface template=operation-contract contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster getCurrentReading operation contract

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="I100"></a>

## 1. Purpose and participants

**Boundary:** The operation is the public query boundary; validation, buffering, and controller orchestration remain internal or external respectively.

**Participants:** Controller invokes the operation and Aster export adapter executes it read-only.

<!-- document-kit:content:purpose:start -->
getCurrentReading returns the newest valid value for one known sensor without exposing history mutation or acting as a generic data schema.
<!-- document-kit:content:purpose:end -->

<a id="I101"></a>

## 2. Operation contract

**Operations:** getCurrentReading(sensorId) returns Reading, NOT_READY, UNKNOWN_SENSOR, or TIMEOUT and has no retry or mutation side effect.

**Data shapes:** Request is {sensorId: string}; Reading is {sensorId, utcTimestamp, celsius, quality?} with finite numeric celsius.

<!-- document-kit:content:operation:start -->
The request contains only sensorId; success returns the current Reading and does not accept time ranges, writes, or arbitrary buffer indices.
<!-- document-kit:content:operation:end -->

<a id="I102"></a>

## 3. Errors and invariants

**Error behavior:** UNKNOWN_SENSOR, NOT_READY, and TIMEOUT are terminal outcomes; none creates, updates, or backdates a Reading.

**Invariants:** At most one terminal outcome per request, no history mutation, and success always returns the newest valid value at read time.

<!-- document-kit:content:errors:start -->
Unknown identifiers return UNKNOWN_SENSOR before buffer access, empty history returns NOT_READY, and timeout returns no value or fallback timestamp.
<!-- document-kit:content:errors:end -->

<a id="I103"></a>

## 4. Versioning and compatibility

**Version policy:** Major changes alter required fields or outcomes; optional response fields and additive documentation are minor compatible changes.

**Compatibility policy:** Old callers may ignore quality and new callers must accept its absence; UNKNOWN_SENSOR and NOT_READY never change meaning.

<!-- document-kit:content:compatibility:start -->
Version 1 returns three required fields; version 2 may add optional quality while preserving request shape, operation name, and all error meanings.
<!-- document-kit:content:compatibility:end -->

<a id="I104"></a>

## 5. Model and verification trace

**Model elements:** Example::InteractionFlow defines the fictional request/validation/response order exercised by the operation.

**Verification links:** Success, unknown, empty, timeout, no-mutation, and version 1/2 response checks are planned verification cases.

<!-- document-kit:content:trace:start -->
The operation traces to Example::InteractionFlow and planned success, unknown, empty, timeout, and compatibility checks.
<!-- document-kit:content:trace:end -->
