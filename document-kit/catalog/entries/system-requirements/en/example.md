<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=requirements template=system-requirements language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# Aster remote monitoring capability requirements

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="R100"></a>

## 1. Purpose and scope

**System boundary:** The capability includes local sensing, buffering, and remote reading; irrigation control and long-term analytics are separate systems.

<!-- document-kit:content:purpose:start -->
Aster introduces a new remote monitoring capability for a greenhouse that previously used manual readings. The goal is continuous visibility, not a narrow change to an existing interface.
<!-- document-kit:content:purpose:end -->

<a id="R101"></a>

## 2. Requirements

**Normative statements:** Provide 60-second sampling, UTC timestamps, ten-reading outage history, latest-reading queries, and explicit invalid-data rejection.

**Rationale:** Operators need trustworthy current conditions and a small outage history without turning the hub into a control or analytics platform.

<!-- document-kit:content:requirements:start -->
The capability shall sample once per minute, retain the latest ten valid readings during connectivity loss, and expose current and recent values to an authorized controller.
<!-- document-kit:content:requirements:end -->

<a id="R102"></a>

## 3. Acceptance conditions

**Measurable conditions:** From empty history and fake UTC T0 = 2026-01-01T00:00:00Z, inject value 20.0+(n-1)/10 at T0+60n seconds, n=1..60. Require all 59 adjacent intervals to be 60±0.1 seconds, all timestamps to equal T0+60n in UTC, and 60 accepted samples in the hour. Disconnect after tick 58 and reconnect just after tick 60 while collection continues; authorized latest/history queries after reconnection must return latest 25.9 at 2026-01-01T01:00:00Z and ordered samples n=51..60 (25.0..25.9) with original timestamps. In an independent reset case inject NaN at T0+60; reject it and return NOT_READY with empty history.

**Verification method:** Drive the declared schedule with a fake UTC clock and scripted probe; record each accepted tick and compare exact timestamp, cadence and total count. Inspect the retained history and authorized latest/history responses after reconnection, then separately test invalid rejection from a reset state. All criteria remain unexecuted.

<!-- document-kit:content:acceptance:start -->
Capability acceptance combines a scheduled sampling demonstration, an outage demonstration, and an authorized-query demonstration with measurable outputs.
<!-- document-kit:content:acceptance:end -->

<a id="R103"></a>

## 4. Model and design trace

**Model elements:** Example::Sensor, Example::Controller, and Example::WeatherService identify the new capability's participants.

**Design links:** Aster system-context and reading-pipeline designs own realization; the requirements retain only their observable goals and constraints.

<!-- document-kit:content:trace:start -->
The new capability decomposes into context, composition, interaction, data, operation, and verification artifacts rather than duplicating those details here.
<!-- document-kit:content:trace:end -->

<a id="R104"></a>

## 5. Status and limits

**Unresolved items:** Controller enrollment, credential management, long-term storage, and actuator safety are outside this worked capability example.

<!-- document-kit:content:limits:start -->
Enrollment, authorization mechanics, multi-year retention, and actuator safety requirements need separate analysis before a real capability is approved.
<!-- document-kit:content:limits:end -->
