<!-- document-kit: profile=xdesign-sysml-docs@1.2.0 class=verification template=verification-test-plan contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
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

**Inputs:** Twelve values 20.0–21.1 Celsius at 60-second ticks, outage after value 20.9, and a final query after value 21.1.

**Procedure:** Record cadence and buffer after each tick, verify controller failure does not stop collection, then query after the twelfth sample.

**Expected results:** Twelve timestamps are 60 seconds apart, the buffer holds 20.2–21.1, and final export returns 21.1 with its exact UTC timestamp.

<!-- document-kit:content:cases:start -->
The main case collects twelve valid values, drops the controller after the tenth, and checks collection and export before, during, and after the outage.
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
