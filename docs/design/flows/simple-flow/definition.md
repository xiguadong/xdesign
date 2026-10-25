# SimpleFlow definition



The declared SimpleFlow definition: the designed flow, its five exit gates, the 18 fixed rules, the four checker command contracts, and the checkpoint evaluation class. The shared flow vocabulary is in design/flow.md; the shared checker boundary is in design/adapters.md.



<a id="d01"></a>

## 1. The designed simple flow

![What the designed flow is made of](assets/designed-flow.svg)

> **Figure D01-1 — Designed SimpleFlow plan.**  
> **Question:** Which stages and exit gates make up the designed flow?  
> **Model view:** `XDesignDetailedDesign::designedFlowTree`, a browser view of `DesignedSimpleFlow`.  
> **Companions:** [D05](#d05) for gate criteria and [F03](../../flow.md#f03) for selectable flows.  
> **Status:** renderable-now; design view with a tested first executable slice.  
> **Reading:** Four stages retain the SimpleFlow order and the usage adds gates to their exits. The table below distinguishes the four declared fixed checks from the final semantic judgment.  
> **Limits:** The figure does not show implemented checker commands or a run that passed these gates.

`DesignedSimpleFlow :> SimpleFlow` keeps the four stages in the definition's order and adds
an exit gate to every one of them. Four gates are syntactic — facts the implemented
fixed commands check; the one judgment stays where the definition put it, at the end:

| Stage | Exit gate | Kind | Established by |
| --- | --- | --- | --- |
| `requirements` | `simpleRequirementsAccepted` | syntactic | `simpleRequirementsCheck`, 2 declared rules in [D06](#d06) |
| `design` | `simpleDesignAccepted` | syntactic | `simpleDesignCheck`, 7 declared rules in [D06](#d06) |
| `implement` | `simpleImplementationAccepted` | syntactic | `simpleImplementationCheck`, 2 declared rules in [D06](#d06) |
| `verify` | `simpleRecordAccepted` | syntactic | `simpleRecordCheck`, 7 declared rules in [D06](#d06) |
| `verify` | `verificationAccepted` | semantic | a judge, agent or human — inherited and restated ([D05](#d05)) |

Each syntactic gate states its criteria and names a checker. The four checkers, their
command paths, and 18 rules are declared in the model and explained in
[D06](#d06). A declared path alone cannot authorize a visit:
the command must run and return a valid, retained, revision-bound report
([D07](#d07)).

Why the base definition is not the place for this: adding gates to `SimpleFlow` would
apply them to `xdesign.workflowFlow` and to every run already recorded against it. A completed visit whose
stage gained a new exit checkpoint would lose its authorization retrospectively — the
record would be rewritten by a design change. The design instead states the gated
specialization and instantiates it as `xdesign.designedSimpleFlow`; the same portfolio
instantiates `NestedFlowExample` for the nested case
([F05](../../lifecycle.md#f05)).

<a id="d05"></a>

## 2. The gates, exactly

Each gate states the criteria a judge or script must establish. The four syntactic
gates each name one checker, command URI, and declared rule set. D06 specifies what
those commands must decide; command existence and passing execution are checked
separately. The semantic gate names its permitted judges and has no checker.

| Gate | Kind | Criteria (verbatim from the model) | Check at design stage | Establishes |
| --- | --- | --- | --- | --- |
| `simpleRequirementsAccepted` | syntactic | "Every requirement declared in the project model carries its rationale, and every satisfy link resolves to a requirement that is declared." | `simpleRequirementsCheck`, 2 rules in [D06](#d06) | identifier conformance, trace conformance |
| `simpleDesignAccepted` | syntactic | "The design declares the run class, the factory, the functions that evaluate gates and advance the run, and the checkpoint class with its syntactic and semantic adaptations and its evaluate operation; every exit checkpoint of this flow is an instance of that class; every stage of this flow declares its exit checkpoint; every design element names its documentation section; and all nine SysML diagram views are declared, drawn and carried by their document." | `simpleDesignCheck`, 7 rules in [D06](#d06) | artifact presence, definition coverage |
| `simpleImplementationAccepted` | syntactic | "The stage's current visit records at least one artifact, and every recorded artifact exists at its URI." | `simpleImplementationCheck`, 2 rules in [D06](#d06) | artifact presence |
| `simpleRecordAccepted` | syntactic | "The run records at least one visit per stage; every completed visit has a latest passed assessment of each exit gate at its artifact revision; and evidence of every passed assessment on current or completed visits is retained." | `simpleRecordCheck`, 7 rules in [D06](#d06) | trace conformance, artifact presence |
| `verificationAccepted` | semantic | "Verification evidence demonstrates that the implementation satisfies the requirements." and "The record and the evidence stand on their own: a reader can re-examine the judgment from them without re-asking." | — (no checker) | judged by an agent or a human (`agentMayJudge`, `humanMayJudge`) |

Every gate is an instance of the checkpoint class: the four syntactic gates are
`SimpleFlowSyntacticCheckpoint` instances and the semantic gate is a
`SimpleFlowSemanticCheckpoint` instance ([D11](#d11)). The class adapts both rule kinds
for the run, so the flow evaluates a gate through the class that declares how it is
decided, not through a case analysis beside it.

The semantic gate is inherited from the flow definition and restated with both
criteria; the four syntactic gates are the design's. Each syntactic gate names one
checker and sets no judge flag; the semantic gate names no checker and permits an
agent or human ([F02](../../adapters.md#f02)). The compiler resolves
these declarations but does not evaluate the model's assertions
([T04](../../../architecture/constraints.md#t04)). The checker commands and their reports must establish
the scripted outcomes before a run may use them.

<a id="d06"></a>

## 3. The rules documented for the syntactic gates

The designed flow declares four checkers and 18 `CheckerRule` usages. Each rule has an ID,
condition, and input set in the model; the tables below expand their diagnostics and
implementation contract. A command may apply but must not redefine a rule. The gate's
outcome is the conjunction of its rules' verdicts, with exact rule-set equality between
declaration and report. This implements the declaration side of
`checkerRulesAreDeclared` ([R01](../../../requirements/requirements.md#r01)); report agreement
still requires execution ([D07](#d07)).

For every rule, `Reads` is the input set and `Condition` is the pass condition. The
implementer evaluates the condition against the checked revision, emits one record with
`rule_id`, `verdict`, `inputs`, `observed`, and `failures`, and sets `verdict = passed` only
when the condition holds. The report names the checker ID, stage, assessed artifact
revision, all declared rule IDs once each, the per-rule results, and a retained report
URI. A missing input or unparseable snapshot means no trustworthy report and records
`Unevaluated`; a rule condition that evaluates false records `Failed`. A report with
missing, extra, duplicate, stale or contradictory rule results is rejected before
creating an assessment. [D31](../../../interfaces/documentation-contract.md#d31) defines the project
documentation rule contract; the specific expected diagnostics and observed evidence for
these implemented SimpleFlow rules follow the conditions below.

The invocation checks plan identity, the current visit, and recorded artifact
bytes before reporting rules. If that preflight fails, it returns no trusted report:
a wrong plan hash, an absent current visit, or a missing recorded file is
`Unevaluated`. These conditions overlap `flow-is-supported`, `visit-exists`, and
artifact existence. Their rule rows still describe the required conditions. An
empty current implementation artifact set is an evaluated `Failed` result, while
an unreadable file named in that set is an unevaluated input failure. Missing
documentation figures, checker command targets, or retained evidence discovered
after preflight can produce evaluated `Failed` rules with diagnostics below.

| Checker | Rules |
| --- | --- |
| `simpleRequirementsCheck` | 2 |
| `simpleDesignCheck` | 7 |
| `simpleImplementationCheck` | 2 |
| `simpleRecordCheck` | 7 |

### 3.1 `simpleRequirementsCheck` — 2 rules

| Rule id | Pass condition | Reads |
| --- | --- | --- |
| `requirements-carry-rationale` | `XDesignRequirements` declares at least one requirement, and every requirement declaration inside it carries a nonempty rationale attribute. | `mbse/model/requirements.sysml` |
| `satisfy-links-resolve` | The declared traceability source has at least one satisfy link, and every satisfy link names a requirement declared inside `XDesignRequirements`. | `mbse/model/requirements.sysml`, `mbse/model/traceability.sysml` |

### 3.2 `simpleDesignCheck` — 7 rules

| Rule id | Pass condition | Reads |
| --- | --- | --- |
| `design-declarations-present` | The run, factory, gate, advance, designed flow definition and `xdesign`-owned usage, checkpoint class, both adaptations, and checkpoint evaluation action are declared. | `mbse/model/detailedDesign.sysml`, `mbse/model/composition.sysml` |
| `every-stage-gated` | Every stage names nonempty exit gates declared by this flow and typed by `SimpleFlowCheckpoint`; each syntactic exit binds its designated one of the four fixed SimpleFlow checkers. | `mbse/model/detailedDesign.sysml` |
| `design-elements-carry-doc-sections` | Every top-level detailed design element has a `DocSection` that resolves to a document and section. | `mbse/model/detailedDesign.sysml`, `docs/` |
| `checker-commands-exist` | Every declared SimpleFlow checker command URI names an existing repository file. | `mbse/model/detailedDesign.sysml`, `tools/checks/` |
| `checker-rules-are-declared` | All four fixed SimpleFlow checkers are declared, each with at least one rule with a nonempty ID, condition, and reads value. | `mbse/model/detailedDesign.sysml` |
| `rule-sets-agree-with-commands` | Each of the four fixed checkers' declared rule IDs exactly equal its command's rule ID list. | `mbse/model/detailedDesign.sysml`, `tools/checks/` |
| `structure-graphs-are-carried` | Each of the nine primary SimpleFlow views names a distinct model figure, and every required figure exists and is embedded in the document named by its `DocFigure` annotation. Supporting run and interface views are included. | `mbse/model/detailedDesign.sysml`, `docs/` |

The nine views and their source-model elements are inventoried in
[D12](views.md#d12). This rule requires each named view to be declared and its
figure to be generated and carried by the document; three generic overview figures do not
meet the nine-view coverage requirement. Their engineering meaning is reviewed separately
through the planned `SimpleFlowViewReview` case ([D15](../../../verification/requirements.md#d15)).

### 3.3 `simpleImplementationCheck` — 2 rules

| Rule id | Pass condition | Reads |
| --- | --- | --- |
| `visit-exists` | The checked implementation stage has a current visit in the run record. | runtime run record JSON |
| `artifacts-of-current-visit-exist` | The current implementation visit records at least one artifact URI and every recorded artifact exists at that URI. | runtime run record JSON, recorded artifact URIs |

### 3.4 `simpleRecordCheck` — 7 rules

| Rule id | Pass condition | Reads |
| --- | --- | --- |
| `flow-is-supported` | The run record has `plan_id = designedSimpleFlow` and `plan_sha256` equal to the SHA-256 of the retained frozen plan serialization. | runtime run record JSON, retained plan serialization |
| `every-stage-visited` | Every planned stage has at least one visit and every visit names a stage in that plan. | runtime run record JSON |
| `completed-visits-authorized` | For every completed visit, the latest assessment of each required exit gate passed at that visit's artifact revision and snapshot digest, and every recorded artifact still matches its file digest. | runtime run record JSON, recorded artifact URIs |
| `assessments-name-exit-gates` | Every passed assessment names an exit gate of its visit's stage. | runtime run record JSON |
| `passed-assessments-carry-evidence` | Every passed assessment names at least one evidence URI. | runtime run record JSON |
| `evidence-exists` | Every evidence URI of every passed assessment on current or completed visits resolves to retained evidence. | runtime run record JSON, recorded evidence URIs |
| `designed-runs-are-typed-by-the-design-class` | A run on the designed SimpleFlow records its execution as a `FlowRun` of plan `designedSimpleFlow`. | runtime run record JSON |

The rule table above is the decision source. This output table makes each rule
implementation-followable: `observed` must include the named facts, and `failures`
contains one entry for each offending identifier or URI. For a rule that finds no
offenders, `failures` is empty. Every row is a model-declared condition implemented by its fixed command. Passing execution requires a retained command report for the assessed revision.

| Rule id | Failure diagnostic when the condition is false | Required `observed` evidence | Status |
| --- | --- | --- | --- |
| `requirements-carry-rationale` | No requirements are declared, or requirement `<id>` has no rationale attribute. | Inspected requirement IDs and IDs missing rationale. | implemented |
| `satisfy-links-resolve` | No satisfy links are declared, or satisfy link `<id>` targets an undeclared requirement or a requirement outside `XDesignRequirements`. | Inspected satisfy links, their qualified targets, and resolved project requirement IDs. | implemented |
| `design-declarations-present` | Required design declaration `<name>` is missing. | Presence result for each named run, factory, function, flow, checkpoint class, adaptation, and action. | implemented |
| `every-stage-gated` | Stage `<name>` has no exit checkpoint; checkpoint `<name>` is not a declared `SimpleFlowCheckpoint` instance or its syntactic exit lacks the designated checker binding. | Stage-to-gate map, each gate's declaration and type, and syntactic gate-to-checker bindings. | implemented |
| `design-elements-carry-doc-sections` | Design element `<name>` has no resolvable `DocSection` document and anchor. | Every inspected top-level element, declared target, and path/anchor resolution result. | implemented |
| `checker-commands-exist` | Checker `<name>` names missing command `<path>`. | Checker IDs, command URIs, and existence results. | implemented |
| `checker-rules-are-declared` | A fixed checker `<name>` is missing, has no rule, or rule `<id>` lacks an ID, condition, or reads value. | Four required checker IDs, checker-to-rule map, and completeness result for each required field. | implemented |
| `rule-sets-agree-with-commands` | A fixed checker `<name>` is missing, declares a rule ID absent from its command list, or the command lists an undeclared ID. | Four required checker IDs and declared/command rule ID sets, with their symmetric difference. | implemented |
| `structure-graphs-are-carried` | `DocFigure` asset `<path>` is missing or is not embedded in its declared document; named view `<name>` lacks its figure or reuses another primary view's asset. | View-to-asset and asset-to-document map, distinctness, existence and embed results for the supporting overviews and nine question views. | implemented |
| `visit-exists` | Checked stage `<name>` has no visit. | Checked stage ID and matching visit IDs. | implemented |
| `artifacts-of-current-visit-exist` | Current implementation visit `<id>` records no artifact, or cites missing artifact `<URI>`. | Visit ID, nonempty recorded artifact URI list, and existence results. | implemented |
| `flow-is-supported` | Run record names another flow plan, omits its hash, or its hash differs from the retained frozen plan serialization. | `plan_id`, `plan_sha256`, and independently computed plan hash. | implemented |
| `every-stage-visited` | Flow stage `<name>` lacks a visit, or visit `<id>` names a stage outside the assigned flow. | Assigned stage IDs and all visit-to-stage links. | implemented |
| `completed-visits-authorized` | Completed visit `<id>` has a missing, failed, unevaluated, or stale latest assessment of exit gate `<name>`, or artifact `<URI>` no longer matches its recorded file digest. | Each completed visit's revision and snapshot digest, required gate IDs, latest assessment per gate in append order, and independently computed file digests. | implemented |
| `assessments-name-exit-gates` | Passed assessment `<id>` names a gate outside visit `<id>`'s stage exits. | Passed assessment-to-gate links and the stage's exit gate IDs. | implemented |
| `passed-assessments-carry-evidence` | Passed assessment `<id>` has no evidence URI. | Passed assessment IDs and their evidence URI lists. | implemented |
| `evidence-exists` | Passed assessment `<id>` on current or completed visit `<id>` cites missing evidence `<URI>`. | Those visits' passed assessments, evidence URIs, and retrieval results. | implemented |
| `designed-runs-are-typed-by-the-design-class` | Run on `designedSimpleFlow` lacks `FlowRun` typing. | Assigned flow and resolved execution type; mark other SimpleFlow usages not applicable. | implemented |

<a id="d07"></a>

## 4. Checker command contract

The designed flow declares four checkers, their command paths, and all 18 rules in
`mbse/model/detailedDesign.sysml`. The command at each path must apply the declared
conditions. The executable slice reads both a SysML wish source URI (`wishModel`)
and an executable JSON run-record snapshot URI (`recordUri`). The source is a legacy
independent trace or an imported store material; the record carries visit and assessment
data that the implementation can check. A separate adapter is needed before the commands
can audit all historical `mbse/wishes/*.sysml` records or store-internal claim mappings.
The commands require an existing SysML wish file as a control precondition, but the four
D06 rule sets do not validate that file's internal trace relationships. The wish path
identifies the run; it is not itself a passing per-wish trace check.

The invocation also carries `checkerId`, `planId`, `planSha256`, `stage`,
`artifactRevision`, and `snapshotDigest`. The last value is the SHA-256 digest of the
canonical artifact set: URI and file-digest pairs for the current visit. It is distinct
from the JSON record URI and from the frozen plan hash. A fixed checker verifies those
files against the requested digest before it can pass. Its report returns the assessed
revision and artifact-set digest, exact rule IDs once each, per-rule verdicts and
diagnostics, overall verdict, and retained report URI. The caller rejects a stale or
malformed report; one successful command process is not itself a passing gate.

For the requirements and design commands, every repository file actually read to
decide a rule must also be listed in the current visit's artifact set with its exact
file digest. This includes the model sources, documentation sources and figures, and
checker commands that a rule inspects. Otherwise the command cannot establish that
its answer belongs to the visited snapshot and returns an unevaluated command error.
The frozen plan, executable run-record input, and newly written report are separate
exchange artifacts: including the run record or its own report in the visit digest
would make the check circular. The raw report lists its read-file digests to
make the decision auditable; the visit digest remains the authorization anchor
for these repository inputs.

The implemented report adapter requires hashes for the wish, frozen plan, and run
record bytes and for every input named by a passed rule. It checks that each
passed rule's inputs cover its declared file or runtime reads. The record checker
also rejects malformed visit, assessment, and lifecycle fields before applying
its seven rules. These checks bind the report to the bytes it evaluated; command
and harness version binding remains a separate delivery obligation.

| Planned checker | Designed command contract |
| --- | --- |
| `simpleRequirementsCheck` | `tools/checks/simple_requirements.py`; reads `mbse/model/requirements.sysml` and `mbse/model/traceability.sysml`; reports two rules. |
| `simpleDesignCheck` | `tools/checks/simple_design.py`; reads `mbse/model/detailedDesign.sysml`, `mbse/model/composition.sysml`, command files, documentation sections and figures; reports seven rules. |
| `simpleImplementationCheck` | `tools/checks/simple_implementation.py`; reads the JSON run record's current implementation visit and its artifact URIs; reports two rules. |
| `simpleRecordCheck` | `tools/checks/simple_record.py`; reads the JSON run record, its frozen plan identity and retained evidence URIs; reports seven rules. |

Research prototypes under `cache/research/checks/` do not count as the project
implementation. Executed evidence must name the repository command, source revision,
input snapshot and retained report. The pinned SysML compiler (`parse` and `check`)
remains a separate mechanical check with the limits in [T04](../../../architecture/constraints.md#t04).

### 4.1 What is not established

- **A declared command path is not an executed result.** Each gate now names its checker;
  run conformance requires the command file, an exact report, and a recorded assessment.
- **`VModel`'s three checkers do not exist and declare no rules.** The paths in
  `architecture.sysml` (`requirement_trace.py`, `implementation_artifacts.py`,
  `delivery_record.py`) are intended locations, not project code.
- **The designed flow is not the default.** `xdesign.'flow'` is the abstract common
  origin, `xdesign.selectedFlow` points to `xdesign.vModel`, and a wish can select
  `xdesign.designedSimpleFlow` at registration
  ([F03](../../flow.md#f03)).
- **The documentation audit and design gate have different scope.** The stand-alone
  audit checks `DocSection` and `DocFigure` paths, anchors, assets, and embeds
  ([T02](../../../architecture/constraints.md#t02)); the seven-rule `simpleDesignCheck` must also check
  declarations, command files, and rule-set agreement. Neither judges the engineering
  content of a figure.
- **Constraint bodies are not evaluated** ([T04](../../../architecture/constraints.md#t04)): the run class's
  assertion and the gates' kind/judge split are verified by reading, and the reading is
  stated in [D05](#d05).
- **`wishProductsAreRetained`, `modelStandsWithoutWishes` and
  `runRecordIsScriptChecked` remain unsupported by the tool**
  ([R01](../../../requirements/requirements.md#r01)); nothing in this design changed that.

<a id="d11"></a>

## 5. The checkpoint class and its evaluation loop

![The checkpoint class's evaluate operation](assets/checkpoint-evaluation.svg)

> **Figure D11-1 — Checkpoint evaluation loop.**  
> **Question:** How does one checkpoint class handle fixed checks and semantic judgment?  
> **Model view:** `XDesignDetailedDesign::checkpointEvaluationFlow`, an action-flow view of `EvaluateSimpleFlowCheckpoint`.  
> **Companions:** [D06](#d06) for rule conditions and [D10](views.md#d10) for checker and judge interfaces.  
> **Status:** renderable-now; design view with a tested first executable slice.  
> **Reading:** The common evaluate operation chooses the syntactic or semantic adaptation by checkpoint kind. Both paths produce an assessment linked to the gate and the current visit.  
> **Limits:** The diagram does not prove the declared command files exist, execute, or satisfy their rule conditions.

Every gate of the designed flow is an instance of one class, `SimpleFlowCheckpoint`: the
flow's checkpoints are not five unrelated `Checkpoint` usages, and the run does not judge
them beside their class. The class carries the common checkpoint contract — source stage,
kind, criteria, checkers, judge flags — and owns the `evaluate` operation and the
assessments that operation produces. Two adaptations put the two rule kinds on that one
class:

| Class | Kind | Adapts | Operation |
| --- | --- | --- | --- |
| `SimpleFlowCheckpoint` | both | the common checkpoint contract and the assessment loop | `EvaluateSimpleFlowCheckpoint` |
| `SimpleFlowSyntacticCheckpoint` | syntactic | the declared checker, command URI, and rules in [D06](#d06) | `RunSyntacticCheck` |
| `SimpleFlowSemanticCheckpoint` | semantic | the semantic rules: criteria a permitted judge reads, with no checker | `TakeSemanticJudgment` |

The loop closes on the class:

1. `designedSimpleFlow.checkpoints` is typed `SimpleFlowCheckpoint`; each gate is an
   instance of one adaptation.
2. `SimpleFlowRun.checkpoints` holds those instances, and its `evaluateGate` hands each
   gate to the class's `evaluate` operation ([D04](implementation.md#d04)).
3. The syntactic adaptation's operation runs its declared checker command and applies
   the declared rules; the semantic adaptation's operation presents the
   criteria to a permitted judge. Each produces a `CheckpointAssessment`
   ([E04](../../evidence-and-revisions.md#e04)).
4. The assessment's `checkpoint` points back to the class instance it judged, and the class
   keeps the assessments it produced — the design and the record point at each other.

What the design deliberately does not encode is the *logic of a rule*. A syntactic rule's
condition is declared in the model and explained in [D06](#d06); its command applies it
([D07](#d07)). A semantic rule's criterion is prose for the judge. Where a
condition does not reduce well to one SysML expression, the condition is written in the
documentation, not moved into code beside the model: the commands apply the rules, they do
not define them. The class fixes who evaluates a checkpoint and what is recorded; the
documentation fixes what each rule means.

<a id="d88"></a>

## 6. Configurable check bindings

The four syntactic checkpoints keep their declared criteria and rules, but the *executor*
that establishes each of them can be selected per wish when the wish is registered. A
binding chooses one of three modes for one checkpoint:

| Mode | Executed by | Recorded authority | Recorded assessor | Deterministic |
| --- | --- | --- | --- | --- |
| `default` | the flow's built-in checker command | `script` | the checker id, for example `simpleRequirementsCheck` | yes |
| `cli` | an agent CLI chosen from `codex` and `kimi`, with a named model | `agent` | `<cli>:<model>`, for example `codex:gpt-6-astra` | no — an agent judgment |
| `command` | a project-relative checker command that implements the same invocation and report interface as the built-in command | `script` | the checker id | as the supplied command |

The selectable binding is data, not a second checker definition: the plan still names the
same four checkpoints, each with the same declared rules. The binding decides which
program answers those rules and therefore which authority the assessment records. The
custom runner is described by `RsCheckerSpec.custom` in `src/core/plan.rs`; the flow's
built-in declarations are in `src/simple_flow/blueprint.rs`; execution and report
validation are in `src/simple_flow/check.rs`; and registration accepts the binding in
`src/stage_cli/mod.rs`.

### 6.1 The binding file and its validation

The bindings for one registration are declared in a strict JSON file passed as
`xdesign init WISH_ID --flow simple-flow --checks FILE`. The [stage CLI](../../stage-cli.md#d89)
owns the option, the exact schema and its validation errors. In summary, the file names
`schema_version = 1` and a `checks` array; each entry names one of the four syntactic
checkpoint ids and its mode. An omitted checkpoint is `default`. Duplicate or unknown
checkpoint ids, unknown modes, a `cli` mode without `codex`/`kimi` or without a non-empty
model, a `command` mode without a normalized project-relative `.py` `command_uri` that
exists in the repository, and unknown fields are rejected before any run exists. A
non-default binding is rejected for `wish-verify`, which declares only semantic
checkpoints ([D57](../wish-verify/definition.md#d57)); a default-only file for that flow
is a no-op. Bindings are not accepted on a later `next` or `check`; changing one requires
a new run.

The same choices can be made from the operator page. Its create-wish action sends the
bindings to the same `init` command, and the plan detail shows the binding of each
checkpoint in the `mode`/`cli`/`model`/`command_uri` form owned by the
[web operator API](../../../interfaces/web-api.md#i17). The page can select `default` or
`cli` only; a `command` binding is configured through the CLI option.

### 6.2 Plan freezing and reconstruction

A default-only registration is unchanged: with no non-default binding, the plan is the
built-in plan, no custom runner is serialized, and the frozen plan bytes and hash are the
same as before this feature. `flow-is-supported` and the existing runs therefore keep
their recorded plan identity.

A registration with any non-default binding freezes the binding into the plan before the
registration is published. The frozen plan carries, per checkpoint, the runner
(`cli` or `process`), the CLI and model or the command URI, and its `plan_sha256` binds
those bytes like every other plan field. `next`, `check`, the record checker, restore and
the web page reconstruct the binding from the stored frozen plan; they do not re-read the
`--checks` file or a workspace configuration. Editing the file after registration has no
effect on that run, and this slice exposes no rebind action.

### 6.3 Execution, authority, and retained evidence

For each syntactic checkpoint in the current visit, the bound executor runs and must
answer the checkpoint's declared rule ids. The `default` and `command` modes use the
deterministic command contract in [D07](#d07); a `cli` binding instead invokes the agent
CLI non-interactively and read-only. For `codex` the invocation is
`codex exec --skip-git-repo-check --sandbox read-only -m MODEL PROMPT`; for `kimi` it is
`kimi -m MODEL -p PROMPT`. The prompt presents the
checkpoint criteria, the declared rules, and the reviewed artifact snapshot, and requires
one JSON verdict object; programs can be replaced for development and tests with
`XDESIGN_AGENT_CODEX_BIN` and `XDESIGN_AGENT_KIMI_BIN`.

The runtime validates the answer before it creates an assessment: the report must name
the checkpoint's exact rule set, the assessed artifact revision, and the snapshot digest
it was given. The assessment then records `judged_by = agent` and the assessor
`<cli>:<model>`, so a custom run is never presented as a deterministic default check.
The retained report is the assessment's evidence, and the run retains the command's
`.stdout.log` and `.stderr.log` beside it in the report directory; for store-backed runs
those bytes are published as materials. A malformed answer, a failed agent run, or a
timeout produces no passed assessment; the gate stays unevaluated and blocks the visit
([E04](../../evidence-and-revisions.md#e04)).

The invocation is bounded by the same checker deadline as a process checker:
`XDESIGN_CHECKER_TIMEOUT_SECS` defaults to 600 seconds, the whole process tree is killed
when the deadline expires, and an explicit `CheckerTimeout` is recorded rather than a
pass. The read-only flag and prompt are the agent-run contract; they do not turn an agent
judgment into a fixed check.

### 6.4 What this does not change

- The four gates, their criteria and their 18 rules are unchanged; a binding changes the
  executor, not what must be established. Rule-set equality, report shape, revision and
  snapshot binding, and evidence retention apply to every mode.
- A `default`-only run behaves exactly as before. Deterministic authorization claims hold
  only for `default` and for a command that is itself deterministic; a `cli` assessment is
  an agent judgment.
- The semantic `verificationAccepted` gate is not bindable: it remains a supplied agent or
  human judgment through the result file ([F02](../../adapters.md#f02)).
- `wish-verify` declares no syntactic checkpoint and keeps its two semantic judgments;
  non-default bindings are outside this slice.

### 6.5 The wish-verify boundary

`DesignedWishVerifyFlow` has no checker, so it has nothing to bind: both of its gates are
semantic and are judged in the stage interaction. The existing
`judgmentMatchesCheckpointKind` invariant still holds for `wish-verify` runs and for
every default-only SimpleFlow run. The configurable binding is the one recorded extension:
when a syntactic checkpoint's frozen plan names a `cli` runner, its assessment records
agent authority with the executor in the assessor field, and restore keeps that record
instead of rejecting it as a script-only assessment. The binding is part of the plan, so
the authorization rule remains reconstructable from the stored record.

The verification cases for this contract are [CUSTOM-01 to CUSTOM-06](../../../verification/design.md#d95);
they are verification design, not executed acceptance.
