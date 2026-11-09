<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=interface template=data-contract contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster Reading data contract

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="I100"></a>

## 1. Purpose and participants

**Boundary:** The contract covers serialized Reading values at the Aster export boundary, not operation sequencing or storage technology.

**Participants:** Aster export adapter serializes values and external controllers validate and consume them.

<!-- document-kit:content:purpose:start -->
The data contract defines exchanged Reading values and their validation and version rules without specifying how a controller orchestrates requests.
<!-- document-kit:content:purpose:end -->

<a id="I101"></a>

## 2. Data contract

**Data shapes:** Object fields are sensorId:string, utcTimestamp:UTC ISO8601 string, celsius:number, and optional quality:valid|suspect.

**Required fields:** sensorId, utcTimestamp, and celsius are required; quality may be absent and null is not a substitute for any required field.

<!-- document-kit:content:schema:start -->
A Reading object requires sensorId, utcTimestamp, and celsius; quality is optional and never changes the meaning of celsius or timestamp.
<!-- document-kit:content:schema:end -->

<a id="I102"></a>

## 3. Validation and invariants

**Validation rules:** Require nonempty sensorId, UTC timestamp, finite celsius, valid optional quality, and rejection of non-finite numeric encodings.

**Invariants:** Required fields retain their meanings, timestamps are never synthesized, and schema version does not alter measurement provenance.

<!-- document-kit:content:validation:start -->
Consumers reject unknown sensorId types, non-UTC or missing timestamps, non-finite numbers, unexpected quality values, and mutable shared references.
<!-- document-kit:content:validation:end -->

<a id="I103"></a>

## 4. Versioning and compatibility

**Version policy:** Removing or retyping a required field is major; adding optional quality is minor; unknown optional fields may be ignored by consumers.

**Compatibility policy:** Version 1 consumers accept version 2 objects by ignoring quality, and version 2 consumers accept version 1 objects without quality.

<!-- document-kit:content:compatibility:start -->
Version 1 requires the three core fields; version 2 adds optional quality. Producers must not require new consumers to understand older version 1 values.
<!-- document-kit:content:compatibility:end -->

<a id="I104"></a>

## 5. Model and verification trace

**Model elements:** The data-model Reading is the fictional semantic source for this serialized contract.

**Verification links:** Schema validation, required-field, finite-number, optional-quality, and version 1/2 interoperability cases are planned checks.

<!-- document-kit:content:trace:start -->
The contract traces to the data-model Reading and planned valid, missing-field, non-finite, unknown-quality, and version compatibility cases.
<!-- document-kit:content:trace:end -->
