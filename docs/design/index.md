# Detailed design

This class organizes the design along two dimensions: cross-flow common design (flow model, lifecycle, evidence, adapters) and one flow's specific design (definition, implementation, views). The class-and-function API design remains a separate level under design/api.

<a id="d42"></a>

## Scope and reading path

The project SysML model is the structural source. These pages explain the modeled behavior, the evidence and revision semantics, the checker/judge boundaries, the flow-specific declarations and realization, and the limits of the implemented slices. They do not replace the model, the source, or the execution records.

## Design pages and ownership

| Page | Scope | Owns |
| --- | --- | --- |
| [flow.md](flow.md) | common | flow model, catalog, stages/checkpoints/gates, ownership table |
| [lifecycle.md](lifecycle.md) | common | two-command stage interaction and generic run lifecycle, advancement, reopening, blocking/rework |
| [evidence-and-revisions.md](evidence-and-revisions.md) | common | revisions, assessments, artifact identity, retained evidence, invariants |
| [adapters.md](adapters.md) | common | checker-command and judge boundaries, reports, authority |
| [web-operator.md](web-operator.md) | operator surface | embedded web program: position, request flow, view and executor wording, start-work handoff, embedded-asset release model |
| [checkpoint-library.md](checkpoint-library.md) | authoring source | reusable syntactic and semantic checkpoint definitions, versions, scripts and scaffolds, plan binding and freezing |
| [storage.md](storage.md) | operational persistence | store authority, revisions, materials, atomic save, queries, checkout and recovery |
| [flows/simple-flow/](flows/simple-flow/index.md) | flow-specific | four-stage definition, implementation, views |
| [flows/wish-verify/](flows/wish-verify/index.md) | flow-specific | two-stage definition, implementation, views |
| [api/](api/index.md) | class/function | one design document per API |

## Where new behavior goes

| Change | Owning page |
| --- | --- |
| A stage, checkpoint, gate, or flow definition | [flow.md](flow.md) for the common vocabulary; the flow's definition.md for declared values |
| Run creation, visits, advancement, reopening, blocked correction | [lifecycle.md](lifecycle.md) for the common behavior; the flow's implementation.md for its run/factory |
| The agent's `next` → workspace work → `check` interaction | [lifecycle.md](lifecycle.md#f04); work content in [flow.md](flow.md#f01) |
| Revision, assessment, artifact, or evidence semantics | [evidence-and-revisions.md](evidence-and-revisions.md); record fields in [interfaces/record-format.md](../interfaces/record-format.md) |
| Store schema, save/recovery, queries, material integrity or checkout | [storage.md](storage.md); [store format](../interfaces/store-format.md); storage API pages |
| Checker command, judge authority, reports, refusal/error boundary | [adapters.md](adapters.md); a flow's definition.md for its commands and rules |
| Per-wish check binding (default, agent CLI, custom command, library checkpoint) | the flow's definition.md and [stage-cli.md](stage-cli.md#d89) |
| A reusable checkpoint definition, its script, or its version history | [checkpoint-library.md](checkpoint-library.md); [D88](flows/simple-flow/definition.md#d88) for the binding |
| Real-project check ownership, definition/binding/execution history, or cross-project reuse | [As-built project boundary](checkpoint-library.md#d107), [proposed sharing and federation](checkpoint-library.md#d108), [remediation flow](checkpoint-library.md#d109) |
| The operator web page, its local HTTP actions, or an embedded asset | [web-operator.md](web-operator.md); HTTP contract in [interfaces/web-api.md](../interfaces/web-api.md#i16) |
| A flow's module class, factory, adapter, Rust realization, or limits | that flow's implementation.md |
| A view, view applicability, figure coverage, or figure ownership | that flow's views.md; presentation contract in [interfaces/documentation-contract.md](../interfaces/documentation-contract.md) |
| Class-and-function contract | [design/api/](api/index.md) (one page per API) |

## Reading paths

| Reader | Start here | Then |
| --- | --- | --- |
| Understand the design model | [flow.md](flow.md) | [lifecycle.md](lifecycle.md), [evidence-and-revisions.md](evidence-and-revisions.md), [adapters.md](adapters.md) |
| Design or change SimpleFlow | [flows/simple-flow/index.md](flows/simple-flow/index.md) | definition.md, implementation.md, views.md |
| Design or change WishVerifyFlow | [flows/wish-verify/index.md](flows/wish-verify/index.md) | definition.md, implementation.md, views.md |
| Use or change an API | [design/api/index.md](api/index.md) | the per-API page and the owning flow/API design section |

The [stage CLI](stage-cli.md#d85) defines the common work/check implementation for the two local flow slices.
