# Interface control overview

> Project: xdesign | Status: draft interface contracts | Review position: no frozen
> allocated or interface baseline.

<a id="i10"></a>

## I10 Scope, reading order, and control position

This directory owns the interface inventory and the contracts between modeled
providers and consumers. The [interface control document](01_interfaces.md#i01)
assigns IF-001 through IF-004 to the modeled exchanges, lists their available typed
fields, records change and trace rules, and identifies open operational contract fields.
Start with its [inventory](01_interfaces.md#i01), read the applicable interface
definition, then use its [traceability](01_interfaces.md#i07) and
[open-issue list](01_interfaces.md#i09) when planning integration.

The architecture's [system boundary](../arch/01_system_composition.md#b02) and
[design](../arch/02_design.md#b05) explain where these interfaces fit. The
[requirements specification](../requirements/01_baseline.md#r09) states the interface
obligations, while the [review checklist](../review/02_checklist.md#v10) identifies
what must be decided before an interface baseline can be accepted.

The SysML model establishes parties, ports, exchange direction, and selected typed
items. Transport, complete payload schemas, timing, retries, errors, security authority,
and compatibility policy are still open. The four entries are therefore draft
contracts; the present documents do not authorize an implementation or integration
baseline.
