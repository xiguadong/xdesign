# Current status

This page is the single maintained summary of current capability, baseline, and open-work
status. Historical outcomes and their evidence stay in `records/wishes/wish-history.md` and
the plan/review/execution records; this page links but does not rewrite them.

<a id="a12"></a>

## Status, baselines, and evidence

The requirements, allocated design, and delivered product are distinct engineering
baselines. A document's presence does not freeze one. [D30](../interfaces/documentation-contract.md#d30)
defines the review and change record expected before claiming a baseline. The current wish
status and open implementation items are maintained in [V02](wishes/wish-history.md#v02)
and [V04](wishes/wish-history.md#v04); wish-0008's own outcome must be read from
[V07](wishes/wish-history.md#v07) and `mbse/wishes/wish-0008.sysml`.

| Baseline or artifact | Current state | Review or evidence |
| --- | --- | --- |
| Functional requirements | Working; no SRR freeze | [R13](../requirements/requirements.md#r13), [V09](../verification/review-criteria.md#v09) |
| Allocated architecture and module design | Working; no PDR or CDR freeze | [B28](reviews/architecture-baseline.md#b28), [D40](../design/flows/simple-flow/implementation.md#d40) |
| Product | Four-stage SimpleFlow slice and two-stage wish-verify slice executed; complete workflow application and VModel gate commands incomplete; no FRR | [P08](executions/simple-flow.md#p08), [P28](plans/wish-verify.md#p28), [V08](../verification/review-criteria.md#v08) |
| SysML model and rendered figures | Revision-4 parse/check, render, audit, and figure verification passed for their stated scopes | [V07](wishes/wish-history.md#v07) |

Run the pinned compiler and inspect its diagnostics as described in [T02](../architecture/constraints.md#t02).
The renderer writes model figures in each document directory's `assets/` and the site builder writes its
output under `cache/docs-site/` ([T03](../architecture/constraints.md#t03)). Review evidence for
each wish stays under `cache/evidence/<wish>/`, with the assessed artifact revision recorded
in that wish's model. Wish-0007 established the nine-figure presentation review
([V06](wishes/wish-history.md#v06)); wish-0008 reran final-asset checks after its
revision-2 stage-document and figure relocation changes, then reran them for revision-3
directory overviews and path migration, then for revision-4 local asset placement
([V07](wishes/wish-history.md#v07)). Later figure changes need
their own corresponding evidence.

<a id="a13"></a>

## Open documentation and design work

| Open item | Why it remains open | Responsible role |
| --- | --- | --- |
| Complete per-requirement method, phase, and acceptance criteria | An SRR cannot freeze an incomplete verification plan ([R11](../verification/requirements.md#r11)) | Requirements owner, unassigned |
| Choose and justify architecture and interface baselines | PDR lacks concept comparison and the draft ICD has open contracts ([B28](reviews/architecture-baseline.md#b28), [I09](../interfaces/contracts.md#i09)) | Architect and integrator, unassigned |
| Integrate the workflow application and VModel commands | The four-stage SimpleFlow run and commands execute, but generic controller integration, durable persistence, and three VModel gate commands remain open ([D40](../design/flows/simple-flow/implementation.md#d40), [P12](reviews/simple-flow-fixes.md#p12)) | Module owner, unassigned |
| Hold SRR, PDR, CDR, and FRR with revision-bound evidence | Stage documents and individual wish assessments do not constitute those gate decisions ([V14](../verification/acceptance.md#v14)) | Review authorities, unassigned |

<a id="b32"></a>

## Current execution boundary

| Scope | Current state |
| --- | --- |
| Default VModel | Eleven stages, their gates, and the verification correspondences are modeled. The three fixed VModel gate commands are declared but not implemented; a complete default-flow run has not executed. |
| Four-stage `designedSimpleFlow` | The requirements → design → implement → verify slice, including four fixed checks and a terminal semantic judgment boundary, executes locally ([P08](executions/simple-flow.md#p08)). It is a selected flow within xdesign, not proof that this eleven-stage route runs. |
| Wish records | Each wish trace records which flow it actually selected and the evidence for its own assessments. A modeled gate or declared verification case is not a passing result ([V02](wishes/wish-history.md#v02)). |

This is a dependency and authorization sequence, not a dated project schedule.
Dates, effort, owners, and deadlines would require a separate project plan. For
the current development and acceptance work on xdesign itself, use the
[project management overview](index.md#p01) and the
[wish status](wishes/wish-history.md#v02).

<a id="r03"></a>

## Enforcement status

Three checkers are declared in `VModel`, none of which declares rules or has an
implemented command. The separate `designedSimpleFlow` declares four checkers and
18 rules in [D06](../design/flows/simple-flow/definition.md#d06). Their commands
under `tools/checks/` now run against a revision-bound JSON record and file
snapshot; [D07](../design/flows/simple-flow/definition.md#d07) defines their
exchange. Research prototypes under `cache/research/checks/` remain historical
work and are not this implementation.

The stand-alone `tools/check_documentation.py --json` command audits documentation
targets, embeds, and figure-block fields ([D31](../interfaces/documentation-contract.md#d31)).
It is project code, but is not a `Checker` on `VModel` or `designedSimpleFlow`.
A clean result cannot decide whether
the explanatory text and selected model content agree.

| Checker | Command | Establishes | Rules | Implemented |
| --- | --- | --- | --- | --- |
| `requirementTraceCheck` | intended `tools/checks/requirement_trace.py` (not project code) | trace conformance, identifier conformance | none | no |
| `implementationArtifactCheck` | intended `tools/checks/implementation_artifacts.py` (not project code) | artifact presence, definition coverage | none | no |
| `deliveryRecordCheck` | intended `tools/checks/delivery_record.py` (not project code) | artifact presence, trace conformance | none | no |

The four implemented SimpleFlow checkers are listed with their exact rule sets in
[D06](../design/flows/simple-flow/definition.md#d06). A successful example run
retains one report per syntactic gate under `cache/implementation/demo/reports/`.

Consequences, stated rather than papered over:

- `VModel`'s three syntactic checkpoints that name checkers (`requirementsAccepted`,
  `implementationReady`, `deliveryAccepted`, see
  [F03](../design/flow.md#f03)) cannot pass until the commands exist;
  `gateSyntaxIsRunnable` ([R01](../requirements/requirements.md#r01)) stands unmet for `VModel`.
- The designed SimpleFlow's four syntactic checks now have declared rules and
  commands. Their success establishes those rules for the checked snapshots,
  subject to the explicit semantic verification judgment.
- The pinned SysML compiler remains a separate fixed check. Its clean result does
  not decide the engineering meaning of the model or the 26 unevaluated constraints.
- The two project/model boundary requirements without satisfiers in [R01](../requirements/requirements.md#r01) remain
  maintained by hand; the new record checker covers the designed SimpleFlow slice
  only.
