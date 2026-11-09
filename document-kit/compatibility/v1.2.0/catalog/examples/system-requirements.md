<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=requirements template=system-requirements contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
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

**Measurable conditions:** Sixty valid readings arrive in one hour, the newest ten survive a 20-minute outage, and an authorized controller retrieves the expected latest value.

**Verification method:** Demonstrate with fake time, a scripted probe, an outage switch, and an authorized test controller; this example does not execute the demonstrations.

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
