# Portable document kit interaction contract

<a id="i20"></a>

## 1. Scope, selected project, and schemas

This is the cross-boundary contract for the [document kit engine](../design/document-kit.md#d113).
A caller selects one target project root and one versioned profile directory.
The CLI selects that pair per invocation; an MCP process is bound to it at launch
and tool arguments cannot redirect it. Every path is local, POSIX, and relative
to the selected project or pack. Absolute paths, `.` or `..` components,
unexpected separators, and symlink escapes are errors.

The engine accepts schema `1.0` and `1.1` and dispatches behavior by schema, not
by a caller's preferred interpretation. Unknown schemas fail `invalid-profile`.
The root 1.1 pack is `xdesign-sysml-docs` version `1.1.0` with digest algorithm
version `"2"`. A byte-exact v1 compatibility pack lives at
`compatibility/v1.0.0/` and uses schema `1.0` and digest version `"1"`. Later
releases have new versions and retain earlier 1.1 packs at selectable roots.
Derived profiles use their own identity and resource closure rather than the
bundled inventory or source-coverage rule.

`get_contract` and `plan_document` remain available before model sources are
authored. Create and check require configured model roots with at least one
readable `.sysml` file. Rules, templates, catalog, source, fixture, and
compatibility resources must be normalized readable files beneath the selected
pack. Conflicting roles, duplicate JSON keys, malformed UTF-8/JSON, and unsafe
resources fail before target writes.

## 2. Digest and resource closure

Digest version `"1"` preserves the existing algorithm: sort `profile.json`, the
declared rules file, every declared class template, and the figure-block template
bytewise by UTF-8 path. For each, feed the path, NUL, decimal byte length, NUL,
then exact bytes to one SHA-256 hasher. The bundled v1 digest is
`1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6`.

Digest version `"2"` initializes SHA-256 with
`xdesign-doc-kit-contract\0` + ASCII `"2"` + NUL. Sort unique normalized
pack-relative paths bytewise by UTF-8 and feed path, NUL, decimal length, NUL,
and bytes for each. The closure includes:

- `profile.json`, `RULES.md`, `README.md`, and `SKILL.md`;
- catalog manifest, source register, routing guide, shared figure block, and all
  declared templates, guides, examples, and caller-input fixtures;
- mixed fixture, coverage guide, model stub, and all declared figure assets;
- every compatibility descriptor and the complete declared resource closure of
  each compatibility pack, recursively using that pack's schema.

Duplicate references to one path are hashed once; conflicting roles or unreadable
bytes fail. There is no directory-glob or network inclusion. A byte change in any
covered resource changes the digest. Every 1.1 release is immutable and a changed
release has a new version, so corrections cannot silently redefine old documents.
MCP pins the selected closure, including compatibility bytes, at launch and
refuses drift until restarted; CLI operations revalidate it before writes.

Every result includes top-level `profile_id`, `version`, `contract_sha256`, and
`contract_digest_version`. The last is the string `"1"` for schema 1.0 and `"2"`
for schema 1.1. Per-document effective identity is defined in section 4.

## 3. Catalog, manifest, and source records

`profile.json` has `schema_version`, `profile_id`, `version`, `rules_file`,
`skill_file`, class routes, project/model roots, anchor/figure/validation policy,
`catalog`, and `compatibility_packs`. Each class has exactly one
`default_template_id`; zero or multiple defaults fail with `template-default`.
The generic `template` resource matches the generic manifest entry's template.

`catalog.manifest`, `catalog.sources`, `catalog.routing_guide`, and
`catalog.mixed_fixture` identify the machine-readable catalog inputs. Each
`compatibility_packs` entry identifies profile, version, root, expected contract
digest, and digest version. The engine validates every descriptor before using a
compatibility pack.

A schema 1.1 manifest contains `schema_version`, `profile_id`, `version`, and
`entries`. Every entry has:

- unique lowercase kebab-case `template_id` and a declared `document_class`;
- nonempty `title`, `scenario`, `when_to_use`, `when_not_to_use`;
- `required_inputs`, `dependencies`, and `required_sections` using the existing
  key/heading/required-fields contract;
- normalized `template`, `guide`, `example`, and `fixture` resource paths;
- a nonempty `sources` list and nonempty implemented `checks` list;
- `view_concerns`, possibly empty, each with `concern`, `section_key`,
  `applicability`, `trigger`, and allowed `alternatives`.

`applicability` is `expected`, `conditional`, or `not-applicable`; alternatives
are `figure`, `table`, `prose`, and `not-applicable`. A conditional concern has a
nonempty trigger. Class or template identity alone never mandates a figure.
Invalid IDs, duplicate entries/section keys/field tokens, unsafe or unreadable
resources, unknown checks, and the wrong bundled 16-entry inventory fail
`invalid-catalog`.

A source record has `source_id`, `title`, `publisher`, `retrieved_at`,
`ideas_used`, `limitations`, exactly one primary `url` or `local_path` (the other
`null`), `additional_urls`, `license_name`, `license_version`, and `license_url`.
Source-record item types are data projections only; optional multiplicities are
not constraint declarations. `DocumentCatalogValidator` alone enforces their
invariants. License name/version may be `unknown` or `unlicensed`; `license_url`
is then `null`. Otherwise it is an absolute HTTP(S) URL. The field exists, so
explicit null differs from missing. Sentinel strings are not valid URL values.

Each entry-source pair has `source_id`, `copying_decision` (`not-copied` or
`copied`), and `copying_review`. It is `null` for `not-copied`; for `copied` it
contains `reviewer`, `reviewed_at`, `permission`, and `evidence`. Presence is a
structural check, not permission adequacy. Every bundled research ID is cited by
at least one scenario, and every entry cites at least one known or project-local
normative source.

Schema 1.0 `get_contract.catalog` is a synthesized compatibility projection:
`template_id` and scenario-only fields are `null`, and the source register names
only the actual selected pack. It is exempt from the 1.1 manifest/guide/example/
source package and has no independently selectable IDs. Supplying `template_id`
for a 1.0 plan/create operation is `unknown-template` before writing.

## 4. Effective identity and compatibility routing

`effective_identity` contains exactly `profile_id`, `version`, `document_class`,
`template_id`, `contract_sha256`, and `contract_digest_version`. For every
existing or newly created schema 1.0 document, `template_id` is `null` and digest
version is `"1"`.

A new 1.1 marker has this form, with values substituted:

```text
<!-- document-kit: profile={profile_id}@{version} class={document_class} template={template_id} contract={contract_sha256} digest-version={contract_digest_version} -->
```

A 1.1 registry entry has `path`, `class`, `profile_id`, `version`,
`contract_sha256`, `template_id`, and `contract_digest_version`. Registry
`schema_version` remains 1, with the final two as additive optional keys for old
readers. No document hash is stored in the registry, so valid updates preserve
registry bytes while the authorized document hash changes. Schema 1.0 creation
keeps the original v1 marker and registry shape; response-only legacy fields are
not backfilled.

The resolver selects the effective contract before judging identity:

1. An explicitly selected original v1 or 1.1 pack is used when the document's own
   marker and registry agree with its computed identity. Custom v1 packs need no
   allowlist.
2. Under a 1.1 catalog, only exact original bundled
   `xdesign-sysml-docs@1.0.0` marker and registry matching the known v1 digest
   automatically selects `compatibility/v1.0.0/`. Missing legacy fields mean
   `template_id=null` and digest version `"1"`, not a mismatch or rewrite.
3. Unknown/custom v1 identities fail `unsupported-legacy-identity` under the 1.1
   route and require their original pack. A document from another 1.1
   version/digest fails `registry-mismatch` and requires its original pack.
   Automatic cross-1.1-release compatibility and in-place migration are excluded.
4. Missing evidence or a class/template/profile/version/digest disagreement fails
   `registry-mismatch` before writes.

A recognized legacy difference from the selected catalog is not an own-identity
mismatch. A mismatch among the document's own marker, registry, recorded
template, and resolved contract is a mismatch. Valid updates preserve effective
identity and registry bytes. Unmanaged pages are not registered, reclassified,
rewritten, or forced into a template; update/attach refuses them.

## 5. Shared operations

Both adapters invoke the same core operations. Existing positional callers are
preserved by appending optional `template_id` only to plan/create.

| Operation | Input | Successful additive result |
| --- | --- | --- |
| `get_contract(profile_root=None)` | Optional profile root. | Existing profile/rules/templates plus `catalog`, source records, and `contract_digest_version`. |
| `plan_document(target_root, document_class, name, profile_root=None, template_id=None)` | Class, semantic slug, optional 1.1 template ID. | Existing path/exists/hash/required inputs plus `template_id` and `effective_identity`. No write. |
| `create_document(target_root, document_class, name, metadata, sections, profile_root=None, expected_contract_sha256=None, template_id=None)` | Class, slug, caller content, optional 1.1 template ID and expected selected digest. | New path, document hash, anchors, `template_id`, `effective_identity`, and `semantic_review_required=true`. Existing files are refused. |
| `update_section(..., fields=None, profile_root=None, expected_contract_sha256=None)` | Existing class/slug/section/content/expected document hash; optional fields. | Changed path/hash and `effective_identity`; other sections and registry bytes remain. |
| `attach_figure(..., figure, expected_sha256, profile_root=None, expected_contract_sha256=None, concern=None)` | Existing class/slug/section/figure/expected document hash; optional concern only with no view concerns, otherwise exactly one declared concern. | Changed path/hash, local figure number, `effective_identity`, and provenance review notes. |
| `check_documents(target_root, document_class=None, profile_root=None)` | Optional class filter and selected pack. | Existing paths/hashes/diagnostics plus `effective_identity_by_path` and `semantic_review_required=true`; per-path maps may be empty when no managed document is selected. |

`metadata` retains nonempty `title`, `status`, `revision`, and `model_sources`.
Section content and required labeled fields remain caller-supplied. For 1.1,
a section with view concerns also has a `diagram_decisions` field containing a
JSON array. Each object has `concern`, `applies` (Boolean), `choice`, `rationale`,
`asset`, and `model_view`.

Expected concerns require an explicit decision; conditional concerns require an
explicit applicability Boolean. `choice` is `figure`, `table`, `prose`, or
`not-applicable` and must be allowed by the manifest. Figure requires
`applies=true`, a semantic `assets/<name>.svg` asset, model view, rationale, and
conforming figure block. Other choices require a rationale and no asset.
`attach_figure` appends optional `concern` after `expected_contract_sha256`; omission is allowed
only with no view concerns. Otherwise exactly one declared concern selects exactly one decision,
never an implicit first decision. An omitted required concern is `missing-diagram-decision`; an
unknown concern is `invalid-diagram-decision`. Attachment to a table/prose/not-applicable
decision always fails `invalid-diagram-decision` before writing, even when selected correctly,
because that decision carries no asset. Missing or malformed decisions are deterministic failures;
adequacy is semantic review.

## 6. Checks, writes, security, and diagnostics

Managed validation resolves each document before checking its route, marker,
registry, exact ordered headings, required fields/content, anchors, placeholders,
links, model locators, and figure rules. Existing unmanaged pages keep their
established project checks and are never adopted implicitly. Failed updates leave
existing document bytes unchanged. Update/attach recheck expected document
SHA-256 immediately before replacement. Create removes only a newly created page
if registry publication fails; an interrupted page/registry pair is reported
without claiming two-file atomicity.

Resource reads use normalized no-follow paths and regular-file checks. External
source and non-null license URLs must be absolute `http` or `https` URLs with a
network location and no credentials or control characters. `javascript:`,
`file:`, `data:`, protocol-relative, backslash, and unsafe Markdown destinations
are rejected. Raw HTML is limited to exact managed `<!-- document-kit:... -->`
marker/content comments and `<a id="..."></a>` anchors. Scripts, event handlers,
other HTML, executable includes, network retrieval, and code execution/import/
evaluation are forbidden.

An unsuccessful operation returns `ok=false`, an `error` with stable `code` and
`message`, and diagnostics. Every diagnostic has exactly `rule`, `code`, `file`,
`line`, and `detail`; `file` is project-relative or empty, and `line` is a
positive integer or null. Existing codes retain their meanings. The complete
vocabulary of operation-error and structural-finding codes used by this contract
and its v1 compatibility behavior is:

| Code | Condition |
| --- | --- |
| `atomic-replace-failed` | Atomic document/registry replacement fails at the filesystem boundary; no silent cross-filesystem or partial-write fallback is claimed. |
| `contract-drift` | With actual recomputed `D` nonzero and not `Z`, selected/effective contract bytes or a compatibility descriptor disagree with `D`, including descriptor `contract_sha256=Z`. Recomputed `D == Z` is always `invalid-catalog` before any drift comparison. |
| `copying-review-required` | `copied` lacks reviewer/date/permission/evidence. |
| `document-exists` | Creation's destination document already exists and cannot be overwritten without an explicit update operation. |
| `duplicate-anchor` | A managed anchor ID is repeated where it must be unique. |
| `duplicate-figure` | A figure asset or figure number is repeated within its uniqueness scope. |
| `empty-content` | Required section prose is empty or its content boundaries are missing, duplicated, or reversed. |
| `figure-block` | A figure's embed, adjacent block, section/figure label, or ordered required fields do not agree. |
| `figure-status` | The primary figure status is outside the selected profile's vocabulary. |
| `fixture-mismatch` | Acceptance example inputs, created bytes, model/conditional SVGs, manifest shape/hash binding, or mixed coverage differ after request, pack, and target checks. |
| `invalid-catalog` | Schema 1.1 catalog manifest identity/version or entry/declared-list/resource/check-ID shape is malformed, bundled inventory is invalid, a selected-pack sentinel defect exists, or recomputed `D == Z`, even when a descriptor also claims `Z`. Generic default errors are `template-default`; profile schema errors are `invalid-profile`. |
| `invalid-diagram-decision` | Invalid concern/applicability/choice/asset/model-view/figure agreement, including an unknown concern or attachment to a non-figure decision. |
| `invalid-docfigure` | A supported `DocFigure` locator's target document is absent, or its named figure is absent from or not embedded in the declared document. Independent document-asset validation may also report `missing-figure`. |
| `invalid-docsection` | A supported `DocSection` locator names a missing document or section anchor. |
| `invalid-document-name` | The caller's document slug is empty or violates the class filename pattern. |
| `invalid-license` | All license fields exist, but their identity/nullability combination is invalid: a known identity has null `license_url`, or `unknown`/`unlicensed` identity has a non-null URL. Missing fields are `invalid-source`; a present unsafe URL is `unsafe-url`. |
| `invalid-link` | A safe local Markdown destination or fragment does not resolve to an existing document/anchor. |
| `invalid-profile` | Unsupported profile `schema_version`, malformed profile/policy/template/resource closure, invalid model-root configuration (shape, duplicates, unsafe declaration, or no required root), malformed compatibility descriptor, or invalid profile-managed route/token. Catalog manifest schema is `invalid-catalog`; a configured root's absence/readability is `missing-model-root` or `unreadable-input` as applicable. |
| `invalid-request` | Malformed caller request, including invalid JSON/arguments, unknown document class or section, malformed expected digest, or `expected_contract_sha256=Z`. Schema 1.0 template selection is instead `unknown-template`. |
| `invalid-source` | A source record/usage field set or required identity text is invalid (including missing `license_name`, `license_version`, or `license_url`), a source ID is unknown/duplicated, primary `url`/`local_path` cardinality is wrong, or copying decision/usage shape is invalid. Present unsafe URLs are `unsafe-url`; license nullability is `invalid-license`; missing copied-review evidence is `copying-review-required`. |
| `missing-anchor` | A required section anchor is absent or violates the profile's anchor format/placement. |
| `missing-diagram-decision` | Attachment or validation has no selected decision for an applicable concern, including an omitted selector or absent decision. |
| `missing-document` | A required target document or parent file is absent during project-file access or safe write preparation. |
| `missing-field` | Required section fields are missing, empty, or do not match the selected template's field set. |
| `missing-figure` | A required canonical SVG asset is absent during document figure validation, independently of a model locator's `invalid-docfigure` result. |
| `missing-metadata` | Required title, status, revision, or `model_sources` metadata is absent or malformed. |
| `missing-model-root` | A well-formed configured required root is absent, is not a directory, or has no readable `.sysml` file after a successful scan; absent optional roots are ignored. Configuration errors are `invalid-profile`; scan I/O errors are `unreadable-input`. |
| `missing-model-source` | A declared model source is missing, duplicated, malformed, outside the configured roots, or not rendered as required path code spans. |
| `registry-mismatch` | Registry/marker/document ownership or effective identity disagrees, including target sentinel identity, copied examples, forged registries, unmanaged updates, or a document from another 1.1 release. |
| `section-headings` | The H1 or exact required ordered H2 chapter structure is missing, duplicated, reordered, or otherwise differs. |
| `stale-document` | After registry ownership and document readability are established, the current document hash differs from the caller's expected SHA-256 before effective-contract resolution and an update or figure commit. |
| `template-class-mismatch` | The selected entry belongs to another document class. |
| `template-default` | A class has zero, multiple, unknown, non-generic, or class-mismatched generic defaults. |
| `unknown-template` | Unknown or malformed schema 1.1 ID, or any supplied `template_id` under schema 1.0, which has no selectable IDs. |
| `unreadable-input` | An existing project/target/model input cannot be inspected, opened as a regular file, scanned, or decoded as required. Absent target files are `missing-document`; configured root absence/no-source is `missing-model-root`; profile/catalog readers retain their specific codes. |
| `unresolved-placeholder` | A profile-forbidden unresolved template placeholder remains in managed content. |
| `unsafe-markdown` | Disallowed raw HTML, script/event handlers, or executable include in catalog or content Markdown. URL safety is `unsafe-url`; a safe but broken local link is `invalid-link`. |
| `unsafe-path` | Absolute/traversing/non-normalized local path, symlink escape or component, or unsafe target/model-source/figure/local destination outside a more specific configuration or annotation wrapper. |
| `unsafe-url` | A present source, additional, license, or Markdown URL is empty, has controls, is non-absolute, has a forbidden scheme, credentials, backslashes, or is protocol-relative. Source field shape is `invalid-source`; license nullability is `invalid-license`; local path safety is `unsafe-path`. |
| `unsupported-annotation` | A `DocSection`/`DocFigure` annotation form or field set is unsupported/malformed, or its locator path is unsafe in annotation context. Missing locator targets are `invalid-docsection`/`invalid-docfigure`; other unsafe local paths are `unsafe-path`. |
| `unsupported-legacy-identity` | Unknown/custom v1 identity under a 1.1 catalog or unavailable bundled-v1 compatibility resources; select the original valid pack. A different 1.1 release is instead `registry-mismatch`. |

Request shape and route selection precede profile/catalog validation; selected-pack
validation and digest integrity precede target identity resolution; document,
model, and figure checks precede acceptance-fixture comparison. Combined defects
at one stage use these deterministic rules:

- Profile `schema_version` is evaluated before catalog manifest schema. Model-root
  configuration shape/duplicates/unsafe declarations are `invalid-profile` before
  existence checks; a well-formed missing/source-less required root is
  `missing-model-root`, a root scan I/O failure is `unreadable-input`, and a
  symlink in the discovered tree is `unsafe-path`.
- Source record field-set/identity and primary-locator checks precede primary and
  additional URL/local-path checks, which precede license nullability and license
  URL checks. Missing license fields are therefore `invalid-source`; present
  unsafe URLs are `unsafe-url`; only a complete record's invalid nullability
  combination is `invalid-license`. Other declared catalog list-shape defects are
  `invalid-catalog`, not a reason to broaden `invalid-source`.
- Update/attach checks request shape, registry ownership, document readability,
  expected hash freshness, then effective-contract identity. Creation refuses an
  existing destination before reporting a duplicate registry owner.
- Annotation wrappers report `unsupported-annotation` for unsafe locator paths.
  `invalid-docfigure` belongs to model-locator validation, while `missing-figure`
  belongs to document-asset validation; one missing asset can legitimately produce
  both independent findings. Independent document/figure findings are collected
  and sorted by `(file, line, code, detail)`; that reporting order is not a
  semantic priority or a pass/fail shortcut.

Schema 1.0 selection remains `unknown-template`, recomputed `D == Z` remains
`invalid-catalog` before drift, a different 1.1 release remains
`registry-mismatch`, and unknown/custom v1 identity remains
`unsupported-legacy-identity`. `DOC-001` through `DOC-010` and `input` are
diagnostic rule labels, not additional diagnostic codes; CLI exit statuses and
MCP transport flags are not new codes either.

`check_documents` returns `ok=false` when findings exist. CLI emits one JSON
object and exits 0 on success/pass, 1 on rejected operation/failed check, and 2
on invalid invocation. MCP tool errors carry the same core code and diagnostics;
the transport error flag is not a semantic assessment. Structural reports include
`semantic_review_required=true` and explicit unevaluated checks.

## 7. CLI and MCP resources

The CLI adds optional `--template-id` to `plan-document` and `create-document` and
optional `--concern` to `attach-figure`. MCP adds optional `template_id` to the
same plan/create tools and optional `concern` to `attach_figure` with identical
validation. Update and check have no template selector because registry identity
determines the entry. Attach's concern selects a view concern, not a template.
DOC-12 compares valid, omitted, and mismatched concern results across both adapters.

MCP exposes existing tools and these resources:

| Resource | Contract |
| --- | --- |
| `xdesign-doc-kit://contract` | Complete `get_contract` JSON including catalog and digest version. |
| `xdesign-doc-kit://rules` | Rules text. |
| `xdesign-doc-kit://catalog` | New JSON catalog and source-register object identical to CLI `get_contract.catalog`. |
| `xdesign-doc-kit://template/{document_class}` | Existing class template: original v1 template under v1, generic default under 1.1. |
| `xdesign-doc-kit://template/{document_class}/{template_id}` | New exact Markdown for one selected entry; unknown or class-mismatched IDs are not found. |

The catalog is a fixed resource and both template forms are resource templates.
CLI `get_contract` exposes the same catalog object, source records, and exact
template bytes. MCP rejects root/profile selector arguments and checks selected
and compatibility digests on every call. No network resource is introduced.

## 8. Evidence and acceptance boundary

Consumers pin exact pack bytes by contract digest for an assessed wish and review
changed digests before reusing a result. Evidence identifies assessed revision,
effective identities/digests/versions, commands/transcripts, fixtures,
before/after hashes, diagnostics, and unevaluated checks. Scenario fitness,
source/attribution/license adequacy, coverage, diagram decisions, model meaning,
and final figure quality remain separate judgments. CLI/MCP results cannot
authorize a stage gate by themselves.

This interface is the design basis. Source and executed evidence, not this page's
existence, establish availability. [DOC-01 through DOC-15](../verification/design.md#d114)
define regression and scenario-catalog acceptance cases, including isolated and
mixed fixtures, original-pack compatibility, and independent semantic review.

## 9. Stored examples, fixtures, and acceptance evidence

### 9.1 Stored sentinel and actual digest

For every schema 1.1 stored example `E`, `Z` is exactly 64 lowercase ASCII zero
characters and `D` is the actual selected contract digest. The first line is the
exact managed marker from section 4 with actual non-contract identity,
`contract=Z`, and `digest-version=2`. `Z` occurs exactly once in all of `E` and
only in that field; `D` occurs nowhere in `E`. `Z` is invalid in every other pack
resource, including the bundled pack manifest, fixtures, mixed resources, model
stub, figures, and compatibility packs. Missing, duplicate, misplaced, malformed,
uppercase, prefixed, alternate-length, actual-digest, or unknown contract values
are selected-pack defects. Schema 1.0 contains no stored examples or sentinel and
keeps its existing marker/registry and digest behavior; derived 1.1 packs apply
the same rule to all declared examples.

Version 2 hashes the exact sentinel-bearing example bytes without stripping,
zeroing, canonicalization, or any other normalization. Recomputed `D == Z` is
`invalid-catalog`, even if a descriptor also claims `Z`; a descriptor claiming `Z`
with nonzero recomputation is `contract-drift`; caller
`expected_contract_sha256=Z` is `invalid-request` before catalog or target writes.
All selected, created, returned, and MCP-pinned identities use actual nonzero `D`.
Examples are not templates, and no catalog/template resource returns example bytes
as a selected contract. A test-only zero-returning digest provider is confined to
the catalog-loader/digest-strategy test seam and is unreachable through product
pack inputs, production SHA-256, CLI, or MCP.

### 9.2 Canonical full-input fixtures

Each isolated `catalog/fixtures/<template-id>.json` is the sole canonical caller-
input source for its acceptance creation. Its shape is:

| Key | Contract |
| --- | --- |
| `profile_id`, `version`, `contract_digest_version` | Selected catalog identity from which the test recomputes `D`; digest version is the string `"2"`. |
| `document_class`, `name`, `template_id` | Existing plan/create route and explicit entry selection. |
| `metadata`, `sections` | Complete existing create input, including model sources, every caller section/field, and diagram decisions. |
| `models` | Array of every required `{pack,target}` model input; the fictional model stub is parser-valid. |
| `figures` | Array of every required conditional figure input and its figure-block metadata; figure-free examples use `[]`. |
| `expected_document` | Pack-relative path of the entry's stored example. |

The fixture contains no `D`, `Z`, `contract_sha256`, or
`expected_contract_sha256`. `catalog/mixed/fixture.json` carries the same selected
identity at top level plus complete `documents`, `coverage`, `models`, and
`figures` inputs. Each `documents[]` item retains its full plan/create input and
an `expected_document` reference to one of the same 16 stored examples. The mixed
fixture introduces no independent example bytes, sentinel form, or requirement to
instantiate all 16 entries. It does instantiate all five classes and at least two
template IDs in one class and retains its concern-to-template/omission coverage.

### 9.3 External manifest and exact full-byte comparison

Before creation, the acceptance test materializes one external manifest per
isolated or mixed fixture beneath `cache/document-template-library-acceptance/`.
The path is outside the selected pack and digest closure. The manifest contains
exactly:

```json
{
  "expected_contract_sha256": "actual recomputed D",
  "fixture_path": "pack-relative canonical fixture path",
  "fixture_sha256": "SHA-256 of those exact fixture bytes"
}
```

The strings in this example describe values, not literal sentinel alternatives.
The manifest has no caller-input copy and no `manifest_sha256`, `self_sha256`, or
equivalent self/own-hash field. Such a field is `fixture-mismatch` before creation;
the test does not exclude or normalize it. Separate S-12/S-16 evidence computes
and retains the manifest's SHA-256, actual `D`, and fixture references. The hash is
never read from the manifest. Recomputed digest, fixture path/hash, or retained
manifest-hash drift is a stale-fixture failure and never silently updates any
fixture, example, marker, registry, or manifest.

For stored bytes `E` and created bytes `C`, the test-only comparator:

1. Recomputes nonzero `D`, validates the selected pack and exact fixture/manifest
   binding, and rejects defects before target writes.
2. Validates `E` as the manifest-named stored example, including its exact first
   marker, total-file sentinel count, and absence of `D`.
3. Independently parses and asserts `C`'s actual marker and registry path,
   profile/version, class, template, `D`, and digest version before comparison.
4. Copies `E` only in memory and replaces only the parsed first-line `contract=`
   value span `Z` with `D`, producing `E'`; it does not globally replace either
   value or change any other byte.
5. Asserts the created marker equals the `E'` marker and compares all `E'` bytes
   with all `C` bytes, including whitespace, line endings, and trailing newline.
6. Runs `check_documents` against unchanged `C` and registry, requires the declared
   structural predicates to pass, and asserts stored, created, registry, and
   manifest before/after hashes in evidence.
7. Repeats the same algorithm independently for every mixed `expected_document`.

Marker stripping, body-only comparison, whitespace or line-ending normalization,
other-field normalization, and global replacement are unapproved and fail crafted
negative fixtures as `fixture-mismatch`. Product plan/create/update/attach/check
and digest computation never perform the acceptance substitution.

### 9.4 Diagnostic precedence and preservation

Evaluate request shape, selected pack, target identity, then fixture comparison.
`expected_contract_sha256=Z` is `invalid-request`; selected-pack sentinel defects
and recomputed zero are `invalid-catalog`; a zero descriptor with nonzero actual
recomputation is `contract-drift`; a target marker or registry containing `Z`, a
copied example, or a forged target registry is `registry-mismatch`; only a later
byte discrepancy is `fixture-mismatch`. Copied examples are not adopted or
registered, and pack validation is not misapplied to target identities. Every
negative case leaves target documents and registry bytes unchanged.

Original custom v1 packs still work by explicit selection without a known-digest
allowlist. Exact bundled-v1 routing under 1.1 precedes own-identity mismatch
checks, and original 1.1 packs remain explicitly selectable after a later release.
A different 1.1 release, unknown identity, unmanaged-page update, stale fixture,
or unapproved comparison fails closed without migration or silent refresh.
