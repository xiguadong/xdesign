# Portable document kit architecture

<a id="b33"></a>

## 1. Component boundary and ownership

`XDesignArchitecture::PortableDocumentKit` is the project-owned source for a
portable documentation contract. Its versioned source lives under
`document-kit/`, separately from authored project pages under `docs/` and from the
operational `.xdesign/` store. A selected profile gives the allowed
document classes, destination paths, section order, anchors, and figure rules.
The 1.1 catalog adds scenario manifests, source/license records, guides, examples,
and compatibility packs while retaining the same component and target-project boundary.
The target root, never the kit's location, determines where documents are written.

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
| `contract` | `DocumentContractSource` | Own profile, rules, templates, catalog/source manifests, compatibility roots, and versioned digest inputs. |
| `authoring` | `DocumentAuthoringBoundary` | Select effective entries, resolve paths and compatibility, write supplied content, and validate authored pages. |
| `cli` | `DocumentCliBoundary` | Expose deterministic commands, catalog data, selected identities, and diagnostics to scripts and CI. |
| `mcp` | `DocumentMcpBoundary` | Expose the same operations, catalog, and template resources to MCP-capable agents. |

The [detailed design](../design/document-kit.md#d113) specializes the
authoring boundary as `DocumentContractEngine` with catalog validation,
template routing, and compatibility resolution. The two access
boundaries remain adapters that reference that engine. This is a model allocation;
DOC-06 and DOC-12 exercise whether the source realization shares the engine and results.

## 2. Catalog and compatibility structure

The current root pack is schema 1.2.0 with 21 entries across ten classes.
`catalog/manifest.json` binds each scenario to one existing class route, required
sections, checks, view concerns, and template/guide/example/fixture resources.
`catalog/sources.json` owns source and license identity. Byte-exact v1 and 1.1.0
packs live at `compatibility/v1.0.0/` and `compatibility/v1.1.0/`. Later releases
have new versions and retain earlier packs at selectable compatibility roots; no
current catalog edit silently retargets an old document. The earlier 1.1 catalog
had 16 entries in the five flat classes.

`DocumentContractSource` carries the catalog/source/compatibility URIs and digest
identity alongside the original profile/rules/template URIs. The detailed-design
catalog validator loads that closure. The router chooses an entry for new work;
the compatibility resolver chooses each existing document's effective contract
before identity comparison. These are specializations inside the existing
`authoring` part, not new operational stores or documentation authorities.

Schema 1.0 packs and documents retain their original behavior and v1 digest. Under
a 1.1 catalog, only the exact bundled-v1 marker/registry identity automatically
uses retained compatibility resources. Other custom v1 identities use their
original pack explicitly, and a different 1.1 release fails closed until its
original pack is selected. Unmanaged pages remain outside catalog adoption.

This boundary lets a consumer project pin or adapt the profile while keeping
its model and documents as its own engineering products. It does not make the
kit a second authority for wish status or verification results. In this
repository, `docs/AGENTS.md` and the [documentation and SysML contract](../interfaces/documentation-contract.md#d27)
remain the source rules from which the bundled xdesign profile is derived.
The [engine design](../design/document-kit.md#d113) gives its operations and
check scope; the [acceptance cases](../verification/design.md#d114) identify
what must be exercised in another project.

## 3. Example and acceptance-evidence boundary

`DocumentContractSource` owns the exact stored example and canonical full-input
fixture bytes as digest-covered catalog resources. Each schema 1.1 example has one
64-zero sentinel only in its first-line managed-marker `contract` field; that
sentinel is data in the stored resource, never a selected or registered identity.
The digest hashes those exact bytes without normalization. A bundled fixture
contains all plan/create caller content, model and conditional figure inputs, and
selected profile/version/digest-version identity, but neither the actual nor the
sentinel digest. These ownership rules do not change the four logical parts or add
an operational store.

The acceptance harness is outside `DocumentContractSource` and the selected pack.
It materializes external acceptance manifests under the consumer project's
`cache/`, each containing only actual `expected_contract_sha256`, `fixture_path`,
and `fixture_sha256`. Their bytes are outside the pack digest closure, and their
own SHA-256 values are computed into separate acceptance evidence rather than
stored inside the manifests. The harness's one-field, in-memory example comparison
is not an engine operation and never rewrites a stored example, created page,
registry, or selected digest. Pack sentinel defects, target identity defects, and
fixture comparison failures therefore remain distinct at the existing authoring
boundary. Original v1 packs retain no-example/no-sentinel behavior; original 1.1
packs remain explicitly selectable after later releases.

<a id="b34"></a>

## 4. Structured release boundary

The schema 1.2 release preserves the four existing component boundaries and is
present in the current `document-kit/` source. The contract source owns the
route/template catalog, the retained root-level v1 pack and the exact retained
1.1.0 closure including its original v1 child. The authoring boundary resolves a
class/name/layer selection, validates each page against its recorded effective
contract, and reports whole-project or selected-document scope. CLI and MCP remain
adapters over the same engine; neither is an additional documentation authority.
The retained compatibility closures are immutable inputs, not new operational
stores or migration targets.

`XDesignDetailedDesign::StructuredDocumentKitDesign` allocates the route,
template, compatibility, operation, consumer and verification responsibilities.
The [detailed design](../design/document-kit.md#d115) and
[public interaction contract](../interfaces/document-kit-api.md#i21) define their
observable behavior and [planned verification](../verification/design.md#d116).
Source presence does not establish SDK case execution or semantic acceptance, and
this bilingual assignment does not add R20 satisfaction links.

<a id="b35"></a>

## 5. Bilingual release boundary

`XDesignArchitecture::BilingualDocumentKitRelease` specializes the existing
`PortableDocumentKit` allocation for the proposed `xdesign-sysml-docs@1.3.0`
contract. It does not add a fifth component. `contract` owns one current catalog
at `catalog/entries/<template-id>/<language>/`, shared language-neutral inputs,
localized figure-block templates, and immutable compatibility closures;
`authoring` resolves language and persisted identity through the same engine;
`cli` and `mcp` expose that engine without separate translation or discovery
rules.

The current tree has exactly 21 entry directories, two variants per entry and four
language-owned resources per variant. Shared sources, routing, mixed inputs,
models and SVGs are referenced once rather than copied by language. The
`compatibility/` tree is historical release storage, not an editable mirror of the
current catalog. The proposed 1.3 distribution retains the complete current 1.2.0
closure at `compatibility/v1.2.0/` in addition to the existing 1.0.0 and 1.1.0
closures; nested closures remain intact and independently selectable.

Language is part of the authoring contract and new managed identity, not the
class/name/layer route. The target project still owns documents, model sources,
links, figures and the single registry. A second creation at an occupied route is
a collision, never a language conversion. No component infers language from a
locale or prose, invents engineering content, or treats structural output as a
Chinese-quality or English-equivalence judgment.

The [detailed bilingual design](../design/document-kit.md#d122) allocates the
resource, operation and compatibility mechanisms; the [public contract](../interfaces/document-kit-api.md#i23)
defines selectors and resources; [BDK-01 through BDK-09](../verification/requirements.md#v20)
and the [verification design](../verification/design.md#d121) remain planned
implementation-verification obligations. The implementation projects this allocation through `bilingual.py`, the existing
core, CLI and MCP adapters, entry-owned resources and exact retained closures.
Product existence does not establish BDK execution, semantic equivalence or
engineering acceptance; those remain revision-bound verification obligations.
