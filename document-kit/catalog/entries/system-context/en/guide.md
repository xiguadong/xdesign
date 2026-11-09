# System context authoring guide

Select this template to establish the system boundary, external actors, external systems, and exchanged information. Do not select it for internal part decomposition alone or runtime node allocation without external context; use component-composition or a deployment-specific derived profile.

## Required content

Provide boundary ownership, external actors and their goals, information flows and interface participants, lifecycle under external failures, constraints and rationale, and model/verification trace. Dependencies are capability scope, external model participants, and exchange definitions; internal component and operation documents are downstream links, not content to copy.

Worked field: `information_flows = "Validated readings leave AsterHub for Operator, calibration commands enter from Operator, and forecasts arrive separately from WeatherService."` This identifies direction, owner, and meaning.

## Structure

Use boundary, external interfaces, interactions/lifecycle, constraints, and trace in order. State explicitly what is outside the system and which external losses do not stop internal behavior.

## Depth and omission

Right-size to boundary and exchanges, then link internal parts and runtime details. Omit internal decomposition and record component-composition as its owner; a context table needs an explicit reason when replacing the expected context view.

## Views and alternatives

A context view is expected because inside/outside placement and exchange direction are visual concerns. Use figure, table, or prose according to the declared decision; a table substitute requires a rationale, not silent omission.

## Sources and review

Source IDs: DPR, ARC42, C4. Review actor completeness, boundary correctness, flow direction, and whether the figure actually shows the claimed boundary and exchanges. C4 notation is optional.
