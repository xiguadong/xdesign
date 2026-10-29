# Portable document kit architecture

<a id="b33"></a>

## 1. Component boundary and ownership

`XDesignArchitecture::PortableDocumentKit` is the project-owned source for a
portable documentation contract. Its versioned source lives under
`document-kit/`, separately from authored project pages under `docs/` and from
the operational `.xdesign/` store. A selected profile gives the allowed
document classes, destination paths, section order, anchors, and figure rules.
Templates provide the starting Markdown. `RULES.md` explains the decisions
and review limits. The target project root determines where documents are
written; the kit's own location does not become the target.

The contract engine is specified to resolve and validate paths before writing.
The CLI and stdio MCP server are adapters over that same engine. The portable
skill gives agents the design-first sequence using the rule pack and
templates; the validator reports malformed output and must be bound into a
project gate when progression must be blocked. A separate drawing tool
produces a model-derived SVG, and the kit validator
checks its attachment, section, caption fields, and path. Visual readability
and agreement with the SysML source require separate review.

The model decomposes `PortableDocumentKit` into four logical parts:

| Part | Model type | Responsibility |
| --- | --- | --- |
| `contract` | `DocumentContractSource` | Own the profile, rules, and template URIs as versioned source. |
| `authoring` | `DocumentAuthoringBoundary` | Resolve paths, write supplied content, and validate authored pages. |
| `cli` | `DocumentCliBoundary` | Expose deterministic commands to scripts and CI. |
| `mcp` | `DocumentMcpBoundary` | Expose tools and resources to MCP-capable agents. |

The [detailed design](../design/document-kit.md#d113) specializes the
authoring boundary as `DocumentContractEngine` and the two access
boundaries as adapters that reference that engine. This is a model allocation;
DOC-06 exercises whether the source realization shares the same engine.

This boundary lets a consumer project pin or adapt the profile while keeping
its model and documents as its own engineering products. It does not make the
kit a second authority for wish status or verification results. In this
repository, `docs/AGENTS.md` and the [documentation and SysML contract](../interfaces/documentation-contract.md#d27)
remain the source rules from which the bundled xdesign profile is derived.
The [engine design](../design/document-kit.md#d113) gives its operations and
check scope; the [acceptance cases](../verification/design.md#d114) identify
what must be exercised in another project.
