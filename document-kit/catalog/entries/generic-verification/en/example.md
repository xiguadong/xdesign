<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=verification template=generic-verification language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
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

**Inputs:** Use exactly the clock epoch, values, NaN, connection events and query times in the case table; no preload or unlisted collection occurs.

**Procedure:** Reset for each row, apply events in time order, record validation, local history and remote responses, and compare every value, timestamp, ordering and error with that row.

**Expected results:** All outputs must equal the case-table expectations, including exact UTC instants, FIFO range, NOT_READY and outage suppression.

<!-- document-kit:content:cases:start -->
Each planned case independently starts with an empty buffer, a connected exporter and fake UTC clock T0 = 2026-01-01T00:00:00Z. The table allocates every stimulus and query; it reports expectations only.

| Case | Scheduled events | Planned expected output |
| --- | --- | --- |
| Cadence | At T0+60n seconds inject 20.0+(n-1)/10, n=1..10; query after each tick. | Query n returns that value and timestamp T0+60n; adjacent accepted intervals are 60±0.1 seconds. |
| Invalid | At T0+60 inject NaN, then query at T0+61. | Reject NaN, keep buffer empty, return NOT_READY. |
| FIFO | At T0+60n inject 20.0+(n-1)/10, n=1..11; inspect/query at T0+661. | Retain n=2..11 (20.1..21.0) in order with timestamps T0+120..660; latest is 21.0 at 2026-01-01T00:11:00Z. |
| Empty | Query at T0 before any injection. | NOT_READY; empty history remains empty. |
| Outage | Accept 20.0 at T0+60; immediately disconnect export until T0+180; supply no more samples; query remote at T0+90, local at T0+150 and remote after reconnection at T0+181. | Suppress remote export during outage; local and recovered queries return 20.0 at 2026-01-01T00:01:00Z; no fabricated newer sample. |
<!-- document-kit:content:cases:end -->

<a id="V103"></a>

## 4. Acceptance criteria and trace

**Pass criteria:** Every case-table observation must match; only cadence has the stated ±0.1-second tolerance. No invalid insertion, reordered history, synthesized timestamp or outage export is permitted.

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
