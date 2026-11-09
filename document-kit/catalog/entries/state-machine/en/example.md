<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=design template=state-machine language=en contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# Aster sampling lifecycle state machine

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="D100"></a>

## 1. Purpose and allocation

**Design subject:** AsterHub lifecycle states, events, guards, transitions, terminal behavior, and fault recovery.

**Requirements links:** Continuous sampling, explicit invalid-data handling, and latest-valid retention requirements drive the lifecycle.

<!-- document-kit:content:purpose:start -->
The lifecycle model distinguishes stopped, ready, sampling, and fault states so event handling and recovery are explicit rather than implied by a linear procedure.
<!-- document-kit:content:purpose:end -->

<a id="D101"></a>

## 2. States and invariants

**States:** Stopped, Ready, Sampling, and Fault are the meaningful states; history remains readable in every state.

**Invariants:** Samples are created only in Sampling, Fault never clears history, and Stopped never emits a new timestamp.

**Diagram decisions:** [{"concern":"lifecycle-states","applies":true,"choice":"table","rationale":"Four states and their guards are clearer in a transition table than a diagram.","asset":null,"model_view":null}]

<!-- document-kit:content:states:start -->
Stopped has no collection, Ready waits for start, Sampling collects periodically, and Fault suspends acquisition while preserving history; no state view is needed for four states.
<!-- document-kit:content:states:end -->

<a id="D102"></a>

## 3. Events and guards

**Events:** initialize acquires lifecycle resources without clearing history; start enables collection; stop pauses; tick samples; probeFault faults; reset requests recovery; shutdown releases resources.

**Guards:** initialize requires Stopped and available resources; start requires Ready; tick requires Sampling and a working probe; probeFault requires Sampling and detected failure; reset requires Fault and successful self-test; shutdown requires Stopped.

<!-- document-kit:content:events:start -->
Events are initialize, start, stop, tick, probeFault, reset and shutdown. The transition table defines accepted source states and guards. Every unlisted pair or failed guard is rejected with no state, resource or history change; terminal shutdown rejects every event.
<!-- document-kit:content:events:end -->

<a id="D103"></a>

## 4. Transitions and lifecycle

**Transitions:** Stopped to Ready on initialize, Ready to Sampling on start, Sampling to Fault on probeFault, Fault to Sampling on reset, and any to Stopped on stop.

**Terminal states:** Shutdown from Stopped is terminal and releases resources without producing another reading or clearing retained history.

<!-- document-kit:content:transitions:start -->
The planned transition table is the complete event oracle. Initial state is Stopped with empty history and no acquired lifecycle resources. Existing retained history is preserved across reinitialization.

| Source | Event / guard | Target and action | History effect |
| --- | --- | --- | --- |
| Stopped | initialize / resources available | Ready; acquire resources | Preserve |
| Ready | start / true | Sampling; enable ticks | Preserve |
| Sampling | tick / probe working | Sampling; validate and append | Append valid value, FIFO if full; invalid leaves unchanged |
| Sampling | probeFault / failure detected | Fault; suspend ticks | Preserve |
| Fault | reset / self-test passes | Sampling; resume next fresh tick | Preserve |
| Any nonterminal | stop / true | Stopped; disable ticks | Preserve |
| Stopped | shutdown / true | Terminal; release resources | Preserve |
| Any | Unlisted event or failed guard | Reject; no state or resource change | Preserve |
| Terminal | Any event | Reject; remain terminal | Preserve |
<!-- document-kit:content:transitions:end -->

<a id="D104"></a>

## 5. Recovery and constraints

**Recovery behavior:** Run self-test before reset, remain in Fault on failure, and resume with the next fresh sample after success.

**Constraints:** No automatic history deletion, no fabricated recovery sample, and no tick processing while Fault is active.

<!-- document-kit:content:recovery:start -->
A failed self-test keeps Fault active and raises an explicit maintenance status; a successful reset resumes cadence with a fresh timestamp, never a backdated value.
<!-- document-kit:content:recovery:end -->

<a id="D105"></a>

## 6. Projection and verification

**Source projection:** LifecycleController and ProbeSelfTest are intended units; transitions are evaluated against an explicit event log.

**Verification links:** Illegal-tick, failed-reset, successful-reset, stop, shutdown, and history-retention cases are planned checks.

<!-- document-kit:content:projection:start -->
The projection is a transition-table test matrix covering illegal events, guard failure, recovery, terminal shutdown, and history preservation.
<!-- document-kit:content:projection:end -->
