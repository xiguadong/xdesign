<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=requirements template=generic-requirements contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
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

**Measurable conditions:** Ten valid samples arrive at 60±0.1-second intervals, invalid input is rejected, and the last valid reading remains available throughout a two-minute outage.

**Verification method:** Demonstrate sampling and outage behavior with a fake clock, scripted probe inputs, and an export recorder.

<!-- document-kit:content:acceptance:start -->
Acceptance uses a fictional ten-minute run with a controllable clock and one injected invalid sample; these are planned criteria, not an executed test result.
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
