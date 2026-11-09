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
