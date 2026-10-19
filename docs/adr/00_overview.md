# Architecture decision overview

> Project: xdesign | Status: draft decision register | Audience: architects, model authors,
> implementers, and reviewers. This overview records accepted choices only when a
> decision record exists; it does not turn an open question into an accepted decision.

Read this register after the [architecture design](../arch/02_design.md#b24) to see
which choices have an accepted rationale. The design currently has working choices but
no accepted ADR; [J02](#j02) explains how a numbered decision record enters the register.

<a id="j01"></a>

## J01 Decision register

There are **no accepted architecture decision records** in `docs/adr/` at this revision.
The model and design documents contain working design choices, but no option comparison,
decision authority, date, and accepted ADR have been recorded for them. The architecture
review must not infer an ADR from a modeled arrangement.

| ADR | Decision | State | Architecture section |
| --- | --- | --- | --- |
| None registered | No accepted decision record | Open register | [Architecture overview](../arch/00_overview.md#a01) |

<a id="j02"></a>

## J02 Adding and controlling a decision

For a consequential architecture choice, create `0001_<subject>.md`, then continue with
the next unused four-digit number. Record the context and constraints, the selected
decision, alternatives and reasons, consequences and follow-up, and links to the affected
requirements, architecture, module design, interface control, and model views. Add a row
to [J01](#j01), and link it from the architecture section it changes. The draft pattern
in `cache/templates/docs/adr/0000_template.md` supplies the field order; the repository's
numbered-document and stable-anchor rules still apply.

An accepted record keeps its number and decision body. A later reversal gets a new ADR
that supersedes it, while the old record remains visible with its status. Candidate
questions, such as checker command packaging or interface transport, remain open design
work until the alternatives and authority are documented. Their current gaps are tracked
in [V04](../review/01_wish_status.md#v04) and [I09](../icd/01_interfaces.md#i09).
