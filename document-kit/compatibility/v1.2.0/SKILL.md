---
name: sysml-document-authoring
description: Use a versioned document contract to author and check SysML-backed requirements, architecture, design, interface, and verification documents before implementing code.
---

# SysML document authoring

Use this skill when a wish must produce SysML-backed design documents. It is
provider-neutral: an agent using GPT, Claude, Kimi, DeepSeek, GLM, or another
model follows the same contract. The document kit supplies rules and templates;
the chosen project's SysML model and authored documents supply the engineering
content. Do not generate engineering claims from the template alone.

## Load the contract

1. Select the target project root and a pinned, checked-out `document-kit`
   profile. Read the project's `AGENTS.md` and document map, if present, to
   find owning pages and stricter local rules.
2. Read `profile.json`, `RULES.md`, and `catalog/routing.md`. Choose the
   generic default or an explicit `template_id`, then read that entry's guide
   and exact template resource; its example is fictional and is not evidence.
   When an MCP/CLI adapter is configured, call `get_contract`, inspect the
   catalog, and read the applicable template from the same pinned pack. Use
   its reported actual contract digest for the wish. A schema 1.0 pack has no
   selectable template IDs and keeps its original class template behavior.
3. Inspect existing SysML elements, documentation, stable section IDs, and
   relevant wish records. Preserve existing page ownership and anchors.

The CLI and MCP adapters have the same six authoring/checking operations.
Bind one MCP process to one project root and profile; do not redirect a tool
call to another project. If the host has no MCP support, use the CLI. If the
adapter is unavailable, use the selected project's file tools and inspect
the exact template headings and fields manually. In every mode, run the
project's compiler and documentation checks separately.

## Stage sequence

| Stage | Authoring action | Check before proceeding |
| --- | --- | --- |
| Requirements | Register the wish in the project's authority, then model requirements. Call `plan_document` for each owning page and `create_document` or `update_section` with rationale, measurable acceptance conditions, and model links. | Call `check_documents`; run the model compiler for changed SysML and review requirement adequacy. |
| Architecture and design | Establish composition, behavior, constraints, decisions, interfaces, and planned verification in SysML. Plan and author architecture, design, interface, and verification pages through their class templates. | Call `check_documents`, run the compiler and strict docs build, and judge whether the documents explain the model and wish. |
| Figures | Use the project's separate drawing component to render selected SysML views. Place the canonical SVG beside its owning page; call `attach_figure` using the selected pack's figure block and the declared `concern` when the entry has view concerns. Keep `DocSection`, `DocFigure`, renderer mapping, diagram decision, embed, and six-field block aligned. | Inspect the final SVG for source fidelity and readability; rerun model/document checks. |
| Implementation | Implement against the reviewed design and link the relevant source and tests to its sections. Reassess any design revision made during coding. | Run applicable code and integration checks; rerun document and model checks for changed inputs. |
| Verification | Execute the planned cases, retain exact commands, outputs, artifact revision, judgment, and residual risks in the project's evidence store. | Call `check_documents` again and obtain the separate engineering judgment required by the flow. |

Resolve the class route and required inputs before a write. For an update or
figure attachment, pass the current document SHA-256 as `expected_sha256`
and read the new digest before another mutation. Supply stable anchor IDs
explicitly; do not infer them from display numbers. Never accept
an unresolved template token or a fabricated source, test, or review outcome.

In a gated SimpleFlow run, complete the requirements and design products before
the implementation stage. Submit each stage's actual artifacts and checker
results to that run's applicable gates. A documentation check report is evidence
for the rules it checked; it is not a semantic gate assessment or final
engineering acceptance. If an input is missing or a check is unevaluated,
record that state and keep the wish at the current gate.

Bind `check_documents` into another project's design-stage gate or CI if
failed documents must block progression. The skill alone cannot block a
flow transition.

## Structured API and flow authoring

Read [API.md](API.md). Select `api-design` with its configured layer, or one of
the four flow roles with a flow name. Plan before supplying content; create link
targets before a flow index. Keep API declarations, side effects, model rationale,
source projection and verification trace explicit. Use selected checks for a
registered page and whole-project checks for the project. Record the reported
scope and effective identity. Inspect page and registry before retrying an
incomplete or unknown publication outcome. Never adopt a stored example.
