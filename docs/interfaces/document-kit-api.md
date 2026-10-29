# Portable document kit interaction contract

<a id="i20"></a>

## 1. Scope and selected project

This is the planned cross-boundary contract for the
[document kit engine](../design/document-kit.md#d113). A caller selects one
project root and one versioned profile directory. The profile directory
contains `profile.json`, its declared `RULES.md`, and all declared Markdown
templates. The engine reads these bytes as one contract and reports a SHA-256
digest. Sort the unique relative paths of `profile.json`, the declared
rules file, every declared document template, and the figure-block template
in bytewise UTF-8 order. Feed one SHA-256 hasher for each file with its UTF-8
path, a NUL byte, its decimal byte length, a NUL byte, then its exact bytes.
A tool response identifies
`profile_id`, `version`, and `contract_sha256`. An MCP process is bound to
one selected project/profile at launch; tool arguments cannot redirect it to
a different root or contract. The CLI selects the same pair per invocation.

Paths in the profile and generated documents are POSIX, project-relative
paths. The engine rejects absolute paths, `.` or `..` components, unexpected
separators, and symlink escapes. A class route must resolve beneath the
profile's `docs_root`. The `{name}` value is a semantic lowercase kebab-case
stem. The profile's route and exact section list determine the destination;
the caller cannot supply an arbitrary output path.
The rules file and template paths must also be relative, normalized, and
contained within the selected profile directory. Absolute paths, traversal,
symlink escapes, duplicate resource paths with conflicting roles, and
unreadable files invalidate the contract before any operation.

`project.model_roots` are required SysML source directories; each must
exist, and the combined required roots must contain at least one readable
`.sysml` file. `project.optional_model_roots` are scanned when present and
do not fail merely because the directory is absent. A present but unreadable
optional root is an error. The xdesign profile requires `mbse/model` and
optionally includes legacy/bootstrap `mbse/wishes`.
`get_contract` and `plan_document` remain available before those model
files are authored; `create_document` and `check_documents` enforce the
model-source requirement.

## 2. Shared operations

Both adapters invoke the same core operations. JSON object keys below are
normative for CLI input files and MCP tool arguments; omitted optional keys
are noted explicitly.

| Operation | Input | Successful result |
| --- | --- | --- |
| `get_contract` | No document input. | Profile identity/digest, profile data, rule text, and templates. |
| `plan_document` | `document_class`, `name`. | Project-relative `path`, `exists`, current `sha256` if present, required metadata, ordered sections/fields, and contract identity. No write. |
| `create_document` | `document_class`, `name`, `metadata`, `sections`. | New `path`, document `sha256`, anchors, contract identity, and `semantic_review_required=true`. Existing files are refused. |
| `update_section` | `document_class`, `name`, `section_key`, `content`, required `expected_sha256`, optional `fields`. | Changed path and `sha256`; all other sections and anchors remain. |
| `attach_figure` | `document_class`, `name`, `section_key`, `figure`, required `expected_sha256`. | Changed path and `sha256`, local figure number, and any provenance review notes. |
| `check_documents` | Optional `document_class`; absence means all classes. | `ok`, checked paths, per-path SHA-256, deterministic diagnostics, contract identity, and `semantic_review_required=true`. |

`metadata` has nonempty `title`, `status`, `revision`, and
`model_sources`; the last is a nonempty list of readable project-relative
`.sysml` paths beneath a configured model root. Reject duplicates and
backticks in these paths; preserve input order and render the template's
single `{{model_sources}}` token as comma-separated Markdown code spans:

```markdown
> Model sources: `mbse/model/a.sysml`, `mbse/model/b.sysml`
```

`sections` is keyed by every required section key in the chosen class,
with no missing or foreign key. Each value has a caller-allocated `anchor`,
a nonempty `content` explanation, and `fields` containing every named
required field with nonempty text. The template fixes the H1, ordered H2
headings, labels, anchor placement, and managed marker. The writer must not
fill absent engineering claims with generic prose.

`figure` has `asset` (a semantic `assets/<name>.svg` path beneath the
carrying page), `title`, and nonempty `Question`, `Model view`,
`Companions`, `Status`, `Reading`, and `Limits` values. The canonical
SVG must already exist; attachment adds one embed and adjacent block in the
named anchored section. The status begins with a value allowed by the
profile. A repeated asset or figure number is refused. Attachment does not
establish that the SVG is readable or faithful to the SysML view.
The exact Markdown comes from the profile's `figure_block.template`,
`templates/figure-block.md`. `section_id` and `ordinal` form the
figure number; `title` supplies its alt text and title; `asset` supplies
the embed path; `field_tokens` maps each ordered field label to its template
token. The first `Status` term is one of `renderable-now`, `planned`,
`table-substitute`, or `external-tool`; `designed-only` and
`implemented` are separate software-claim qualifiers, not primary figure
statuses.

## 3. Checks, writes, and errors

Managed documents are identified by the profile's marker and a project-local
`docs/.document-kit-managed.json` registry containing path, class, profile
identity, version, and contract digest. Removing or changing a marker cannot
silently downgrade a registered page to a legacy baseline check. Existing
pages without a registry entry keep their established project rules; this
starter profile does not force their headings into a new template.

The validator checks class path/name, one H1, ordered exact H2 chapters,
required nonempty labeled fields and section prose, explicit unique anchors,
unresolved placeholders, local links, figure embeds/blocks/assets, and
`DocSection`/`DocFigure` targets found under configured model roots. It
reports unsupported or malformed locator forms as diagnostics rather than
silently counting them as passes. It does not parse or resolve SysML names:
the selected compiler remains the authority for those checks. Model truth,
design adequacy, and figure readability require separate review.

An unsuccessful operation returns `ok=false`, an `error` object with stable
`code` and `message`, and `diagnostics` with rule/code, project-relative
file, line when available, and reason. Missing inputs, unreadable source,
unsafe paths, duplicate anchors, stale hashes, and malformed profiles fail
closed. `create_document` never overwrites. Updates require an exact
`expected_sha256`; the engine checks it again immediately before replacing
the target. A failed write leaves existing document bytes unchanged.
Temporary files belong under the selected project's `cache/`; an atomic
replace across filesystems may be refused. A two-file page/registry update
may be interrupted, so validation must report a missing page or registry
entry honestly and a retry must not silently overwrite a surviving page.

Every diagnostic has exactly `rule` (a DOC-001..010 ID or `input`),
`code` (stable lowercase kebab-case), `file` (project-relative path,
or empty for a profile-wide error), `line` (positive integer or null), and
`detail` (human-readable reason). The initial code vocabulary is:

| Rule family | Stable codes |
| --- | --- |
| Contract/input | `invalid-profile`, `contract-drift`, `invalid-request`, `unreadable-input`, `missing-model-root` |
| Path/write | `unsafe-path`, `invalid-document-name`, `missing-document`, `document-exists`, `stale-document`, `atomic-replace-failed` |
| Template/section | `missing-metadata`, `section-headings`, `missing-anchor`, `duplicate-anchor`, `missing-field`, `empty-content`, `unresolved-placeholder`, `registry-mismatch` |
| Model/link | `missing-model-source`, `invalid-link`, `invalid-docsection`, `invalid-docfigure`, `unsupported-annotation` |
| Figure | `missing-figure`, `figure-block`, `figure-status`, `duplicate-figure` |

`check_documents` returns its normal structured report with `ok=false`
when findings exist. An invalid MCP request or refused write is a tool error
whose text contains the JSON error envelope with the same core code and
diagnostics as the CLI; the transport's error flag is not a semantic
assessment. A caller must inspect `ok` or the error code before proceeding.

The CLI emits one JSON result to stdout and exits 0 on a successful operation
or passing check, 1 on a rejected operation or failed check, and 2 on an
invalid invocation. The stdio MCP server exposes the six operations as tools
and `xdesign-doc-kit://contract` and `xdesign-doc-kit://rules` as
resources. `xdesign-doc-kit://template/{document_class}` is a resource
template discovered through `list_resource_templates`. The adapters do
not maintain separate validation rules. The server pins the contract digest
at launch and refuses tool/resource calls after the on-disk pack drifts,
until restarted with the new version.

## 4. Version and evidence boundary

The profile schema and pack version are separate: a compatible schema may
have a new rule/template version. A consumer pins the pack bytes by digest
for an assessed wish and reviews a changed digest before reusing an earlier
document result. CLI/MCP results state the files and predicates checked,
including any unevaluated checks. They cannot authorize a design-stage gate
by themselves; the project flow must bind the checker and record any separate
semantic judgment at the artifact revision it reviewed.

This interface is the design basis. Source and executed evidence, not this
page's existence, establish whether an endpoint is available.
[DOC-01 through DOC-07](../verification/design.md#d114) define its
acceptance cases.
