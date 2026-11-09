<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=verification template=generic-verification contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster sampling verification plan

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="V100"></a>

## 1. Scope and basis

**Requirements links:** The 60-second sampling, invalid-value rejection, UTC, and latest-valid requirements are the acceptance basis.

**Design links:** The reading-pipeline design supplies validator, ring buffer, and exporter responsibilities to test.

<!-- document-kit:content:scope:start -->
The plan evaluates sampling cadence, invalid-input rejection, and latest-valid behavior for the fictional Aster hub; no case is claimed executed.
<!-- document-kit:content:scope:end -->

<a id="V101"></a>

## 2. Methods and environment

**Methods:** Use demonstration with deterministic inputs, boundary-value analysis for cadence, and inspection of exported reading order.

**Environment:** Run in an isolated test process with fake UTC clock, in-memory buffer, and no physical sensor or network controller.

<!-- document-kit:content:methods:start -->
A fake clock drives timestamps, a scripted probe supplies finite and non-finite values, and a recording exporter captures controller-visible results.
<!-- document-kit:content:methods:end -->

<a id="V102"></a>

## 3. Verification cases

**Inputs:** Sixty-second clock ticks, readings 20.0 through 20.9 Celsius, one NaN, and one additional 21.0 sample.

**Procedure:** Drive each case from an empty buffer, record validation and exports, then compare timestamps, values, order, and error results.

**Expected results:** Cadence remains 60±0.1 seconds, NaN is rejected, FIFO retains the newest ten, empty returns NOT_READY, and outage preserves latest valid.

<!-- document-kit:content:cases:start -->
Cases cover ten scheduled samples, NaN input, the eleventh valid sample, empty export, and a two-minute controller outage.
<!-- document-kit:content:cases:end -->

<a id="V103"></a>

## 4. Acceptance criteria and trace

**Pass criteria:** All ten valid samples pass, invalid input never reaches the buffer, eviction is FIFO, and all exported values equal their scripted inputs.

**Trace matrix:** Sampling maps to cadence case, validation to NaN case, FIFO to eviction case, and latest-valid to empty/outage cases.

<!-- document-kit:content:criteria:start -->
Every planned case must match its exact value, timestamp, ordering, and error expectations with no unreviewed tolerance beyond the stated cadence bound.
<!-- document-kit:content:criteria:end -->

<a id="V104"></a>

## 5. Status and limits

**Unexecuted cases:** All cases are planned; no command, observation, or executed outcome is asserted by this example.

**Residual risks:** A deterministic fake-clock plan cannot establish embedded timing, hardware calibration, security, or production reliability.

<!-- document-kit:content:limits:start -->
Hardware jitter, concurrent access, physical probe accuracy, and real network behavior remain outside this fictional plan.
<!-- document-kit:content:limits:end -->
