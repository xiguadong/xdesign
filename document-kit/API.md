# Bilingual document-kit consumer API

This guide describes schema 1.3, release 1.3.0 and digest version 3. The current contract below takes precedence over the retained schema 1.2/1.1/1.0 reference later in this file. Select a complete pack explicitly; a source archive includes all resources.

## Selection grammar, defaults and errors


The bundled `language_policy` is exactly:

```json
{"supported_ids": ["en", "zh-CN"], "default": "en"}
```

Language values are case-sensitive. Omission under schema 1.3 means explicit `en`;
there is no locale, environment, content or neighboring-document inference and no
fallback to another variant after selection. A supplied well-formed but unlisted
ID returns `unknown-language`. Well-formed selector grammar is
`[A-Za-z]{2,3}(?:[-_][A-Za-z0-9]{2,8})*`, which deliberately classifies `zh`,
`en-US`, `ZH-CN`, and `zh_CN` as unsupported rather than malformed. Empty,
non-string, whitespace, non-ASCII, repeated/leading/trailing-separator, overlong
or conflicting selector inputs are `invalid-request`.

`unknown-language` is the only new operation-error code in this section. A missing,
unsupported or disagreeing **persisted** language is instead `registry-mismatch`.
An explicitly conflicting selector on an operation that exposes one is
`invalid-request`. No `identity-mismatch` or prose-language error exists: arbitrary
caller prose neither selects nor changes identity, and prose adequacy remains
BDK-06 independent semantic review.

Plan/create and language-addressed resource reads accept selection. Update,
attachment and selected-document check accept an optional assertion that must
match the recorded variant; omission uses that variant and never the current
default. Whole-project/class-filter checks reject non-null language as
`invalid-request`. A same-route second create retains `document-exists` when the
file exists and `registry-mismatch` when only its registry path is occupied.

Under every selected original pack, any supplied language selector/assertion is
`invalid-request`, even if the value would be unsupported under 1.3. Omission
preserves original behavior. When 1.3 resolves an old effective page, omission
permits its old update/check behavior; supplying language for that old effective
page is `invalid-request`. Old language-less entries are not backfilled. Original
loaders may reject a foreign language key, digest-version 3 or mixed registry;
this is retained fail-closed behavior, not an instruction to migrate.

## Python operations and results

The public exports remain the six functions and `DocumentKitError`. Existing
positional parameters and defaults are unchanged; every language addition is
keyword-only and follows the existing keyword arguments:

```python
get_contract(profile_root=None)
plan_document(target_root, document_class, name, profile_root=None,
              template_id=None, *, layer=None, language=None)
create_document(target_root, document_class, name, metadata, sections,
                profile_root=None, expected_contract_sha256=None,
                template_id=None, *, layer=None, language=None)
update_section(target_root, document_class, name, section_key, content,
               expected_sha256, fields=None, profile_root=None,
               expected_contract_sha256=None, *, layer=None, language=None)
attach_figure(target_root, document_class, name, section_key, figure,
              expected_sha256, profile_root=None, expected_contract_sha256=None,
              concern=None, *, layer=None, decision=None, language=None)
check_documents(target_root, document_class=None, profile_root=None, *,
                selection=None, language=None)
```

`route_selection` and the `selection` input remain exactly the I21
class/name/layer object. Language is a separate argument and may not be inserted
into that object. Create/update/attach JSON payloads likewise retain their I21
keys; Python callers select language only by the keyword argument. Metadata and
section caller keys, including the English `figure` member names, do not change.
Supplying a root/output path or language inside a payload is an extra-key
`invalid-request`, never a silent override.

| Operation | Schema 1.3 success result and behavior |
| --- | --- |
| `get_contract` | Existing fields plus `language_policy`, complete variant declarations in `catalog.entries`, `resource_index`, `shared_resource_index`, and `resource_bytes`. The existing `templates` projection and `templates["figure-block"]` explicitly use default `en`; `figure_templates` maps both language IDs to their exact figure-block strings. No project is read or written. |
| `plan_document` | Existing fields plus `language` and a seven-field `effective_identity`. `required_sections` contains the selected variant's headings, stable keys and localized `field_labels`; `figure_block` describes its localized labels/template. Route and existence reporting are unchanged and no file is written. |
| `create_document` | Existing fields plus `language`, seven-field effective identity and unchanged `route_selection`. The new marker/registry/page bytes agree. Existing destinations and occupied registry paths retain their collision behavior. |
| `update_section` | Existing fields plus `language` and effective identity. Only the selected stable-key section changes; localized labels are preserved and marker/registry bytes remain identical. A supplied assertion must match the persisted variant. |
| `attach_figure` | Existing fields plus `language` and effective identity. The existing figure input keys are rendered through the effective variant's figure block; all other section and registry bytes are preserved. |
| `check_documents` | Existing fields, language-bearing effective identities for new pages and language-less identities for old pages. `scope.language` is the selected assertion or null. Whole/class scope and all context-scan disclosures remain unchanged. |

Every new effective identity has exactly `profile_id`, `version`,
`document_class`, `template_id`, `language`, `contract_sha256`, and
`contract_digest_version`. New top-level plan/create/update/attach results also
have `language`; old effective identities have no language member. Selected-pack
identity at the result top level remains distinct from each old or new page's
effective identity.

`resource_index` is an array of exactly 168 objects, one for every
`(template_id, language, kind)` in 21 × 2 × 4. Each object has exactly
`template_id`, `document_class`, `language`, `kind`, `path`, `sha256`, and
`mime_type`; `kind` is `template`, `guide`, `example`, or `fixture`, `path` is the
manifest-declared pack-relative path, and SHA-256 covers the exact bytes.
`shared_resource_index` uses exactly `path`, `sha256`, and `mime_type` for shared
sources, routing, mixed inputs, models/assets and both figure-block templates.
Repeated references to a shared path have one index entry.

`resource_bytes` is an object keyed by each current index path whose values are
the exact UTF-8 resource strings, including guide, example and fixture bytes. It
contains no retained-compatibility mirror and no synthesized translation. An
example returned under `kind="example"` remains an example and cannot become a
selected template. CLI `get-contract` emits the same object; Python consumers use
the returned mapping directly. MCP uses the equivalent catalog index and resource
reads below rather than creating a second byte projection.

## Marker, registry and selected/effective contracts

The exact new marker form is:

```text
<!-- document-kit: profile={profile_id}@{version} class={document_class} template={template_id} language={language} contract={contract_sha256} digest-version={contract_digest_version} -->
```

Registry `schema_version` remains 1. A new entry has exactly `path`, `class`,
`profile_id`, `version`, `contract_sha256`, `template_id`,
`contract_digest_version`, and `language`; the last value is required and the
digest version is `"3"`. Existing old entries have no language key and keep their
old exact key set. The registry stores no document hash. Missing, extra,
unsupported or disagreeing language in a new marker/registry/resolved variant is
`registry-mismatch` before mutation.

Under the selected 1.3 pack, whole/selected checks resolve exact bundled 1.0.0,
1.1.0, 1.2.0 and 1.3.0 entries under their effective contracts. Update/attachment
can operate on those old pages without language and preserves their marker,
language-less effective identity and registry bytes. Original selected packs keep
their original strict registry behavior and may reject the new key or foreign new
pages; they are not required to understand a mixed registry. The 1.3 pack is the
supported mixed-project checker/updater, and no operation adopts an unmanaged
page or silently migrates an identity.

## JSON CLI contract

The command and existing flag names are retained. `--language <id>` is added to
`plan-document`, `create-document`, `update-section`, `attach-figure`, and selected
`check-documents`. For check it requires `--class` and `--name`; `--language`
without `--name` is `invalid-request`. Repeating `--language`, supplying it through
both a selection object and the flag, or placing it in an operation JSON payload
is a conflicting/invalid request rather than last-value-wins behavior. All
`--layer` rules remain unchanged.

Create input remains exactly `metadata` and `sections`; update input remains
`content`, `expected_sha256`, and optional `fields`; attach input remains `figure`,
`expected_sha256`, and optional `decision`. Contract pins still use
`--expected-contract-sha256`, and update/attach still use the current page hash.
CLI emits one JSON object and retains exit 0 for success, 1 for a rejected
operation/failing check and 2 for invalid invocation. `unknown-language` and
`invalid-request` use the same core envelope and diagnostic shape as every other
operation error.

For example, after supplying a real context model and complete Chinese caller
input at `cache/zh-create.json`, the consumer sequence is:

```sh
xdesign-doc-kit plan-document --target-root . --profile-root /path/to/kit \
  --class requirements --name sensor-requirements --language zh-CN
xdesign-doc-kit create-document --target-root . --profile-root /path/to/kit \
  --class requirements --name sensor-requirements --language zh-CN \
  --input cache/zh-create.json
xdesign-doc-kit check-documents --target-root . --profile-root /path/to/kit \
  --class requirements --name sensor-requirements --language zh-CN
xdesign-doc-kit update-section --target-root . --profile-root /path/to/kit \
  --class requirements --name sensor-requirements --section purpose \
  --input cache/zh-update.json
```

The update intentionally omits language and uses the persisted variant. Omitting
language from plan/create in this release would select `en`, not inspect the
Chinese payload. The equivalent Python calls use the signatures above, with
`language="zh-CN"` on plan/create and the optional selected-check assertion.

## Bound stdio MCP tools and resources

The six MCP tools retain their names and launch-bound project/profile. Optional
`language` arguments mirror Python on plan/create/update/attach and selected
check; `get_contract` has no language filter because it enumerates every variant.
An absent expected contract digest uses the launch pin, and any supplied digest
must match it. Root/profile redirection, including redirection smuggled into a
selection object, is rejected before language processing. Tool errors serialize
`DocumentKitError.as_dict()` in full; a completed check with findings remains a
normal structured `ok=false` report.

Existing fixed resources `xdesign-doc-kit://contract`,
`xdesign-doc-kit://rules`, and `xdesign-doc-kit://catalog` remain. The catalog
resource contains the same language policy, entries and resource indexes as
`get_contract`. Existing template reads have these explicit-default semantics
under 1.3:

| Resource | Bytes and selection |
| --- | --- |
| `xdesign-doc-kit://template/{document_class}` | The class generic entry's `en` template. |
| `xdesign-doc-kit://template/{document_class}/{template_id}` | The named entry's `en` template. Omission means the declared default, not a response to client locale. |
| `xdesign-doc-kit://template/{document_class}/{template_id}/{language}` | The named entry and language's exact template bytes. |
| `xdesign-doc-kit://entry/{template_id}/{language}/{resource_kind}` | Canonical read for any declared `template`, `guide`, `example`, or `fixture` variant. |

Every URI segment is one exact value; encoded separators, traversal, extra
segments and arbitrary filesystem paths are invalid. Unknown class/template/kind
retains resource-not-found behavior. Malformed language is `invalid-request` and
well-formed unsupported language is `unknown-language`, both with the core error
envelope; these are not silently converted to not-found or English bytes.
Discovery through `catalog` and the registered resource templates enumerates all
42 variants and four kinds without a filesystem glob. MCP revalidates the launch
pin and retained child bindings on every tool/resource request.

For example, the Chinese generic-requirements guide and fixture reads are:

```text
xdesign-doc-kit://entry/generic-requirements/zh-CN/guide
xdesign-doc-kit://entry/generic-requirements/zh-CN/fixture
```

The corresponding example URI returns the fictional example, never a template
selection. No network resource or seventh authoring tool is added.


## Localized page and figure serialization

Each selected variant supplies exact metadata labels, H2 headings and per-field
labels. Caller keys and model identities remain stable. Metadata label allocation
is `Status`/`Revision`/`Model sources` in English and `状态`/`修订`/`模型来源` in
Chinese. The ASCII `:**` delimiter, vertical metadata separators and content
comments are stable serialization syntax.

The figure API object retains its eight existing English member names. Rendered
figure labels are:

| Semantic token | English output | zh-CN output |
| --- | --- | --- |
| Figure label | `Figure <section-id>-<ordinal>` | `图 <section-id>-<ordinal>` |
| `question` | `Question` | `问题` |
| `model_view` | `Model view` | `模型视图` |
| `companions` | `Companions` | `配套材料` |
| `status` | `Status` | `状态` |
| `reading` | `Reading` | `解读` |
| `limits` | `Limits` | `限制` |

`figure_number` remains `<section-id>-<ordinal>`. Statuses, qualifiers, concern and
choice IDs, asset paths and model-view identities retain their machine tokens.
Shared SVGs have one canonical path per consumer document and one pack resource;
localized captions do not duplicate those SVG bytes.

The initial variant manifests use the following exact field-label allocation.
English labels are the current 1.2 strings; Chinese labels are required strings,
not suggestions for later ad hoc translation.

| Current English field label | Required zh-CN label |
| --- | --- |
| Alternate flows | 备选流程 |
| Api identity | API 标识 |
| Api kind | API 类型 |
| Applicability reasons | 适用性理由 |
| Behavior | 行为 |
| Boundary | 边界 |
| Chosen design | 选定设计 |
| Compatibility constraints | 兼容性约束 |
| Compatibility policy | 兼容性策略 |
| Components | 组成要素 |
| Constraints | 约束 |
| Current behavior | 当前行为 |
| Data shapes | 数据结构 |
| Declaration | 声明 |
| Design links | 设计链接 |
| Design rationale | 设计依据 |
| Design subject | 设计对象 |
| Diagram decisions | 图示决策 |
| Entities | 实体 |
| Environment | 环境 |
| Error behavior | 错误行为 |
| Errors | 错误 |
| Events | 事件 |
| Exit gates | 退出门控 |
| Expected results | 预期结果 |
| External actors | 外部参与者 |
| Fields | 字段 |
| Figure references | 图引用 |
| Flow scope | 流程范围 |
| Guards | 监护条件 |
| Impacted items | 受影响项 |
| Information flows | 信息流 |
| Inputs | 输入 |
| Interactions | 交互 |
| Interface participants | 接口参与者 |
| Interfaces | 接口 |
| Invariants | 不变量 |
| Layer | 层次 |
| Lifecycle | 生命周期 |
| Measurable conditions | 可测量条件 |
| Methods | 方法 |
| Migration notes | 迁移说明 |
| Model elements | 模型元素 |
| Model realization or reason | 模型实现或理由 |
| Model views | 模型视图 |
| Navigation links | 导航链接 |
| Normative statements | 规范性陈述 |
| Operations | 操作 |
| Ordering | 顺序 |
| Outputs | 输出 |
| Ownership | 归属 |
| Participants | 参与者 |
| Parts | 组成部分 |
| Pass criteria | 通过准则 |
| Procedure | 步骤 |
| Proposed behavior | 拟议行为 |
| Purpose | 目的 |
| Rationale | 依据 |
| Recovery behavior | 恢复行为 |
| Relationships | 关系 |
| Required fields | 必填字段 |
| Requirements links | 需求链接 |
| Residual risks | 残余风险 |
| Responsibilities | 职责 |
| Review questions | 评审问题 |
| Side effects | 副作用 |
| Source projection | 源码映射 |
| Stages | 阶段 |
| States | 状态 |
| System boundary | 系统边界 |
| Terminal states | 终止状态 |
| Trace matrix | 追踪矩阵 |
| Tradeoffs | 权衡 |
| Transitions | 转换 |
| Unexecuted cases | 未执行用例 |
| Unresolved items | 未决事项 |
| Validation rules | 校验规则 |
| Verification links | 验证链接 |
| Verification method | 验证方法 |
| Version policy | 版本策略 |
| View coverage | 视图覆盖范围 |

The initial exact heading allocation is:

| Current English heading | Required zh-CN heading |
| --- | --- |
| 1. Allocation | 1. 分配 |
| 1. Change summary | 1. 变更摘要 |
| 1. Context and boundary | 1. 背景与边界 |
| 1. Coverage and applicability | 1. 覆盖范围与适用性 |
| 1. Purpose and allocation | 1. 目的与分配 |
| 1. Purpose and participants | 1. 目的与参与者 |
| 1. Purpose and scope | 1. 目的与范围 |
| 1. Scope and basis | 1. 范围与依据 |
| 1. Stages and gates | 1. 阶段与门控 |
| 2. Affected requirements | 2. 受影响的需求 |
| 2. Behavior and constraints | 2. 行为与约束 |
| 2. Behavior and source projection | 2. 行为与源码映射 |
| 2. Composition and responsibilities | 2. 组成与职责 |
| 2. Data contract | 2. 数据契约 |
| 2. Entities and fields | 2. 实体与字段 |
| 2. External interfaces | 2. 外部接口 |
| 2. Interface contract | 2. 接口契约 |
| 2. Methods and environment | 2. 方法与环境 |
| 2. Operation contract | 2. 操作契约 |
| 2. Participants and interfaces | 2. 参与者与接口 |
| 2. Parts and responsibilities | 2. 组成部分与职责 |
| 2. Requirements | 2. 需求 |
| 2. Review questions and methods | 2. 评审问题与方法 |
| 2. States and invariants | 2. 状态与不变量 |
| 2. Structure and interfaces | 2. 结构与接口 |
| 2. Views and references | 2. 视图与引用 |
| 3. Acceptance conditions | 3. 验收条件 |
| 3. Behavior and constraints | 3. 行为与约束 |
| 3. Behavior and lifecycle | 3. 行为与生命周期 |
| 3. Compatibility and migration | 3. 兼容性与迁移 |
| 3. Errors and invariants | 3. 错误与不变量 |
| 3. Events and guards | 3. 事件与监护条件 |
| 3. Inputs and procedure | 3. 输入与步骤 |
| 3. Interactions and lifecycle | 3. 交互与生命周期 |
| 3. Main interaction flow | 3. 主交互流程 |
| 3. Relationships and interfaces | 3. 关系与接口 |
| 3. Relationships and ownership | 3. 关系与归属 |
| 3. Status and limits | 3. 状态与限制 |
| 3. Trace and verification | 3. 追踪与验证 |
| 3. Validation and invariants | 3. 校验与不变量 |
| 3. Verification and limits | 3. 验证与限制 |
| 3. Verification cases | 3. 验证用例 |
| 4. Acceptance conditions | 4. 验收条件 |
| 4. Acceptance criteria and trace | 4. 验收准则与追踪 |
| 4. Alternate flows and errors | 4. 备选流程与错误 |
| 4. Behavior and lifecycle | 4. 行为与生命周期 |
| 4. Constraints and decisions | 4. 约束与决策 |
| 4. Decisions and rationale | 4. 决策与依据 |
| 4. Invariants and lifecycle | 4. 不变量与生命周期 |
| 4. Model and design trace | 4. 模型与设计追踪 |
| 4. Transitions and lifecycle | 4. 转换与生命周期 |
| 4. Versioning and compatibility | 4. 版本管理与兼容性 |
| 5. Behavior and constraints | 5. 行为与约束 |
| 5. Constraints and decisions | 5. 约束与决策 |
| 5. Implementation projection and verification | 5. 实现映射与验证 |
| 5. Model and verification trace | 5. 模型与验证追踪 |
| 5. Projection and verification | 5. 映射与验证 |
| 5. Recovery and constraints | 5. 恢复与约束 |
| 5. Status and limits | 5. 状态与限制 |
| 5. Trace and limits | 5. 追踪与限制 |
| 5. Trace and verification | 5. 追踪与验证 |
| 6. Projection and verification | 6. 映射与验证 |
| 6. Status and limits | 6. 状态与限制 |
| 6. Trace and verification | 6. 追踪与验证 |
| Behavior and constraints | 行为与约束 |
| Declaration and contract | 声明与契约 |
| Purpose and boundary | 目的与边界 |
| Scope and navigation | 范围与导航 |
| Status and limits | 状态与限制 |
| Status and trace | 状态与追踪 |
| Trace and verification | 追踪与验证 |

Each variant may refine title, guidance, examples and prose without literal
translation, but may not weaken these section/field meanings or their ordered
engineering obligations. Missing labels, duplicate labels within a section,
English-label substitution in a selected Chinese structural position and malformed
variant/template disagreement are catalog defects. Whether the remaining prose is
actually Chinese, scholarly or equivalent is not inferred by this table or a
deterministic scan; BDK-06 judges it.


## Digest, sentinel and retained closure contract

Digest version `"3"` initializes SHA-256 with
`xdesign-doc-kit-contract\0` followed by ASCII `3\0`, then sorts unique normalized
pack-relative paths bytewise by UTF-8 and feeds each path, NUL, ASCII decimal byte
length, NUL and exact bytes. The explicit closure includes profile, consumer
README/RULES/SKILL/API, manifest, all 168 variant resources, shared source/routing/
mixed/model/asset resources, both figure blocks, and every declared compatibility
descriptor and recursive child byte. Shared paths are hashed once; no directory
glob, network input, text normalization or sentinel masking is permitted.

| Release and selectable root | Digest version | Required recomputation |
| --- | --- | --- |
| `compatibility/v1.0.0/` | `"1"` | `1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6` |
| `compatibility/v1.1.0/` | `"2"` | `e2b827c6e929f4a8d03344a91ca72f2eb837962e916d5e7f0487e315e1440cef` |
| `compatibility/v1.2.0/` | `"2"` | `a4ab976f05769bff34f3b6d3acd0de1dafb728dcf372d099fc3ac938758cf698` |
| new root | `"3"` | Computed only after the exact future 1.3 bytes exist; no digest is asserted by this design. |

The 1.2.0 closure includes its original 1.0.0 and 1.1.0 children; the 1.1.0
closure includes its original v1 child. The root-level 1.0.0 and 1.1.0 selectable
copies are also retained. Only these exact historical duplicate identities are
permitted, and each copy must match its canonical relative paths and bytes. Any
other duplicate identity, differing duplicate bytes, cycle, overlapping nonnested
root or symlink is invalid. Child digests are always recomputed relative to the
child root before its descriptor is trusted; parent hashing prefixes those exact
bytes with their parent-relative paths.

Each of the 42 new stored examples has exactly one 64-lowercase-zero sentinel `Z`
in the `contract` field of its exact first-line marker, actual non-contract
identity including `language`, and digest-version `"3"`; its actual digest `D`
occurs nowhere in the example. No other 1.3-owned resource contains `Z`.
Compatibility children enforce their own example/sentinel domains independently,
while the parent hashes every retained byte. Recomputed `D == Z` is
`invalid-catalog`; a zero descriptor with nonzero actual recomputation is
`contract-drift`; caller expected `Z` is `invalid-request`; a zero target marker or
registry is `registry-mismatch`.

Every isolated and mixed fixture omits `D`, `Z`, `contract_sha256`, and
`expected_contract_sha256`. The three-field external manifest and its separately
computed evidence hash remain unchanged. The acceptance comparator independently
asserts created/stored language and all other identity fields, replaces only the
parsed first-marker `contract=` sentinel span in memory, and compares all
remaining bytes, whitespace and line endings. Product operations never perform
that substitution.

## Failure order, packaging and evidence boundary

Adapter request-shape checks and MCP binding rejection come first. Dispatch then
preserves old-pack language rejection and old operation precedence. Under 1.3,
language syntax is checked before pack-dependent support resolution; selected
pack/child validation and digest integrity precede target identity resolution.
Create keeps existing-file before occupied-registry collision precedence.
Update/attachment keep ownership, readability and current-hash checks before
effective-contract and language-assertion checks. Candidate content, model and
figure checks precede publication; selected and effective pins plus document hash
are rechecked immediately before replacement. No error path changes existing
pages, registry or outside-target bytes unless the existing publication-outcome
contract explicitly reports a page-only or unknown state.

The supported package is the complete copied or pinned distribution, not just the
Python wheel's modules. It includes all declared current resources and exact old
closures at their selectable roots. Source-distribution assembly must enumerate
those files for packaging without making that enumeration a runtime discovery
rule. Installation into a consumer cache never creates another editable current
catalog: the consumer selects one copied root and pins its digest, while any
earlier pinned copy remains a distinct reviewed release rather than a synchronized
mirror.

Evidence records selected/effective languages and digests, every resource-index
hash, exact inputs and rendered bytes, all commands/exits/transcripts, before/after
nonwrite hashes, old-closure comparisons and unevaluated checks. The independent
reviewer records 21 per-entry Chinese-quality and English-equivalence judgments.
Structural passes, this interface's existence and compiler reports establish none
of those semantic judgments and do not execute BDK-01 through BDK-09.

---

# Retained structured and original release reference

This standalone guide describes schema 1.2 and the retained schema 1.0/1.1
contracts. Copy the complete distribution, select its profile directory, and pin
`get_contract.contract_sha256`. The Python module, JSON CLI and actual stdio MCP
server execute the same operations. They render only caller-supplied engineering
content. Structural success always requires separate semantic review.

Install with Python 3.12 (minimum 3.11) using a project-local environment:

```sh
python3.12 -m venv cache/document-kit-venv
PIP_CACHE_DIR="$PWD/cache/pip" cache/document-kit-venv/bin/pip install '/path/to/document-kit[mcp]'
```

Or run from a copied distribution with `PYTHONPATH=/path/to/document-kit`
and `python -m xdesign_doc_kit`; launch MCP with
`python -m xdesign_doc_kit.mcp_server --project-root /path/to/project --profile /path/to/document-kit`.
Install the declared MCP extra for the server. The project must already exist;
planning needs no model. Create/check/update/attach require readable context
models. Keep temporary inputs, environments and check output in project `cache/`.

## Routes and complete structured template fields

| Class | Generic template ID | Route | `name` means | `layer` |
| --- | --- | --- | --- | --- |
| `api-design` | `generic-api-design` | `docs/design/api/{layer}/{name}.md` | API basename; `index` reserved | required, selected from the layer set |
| `flow-index` | `generic-flow-index` | `docs/design/flows/{name}/index.md` | flow basename | absent |
| `flow-definition` | `generic-flow-definition` | `docs/design/flows/{name}/definition.md` | flow basename | absent |
| `flow-implementation` | `generic-flow-implementation` | `docs/design/flows/{name}/implementation.md` | flow basename | absent |
| `flow-views` | `generic-flow-views` | `docs/design/flows/{name}/views.md` | flow basename | absent |

| Class | Section key → heading: required fields |
| --- | --- |
| `api-design` | `purpose` → `Purpose and boundary`: `api_identity`, `api_kind`, `layer`, `purpose`; `contract` → `Declaration and contract`: `declaration`, `inputs`, `outputs`, `errors`, `side_effects`; `behavior` → `Behavior and constraints`: `behavior`, `invariants`, `constraints`; `trace` → `Trace and verification`: `model_realization_or_reason`, `source_projection`, `requirements_links`, `design_links`, `verification_links`; `limits` → `Status and limits`: `unresolved_items`. |
| `flow-index` | `scope` → `Scope and navigation`: `flow_scope`, `navigation_links`; `trace` → `Status and trace`: `model_elements`, `requirements_links`, `verification_links`, `unresolved_items`. |
| `flow-definition` | `stages` → `1. Stages and gates`: `stages`, `transitions`, `exit_gates`; `behavior` → `2. Behavior and constraints`: `behavior`, `invariants`; `trace` → `3. Trace and verification`: `model_elements`, `requirements_links`, `verification_links`. |
| `flow-implementation` | `allocation` → `1. Allocation`: `components`, `interfaces`; `projection` → `2. Behavior and source projection`: `behavior`, `source_projection`, `constraints`; `trace` → `3. Verification and limits`: `model_elements`, `verification_links`, `unresolved_items`. |
| `flow-views` | `coverage` → `1. Coverage and applicability`: `view_coverage`, `applicability_reasons`; `views` → `2. Views and references`: `model_views`, `figure_references`, `verification_links`; `limits` → `3. Status and limits`: `unresolved_items`. |


## 1. Selection and profile representation

The new `document_class` values are `api-design`, `flow-index`, `flow-definition`,
`flow-implementation`, and `flow-views`. The existing five class names are retained.
A selection is exactly `{ "document_class": string, "name": string, "layer": string|null }`.
`name` is the API basename for `api-design`, the flow basename for flow classes,
and the existing filename slug for a flat class. `layer` is required only for
`api-design` and must be null/omitted for all other classes. Omission is normalized
to null in returned `route_selection`. Components are lowercase semantic kebab-case;
the bundled API layers are `core`, `simple-flow`, `wish-verify`, `storage`, and
`index` is a reserved API name. The flow class `flow-index` creates a genuine
`index.md` page and accepts an ordinary flow name.

The exact bundled routes and template fields are listed below. For example, selections
`api-design`/`create-run`/`core` and `flow-definition`/`simple-flow`/null produce
`docs/design/api/core/create-run.md` and
`docs/design/flows/simple-flow/definition.md`. The other flow classes produce
`index.md`, `implementation.md`, and `views.md` in that same flow directory.
A reviewed derived schema 1.2 pack can configure its allowed layers and safe route
prefixes within the documented selector grammar. Pack validation rejects ambiguous
or overlapping routes, path traversal, absolute paths, unexpected separators and
symlink components. A target path is never a caller-supplied create/update/attach
argument.

## 2. Python operations

The `xdesign_doc_kit` public exports remain the six functions and `DocumentKitError`.
Existing positional parameters and defaults are retained; additions are keyword-only:

```python
get_contract(profile_root=None)
plan_document(target_root, document_class, name, profile_root=None,
              template_id=None, *, layer=None)
create_document(target_root, document_class, name, metadata, sections,
                profile_root=None, expected_contract_sha256=None,
                template_id=None, *, layer=None)
update_section(target_root, document_class, name, section_key, content,
               expected_sha256, fields=None, profile_root=None,
               expected_contract_sha256=None, *, layer=None)
attach_figure(target_root, document_class, name, section_key, figure,
              expected_sha256, profile_root=None, expected_contract_sha256=None,
              concern=None, *, layer=None, decision=None)
check_documents(target_root, document_class=None, profile_root=None, *, selection=None)
```

`profile_root` is a selected pack directory; `target_root` is an existing selected
project directory. `template_id` is optional for 1.1/1.2 plan/create and defaults
to the class's generic entry; it is never used to retarget an existing document.
`expected_contract_sha256`, when supplied, is a nonzero lowercase 64-hex digest
of the selected pack and is rechecked before publication. `expected_sha256` on
update/attach is the current document byte hash. `fields`, if supplied, maps only
required field keys for that section to nonempty strings. `figure` has exactly
`asset`, `title`, `Question`, `Model view`, `Companions`, `Status`, `Reading`,
and `Limits`; `asset` names an existing `assets/<semantic-name>.svg` relative to
the page. `concern` is null for the five bundled 1.2 entries and selects an exact
declared concern in a derived entry. `decision` is null except for an as-yet
unselected schema 1.2 `optional` concern that permits a figure; it is the exact
six-field object described below, supplied by the caller.

`selection` is null for whole-project/class-filter checking; otherwise it is the
three-key selection object above, with explicit null `layer` when inapplicable.
If both `document_class` and `selection` are supplied, they must agree. The class
filter without `selection` retains project-global checks; under 1.2 its document
set uses recorded classes and complete matching from the
route and scope contract.
A selected check accepts only a registered managed target and reports its
scope; it is not a whole-project acceptance. Schema 1.0/1.1 selections reject
non-null `layer` or `selection` as `invalid-request`, leaving their old whole/
class-filter operations and original result fields unchanged.

| Operation | Success result and effects |
| --- | --- |
| `get_contract` | Existing selected `profile_id`, `version`, `contract_sha256`, `contract_digest_version`, `profile`, `rules`, `templates`, `catalog`, and `semantic_review_required`; 1.2 `profile` includes the new route declarations and `templates` includes the exact five new generic and existing catalog template bytes. No project is written. |
| `plan_document` | Existing `document_class`, `name`, `path`, `exists`, `sha256` (null if absent), `template_id`, `effective_identity`, `required_metadata`, `required_sections`, `anchor_policy`, `figure_block`, and `semantic_review_required`; 1.2 adds normalized `route_selection`. No model source need exist and no file is written. A readable unmanaged existing page is reported, not adopted. |
| `create_document` | Existing `path`, new document `sha256`, `anchors`, `template_id`, `effective_identity`, and `semantic_review_required`; 1.2 adds `route_selection`. The selected pack identity is also returned at top level. Create requires complete caller metadata and sections, validates the candidate and refuses an existing destination or registry collision. |
| `update_section` | Existing changed `path`, `sha256`, `effective_identity`, and `semantic_review_required`; 1.2 adds `route_selection`. The effective recorded template, not today's default, defines section keys and fields. Only the chosen section changes; marker, other sections and registry remain byte-identical. |
| `attach_figure` | Existing changed `path`, `sha256`, `figure_number`, `effective_identity`, `provenance_review_notes`, and `semantic_review_required`; 1.2 adds `route_selection`. The supplied existing asset and six fields form the figure block in the selected section. Stale hashes and prohibited choices refuse before publication. |
| `check_documents` | Existing `checked_paths`, `scanned_paths`, hashes, `effective_identity_by_path`, `scope`, `global_findings`, `diagnostics`, `unevaluated_checks` and `semantic_review_required`. Under 1.2, `scope.selection` is the normalized object or null; selected mode lists only the selected path in `checked_paths`, while `scanned_paths` discloses context scanning. All reports retain the selected pack identity at top level. |

All successful result objects include `ok=true` and selected `profile_id`,
`version`, `contract_sha256` and `contract_digest_version`. Each effective identity
contains exactly `profile_id`, `version`, `document_class`, `template_id`,
`contract_sha256`, `contract_digest_version`. A successful check means deterministic
predicates passed, not engineering or figure acceptance. A check with findings
returns a report with `ok=false` and diagnostics, rather than an exception.
For rejected operations Python raises `DocumentKitError`; `as_dict()` yields the
error envelope below.

A check report is returned only after its required inputs and scan scope can be
read. An invalid request, invalid pack, unreadable required input, malformed
shared registry or unregistered selection instead raises `DocumentKitError` with
the same error envelope used by other operations. Findings in readable pages (including identity, route,
anchor and locator defects) return `ok=false` with the advertised checked/scanned
scope. Neither a partial scan nor an exception is a completed check report.

## 3. Metadata, sections and diagrams

Each `metadata` object has exactly `title`, `status`, `revision`, and `model_sources`.
The first three are nonempty strings; `model_sources` is a nonempty list of distinct,
readable project-relative `.sysml` regular files under configured model roots.
At least one readable `.sysml` must be present in a required root. Each page,
including a flow index, supplies its own list. Language-side API rationale does not
replace the context model input or claim that the API is realized by that model.

`sections` is an object keyed by every section in the selected template and no
others. Each value has exactly `anchor`, `content`, and `fields`: a globally unique
profile-valid anchor, nonempty caller prose, and the exact required field map from
plan. Field values are nonempty strings; `not applicable` is accompanied by the
author's reason. The five new bundled templates and required fields are enumerated
in the template field table below. The API layer field equals
the selection's layer; a mismatch is `invalid-request` on create/update and the
same diagnostic on check. Other engineering-field meaning remains for review.
A derived template may use `diagram_decisions` as a string containing the JSON
array specified in the retained contract below. An applicable `figure` decision requires the supplied SVG and complete
block in the document; a `table`, `prose`, or `not-applicable` decision has a reason
and cannot be attached as a figure. Schema 1.2 adds the concern applicability value
`optional` with a nonempty trigger: its section's decision array may initially be
empty. To attach its first figure, pass `concern` and a `decision` object with
exactly `concern`, `applies`, `choice`, `rationale`, `asset`, `model_view`: the same
concern, `true`, `figure`, a nonempty caller rationale, and the same asset and model
view as the `figure` input. The operation inserts that decision and the figure block
in the same atomic document replacement. A present non-figure decision is refused
even if a replacement `decision` is submitted. The five bundled structured templates
declare no required concern, so a free optional figure uses `concern=null` and
`decision=null`; unknown concerns are rejected. Supplying `decision` under an
original 1.0/1.1 pack, or for a target whose effective pack is 1.0/1.1, is
`invalid-request`. Clearing/changing an attached optional figure decision through
update so its retained block no longer matches is `invalid-diagram-decision`;
update is not a figure removal operation. All figure fidelity and model meaning
are separately reviewed.

## 4. CLI, MCP tools and resources

The CLI retains `get-contract`, `plan-document`, `create-document`, `update-section`,
`attach-figure`, `check-documents`, `--target-root`, `--profile-root`, `--class`,
`--name`, `--template-id`, `--section`, `--concern` and `--input`. It adds `--layer`
to plan/create/update/attach and to selected check; it adds
`--expected-contract-sha256` to create/update/attach. An attach `--input` JSON object
may add the optional `decision` object alongside `figure` and `expected_sha256`.
For `check-documents`, no
`--name` means the existing whole-project or `--class` filter; `--name` requires
`--class` and selects one registered document, with `--layer` required for
`api-design`. `--layer` without `--name`, or a layer for another class, is
`invalid-request`. The CLI maps these flags into the exact Python selection object;
its existing JSON input file or JSON-string handling remains unchanged. It emits
one JSON result and exits 0 for success, 1 for rejected operations or failing checks,
and 2 for invalid invocation.

For 1.2 calls, the create JSON object has exactly `metadata` and `sections`;
update has required `content`, `expected_sha256` and optional `fields`; attach
has required `figure`, `expected_sha256` and optional `decision`. Reject extra
payload keys as `invalid-request`, including an `output_path` or alternate root.
Selectors and the expected contract digest come from the documented flags, never
silently ignored JSON keys. Original selected packs keep their old payload behavior.

The MCP tools retain their existing names and bound project/profile selectors.
Plan/create/update/attach add optional `layer`; create/update/attach add optional
`expected_contract_sha256`; attach adds optional `decision`; `check_documents` adds
optional `selection`. If an
expected digest is absent the launch-pinned digest is used; if supplied it must
match that pin. A well-formed nonmatching digest is `contract-drift`; a zero or
malformed digest is `invalid-request`. Tool arguments cannot redirect the
launch-selected root or pack.
The same core decisions and fields are returned. A rejected operation is an MCP
tool error with the core error envelope as JSON; a completed check with findings
returns a normal structured report with `ok=false`, not an error transport flag.

For 1.2 operations, adapters perform the shared request-shape preflight before
live pack validation; MCP first rejects attempted binding redirection. They do not call the existing
live-contract guard before that preflight and thereby change core error priority.
The launch pin is checked at the selected-pack phase, and on writes before
publication. Error transport serializes `DocumentKitError.as_dict()` in full,
including `outcome`, rather than rebuilding only code/message/diagnostics.

Existing resources remain `xdesign-doc-kit://contract`, `xdesign-doc-kit://rules`,
`xdesign-doc-kit://catalog`, `xdesign-doc-kit://template/{document_class}`, and
`xdesign-doc-kit://template/{document_class}/{template_id}`. In 1.2 the first three use the
selected 1.2 identity, the class template resource returns that class's generic
exact template bytes, and the selected template resource returns the exact named
entry bytes. `get_contract.templates` exposes the same bytes to CLI and Python.
Unknown/class-mismatched template IDs fail; examples are never resources or
selected templates. MCP revalidates the selected closure and retained child
bindings against its launch pin on every tool/resource request; drift requires a
restart with a reviewed pack.

## 5. Consumer example

The following is a fictional consumer example; its engineering claims are caller inputs. The caller must first provide `mbse/model/context.sysml`,
allocate unused anchors, and place the complete JSON payload shown below at
`cache/api-create.json`.

```sh
xdesign-doc-kit plan-document --target-root . --profile-root /path/to/kit \
  --class api-design --name create-run --layer core
xdesign-doc-kit create-document --target-root . --profile-root /path/to/kit \
  --class api-design --name create-run --layer core --input cache/api-create.json
xdesign-doc-kit check-documents --target-root . --profile-root /path/to/kit \
  --class api-design --name create-run --layer core
```

```json
{
  "metadata": {"title": "Create run", "status": "draft", "revision": "1", "model_sources": ["mbse/model/context.sysml"]},
  "sections": {
    "purpose": {"anchor": "U201", "content": "This fictional operation creates a run.", "fields": {"api_identity": "Example::create_run", "api_kind": "function", "layer": "core", "purpose": "Create a run for one plan."}},
    "contract": {"anchor": "U202", "content": "The caller supplies a plan and receives a run or an error.", "fields": {"declaration": "`create_run(plan) -> Result<Run, Error>`", "inputs": "One plan.", "outputs": "One run.", "errors": "Invalid plans are refused.", "side_effects": "No persistence in this fictional example."}},
    "behavior": {"anchor": "U203", "content": "The plan is validated before construction.", "fields": {"behavior": "Validate, then construct.", "invariants": "No run exists for an invalid plan.", "constraints": "No persistence is performed."}},
    "trace": {"anchor": "U204", "content": "The source projection and review are planned.", "fields": {"model_realization_or_reason": "Language-side helper; the context model identifies its boundary.", "source_projection": "src/core/factory.rs is the proposed location.", "requirements_links": "Plan validation requirement, pending link allocation.", "design_links": "Run design, pending link allocation.", "verification_links": "Invalid-plan case, pending link allocation."}},
    "limits": {"anchor": "U205", "content": "No implementation or result is claimed.", "fields": {"unresolved_items": "The example does not specify concurrency."}}
  }
}
```

The create result's `sha256` is the hash supplied by the caller to an update
payload such as `{"content":"Revised boundary.","expected_sha256":"<current-hash>"}`;
save that payload as `cache/api-update.json` and invoke:

```sh
xdesign-doc-kit update-section --target-root . --profile-root /path/to/kit \
  --class api-design --name create-run --layer core --section purpose \
  --input cache/api-update.json
```

`<current-hash>` denotes the actual nonzero SHA-256 returned by the prior operation,
not a placeholder accepted by the tool. The equivalent Python sequence, using the
same JSON input and an isolated project, is:

```python
import json
from pathlib import Path
from xdesign_doc_kit import plan_document, create_document, check_documents, update_section

payload = json.loads(Path("cache/api-create.json").read_text())
pack = "/path/to/kit"
plan = plan_document(".", "api-design", "create-run", pack, layer="core")
created = create_document(".", "api-design", "create-run", payload["metadata"],
                          payload["sections"], pack, layer="core")
selection = {"document_class": "api-design", "name": "create-run", "layer": "core"}
report = check_documents(".", profile_root=pack, selection=selection)
assert report["ok"] and report["checked_paths"] == [created["path"]]
updated = update_section(".", "api-design", "create-run", "purpose",
                         "Revised boundary.", created["sha256"], profile_root=pack,
                         layer="core")
```

For an MCP process already bound to the same project and pack, the `tools/call`
parameters for selected checking are the following; plan/create/update use the
same class/name/layer and caller data shown above, without root arguments:

```json
{"name":"check_documents","arguments":{"selection":{"document_class":"api-design","name":"create-run","layer":"core"}}}
```

## 6. Failure order, access and publication

Request shape, slug/layer syntax and supplied expected-digest shape are checked
first; load and validate the selected pack and all declared child bindings next;
then resolve the target route and reject symlink components. Create checks model
inputs, refuses an existing target (`document-exists`) before an occupied registry
path (`registry-mismatch`), validates content and anchor/link context, rechecks
the selected pin, and publishes. Update/attach check registry ownership, target
readability, current hash (`stale-document`), effective identity, section and
figure/decision validity, candidate content, then recheck both pin and hash before
replacement. An unmanaged update/attach is `registry-mismatch`; check applies its
ordinary project checks without adoption. A selected check of an unmanaged target
is `registry-mismatch`. Independent findings are sorted by file, line, code and
detail; reporting order does not imply semantic priority.

Request shape means argument types, allowed object keys, required members and
hash/slug syntax; required content semantics, allowed layers/reserved names and
concern membership need the loaded contract. On update/attach, an unknown concern,
disallowed decision or empty figure caption follows ownership and current-hash
checks, while a non-object `figure` or malformed expected hash precedes pack
loading. Test simultaneous faults to establish this order. For original selected
packs, existing operation precedence is preserved; this ordered extension applies
when 1.2 is selected, including its operations on retained pages.

A malformed `name` or reserved xdesign API `index` is `invalid-document-name`.
Missing, malformed, disallowed or unexpected `layer`, an unknown class, and a
class/selection disagreement are `invalid-request`; a symlink in a resolved target
or source is `unsafe-path`. A nonzero expected digest differing from the selected
pack is `contract-drift`. An unknown concern or mismatch between caller `decision`
and `figure` is `invalid-diagram-decision`; a missing decision for an unselected
optional concern is `missing-diagram-decision`. These checks precede writes, with
ownership and stale-hash precedence as stated above.

Whole-project check under 1.2 validates exact bundled v1, 1.1.0 and 1.2.0 pages
against their respective retained/effective contracts, plus unmanaged project
checks. Selected check verifies the selected registered page and its global anchor,
link and model-locator context as described in the design; it reports its scope,
including scans. The original pack roots remain selectable at
`compatibility/v1.0.0/` and `compatibility/v1.1.0/` beneath the 1.2 distribution.
Original packs retain their old behavior and their whole-project
checks may refuse foreign 1.2 identities. Unknown or forged selected identities
are `registry-mismatch` (unknown custom v1 remains `unsupported-legacy-identity`)
and never migrate. A known retained pack's descriptor mismatch is `contract-drift`;
a copied stored example with sentinel identity is `registry-mismatch`.

Compatibility bindings share one documentation root/registry and coherent class
routes as specified in the compatibility policy below.
A derived pack with conflicting roots, routes or
digests for the same release fails `invalid-profile`; it cannot silently retarget
old pages. This namespace validation follows child-byte validation and descriptor
comparison, so a wrong declared child digest remains `contract-drift`.

The error envelope is `{"ok":false,"error":{"code":...,"message":...},
"diagnostics":[{"rule":...,"code":...,"file":...,"line":...,"detail":...}]}`.
Existing codes keep their I20 meanings. New stable 1.2 codes are `access-denied`
(for denied route/lock/staging access before atomic replacement),
`publication-incomplete` (a newly created page is known to remain without its
registry entry), and `publication-unknown` (post-publication outcome cannot be
established). Denied readable project inputs use `unreadable-input`; denied atomic
page/registry replacement uses `atomic-replace-failed`; pack readers retain their
specific `invalid-profile`/`invalid-catalog` classifications. For these publication
or access errors the envelope additionally has `outcome` with exactly `operation`,
`phase`, `publication_state` (`none`, `page-only`, or `unknown`) and
`path` (safe project-relative path or null). Python exposes the same fields on the
exception and through `as_dict()`. For errors in a retained 1.0/1.1 operation,
existing codes and envelope remain unchanged except that a previously uncaught OS
exception is converted to the appropriate documented access error.

`operation` is one of the six Python operation names; `phase` is `pack-read`,
`model-read`, `document-read`, `registry-read`, `route`, `lock`, `staging`,
`page-publish`, `registry-publish`, `cleanup`, or `response`. `none` means this
invocation published no page or registry change (cache files and newly created
empty directories are not publication), `page-only` means only its new page is
known to remain, and `unknown` requires inspection after an error following
publication or an unconfirmed effect. A denied pack resource may put a normalized
pack-relative locator in diagnostic detail while `outcome.path` is null; it must
not expose an unsafe path as a project-relative target.

A failed atomic replacement never falls back to partial copying. If registry
publication after a new page fails, cleanup removes only the created page after
verifying it is still that creation; successful cleanup returns the underlying
failure with `publication_state=none`. If cleanup is denied and the orphan can be
confirmed, return `publication-incomplete` and `page-only`; if its state cannot be
confirmed, return `publication-unknown` and `unknown`. A later project check
reports a marker without registry as mismatch and does not adopt it. An error after
publication whose effect cannot be established is `publication-unknown`; inspect
the page and registry before retrying. These outcomes do not promise a two-file
transaction or deletion of a pre-existing file. Before publication, rejected and
denied requests leave existing page/registry and outside-target bytes unchanged.

## Compatibility and digest policy

The original resource closures are retained byte-for-byte at
`compatibility/v1.0.0` (digest `1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6`, version 1)
and `compatibility/v1.1.0` (digest `e2b827c6e929f4a8d03344a91ca72f2eb837962e916d5e7f0487e315e1440cef`, version 2).
Use either directory as `profile_root` or `--profile-root` to explicitly select it.
The new pack checks and updates all three exact identities in one registry;
updates preserve original markers, identity and registry bytes. Original packs
can update their own pages in a mixed registry; their whole-project checks can
refuse future identities. No operation silently migrates or adopts a page.

The 1.2 digest is computed over explicit resource paths and exact bytes, including
this guide and every recursively retained closure. Each owning pack validates
its examples independently; the parent hashes their unchanged bytes. A stored
example uses a zero sentinel only in its first marker's contract field. It is
not a template or an acceptable target identity. The two identical bundled v1
copies are the sole permitted duplicate binding. A derived profile must keep one
document root/registry, compatible shared class routes and nonoverlapping routes.
Custom releases require exact verified descriptors; the original custom-v1
explicit selection remains supported. API layer vocabularies and route prefixes
are profile-defined; schema 1.2 permits only the safe component grammar and
finite reserved names, never arbitrary output paths.

## Original retained contract details

The following original shapes and rules continue to apply when selecting the
original 1.0/1.1 pack. The schema 1.2 additions above take precedence for 1.2.

## 1. Scope, selected project, and schemas

This is the cross-boundary contract for the document kit engine (see the selected profile and catalog).
A caller selects one target project root and one versioned profile directory.
The CLI selects that pair per invocation; an MCP process is bound to it at launch
and tool arguments cannot redirect it. Every path is local, POSIX, and relative
to the selected project or pack. Absolute paths, `.` or `..` components,
unexpected separators, and symlink escapes are errors.

This section specifies the original schema `1.0` and `1.1` behavior. The structured contract above governs schema `1.2`; dispatch remains by schema, not
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
are rejected. Raw HTML is limited to exact managed document-kit identity
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

Source and executed evidence, not this page's existence, establish engineering acceptance. DOC-01 through DOC-15 (see the selected profile and catalog)
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

