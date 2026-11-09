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

The current root pack is immutable 1.2.0 with 21 entries across ten classes: the
16 earlier flat-class entries and five structured entries. The earlier 1.1.0 pack
has five generic defaults and eleven scenarios. A derived profile may change
inventory under its own identity and digest. The exact bundled inventory does not
constrain derived profiles. Byte-exact v1 and 1.1.0 packs under `compatibility/`
preserve original class behavior and accepted digests. Later releases retain prior
packs at selectable roots and require a new version for every changed resource set.

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

The retained 1.1.0 bundled profile keeps five flat classes. Its flow-specific and
per-API routes need a reviewed derived profile; the current [structured release](#d115)
is a separate contract. No new SysML template catalog, migration tool, remote
retrieval, code evaluation, main Rust stage CLI, or store feature is introduced.
[DocumentKitVerification](../verification/design.md#d114) defines acceptance.
Current implementation status and executed results belong to wish and review
records; this design does not establish engineering acceptance.

<a id="d115"></a>

## 9. Structured document release design

This section records the design allocation for the [structured requirements](../requirements/requirements.md#r20)
in `XDesignDetailedDesign::StructuredDocumentKitDesign`. Schema 1.2.0 and its
21-entry catalog are present in the current `document-kit/` source. The R20
requirements retain their proposed requirement-baseline status, and
`mbse/model/traceability.sysml` still asserts no `satisfy` links for those eight
requirements; this bilingual assignment does not change that boundary. The
mapping below identifies design responsibilities without asserting SDK execution
or engineering acceptance. The exact public shapes, errors and examples are in
the [structured interaction contract](../interfaces/document-kit-api.md#i21);
[SDK-01 through SDK-13](../verification/requirements.md#v19) and the
[implementation verification plan](../verification/design.md#d116) define the
required evidence. No existing figure selection changes.

| Requirement | Proposed design owner in `StructuredDocumentKitDesign` |
| --- | --- |
| `structuredDocumentationHasNamedRoutes` | `StructuredRouteResolver` |
| `apiDesignDocumentsCoverContract` | `StructuredTemplateLibrary` |
| `flowDesignDocumentsCoverLayers` | `StructuredTemplateLibrary` |
| `structuredDocumentationUsesSafeOperations` | `StructuredDocumentOperations` |
| `structuredDocumentationKeepsVersionIdentity` | `StructuredCompatibilityResolver` |
| `structuredDocumentationHasSharedSurfaces` | `StructuredDocumentOperations` |
| `documentKitPublicApiIsDocumented` | `StructuredConsumerContract` |
| `structuredDocumentationEvidenceIsReviewable` | `StructuredValidationBoundary` |

These are proposed allocations for the source projections in section 9.4.
`XDesignVerification::StructuredDocumentKitVerification` retains the separate
planned `verify` relationships; they establish neither satisfaction nor execution.
Completed-stage artifacts and passed assessments retain their accepted bytes under
the [revision contract](evidence-and-revisions.md#e03). Any later formal traceability
change needs a supported lifecycle path that preserves those completed-stage
obligations; this design does not assume that an earlier stage can be reopened.

### 9.1 Release, routes and template ownership

The current bundled contract is `xdesign-sysml-docs@1.2.0`, schema `1.2`, with digest
version `"2"`. It retains the five schema 1.1 classes, paths, generic defaults,
16 template IDs, section/field/view contracts and old positional calls. It adds
five classes, each with exactly one generic default. A class and its validated
components select the route; callers never supply an output pathname. The bundled
xdesign layer set is `core`, `simple-flow`, `wish-verify`, `storage`.

| Class | Generic template ID | Route | `name` means | `layer` |
| --- | --- | --- | --- | --- |
| `api-design` | `generic-api-design` | `docs/design/api/{layer}/{name}.md` | API basename; `index` reserved | required, selected from the layer set |
| `flow-index` | `generic-flow-index` | `docs/design/flows/{name}/index.md` | flow basename | absent |
| `flow-definition` | `generic-flow-definition` | `docs/design/flows/{name}/definition.md` | flow basename | absent |
| `flow-implementation` | `generic-flow-implementation` | `docs/design/flows/{name}/implementation.md` | flow basename | absent |
| `flow-views` | `generic-flow-views` | `docs/design/flows/{name}/views.md` | flow basename | absent |

The schema 1.2 profile retains 1.1 fields. Each new `document_classes` entry adds
`selector_kind` (`api` or `flow`); `api-design` also has `allowed_layers` and
`reserved_names`. Its existing `path_template`, `filename_pattern`, `template`,
`required_sections`, and `default_template_id` fields have their usual meanings,
except that the pattern constrains the supplied `name`, not a fixed `index.md`
stem. A missing `selector_kind` means `flat` for the retained classes; schema 1.2
uses the same `^[a-z0-9]+(?:-[a-z0-9]+)*$` `filename_pattern` for every class.
Any class may declare a unique `reserved_names` list of valid slugs; the bundled
API list contains `index`. A derived 1.2 profile narrows names by that list rather
than by an arbitrary regular expression, so route overlap remains decidable. This
restriction does not change original 1.0/1.1 profile parsing. The five flow/API templates, guides, fictional complete examples and fixtures
join the 16 existing entry packages; the bundled inventory is 21 entries and ten
classes. Entries retain the 1.1 manifest shape and source/decision requirements.
A 1.2 fixture has the 1.1 full-input fields plus `layer` (string for API, null for
all other classes); the mixed fixture's document records have the same additive
field. A derived 1.2 profile has its own identity, reviewed catalog and layer set;
it is not forced to include this bundled inventory.

The 1.2 profile, manifest and returned catalog each carry `schema_version="1.2"`;
their identity fields agree, and the source register keeps the 1.1 shape and eight
existing research IDs. New entries cite known sources with explicit copying
decisions, subject to source/license review. The bundled 1.2 inventory is the exact
16 old IDs plus the five listed IDs: the old five classes retain their two-or-more
scenario entries, while the new classes need only their generic default. The mixed
fixture has 11 distinct documents: one per ten classes plus a second template in
one original class, with a concern-to-template/omission coverage list. It reuses
the 21 stored examples rather than creating a second example-byte convention.
Its API `layer` and each name must resolve to the fixture's declared target; the
other classes use null `layer`. The old 1.1 loader and its exactly-16/six-document
inventory remain unchanged.

Route templates are parsed as POSIX components: a static lowercase kebab-case
component, a whole-component `{layer}` or `{name}`, or a filename `{name}.md`;
fixed flow filenames are `index.md`, `definition.md`, `implementation.md`, or
`views.md`. Each route is beneath the configured `docs_root`; placeholders appear
exactly once as the selector kind requires. Names and layers match
`^[a-z0-9]+(?:-[a-z0-9]+)*$`; layer values must occur exactly once in the profile's
nonempty unique allowlist, and reserved names are excluded. A derived profile may
choose different safe static prefixes, roots and layer vocabulary while retaining
these selector shapes. The loader rejects malformed, ambiguous or overlapping
routes: compare component counts and, at each position, intersect static literals,
the slug language minus that route's finite reserved names, and the finite layer
set; `{name}.md` has the literal `.md` suffix. Slug/slug intersections are
nonempty; a layer/slug intersection is tested against each allowed layer after
reserved-name exclusion; layer/layer uses set intersection; static components are
checked against the other component's accepted values. If every component can
match the same path, reject the pair as `invalid-profile`. Each variable occurs
once, so component intersections are independent. This prevents route ownership
from depending on iteration order. No supplied name may add path separators.
The resolver matches the full path, not a string prefix, and reverses a registered
path to one unique class/name/layer before applying that effective contract.

`StructuredTemplateLibrary` owns the five new Markdown templates. Each has one
H1, caller metadata and ordered H2 sections; every section has an explicit unique
anchor, caller `content`, and exactly the fields below. The xdesign API primary
and section anchors use explicitly allocated `U` IDs, enforced in schema 1.2;
flow anchors prefer `D`, while the existing cross-cutting prefixes remain valid.
Anchor uniqueness is checked case-insensitively across the project. Values are nonempty strings; a reasoned
`not applicable` is caller content, not an automatic engineering claim.

| Class | Section key → heading: required fields |
| --- | --- |
| `api-design` | `purpose` → `Purpose and boundary`: `api_identity`, `api_kind`, `layer`, `purpose`; `contract` → `Declaration and contract`: `declaration`, `inputs`, `outputs`, `errors`, `side_effects`; `behavior` → `Behavior and constraints`: `behavior`, `invariants`, `constraints`; `trace` → `Trace and verification`: `model_realization_or_reason`, `source_projection`, `requirements_links`, `design_links`, `verification_links`; `limits` → `Status and limits`: `unresolved_items`. |
| `flow-index` | `scope` → `Scope and navigation`: `flow_scope`, `navigation_links`; `trace` → `Status and trace`: `model_elements`, `requirements_links`, `verification_links`, `unresolved_items`. |
| `flow-definition` | `stages` → `1. Stages and gates`: `stages`, `transitions`, `exit_gates`; `behavior` → `2. Behavior and constraints`: `behavior`, `invariants`; `trace` → `3. Trace and verification`: `model_elements`, `requirements_links`, `verification_links`. |
| `flow-implementation` | `allocation` → `1. Allocation`: `components`, `interfaces`; `projection` → `2. Behavior and source projection`: `behavior`, `source_projection`, `constraints`; `trace` → `3. Verification and limits`: `model_elements`, `verification_links`, `unresolved_items`. |
| `flow-views` | `coverage` → `1. Coverage and applicability`: `view_coverage`, `applicability_reasons`; `views` → `2. Views and references`: `model_views`, `figure_references`, `verification_links`; `limits` → `3. Status and limits`: `unresolved_items`. |

Each entry's `required_sections` is the ordered 1.1 array of objects with exactly
`key`, `heading`, and `required_fields`. The template bytes use the 1.1 marker
line, `# {{title}}`, a blank line, then
`> Status: {{status}} | Revision: {{revision}} | Model sources: {{model_sources}}`.
For each table row's section in order, emit a blank line, standalone
`<a id="{{section_id:<key>}}"></a>`, blank line, `## <heading>`, and a blank
line. Each field is a separate `**<label>:** {{field:<key>}}` line followed by a
blank line; `<label>` replaces underscores with spaces and capitalizes only the
first character. Finally emit `{{content:<key>}}`, with one blank line between
sections and a final LF. Rendering substitutes caller content inside the existing
`<!-- document-kit:content:<key>:start -->` and matching `:end -->` markers;
attachment inserts its figure block after the end marker in that same section.
The entry's template tokens and section/field keys are validated one-to-one.
The flow landing page uses natural headings, like the API pages; the other three
flow roles use local decimal headings. The API `purpose.fields.layer` must equal
the resolved layer on create, check and update. This is a routing consistency
predicate, not a judgment about the caller's API identity or engineering prose.

The five new bundled entries declare no mandatory `view_concerns`: an author can
attach an independently supplied figure to any of their sections with `concern=null`.
The views fields still require caller-authored applicability and references. Unknown
concerns are rejected. A derived entry may declare the existing 1.1 concern/decision
schema; table/prose/not-applicable decisions reject figure attachment and retain a
reason. Schema 1.2 additionally permits `applicability="optional"`, with a nonempty
trigger and an allowed alternatives list, for a concern that need not be selected.
Its section still has a `diagram_decisions` field, a string containing a JSON array;
an empty array is structurally valid for that optional concern and leaves its
applicability for semantic review. Existing expected/conditional/not-applicable
concerns keep their 1.1 requirements unchanged.

For an unselected optional concern that permits `figure`, 1.2 `attach_figure` may
receive an explicit caller `decision` with the six I20 decision fields. It atomically
adds the figure decision and its matching supplied block to that section; a complete
page can therefore be created and checked before attachment. A stored non-figure
decision cannot be overridden by attachment, and a duplicated embedded asset remains
an error. A caller may explicitly clear an optional non-figure decision before
making a separate figure decision. An update cannot clear or change a stored
figure decision so that its retained block no longer matches; removing or replacing
an attached figure is outside these operations. No rationale is synthesized from a figure
caption. Legacy selected 1.0/1.1 packs retain their original concern and attachment
behavior, and no output supplies facts, model realization, images, code or execution
evidence.

### 9.2 Identity and compatibility allocation

`StructuredCompatibilityResolver` keeps registry schema 1 and the exact existing
entry keys: `path`, `class`, `profile_id`, `version`, `contract_sha256`, and,
for catalog documents, `template_id` and `contract_digest_version`. New entries
use digest version `"2"` and the existing first-line marker, with version `1.2.0`
and one of the new class/template IDs; layer and flow components are recovered
from the validated path, not stored as new keys. This keeps the original registry
reader able to load the whole registry when an original pack updates its own page.
It does not make that reader understand a new class: its unchanged whole-project
check may report an identity/class mismatch. Updates and attachments under any
recognized effective pack do not rewrite the registry or marker.

Before replacing the root pack, implementation copies the complete original
1.1.0 resource closure byte-for-byte to `compatibility/v1.1.0/`; that copy retains
its own `compatibility/v1.0.0/` child. The existing root-level
`compatibility/v1.0.0/` is also retained byte-for-byte at its original selectable
path. The bundled 1.2 profile's `compatibility_packs` has exactly the two 1.1-shaped
descriptors for `compatibility/v1.0.0/` and `compatibility/v1.1.0/`, with their
original profile IDs, versions, digests and digest-version strings; the 1.1 child
retains its original v1 descriptor. The resolver validates the full descriptor
graph and indexes the two original identities plus the selected 1.2.0 identity.
The only permitted duplicate identity in the bundled graph is these two v1 copies:
their profile-relative closure paths and bytes must match exactly, and the root-level
copy is the canonical v1 binding. Other duplicate identities, cycles, overlapping
non-nested roots and symlink components fail. The 1.1 child must recompute
`e2b827c6e929f4a8d03344a91ca72f2eb837962e916d5e7f0487e315e1440cef` with digest
version `"2"`; its nested v1 must recompute
`1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6` with version
`"1"`; the root-level v1 copy must have that same digest and closure. Descriptor
profile, version, digest and digest-version must all agree with each validated
child. Custom packs or other releases are not inferred; a separately
reviewed derived 1.2 pack may add an exact validated binding, otherwise affected
checks/writes fail. The selected digest and a page's effective digest are distinct.

The resolver also validates a single project namespace for a selected 1.2 family.
Every bound pack has the same `project.docs_root`, so all releases use the same
registry. A class present in multiple packs has the same path template and
selector kind (missing means `flat`); different classes' routes must not overlap
across packs. Each `(profile_id, version)` maps to one digest and digest version,
even when two byte-identical v1 roots carry that release. Conflicting releases or
incompatible namespace bindings fail `invalid-profile` before project access.
A derived profile may change roots and prefixes only with compatible bindings
or without those bindings; copying a descriptor does not relocate an old page.
Automatically bound routes must satisfy the 1.2 selector grammar and slug
pattern, making cross-pack overlap checks decidable; incompatible custom legacy
routes remain usable by explicit selection of their original pack.
Each page's model-source policy remains its effective pack's policy. Context
scans use the union of selected and actually checked effective model roots, with
each checked page independently satisfying its own required-root/source rules.

The new pack's version-2 digest is SHA-256 over the prefix
`xdesign-doc-kit-contract\0` followed by ASCII `2\0`, then every distinct
pack-relative resource path sorted by UTF-8 bytes, NUL, ASCII decimal byte length,
NUL and exact file bytes. Its closure contains its profile, rules, README, SKILL,
consumer `API.md`, manifest, sources, routing guide, all templates/guides/examples/
fixtures, mixed resources and figure block, plus each declared child closure
recursively at its prefixed path. Shared paths are hashed once; conflicting roles
or bytes fail. Paths are explicit, not globs, and no digest bytes are normalized.
The 1.1 and v1 loaders recompute their own digests from paths relative to their own
roots before the enclosing descriptor is trusted. The selected 1.2 digest is not
specified until the future exact bytes exist.

Stored-example sentinel validation is **per owning pack**, not a scan that treats
retained child examples as parent resources. The 1.2 owner's 21 declared examples
have exactly one 64-zero sentinel in their exact first-line marker contract field;
its other owned files have none. The 1.1 child independently enforces its original
16-example sentinel rules and its v1 child independently has no examples/sentinel.
The parent still hashes every retained byte as stored. Every example's non-contract
marker fields match its owning profile/version/class/template/digest version; no
example contains the owner's actual digest. A sentinel elsewhere, duplicate or
malformed marker, or zero recomputed digest fails `invalid-catalog`. A zero
compatibility descriptor with a nonzero actual child digest is `contract-drift`;
it is classified by parsing that descriptor value, never by modifying bytes used
for hashing. A zero caller expected digest is `invalid-request`; a zero or forged
target marker/registry is `registry-mismatch`. Child sentinel locations are
accepted only after that child's own complete validation and exact descriptor
binding, not by globally ignoring nested zeros.

Canonical digest-covered fixtures contain full caller inputs, selected non-digest
identity, model/conditional figure inputs and an example reference, but neither
the actual selected digest nor the zero sentinel. Acceptance manifests live in
`cache/` outside every pack closure and contain exactly the actual digest, fixture
path and fixture SHA-256; their own SHA-256 is separate evidence, not a field in
the manifest. The existing one-field in-memory sentinel-to-actual comparison,
independent marker/registry assertions and full-byte equality apply to all 21
selected 1.2 examples and separately to unchanged child examples. No example is
returned as a template or adopted as a document.

### 9.3 Operation, validation and publication allocation

`StructuredRouteResolver` validates the request, class/name/layer and selected
contract, then resolves a safe project-relative path with no symlink components.
Planning is read-only and may precede model creation. Creation requires each
page's caller metadata and sections, checks configured model roots, and uses an
exclusive no-replace publication. An existing unmanaged page is observable by
plan and ordinary project checks but cannot be overwritten, registered, updated
or attached to by the kit. A forged marker without its registry entry is an error.

`StructuredDocumentOperations` resolves the effective contract from a registered
entry before enforcing its sections and route. Update and attachment first check
request shape, registry ownership, document readability and expected SHA-256,
then effective identity, section/concern and candidate validation; recheck the
hash and selected pack immediately before atomic replacement. A successful update
or attachment preserves the registry and every other section. All supported packs
use the same root-bound writer, no-follow component access and lock. Missing,
malformed, duplicate, unsafe or unreadable metadata/model sources fail before
publication. Detailed failure codes and the non-atomic create/page/registry
boundary are defined in I21, not inferred from a generic exception.

`StructuredValidationBoundary` has whole-project and selected-document scopes.
Whole-project mode resolves every registered identity, validates managed pages
under their effective contracts and applies the ordinary project checks to
unmanaged pages. Selected mode requires a registered target selected by the same
class/name/layer tuple; it validates that page under its effective contract and
reports its own checked path, identity and context scope. It scans readable project
documents for explicit anchors and reports collisions involving the selected page,
checks that page's local links and model annotations targeting it, and validates the
shared registry's shape and unique paths. Unrelated pages' template/identity
findings are not claimed as checked. An unreadable document needed for the global
anchor scan fails `unreadable-input`; the result never asserts whole-project success.
Whole-project and selected checks are read-only and list checked and scanned paths,
global/context findings and `semantic_review_required=true`.

In schema 1.2, a class filter selects registered pages by their recorded class
and unmanaged pages by a complete route match, including fixed flow filenames
and allowed layers. It never uses the text before `{name}` as a prefix: that
would select all four flow roles together and miss an unexpanded API layer.
Registry shape/path integrity, project anchor uniqueness and model locators
remain global in class-filter mode. Unrelated managed-page template and identity
checks remain outside that mode; an unknown identity in a selected class fails
its effective resolution. Original selected packs keep their old filtering.

Anchor context is an occurrence index of every unfenced explicit ID, not just
the first path for an ID. Selected checks report every collision involving the
selected page regardless of scan order, including duplicates within that page;
local fragment links are resolved in their named target file. Selected mode
validates model annotations targeting that page; malformed annotations whose
target cannot be determined are reported as context findings, since their
irrelevance cannot be established. It does not validate unrelated page templates
or report unrelated anchor collisions as selected failures. Create/update/attach
validate the candidate's same links and relevant model locators using a virtual
candidate at its resolved path, while retaining the existing project-wide anchor
uniqueness prerequisite. A model annotation can therefore refer to the page being
created without requiring premature publication. Required model reads and context
scans must succeed before a passing result or publication.

### 9.4 Source and test projection

| Design element | Intended source or verification projection |
| --- | --- |
| `StructuredRouteResolver` | schema/route validation in `document-kit/xdesign_doc_kit/catalog.py`; route matching and selection in `core.py` |
| `StructuredTemplateLibrary` | `document-kit/profile.json`, `catalog/manifest.json`, five new templates/guides/examples/fixtures, refreshed 1.2 copies of the original 16 entries, sources and routing/mixed fixtures |
| `StructuredCompatibilityResolver` | `compatibility.py` and schema-dispatched pack loading in `catalog.py`; exact retained `compatibility/v1.0.0/` and `compatibility/v1.1.0/` closures |
| `StructuredDocumentOperations` | `core.py` read/write/check, safe publication and typed I/O failure boundary |
| `StructuredConsumerContract` | `__init__.py`, `__main__.py`, `mcp_server.py`, distributed `API.md`, README/RULES/SKILL and [I21](../interfaces/document-kit-api.md#i21) |
| `StructuredValidationBoundary` | `document-kit/tests/test_structured.py`, existing catalog/compatibility/security/example/MCP suites, and [SDK-01–13](../verification/requirements.md#v19) |

These paths are planned source allocations. The implementation must include real
stdio MCP tests rather than treating a skipped optional adapter test as parity.
The independent semantic review covers engineering content, links, model meaning,
figure fidelity and applicability, as well as source/license adequacy; mechanical
checks and compiler reports do not establish these judgments.

<a id="d122"></a>

## 10. Bilingual release and entry-owned catalog design

`XDesignDetailedDesign::BilingualDocumentKitDesign` answers the accepted [R22
requirements](../requirements/requirements.md#r22) with the
`xdesign-sysml-docs@1.3.0`, schema `1.3`, digest-version `"3"` release. The eight
new `satisfy` links in `mbse/model/traceability.sysml` identify only the following
design responsibilities. They allocate responsibility; implementation is projected in `document-kit/`,
while BDK execution and semantic acceptance require separate evidence.

| R22 requirement | Defined design satisfier | Planned evidence |
| --- | --- | --- |
| `documentKitCatalogHasCoherentResourceLayout` | `BilingualCatalogLayout` | BDK-01, BDK-07, BDK-09 |
| `documentKitSupportsCompleteChineseAndEnglish` | `BilingualVariantLibrary` | BDK-01, BDK-03, BDK-05, BDK-06 |
| `documentTemplateLanguageSelectionIsDeterministic` | `BilingualLanguageResolver` | BDK-02 through BDK-05 |
| `documentTemplateLanguageIdentityIsPersistent` | `BilingualIdentityResolver` | BDK-02, BDK-04, BDK-05, BDK-07 |
| `documentTemplateLanguageSurfacesAreShared` | `BilingualConsumerContract` | BDK-03, BDK-05, BDK-08 |
| `chineseTemplatesUseScholarlyEngineeringProse` | `BilingualScholarlyProseContract` | BDK-03, BDK-06, BDK-09 |
| `bilingualDocumentKitPreservesCompatibility` | `BilingualCompatibilityResolver` | BDK-02, BDK-04, BDK-07, BDK-09 |
| `bilingualDocumentKitEvidenceIsReviewable` | `BilingualValidationBoundary` | BDK-01 through BDK-09 |

The [public contract](../interfaces/document-kit-api.md#i23) owns exact operation
and resource shapes and the initial English/Chinese label matrix. The
[verification design](../verification/design.md#d121) owns executable procedures
and review oracles. This section allocates the resource and software design.

### 10.1 Exact current tree and authoritative discovery

The current 1.2 root has 21 entries and digest
`a4ab976f05769bff34f3b6d3acd0de1dafb728dcf372d099fc3ac938758cf698`.
Implementation first copies its complete distribution to
`compatibility/v1.2.0/`, retaining its nested compatibility directories and exact
bytes, and only then constructs the new root. The 1.3 current-resource tree is:

```text
document-kit/
├── profile.json
├── README.md
├── RULES.md
├── SKILL.md
├── API.md
├── pyproject.toml
├── catalog/
│   ├── manifest.json
│   ├── entries/
│   │   ├── generic-requirements/
│   │   │   ├── en/      {template.md, guide.md, example.md, fixture.json}
│   │   │   └── zh-CN/   {template.md, guide.md, example.md, fixture.json}
│   │   └── <one identical role layout for each of the other 20 IDs>/
│   └── shared/
│       ├── sources.json
│       ├── routing.md
│       └── mixed/
│           ├── fixture.json
│           ├── coverage.md
│           ├── mbse/model/example.sysml
│           └── assets/{system-context.svg, interaction-flow.svg}
├── templates/
│   ├── figure-block.en.md
│   └── figure-block.zh-CN.md
├── compatibility/
│   ├── v1.0.0/                 # existing exact closure
│   ├── v1.1.0/                 # existing exact closure, including nested v1
│   └── v1.2.0/                 # exact current closure, including its nested packs
└── xdesign_doc_kit/            # existing Python package plus bilingual.py
```

The 21 directory names are exactly `generic-requirements`,
`generic-architecture`, `generic-design`, `generic-interface`,
`generic-verification`, `system-requirements`, `change-requirements`,
`system-context`, `component-composition`, `interaction-flow`, `state-machine`,
`data-model`, `operation-contract`, `data-contract`, `verification-test-plan`,
`design-review`, `generic-api-design`, `generic-flow-index`,
`generic-flow-definition`, `generic-flow-implementation`, and `generic-flow-views`.
Each contains exactly `en/` and `zh-CN/`; each variant contains exactly the four
shown role files. This yields 21 directories, 42 variants and 168 variant
resources. No current `catalog/templates/`, `catalog/guides/`,
`catalog/examples/`, or `catalog/fixtures/` tree remains.

`profile.catalog` has exactly `manifest`, `sources`, `routing_guide`,
`mixed_fixture`, and `entry_root`, with values `catalog/manifest.json`,
`catalog/shared/sources.json`, `catalog/shared/routing.md`,
`catalog/shared/mixed/fixture.json`, and `catalog/entries`. The manifest explicitly
binds every resource role and language to the conventional path above. The loader
constructs the expected path only to compare it with the declaration; it does not
enumerate a directory to discover a contract. Duplicate paths, duplicate
role/language bindings, cross-entry or cross-language paths, missing or additional
current resources, unsafe normalization and symlink components fail before target
access. A test-only filesystem walk checks the inverse coverage and is never a
runtime discovery mechanism. Shared paths referenced by several variants are read
and hashed once.

The current English resources supply the initial semantic content, not a second
editable tree. Their new 1.3 markers, fixture identity and paths change with the
release. Chinese resources are independently authored in the same entry-owned
locations. Retained files under `compatibility/` are never edited to make them
match current resources, and no symlink or generated synchronization process makes
one tree stand in for another.

### 10.2 Schema and variant data structures

`profile.json` retains the 1.2 project, route, anchor and validation policies except
where this schema explicitly changes them. Its exact bilingual additions are:

| Declaration | Required representation |
| --- | --- |
| Release | `schema_version="1.3"`, `profile_id="xdesign-sysml-docs"`, `version="1.3.0"` |
| `language_policy` | Exactly `supported_ids` and `default`; values `["en","zh-CN"]` and `"en"`. IDs are case-sensitive and ordered; the default must occur once in the list. |
| `document_classes` | The existing ten routes and `default_template_id` values. Schema 1.3 removes duplicate class-level `template` and `required_sections`; planning resolves the default entry's `en` variant. Selector kind, layer and reserved-name rules are unchanged. |
| `figure_block` | Stable field tokens, statuses, qualifiers, asset rules and `templates={"en":"templates/figure-block.en.md","zh-CN":"templates/figure-block.zh-CN.md"}`. Localized labels are variant data, not new status values. |
| `validation.managed_marker` | `<!-- document-kit: profile={profile_id}@{version} class={document_class} template={template_id} language={language} contract={contract_sha256} digest-version={contract_digest_version} -->` |
| `compatibility_packs` | Exactly the three descriptors for 1.0.0/digest-version 1, 1.1.0/digest-version 2 and 1.2.0/digest-version 2 at the roots above. |

The manifest has exactly `schema_version`, `profile_id`, `version`, and `entries`.
An entry has exactly `template_id`, `document_class`, `sources`, `checks`, and
`variants`; `variants` has exactly the two keys `en` and `zh-CN`. Source usages,
copying reviews, source records, check IDs and route semantics retain their 1.2
meaning. Each variant has exactly:

| Member | Contract |
| --- | --- |
| Selection prose | `title`, `scenario`, `when_to_use`, `when_not_to_use`, `required_inputs`, `dependencies`, with their existing nonempty string/list shapes |
| Structure | `required_sections`, an ordered array of objects with exactly `key`, `heading`, `required_fields`, and `field_labels` |
| Localized field labels | `field_labels` is an object whose keys and order equal `required_fields`; every value is a nonempty label distinct within that section. Stable field keys remain the caller/API keys. |
| View concerns | `view_concerns`, with the existing members and alternatives. `concern`, `section_key`, `applicability`, and `alternatives` are machine-equivalent across variants; only the explanatory `trigger` may be localized. |
| Four resources | `resources`, an object with exactly `template`, `guide`, `example`, and `fixture`, each the exact normalized path in section 10.1 |

The two variants of an entry must have identical `template_id`, class, ordered
section keys, ordered required field keys, checks, concern machine fields, anchor
policy and model-source rules. Headings, labels and explanatory prose may differ
only as the declared language requires. A heading retains its existing decimal or
natural form. The initial exhaustive Chinese heading/field allocation is normative
in [I23](../interfaces/document-kit-api.md#i23); an implementation may not invent a
missing label or silently reuse English. The validator compares template tokens,
headings and labels with the selected variant rather than deriving labels from
field keys.

Each isolated fixture has exactly `profile_id`, `version`,
`contract_digest_version`, `language`, `document_class`, `name`, `layer`,
`template_id`, `metadata`, `sections`, `models`, `figures`, and
`expected_document`. The first three identity values are
`xdesign-sysml-docs`, `1.3.0`, and `"3"`; `language` equals its manifest variant
and its `expected_document` is that same variant's `example.md`. The other shapes
retain 1.2 behavior, including a string `layer` only for API selections and null
elsewhere. Canonical fixture input still has no actual or sentinel contract digest
or expected-digest key.

The mixed fixture has the existing top-level `profile_id`, `version`,
`contract_digest_version`, `documents`, `coverage`, `models`, and `figures`. Each
`documents[]` item adds required `language` to the existing class/name/layer,
template, caller content and expected-example fields. Each shared figure-input
record also adds `language`, allowing two localized metadata records to reference
one language-neutral SVG `pack` path without copying the SVG. The external
acceptance manifest remains exactly `expected_contract_sha256`, `fixture_path`,
and `fixture_sha256`; language is evidenced by the fixture path and parsed fixture,
not by adding a fourth manifest field.

### 10.3 Language selection and deterministic rejection

`BilingualLanguageResolver` treats a supplied language as a selector only for new
plan/create operations and language-addressed resource reads. Omission selects the
profile default `en`; omission never consults locale, environment, filename,
content, neighboring documents or system defaults. A well-formed unsupported ID
returns the new stable `unknown-language` code. For this contract, well-formed
selector syntax is `[A-Za-z]{2,3}(?:[-_][A-Za-z0-9]{2,8})*`, so `zh`, `en-US`,
`ZH-CN`, and `zh_CN` are well-formed but unsupported. Empty or non-string values,
whitespace, non-ASCII text, repeated/leading/trailing separators, overlong pieces,
and conflicting selector channels are `invalid-request`. Only exact `en` and
`zh-CN` select resources; there is no alias normalization or fallback.

Update, attachment and selected-document check expose an optional language
**assertion**, not a reselection mechanism. Omission uses the persisted effective
variant; a supplied value must equal it. A conflicting assertion is
`invalid-request`. Whole-project/class-filter checks reject a non-null language as
`invalid-request` rather than treating it as a filter. Route selection remains the
existing three-key class/name/layer object; language never enters or changes a
route.

Under any selected original 1.0/1.1/1.2 pack, every supplied language selector or
assertion is `invalid-request`, including an unsupported ID; omission retains the
old behavior. The adapters reject a language member smuggled into a create/update/
attach payload or route-selection object instead of silently ignoring it. MCP
binding redirection is rejected before language processing. New-pack request shape
and route validation precede full pack loading; update/attachment retain the
existing ownership/readability/stale-hash precedence before effective-identity and
language-assertion checks.

A second create in another language at the same occupied route returns
`document-exists` when the destination file exists, or `registry-mismatch` when
only the registry owns that path. It never converts or overwrites the existing
page. Arbitrary caller prose is not scanned to infer or mutate language. A
prose-only edit with otherwise valid identity has no required identity-mismatch
rejection; translation adequacy is exclusively BDK-06 semantic review.

### 10.4 Marker, registry and effective identity

A schema 1.3 effective identity has exactly the six existing fields plus
`language`. New plan/create/update/attach responses include top-level `language`
and the seven-field `effective_identity`; checks expose the same per-path identity
and `scope.language` (the selected assertion or null). `route_selection` keeps its
existing three fields. Top-level selected-pack results identify `version=1.3.0`
and digest version `"3"`, while old pages retain their own six-field or v1
language-less identity.

The marker is the exact profile declaration in section 10.2 with actual nonzero
values. Registry `schema_version` remains 1 and a new entry has the existing seven
keys plus required `language`. The new loader requires language for digest-version
3 entries, forbids it for effective old entries, and compares marker, registry and
resolved variant before any write. Missing, unsupported or disagreeing persisted
language is `registry-mismatch`, not `unknown-language`. An old language-less
identity is never backfilled; a new identity may not borrow an old language-less
marker or registry entry.

When the 1.3 pack is selected, whole-project and selected checks resolve exact
1.0.0, 1.1.0, 1.2.0 and 1.3.0 entries independently. Update/attachment under the
new selected pack can therefore change an old page under its effective old
contract while preserving its marker and registry identity. Supplying a language
assertion for such an old effective page is `invalid-request`. Original packs keep
their old loader behavior: a foreign language key, digest-version 3 or new identity
may make a mixed registry/check fail as `registry-mismatch`; the design does not
weaken that rejection or silently patch retained behavior. Old positional calls
and old-page operations remain valid in their supported old-only registries, and
the new pack supplies the supported mixed-project path.

### 10.5 Localized rendering, checking and figures

`BilingualDocumentOperations` binds one `BilingualResourceVariant` to the existing
renderer and validator. The selected variant supplies exact H2 headings, metadata
labels, field labels, figure template and figure-field labels. Stable section keys,
field keys, content-boundary comments, anchors, routes, model paths, diagram
concern IDs, JSON decision keys, choice tokens, API layer values and status tokens
remain unchanged. The ASCII Markdown delimiter `:**` and metadata separators are
serialization syntax, not an English fallback. `change_fields`, section lookup,
API-layer comparison and diagram-decision lookup use declared labels instead of
hard-coded English text.

The Python `figure` input retains its existing keys `asset`, `title`, `Question`,
`Model view`, `Companions`, `Status`, `Reading`, and `Limits` for every language.
The Chinese renderer emits `图 <anchor>-<ordinal>` and labels `问题`, `模型视图`,
`配套材料`, `状态`, `解读`, and `限制`; English retains `Figure` and the original
six labels. Metadata uses `状态`, `修订`, and `模型来源` in Chinese and the
original English labels in English. Primary figure statuses and software-claim
qualifiers remain the existing machine tokens. `figure_number`, asset naming,
canonical SVG ownership, duplicate detection and six-field validation are
language-independent. Shared SVG bytes are not translated or copied; figure
captions and block prose are variant-owned. Source fidelity, readability and
translation quality remain separate judgments.

The structural checker establishes declaration/path binding, exact selected
labels/headings, complete caller fields, marker/registry identity and same-language
fixture/example references. It does not infer natural prose language or claim that
translations are equivalent. `unevaluated_checks` explicitly adds Chinese
scholarly quality and English semantic equivalence to the existing compiler,
engineering-adequacy and figure-readability limits.

### 10.6 Shared consumer and resource surfaces

The six public names and old positional parameters are unchanged. Schema 1.3 adds
keyword-only `language=None` to plan/create/update/attach and an optional language
assertion to selected check; CLI adds `--language` to the corresponding operations,
and MCP adds equivalent optional arguments. `get_contract` returns the complete
language policy, variant catalog and all four exact resource-byte kinds, with the
existing template projection explicitly using default `en` when language is
omitted. No seventh public authoring operation is introduced.

MCP retains contract/rules/catalog and both existing template URI forms, whose
omitted language is `en` under 1.3. The canonical language-addressed form is
`xdesign-doc-kit://entry/{template_id}/{language}/{resource_kind}` with
`resource_kind` exactly `template`, `guide`, `example`, or `fixture`; an additional
three-component language-addressed template form is specified in I23 for symmetry
with the retained URIs. Every read resolves only the manifest declaration and
returns exact bytes. Examples are readable examples, never selectable templates.
[I23](../interfaces/document-kit-api.md#i23) is the single exact signature, result,
resource-index, error-envelope and discovery contract for Python, JSON CLI and
real stdio MCP.

### 10.7 Complete fixtures and scholarly authoring allocation

Every one of the 42 isolated fixtures uses complete same-language metadata,
section prose, field values, model inputs and applicable figure decisions. The
acceptance comparator independently asserts actual language in stored and created
marker/registry identities, substitutes only the first marker's contract sentinel
in memory, and compares every other byte. It never treats an English expected
example as the target of a Chinese fixture.

The single mixed project has 20 documents so every class occurs in both languages
and requirements also uses two template IDs:

| Class | `en` document | `zh-CN` document and figure/alternative |
| --- | --- | --- |
| requirements | `generic-requirements` | `system-requirements` |
| architecture | `system-context`, figure | `system-context`, reasoned table |
| design | `interaction-flow`, figure | `interaction-flow`, reasoned prose |
| interface | `operation-contract` | `operation-contract` |
| verification | `verification-test-plan` | `verification-test-plan` |
| api-design | `generic-api-design` | `generic-api-design` |
| flow-index | `generic-flow-index` | `generic-flow-index` |
| flow-definition | `generic-flow-definition` | `generic-flow-definition` |
| flow-implementation | `generic-flow-implementation` | `generic-flow-implementation` |
| flow-views | `generic-flow-views` | `generic-flow-views` |

`catalog/shared/mixed/coverage.md` maps these selections and reasoned omissions,
including the entries not instantiated in the mixed root. This retains expected
and conditional concerns, figure/table/prose alternatives and two IDs in one
original class without requiring all 42 variants in the mixed project; BDK-03
requires all 42 independently.

The future Chinese author is the Kimi runner selected by the wish. The author
works entry by entry from the English engineering obligations, uses the I23
terminology allocation, and applies D121's class-specific rubric to template
instructions, guides, examples and fixture prose. Chinese text states the problem
and boundary, justifies decisions, explains method/design and evidence, and names
limitations with precise, objective, connected prose. It does not replace classes
with thesis chapters, add unverifiable citations or upgrade fictional/planned
claims into implementation or review results. English resources retain equivalent
normative meaning without literal back-translation.

A checker independent of the author reviews every section and field of all 21
pairs, records per-entry terminology/normative maps and citation resolutions, and
corrects findings before acceptance. Mechanical same-language and placeholder
checks support but cannot replace that review. No deterministic test compares
Chinese and English prose by mirrored wording.

### 10.8 Source and verification projection

The allocation follows the actual schema-1.2 extension points rather than
introducing a parallel engine:

| Design responsibility | Concrete source or verification owner |
| --- | --- |
| `BilingualCatalogLayout`, `BilingualVariantLibrary` | `profile.json`, `catalog/manifest.json`, entry-owned resources, `catalog/shared/`; schema/closure loading in `xdesign_doc_kit/catalog.py`; variant selection and resource indexing in new `xdesign_doc_kit/bilingual.py` |
| `BilingualLanguageResolver` | `bilingual.py` selector/default/resource functions; `structured.py` plan/preflight and operation integration; unchanged route grammar in `routes.py` |
| `BilingualIdentityResolver` | `compatibility.py` identity/marker/registry/effective resolution; `core.py` rendering values and registry loading; `structured.py` assertions |
| Localized renderer/check/figures | `core.py`, `structured.py`, `catalog.py`, and the two declared figure-block templates |
| `BilingualCompatibilityResolver` | `compatibility.py` digest-version 3 and recursive resolution, `catalog.py` descriptors/closure/sentinel validation, exact `compatibility/v1.0.0/`, `v1.1.0/`, `v1.2.0/` |
| `BilingualConsumerContract` | unchanged exports in `__init__.py`; `__main__.py` JSON CLI; `mcp_server.py` bound tools/resources; distributed `README.md`, `RULES.md`, `SKILL.md`, `API.md`; `pyproject.toml` remains the Python package manifest with no new runtime dependency |
| `BilingualValidationBoundary` | new `tests/test_bilingual.py`, `test_bilingual_transports.py`, `test_bilingual_compatibility.py`; affected existing catalog/core/security/example/compatibility/structured/acceptance/MCP suites; [D121](../verification/design.md#d121) |
| `BilingualScholarlyProseContract` | all 21 `zh-CN` and `en` variant sets plus independent revision-bound per-entry review evidence, not a production prose classifier |

The supported distribution remains the complete copied/pinned directory selected
by `profile_root`, including Python source, consumer documentation, all declared
current resources and every retained closure. Package installation does not move
that selected resource root into `importlib.resources` or create another editable
copy. Implementation adds `document-kit/MANIFEST.in` as the reviewed
source-distribution file list so a source archive contains the same complete
closure; BDK-08 installs/copies that distribution into
a fresh consumer and recomputes its digest before use. No network retrieval or new
MCP dependency is needed beyond the existing extra.

### 10.9 Release sequence and limits

Implementation is bounded into: (1) freeze and verify the 1.2 closure; (2) copy the
three immutable old closures and create entry-owned English paths; (3) implement
schema/variant/identity and adapter support; (4) author/review all Chinese pairs;
(5) build isolated/mixed fixtures and run all BDK/DOC/SDK evidence. The evidence
implementation plan decomposes these into smaller reviewable units and assigns
acceptance oracles to each file.

The 1.3 contract digest is deliberately unspecified until those exact product
bytes exist. The design has not moved a catalog file, edited a product resource,
implemented `bilingual.py`, executed a product BDK case or obtained the 21
independent prose judgments. Existing SimpleFlow figures, gates and class/name/
layer routes are unchanged; no locale inference, automatic migration, remote
retrieval, parallel editable catalog, new diagram or flow/controller change is
introduced.
