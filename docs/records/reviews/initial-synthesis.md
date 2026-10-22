# Design review synthesis

> Review basis: 19 October 2026 working tree. The local source reports are
> `cache/kimi_review/00_synthesis.md` with seven domain reports, and
> `cache/cc_review/design_review.md` with `validation_report.md`. The `cache/`
> reports are local evidence and are not version controlled. This page retains the
> decisions that affect the implementation.

<a id="p02"></a>

## Joint judgment and dispositions

Both reviews find that the model vocabulary and independent wish records are a usable
foundation, but a compiler pass does not establish the workflow's behavior. The
current design contains authorization, transition, and integration conflicts. The
first executable slice can be built after those contracts are made consistent;
completion of SimpleFlow alone does not establish the full V-model product goal.

| Finding from the two reviews | Decision for the SimpleFlow slice | Acceptance evidence |
| --- | --- | --- |
| A later failed or unevaluated gate assessment can be ignored by the Rust design, although the model selects the latest per gate. | The latest assessment of each exit gate at the current artifact revision is authoritative. A later failure or refusal revokes an earlier pass. | Pass-then-fail, fail-then-pass, missing, duplicate, and stale assessment cases. |
| An artifact can change at the same URI without changing the designed revision. | Recording a content change is an explicit revision event. Bind a check report and assessment to that revision and preserve earlier entries. | In-place edit simulation invalidates the earlier pass. |
| The detailed action reads every flow gate and its exclusive branches can skip one of the two verify-stage gates. | Select the current visit's stage exits and evaluate each checkpoint once, in plan order, by its kind. | Verify-stage mixed syntactic and semantic gates both produce assessments. |
| Generic and detailed/Rust designs disagree on a failed visit: remain blocked or immediately open rework. | A failed or unevaluated active visit remains current and blocked. Corrective activity and a fresh assessment occur in that visit. A new visit is opened only to revisit a completed stage. | Block, correct, reevaluate, and explicit reopen histories. |
| Run construction and plan validation omit required wiring or allow vacuous authorization. | Validate a connected, gated, single-terminal plan, every referenced gate, judge authority, checker rules, and command report. Supply checker and judge adapters and wish-model identity at construction. Start an untouched visit at revision zero; use a visit index local to each stage. | Malformed-plan and malformed-report cases reject before advancement; construction has no implicit dependencies. |
| The checker interface cannot identify the selected checker or wish model; judgment authority is recorded but not enforced. | Dispatch with the checker specification, wish model, stage, and revision. Reject a report with missing, extra, duplicate, or contradictory rule verdicts. Match semantic reviewer authority to the gate flags. | Fake and real adapter tests plus invalid-authority cases. |
| Kimi found model-to-document links that resolve but point at irrelevant prose, stale requirement counts, and a nondeterministic diagram renderer. | Correct the semantic targets and inventories, attach Rust elements to their specific API pages, sort renderer inputs, then regenerate affected figures. | Documentation locator audit, figure verification, site build, and inspection of the API trace. |
| Local historical evidence is often only in ignored `cache/`, and a live flow definition does not freeze the plan used for a decision. | Record exact artifact identities and a plan snapshot or digest for new runs. Treat unrecoverable historical evidence as a visible limitation rather than a current pass. | Exported run record contains revision-bound evidence and plan identity; missing evidence fails the record check. |

Additional review gaps remain outside this first slice: the default eleven-stage V-model
has no executable gate commands; upstream changes do not yet invalidate downstream
stages across a delivered workflow; wish instances are not demonstrably bound into
the generic workflow-owned collections; and early structural gates alone do not judge
engineering adequacy. The [implementation plan](../plans/simple-flow.md#p03)
states the narrower acceptance claim. Project SRR, PDR, CDR, and FRR decisions remain
unrecorded.

The pre-change mechanical baseline was a passing documentation locator audit and a
SysML parse/check with 26 unevaluated constraints. Those results prove only the checks
performed, and their counts must be regenerated after the design changes.
