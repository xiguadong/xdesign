# Wish to delivery: the whole modeled route

> Status: default VModel design view, 20 October 2026. The route and its gates are
> modeled; the full route is not an executable application or an accepted project
> delivery. No calendar dates or durations have been assigned.

This page is the overview for a requester or developer asking how one wish can become
an implementation and a delivered outcome. It joins the stage sequence, exit gates,
verification correspondences, and wish evidence in one place. The
[lifecycle action](00_overview.md#a03) explains what the controller does within a
stage; the [package view](02_design.md#d21) explains model organization; the
[SimpleFlow run timeline](../details/simple_flow/03_structure_graphs.md#d08) explains
one four-stage run. The route below shows the default eleven-stage path they do not
show together.

The [xdesign composition](01_system_composition.md#b01) owns the selectable
flows under its abstract `'flow'` origin and selects `xdesign.vModel` by default.
The graph projects that usage's `VModel` definition; another selected flow has
its own stage and gate set ([F03](../details/01_flows_and_checkpoints.md#f03)).

<a id="b30"></a>

## B30 The route from wish to delivery

![Default VModel wish-to-delivery route](assets/03_wish_to_delivery.svg)

> **Figure B30-1 — Wish-to-delivery route.**  
> **Question:** Which stages and exit gates take a wish through definition, implementation, verification, validation, and delivery?  
> **Model view:** Native stage/checkpoint projection of `XDesignArchitecture::VModel`, rendered from the pinned compiler export by `tools/sysml_web/vmodel_overview.py`.  
> **Companions:** [F03](../details/01_flows_and_checkpoints.md#f03) for flow selection, [B31](#b31) for gate criteria and work products, and [E03](../details/02_records_and_evidence.md#e03) for current-revision authorization.  
> **Status:** renderable-now; modeled default route, not an executed end-to-end run.  
> **Reading:** Follow solid arrows down the left side to implementation, then up the right side to delivery. Every box names the stage's exit checkpoint and its kind. Dashed arrows point from a verification or validation stage to the development stage it checks; they do not mean work can skip the intervening stages.  
> **Limits:** This definition view has no elapsed time, assigned people, particular wish status, or executed result. It shows one linear default path and cannot establish that a gate passed.

The figure reads native `initialStage`, `nextStages`, `exitCheckpoints`,
`Checkpoint::sourceStage`, `Checkpoint::kind`, and `verifiesStage` references from
`VModel`. It fails to render if the export no longer has one connected route or a
verification pairing that this layout can show faithfully. The gate criteria remain
in the model and are summarized below. A wish selects a flow when received; its
independent trace records the actual route, visits, revisions, assessments, and
evidence ([A02](00_overview.md#a02), [E01](../details/02_records_and_evidence.md#e01)).

<a id="b31"></a>

## B31 What each gate establishes

The stage names and gate order below come from `VModel` in
`mbse/model/architecture.sysml`. The "work or evidence" column paraphrases each
checkpoint's modeled criterion so a reader can identify the expected result. A gate
authorizes leaving its stage only after a passing current-revision assessment
([B03](01_system_composition.md#b03)).

| Stage | Work or evidence expected at its exit | Exit gate and authority |
| --- | --- | --- |
| Wish intake | Recorded wish, scope, and intended outcome | `intakeAccepted` — semantic; permitted agent or human |
| Stakeholder needs | Agreed needs and acceptance criteria | `needsAccepted` — semantic; human |
| System requirements | Requirements and verification criteria traced to the wish | `requirementsAccepted` — syntactic; fixed checker |
| System architecture | Parts, relationships, and interfaces addressing requirements | `architectureAccepted` — semantic; permitted agent or human |
| Detailed design | Design supporting implementation and component verification | `designAccepted` — semantic; permitted agent or human |
| Implementation | Implementation artifacts and build evidence | `implementationReady` — syntactic; fixed checker |
| Component verification | Evidence against detailed design | `componentsVerified` — semantic; permitted agent or human |
| Integration verification | Evidence for architectural interfaces | `integrationVerified` — semantic; permitted agent or human |
| System verification | Evidence against system requirements | `systemVerified` — semantic; permitted agent or human |
| Acceptance validation | Evidence of the intended stakeholder outcome, judged against needs | `wishValidated` — semantic; human |
| Delivery | Delivery artifacts, traceability, and acceptance evidence recorded | `deliveryAccepted` — syntactic; fixed checker |

The four dashed correspondences are `componentVerification → detailedDesign`,
`integrationVerification → systemArchitecture`, `systemVerification →
systemRequirements`, and `acceptanceValidation → stakeholderNeeds`. Verification
checks the built system against prior definitions; acceptance validation judges
whether the intended outcome was achieved. The implementation stage sits between
definition and verification. The final delivery gate checks that the resulting
record exists; it does not replace the earlier engineering judgments.

<a id="b32"></a>

## B32 Current execution boundary

| Scope | Current state |
| --- | --- |
| Default VModel | Eleven stages, their gates, and the verification correspondences are modeled. The three fixed VModel gate commands are declared but not implemented; a complete default-flow run has not executed. |
| Four-stage `designedSimpleFlow` | The requirements → design → implement → verify slice, including four fixed checks and a terminal semantic judgment boundary, executes locally ([P08](../prj_manage/03_simple_flow_execution_record.md#p08)). It is a selected flow within xdesign, not proof that this eleven-stage route runs. |
| Wish records | Each wish trace records which flow it actually selected and the evidence for its own assessments. A modeled gate or declared verification case is not a passing result ([V02](../review/01_wish_status.md#v02)). |

This is a dependency and authorization sequence, not a dated project schedule.
Dates, effort, owners, and deadlines would require a separate project plan. For
the current development and acceptance work on xdesign itself, use the
[project management overview](../prj_manage/00_overview.md#p01) and the
[wish status](../review/01_wish_status.md#v02).
