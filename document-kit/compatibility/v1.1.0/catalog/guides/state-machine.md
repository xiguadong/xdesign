# State machine authoring guide

Select this template to specify meaningful lifecycle states, events, guards, transitions, terminal behavior, and recovery. Do not select it for stateless behavior or a one-way linear procedure; use interaction-flow for ordered interactions or direct prose for a trivial calculation.

## Required content

Provide state definitions and invariants, events and guards, transitions and terminal states, recovery behavior and constraints, and source/verification projection. Dependencies are lifecycle requirements, model events, and design responsibilities; operation and verification documents consume but do not redefine the transition rules.

Worked field: `guards = "tick is accepted only in Sampling with a working probe; reset is accepted only in Fault after a successful self-test."` This prevents invalid transitions.

## Structure

Use purpose, states/invariants, events/guards, transitions/lifecycle, recovery/constraints, and projection in order. Name illegal or ignored events and specify whether history is preserved or cleared on every transition.

## Depth and omission

Right-size to states that change behavior; do not invent states for every variable value. Omit stateless procedures and record why a state machine adds no useful distinction.

## Views and alternatives

A state view is conditional on multiple meaningful states and event-driven transitions. A state/transition table can be clearer for a small lifecycle and requires an explicit choice and rationale.

## Sources and review

Source IDs: ARC42, GITHUB-DOCS. Review guard completeness, illegal-event handling, terminal/recovery behavior, history invariants, and whether a table or figure faithfully shows the same transitions.
