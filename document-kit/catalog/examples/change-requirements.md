<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=requirements template=change-requirements contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster sampling interval change requirements

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="R100"></a>

## 1. Change summary

**Current behavior:** Normal operation samples at 60-second intervals and exports the same Reading shape to all controllers.

**Proposed behavior:** An active heat alert samples at 30-second intervals; alert end restores 60 seconds without changing Reading fields or query semantics.

<!-- document-kit:content:change:start -->
The existing Aster hub samples every 60 seconds. The proposed change samples every 30 seconds during a heat alert while preserving the normal cadence and existing reader contract.
<!-- document-kit:content:change:end -->

<a id="R101"></a>

## 2. Affected requirements

**Impacted items:** R-SAMPLE-60, scheduler load, heat-alert state, and the cadence verification case require revision; interface and schema requirements do not.

**Rationale:** Faster heat-response decisions need more samples, while limiting the delta to cadence avoids destabilizing existing consumers.

<!-- document-kit:content:impact:start -->
The change affects cadence requirement R-SAMPLE-60 and scheduler capacity, but not validation, buffering, data shape, or latest-reading errors.
<!-- document-kit:content:impact:end -->

<a id="R102"></a>

## 3. Compatibility and migration

**Compatibility constraints:** Keep Reading fields, query operation, UTC format, and latest-valid errors unchanged; the heat-alert rate must be optional and disabled by default.

**Migration notes:** Deploy with alert mode off, enable it per hub through reviewed configuration, and retain 60-second behavior for unconfigured hubs.

<!-- document-kit:content:compatibility:start -->
Existing controllers see only more frequent values and require no migration. The default remains 60 seconds until an authorized alert is active.
<!-- document-kit:content:compatibility:end -->

<a id="R103"></a>

## 4. Acceptance conditions

**Measurable conditions:** Alert mode yields 30±0.1-second samples, exit restores 60±0.1 seconds, and all responses retain the original required fields and errors.

**Verification method:** Demonstrate alert entry/exit with fake time and run the unchanged version 1 controller contract checks.

<!-- document-kit:content:acceptance:start -->
Acceptance compares the old and new cadence in one controlled session and checks that an old controller can parse every response without modification.
<!-- document-kit:content:acceptance:end -->

<a id="R104"></a>

## 5. Trace and limits

**Model elements:** Example::Sensor and the fictional heat-alert state own the changed cadence behavior; Reading and query contracts are unchanged traces.

**Unresolved items:** Battery impact, alert authorization, and the acceptable sustained alert duration require further review.

<!-- document-kit:content:trace:start -->
The delta traces to cadence and alert-state elements only; unresolved power consumption and alert authorization remain explicit rather than assumed solved.
<!-- document-kit:content:trace:end -->
