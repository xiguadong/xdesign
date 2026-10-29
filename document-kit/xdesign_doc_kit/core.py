"""Contract-driven SysML document authoring and structural validation.

Engineering prose and judgments always come from the caller. This module only
renders supplied content and checks deterministic predicates from the pinned
profile, rules, and templates.
"""

from __future__ import annotations

import fcntl
import hashlib
import json
import os
import re
import stat
import uuid
from collections import Counter
from contextlib import contextmanager
from pathlib import Path, PurePosixPath
from typing import Any, Iterator, Mapping


KIT_ROOT = Path(__file__).resolve().parent.parent
REGISTRY_NAME = ".document-kit-managed.json"
CONTENT_START = "<!-- document-kit:content:{key}:start -->"
CONTENT_END = "<!-- document-kit:content:{key}:end -->"
TOKEN = re.compile(r"\{\{([^{}]+)\}\}")
ANCHOR = re.compile(r'^\s*<a\s+id="([A-Za-z][0-9]+)"\s*></a>\s*$', re.I)
IMAGE = re.compile(r"^!\[([^\]]*)\]\(([^)]+\.svg)\)\s*$")
FIGURE_TITLE = re.compile(r"^>\s*\*\*Figure\s+([A-Z][0-9]+)-([0-9]+)\s+—\s+(.+?)\.\*\*\s*$")
FIELD_LINE = re.compile(r"^>\s*\*\*([^*]+):\*\*\s*(.*?)\s*$")
INLINE_LINK = re.compile(r"(?<!!)\[[^\]]+\]\((?:<([^>]+)>|([^\s)]+))\)")
ANNOTATION_START = re.compile(r"@(?:[A-Za-z_]\w*::)*(DocSection|DocFigure)\b")
FIELD_ASSIGNMENT = re.compile(r'\b([A-Za-z_]\w*)\s*=\s*"([^"]*)"\s*;?')


def diagnostic(rule: str, code: str, file: str, detail: str, line: int | None = None) -> dict[str, Any]:
    """Return the exact I20 diagnostic shape."""
    return {"rule": rule, "code": code, "file": file, "line": line, "detail": detail}


class DocumentKitError(ValueError):
    """A refused operation with a stable code and structured diagnostics."""

    def __init__(self, code: str, message: str, diagnostics: list[dict[str, Any]] | None = None):
        super().__init__(message)
        self.code = code
        self.message = message
        self.diagnostics = diagnostics if diagnostics is not None else [
            diagnostic("input", code, "", message)
        ]

    def as_dict(self) -> dict[str, Any]:
        return {"ok": False, "error": {"code": self.code, "message": self.message},
                "diagnostics": self.diagnostics}


def _error(code: str, detail: str, rule: str = "input", file: str = "",
           line: int | None = None) -> DocumentKitError:
    return DocumentKitError(code, detail, [diagnostic(rule, code, file, detail, line)])


def _unique_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def _safe_rel(value: Any, label: str, code: str = "unsafe-path") -> PurePosixPath:
    if (not isinstance(value, str) or not value or "\\" in value or "\x00" in value
            or ":" in value or value.startswith("/") or "//" in value):
        raise _error(code, f"{label} must be a normalized POSIX relative path", "DOC-001")
    parts = value.split("/")
    if any(part in ("", ".", "..") for part in parts):
        raise _error(code, f"{label} contains an unsafe path component", "DOC-001")
    path = PurePosixPath(value)
    if path.is_absolute() or path.as_posix() != value:
        raise _error(code, f"{label} is not normalized", "DOC-001")
    return path


def _root(value: str | Path, label: str) -> Path:
    try:
        path = Path(value).expanduser().absolute()
    except (TypeError, ValueError) as error:
        raise _error("invalid-request", f"{label} is not a directory path") from error
    if path.is_symlink() or not path.is_dir():
        raise _error("unsafe-path", f"{label} must be an existing nonsymlink directory", "DOC-001")
    return path.resolve()


def _reject_symlinks(root: Path, relative: PurePosixPath) -> None:
    current = root
    for part in relative.parts:
        current = current / part
        try:
            mode = current.lstat().st_mode
        except FileNotFoundError:
            continue
        except OSError as error:
            raise _error("unreadable-input", f"Cannot inspect {relative}: {error}") from error
        if stat.S_ISLNK(mode):
            raise _error("unsafe-path", f"Symlink in selected path: {relative}", "DOC-001",
                         relative.as_posix())


@contextmanager
def _parent_fd(root: Path, relative: PurePosixPath, create: bool = False) -> Iterator[tuple[int, str]]:
    """Walk one project-relative parent without following symlinks."""
    fd = os.open(str(root), os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in relative.parts[:-1]:
            try:
                next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            except FileNotFoundError:
                if not create:
                    raise _error("missing-document", f"Missing parent of {relative}", "DOC-001",
                                 relative.as_posix())
                os.mkdir(part, dir_fd=fd)
                next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        yield fd, relative.name
    except OSError as error:
        raise _error("unsafe-path", f"Cannot safely access {relative}: {error}", "DOC-001",
                     relative.as_posix()) from error
    finally:
        os.close(fd)


def _read_project_file(root: Path, relative: str, missing_code: str = "missing-document") -> bytes:
    path = _safe_rel(relative, "Project file")
    with _parent_fd(root, path) as (parent, name):
        try:
            fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
        except FileNotFoundError as error:
            raise _error(missing_code, f"File does not exist: {relative}", "DOC-001", relative) from error
        except OSError as error:
            raise _error("unreadable-input", f"Cannot open {relative}: {error}", "input", relative) from error
        try:
            if not stat.S_ISREG(os.fstat(fd).st_mode):
                raise _error("unreadable-input", f"Not a regular file: {relative}", "input", relative)
            with os.fdopen(fd, "rb") as stream:
                fd = -1
                return stream.read()
        finally:
            if fd >= 0:
                os.close(fd)


def _read_pack_file(root: Path, relative: str) -> bytes:
    path = _safe_rel(relative, "Contract resource", "invalid-profile")
    try:
        with _parent_fd(root, path) as (parent, name):
            fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
            try:
                if not stat.S_ISREG(os.fstat(fd).st_mode):
                    raise OSError("resource is not a regular file")
                with os.fdopen(fd, "rb") as stream:
                    fd = -1
                    return stream.read()
            finally:
                if fd >= 0:
                    os.close(fd)
    except (OSError, DocumentKitError) as error:
        raise _error("invalid-profile", f"Cannot read contract resource {relative}: {error}") from error


def _text(value: Any, label: str, code: str = "missing-metadata") -> str:
    if not isinstance(value, str) or not value.strip():
        raise _error(code, f"{label} must be nonempty text", "DOC-002")
    return value.strip()


def _contract(profile_root: str | Path | None = None) -> dict[str, Any]:
    root = _root(profile_root if profile_root is not None else KIT_ROOT, "Profile root")
    raw_profile = _read_pack_file(root, "profile.json")
    try:
        profile = json.loads(raw_profile.decode("utf-8"), object_pairs_hook=_unique_pairs)
    except (UnicodeError, ValueError) as error:
        raise _error("invalid-profile", f"Invalid profile.json: {error}") from error
    if not isinstance(profile, dict) or profile.get("schema_version") != "1.0":
        raise _error("invalid-profile", "Unsupported profile schema")
    for key in ("profile_id", "version", "rules_file", "document_classes", "project",
                "anchor_policy", "figure_block", "validation"):
        if key not in profile:
            raise _error("invalid-profile", f"Profile is missing {key}")
    if not isinstance(profile["document_classes"], dict) or not profile["document_classes"]:
        raise _error("invalid-profile", "Profile has no document classes")
    if (not isinstance(profile["profile_id"], str) or not profile["profile_id"]
            or not isinstance(profile["version"], str) or not profile["version"]):
        raise _error("invalid-profile", "Profile identity must be nonempty text")
    project = profile["project"]
    if not isinstance(project, dict) or not isinstance(project.get("docs_root"), str):
        raise _error("invalid-profile", "Project documentation root is invalid")
    _safe_rel(project["docs_root"], "Documentation root", "invalid-profile")
    required_roots = _root_list(project, "model_roots")
    optional_roots = _root_list(project, "optional_model_roots")
    if not required_roots:
        raise _error("invalid-profile", "At least one required model root is needed")
    for path in [*required_roots, *optional_roots]:
        _safe_rel(path, "Model root", "invalid-profile")
    if len(set([*required_roots, *optional_roots])) != len(required_roots) + len(optional_roots):
        raise _error("invalid-profile", "Model roots must be unique")
    anchors = profile["anchor_policy"]
    if not isinstance(anchors, dict) or not isinstance(anchors.get("format"), str):
        raise _error("invalid-profile", "Anchor policy is invalid")
    try:
        re.compile(anchors["format"])
    except re.error as error:
        raise _error("invalid-profile", f"Anchor format is invalid: {error}") from error
    validation = profile["validation"]
    if (not isinstance(validation, dict) or not isinstance(validation.get("managed_marker"), str)
            or "{document_class}" not in validation["managed_marker"]
            or not isinstance(validation.get("placeholder_patterns"), list)):
        raise _error("invalid-profile", "Managed validation policy is invalid")
    if f"profile={profile['profile_id']}@{profile['version']}" not in validation["managed_marker"]:
        raise _error("invalid-profile", "Managed marker differs from profile identity")
    for pattern in validation["placeholder_patterns"]:
        if not isinstance(pattern, str):
            raise _error("invalid-profile", "Placeholder pattern must be text")
        try:
            re.compile(pattern)
        except re.error as error:
            raise _error("invalid-profile", f"Invalid placeholder pattern: {error}") from error
    figure = profile["figure_block"]
    if (not isinstance(figure, dict) or not isinstance(figure.get("ordered_fields"), list)
            or not all(isinstance(item, str) for item in figure["ordered_fields"])
            or not isinstance(figure.get("field_tokens"), dict)
            or set(figure["field_tokens"]) != set(figure["ordered_fields"])
            or not isinstance(figure.get("statuses"), list)
            or not all(isinstance(value, str) and value for value in figure["field_tokens"].values())
            or len(set(figure["field_tokens"].values())) != len(figure["field_tokens"])):
        raise _error("invalid-profile", "Figure block policy is invalid")
    resources: dict[str, tuple[str, bytes]] = {"profile.json": ("profile", raw_profile)}

    def add_resource(path: Any, role: str) -> str:
        rel = _safe_rel(path, "Contract resource", "invalid-profile").as_posix()
        if rel in resources and resources[rel][0] != role:
            raise _error("invalid-profile", f"Resource {rel} has conflicting roles")
        if rel not in resources:
            resources[rel] = (role, _read_pack_file(root, rel))
        return rel

    rules_path = add_resource(profile["rules_file"], "rules")
    templates: dict[str, str] = {}
    for class_name, spec in profile["document_classes"].items():
        if not isinstance(spec, dict) or not re.fullmatch(r"[a-z]+(?:-[a-z]+)*", class_name):
            raise _error("invalid-profile", f"Invalid document class {class_name}")
        for key in ("path_template", "filename_pattern", "template", "required_sections"):
            if key not in spec:
                raise _error("invalid-profile", f"{class_name} is missing {key}")
        route = spec["path_template"]
        if not isinstance(route, str) or route.count("{name}") != 1:
            raise _error("invalid-profile", f"{class_name} route must contain one {{name}}")
        _safe_rel(route.replace("{name}", "sample"), "Class route", "invalid-profile")
        if not route.startswith(project["docs_root"] + "/") or not route.endswith(".md"):
            raise _error("invalid-profile", f"{class_name} route escapes documentation root")
        try:
            re.compile(spec["filename_pattern"])
        except (TypeError, re.error) as error:
            raise _error("invalid-profile", f"Invalid filename pattern in {class_name}: {error}") from error
        if not isinstance(spec["required_sections"], list) or not spec["required_sections"]:
            raise _error("invalid-profile", f"{class_name} has no required sections")
        if any(not isinstance(section, dict) for section in spec["required_sections"]):
            raise _error("invalid-profile", f"Invalid required section in {class_name}")
        keys = [section.get("key") for section in spec["required_sections"]]
        if (any(not isinstance(key, str) or not re.fullmatch(r"[a-z]+(?:-[a-z]+)*", key) for key in keys)
                or len(set(keys)) != len(keys)):
            raise _error("invalid-profile", f"Duplicate or invalid section keys in {class_name}")
        for section in spec["required_sections"]:
            if (not isinstance(section.get("heading"), str) or not section["heading"]
                    or not isinstance(section.get("required_fields"), list)
                    or not all(isinstance(field, str) and field for field in section["required_fields"])):
                raise _error("invalid-profile", f"Invalid heading or fields in {class_name}")
        template_path = add_resource(spec["template"], "document-template")
        try:
            templates[class_name] = resources[template_path][1].decode("utf-8")
        except UnicodeError as error:
            raise _error("invalid-profile", f"Template is not UTF-8: {template_path}") from error
        expected_headings = [section.get("heading") for section in spec["required_sections"]]
        actual_headings = re.findall(r"^## (.+)$", templates[class_name], re.M)
        if expected_headings != actual_headings:
            raise _error("invalid-profile", f"Template headings differ from {class_name} profile")
        if re.findall(r"\{\{section_id:([^}]+)\}\}", templates[class_name]) != keys:
            raise _error("invalid-profile", f"Template anchors differ from {class_name} profile")
        if re.findall(r"\{\{content:([^}]+)\}\}", templates[class_name]) != keys:
            raise _error("invalid-profile", f"Template content tokens differ from {class_name} profile")
        expected_fields = [field for section in spec["required_sections"]
                           for field in section.get("required_fields", [])]
        if len(set(expected_fields)) != len(expected_fields):
            raise _error("invalid-profile", f"Field tokens repeat across sections in {class_name}")
        if re.findall(r"\{\{field:([^}]+)\}\}", templates[class_name]) != expected_fields:
            raise _error("invalid-profile", f"Template fields differ from {class_name} profile")
        expected_tokens = ["title", "status", "revision", "model_sources"]
        expected_tokens.extend(f"section_id:{key}" for key in keys)
        expected_tokens.extend(f"field:{field}" for field in expected_fields)
        expected_tokens.extend(f"content:{key}" for key in keys)
        if Counter(TOKEN.findall(templates[class_name])) != Counter(expected_tokens):
            raise _error("invalid-profile", f"Template tokens differ from {class_name} profile")
        marker = validation["managed_marker"].replace("{document_class}", class_name)
        if not templates[class_name].startswith(marker + "\n"):
            raise _error("invalid-profile", f"Template marker differs from {class_name} profile")
    figure_path = add_resource(figure.get("template"), "figure-template")
    try:
        templates["figure-block"] = resources[figure_path][1].decode("utf-8")
    except UnicodeError as error:
        raise _error("invalid-profile", f"Figure template is not UTF-8: {figure_path}") from error
    if re.findall(r"^> \*\*([^*:]+):\*\*", templates["figure-block"], re.M) != figure.get("ordered_fields"):
        raise _error("invalid-profile", "Figure template fields differ from profile")
    expected_figure_tokens = Counter(["asset", "title", "title", "section_id", "ordinal",
                                      *figure["field_tokens"].values()])
    if Counter(TOKEN.findall(templates["figure-block"])) != expected_figure_tokens:
        raise _error("invalid-profile", "Figure template tokens differ from profile")
    try:
        rules = resources[rules_path][1].decode("utf-8")
    except UnicodeError as error:
        raise _error("invalid-profile", f"Rules are not UTF-8: {error}") from error
    hasher = hashlib.sha256()
    for path in sorted(resources, key=lambda value: value.encode("utf-8")):
        data = resources[path][1]
        hasher.update(path.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(len(data)).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(data)
    return {"root": root, "profile": profile, "rules": rules, "templates": templates,
            "contract_sha256": hasher.hexdigest(), "resources": resources}


def _identity(contract: Mapping[str, Any]) -> dict[str, Any]:
    profile = contract["profile"]
    return {"profile_id": profile["profile_id"], "version": profile["version"],
            "contract_sha256": contract["contract_sha256"]}


def get_contract(profile_root: str | Path | None = None) -> dict[str, Any]:
    """Return the complete versioned pack; no target project is needed."""
    contract = _contract(profile_root)
    return {"ok": True, **_identity(contract), "profile": contract["profile"],
            "rules": contract["rules"], "templates": contract["templates"],
            "semantic_review_required": True}


def _class_plan(target_root: str | Path, document_class: str, name: str,
                profile_root: str | Path | None) -> tuple[dict[str, Any], Path, dict[str, Any], str]:
    contract = _contract(profile_root)
    target = _root(target_root, "Target project root")
    profile = contract["profile"]
    if not isinstance(document_class, str) or document_class not in profile["document_classes"]:
        raise _error("invalid-request", f"Unknown document class: {document_class}")
    spec = profile["document_classes"][document_class]
    if (not isinstance(name, str)
            or re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", name) is None
            or re.fullmatch(spec["filename_pattern"], name) is None):
        raise _error("invalid-document-name", f"Invalid document name: {name}", "DOC-001")
    relative = _safe_rel(spec["path_template"].replace("{name}", name), "Document path").as_posix()
    docs_root = _safe_rel(profile["project"]["docs_root"], "Documentation root").parts
    path_parts = PurePosixPath(relative).parts
    if path_parts[:len(docs_root)] != docs_root or not relative.endswith(".md"):
        raise _error("invalid-profile", "Document route must stay under docs_root")
    _reject_symlinks(target, PurePosixPath(relative))
    return contract, target, spec, relative


def plan_document(target_root: str | Path, document_class: str, name: str,
                  profile_root: str | Path | None = None) -> dict[str, Any]:
    """Resolve a class route and required inputs without writing."""
    contract, target, spec, relative = _class_plan(target_root, document_class, name, profile_root)
    path = target.joinpath(*PurePosixPath(relative).parts)
    exists = path.exists()
    digest = hashlib.sha256(_read_project_file(target, relative)).hexdigest() if exists else None
    return {"ok": True, **_identity(contract), "document_class": document_class,
            "name": name, "path": relative, "exists": exists, "sha256": digest,
            "required_metadata": ["title", "status", "revision", "model_sources"],
            "required_sections": spec["required_sections"],
            "anchor_policy": contract["profile"]["anchor_policy"],
            "figure_block": contract["profile"]["figure_block"],
            "semantic_review_required": True}


def _sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _model_files(target: Path, profile: Mapping[str, Any]) -> list[str]:
    project = profile["project"]
    required = project.get("model_roots")
    optional = project.get("optional_model_roots", [])
    if not isinstance(required, list) or not required or not isinstance(optional, list):
        raise _error("invalid-profile", "Model roots must be nonempty required and optional path lists")
    found: list[str] = []
    for root_text in [*required, *optional]:
        root_rel = _safe_rel(root_text, "Model root", "invalid-profile")
        _reject_symlinks(target, root_rel)
        root = target.joinpath(*root_rel.parts)
        if not root.exists() and root_text in optional:
            continue
        if not root.is_dir():
            raise _error("missing-model-root", f"Model root is missing or unreadable: {root_text}",
                         "DOC-004", root_text)
        try:
            def scan_error(error: OSError) -> None:
                raise error
            for current, dirs, files in os.walk(root, followlinks=False, onerror=scan_error):
                current_path = Path(current)
                for directory in list(dirs):
                    if (current_path / directory).is_symlink():
                        raise _error("unsafe-path", f"Symlink inside model root: {current_path / directory}",
                                     "DOC-001")
                for filename in files:
                    file_path = current_path / filename
                    if file_path.suffix != ".sysml":
                        continue
                    if file_path.is_symlink() or not file_path.is_file():
                        raise _error("unsafe-path", f"Unsafe model source: {file_path}", "DOC-001")
                    relative = file_path.relative_to(target).as_posix()
                    _read_project_file(target, relative, "missing-model-source")
                    found.append(relative)
        except OSError as error:
            raise _error("unreadable-input", f"Cannot scan model root {root_text}: {error}",
                         "DOC-004", root_text) from error
    if not any(value.startswith(tuple(path + "/" for path in required)) for value in found):
        raise _error("missing-model-root", "Required model roots contain no readable .sysml file", "DOC-004")
    return sorted(set(found))


def _model_sources(target: Path, profile: Mapping[str, Any], value: Any,
                   known: list[str]) -> str:
    if not isinstance(value, list) or not value:
        raise _error("missing-metadata", "model_sources must be a nonempty list", "DOC-002")
    roots = [*_root_list(profile["project"], "model_roots"),
             *_root_list(profile["project"], "optional_model_roots")]
    output: list[str] = []
    for item in value:
        if not isinstance(item, str) or "`" in item:
            raise _error("missing-model-source", "Model source must be a path without backticks", "DOC-004")
        rel = _safe_rel(item, "Model source")
        if rel.suffix != ".sysml" or not any(item.startswith(root + "/") for root in roots):
            raise _error("missing-model-source", f"Source outside configured model roots: {item}",
                         "DOC-004", item)
        if item in output:
            raise _error("missing-model-source", f"Duplicate model source: {item}", "DOC-004", item)
        if item not in known:
            raise _error("missing-model-source", f"Missing model source: {item}", "DOC-004", item)
        output.append(item)
    return ", ".join(f"`{item}`" for item in output)


def _root_list(project: Mapping[str, Any], key: str) -> list[str]:
    value = project.get(key, [])
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise _error("invalid-profile", f"{key} must be a list of paths")
    return value


def _document_parts(contract: Mapping[str, Any], target: Path, document_class: str,
                    metadata: Any, sections: Any, known_models: list[str]) -> tuple[str, list[str]]:
    spec = contract["profile"]["document_classes"][document_class]
    if not isinstance(metadata, dict) or set(metadata) != {"title", "status", "revision", "model_sources"}:
        raise _error("missing-metadata", "Metadata needs title, status, revision, and model_sources", "DOC-002")
    if not isinstance(sections, dict) or set(sections) != {item["key"] for item in spec["required_sections"]}:
        raise _error("invalid-request", "Sections must match the class's required keys", "DOC-002")
    values: dict[str, str] = {key: _text(metadata[key], key) for key in ("title", "status", "revision")}
    values["model_sources"] = _model_sources(target, contract["profile"], metadata["model_sources"], known_models)
    anchors: list[str] = []
    for part in spec["required_sections"]:
        key = part["key"]
        section = sections[key]
        if not isinstance(section, dict) or set(section) != {"anchor", "content", "fields"}:
            raise _error("invalid-request", f"Section {key} needs anchor, content, and fields", "DOC-002")
        anchor = section["anchor"]
        if not isinstance(anchor, str) or re.fullmatch(contract["profile"]["anchor_policy"]["format"], anchor) is None:
            raise _error("missing-anchor", f"Invalid anchor in section {key}", "DOC-002")
        if anchor in anchors:
            raise _error("duplicate-anchor", f"Duplicate anchor: {anchor}", "DOC-002")
        anchors.append(anchor)
        values[f"section_id:{key}"] = anchor
        content = _text(section["content"], f"Section {key} content", "empty-content")
        values[f"content:{key}"] = f"{CONTENT_START.format(key=key)}\n{content}\n{CONTENT_END.format(key=key)}"
        fields = section["fields"]
        if not isinstance(fields, dict) or set(fields) != set(part["required_fields"]):
            raise _error("missing-field", f"Fields in section {key} do not match the template", "DOC-002")
        for field in part["required_fields"]:
            values[f"field:{field}"] = _text(fields[field], field, "missing-field")
    template = contract["templates"][document_class]
    def substitute(match: re.Match[str]) -> str:
        token = match.group(1)
        if token not in values:
            raise _error("invalid-profile", f"Unknown template token {token}")
        return values[token]
    rendered = TOKEN.sub(substitute, template)
    if not rendered.endswith("\n"):
        rendered += "\n"
    return rendered, anchors


def _registry_path(contract: Mapping[str, Any]) -> str:
    return _safe_rel(contract["profile"]["project"]["docs_root"] + "/" + REGISTRY_NAME,
                     "Registry path").as_posix()


def _load_registry(target: Path, contract: Mapping[str, Any]) -> dict[str, Any]:
    relative = _registry_path(contract)
    if not target.joinpath(*PurePosixPath(relative).parts).exists():
        return {"schema_version": 1, "entries": []}
    try:
        data = json.loads(_read_project_file(target, relative).decode("utf-8"), object_pairs_hook=_unique_pairs)
    except (UnicodeError, ValueError) as error:
        raise _error("registry-mismatch", f"Registry is invalid: {error}", "DOC-002", relative) from error
    if (not isinstance(data, dict) or data.get("schema_version") != 1
            or not isinstance(data.get("entries"), list)):
        raise _error("registry-mismatch", "Registry shape is invalid", "DOC-002", relative)
    paths: set[str] = set()
    for entry in data["entries"]:
        if (not isinstance(entry, dict) or not all(key in entry for key in
                ("path", "class", "profile_id", "version", "contract_sha256"))
                or not all(isinstance(entry[key], str) for key in
                           ("path", "class", "profile_id", "version", "contract_sha256"))):
            raise _error("registry-mismatch", "Registry entry is invalid", "DOC-002", relative)
        path = _safe_rel(entry["path"], "Registered document")
        if path.as_posix() in paths:
            raise _error("registry-mismatch", "Duplicate registry document", "DOC-002", relative)
        paths.add(path.as_posix())
    return data


def _registry_entry(contract: Mapping[str, Any], relative: str, document_class: str) -> dict[str, Any]:
    return {"path": relative, "class": document_class, **_identity(contract)}


def _registry_bytes(data: Mapping[str, Any]) -> bytes:
    entries = sorted(data["entries"], key=lambda item: item["path"])
    return (json.dumps({"schema_version": 1, "entries": entries}, ensure_ascii=False,
                       indent=2, sort_keys=True) + "\n").encode("utf-8")


@contextmanager
def _writer_lock(target: Path) -> Iterator[None]:
    cache_rel = PurePosixPath("cache/document-kit.lock")
    with _parent_fd(target, cache_rel, create=True) as (parent, name):
        fd = os.open(name, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600, dir_fd=parent)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX)
            yield
        finally:
            fcntl.flock(fd, fcntl.LOCK_UN)
            os.close(fd)


def _stage(target: Path, data: bytes) -> str:
    relative = f"cache/document-kit-{uuid.uuid4().hex}.tmp"
    with _parent_fd(target, PurePosixPath(relative), create=True) as (parent, name):
        fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                     0o600, dir_fd=parent)
        try:
            with os.fdopen(fd, "wb") as stream:
                fd = -1
                stream.write(data)
                stream.flush()
                os.fsync(stream.fileno())
        finally:
            if fd >= 0:
                os.close(fd)
    return relative


def _replace(target: Path, relative: str, data: bytes, *, create: bool = False) -> None:
    staged = _stage(target, data)
    try:
        with _parent_fd(target, PurePosixPath(staged)) as (source_dir, source_name):
            with _parent_fd(target, PurePosixPath(relative), create=True) as (dest_dir, dest_name):
                try:
                    if create:
                        os.link(source_name, dest_name, src_dir_fd=source_dir,
                                dst_dir_fd=dest_dir, follow_symlinks=False)
                    else:
                        os.replace(source_name, dest_name, src_dir_fd=source_dir,
                                   dst_dir_fd=dest_dir)
                except FileExistsError as error:
                    raise _error("document-exists", f"Document already exists: {relative}",
                                 "DOC-001", relative) from error
                except OSError as error:
                    raise _error("atomic-replace-failed", f"Cannot atomically write {relative}: {error}",
                                 "DOC-001", relative) from error
    finally:
        try:
            (target / staged).unlink()
        except FileNotFoundError:
            pass


def _expected_contract(contract: Mapping[str, Any], expected: str | None) -> None:
    if expected is None:
        return
    if not isinstance(expected, str) or contract["contract_sha256"] != expected:
        raise _error("contract-drift", "Selected contract changed during the operation")


def _registry_matches(registry: Mapping[str, Any], relative: str,
                      contract: Mapping[str, Any], document_class: str) -> bool:
    wanted = _registry_entry(contract, relative, document_class)
    return any(all(entry.get(key) == value for key, value in wanted.items())
               for entry in registry["entries"])


def _masked_lines(source: str) -> list[str]:
    """Hide fenced examples while retaining line numbers."""
    result: list[str] = []
    fence: str | None = None
    for line in source.splitlines():
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker and fence is None:
            fence = marker.group(1)[0]
            result.append("")
            continue
        if marker and fence == marker.group(1)[0]:
            fence = None
            result.append("")
            continue
        result.append("" if fence else line)
    return result


def _anchor_map(texts: Mapping[str, str]) -> tuple[dict[str, tuple[str, int]], list[dict[str, Any]]]:
    anchors: dict[str, tuple[str, int]] = {}
    findings: list[dict[str, Any]] = []
    for relative in sorted(texts):
        for number, line in enumerate(_masked_lines(texts[relative]), 1):
            found = ANCHOR.fullmatch(line)
            if found is None:
                continue
            anchor = found.group(1).casefold()
            if anchor in anchors:
                findings.append(diagnostic("DOC-002", "duplicate-anchor", relative,
                                           f"Anchor {anchor} is also in {anchors[anchor][0]}", number))
            else:
                anchors[anchor] = (relative, number)
    return anchors, findings


def _rel_target(source: str, target: str) -> tuple[str, str | None] | None:
    if target.startswith(("https://", "http://", "mailto:", "tel:", "data:")):
        return None
    without_query = target.split("?", 1)[0]
    path_text, separator, fragment = without_query.partition("#")
    if path_text.startswith("/") or "\\" in path_text or "%" in path_text:
        return ("", fragment if separator else None)
    if not path_text:
        return (source, fragment if separator else None)
    parts = list(PurePosixPath(source).parent.parts)
    for part in path_text.split("/"):
        if part in ("", "."):
            continue
        if part == "..":
            if not parts:
                return ("", fragment if separator else None)
            parts.pop()
        else:
            parts.append(part)
    return ("/".join(parts), fragment if separator else None)


def _local_link_findings(target: Path, relative: str, text: str,
                         anchors: Mapping[str, tuple[str, int]]) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    slugs_by_path: dict[str, set[str]] = {}

    def heading_slugs(path: str) -> set[str]:
        if path in slugs_by_path:
            return slugs_by_path[path]
        if not path.endswith(".md"):
            return set()
        try:
            source = text if path == relative else _read_project_file(target, path).decode("utf-8")
        except (DocumentKitError, UnicodeError):
            return set()
        counts: dict[str, int] = {}
        slugs: set[str] = set()
        for row in _masked_lines(source):
            heading = re.match(r"^#{1,6}\s+(.+?)\s*#*\s*$", row)
            if heading is None:
                continue
            label = re.sub(r"\[([^]]+)\]\([^)]+\)", r"\1", heading.group(1))
            label = re.sub(r"<[^>]+>", "", label)
            label = re.sub(r"[^\w\s-]", "", label.casefold(), flags=re.UNICODE)
            slug = re.sub(r"\s+", "-", label.strip())
            if not slug:
                continue
            occurrence = counts.get(slug, 0)
            counts[slug] = occurrence + 1
            slugs.add(slug if occurrence == 0 else f"{slug}-{occurrence}")
        slugs_by_path[path] = slugs
        return slugs

    for number, line in enumerate(_masked_lines(text), 1):
        for found in INLINE_LINK.finditer(line):
            raw = found.group(1) or found.group(2)
            destination = _rel_target(relative, raw)
            if destination is None:
                continue
            path, fragment = destination
            safe = bool(path)
            if safe:
                try:
                    _reject_symlinks(target, _safe_rel(path, "Local link"))
                except DocumentKitError:
                    safe = False
            if not safe or (path != relative and not (target / path).is_file()):
                findings.append(diagnostic("DOC-006", "invalid-link", relative,
                                           f"Local link target does not exist: {raw}", number))
            elif fragment and (fragment.casefold() not in anchors or anchors[fragment.casefold()][0] != path):
                if fragment.casefold() not in heading_slugs(path):
                    findings.append(diagnostic("DOC-006", "invalid-link", relative,
                                               f"Local anchor does not exist: {raw}", number))
    return findings


def _managed_findings(relative: str, text: str, document_class: str,
                      contract: Mapping[str, Any], target: Path,
                      known_models: list[str]) -> list[dict[str, Any]]:
    spec = contract["profile"]["document_classes"][document_class]
    lines = _masked_lines(text)
    findings: list[dict[str, Any]] = []
    marker = contract["profile"]["validation"]["managed_marker"].replace("{document_class}", document_class)
    if not lines or lines[0].strip() != marker:
        findings.append(diagnostic("DOC-002", "registry-mismatch", relative,
                                   "Managed marker is missing or differs from registry", 1))
    h1 = [(index, line[2:].strip()) for index, line in enumerate(lines, 1) if line.startswith("# ")]
    if len(h1) != 1 or not h1[0][1]:
        findings.append(diagnostic("DOC-002", "section-headings", relative,
                                   "Managed page must have one nonempty H1", h1[0][0] if h1 else None))
    h2 = [(index, line[3:].strip()) for index, line in enumerate(lines, 1) if line.startswith("## ")]
    expected = [section["heading"] for section in spec["required_sections"]]
    if [heading for _, heading in h2] != expected:
        findings.append(diagnostic("DOC-002", "section-headings", relative,
                                   "H2 headings differ from the required ordered chapters", h2[0][0] if h2 else None))
    metadata = re.search(r"^> Status: (.+?) \| Revision: (.+?) \| Model sources: (.+)$", text, re.M)
    if metadata is None or not metadata.group(1).strip() or not metadata.group(2).strip():
        findings.append(diagnostic("DOC-002", "missing-metadata", relative,
                                   "Status, revision, or model sources are missing"))
    else:
        raw_sources = metadata.group(3).strip()
        source_items = re.findall(r"`([^`]*)`", raw_sources)
        if not source_items or ", ".join(f"`{item}`" for item in source_items) != raw_sources:
            findings.append(diagnostic("DOC-004", "missing-model-source", relative,
                                       "Model sources must be comma-separated path code spans",
                                       text[:metadata.start()].count("\n") + 1))
        else:
            try:
                _model_sources(target, contract["profile"], source_items, known_models)
            except DocumentKitError as error:
                findings.append(diagnostic("DOC-004", error.code, relative, error.message,
                                           text[:metadata.start()].count("\n") + 1))
    for pattern in contract["profile"]["validation"]["placeholder_patterns"]:
        regex = re.compile(pattern)
        for number, line in enumerate(lines, 1):
            if regex.search(line):
                findings.append(diagnostic("DOC-008", "unresolved-placeholder", relative,
                                           "Unresolved placeholder remains", number))
    for index, section in enumerate(spec["required_sections"]):
        key = section["key"]
        heading_line = h2[index][0] if index < len(h2) else None
        if heading_line is None:
            continue
        before = heading_line - 2
        while before >= 0 and not lines[before].strip():
            before -= 1
        previous = lines[before] if before >= 0 else ""
        anchor_match = ANCHOR.fullmatch(previous)
        if (anchor_match is None or re.fullmatch(
                contract["profile"]["anchor_policy"]["format"],
                anchor_match.group(1)) is None):
            findings.append(diagnostic("DOC-002", "missing-anchor", relative,
                                       f"Section {key} needs a profile-compliant anchor before H2", heading_line))
        end = h2[index + 1][0] - 1 if index + 1 < len(h2) else len(lines)
        block = lines[heading_line:end]
        start_marker = CONTENT_START.format(key=key)
        end_marker = CONTENT_END.format(key=key)
        start_positions = [offset for offset, line in enumerate(block) if line.strip() == start_marker]
        end_positions = [offset for offset, line in enumerate(block) if line.strip() == end_marker]
        field_area = block[:start_positions[0]] if start_positions else block
        for field in section["required_fields"]:
            label = field.replace("_", " ").capitalize()
            field_lines = [(heading_line + offset + 1, line) for offset, line in enumerate(field_area)
                           if line.startswith(f"**{label}:**")]
            if len(field_lines) != 1 or not field_lines[0][1].split("**", 2)[-1].strip():
                findings.append(diagnostic("DOC-002", "missing-field", relative,
                                           f"Required field {label} is empty or missing",
                                           field_lines[0][0] if field_lines else heading_line))
        if (len(start_positions) != 1 or len(end_positions) != 1
                or end_positions[0] <= start_positions[0]
                or not "\n".join(block[start_positions[0] + 1:end_positions[0]]).strip()):
            findings.append(diagnostic("DOC-003", "empty-content", relative,
                                       f"Section {key} prose is empty or boundaries are damaged", heading_line))
    findings.extend(_figure_findings(target, relative, text, contract, managed=True))
    return findings


def _figure_findings(target: Path, relative: str, text: str,
                     contract: Mapping[str, Any], managed: bool) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    lines = _masked_lines(text)
    figure_numbers: set[str] = set()
    seen_assets: set[str] = set()
    for number, line in enumerate(lines, 1):
        image = IMAGE.fullmatch(line)
        if image is None:
            continue
        asset = image.group(2)
        if managed and re.fullmatch(r"assets/[a-z0-9]+(?:-[a-z0-9]+)*\.svg", asset) is None:
            findings.append(diagnostic("DOC-005", "figure-block", relative,
                                       f"Figure asset path is not semantic: {asset}", number))
        destination = _rel_target(relative, asset)
        asset_ok = False
        if destination is not None and destination[0]:
            try:
                safe_path = _safe_rel(destination[0], "Figure asset")
                _reject_symlinks(target, safe_path)
                asset_ok = (target / safe_path).is_file()
            except DocumentKitError:
                asset_ok = False
        if not asset_ok:
            findings.append(diagnostic("DOC-005", "missing-figure", relative,
                                       f"Figure asset does not exist: {asset}", number))
        if not managed:
            continue
        if asset in seen_assets:
            findings.append(diagnostic("DOC-005", "duplicate-figure", relative,
                                       f"Repeated figure asset: {asset}", number))
        seen_assets.add(asset)
        title = (FIGURE_TITLE.fullmatch(lines[number + 1].rstrip())
                 if number + 1 < len(lines) and not lines[number].strip() else None)
        if title is None:
            findings.append(diagnostic("DOC-005", "figure-block", relative,
                                       "Figure has no adjacent numbered block", number))
            continue
        figure_number = f"{title.group(1)}-{title.group(2)}"
        headings = [index for index, row in enumerate(lines[:number - 1]) if row.startswith("## ")]
        carrying_anchor: str | None = None
        if headings:
            before_heading = headings[-1] - 1
            while before_heading >= 0 and not lines[before_heading].strip():
                before_heading -= 1
            found_anchor = ANCHOR.fullmatch(lines[before_heading]) if before_heading >= 0 else None
            carrying_anchor = found_anchor.group(1) if found_anchor else None
        if (carrying_anchor is None or carrying_anchor.casefold() != title.group(1).casefold()
                or int(title.group(2)) < 1 or image.group(1) != title.group(3)):
            findings.append(diagnostic("DOC-005", "figure-block", relative,
                                       "Figure number or title does not match its carrying section and embed", number))
        if figure_number in figure_numbers:
            findings.append(diagnostic("DOC-005", "duplicate-figure", relative,
                                       f"Repeated figure number: {figure_number}", number))
        figure_numbers.add(figure_number)
        values: dict[str, str] = {}
        ordered_seen: list[str] = []
        for item in lines[number + 2:number + 8]:
            field = FIELD_LINE.fullmatch(item.rstrip())
            if field:
                values[field.group(1)] = field.group(2).strip()
                ordered_seen.append(field.group(1))
        ordered = contract["profile"]["figure_block"]["ordered_fields"]
        if any(not values.get(field) for field in ordered) or ordered_seen != ordered:
            findings.append(diagnostic("DOC-005", "figure-block", relative,
                                       "Figure block has empty or missing required fields", number))
        status = values.get("Status", "")
        if status and status.split(";", 1)[0].strip() not in contract["profile"]["figure_block"]["statuses"]:
            findings.append(diagnostic("DOC-005", "figure-status", relative,
                                       f"Unknown primary figure status: {status}", number))
    if managed:
        for number, line in enumerate(lines, 1):
            if FIGURE_TITLE.fullmatch(line.rstrip()) and not (number >= 3 and IMAGE.fullmatch(lines[number - 3])):
                findings.append(diagnostic("DOC-005", "figure-block", relative,
                                           "Numbered figure block has no adjacent embed", number))
    return findings


def _sysml_without_comments(source: str) -> str:
    result: list[str] = []
    index = 0
    mode = "code"
    while index < len(source):
        pair = source[index:index + 2]
        if mode == "code" and pair == "//":
            mode = "line"
            result.extend("  ")
            index += 2
        elif mode == "code" and pair == "/*":
            mode = "block"
            result.extend("  ")
            index += 2
        elif mode == "block" and pair == "*/":
            mode = "code"
            result.extend("  ")
            index += 2
        elif mode == "line" and source[index] == "\n":
            mode = "code"
            result.append("\n")
            index += 1
        elif mode in ("line", "block"):
            result.append("\n" if source[index] == "\n" else " ")
            index += 1
        elif source[index] == '"':
            result.append('"')
            index += 1
            while index < len(source):
                result.append(source[index])
                if source[index] == "\\" and index + 1 < len(source):
                    index += 1
                    result.append(source[index])
                elif source[index] == '"':
                    index += 1
                    break
                index += 1
        else:
            result.append(source[index])
            index += 1
    return "".join(result)


def _annotation_findings(target: Path, models: list[str], texts: Mapping[str, str],
                         anchors: Mapping[str, tuple[str, int]]) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    for model in models:
        try:
            source = _sysml_without_comments(_read_project_file(target, model).decode("utf-8"))
        except UnicodeError:
            findings.append(diagnostic("DOC-004", "unreadable-input", model,
                                       "Model source is not UTF-8"))
            continue
        for match in ANNOTATION_START.finditer(source):
            kind = match.group(1)
            line = source.count("\n", 0, match.start()) + 1
            tail = source[match.end():]
            body = re.match(r"\s*\{([^{}]*)\}", tail, re.S)
            if body is None:
                findings.append(diagnostic("DOC-004", "unsupported-annotation", model,
                                           f"Unsupported {kind} locator form", line))
                continue
            assignments = list(FIELD_ASSIGNMENT.finditer(body.group(1)))
            fields = {item.group(1): item.group(2) for item in assignments}
            expected = {"document", "section"} if kind == "DocSection" else {"document", "figure"}
            residue = FIELD_ASSIGNMENT.sub("", body.group(1)).replace(";", "").strip()
            if set(fields) != expected or len(assignments) != len(expected) or residue:
                findings.append(diagnostic("DOC-004", "unsupported-annotation", model,
                                           f"Malformed {kind} locator fields", line))
                continue
            document = fields["document"]
            try:
                _safe_rel(document, "Locator document")
            except DocumentKitError:
                findings.append(diagnostic("DOC-004", "unsupported-annotation", model,
                                           f"Unsafe {kind} document path", line))
                continue
            if document not in texts:
                findings.append(diagnostic("DOC-004", "invalid-docsection" if kind == "DocSection" else "invalid-docfigure",
                                           model, f"Locator document does not exist: {document}", line))
            elif kind == "DocSection":
                section = fields["section"]
                if section.casefold() not in anchors or anchors[section.casefold()][0] != document:
                    findings.append(diagnostic("DOC-004", "invalid-docsection", model,
                                               f"Missing section {section} in {document}", line))
            else:
                figure = fields["figure"]
                try:
                    figure_path = _safe_rel(figure, "Locator figure")
                    _reject_symlinks(target, figure_path)
                except DocumentKitError:
                    findings.append(diagnostic("DOC-004", "unsupported-annotation", model,
                                               f"Unsafe DocFigure asset: {figure}", line))
                    continue
                embedded: list[str] = []
                for row in _masked_lines(texts[document]):
                    image = IMAGE.fullmatch(row)
                    if image:
                        resolved = _rel_target(document, image.group(2))
                        if resolved:
                            embedded.append(resolved[0])
                if not (target / figure).is_file() or figure not in embedded:
                    findings.append(diagnostic("DOC-004", "invalid-docfigure", model,
                                               f"Figure {figure} is absent or not embedded in {document}", line))
    return findings


def check_documents(target_root: str | Path, document_class: str | None = None,
                    profile_root: str | Path | None = None) -> dict[str, Any]:
    """Validate managed and legacy docs with model locator targets."""
    contract = _contract(profile_root)
    target = _root(target_root, "Target project root")
    profile = contract["profile"]
    if document_class is not None and document_class not in profile["document_classes"]:
        raise _error("invalid-request", f"Unknown document class: {document_class}")
    models = _model_files(target, profile)
    docs_root_rel = _safe_rel(profile["project"]["docs_root"], "Documentation root")
    _reject_symlinks(target, docs_root_rel)
    docs_root = target.joinpath(*docs_root_rel.parts)
    if not docs_root.exists():
        texts: dict[str, str] = {}
        bytes_by_path: dict[str, bytes] = {}
    else:
        texts, bytes_by_path = {}, {}
        for path in _docs_files(target, docs_root_rel.as_posix()):
            relative = path.relative_to(target).as_posix()
            _reject_symlinks(target, PurePosixPath(relative))
            raw = _read_project_file(target, relative)
            try:
                texts[relative] = raw.decode("utf-8")
            except UnicodeError:
                raise _error("unreadable-input", f"Document is not UTF-8: {relative}", "DOC-002", relative)
            bytes_by_path[relative] = raw
    registry = _load_registry(target, contract)
    anchors, global_findings = _anchor_map(texts)
    document_findings: list[dict[str, Any]] = []
    registry_paths = {entry["path"]: entry for entry in registry["entries"]}
    for relative, entry in sorted(registry_paths.items()):
        if relative not in texts:
            global_findings.append(diagnostic("DOC-002", "registry-mismatch", relative,
                                              "Registered document is missing"))
        elif entry != _registry_entry(contract, relative, entry["class"]):
            global_findings.append(diagnostic("DOC-002", "registry-mismatch", relative,
                                              "Registry contract identity differs from selected profile"))
    for relative, text in sorted(texts.items()):
        if document_class is not None:
            route = profile["document_classes"][document_class]["path_template"]
            if not relative.startswith(route.split("{name}")[0]):
                continue
        document_findings.extend(_local_link_findings(target, relative, text, anchors))
        entry = registry_paths.get(relative)
        if entry:
            class_name = entry["class"]
            if class_name not in profile["document_classes"]:
                global_findings.append(diagnostic("DOC-001", "registry-mismatch", relative,
                                                  "Registered class is unknown"))
                continue
            spec = profile["document_classes"][class_name]
            stem = PurePosixPath(relative).stem
            if (re.fullmatch(spec["filename_pattern"], stem) is None or
                    spec["path_template"].replace("{name}", stem) != relative):
                document_findings.append(diagnostic("DOC-001", "invalid-document-name", relative,
                                                    "Managed path differs from class route"))
            document_findings.extend(_managed_findings(relative, text, class_name, contract, target, models))
        else:
            # Existing pages keep baseline navigation and model/figure checks.
            if text.startswith("<!-- document-kit: profile="):
                global_findings.append(diagnostic("DOC-002", "registry-mismatch", relative,
                                                  "Managed marker has no registry entry", 1))
            document_findings.extend(_figure_findings(target, relative, text, contract, managed=False))
    global_findings.extend(_annotation_findings(target, models, texts, anchors))
    global_findings.sort(key=lambda item: (item["file"], item["line"] or 0,
                                           item["code"], item["detail"]))
    findings = [*global_findings, *document_findings]
    findings.sort(key=lambda item: (item["file"], item["line"] or 0, item["code"], item["detail"]))
    if document_class is None:
        checked = sorted(bytes_by_path)
    else:
        route_prefix = profile["document_classes"][document_class]["path_template"].split("{name}")[0]
        checked = sorted(path for path in bytes_by_path if path.startswith(route_prefix))
    hashes = {path: _sha(bytes_by_path[path]) for path in checked}
    return {"ok": not findings, **_identity(contract), "checked_paths": checked,
            "sha256_by_path": hashes, "document_hashes": hashes,
            "scanned_paths": sorted(bytes_by_path),
            "document_class_scope": document_class,
            "scope": {"document_class": document_class,
                      "document_checks": checked,
                      "global_checks": ["anchor-uniqueness", "registry-integrity", "model-locators"]},
            "global_findings": global_findings,
            "diagnostics": findings, "semantic_review_required": True,
            "unevaluated_checks": ["SysML compiler syntax/name resolution", "engineering adequacy", "figure readability"]}


def _existing_texts(target: Path, docs_root: str) -> dict[str, str]:
    root = target / docs_root
    texts: dict[str, str] = {}
    if not root.exists():
        return texts
    for path in _docs_files(target, docs_root):
        relative = path.relative_to(target).as_posix()
        _reject_symlinks(target, PurePosixPath(relative))
        try:
            texts[relative] = _read_project_file(target, relative).decode("utf-8")
        except UnicodeError as error:
            raise _error("unreadable-input", f"Document is not UTF-8: {relative}", "DOC-002", relative) from error
    return texts


def _docs_files(target: Path, docs_root: str) -> list[Path]:
    root_rel = _safe_rel(docs_root, "Documentation root")
    _reject_symlinks(target, root_rel)
    root = target.joinpath(*root_rel.parts)
    files: list[Path] = []
    if not root.exists():
        return files
    def scan_error(error: OSError) -> None:
        raise error
    try:
        for current, dirs, names in os.walk(root, followlinks=False, onerror=scan_error):
            current_path = Path(current)
            for name in [*dirs, *names]:
                path = current_path / name
                if path.is_symlink():
                    raise _error("unsafe-path", f"Symlink inside docs root: {path}", "DOC-001")
            files.extend(current_path / name for name in names if name.endswith(".md"))
    except OSError as error:
        raise _error("unreadable-input", f"Cannot scan documentation root: {error}", "DOC-001") from error
    return sorted(files)


def _candidate_findings(target: Path, contract: Mapping[str, Any], relative: str,
                        document_class: str, text: str) -> list[dict[str, Any]]:
    texts = _existing_texts(target, contract["profile"]["project"]["docs_root"])
    texts[relative] = text
    anchors, findings = _anchor_map(texts)
    findings.extend(_managed_findings(relative, text, document_class, contract, target,
                                      _model_files(target, contract["profile"])))
    findings.extend(_local_link_findings(target, relative, text, anchors))
    return findings


def _validate_candidate(target: Path, contract: Mapping[str, Any], relative: str,
                        document_class: str, text: str) -> None:
    findings = _candidate_findings(target, contract, relative, document_class, text)
    if findings:
        first = findings[0]
        raise DocumentKitError(first["code"], first["detail"], findings)


def _commit_contract(profile_root: str | Path | None, first: Mapping[str, Any],
                     expected: str | None) -> None:
    current = _contract(profile_root)
    if current["contract_sha256"] != first["contract_sha256"]:
        raise _error("contract-drift", "Contract bytes changed before write commit")
    _expected_contract(current, expected)


def create_document(target_root: str | Path, document_class: str, name: str,
                    metadata: Mapping[str, Any], sections: Mapping[str, Any],
                    profile_root: str | Path | None = None,
                    expected_contract_sha256: str | None = None) -> dict[str, Any]:
    """Create one complete page and register the pinned contract without overwriting."""
    contract, target, _, relative = _class_plan(target_root, document_class, name, profile_root)
    _expected_contract(contract, expected_contract_sha256)
    known = _model_files(target, contract["profile"])
    rendered, anchors = _document_parts(contract, target, document_class, metadata, sections, known)
    with _writer_lock(target):
        if (target / relative).exists():
            raise _error("document-exists", f"Document already exists: {relative}", "DOC-001", relative)
        registry = _load_registry(target, contract)
        if any(entry["path"] == relative for entry in registry["entries"]):
            raise _error("registry-mismatch", f"Registry already owns {relative}", "DOC-002", relative)
        _validate_candidate(target, contract, relative, document_class, rendered)
        _commit_contract(profile_root, contract, expected_contract_sha256)
        data = rendered.encode("utf-8")
        _replace(target, relative, data, create=True)
        registry["entries"].append(_registry_entry(contract, relative, document_class))
        _replace(target, _registry_path(contract), _registry_bytes(registry))
    return {"ok": True, **_identity(contract), "path": relative, "sha256": _sha(data),
            "anchors": anchors, "semantic_review_required": True}


def _managed_current(target: Path, contract: Mapping[str, Any], relative: str,
                     document_class: str, expected_sha256: str) -> tuple[str, dict[str, Any]]:
    if not isinstance(expected_sha256, str) or re.fullmatch(r"[0-9a-f]{64}", expected_sha256) is None:
        raise _error("invalid-request", "expected_sha256 must be a lowercase SHA-256 hex digest")
    registry = _load_registry(target, contract)
    if not _registry_matches(registry, relative, contract, document_class):
        raise _error("registry-mismatch", f"Selected contract does not own {relative}", "DOC-002", relative)
    data = _read_project_file(target, relative)
    if _sha(data) != expected_sha256:
        raise _error("stale-document", f"Document changed since expected_sha256: {relative}", "DOC-001", relative)
    try:
        text = data.decode("utf-8")
    except UnicodeError as error:
        raise _error("unreadable-input", f"Document is not UTF-8: {relative}", "DOC-002", relative) from error
    marker = contract["profile"]["validation"]["managed_marker"].replace("{document_class}", document_class)
    if not text.startswith(marker + "\n"):
        raise _error("registry-mismatch", f"Managed marker is absent: {relative}", "DOC-002", relative)
    return text, registry


def _section_bounds(text: str, key: str) -> tuple[int, int]:
    start = CONTENT_START.format(key=key)
    end = CONTENT_END.format(key=key)
    if text.count(start) != 1 or text.count(end) != 1:
        raise _error("empty-content", f"Content boundaries for {key} are missing or duplicated", "DOC-002")
    begin = text.index(start) + len(start)
    finish = text.index(end)
    if finish <= begin:
        raise _error("empty-content", f"Content boundaries for {key} are reversed", "DOC-002")
    return begin, finish


def _section_span(text: str, spec: Mapping[str, Any], key: str) -> tuple[int, int]:
    parts = spec["required_sections"]
    index = next((number for number, item in enumerate(parts) if item["key"] == key), None)
    if index is None:
        raise _error("invalid-request", f"Unknown section key: {key}")
    heading = "## " + parts[index]["heading"]
    start = text.find(heading)
    if start < 0:
        raise _error("section-headings", f"Missing section heading: {heading}", "DOC-002")
    next_start = text.find("\n## ", start + len(heading))
    return start, len(text) if next_start < 0 else next_start + 1


def update_section(target_root: str | Path, document_class: str, name: str,
                   section_key: str, content: str, expected_sha256: str,
                   fields: Mapping[str, str] | None = None,
                   profile_root: str | Path | None = None,
                   expected_contract_sha256: str | None = None) -> dict[str, Any]:
    """Replace one section's prose and optionally its required labeled fields."""
    contract, target, spec, relative = _class_plan(target_root, document_class, name, profile_root)
    _expected_contract(contract, expected_contract_sha256)
    _model_files(target, contract["profile"])
    if not any(item["key"] == section_key for item in spec["required_sections"]):
        raise _error("invalid-request", f"Unknown section key: {section_key}")
    replacement = _text(content, "Section content", "empty-content")
    with _writer_lock(target):
        source, _ = _managed_current(target, contract, relative, document_class, expected_sha256)
        begin, finish = _section_bounds(source, section_key)
        changed = source[:begin] + "\n" + replacement + "\n" + source[finish:]
        if fields is not None:
            section = next(item for item in spec["required_sections"] if item["key"] == section_key)
            if not isinstance(fields, dict) or not set(fields).issubset(set(section["required_fields"])):
                raise _error("missing-field", f"Unexpected fields in {section_key}", "DOC-002")
            start, end = _section_span(changed, spec, section_key)
            block = changed[start:end]
            field_end = block.find(CONTENT_START.format(key=section_key))
            if field_end < 0:
                raise _error("empty-content", f"Section {section_key} has no content marker", "DOC-002")
            field_area = block[:field_end]
            for field, value in fields.items():
                label = field.replace("_", " ").capitalize()
                pattern = re.compile(rf"(?m)^(\*\*{re.escape(label)}:\*\*)[^\n]*$")
                if len(pattern.findall(field_area)) != 1:
                    raise _error("missing-field", f"Field {label} is missing", "DOC-002")
                field_area = pattern.sub(lambda match: match.group(1) + " " + _text(value, field, "missing-field"),
                                         field_area)
            changed = changed[:start] + field_area + block[field_end:] + changed[end:]
        _validate_candidate(target, contract, relative, document_class, changed)
        _commit_contract(profile_root, contract, expected_contract_sha256)
        if _sha(_read_project_file(target, relative)) != expected_sha256:
            raise _error("stale-document", f"Document changed before commit: {relative}", "DOC-001", relative)
        data = changed.encode("utf-8")
        _replace(target, relative, data)
    return {"ok": True, **_identity(contract), "path": relative, "sha256": _sha(data),
            "semantic_review_required": True}


def attach_figure(target_root: str | Path, document_class: str, name: str,
                  section_key: str, figure: Mapping[str, Any], expected_sha256: str,
                  profile_root: str | Path | None = None,
                  expected_contract_sha256: str | None = None) -> dict[str, Any]:
    """Add one supplied figure answer beside an existing canonical SVG."""
    contract, target, spec, relative = _class_plan(target_root, document_class, name, profile_root)
    _expected_contract(contract, expected_contract_sha256)
    _model_files(target, contract["profile"])
    if not any(item["key"] == section_key for item in spec["required_sections"]):
        raise _error("invalid-request", f"Unknown section key: {section_key}")
    required = {"asset", "title", *contract["profile"]["figure_block"]["ordered_fields"]}
    if not isinstance(figure, dict) or set(figure) != required:
        raise _error("invalid-request", "Figure needs asset, title, and all six fields")
    asset = figure["asset"]
    if (not isinstance(asset, str) or re.fullmatch(r"assets/[a-z0-9]+(?:-[a-z0-9]+)*\.svg", asset) is None):
        raise _error("unsafe-path", "Figure asset must be semantic assets/<name>.svg", "DOC-005", relative)
    asset_relative = PurePosixPath(relative).parent / asset
    _reject_symlinks(target, asset_relative)
    if not (target / asset_relative).is_file():
        raise _error("missing-figure", f"Canonical SVG does not exist: {asset_relative}", "DOC-005", relative)
    for field in required - {"asset"}:
        _text(figure[field], field, "figure-block")
    primary = figure["Status"].split(";", 1)[0].strip()
    if primary not in contract["profile"]["figure_block"]["statuses"]:
        raise _error("figure-status", f"Unknown primary figure status: {primary}", "DOC-005", relative)
    with _writer_lock(target):
        source, _ = _managed_current(target, contract, relative, document_class, expected_sha256)
        if f"]({asset})" in source:
            raise _error("duplicate-figure", f"Asset is already embedded: {asset}", "DOC-005", relative)
        start, end = _section_span(source, spec, section_key)
        block = source[start:end]
        heading_line = source[:start].count("\n") + 1
        before = source[:start].splitlines()
        anchor = next((found.group(1) for item in reversed(before[-3:])
                       if (found := ANCHOR.fullmatch(item))), None)
        if anchor is None:
            raise _error("missing-anchor", f"Section {section_key} lacks an anchor", "DOC-002", relative, heading_line)
        ordinals = [int(found.group(1)) for found in re.finditer(
            rf"(?m)^> \*\*Figure {re.escape(anchor)}-(\d+) —", block)]
        ordinal = max(ordinals, default=0) + 1
        values = {"asset": asset, "title": _text(figure["title"], "Figure title", "figure-block"),
                  "section_id": anchor, "ordinal": str(ordinal)}
        values.update({token: _text(figure[label], label, "figure-block")
                       for label, token in contract["profile"]["figure_block"]["field_tokens"].items()})
        template = contract["templates"]["figure-block"]
        rendered = TOKEN.sub(lambda found: values[found.group(1)], template)
        _, finish = _section_bounds(source, section_key)
        insertion = finish + len(CONTENT_END.format(key=section_key))
        changed = source[:insertion] + "\n\n" + rendered.rstrip() + "\n" + source[insertion:]
        _validate_candidate(target, contract, relative, document_class, changed)
        _commit_contract(profile_root, contract, expected_contract_sha256)
        if _sha(_read_project_file(target, relative)) != expected_sha256:
            raise _error("stale-document", f"Document changed before commit: {relative}", "DOC-001", relative)
        data = changed.encode("utf-8")
        _replace(target, relative, data)
    return {"ok": True, **_identity(contract), "path": relative, "sha256": _sha(data),
            "figure_number": f"{anchor}-{ordinal}", "provenance_review_notes": [
                "Review that the SVG faithfully renders the cited SysML view"],
            "semantic_review_required": True}
