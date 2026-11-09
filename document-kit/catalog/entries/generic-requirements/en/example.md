<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=requirements template=generic-requirements language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# Aster sensor monitoring requirements

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="R100"></a>

## 1. Purpose and scope

**System boundary:** The Aster hub samples its local probe and exports readings; greenhouse actuators and weather prediction are external.

<!-- document-kit:content:purpose:start -->
Aster is a fictional greenhouse sensor hub. The requirements cover periodic sampling and export of one temperature reading while leaving actuator control outside the product boundary.
<!-- document-kit:content:purpose:end -->

<a id="R101"></a>

## 2. Requirements

**Normative statements:** Sample at 60-second intervals, record UTC timestamps, reject non-finite values, and preserve the latest valid reading during export outages.

**Rationale:** Predictable cadence and validated timestamps let the external controller compare independent sensor hubs without accepting corrupt readings.

<!-- document-kit:content:requirements:start -->
The hub shall sample every 60 seconds, timestamp each sample in UTC, and export a reading only after validation. Timely delivery matters because irrigation decisions consume the data.
<!-- document-kit:content:requirements:end -->

<a id="R102"></a>

## 3. Acceptance conditions

**Measurable conditions:** Cadence case: inject 20.0+i/10 Celsius at T0+60(i+1) seconds for i=0..9; ten samples must have adjacent intervals 60±0.1 seconds and UTC timestamps exactly equal to those fake-clock instants. Invalid case: inject NaN at T0+60 seconds; reject it and retain empty history. Outage case: accept 20.0 at T0+60 seconds, disconnect export immediately afterwards until T0+180 seconds, and supply no further samples; local latest queries at T0+90 and T0+150 seconds must retain 20.0 and 2026-01-01T00:01:00Z.

**Verification method:** Use a fake UTC clock, scripted probe and recorder. Compare every timestamp with T0+60(i+1), compare adjacent cadence intervals, inspect rejection and buffer state in the independent NaN case, and compare the value and original timestamp in both local outage queries. These are planned comparisons, not executed evidence.

<!-- document-kit:content:acceptance:start -->
All criteria below are planned for this fictional example. Cadence, invalid-input and outage cases run independently from an empty buffer at T0 = 2026-01-01T00:00:00Z; no result has been observed.
<!-- document-kit:content:acceptance:end -->

<a id="R103"></a>

## 4. Model and design trace

**Model elements:** Example::Sensor and Example::Controller provide the fictional source elements for the sampling requirement.

**Design links:** The Aster reading-pipeline design owns validation and export behavior; no implementation result is asserted here.

<!-- document-kit:content:trace:start -->
Example::Sensor and Example::Controller identify the model participants for the sampling requirement and its intended sampling design.
<!-- document-kit:content:trace:end -->

<a id="R104"></a>

## 5. Status and limits

**Unresolved items:** Probe calibration limits, real scheduler jitter, authentication, and production retention remain for the actual project to decide.

<!-- document-kit:content:limits:start -->
Probe accuracy, hardware timing jitter, and security deployment settings are not specified by this small fictional requirements example.
<!-- document-kit:content:limits:end -->
