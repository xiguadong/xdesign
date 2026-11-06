# Interface control overview

> Project: xdesign | Status: draft interface contracts | Review position: no frozen
> allocated or interface baseline.

<a id="i10"></a>

## Scope, reading order, and control position

This directory owns the interface inventory and the contracts between modeled
providers and consumers. The [interface control document](contracts.md#i01)
assigns IF-001 through IF-004 to the modeled exchanges, lists their available typed
fields, records change and trace rules, and identifies open operational contract fields.
Start with its [inventory](contracts.md#i01), read the applicable interface
definition, then use its [traceability](contracts.md#i07) and
[open-issue list](contracts.md#i09) when planning integration.

The architecture's [system boundary](../architecture/system-composition.md#b02) and
[design](../architecture/system-context.md#b05) explain where these interfaces fit. The
[requirements specification](../requirements/requirements.md#r09) states the interface
obligations, while the [review checklist](../verification/review-criteria.md#v10) identifies
what must be decided before an interface baseline can be accepted.

The SysML model establishes parties, ports, exchange direction, and selected typed
items. Transport, complete payload schemas, timing, retries, errors, security authority,
and compatibility policy are still open. The four entries are therefore draft
contracts; the present documents do not authorize an implementation or integration
baseline.

The separate [store format](store-format.md#i11) specifies the local persistence interface:
format and schema identities, strict save requests, revision manifests, operation receipts,
store commands, exit codes, and checkout/recovery behavior. It is a local file/SQLite
contract, not a network interface or a frozen migration baseline.

The separate [operator web API](web-api.md#i16) specifies the local loopback HTTP contract
of the embedded operator page: routes, envelopes, status codes, field tables, the
create/work/check actions, and the [checkpoint library endpoints](web-api.md#i19). Its command line is in the
[stage CLI](../design/stage-cli.md#d90) and its program design in the
[operator web design](../design/web-operator.md#d91). It is a local operator surface, not
one of the four modeled exchanges above and not the documentation site's interface.

The [portable document kit interaction contract](document-kit-api.md#i20)
describes the original CLI/MCP arguments, results, path and update guards. Its
[structured release contract](document-kit-api.md#i21) designs the later Python,
CLI and MCP additions; that release is not yet implemented or accepted.
