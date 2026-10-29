# Document kit

This directory is a versioned authoring contract for SysML-backed engineering
documents. Copy or pin the complete directory in a consumer project. Its
`profile.json` supplies the allowed document paths, exact ordered chapters,
required fields, anchor policy, and figure block. `RULES.md` explains the
rules, `templates/` supplies the Markdown, and `SKILL.md` gives an agent the
design-before-implementation sequence. The Python package uses one core for
the JSON CLI and stdio MCP server; it does not generate engineering claims.

| Path | Purpose |
| --- | --- |
| `profile.json` | Versioned, machine-readable document contract |
| `RULES.md` | Normative DOC-001..010 explanations and evidence limits |
| `templates/*.md` | Five class templates and the exact figure block |
| `SKILL.md` | Provider-neutral authoring and check sequence |
| `xdesign_doc_kit/` | Shared authoring/checking core, CLI, and MCP adapter |
| `tests/` | Cross-project acceptance and transport checks |

## Use in another project

Keep this pack at a stable version and select a project root containing its own
`mbse/model/` and `docs/`. Adapt the profile in a reviewed derived pack if its
routes or model roots differ. This starter profile covers five flat classes:
requirements, architecture, design, interface, and verification. It does not
generate xdesign's flow-specific or per-API page layouts without a derived
profile. The selected pack's content digest should be retained with the wish
whose design it governs.

Install the package with Python 3.11 or newer. Keep the environment and pip
cache under the consumer project's `cache/`:

```sh
python3.11 -m venv /path/to/project/cache/document-kit-venv
PIP_CACHE_DIR=/path/to/project/cache/pip \
  /path/to/project/cache/document-kit-venv/bin/pip install '/path/to/document-kit[mcp]'
```

The CLI accepts one target project and profile per invocation. It emits a JSON
result; a rejected operation or failed check exits 1, and an invalid invocation
exits 2. For example:

```sh
/path/to/project/cache/document-kit-venv/bin/xdesign-doc-kit \
  plan-document --target-root /path/to/project \
  --profile-root /path/to/document-kit \
  --class requirements --name payment-boundary

/path/to/project/cache/document-kit-venv/bin/xdesign-doc-kit \
  check-documents --target-root /path/to/project \
  --profile-root /path/to/document-kit
```

The stdio MCP server binds one project root and one pack at launch. A client
configuration can use this command and arguments:

```json
{
  "command": "/path/to/project/cache/document-kit-venv/bin/xdesign-doc-mcp",
  "args": [
    "--project-root", "/path/to/project",
    "--profile", "/path/to/document-kit"
  ]
}
```

The server exposes `get_contract`, `plan_document`, `create_document`,
`update_section`, `attach_figure`, and `check_documents` as tools. It also
exposes the contract and rules as resources and one resource template for each
class's Markdown template. An agent using GPT, Claude, Kimi, DeepSeek, GLM, or
another MCP-capable host can follow `SKILL.md` and call the same operations.
The host must support local stdio MCP servers; copy the skill text into the
host's skill or instruction mechanism when needed.

Read `profile.json`, `RULES.md`, and `SKILL.md` before issuing writes.
`plan_document` gives the only allowed destination and lists every required
section and field. `create_document` takes a JSON input file with `metadata`
(`title`, `status`, `revision`, and a nonempty `model_sources` path list) and a
`sections` object keyed by every required section. Each section supplies an
explicit `anchor`, nonempty `content`, and its required `fields`. The CLI's
`--input` file for `update_section` supplies `content`, `expected_sha256`, and
optional `fields`; the file for `attach_figure` supplies `figure` and
`expected_sha256`. MCP tools take the equivalent structured arguments.
The xdesign repository maintains the [exact request and result design contract](../docs/interfaces/document-kit-api.md#i20).
`update_section` and `attach_figure` require the current document SHA-256.
`check_documents` reports structural findings and leaves semantic review
explicit. Run the consumer project's SysML compiler and strict documentation
build separately. Bind `check_documents` into its design-stage gate or CI when
failed documents must prevent implementation.

In xdesign, the existing documentation audit and strict build remain separate:
`python3 -B tools/check_documentation.py --json` and
`python3 -B tools/docs.py build`. The profile does not rewrite existing pages
to fit a new template; registered managed pages receive its exact chapter and
field checks, while existing pages retain project-level checks.
