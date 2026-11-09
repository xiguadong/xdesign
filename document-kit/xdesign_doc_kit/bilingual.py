from __future__ import annotations

import hashlib
import json
import re
from collections import Counter
from pathlib import Path, PurePosixPath
from typing import Any, Mapping

from .compatibility import (
    DocumentKitError, _error, _load_json, _read_pack_file, _reject_symlinks,
    _safe_rel, _text, _unique_pairs, digest_v3, identity, marker,
)

LANGUAGES = ("en", "zh-CN")
DEFAULT_LANGUAGE = "en"
RESOURCE_KINDS = ("template", "guide", "example", "fixture")
FIGURE_INPUT_FIELDS = ("Question", "Model view", "Companions", "Status", "Reading", "Limits")
METADATA_LABELS = {
    "en": {"Status": "Status", "Revision": "Revision", "Model sources": "Model sources"},
    "zh-CN": {"Status": "状态", "Revision": "修订", "Model sources": "模型来源"},
}
FIGURE_LABELS = {
    "en": {"figure_label": "Figure", **{key: key for key in FIGURE_INPUT_FIELDS}},
    "zh-CN": {
        "figure_label": "图", "Question": "问题", "Model view": "模型视图",
        "Companions": "配套材料", "Status": "状态", "Reading": "解读", "Limits": "限制",
    },
}
LANGUAGE_GRAMMAR = re.compile(r"[A-Za-z]{2,3}(?:[-_][A-Za-z0-9]{2,8})*")
BUNDLED_INVENTORY = {
    "generic-requirements": "requirements", "generic-architecture": "architecture",
    "generic-design": "design", "generic-interface": "interface",
    "generic-verification": "verification", "system-requirements": "requirements",
    "change-requirements": "requirements", "system-context": "architecture",
    "component-composition": "architecture", "interaction-flow": "design",
    "state-machine": "design", "data-model": "design", "operation-contract": "interface",
    "data-contract": "interface", "verification-test-plan": "verification",
    "design-review": "verification", "generic-api-design": "api-design",
    "generic-flow-index": "flow-index", "generic-flow-definition": "flow-definition",
    "generic-flow-implementation": "flow-implementation", "generic-flow-views": "flow-views",
}
MIXED_SELECTION = {
    "requirements": ("generic-requirements", "system-requirements"),
    "architecture": ("system-context", "system-context"),
    "design": ("interaction-flow", "interaction-flow"),
    "interface": ("operation-contract", "operation-contract"),
    "verification": ("verification-test-plan", "verification-test-plan"),
    "api-design": ("generic-api-design", "generic-api-design"),
    "flow-index": ("generic-flow-index", "generic-flow-index"),
    "flow-definition": ("generic-flow-definition", "generic-flow-definition"),
    "flow-implementation": ("generic-flow-implementation", "generic-flow-implementation"),
    "flow-views": ("generic-flow-views", "generic-flow-views"),
}
ZERO = "0" * 64


def validate_language_syntax(value: Any) -> None:
    if value is None:
        return
    if not isinstance(value, str) or not value.isascii() or LANGUAGE_GRAMMAR.fullmatch(value) is None:
        raise _error("invalid-request", "language must be a well-formed ASCII language identifier")


def select_language(contract: Mapping[str, Any], value: Any) -> str:
    validate_language_syntax(value)
    policy = contract["profile"]["language_policy"]
    language = policy["default"] if value is None else value
    if language not in policy["supported_ids"]:
        raise _error("unknown-language", f"Unsupported language identifier: {language}")
    return language


def assert_effective_language(contract: Mapping[str, Any], effective: Mapping[str, Any], value: Any) -> str | None:
    validate_language_syntax(value)
    if effective["contract_digest_version"] != "3":
        if value is not None:
            raise _error("invalid-request", "A language assertion cannot be applied to a language-less historical page")
        return None
    persisted = effective.get("language")
    if persisted not in contract["profile"]["language_policy"]["supported_ids"]:
        raise _error("registry-mismatch", "Persisted language is missing or unsupported", "DOC-002")
    if value is not None:
        if value not in contract["profile"]["language_policy"]["supported_ids"]:
            raise _error("unknown-language", f"Unsupported language identifier: {value}")
        if value != persisted:
            raise _error("invalid-request", "language assertion differs from the persisted effective variant")
    return persisted


def language_result(effective: Mapping[str, Any]) -> dict[str, str]:
    return {"language": effective["language"]} if "language" in effective else {}


def field_label(spec: Mapping[str, Any], section_key: str, field: str) -> str:
    section = next(item for item in spec["required_sections"] if item["key"] == section_key)
    return section.get("field_labels", {}).get(field, field.replace("_", " ").capitalize())


def figure_policy(contract: Mapping[str, Any], effective: Mapping[str, Any] | None = None) -> dict[str, Any]:
    result = dict(contract["profile"]["figure_block"])
    language = effective.get("language") if effective is not None else None
    if language is None:
        return result
    labels = FIGURE_LABELS[language]
    result["template"] = result["templates"][language]
    result["ordered_fields"] = [labels[key] for key in FIGURE_INPUT_FIELDS]
    result["field_labels"] = {key: labels[key] for key in FIGURE_INPUT_FIELDS}
    result["figure_label"] = labels["figure_label"]
    result["label_format"] = f"{labels['figure_label']} <section-id>-<ordinal>"
    return result


def metadata_policy(contract: Mapping[str, Any], effective: Mapping[str, Any]) -> dict[str, str]:
    return METADATA_LABELS.get(effective.get("language", "en"), METADATA_LABELS["en"])


def mime_type(path: str) -> str:
    if path.endswith(".json"):
        return "application/json"
    if path.endswith(".svg"):
        return "image/svg+xml"
    return "text/markdown"


def indexes(contract: Mapping[str, Any]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, str]]:
    resources = contract["resources"]
    variant_index = []
    for entry in contract["entries_by_id"].values():
        for language in LANGUAGES:
            variant = entry["variants"][language]
            for kind in RESOURCE_KINDS:
                path = variant["resources"][kind]
                variant_index.append({
                    "template_id": entry["template_id"], "document_class": entry["document_class"],
                    "language": language, "kind": kind, "path": path,
                    "sha256": hashlib.sha256(resources[path]).hexdigest(), "mime_type": mime_type(path),
                })
    shared_index = [
        {"path": path, "sha256": hashlib.sha256(resources[path]).hexdigest(), "mime_type": mime_type(path)}
        for path in sorted(contract["shared_resource_paths"], key=lambda value: value.encode("utf-8"))
    ]
    paths = [item["path"] for item in variant_index] + [item["path"] for item in shared_index]
    return variant_index, shared_index, {path: resources[path].decode("utf-8") for path in paths}


def declared_resource(contract: Mapping[str, Any], template_id: str, language: Any, kind: str) -> tuple[str, bytes]:
    selected = select_language(contract, language)
    if kind not in RESOURCE_KINDS:
        raise DocumentKitError("resource-not-found", f"Unknown resource kind: {kind}")
    entry = contract["entries_by_id"].get(template_id)
    if entry is None:
        raise DocumentKitError("resource-not-found", f"Unknown template ID: {template_id}")
    path = entry["variants"][selected]["resources"][kind]
    return path, contract["resources"][path]


def _validate_sections(value: Any) -> None:
    if not isinstance(value, list) or not value:
        raise _error("invalid-catalog", "Variant required_sections must be a nonempty array", "DOC-002")
    section_keys = set()
    field_keys = set()
    for section in value:
        if not isinstance(section, dict) or set(section) != {"key", "heading", "required_fields", "field_labels"}:
            raise _error("invalid-catalog", "Variant section shape is invalid", "DOC-002")
        key = section["key"]
        if not isinstance(key, str) or re.fullmatch(r"[a-z]+(?:-[a-z]+)*", key) is None or key in section_keys:
            raise _error("invalid-catalog", "Variant section keys are duplicated or malformed", "DOC-002")
        section_keys.add(key)
        _text(section["heading"], "Variant section heading", "invalid-catalog")
        fields = section["required_fields"]
        labels = section["field_labels"]
        if not isinstance(fields, list) or not fields or any(not isinstance(field, str) or not field for field in fields):
            raise _error("invalid-catalog", "Variant required_fields are invalid", "DOC-002")
        if not isinstance(labels, dict) or list(labels) != fields:
            raise _error("invalid-catalog", "Variant field_labels must exactly follow required_fields", "DOC-002")
        if any(not isinstance(label, str) or not label.strip() for label in labels.values()) or len(set(labels.values())) != len(fields):
            raise _error("invalid-catalog", "Variant field labels must be nonempty and distinct within a section", "DOC-002")
        for field in fields:
            if field in field_keys:
                raise _error("invalid-catalog", f"Repeated field token: {field}", "DOC-002")
            field_keys.add(field)


def _validate_template(profile: Mapping[str, Any], entry_id: str, variant: Mapping[str, Any], text: str, language: str) -> None:
    sections = variant["required_sections"]
    keys = [section["key"] for section in sections]
    fields = [field for section in sections for field in section["required_fields"]]
    if re.findall(r"^## (.+)$", text, re.M) != [section["heading"] for section in sections]:
        raise _error("invalid-catalog", f"Template headings differ for {entry_id}/{language}", "DOC-002")
    if re.findall(r"\{\{section_id:([^}]+)\}\}", text) != keys \
            or re.findall(r"\{\{content:([^}]+)\}\}", text) != keys \
            or re.findall(r"\{\{field:([^}]+)\}\}", text) != fields:
        raise _error("invalid-catalog", f"Template section tokens differ for {entry_id}/{language}", "DOC-002")
    expected = ["profile_id", "version", "document_class", "template_id", "language",
                "contract_sha256", "contract_digest_version", "title", "status", "revision", "model_sources"]
    expected += [f"section_id:{key}" for key in keys]
    expected += [f"field:{field}" for field in fields]
    expected += [f"content:{key}" for key in keys]
    if Counter(re.findall(r"\{\{([^{}]+)\}\}", text)) != Counter(expected):
        raise _error("invalid-catalog", f"Template tokens differ for {entry_id}/{language}", "DOC-002")
    marker_template = re.sub(r"\{([^{}]+)\}", lambda match: "{{" + match.group(1) + "}}",
                             profile["validation"]["managed_marker"])
    if not text.startswith(marker_template + "\n"):
        raise _error("invalid-catalog", f"Template marker differs for {entry_id}/{language}", "DOC-002")
    metadata = metadata_policy({}, {"language": language})
    metadata_line = f"> {metadata['Status']}: {{{{status}}}} | {metadata['Revision']}: {{{{revision}}}} | {metadata['Model sources']}: {{{{model_sources}}}}"
    if metadata_line not in text:
        raise _error("invalid-catalog", f"Template metadata labels differ for {entry_id}/{language}", "DOC-002")
    for section in sections:
        for field in section["required_fields"]:
            label = section["field_labels"][field]
            if f"**{label}:** {{{{field:{field}}}}}" not in text:
                raise _error("invalid-catalog", f"Template field label differs for {entry_id}/{language}: {field}", "DOC-002")


def _reject_hash_keys(value: Any) -> None:
    if isinstance(value, dict):
        for key, item in value.items():
            if key in {"contract_sha256", "expected_contract_sha256"}:
                raise _error("invalid-catalog", "Bundled fixtures must not store a contract digest", "DOC-007")
            _reject_hash_keys(item)
    elif isinstance(value, list):
        for item in value:
            _reject_hash_keys(item)


def load_bilingual_contract(root: Path, legacy_loader) -> dict[str, Any]:
    from .catalog import (
        BUNDLED_INVENTORY as OLD_INVENTORY, _markdown_security, _validate_concerns,
        _validate_source, _validate_usage, _string_list,
    )
    from .routes import request_selection, resolve, validate_family, validate_routes

    resources: dict[str, bytes] = {}
    roles: dict[str, str] = {}

    def add(value: Any, role: str, code: str = "invalid-catalog") -> str:
        relative = _safe_rel(value, "Catalog resource", code).as_posix()
        if relative in roles and roles[relative] != role:
            raise _error(code, f"Resource {relative} has conflicting or duplicate roles", "DOC-002")
        roles[relative] = role
        if relative not in resources:
            try:
                data = _read_pack_file(root, relative)
            except DocumentKitError as error:
                raise _error(code, f"Cannot read catalog resource {relative}: {error.message}", "DOC-002") from error
            try:
                data.decode("utf-8")
            except UnicodeError as error:
                raise _error(code, f"Catalog resource is not UTF-8: {relative}", "DOC-002") from error
            resources[relative] = data
        return relative

    def json_resource(value: Any, role: str, code: str = "invalid-catalog") -> Any:
        relative = add(value, role, code)
        try:
            return json.loads(resources[relative].decode("utf-8"), object_pairs_hook=_unique_pairs)
        except (UnicodeError, ValueError) as error:
            raise _error(code, f"Invalid JSON resource {relative}: {error}", "DOC-002") from error

    profile = json_resource("profile.json", "profile", "invalid-profile")
    if not isinstance(profile, dict) or profile.get("schema_version") != "1.3":
        raise _error("invalid-profile", "Unsupported profile schema")
    for key in ("profile_id", "version", "rules_file", "skill_file", "document_classes", "project",
                "anchor_policy", "figure_block", "validation", "catalog", "compatibility_packs", "language_policy"):
        if key not in profile:
            raise _error("invalid-profile", f"Profile is missing {key}")
    if profile["language_policy"] != {"supported_ids": ["en", "zh-CN"], "default": "en"}:
        raise _error("invalid-profile", "Schema 1.3 language_policy must declare exact en/zh-CN IDs and en default")
    if not isinstance(profile["profile_id"], str) or not profile["profile_id"] \
            or not isinstance(profile["version"], str) or not profile["version"]:
        raise _error("invalid-profile", "Profile identity must be nonempty text")
    project = profile["project"]
    if not isinstance(project, dict) or not isinstance(project.get("docs_root"), str):
        raise _error("invalid-profile", "Project documentation root is invalid")
    docs_root = _safe_rel(project["docs_root"], "Documentation root", "invalid-profile").as_posix()
    required_roots = project.get("model_roots")
    optional_roots = project.get("optional_model_roots", [])
    if not isinstance(required_roots, list) or not required_roots or not isinstance(optional_roots, list) \
            or any(not isinstance(item, str) for item in [*required_roots, *optional_roots]):
        raise _error("invalid-profile", "Model roots must be nonempty required and optional path lists")
    if len(set([*required_roots, *optional_roots])) != len(required_roots) + len(optional_roots):
        raise _error("invalid-profile", "Model roots must be unique")
    for path in [*required_roots, *optional_roots]:
        _safe_rel(path, "Model root", "invalid-profile")
    anchors = profile["anchor_policy"]
    if not isinstance(anchors, dict) or not isinstance(anchors.get("format"), str):
        raise _error("invalid-profile", "Anchor policy is invalid")
    try:
        re.compile(anchors["format"])
    except re.error as error:
        raise _error("invalid-profile", f"Anchor format is invalid: {error}") from error
    validation = profile["validation"]
    marker_tokens = {"profile_id", "version", "document_class", "template_id", "language",
                     "contract_sha256", "contract_digest_version"}
    if not isinstance(validation, dict) or not isinstance(validation.get("managed_marker"), str) \
            or set(re.findall(r"\{([^{}]+)\}", validation["managed_marker"])) != marker_tokens \
            or not isinstance(validation.get("placeholder_patterns"), list):
        raise _error("invalid-profile", "Managed validation policy is invalid")
    for pattern in validation["placeholder_patterns"]:
        try:
            re.compile(pattern)
        except (TypeError, re.error) as error:
            raise _error("invalid-profile", f"Invalid placeholder pattern: {error}") from error
    figure = profile["figure_block"]
    if not isinstance(figure, dict) or not isinstance(figure.get("field_tokens"), dict) \
            or set(figure["field_tokens"]) != set(FIGURE_INPUT_FIELDS) \
            or figure["field_tokens"] != {key: key.replace(" ", "_").lower() for key in FIGURE_INPUT_FIELDS} \
            or not isinstance(figure.get("statuses"), list) or not figure["statuses"] \
            or not isinstance(figure.get("templates"), dict) or set(figure["templates"]) != set(LANGUAGES):
        raise _error("invalid-profile", "Bilingual figure block policy is invalid")
    if any(figure["templates"][language] != f"templates/figure-block.{language}.md" for language in LANGUAGES):
        raise _error("invalid-profile", "Figure templates must bind each exact language-owned path")
    classes = profile["document_classes"]
    if not isinstance(classes, dict) or not classes:
        raise _error("invalid-profile", "document_classes must be a nonempty object")
    for class_name, spec in classes.items():
        if not isinstance(spec, dict) or re.fullmatch(r"[a-z]+(?:-[a-z]+)*", class_name) is None:
            raise _error("invalid-profile", f"Invalid class {class_name}")
        for key in ("path_template", "filename_pattern", "default_template_id"):
            if key not in spec:
                raise _error("invalid-profile", f"{class_name} is missing {key}")
        if "template" in spec or "required_sections" in spec:
            raise _error("invalid-profile", f"{class_name} duplicates variant-owned template declarations")
        route = spec["path_template"]
        if not isinstance(route, str) or route.count("{name}") != 1 or not route.endswith(".md"):
            raise _error("invalid-profile", f"Invalid class route {route}")
        _safe_rel(route.replace("{name}", "sample"), "Class route", "invalid-profile")
        if not route.startswith(docs_root + "/"):
            raise _error("invalid-profile", f"Class route escapes documentation root: {route}")
        try:
            re.compile(spec["filename_pattern"])
        except (TypeError, re.error) as error:
            raise _error("invalid-profile", f"Invalid filename pattern in {class_name}: {error}") from error
        if not isinstance(spec["default_template_id"], str) or not spec["default_template_id"]:
            raise _error("template-default", f"Class {class_name} has no generic default", "DOC-002")
    validate_routes(profile)

    manifest_info = profile["catalog"]
    if not isinstance(manifest_info, dict) or set(manifest_info) != {"manifest", "sources", "routing_guide", "mixed_fixture", "entry_root"} \
            or manifest_info["entry_root"] != "catalog/entries":
        raise _error("invalid-profile", "Bilingual catalog declarations are invalid")
    manifest = json_resource(manifest_info["manifest"], "manifest")
    sources_doc = json_resource(manifest_info["sources"], "sources")
    routing_guide = add(manifest_info["routing_guide"], "routing-guide")
    mixed_path = add(manifest_info["mixed_fixture"], "mixed-fixture")
    try:
        mixed_fixture = json.loads(resources[mixed_path].decode("utf-8"), object_pairs_hook=_unique_pairs)
    except ValueError as error:
        raise _error("invalid-catalog", f"Invalid mixed fixture: {error}", "DOC-007") from error
    if not isinstance(manifest, dict) or set(manifest) != {"schema_version", "profile_id", "version", "entries"} \
            or manifest.get("schema_version") != "1.3" or manifest.get("profile_id") != profile["profile_id"] \
            or manifest.get("version") != profile["version"] or not isinstance(manifest.get("entries"), list):
        raise _error("invalid-catalog", "Bilingual manifest identity or shape is invalid", "DOC-002")
    if not isinstance(sources_doc, dict) or set(sources_doc) != {"sources"} or not isinstance(sources_doc.get("sources"), list):
        raise _error("invalid-source", "Source register shape is invalid", "DOC-006")
    source_records = sources_doc["sources"]
    source_ids = set()
    source_locals = []
    for record in source_records:
        _validate_source(record)
        if record["source_id"] in source_ids:
            raise _error("invalid-source", f"Duplicate source ID: {record['source_id']}", "DOC-006")
        source_ids.add(record["source_id"])
        if record["local_path"] is not None:
            source_locals.append(record["local_path"])
    bundled = profile["profile_id"] == "xdesign-sysml-docs" and profile["version"] == "1.3.0"
    entries = {}
    fixtures_by_id = {}
    variant_resource_paths = set()
    shared_paths = {manifest_info["sources"], routing_guide, mixed_path, *figure["templates"].values()}
    variant_selection_keys = {"title", "scenario", "when_to_use", "when_not_to_use", "required_inputs", "dependencies",
                              "required_sections", "view_concerns", "resources"}
    for entry in manifest["entries"]:
        if not isinstance(entry, dict) or set(entry) != {"template_id", "document_class", "sources", "checks", "variants"}:
            raise _error("invalid-catalog", "Bilingual entry fields differ from the required set", "DOC-002")
        entry_id = entry["template_id"]
        if not isinstance(entry_id, str) or re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", entry_id) is None or entry_id in entries:
            raise _error("invalid-catalog", f"Duplicate or invalid template ID: {entry_id}", "DOC-002")
        if not isinstance(entry["document_class"], str) or entry["document_class"] not in classes:
            raise _error("invalid-catalog", f"Unknown entry class for {entry_id}", "DOC-002")
        for usage in entry["sources"] if isinstance(entry["sources"], list) else []:
            _validate_usage(usage, source_ids)
        if not isinstance(entry["sources"], list) or not entry["sources"]:
            raise _error("invalid-source", f"Entry {entry_id} has no sources", "DOC-006")
        checks = _string_list(entry["checks"], "Entry checks", False)
        from .catalog import CHECK_IDS
        if not set(checks).issubset(CHECK_IDS):
            raise _error("invalid-catalog", f"Unimplemented check IDs for {entry_id}", "DOC-002")
        variants = entry["variants"]
        if not isinstance(variants, dict) or set(variants) != set(LANGUAGES):
            raise _error("invalid-catalog", f"Entry {entry_id} must declare exact en and zh-CN variants", "DOC-002")
        fixtures_by_id[entry_id] = {}
        for language in LANGUAGES:
            variant = variants[language]
            if not isinstance(variant, dict) or set(variant) != variant_selection_keys:
                raise _error("invalid-catalog", f"Variant fields differ for {entry_id}/{language}", "DOC-002")
            for key in ("title", "scenario"):
                _text(variant[key], key, "invalid-catalog")
            for key in ("when_to_use", "when_not_to_use"):
                _string_list(variant[key], key, False)
            for key in ("required_inputs", "dependencies"):
                _string_list(variant[key], key)
            _validate_sections(variant["required_sections"])
            _validate_concerns(variant["view_concerns"], variant["required_sections"], True)
            declarations = variant["resources"]
            if not isinstance(declarations, dict) or set(declarations) != set(RESOURCE_KINDS):
                raise _error("invalid-catalog", f"Variant resources differ for {entry_id}/{language}", "DOC-002")
            for kind in RESOURCE_KINDS:
                extension = "json" if kind == "fixture" else "md"
                expected = f"catalog/entries/{entry_id}/{language}/{kind}.{extension}"
                if declarations[kind] != expected:
                    raise _error("invalid-catalog", f"Declared path differs from canonical variant path: {declarations[kind]}", "DOC-002")
                path = add(declarations[kind], f"variant:{entry_id}:{language}:{kind}")
                variant_resource_paths.add(path)
            for kind in ("template", "guide", "example"):
                _markdown_security(resources[declarations[kind]].decode("utf-8"), declarations[kind])
            _validate_template(profile, entry_id, variant, resources[declarations["template"]].decode("utf-8"), language)
            try:
                fixture = json.loads(resources[declarations["fixture"]].decode("utf-8"), object_pairs_hook=_unique_pairs)
            except ValueError as error:
                raise _error("invalid-catalog", f"Invalid fixture for {entry_id}/{language}: {error}", "DOC-007") from error
            expected_keys = {"profile_id", "version", "contract_digest_version", "language", "document_class", "name", "layer",
                             "template_id", "metadata", "sections", "models", "figures", "expected_document"}
            if not isinstance(fixture, dict) or set(fixture) != expected_keys:
                raise _error("invalid-catalog", f"Fixture fields differ for {entry_id}/{language}", "DOC-007")
            if fixture["profile_id"] != profile["profile_id"] or fixture["version"] != profile["version"] \
                    or fixture["contract_digest_version"] != "3" or fixture["language"] != language \
                    or fixture["document_class"] != entry["document_class"] or fixture["template_id"] != entry_id \
                    or fixture["expected_document"] != declarations["example"]:
                raise _error("invalid-catalog", f"Fixture identity differs for {entry_id}/{language}", "DOC-007")
            _reject_hash_keys(fixture)
            fixtures_by_id[entry_id][language] = fixture
        en_sections = variants["en"]["required_sections"]
        zh_sections = variants["zh-CN"]["required_sections"]
        if [(s["key"], s["required_fields"]) for s in en_sections] != [(s["key"], s["required_fields"]) for s in zh_sections]:
            raise _error("invalid-catalog", f"Variant section/field machine equivalence fails for {entry_id}", "DOC-002")
        for language in LANGUAGES:
            concerns = [{k: v for k, v in concern.items() if k != "trigger"} for concern in variants[language]["view_concerns"]]
            if language == "en":
                en_concerns = concerns
            elif concerns != en_concerns:
                raise _error("invalid-catalog", f"Variant concern machine equivalence fails for {entry_id}", "DOC-002")
        entries[entry_id] = entry

    if bundled:
        actual_inventory = {entry_id: entry["document_class"] for entry_id, entry in entries.items()}
        if actual_inventory != BUNDLED_INVENTORY:
            raise _error("invalid-catalog", "Bundled 21-entry bilingual inventory is invalid", "DOC-002")
        cited = {usage["source_id"] for entry in manifest["entries"] for usage in entry["sources"]}
        if not {"DPR", "GOOD-DOCS", "GITHUB-DOCS", "SPECD", "C4", "OPENSPEC", "ARC42", "REQ42"}.issubset(cited):
            raise _error("invalid-catalog", "Bundled eight-source coverage is incomplete", "DOC-002")
    for class_name, spec in classes.items():
        default_id = spec["default_template_id"]
        if default_id not in entries:
            raise _error("template-default", f"Unknown default template {default_id}", "DOC-002")
        if entries[default_id]["document_class"] != class_name or not default_id.startswith("generic-"):
            raise _error("template-default", f"Default template is not a class generic: {default_id}", "DOC-002")

    def validate_inputs(value: Any, label: str, figures: bool, mixed: bool = False) -> None:
        if not isinstance(value, list):
            raise _error("invalid-catalog", f"{label} must be an array", "DOC-007")
        required = {"pack", "target", "title", *FIGURE_INPUT_FIELDS} if figures else {"pack", "target"}
        if mixed and figures:
            required.add("language")
        for item in value:
            if not isinstance(item, dict) or set(item) != required:
                raise _error("invalid-catalog", f"{label} input shape is invalid", "DOC-007")
            shared_paths.add(add(item["pack"], "figure" if figures else "model"))
            _safe_rel(item["target"], f"{label} target")
            if figures:
                for key in required - {"pack", "target", "language"}:
                    _text(item[key], f"{label}.{key}", "invalid-catalog")
                if mixed:
                    select_language({"profile": profile}, item["language"])

    def validate_caller(document: Any, entry: Mapping[str, Any], language: str, models: list[dict[str, Any]]) -> None:
        if not isinstance(document, dict):
            raise _error("invalid-catalog", "Fixture caller input must be an object", "DOC-007")
        entry_id = entry["template_id"]
        try:
            resolve(profile, request_selection(document["document_class"], document["name"], document["layer"]))
        except DocumentKitError as error:
            raise _error("invalid-catalog", f"Invalid fixture route: {error.message}") from error
        variant = entry["variants"][language]
        metadata = document["metadata"]
        if not isinstance(metadata, dict) or set(metadata) != {"title", "status", "revision", "model_sources"}:
            raise _error("invalid-catalog", f"Fixture metadata is invalid for {entry_id}/{language}", "DOC-007")
        for key in ("title", "status", "revision"):
            _text(metadata[key], f"{entry_id}.{key}", "invalid-catalog")
        if not isinstance(metadata["model_sources"], list) or not metadata["model_sources"]:
            raise _error("invalid-catalog", f"Fixture model_sources is invalid for {entry_id}/{language}", "DOC-007")
        model_targets = {item["target"] for item in models}
        if any(not isinstance(item, str) or item not in model_targets for item in metadata["model_sources"]):
            raise _error("invalid-catalog", f"Fixture model source is not materialized for {entry_id}/{language}", "DOC-007")
        sections = document["sections"]
        required_sections = variant["required_sections"]
        if not isinstance(sections, dict) or set(sections) != {item["key"] for item in required_sections}:
            raise _error("invalid-catalog", f"Fixture sections differ for {entry_id}/{language}", "DOC-007")
        for section in required_sections:
            value = sections[section["key"]]
            if not isinstance(value, dict) or set(value) != {"anchor", "content", "fields"}:
                raise _error("invalid-catalog", f"Fixture section shape differs for {entry_id}/{language}", "DOC-007")
            if not isinstance(value["anchor"], str) or re.fullmatch(profile["anchor_policy"]["format"], value["anchor"]) is None:
                raise _error("invalid-catalog", f"Fixture anchor is invalid for {entry_id}/{language}", "DOC-007")
            _text(value["content"], f"{entry_id}.{section['key']}.content", "invalid-catalog")
            fields = value["fields"]
            if not isinstance(fields, dict) or set(fields) != set(section["required_fields"]):
                raise _error("invalid-catalog", f"Fixture fields differ for {entry_id}/{language}.{section['key']}", "DOC-007")
            for field, field_value in fields.items():
                _text(field_value, f"{entry_id}.{field}", "invalid-catalog")
                if field == "diagram_decisions":
                    try:
                        decisions = json.loads(field_value)
                    except ValueError as error:
                        raise _error("invalid-catalog", f"Fixture diagram decisions are malformed for {entry_id}/{language}", "DOC-007") from error
                    if not isinstance(decisions, list):
                        raise _error("invalid-catalog", f"Fixture diagram decisions must be an array for {entry_id}/{language}", "DOC-007")

    for entry_id, entry in entries.items():
        for language in LANGUAGES:
            fixture = fixtures_by_id[entry_id][language]
            validate_inputs(fixture["models"], f"{entry_id}/{language}.models", False)
            validate_inputs(fixture["figures"], f"{entry_id}/{language}.figures", True)
            validate_caller(fixture, entry, language, fixture["models"])

    expected_mixed_keys = {"profile_id", "version", "contract_digest_version", "documents", "coverage", "models", "figures"}
    if not isinstance(mixed_fixture, dict) or set(mixed_fixture) != expected_mixed_keys \
            or mixed_fixture["profile_id"] != profile["profile_id"] or mixed_fixture["version"] != profile["version"] \
            or mixed_fixture["contract_digest_version"] != "3" or not isinstance(mixed_fixture["documents"], list):
        raise _error("invalid-catalog", "Bilingual mixed fixture identity or shape is invalid", "DOC-007")
    _reject_hash_keys(mixed_fixture)
    coverage = mixed_fixture["coverage"]
    if not isinstance(coverage, dict) or set(coverage) != {"pack", "target"}:
        raise _error("invalid-catalog", "Mixed coverage shape is invalid", "DOC-007")
    coverage_path = add(coverage["pack"], "coverage")
    shared_paths.add(coverage_path)
    _safe_rel(coverage["target"], "Mixed coverage target")
    _markdown_security(resources[coverage_path].decode("utf-8"), coverage_path)
    _markdown_security(resources[routing_guide].decode("utf-8"), routing_guide)
    mixed_anchors = set()
    for document in mixed_fixture["documents"]:
        required = {"document_class", "name", "layer", "template_id", "language", "metadata", "sections", "expected_document"}
        if not isinstance(document, dict) or set(document) != required:
            raise _error("invalid-catalog", "Mixed document input shape is invalid", "DOC-007")
        language = select_language({"profile": profile}, document["language"])
        entry = entries.get(document["template_id"])
        if entry is None or entry["document_class"] != document["document_class"] \
                or document["expected_document"] != entry["variants"][language]["resources"]["example"]:
            raise _error("invalid-catalog", "Mixed document identity or same-language example reference is invalid", "DOC-007")
        validate_caller(document, entry, language, mixed_fixture["models"])
        for section in document["sections"].values():
            anchor = section["anchor"].casefold()
            if anchor in mixed_anchors:
                raise _error("invalid-catalog", f"Mixed fixture repeats anchor {section['anchor']}", "DOC-007")
            mixed_anchors.add(anchor)
    if bundled:
        documents = mixed_fixture["documents"]
        actual = {(d["document_class"], d["language"]): d["template_id"] for d in documents}
        expected = {(class_name, language): template_id for class_name, pair in MIXED_SELECTION.items()
                    for language, template_id in zip(LANGUAGES, pair)}
        if len(documents) != 20 or actual != expected:
            raise _error("invalid-catalog", "Bundled mixed fixture does not match the exact 20-document bilingual allocation", "DOC-007")
    validate_inputs(mixed_fixture["models"], "mixed.models", False)
    validate_inputs(mixed_fixture["figures"], "mixed.figures", True, mixed=True)

    add(profile["rules_file"], "rules", "invalid-profile")
    add(profile["skill_file"], "skill", "invalid-profile")
    add("README.md", "readme", "invalid-profile")
    add("API.md", "consumer-api", "invalid-profile")
    for language in LANGUAGES:
        add(figure["templates"][language], "figure-template", "invalid-profile")
    for local_path in source_locals:
        shared_paths.add(add(local_path, "source-local"))
    owned_paths = set(resources)

    compatibility_contracts = []
    descriptor_pairs = []
    if not isinstance(profile["compatibility_packs"], list):
        raise _error("invalid-profile", "compatibility_packs must be an array")
    child_roots = set()
    for descriptor in profile["compatibility_packs"]:
        if not isinstance(descriptor, dict) or set(descriptor) != {"profile_id", "version", "root", "contract_sha256", "contract_digest_version"}:
            raise _error("invalid-profile", "Compatibility descriptor shape is invalid")
        compatibility_root = _safe_rel(descriptor["root"], "Compatibility root", "invalid-profile").as_posix()
        _reject_symlinks(root, PurePosixPath(compatibility_root))
        if compatibility_root in child_roots or any(compatibility_root.startswith(r + "/") or r.startswith(compatibility_root + "/") for r in child_roots):
            raise _error("invalid-profile", "Overlapping compatibility roots")
        child_roots.add(compatibility_root)
        compatibility = legacy_loader(root / compatibility_root)
        compatibility_contracts.append(compatibility)
        descriptor_pairs.append((descriptor, compatibility, compatibility_root))
        for relative, value in compatibility["resources"].items():
            data = value[1] if isinstance(value, tuple) else value
            path = add(f"{compatibility_root}/{relative}", "compatibility")
            if resources[path] != data:
                raise _error("invalid-catalog", f"Compatibility bytes differ after loading: {path}", "DOC-002")

    for relative in owned_paths:
        if relative.endswith(".md"):
            _markdown_security(resources[relative].decode("utf-8"), relative)
    contract_sha256 = digest_v3(resources)
    if not isinstance(contract_sha256, str) or re.fullmatch(r"[0-9a-f]{64}", contract_sha256) is None or contract_sha256 == ZERO:
        raise _error("invalid-catalog", "Recomputed contract digest is zero or malformed", "DOC-002")
    fixture_paths = {entry["variants"][language]["resources"]["fixture"] for entry in manifest["entries"] for language in LANGUAGES} | {mixed_path}
    for path in fixture_paths:
        if contract_sha256.encode("ascii") in resources[path] or ZERO.encode("ascii") in resources[path]:
            raise _error("invalid-catalog", f"Bundled fixture stores a contract digest or sentinel: {path}", "DOC-007", path)
    for entry in manifest["entries"]:
        for language in LANGUAGES:
            path = entry["variants"][language]["resources"]["example"]
            data = resources[path]
            expected_marker = marker({
                "profile_id": profile["profile_id"], "version": profile["version"],
                "document_class": entry["document_class"], "template_id": entry["template_id"],
                "language": language, "contract_sha256": ZERO, "contract_digest_version": "3",
            })
            if not data.startswith((expected_marker + "\n").encode("utf-8")) \
                    or data.count(ZERO.encode("ascii")) != 1 or contract_sha256.encode("ascii") in data:
                raise _error("invalid-catalog", f"Stored example marker or sentinel is invalid: {path}", "DOC-002", path, 1)
    example_paths = {entry["variants"][language]["resources"]["example"] for entry in manifest["entries"] for language in LANGUAGES}
    for path in owned_paths:
        if path not in example_paths and ZERO.encode("ascii") in resources[path]:
            raise _error("invalid-catalog", f"Stored-example sentinel occurs outside its contract field: {path}", "DOC-002", path)
    for descriptor, compatibility, compatibility_root in descriptor_pairs:
        if compatibility["profile"]["profile_id"] != descriptor["profile_id"] \
                or compatibility["profile"]["version"] != descriptor["version"] \
                or compatibility["contract_sha256"] != descriptor["contract_sha256"] \
                or compatibility["contract_digest_version"] != descriptor["contract_digest_version"]:
            raise _error("contract-drift", f"Compatibility contract differs from descriptor: {compatibility_root}")
    if bundled:
        expected_bindings = {
            "compatibility/v1.0.0": ("xdesign-sysml-docs", "1.0.0", "1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6", "1"),
            "compatibility/v1.1.0": ("xdesign-sysml-docs", "1.1.0", "e2b827c6e929f4a8d03344a91ca72f2eb837962e916d5e7f0487e315e1440cef", "2"),
            "compatibility/v1.2.0": ("xdesign-sysml-docs", "1.2.0", "a4ab976f05769bff34f3b6d3acd0de1dafb728dcf372d099fc3ac938758cf698", "2"),
        }
        actual_bindings = {d["root"]: tuple(d[k] for k in ("profile_id", "version", "contract_sha256", "contract_digest_version"))
                           for d in profile["compatibility_packs"]}
        if actual_bindings != expected_bindings:
            raise _error("contract-drift", "Bundled compatibility bindings differ from the exact original releases")
    validate_family(profile, compatibility_contracts)

    figure_templates = {}
    for language in LANGUAGES:
        path = figure["templates"][language]
        text = resources[path].decode("utf-8")
        labels = FIGURE_LABELS[language]
        if re.findall(r"^> \*\*([^*:]+):\*\*", text, re.M) != [labels[key] for key in FIGURE_INPUT_FIELDS]:
            raise _error("invalid-profile", f"Figure template fields differ for {language}")
        expected_tokens = Counter(["asset", "title", "title", "section_id", "ordinal",
                                   *figure["field_tokens"].values()])
        if Counter(re.findall(r"\{\{([^{}]+)\}\}", text)) != expected_tokens:
            raise _error("invalid-profile", f"Figure template tokens differ for {language}")
        if not text.startswith(f"![{{{{title}}}}]({{{{asset}}}})\n\n> **{labels['figure_label']} "):
            raise _error("invalid-profile", f"Figure template label differs for {language}")
        figure_templates[language] = text
    templates = {}
    for class_name, spec in classes.items():
        entry = entries[spec["default_template_id"]]
        path = entry["variants"]["en"]["resources"]["template"]
        templates[class_name] = resources[path].decode("utf-8")
    templates["figure-block"] = figure_templates["en"]
    contract = {
        "root": root, "profile": profile, "rules": resources[profile["rules_file"]].decode("utf-8"),
        "templates": templates, "figure_templates": figure_templates,
        "resources": resources, "contract_sha256": contract_sha256, "contract_digest_version": "3",
        "entries_by_id": entries, "fixtures_by_id": fixtures_by_id, "mixed_fixture": mixed_fixture,
        "compatibility_contracts": compatibility_contracts, "shared_resource_paths": shared_paths,
        "variant_resource_paths": variant_resource_paths, "metadata_labels": METADATA_LABELS, "figure_labels": FIGURE_LABELS,
    }
    resource_index, shared_index, resource_bytes = indexes(contract)
    contract["resource_index"] = resource_index
    contract["shared_resource_index"] = shared_index
    contract["resource_bytes"] = resource_bytes
    contract["catalog"] = {
        "schema_version": "1.3", "profile_id": profile["profile_id"], "version": profile["version"],
        "language_policy": profile["language_policy"], "entries": manifest["entries"], "sources": source_records,
        "resource_index": resource_index, "shared_resource_index": shared_index,
    }
    return contract
