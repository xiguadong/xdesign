<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=verification template=verification-test-plan language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# Aster outage and cadence test plan

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="V100"></a>

## 1. Scope and basis

**Requirements links:** The cadence and latest-valid requirements define observable acceptance for these planned cases.

**Design links:** Collector, RingBuffer, Exporter, and fake-clock interfaces identify the units under test.

<!-- document-kit:content:scope:start -->
The plan tests 60-second cadence and latest-valid behavior during a controller outage; it does not record an executed result or substitute review for execution.
<!-- document-kit:content:scope:end -->

<a id="V101"></a>

## 2. Methods and environment

**Methods:** Use a scheduled demonstration, FIFO boundary analysis, and exact output comparison rather than inspection alone.

**Environment:** Use fake UTC time, an in-memory buffer, a scripted valid probe, and a controllable controller-connection flag.

<!-- document-kit:content:methods:start -->
Deterministic demonstration drives twelve samples and a two-minute outage while recording every accepted value and exported response.
<!-- document-kit:content:methods:end -->

<a id="V102"></a>

## 3. Verification cases

**Inputs:** Use exactly the initial state, twelve tick/value pairs, tick-10 disconnect, post-tick-12 reconnect and queries in the case table.

**Procedure:** Apply the declared event ordering, record every tick and buffer state, issue both outage queries and the recovery query, then compare original timestamps and suppression with the table.

**Expected results:** Twelve timestamps are 60 seconds apart, the buffer holds 20.2–21.1, and final export returns 21.1 with its exact UTC timestamp.

<!-- document-kit:content:cases:start -->
This planned case starts at fake UTC T0 = 2026-01-01T00:00:00Z with empty buffer and connected controller. At T0+60n seconds inject 20.0+(n-1)/10 Celsius, n=1..12. Process tick 10 first, then disconnect immediately at T0+600 seconds. Continue ticks 11 and 12 while disconnected; reconnect at T0+720 seconds only after tick 12.

| Observation | Planned oracle |
| --- | --- |
| Query at T0+601, during outage | No remote Reading response; buffer retains 20.0..20.9 with original UTC timestamps. |
| Query at T0+661, after tick 11 | No remote Reading response; local buffer retains 20.1..21.0. |
| Inspect after tick 12, before reconnect | Buffer retains 20.2..21.1, timestamps T0+180..720 in 60-second steps. |
| Remote query T0+721, after reconnect | Return 21.1 and original timestamp 2026-01-01T00:12:00Z; history remains unchanged. |
<!-- document-kit:content:cases:end -->

<a id="V103"></a>

## 4. Acceptance criteria and trace

**Pass criteria:** No missed tick, ten retained values are exactly 20.2–21.1, outage exports are suppressed, and recovery returns only 21.1.

**Trace matrix:** Cadence maps to tick records, FIFO to retained history, latest-valid to recovery response, and read-only export to unchanged timestamps.

<!-- document-kit:content:criteria:start -->
Pass requires exact cadence, exact retained values and order, no export during outage, and exact latest-valid response after recovery; all remain planned expectations.
<!-- document-kit:content:criteria:end -->

<a id="V104"></a>

## 5. Status and limits

**Unexecuted cases:** Every case is unexecuted in this fictional document; no transcript or observed outcome is asserted.

**Residual risks:** Deterministic simulation cannot demonstrate embedded timing, hardware faults, transport security, or production load behavior.

<!-- document-kit:content:limits:start -->
The plan does not measure real scheduler jitter, physical sensor accuracy, network behavior, or multi-controller concurrency.
<!-- document-kit:content:limits:end -->
