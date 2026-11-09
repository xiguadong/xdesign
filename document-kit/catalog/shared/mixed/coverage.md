# Mixed bilingual fixture coverage

This fictional fixture instantiates 20 documents across all ten classes in English and Chinese. It verifies route and identity sharing, not semantic acceptance or executed engineering results.

| Entry | English | Chinese | Coverage rationale and omissions |
| --- | --- | --- | --- |
| generic-requirements | instantiated | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | General requirements contract |
| generic-architecture | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | General architecture contract |
| generic-design | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | General design contract |
| generic-interface | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | General interface contract |
| generic-verification | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | General verification contract |
| system-requirements | omitted: the other generic or scenario entry answers this mixed project's question | instantiated | Use for goals, scope, functional and quality requirements, not a narrow delta |
| change-requirements | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | Use for current/proposed behavior, impact and acceptance, not greenfield scope |
| system-context | instantiated | instantiated | Use for actors, external systems and exchanged information, not internal decomposition |
| component-composition | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | Use for responsibilities, parts, connections and refinement, not context alone |
| interaction-flow | instantiated | instantiated | Use for participants, ordering and alternate flows, not static data or a pure calculation |
| state-machine | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | Use for states, events, guards and recovery, not stateless or one-way procedures |
| data-model | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | Use for fields, ownership, persistence or exchanged meaning, not interaction sequencing |
| operation-contract | instantiated | instantiated | Use for a command, query, endpoint or tool operation, not schema alone |
| data-contract | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | Use for schema, validation, required fields and compatibility, not operation orchestration |
| verification-test-plan | instantiated | instantiated | Use for inputs, environment, procedure and pass criteria, not executed-result claims |
| design-review | omitted: the other generic or scenario entry answers this mixed project's question | omitted: isolated BDK-03 coverage owns this variant; duplicating it would not add a mixed-route question | Use for model meaning, design adequacy and source fidelity, not a premature pass |
| generic-api-design | instantiated | instantiated | generic |
| generic-flow-index | instantiated | instantiated | generic |
| generic-flow-definition | instantiated | instantiated | generic |
| generic-flow-implementation | instantiated | instantiated | generic |
| generic-flow-views | instantiated | instantiated | generic |

The Chinese system-context document chooses a table because three participants and two exchange directions are clearer in tabular form. The Chinese interaction-flow document chooses ordered prose because one main path and three terminal outcomes need no additional sequence figure. English uses the two shared SVGs for the corresponding visual concerns.

Every omitted entry still has an independent English and Chinese fixture. The mixed root therefore tests composition without weakening the all-42 isolated coverage boundary.
