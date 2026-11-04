# Portable document kit design

<a id="d113"></a>

## 1. Contract-driven authoring

`XDesignDetailedDesign::DocumentContractEngine` takes a selected project root
and a versioned profile. The profile lists document classes, routes, required
chapters, anchor policy, and generic template. Schema 1.1 adds a catalog of
scenario entries, each with selection guidance, required sections, checks,
view concerns, template, guide, example, fixture, and source usage. The same
selected entry drives planning, rendering, updating, and checking. Caller content
supplies all engineering claims; templates never invent them.

The root pack is immutable 1.1.0 with five generic defaults and eleven scenarios.
A derived 1.1 profile may change inventory under its own identity and digest.
The exact bundled inventory does not constrain derived profiles. A byte-exact
v1 pack under `compatibility/v1.0.0/` preserves original class behavior and the
accepted digest. Later releases retain prior 1.1 packs at selectable roots and
require a new version for every changed resource set.

## 2. Operations and invariants

| Operation | Result or invariant |
| --- | --- |
| Read contract | Return profile/digest identity, digest version, classes, generic templates, rules, catalog entries, and source records. Schema 1.0 returns a synthesized compatibility projection with null template/scenario fields. |
| Plan document | Accept optional `template_id` only for 1.1, choose that entry or the class's one generic default, and return the existing class/slug route, selected entry inputs, and effective identity. No write. |
| Create document | Render the selected entry from caller content and diagram decisions, validate before writing, and persist matching effective identity in marker and registry. Existing destinations are refused. |
| Update section | Resolve the document's effective contract before checking identity, then update one stable anchor with the recorded entry's fields. Do not reselect a default or rewrite registry bytes. |
| Attach figure | Append optional `concern` after `expected_contract_sha256`; omit it only with no view concerns. Otherwise select exactly one declared concern/decision. Non-figure decisions always reject with `invalid-diagram-decision` before writing. |
| Check documents | Resolve each registry entry independently and return selected and per-document effective identities, hashes, structured findings, and unevaluated checks; per-path maps may be empty when no managed document is selected. |

The engine's six public operations are unchanged. Plan/create append optional
`template_id`; every result adds `contract_digest_version`, plan/create add
`template_id` and `effective_identity`, update/attach add `effective_identity`,
and check adds `effective_identity_by_path`. Every schema 1.0 document has null
template ID and digest version `"1"`. Schema 1.0 selection input is rejected as
`unknown-template` because no selectable IDs exist in that schema.

`DocumentTemplateCatalog`, `DocumentTemplateEntry`, `DocumentSourceRecord`,
`DocumentSourceUsage`, `DocumentViewConcern`, `DocumentDiagramDecision`,
`DocumentEffectiveIdentity`, `DocumentCompatibilityBinding`,
`DocumentContractDigestStrategy`, `DocumentClassDefault`, `DocumentPathDigest`,
`DocumentPathEffectiveIdentity`, and `DocumentValidationReport` carry the data.
`DocumentCatalogValidator`, `DocumentTemplateRouter`, and
`DocumentCompatibilityResolver` specialize validation, selection, and retained
contract resolution. Source-record item types are data projections, not constraint
declarations; optional multiplicities do not enforce invariants. The validator
solely enforces exactly one primary locator, required-but-nullable license URL,
and a copying review that is null only for `not-copied` or an object only for
`copied`. `ReadDocumentCatalogResource` and `ReadDocumentTemplateResource` model
additive MCP resources, not new tools.

## 3. Schema and source allocation

The planned source projection is:

| Model element | Source projection |
| --- | --- |
| `DocumentContractSource` | `document-kit/profile.json`, `RULES.md`, `README.md`, `SKILL.md`, `catalog/`, and `compatibility/` |
| Catalog/source/digest/router models | `document-kit/xdesign_doc_kit/catalog.py` |
| Identity/marker/registry/compatibility models | `document-kit/xdesign_doc_kit/compatibility.py` |
| `DocumentContractEngine` and six actions | `document-kit/xdesign_doc_kit/core.py` |
| `DocumentCliAdapter` | `document-kit/xdesign_doc_kit/__main__.py` |
| `DocumentMcpAdapter` and two resources | `document-kit/xdesign_doc_kit/mcp_server.py` |
| Acceptance fixtures | `document-kit/tests/test_acceptance.py`, `test_core.py`, `test_mcp_server.py`, `test_catalog.py`, `test_compatibility.py`, `test_examples.py`, `test_security.py` |

Each manifest entry has a unique ID, class, scenario, use/omission guidance,
required inputs/dependencies, required sections, four resource URIs, sources,
implemented check IDs, and view concerns. Each class has exactly one generic
default. Missing, duplicate, unsafe, unreadable, or foreign declarations fail
`invalid-catalog` before target writes. The source register carries primary and
additional locators, retrieval, ideas, limits, and license fields. Entry-source
pairs carry `not-copied` or `copied` decisions; copied material requires a
recorded review structurally and a separate adequacy judgment.

The [interface contract](../interfaces/document-kit-api.md#i20) gives exact
profile/manifest fields, marker and registry representations, resource layout,
digest framing, operation results, and diagnostics. The [portable architecture](../architecture/document-kit.md#b33)
retains the four logical boundaries and their ownership.

## 4. Effective identity and compatibility

`DocumentEffectiveIdentity` contains `profile_id`, `version`, `document_class`,
`template_id`, `contract_sha256`, and `contract_digest_version`. New 1.1 markers
and registry entries record all six fields. Registry schema 1 gains optional
`template_id` and `contract_digest_version` keys, so old registries remain
readable. Registry entries contain no document hash: valid updates preserve
registry bytes while changing only authorized document content.

Resolution precedes mismatch checks. An explicitly selected original v1 or 1.1
pack uses its computed identity when its own marker and registry agree. Under a
1.1 catalog, only exact bundled `xdesign-sysml-docs@1.0.0` marker/registry
identity and the known v1 digest select retained compatibility resources.
Missing legacy fields infer null/digest version 1 without being rewritten.
Unknown/custom v1 identities and different 1.1 releases fail; the original pack
must be selected explicitly. A recognized legacy identity difference from the
selected catalog is not an own-identity mismatch.

Each write revalidates selected and effective contract pins and expected document
SHA-256. A failed update leaves the existing page unchanged. Create removes only
a newly created page if registry publication fails, while check reports any
interrupted page/registry pair honestly; no two-file atomicity is claimed.

## 5. Digest and release algorithms

Digest version `"1"` uses the existing v1 path/length/byte framing and resource
set, reproducing `1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6`.
Version `"2"` initializes SHA-256 with `xdesign-doc-kit-contract`, NUL, ASCII
`"2"`, NUL, then applies the same path/NUL/length/NUL/byte framing to a sorted
resource closure. The closure covers profile, rules/readme/skill, the bundled pack
manifest, source register, all templates/guides/examples/fixtures, mixed resources,
figure blocks, and the complete declared compatibility packs. Every resource is
local and reviewed; there is no directory-glob or network inclusion.

Every 1.1 release is immutable. A changed byte changes the digest and requires a
new version. Later distributions retain prior packs so old documents remain
usable through original-pack selection. MCP pins the selected closure at launch,
including compatibility bytes, and CLI revalidates it before writes.

The exact stored example bytes are digest inputs, including their managed-marker
sentinel. No sentinel stripping, zeroing, canonicalization, or other normalization
is permitted while hashing. Recomputed `D == Z` is `invalid-catalog`; a pinned
descriptor claiming `Z` while recomputation is nonzero is `contract-drift`; an
`expected_contract_sha256=Z` request is `invalid-request`. These are three distinct
contexts, and none selects or writes a zero identity. Production SHA-256 and both
adapters always use the actual nonzero recomputed `D`; only an isolated test may
inject a zero-returning provider at the catalog-loader/digest-strategy seam.

## 6. Stored examples and acceptance fixtures

### 6.1 Sentinel and full-input resource contract

For schema 1.1, `Z` is exactly 64 lowercase ASCII zero characters. Each manifest-
named stored example `E` has one exact first-line managed marker whose non-contract
profile/version/class/template identity is actual, whose digest version is `"2"`,
and whose only `Z` is that marker's `contract` value. A total-byte scan establishes
exactly one occurrence in `E`; `D` occurs nowhere in `E`. `Z` is forbidden in every
other selected-pack resource, including the bundled pack manifest, guides,
templates, fixtures, source register, mixed resources, model stubs, figure assets,
and compatibility packs. These selected-pack defects are `invalid-catalog` before
any target operation. A target marker or registry containing `Z`, a copied example,
or a forged target registry is instead `registry-mismatch` and is never adopted or
registered. Schema 1.0 has no examples or sentinel and retains its v1 bytes and
digest; a derived 1.1 profile applies the same rule to every example it declares.

`DocumentExampleFixture` models the digest-covered bundled fixture as data. In an
isolated fixture, the existing complete `document_class`, `name`, `template_id`,
`metadata`, `sections`, and `expected_document` values are retained and the fixture
adds top-level `profile_id`, `version`, and `contract_digest_version`. It also has
`models` and `figures` arrays containing every pack/target model or conditional
figure input needed by that example; figure-free examples use an empty `figures`
array. The mixed fixture carries the same selected identity at top level and its
existing `documents`, `coverage`, `models`, and `figures` values. Every mixed
`documents[].expected_document` references one of the same 16 stored examples;
there are no second example bytes or a second sentinel convention. Neither fixture
form stores `D`, `Z`, `contract_sha256`, or `expected_contract_sha256`.

### 6.2 External manifest and evidence binding

Before creation, the acceptance harness—not a sixth public engine operation and not
`DocumentContractSource`—materializes one manifest per isolated or mixed bundled
fixture under `cache/document-template-library-acceptance/`, outside the selected
pack and its digest closure. `DocumentAcceptanceManifest` has exactly three fields:

| Field | Value |
| --- | --- |
| `expected_contract_sha256` | Actual recomputed nonzero `D` supplied to creation. |
| `fixture_path` | Normalized pack-relative path of the canonical bundled fixture. |
| `fixture_sha256` | Recomputed SHA-256 of that exact fixture. |

The external manifest never contains caller inputs, `Z`, `manifest_sha256`,
`self_sha256`, another self/own-hash field, or a second full-input copy. Such an
extra hash field is `fixture-mismatch` before creation, with no field exclusion.
The harness separately computes and retains the external manifest's own SHA-256 in
S-12/S-16 evidence; it never reads that hash from the manifest. Actual-digest,
fixture-path/hash, or retained external-manifest-hash drift is a stale-fixture
failure and never silently refreshes a fixture, example, marker, registry, or
manifest. `DocumentExampleAcceptanceHarness` and `DocumentExampleComparison` model
this acceptance-only boundary and its evidence; neither adds a product API.

### 6.3 Exact comparison and negative cases

DOC-11's acceptance comparator is test-only. After recomputing `D`, validating the
whole selected pack, and validating the external manifest and fixture hash, it
independently parses the created document `C` and its registry and asserts their
actual path/profile/version/class/template/`D`/digest-version identity. It locates
only the parsed first-line `contract=` value span in `E`, replaces that one `Z`
with `D` in an in-memory copy `E'`, asserts the created marker equals the `E'`
marker, and compares every byte of `E'` and `C`. It then runs `check_documents` on
the unchanged `C` and registry and requires the declared structural predicates to
pass. Stored examples, created bytes, selected digests, and registries are
hashed before and after; no product operation performs this substitution.

Marker stripping, body-only comparison, whitespace or line-ending normalization,
other-field normalization, and global replacement are `fixture-mismatch` in crafted
negative fixtures. Evaluation order is request shape, selected pack, target
identity, then fixture comparison. Thus request `Z` is `invalid-request`, selected-
pack sentinel defects or recomputed zero are `invalid-catalog`, a zero descriptor
with nonzero recomputation is `contract-drift`, target zero identities are
`registry-mismatch`, and only a later byte discrepancy is `fixture-mismatch`.
Every rejection leaves target documents and registry bytes unchanged.

### 6.4 Acceptance allocation and compatibility negatives

`document-kit/tests/test_examples.py` owns S-09's 16 isolated positives and S-10's
six current mixed references, all five classes, two templates in one class, and the
coverage list. The same suite owns S-01/S-03/S-04/S-07/S-08 comparison and target
identity cases and S-11/S-12 example-drift and propagation cases. Catalog/security
suites own whole-pack S-02/S-05 scans, S-06's isolated zero provider and proof that
CLI/MCP cannot reach it, self-hash manifest rejection, stale fixture/manifest
hashes, and one-byte digest mutations in every resource category. Compatibility
suites retain DOC-01 through DOC-07, custom-v1 original-pack selection, bundled-v1
routing, original 1.1-pack selection, unmanaged-page refusal, and S-13's unchanged
v1 digest/no-sentinel behavior. CLI/MCP tests own S-14: actual digest propagation,
pins, and template resources that never return example bytes as a contract.

## 7. Diagram decisions and check boundaries

Manifest view concerns name their section, applicability, trigger, and allowed
alternatives. The corresponding section's `diagram_decisions` field is a JSON
array of `{concern, applies, choice, rationale, asset, model_view}` objects.
Expected concerns require an explicit decision; conditional concerns require a
Boolean applicability decision. Figure choices require a conforming asset and
figure block; table, prose, or not-applicable choices require a rationale and no
asset. No class-wide or template-only diagram mandate exists.

Required headings, IDs, content, placeholders, paths, source/license records,
identity, digest, diagram decisions, and figure blocks are deterministic.
The existing [xdesign documentation audit](../interfaces/documentation-contract.md#d31)
checks this repository's `DocSection` and `DocFigure` targets and local
embeds. The compiler separately checks SysML syntax and supported references.
Source/license adequacy, scenario fitness, coverage, rationale quality, model
meaning, and final figure fidelity/readability remain review judgments.

Diagnostics reuse existing codes where applicable and add `invalid-catalog`,
`unknown-template`, `template-class-mismatch`, `template-default`,
`unsupported-legacy-identity`, `invalid-source`, `invalid-license`,
`copying-review-required`, `unsafe-url`, `unsafe-markdown`,
`missing-diagram-decision`, `invalid-diagram-decision`, and `fixture-mismatch`.
An unknown schema 1.1 ID and every supplied schema 1.0 `template_id` are
`unknown-template`, because schema 1.0 has no selectable IDs; request digest `Z`
remains `invalid-request`. Recomputed `D == Z` is always `invalid-catalog` before
any descriptor comparison, while `contract-drift` requires actual recomputation
that is nonzero and not `Z`. Unknown/custom v1 identity under a 1.1 catalog is
`unsupported-legacy-identity`, while a document from another 1.1 release is
`registry-mismatch`. The I20 table is the single complete enumeration of all 41
operation-error and structural-finding codes, including `invalid-profile`,
`registry-mismatch`, and the retained v1 document/model/figure codes; rule labels
such as `input` and `DOC-001` through `DOC-010` are not additional codes. Request
and route validation precedes selected-pack checks, selected-pack checks precede
target identity resolution, and content/model/figure checks precede fixture
comparison. Profile `schema_version` errors are `invalid-profile`, while catalog
manifest schema errors are `invalid-catalog`. Model-root configuration defects
precede root existence/scan results: malformed configuration is `invalid-profile`,
a well-formed missing or source-less required root is `missing-model-root`, and a
scan I/O failure is `unreadable-input`. Source record field-set, identity, and
primary-locator checks precede URL/local-path checks, then license nullability
and license URL checks: missing license fields are `invalid-source`, present
unsafe URLs are `unsafe-url`, and only a complete record's invalid nullability
combination is `invalid-license`. Update/attach checks registry ownership,
document readability, expected hash freshness, then effective-contract identity.
Annotation locator path failures stay `unsupported-annotation`; model-locator
`invalid-docfigure` and document-asset `missing-figure` are independent findings.
Independent document/figure findings are collected and sorted by file, line,
code, and detail, which is reporting order rather than semantic priority. Each
diagnostic retains the exact I20 shape and never becomes an acceptance verdict.

## 8. Source allocation and scope

The `document-kit/` pack owns the profile, rules, 16 template/guide/example/
fixture sets, source register, routing guide, one mixed example, compatibility
resources, and portable skill. `core.py` stays the six-operation facade;
`catalog.py` owns catalog/schema/source/digest/selection and bundled-fixture
validation, and `compatibility.py` owns identity/marker/registry/original-pack
resolution and target sentinel mismatch handling. The acceptance comparator and
external manifests belong to `document-kit/tests/test_examples.py` and acceptance
evidence under `cache/`, not to the public engine. Catalog/security tests own the
isolated digest-provider seam. CLI and MCP adapters add optional selection and
figure concern, the catalog, and second-template resources without a zero-provider
or example-comparison path.

The bundled profile keeps five flat classes. Flow-specific and per-API routes
need a reviewed derived profile. No new SysML template catalog, migration tool,
remote retrieval, code evaluation, main Rust stage CLI, or store feature is
introduced. [DocumentKitVerification](../verification/design.md#d114) defines
acceptance. Current implementation status and executed results belong to wish and
review records; this design does not establish implementation or engineering acceptance.
