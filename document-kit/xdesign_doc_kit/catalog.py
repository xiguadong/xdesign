from __future__ import annotations

import html
import json
import re
from pathlib import Path, PurePosixPath
from typing import Any, Mapping
from urllib.parse import urlsplit

from .compatibility import (
    DocumentKitError, _error, _load_json, _read_pack_file, _root, _safe_rel,
    _text, _unique_pairs, digest_v2, identity, marker,
)


ENTRY_FIELDS = {
    "template_id", "document_class", "title", "scenario", "when_to_use", "when_not_to_use",
    "required_inputs", "dependencies", "required_sections", "template", "guide", "example",
    "fixture", "sources", "checks", "view_concerns",
}
CHECK_IDS = {
    "route", "headings", "fields", "anchors", "placeholders", "links", "model-locators",
    "figure-blocks", "diagram-decisions", "sources", "license", "markdown-security", "fixtures",
}
SOURCE_USAGE_FIELDS = {"source_id", "copying_decision", "copying_review"}
DECISION_FIELDS = {"concern", "applies", "choice", "rationale", "asset", "model_view"}
CHOICES = {"figure", "table", "prose", "not-applicable"}


def _resource(root: Path, relative: Any, resources: dict[str, bytes]) -> str:
    path = _safe_rel(relative, "Catalog resource", "invalid-catalog").as_posix()
    if path not in resources:
        resources[path] = _read_pack_file(root, path)
    return path


def _json_resource(root: Path, relative: str, resources: dict[str, bytes]) -> Any:
    _resource(root, relative, resources)
    return _load_json(root, relative, "invalid-catalog")


def _nonempty_list(value: Any, label: str, allow_empty: bool = False) -> list[Any]:
    if not isinstance(value, list) or (not allow_empty and not value):
        raise _error("invalid-catalog", f"{label} must be a nonempty list", "DOC-002")
    return value


def _string_list(value: Any, label: str, allow_empty: bool = True) -> list[str]:
    values = _nonempty_list(value, label, allow_empty)
    if any(not isinstance(item, str) or not item.strip() for item in values):
        raise _error("invalid-catalog", f"{label} must contain nonempty strings", "DOC-002")
    return values


def _safe_url(value: Any) -> str:
    if not isinstance(value, str) or not value or any(ord(char) < 32 or ord(char) == 127 for char in value):
        raise _error("unsafe-url", "URL is empty or contains control characters", "DOC-006")
    try:
        parsed = urlsplit(value)
    except ValueError as error:
        raise _error("unsafe-url", f"URL is malformed: {value}", "DOC-006") from error
    if parsed.scheme.lower() not in ("http", "https") or not parsed.netloc or parsed.username or parsed.password:
        raise _error("unsafe-url", f"URL must be absolute HTTP(S) without credentials: {value}", "DOC-006")
    if "\\" in value:
        raise _error("unsafe-url", f"URL contains a backslash: {value}", "DOC-006")
    return value


def _validate_source(record: Any) -> None:
    if not isinstance(record, dict):
        raise _error("invalid-source", "Source record must be an object", "DOC-006")
    required = {"source_id", "title", "publisher", "retrieved_at", "ideas_used", "limitations",
                "url", "local_path", "additional_urls", "license_name", "license_version", "license_url"}
    if set(record) != required:
        raise _error("invalid-source", "Source record fields differ from the required set", "DOC-006")
    for key in ("source_id", "title", "publisher", "retrieved_at", "license_name", "license_version"):
        _text(record[key], key, "invalid-source")
    _string_list(record["ideas_used"], "Source ideas_used", False)
    _string_list(record["limitations"], "Source limitations", False)
    if (record["url"] is None) == (record["local_path"] is None):
        raise _error("invalid-source", "Exactly one primary url or local_path is required", "DOC-006")
    if record["url"] is not None:
        _safe_url(record["url"])
    if record["local_path"] is not None:
        _safe_rel(record["local_path"], "Source local_path")
    _nonempty_list(record["additional_urls"], "Source additional_urls", True)
    for url in record["additional_urls"]:
        _safe_url(url)
    unresolved = record["license_name"] in ("unknown", "unlicensed") or record["license_version"] in ("unknown", "unlicensed")
    if record["license_url"] is None:
        if not unresolved:
            raise _error("invalid-license", "license_url may be null only when license identity is unresolved", "DOC-006")
    else:
        if unresolved:
            raise _error("invalid-license", "license_url must be null when license identity is unresolved", "DOC-006")
        _safe_url(record["license_url"])


def _validate_usage(usage: Any, source_ids: set[str]) -> None:
    if not isinstance(usage, dict) or set(usage) != SOURCE_USAGE_FIELDS:
        raise _error("invalid-source", "Entry-source usage fields are invalid", "DOC-006")
    if not isinstance(usage["source_id"], str) or not usage["source_id"]:
        raise _error("invalid-source", "source_id must be nonempty text", "DOC-006")
    if usage["source_id"] not in source_ids:
        raise _error("invalid-source", f"Unknown source ID: {usage['source_id']}", "DOC-006")
    if usage["copying_decision"] not in ("not-copied", "copied"):
        raise _error("invalid-source", "copying_decision must be not-copied or copied", "DOC-006")
    if usage["copying_decision"] == "copied":
        review = usage["copying_review"]
        if not isinstance(review, dict) or set(review) != {"reviewer", "reviewed_at", "permission", "evidence"}:
            raise _error("copying-review-required", "Copied material needs reviewer/date/permission/evidence", "DOC-006")
        for key in review:
            _text(review[key], f"copying_review.{key}", "copying-review-required")
    elif usage["copying_review"] is not None:
        raise _error("invalid-source", "copying_review must be null for not-copied", "DOC-006")


def _validate_sections(value: Any) -> None:
    sections = _nonempty_list(value, "required_sections")
    keys: set[str] = set()
    fields: set[str] = set()
    for section in sections:
        if not isinstance(section, dict) or set(section) != {"key", "heading", "required_fields"}:
            raise _error("invalid-catalog", "Section shape is invalid", "DOC-002")
        key = section["key"]
        if not isinstance(key, str) or re.fullmatch(r"[a-z]+(?:-[a-z]+)*", key) is None or key in keys:
            raise _error("invalid-catalog", f"Duplicate or invalid section key: {key}", "DOC-002")
        keys.add(key)
        _text(section["heading"], "Section heading", "invalid-catalog")
        for field in _string_list(section["required_fields"], "required_fields"):
            if field in fields:
                raise _error("invalid-catalog", f"Repeated field token: {field}", "DOC-002")
            fields.add(field)


def _validate_concerns(value: Any, sections: list[dict[str, Any]]) -> None:
    concerns = _nonempty_list(value, "view_concerns", True)
    section_keys = {section["key"] for section in sections}
    names: set[str] = set()
    for concern in concerns:
        if not isinstance(concern, dict) or set(concern) != {"concern", "section_key", "applicability", "trigger", "alternatives"}:
            raise _error("invalid-catalog", "View concern shape is invalid", "DOC-002")
        name = concern["concern"]
        if not isinstance(name, str) or not name or name in names:
            raise _error("invalid-catalog", f"Duplicate or empty view concern: {name}", "DOC-002")
        names.add(name)
        if not isinstance(concern["section_key"], str) or not concern["section_key"]:
            raise _error("invalid-catalog", "Concern section_key must be nonempty text", "DOC-002")
        if concern["section_key"] not in section_keys:
            raise _error("invalid-catalog", f"Concern section is unknown: {concern['section_key']}", "DOC-002")
        if concern["applicability"] not in ("expected", "conditional", "not-applicable"):
            raise _error("invalid-catalog", "Invalid view applicability", "DOC-002")
        if concern["applicability"] == "conditional":
            _text(concern["trigger"], "Conditional trigger", "invalid-catalog")
        alternatives = set(_string_list(concern["alternatives"], "View alternatives", False))
        if not alternatives or not alternatives.issubset(CHOICES):
            raise _error("invalid-catalog", f"Invalid view alternatives: {alternatives}", "DOC-002")


def _validate_entry(entry: Any, classes: Mapping[str, Any], source_ids: set[str], bundled: bool) -> None:
    if not isinstance(entry, dict) or set(entry) != ENTRY_FIELDS:
        raise _error("invalid-catalog", "Catalog entry fields differ from the required set", "DOC-002")
    if not isinstance(entry["template_id"], str) or re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", entry["template_id"]) is None:
        raise _error("invalid-catalog", f"Invalid template ID: {entry['template_id']}", "DOC-002")
    if not isinstance(entry["document_class"], str) or not entry["document_class"]:
        raise _error("invalid-catalog", "Entry document_class must be nonempty text", "DOC-002")
    if entry["document_class"] not in classes:
        raise _error("invalid-catalog", f"Unknown document class: {entry['document_class']}", "DOC-002")
    for key in ("title", "scenario"):
        _text(entry[key], key, "invalid-catalog")
    for key in ("when_to_use", "when_not_to_use"):
        _string_list(entry[key], key, False)
    for key in ("required_inputs", "dependencies"):
        _string_list(entry[key], key)
    _validate_sections(entry["required_sections"])
    for key in ("template", "guide", "example", "fixture"):
        _safe_rel(entry[key], f"Entry {key}", "invalid-catalog")
    for usage in _nonempty_list(entry["sources"], "Entry sources"):
        _validate_usage(usage, source_ids)
    checks = set(_string_list(entry["checks"], "Entry checks", False))
    if not checks or not checks.issubset(CHECK_IDS):
        raise _error("invalid-catalog", f"Unimplemented check IDs: {sorted(checks - CHECK_IDS)}", "DOC-002")
    _validate_concerns(entry["view_concerns"], entry["required_sections"])
    if bundled and not any(usage["source_id"] for usage in entry["sources"]):
        raise _error("invalid-source", "Bundled entry has no source", "DOC-006")


def _safe_markdown_destination(destination: Any, image: bool = False, angle: bool = False) -> None:
    if not isinstance(destination, str):
        raise _error("unsafe-url", "Markdown destination must be text", "DOC-006")
    if re.search(r"&(?:#0+|#x0+);", destination, re.I):
        raise _error("unsafe-url", "Markdown destination contains an encoded NUL", "DOC-006")
    destination = html.unescape(destination).strip()
    if not destination or any(ord(char) < 32 or ord(char) == 127 for char in destination) \
            or (not angle and any(char.isspace() for char in destination)) \
            or (angle and any(char.isspace() and char != " " for char in destination)):
        raise _error("unsafe-url", "Markdown destination is empty or contains unsupported whitespace or controls", "DOC-006")
    scheme = re.match(r"^([A-Za-z][A-Za-z0-9+.-]*):", destination)
    if image and (scheme is not None or destination.startswith("//")):
        raise _error("unsafe-url", f"Image destinations must be local: {destination}", "DOC-006")
    if scheme is not None:
        if scheme.group(1).lower() not in ("http", "https"):
            raise _error("unsafe-url", f"Forbidden Markdown URL scheme: {destination}", "DOC-006")
        _safe_url(destination)
        return
    if destination.startswith("//"):
        raise _error("unsafe-url", f"Protocol-relative Markdown URL is forbidden: {destination}", "DOC-006")
    without_query = destination.split("?", 1)[0]
    path, _, _ = without_query.partition("#")
    if image and not path:
        raise _error("unsafe-path", "Image destination must name a local file", "DOC-001")
    if path.startswith("/") or "\\" in path or "%" in path:
        raise _error("unsafe-path", f"Unsafe local Markdown destination: {destination}", "DOC-001")
    if path and any(part in ("", ".", "..") for part in path.split("/")):
        raise _error("unsafe-path", f"Non-normalized or traversing Markdown destination: {destination}", "DOC-001")


def _bracket_end(text: str, start: int) -> int | None:
    depth = 0
    code = False
    index = start
    while index < len(text):
        char = text[index]
        if char == "\\":
            index += 2
            continue
        if char == "`":
            code = not code
        elif not code and char == "[":
            depth += 1
        elif not code and char == "]":
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def _destination_end(text: str, start: int) -> int | None:
    depth = 0
    angle = False
    index = start
    while index < len(text):
        char = text[index]
        if char == "\\":
            index += 2
            continue
        if char == "<":
            angle = True
        elif char == ">" and angle:
            angle = False
        elif not angle and char == "(":
            depth += 1
        elif not angle and char == ")":
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def _reference_label(value: str) -> str:
    return " ".join(value.split()).casefold()


def _markdown_destinations(text: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    definitions: dict[str, str] = {}
    offset = 0
    lines = text.splitlines(keepends=True)
    for line_number, line in enumerate(lines, 1):
        stripped = line.lstrip(" ")
        if len(line) - len(stripped) <= 3 and stripped.startswith("["):
            close = _bracket_end(stripped, 0)
            if close is not None and close + 1 < len(stripped) and stripped[close + 1] == ":":
                label = _reference_label(stripped[1:close])
                remainder = stripped[close + 2:].strip()
                end = offset + len(line)
                if not remainder and line_number < len(lines):
                    remainder = lines[line_number].strip()
                    end += len(lines[line_number])
                angle = remainder.startswith("<") and remainder.endswith(">")
                if angle:
                    remainder = remainder[1:-1]
                elif remainder:
                    remainder = remainder.split()[0]
                definitions[label] = remainder
                records.append({"destination": remainder, "image": False, "angle": angle,
                                "line": line_number, "start": offset, "end": end})
        offset += len(line)
    index = 0
    while index < len(text):
        image = text.startswith("![", index)
        if not image and text[index] != "[":
            index += 1
            continue
        open_index = index + 1 if image else index
        close = _bracket_end(text, open_index)
        if close is None:
            index += 1
            continue
        label = text[open_index + 1:close]
        after = close + 1
        line = text.count("\n", 0, index) + 1
        if after < len(text) and text[after] == "(":
            end = _destination_end(text, after)
            if end is None:
                records.append({"error": "Unclosed inline destination", "line": line,
                                "start": index, "end": after + 1})
                index = after + 1
                continue
            destination = text[after + 1:end].strip()
            angle = destination.startswith("<") and destination.endswith(">")
            if angle:
                destination = destination[1:-1]
            records.append({"destination": destination, "image": image, "angle": angle,
                            "line": line, "start": index, "end": end + 1})
            index = end + 1
        elif after < len(text) and text[after] == "[":
            reference_end = _bracket_end(text, after)
            if reference_end is None:
                records.append({"error": "Unclosed reference label", "line": line,
                                "start": index, "end": after + 1})
                index = after + 1
                continue
            reference = text[after + 1:reference_end] or label
            key = _reference_label(reference)
            if key in definitions:
                records.append({"destination": definitions[key], "image": image, "angle": False,
                                "line": line, "start": index, "end": reference_end + 1})
            index = reference_end + 1
        else:
            key = _reference_label(label)
            if key in definitions:
                records.append({"destination": definitions[key], "image": image, "angle": False,
                                "line": line, "start": index, "end": after})
            index = after
    return records


def _mask_code_spans(text: str) -> str:
    masked = re.sub(r"(?ms)^\s*(`{3,}|~{3,}).*?^\s*\1", lambda match: "".join(
        char if char == "\n" else " " for char in match.group(0)), text)
    return re.sub(r"`+[^`]*`+", lambda match: "".join(
        char if char == "\n" else " " for char in match.group(0)), masked)


def _markdown_security_findings(text: str, relative: str,
                                expected_marker: str | None = None) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []

    def add(rule: str, code: str, detail: str, line: int | None = None) -> None:
        findings.append({"rule": rule, "code": code, "file": relative, "line": line, "detail": detail})

    for match in re.finditer(r"<!--(.*?)-->", text, re.S):
        comment = match.group(0)
        content_comment = re.fullmatch(
            r"<!-- document-kit:content:[a-z]+(?:-[a-z]+)*:(?:start|end) -->", comment)
        managed_comment = re.fullmatch(r"<!-- document-kit: profile=[^<>\n]+ -->", comment)
        allowed = comment == expected_marker if expected_marker is not None else managed_comment is not None
        if not allowed and content_comment is None:
            add("DOC-008", "unsafe-markdown", "Disallowed HTML comment", text.count("\n", 0, match.start()) + 1)
    records = _markdown_destinations(text)
    scrubbed = _mask_code_spans(text)
    for record in records:
        if "destination" in record:
            start, end = record["start"], record["end"]
            scrubbed = scrubbed[:start] + "".join(char if char == "\n" else " " for char in scrubbed[start:end]) + scrubbed[end:]
    scrubbed = re.sub(r"<!--.*?-->", "", scrubbed, flags=re.S)
    scrubbed = re.sub(r'<a\s+id="(?:[A-Za-z][0-9]+|\{\{section_id:[a-z]+(?:-[a-z]+)*\}\})"\s*></a>', "", scrubbed)
    if re.search(r"<\s*/?\s*[A-Za-z][^>]*>", scrubbed) \
            or re.search(r"\bon\w+\s*=", scrubbed, re.I) \
            or re.search(r"\{%-?", scrubbed) \
            or re.search(r"\{:\s*", scrubbed):
        add("DOC-008", "unsafe-markdown", "Disallowed raw HTML, executable include, or attribute block")
    for record in records:
        if "error" in record:
            add("DOC-008", "unsafe-markdown", record["error"], record["line"])
            continue
        try:
            _safe_markdown_destination(record["destination"], record["image"], record["angle"])
        except DocumentKitError as error:
            add("DOC-001" if error.code == "unsafe-path" else "DOC-006", error.code, error.message, record["line"])
    return findings


def _markdown_security(text: str, relative: str) -> None:
    findings = _markdown_security_findings(text, relative)
    if findings:
        first = findings[0]
        raise DocumentKitError(first["code"], first["detail"], findings)


BUNDLED_INVENTORY = {
    "generic-requirements": "requirements", "generic-architecture": "architecture",
    "generic-design": "design", "generic-interface": "interface",
    "generic-verification": "verification", "system-requirements": "requirements",
    "change-requirements": "requirements", "system-context": "architecture",
    "component-composition": "architecture", "interaction-flow": "design",
    "state-machine": "design", "data-model": "design", "operation-contract": "interface",
    "data-contract": "interface", "verification-test-plan": "verification",
    "design-review": "verification",
}
ZERO_SENTINEL = "0" * 64


def _validate_entry_template(profile: Mapping[str, Any], entry: Mapping[str, Any], text: str) -> None:
    from collections import Counter

    sections = entry["required_sections"]
    keys = [section["key"] for section in sections]
    fields = [field for section in sections for field in section["required_fields"]]
    if re.findall(r"^## (.+)$", text, re.M) != [section["heading"] for section in sections]:
        raise _error("invalid-catalog", f"Template headings differ for {entry['template_id']}", "DOC-002")
    if re.findall(r"\{\{section_id:([^}]+)\}\}", text) != keys \
            or re.findall(r"\{\{content:([^}]+)\}\}", text) != keys \
            or re.findall(r"\{\{field:([^}]+)\}\}", text) != fields:
        raise _error("invalid-catalog", f"Template section tokens differ for {entry['template_id']}", "DOC-002")
    expected = ["profile_id", "version", "document_class", "template_id", "contract_sha256",
                "contract_digest_version", "title", "status", "revision", "model_sources"]
    expected += [f"section_id:{key}" for key in keys]
    expected += [f"field:{field}" for field in fields]
    expected += [f"content:{key}" for key in keys]
    if Counter(re.findall(r"\{\{([^{}]+)\}\}", text)) != Counter(expected):
        raise _error("invalid-catalog", f"Template tokens differ for {entry['template_id']}", "DOC-002")
    marker_template = re.sub(r"\{([^{}]+)\}", lambda match: "{{" + match.group(1) + "}}",
                             profile["validation"]["managed_marker"])
    if not text.startswith(marker_template + "\n"):
        raise _error("invalid-catalog", f"Template marker differs for {entry['template_id']}", "DOC-002")


def _reject_fixture_hash_keys(value: Any) -> None:
    if isinstance(value, dict):
        for key, item in value.items():
            if key in {"contract_sha256", "expected_contract_sha256"}:
                raise _error("invalid-catalog", "Bundled fixtures must not store a contract digest", "DOC-007")
            _reject_fixture_hash_keys(item)
    elif isinstance(value, list):
        for item in value:
            _reject_fixture_hash_keys(item)


def load_catalog_contract(root: Path, legacy_loader, digest_provider=None) -> dict[str, Any]:
    resources: dict[str, bytes] = {}
    roles: dict[str, str] = {}

    def add(value: Any, role: str, code: str = "invalid-catalog") -> str:
        relative = _safe_rel(value, "Catalog resource", code).as_posix()
        if relative in roles and roles[relative] != role:
            raise _error(code, f"Resource {relative} has conflicting roles", "DOC-002")
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
    if not isinstance(profile, dict) or profile.get("schema_version") != "1.1":
        raise _error("invalid-profile", "Unsupported profile schema")
    for key in ("profile_id", "version", "rules_file", "skill_file", "document_classes", "project",
                "anchor_policy", "figure_block", "validation", "catalog", "compatibility_packs"):
        if key not in profile:
            raise _error("invalid-profile", f"Profile is missing {key}")
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
    marker_tokens = {"profile_id", "version", "document_class", "template_id", "contract_sha256",
                     "contract_digest_version"}
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
    if not isinstance(figure, dict) or not isinstance(figure.get("ordered_fields"), list) \
            or not all(isinstance(item, str) and item for item in figure["ordered_fields"]) \
            or not isinstance(figure.get("field_tokens"), dict) \
            or not all(isinstance(value, str) and value for value in figure["field_tokens"].values()) \
            or set(figure["field_tokens"]) != set(figure["ordered_fields"]) \
            or len(set(figure["field_tokens"].values())) != len(figure["field_tokens"]) \
            or not isinstance(figure.get("statuses"), list) \
            or not all(isinstance(item, str) and item for item in figure["statuses"]):
        raise _error("invalid-profile", "Figure block policy is invalid")

    classes = profile["document_classes"]
    if not isinstance(classes, dict) or not classes:
        raise _error("invalid-profile", "document_classes must be a nonempty object")
    for class_name, spec in classes.items():
        if not isinstance(spec, dict) or re.fullmatch(r"[a-z]+(?:-[a-z]+)*", class_name) is None:
            raise _error("invalid-profile", f"Invalid class {class_name}")
        for key in ("path_template", "filename_pattern", "template", "required_sections"):
            if key not in spec:
                raise _error("invalid-profile", f"{class_name} is missing {key}")
        route = spec["path_template"]
        if not isinstance(route, str) or route.count("{name}") != 1 or not route.endswith(".md"):
            raise _error("invalid-profile", f"Invalid class route {route}")
        _safe_rel(route.replace("{name}", "sample"), "Class route", "invalid-profile")
        if not route.startswith(docs_root + "/"):
            raise _error("invalid-profile", f"Class route escapes documentation root: {route}")
        try:
            re.compile(spec["filename_pattern"])
            _validate_sections(spec["required_sections"])
        except (TypeError, re.error, DocumentKitError) as error:
            raise _error("invalid-profile", f"Invalid class declaration {class_name}: {error}") from error
        if not isinstance(spec.get("default_template_id"), str) or not spec["default_template_id"]:
            raise _error("template-default", f"Class {class_name} has no generic default", "DOC-002")

    manifest_info = profile["catalog"]
    if not isinstance(manifest_info, dict) or set(manifest_info) != {"manifest", "sources", "routing_guide", "mixed_fixture"}:
        raise _error("invalid-profile", "Catalog resource declarations are invalid")
    manifest = json_resource(manifest_info["manifest"], "manifest")
    sources_doc = json_resource(manifest_info["sources"], "sources")
    routing_guide = add(manifest_info["routing_guide"], "routing-guide")
    mixed_path = add(manifest_info["mixed_fixture"], "mixed-fixture")
    try:
        mixed_fixture = json.loads(resources[mixed_path].decode("utf-8"), object_pairs_hook=_unique_pairs)
    except ValueError as error:
        raise _error("invalid-catalog", f"Invalid mixed fixture: {error}", "DOC-007") from error
    if not isinstance(manifest, dict) or set(manifest) != {"schema_version", "profile_id", "version", "entries"} \
            or manifest.get("schema_version") != "1.1" \
            or manifest.get("profile_id") != profile["profile_id"] or manifest.get("version") != profile["version"] \
            or not isinstance(manifest.get("entries"), list):
        raise _error("invalid-catalog", "Catalog manifest identity or shape is invalid", "DOC-002")
    if not isinstance(sources_doc, dict) or set(sources_doc) != {"sources"} \
            or not isinstance(sources_doc.get("sources"), list):
        raise _error("invalid-source", "Source register shape is invalid", "DOC-006")
    source_records = sources_doc["sources"]
    source_ids: set[str] = set()
    source_locals: list[str] = []
    for record in source_records:
        _validate_source(record)
        if record["source_id"] in source_ids:
            raise _error("invalid-source", f"Duplicate source ID: {record['source_id']}", "DOC-006")
        source_ids.add(record["source_id"])
        if record["local_path"] is not None:
            source_locals.append(record["local_path"])
    bundled = profile["profile_id"] == "xdesign-sysml-docs" and profile["version"] == "1.1.0"
    entries: dict[str, Any] = {}
    for entry in manifest["entries"]:
        _validate_entry(entry, classes, source_ids, bundled)
        if entry["template_id"] in entries:
            raise _error("invalid-catalog", f"Duplicate template ID: {entry['template_id']}", "DOC-002")
        entries[entry["template_id"]] = entry
    if bundled:
        actual_inventory = {entry_id: entry["document_class"] for entry_id, entry in entries.items()}
        if actual_inventory != BUNDLED_INVENTORY:
            raise _error("invalid-catalog", "Bundled 16-entry inventory is invalid", "DOC-002")
        cited = {usage["source_id"] for entry in manifest["entries"] for usage in entry["sources"]}
        if not {"DPR", "GOOD-DOCS", "GITHUB-DOCS", "SPECD", "C4", "OPENSPEC", "ARC42", "REQ42"}.issubset(cited):
            raise _error("invalid-catalog", "Bundled eight-source coverage is incomplete", "DOC-002")
        for class_name in classes:
            if sum(entry["document_class"] == class_name and not entry_id.startswith("generic-")
                   for entry_id, entry in entries.items()) < 2:
                raise _error("invalid-catalog", f"Bundled class has fewer than two scenarios: {class_name}", "DOC-002")
    for class_name, spec in classes.items():
        default_id = spec["default_template_id"]
        if default_id not in entries:
            raise _error("template-default", f"Unknown default template {default_id}", "DOC-002")
        default = entries[default_id]
        if default["document_class"] != class_name or not default_id.startswith("generic-"):
            raise _error("template-default", f"Default template is not a class generic: {default_id}", "DOC-002")
        if spec["template"] != default["template"] or spec["required_sections"] != default["required_sections"]:
            raise _error("template-default", f"Class generic declaration differs from {default_id}", "DOC-002")

    def validate_inputs(value: Any, label: str, figures: bool) -> None:
        if not isinstance(value, list):
            raise _error("invalid-catalog", f"{label} must be an array", "DOC-007")
        required = {"pack", "target", "title", *figure["ordered_fields"]} if figures else {"pack", "target"}
        for item in value:
            if not isinstance(item, dict) or set(item) != required:
                raise _error("invalid-catalog", f"{label} input shape is invalid", "DOC-007")
            add(item["pack"], "figure" if figures else "model")
            _safe_rel(item["target"], f"{label} target")
            if figures:
                for key in required - {"pack", "target"}:
                    _text(item[key], f"{label}.{key}", "invalid-catalog")

    def validate_caller(document: Any, entry: Mapping[str, Any], models: list[dict[str, Any]]) -> None:
        if not isinstance(document, dict):
            raise _error("invalid-catalog", "Fixture caller input must be an object", "DOC-007")
        entry_id = entry["template_id"]
        if not isinstance(document["name"], str) \
                or re.fullmatch(classes[entry["document_class"]]["filename_pattern"], document["name"]) is None:
            raise _error("invalid-catalog", f"Fixture name is invalid for {entry_id}", "DOC-007")
        metadata = document["metadata"]
        if not isinstance(metadata, dict) or set(metadata) != {"title", "status", "revision", "model_sources"}:
            raise _error("invalid-catalog", f"Fixture metadata is invalid for {entry_id}", "DOC-007")
        for key in ("title", "status", "revision"):
            _text(metadata[key], f"{entry_id}.{key}", "invalid-catalog")
        if not isinstance(metadata["model_sources"], list) or not metadata["model_sources"]:
            raise _error("invalid-catalog", f"Fixture model_sources is invalid for {entry_id}", "DOC-007")
        if not isinstance(models, list) or any(not isinstance(item, dict) or not isinstance(item.get("target"), str)
                                               for item in models):
            raise _error("invalid-catalog", f"Fixture models are invalid for {entry_id}", "DOC-007")
        model_targets = {item["target"] for item in models}
        if any(not isinstance(item, str) or item not in model_targets for item in metadata["model_sources"]):
            raise _error("invalid-catalog", f"Fixture model source is not materialized for {entry_id}", "DOC-007")
        sections = document["sections"]
        required_sections = entry["required_sections"]
        if not isinstance(sections, dict) or set(sections) != {item["key"] for item in required_sections}:
            raise _error("invalid-catalog", f"Fixture sections differ for {entry_id}", "DOC-007")
        for section in required_sections:
            value = sections[section["key"]]
            if not isinstance(value, dict) or set(value) != {"anchor", "content", "fields"}:
                raise _error("invalid-catalog", f"Fixture section shape differs for {entry_id}", "DOC-007")
            if not isinstance(value["anchor"], str) \
                    or re.fullmatch(profile["anchor_policy"]["format"], value["anchor"]) is None:
                raise _error("invalid-catalog", f"Fixture anchor is invalid for {entry_id}", "DOC-007")
            _text(value["content"], f"{entry_id}.{section['key']}.content", "invalid-catalog")
            fields = value["fields"]
            if not isinstance(fields, dict) or set(fields) != set(section["required_fields"]):
                raise _error("invalid-catalog", f"Fixture fields differ for {entry_id}.{section['key']}", "DOC-007")
            for field, field_value in fields.items():
                _text(field_value, f"{entry_id}.{field}", "invalid-catalog")
                if field == "diagram_decisions":
                    try:
                        decisions = json.loads(field_value)
                    except ValueError as error:
                        raise _error("invalid-catalog", f"Fixture diagram decisions are malformed for {entry_id}", "DOC-007") from error
                    if not isinstance(decisions, list):
                        raise _error("invalid-catalog", f"Fixture diagram decisions must be an array for {entry_id}", "DOC-007")

    isolated: dict[str, Any] = {}
    for entry in manifest["entries"]:
        entry_id = entry["template_id"]
        template_path = add(entry["template"], "template")
        guide_path = add(entry["guide"], "guide")
        example_path = add(entry["example"], "example")
        fixture_path = add(entry["fixture"], "fixture")
        for path in (template_path, guide_path, example_path):
            _markdown_security(resources[path].decode("utf-8"), path)
        _validate_entry_template(profile, entry, resources[template_path].decode("utf-8"))
        try:
            fixture = json.loads(resources[fixture_path].decode("utf-8"), object_pairs_hook=_unique_pairs)
        except ValueError as error:
            raise _error("invalid-catalog", f"Invalid fixture for {entry_id}: {error}", "DOC-007") from error
        expected_keys = {"profile_id", "version", "contract_digest_version", "document_class", "name",
                         "template_id", "metadata", "sections", "models", "figures", "expected_document"}
        if not isinstance(fixture, dict) or set(fixture) != expected_keys:
            raise _error("invalid-catalog", f"Fixture fields differ for {entry_id}", "DOC-007")
        if fixture["profile_id"] != profile["profile_id"] or fixture["version"] != profile["version"] \
                or fixture["contract_digest_version"] != "2" or fixture["document_class"] != entry["document_class"] \
                or fixture["template_id"] != entry_id or fixture["expected_document"] != entry["example"]:
            raise _error("invalid-catalog", f"Fixture identity differs for {entry_id}", "DOC-007")
        _reject_fixture_hash_keys(fixture)
        validate_inputs(fixture["models"], f"{entry_id}.models", False)
        validate_inputs(fixture["figures"], f"{entry_id}.figures", True)
        validate_caller(fixture, entry, fixture["models"])
        isolated[entry_id] = fixture

    expected_mixed_keys = {"profile_id", "version", "contract_digest_version", "documents", "coverage", "models", "figures"}
    if not isinstance(mixed_fixture, dict) or set(mixed_fixture) != expected_mixed_keys \
            or mixed_fixture["profile_id"] != profile["profile_id"] or mixed_fixture["version"] != profile["version"] \
            or mixed_fixture["contract_digest_version"] != "2" or not isinstance(mixed_fixture["documents"], list):
        raise _error("invalid-catalog", "Mixed fixture identity or shape is invalid", "DOC-007")
    _reject_fixture_hash_keys(mixed_fixture)
    coverage = mixed_fixture["coverage"]
    if not isinstance(coverage, dict) or set(coverage) != {"pack", "target"}:
        raise _error("invalid-catalog", "Mixed coverage shape is invalid", "DOC-007")
    coverage_path = add(coverage["pack"], "coverage")
    _safe_rel(coverage["target"], "Mixed coverage target")
    _markdown_security(resources[coverage_path].decode("utf-8"), coverage_path)
    _markdown_security(resources[routing_guide].decode("utf-8"), routing_guide)
    mixed_anchors: set[str] = set()
    for document in mixed_fixture["documents"]:
        if not isinstance(document, dict) or set(document) != {"document_class", "name", "template_id", "metadata", "sections", "expected_document"}:
            raise _error("invalid-catalog", "Mixed document input shape is invalid", "DOC-007")
        if not isinstance(document["template_id"], str) or not isinstance(document["document_class"], str):
            raise _error("invalid-catalog", "Mixed document class and template ID must be text", "DOC-007")
        entry = entries.get(document["template_id"])
        if entry is None or entry["document_class"] != document["document_class"] \
                or document["expected_document"] != entry["example"]:
            raise _error("invalid-catalog", "Mixed document identity or example reference is invalid", "DOC-007")
        validate_caller(document, entry, mixed_fixture["models"])
        for section in document["sections"].values():
            anchor = section["anchor"].casefold()
            if anchor in mixed_anchors:
                raise _error("invalid-catalog", f"Mixed fixture repeats anchor {section['anchor']}", "DOC-007")
            mixed_anchors.add(anchor)
    if bundled:
        mixed_classes = [document["document_class"] for document in mixed_fixture["documents"]]
        if len(mixed_fixture["documents"]) != 6 or set(mixed_classes) != set(classes) \
                or not any(mixed_classes.count(class_name) >= 2 for class_name in classes):
            raise _error("invalid-catalog", "Bundled mixed fixture coverage is incomplete", "DOC-007")
    validate_inputs(mixed_fixture["models"], "mixed.models", False)
    validate_inputs(mixed_fixture["figures"], "mixed.figures", True)

    add(profile["rules_file"], "rules", "invalid-profile")
    add(profile["skill_file"], "skill", "invalid-profile")
    add("README.md", "readme", "invalid-profile")
    figure_template = add(figure["template"], "figure-template", "invalid-profile")
    for spec in classes.values():
        add(spec["template"], "template", "invalid-profile")
    for local_path in source_locals:
        add(local_path, "source-local")

    compatibility_contracts = []
    descriptor_pairs = []
    if not isinstance(profile["compatibility_packs"], list):
        raise _error("invalid-profile", "compatibility_packs must be an array")
    for descriptor in profile["compatibility_packs"]:
        if not isinstance(descriptor, dict) or set(descriptor) != {"profile_id", "version", "root", "contract_sha256", "contract_digest_version"}:
            raise _error("invalid-profile", "Compatibility descriptor shape is invalid")
        compatibility_root = _safe_rel(descriptor["root"], "Compatibility root", "invalid-profile").as_posix()
        compatibility = legacy_loader(root / compatibility_root)
        compatibility_contracts.append(compatibility)
        descriptor_pairs.append((descriptor, compatibility, compatibility_root))
        for relative, value in compatibility["resources"].items():
            data = value[1] if isinstance(value, tuple) else value
            path = add(f"{compatibility_root}/{relative}", "compatibility")
            if resources[path] != data:
                raise _error("invalid-catalog", f"Compatibility bytes differ after loading: {path}", "DOC-002")

    for relative, data in resources.items():
        if relative.endswith(".md"):
            _markdown_security(data.decode("utf-8"), relative)

    provider = digest_provider or digest_v2
    contract_sha256 = provider(resources)
    if not isinstance(contract_sha256, str) or re.fullmatch(r"[0-9a-f]{64}", contract_sha256) is None \
            or contract_sha256 == ZERO_SENTINEL:
        raise _error("invalid-catalog", "Recomputed contract digest is zero or malformed", "DOC-002")
    example_paths = {entry["example"] for entry in manifest["entries"]}
    fixture_paths = {entry["fixture"] for entry in manifest["entries"]} | {mixed_path}
    for path in fixture_paths:
        if contract_sha256.encode("ascii") in resources[path]:
            raise _error("invalid-catalog", f"Bundled fixture stores the actual contract digest: {path}", "DOC-007", path)
    for entry in manifest["entries"]:
        path = entry["example"]
        data = resources[path]
        expected_marker = marker({"profile_id": profile["profile_id"], "version": profile["version"],
                                  "document_class": entry["document_class"], "template_id": entry["template_id"],
                                  "contract_sha256": ZERO_SENTINEL, "contract_digest_version": "2"})
        if not data.startswith((expected_marker + "\n").encode("utf-8")) \
                or data.count(ZERO_SENTINEL.encode("ascii")) != 1 \
                or contract_sha256.encode("ascii") in data:
            raise _error("invalid-catalog", f"Stored example marker or sentinel is invalid: {path}", "DOC-002", path, 1)
    for path, data in resources.items():
        scan_data = data
        if path == "profile.json":
            for descriptor in profile["compatibility_packs"]:
                if descriptor["contract_sha256"] == ZERO_SENTINEL:
                    scan_data, replacements = re.subn(
                        rb'("contract_sha256"\s*:\s*")0{64}(")',
                        rb'\g<1>' + b'f' * 64 + rb'\g<2>', scan_data, count=1)
                    if replacements != 1:
                        raise _error("invalid-catalog", "Zero compatibility descriptor is not uniquely represented", "DOC-002")
        if path not in example_paths and ZERO_SENTINEL.encode("ascii") in scan_data:
            raise _error("invalid-catalog", f"Stored-example sentinel occurs outside its contract field: {path}", "DOC-002", path)
    for descriptor, compatibility, compatibility_root in descriptor_pairs:
        if compatibility["profile"]["profile_id"] != descriptor["profile_id"] \
                or compatibility["profile"]["version"] != descriptor["version"] \
                or compatibility["contract_sha256"] != descriptor["contract_sha256"] \
                or compatibility["contract_digest_version"] != descriptor["contract_digest_version"]:
            raise _error("contract-drift", f"Compatibility contract differs from descriptor: {compatibility_root}")

    templates = {class_name: resources[spec["template"]].decode("utf-8")
                 for class_name, spec in classes.items()}
    templates["figure-block"] = resources[figure_template].decode("utf-8")
    rules = resources[profile["rules_file"]].decode("utf-8")
    catalog = {"schema_version": "1.1", "profile_id": profile["profile_id"],
               "version": profile["version"], "entries": manifest["entries"],
               "sources": source_records}
    return {"root": root, "profile": profile, "rules": rules, "templates": templates,
            "resources": resources, "contract_sha256": contract_sha256,
            "contract_digest_version": "2", "catalog": catalog,
            "entries_by_id": entries, "fixtures_by_id": isolated, "mixed_fixture": mixed_fixture,
            "compatibility_contracts": compatibility_contracts}


def synthesize_v1_catalog(contract: Mapping[str, Any]) -> dict[str, Any]:
    profile = contract["profile"]
    entries = []
    for class_name in profile["document_classes"]:
        entries.append({"template_id": None, "document_class": class_name, "title": None,
                        "scenario": None, "when_to_use": None, "when_not_to_use": None,
                        "required_inputs": None, "dependencies": None, "required_sections": None,
                        "template": None, "guide": None, "example": None, "fixture": None,
                        "sources": None, "checks": None, "view_concerns": None})
    source = {"source_id": profile["profile_id"], "title": profile["profile_id"],
              "publisher": profile["profile_id"], "retrieved_at": None,
              "ideas_used": ["Actual selected schema 1.0 pack"], "limitations": [],
              "url": None, "local_path": ".", "additional_urls": [],
              "license_name": "unknown", "license_version": "unknown", "license_url": None}
    return {"schema_version": "1.0", "profile_id": profile["profile_id"],
            "version": profile["version"], "entries": entries, "sources": [source]}


def select_template(contract: Mapping[str, Any], document_class: str,
                    template_id: str | None) -> dict[str, Any]:
    profile = contract["profile"]
    classes = profile["document_classes"]
    if document_class not in classes:
        raise _error("invalid-request", f"Unknown document class: {document_class}")
    if profile.get("schema_version") == "1.0":
        if template_id is not None:
            raise _error("unknown-template", "Schema 1.0 has no selectable template IDs")
        class_spec = classes[document_class]
        spec = {**class_spec, "document_class": document_class, "view_concerns": [],
                "_template_text": contract["templates"][document_class], "_entry": None}
        return {"spec": spec, "effective_identity": identity(contract, document_class, None)}
    if template_id is None:
        template_id = classes[document_class]["default_template_id"]
    if not isinstance(template_id, str) or re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", template_id) is None:
        raise _error("unknown-template", f"Malformed template ID: {template_id}")
    entries = contract["entries_by_id"]
    if template_id not in entries:
        raise _error("unknown-template", f"Unknown template ID: {template_id}")
    entry = entries[template_id]
    if entry["document_class"] != document_class:
        raise _error("template-class-mismatch", f"Template {template_id} belongs to {entry['document_class']}")
    class_spec = classes[document_class]
    spec = {**class_spec, **entry, "document_class": document_class,
            "_template_text": contract["resources"][entry["template"]].decode("utf-8"), "_entry": entry}
    return {"spec": spec, "effective_identity": identity(contract, document_class, template_id)}


def _section_blocks(text: str, sections: list[dict[str, Any]]) -> dict[str, str]:
    blocks: dict[str, str] = {}
    for index, section in enumerate(sections):
        heading = "## " + section["heading"]
        start = text.find(heading)
        if start < 0:
            continue
        next_start = text.find("\n## ", start + len(heading))
        blocks[section["key"]] = text[start:next_start + 1 if next_start >= 0 else len(text)]
    return blocks


def _parse_decisions(block: str, section_key: str) -> list[dict[str, Any]]:
    decisions = []
    for line in block.splitlines():
        if not line.startswith(("**Diagram decision:**", "**Diagram decisions:**")):
            continue
        try:
            value = json.loads(line.split(":**", 1)[1].strip())
        except ValueError as error:
            raise _error("invalid-diagram-decision", f"Malformed diagram decision in {section_key}: {error}", "DOC-005")
        if not isinstance(value, list):
            raise _error("invalid-diagram-decision", f"Diagram decision field must be an array in {section_key}", "DOC-005")
        for decision in value:
            if not isinstance(decision, dict) or set(decision) != DECISION_FIELDS:
                raise _error("invalid-diagram-decision", f"Diagram decision fields differ in {section_key}", "DOC-005")
            decisions.append(decision)
    return decisions


def diagram_decision_findings(text: str, spec: Mapping[str, Any]) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    blocks = _section_blocks(text, spec["required_sections"])
    parsed: dict[str, Any] = {}
    for section_key, block in blocks.items():
        try:
            parsed[section_key] = _parse_decisions(block, section_key)
        except DocumentKitError as error:
            parsed[section_key] = error

    def add(code: str, detail: str) -> None:
        findings.append({"rule": "DOC-005", "code": code, "file": "", "line": None, "detail": detail})

    reported_errors: set[str] = set()
    for concern in spec["view_concerns"]:
        section_key = concern["section_key"]
        block = blocks.get(section_key)
        if block is None:
            add("missing-diagram-decision", f"Section {section_key} has no selected decision for {concern['concern']}")
            continue
        decisions = parsed[section_key]
        if isinstance(decisions, DocumentKitError):
            if section_key not in reported_errors:
                findings.extend(decisions.diagnostics)
                reported_errors.add(section_key)
            continue
        matching = [decision for decision in decisions if decision["concern"] == concern["concern"]]
        if not matching:
            add("missing-diagram-decision", f"Section {section_key} has no selected decision for {concern['concern']}")
            continue
        if len(matching) != 1:
            add("invalid-diagram-decision", f"Section {section_key} has duplicate decisions for {concern['concern']}")
            continue
        decision = matching[0]
        if not isinstance(decision["applies"], bool):
            add("invalid-diagram-decision", f"Decision for {concern['concern']} has non-Boolean applies")
            continue
        expected = concern["applicability"] == "expected"
        not_applicable = concern["applicability"] == "not-applicable"
        if (expected and not decision["applies"]) or (not_applicable and decision["applies"]):
            add("invalid-diagram-decision", f"Decision applicability differs for {concern['concern']}")
        if decision["choice"] not in concern["alternatives"]:
            add("invalid-diagram-decision", f"Decision choice is not allowed for {concern['concern']}")
        if not isinstance(decision["rationale"], str) or not decision["rationale"].strip():
            add("invalid-diagram-decision", f"Decision rationale is empty for {concern['concern']}")
        if decision["choice"] == "figure" and not decision["applies"]:
            add("invalid-diagram-decision", f"Figure decision is not applicable for {concern['concern']}")
        if decision["choice"] == "not-applicable" and decision["applies"]:
            add("invalid-diagram-decision", f"Not-applicable decision is marked applicable for {concern['concern']}")
        if decision["choice"] == "figure":
            asset = decision["asset"]
            model_view = decision["model_view"]
            if not isinstance(asset, str) or re.fullmatch(r"assets/[a-z0-9]+(?:-[a-z0-9]+)*\.svg", asset) is None:
                add("invalid-diagram-decision", f"Figure decision asset is invalid for {concern['concern']}")
            elif re.search(r"\]\(\s*" + re.escape(asset) + r"\s*\)", block) is None:
                add("invalid-diagram-decision", f"Figure decision asset is not embedded for {concern['concern']}")
            if not isinstance(model_view, str) or not model_view.strip() or f"**Model view:** {model_view}" not in block:
                add("invalid-diagram-decision", f"Figure decision model view differs for {concern['concern']}")
        elif decision["asset"] not in (None, ""):
            add("invalid-diagram-decision", f"Non-figure decision carries an asset for {concern['concern']}")
    for section_key, decisions in parsed.items():
        if isinstance(decisions, DocumentKitError):
            if section_key not in reported_errors:
                findings.extend(decisions.diagnostics)
                reported_errors.add(section_key)
            continue
        declared = {concern["concern"] for concern in spec["view_concerns"] if concern["section_key"] == section_key}
        for decision in decisions:
            if decision["concern"] not in declared:
                add("invalid-diagram-decision", f"Unknown concern in section {section_key}: {decision['concern']}")
    return findings


def attachment_decision(text: str, spec: Mapping[str, Any], section_key: str,
                        concern: str | None) -> dict[str, Any] | None:
    section_concerns = [item for item in spec["view_concerns"] if item["section_key"] == section_key]
    if not section_concerns:
        if concern is not None:
            raise _error("invalid-diagram-decision", f"Section {section_key} has no declared concerns")
        return None
    if concern is None:
        raise _error("missing-diagram-decision", f"Section {section_key} has no selected decision")
    block = _section_blocks(text, spec["required_sections"]).get(section_key)
    if block is None:
        raise _error("missing-diagram-decision", f"Section {section_key} has no selected decision")
    decisions = [decision for decision in _parse_decisions(block, section_key)
                 if decision["concern"] == concern]
    if len(decisions) != 1:
        raise _error("invalid-diagram-decision", f"Unknown or mismatched concern: {concern}")
    decision = decisions[0]
    if decision["choice"] != "figure":
        raise _error("invalid-diagram-decision", f"Concern {concern} is a non-figure decision")
    return decision
