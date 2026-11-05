<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=design template=state-machine contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
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

**Events:** start begins collection, stop pauses it, tick acquires, probeFault enters Fault, reset recovers, and shutdown ends the lifecycle.

**Guards:** start requires Ready, tick requires Sampling and working probe, probeFault requires a detected failure, and reset requires a successful self-test.

<!-- document-kit:content:events:start -->
start, stop, tick, probeFault, reset, and shutdown are the events; tick is accepted only in Sampling and reset only in Fault.
<!-- document-kit:content:events:end -->

<a id="D103"></a>

## 4. Transitions and lifecycle

**Transitions:** Stopped to Ready on initialize, Ready to Sampling on start, Sampling to Fault on probeFault, Fault to Sampling on reset, and any to Stopped on stop.

**Terminal states:** Shutdown from Stopped is terminal and releases resources without producing another reading or clearing retained history.

<!-- document-kit:content:transitions:start -->
Stopped→Ready→Sampling→Fault→Sampling covers startup, failure, and recovery; any nonterminal state may enter Stopped on stop.
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
