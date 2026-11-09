# Catalog routing guide

Choose one generic default for a conventional class document or a scenario for a specific engineering question. Do not infer scenario coverage from a class count.

| Concern | Template |
| --- | --- |
| New system or major capability | system-requirements |
| Existing behavior change | change-requirements |
| External boundary and actors | system-context |
| Internal parts and connections | component-composition |
| Runtime ordering and interactions | interaction-flow |
| Lifecycle states and transitions | state-machine |
| Entities, relationships and invariants | data-model |
| Operation request/response/errors | operation-contract |
| Data shape and compatibility | data-contract |
| Executable verification plan | verification-test-plan |
| Semantic design review | design-review |

A generic class template remains available for every class. A project may use different templates in the same class. Record reasoned omissions in a coverage list rather than assuming every scenario is required. A diagram is chosen only when it answers the engineering question; a table or reasoned prose alternative can be sufficient.

## Structured API and flow routes

| Class | Generic template ID | Route | `name` means | `layer` |
| --- | --- | --- | --- | --- |
| `api-design` | `generic-api-design` | `docs/design/api/{layer}/{name}.md` | API basename; `index` reserved | required, selected from the layer set |
| `flow-index` | `generic-flow-index` | `docs/design/flows/{name}/index.md` | flow basename | absent |
| `flow-definition` | `generic-flow-definition` | `docs/design/flows/{name}/definition.md` | flow basename | absent |
| `flow-implementation` | `generic-flow-implementation` | `docs/design/flows/{name}/implementation.md` | flow basename | absent |
| `flow-views` | `generic-flow-views` | `docs/design/flows/{name}/views.md` | flow basename | absent |


Use the API page for an operation or type contract, including inputs, outputs, errors, side effects and trace. Use the flow index for navigation, definition for stages and gates, implementation for allocation and source projection, and views for applicability and references. Read API.md (`API.md` in the distribution) for complete inputs. Original flat selections and defaults remain available.
