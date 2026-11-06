from __future__ import annotations

import hashlib
import json
import os
import re
import stat
from contextlib import contextmanager
from pathlib import Path, PurePosixPath
from typing import Any, Iterator, Mapping


class DocumentKitError(ValueError):
    def __init__(self, code: str, message: str, diagnostics: list[dict[str, Any]] | None = None, outcome: dict[str, Any] | None = None):
        super().__init__(message)
        self.outcome = outcome
        self.code = code
        self.message = message
        self.diagnostics = diagnostics if diagnostics is not None else [
            diagnostic("input", code, "", message)
        ]

    def as_dict(self) -> dict[str, Any]:
        return {"ok": False, "error": {"code": self.code, "message": self.message},
                "diagnostics": self.diagnostics, **({"outcome": self.outcome} if self.outcome is not None else {})}


def diagnostic(rule: str, code: str, file: str, detail: str, line: int | None = None) -> dict[str, Any]:
    return {"rule": rule, "code": code, "file": file, "line": line, "detail": detail}


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
            or ":" in value or value.startswith("/") or "//" in value
            or any(ord(char) < 32 or ord(char) == 127 for char in value)):
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
    if any(parent.is_symlink() for parent in (path, *path.parents)) or not path.is_dir():
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
    fd = -1
    try:
        try:
            fd = os.open(str(root), os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            for part in relative.parts[:-1]:
                try:
                    next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                except FileNotFoundError:
                    if not create:
                        raise _error("missing-document", f"Missing parent of {relative}", "DOC-001", relative.as_posix())
                    os.mkdir(part, dir_fd=fd)
                    next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                os.close(fd)
                fd = next_fd
        except OSError as error:
            from .io_boundary import CURRENT
            current = CURRENT.get()
            reading = current and current.phase in ("document-read", "model-read", "registry-read")
            code = ("unreadable-input" if reading else "access-denied") if isinstance(error, PermissionError) else "unsafe-path"
            raise _error(code, f"Cannot safely access {relative}: {error}", "DOC-001", relative.as_posix()) from error
        yield fd, relative.name
    finally:
        if fd >= 0:
            os.close(fd)


def _read_project_file(root: Path, relative: str, missing_code: str = "missing-document") -> bytes:
    from .io_boundary import phase
    phase("registry-read" if relative.endswith(".json") else "model-read" if relative.endswith(".sysml") else "document-read", relative)
    path = _safe_rel(relative, "Project file")
    _reject_symlinks(root, path)
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
    from .io_boundary import phase
    phase("pack-read")
    path = _safe_rel(relative, "Contract resource", "invalid-profile")
    _reject_symlinks(root, path)
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


def _sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _load_json(root: Path, relative: str, code: str = "invalid-profile") -> Any:
    try:
        return json.loads(_read_pack_file(root, relative).decode("utf-8"),
                          object_pairs_hook=_unique_pairs)
    except (UnicodeError, ValueError) as error:
        raise _error(code, f"Invalid JSON resource {relative}: {error}") from error


def identity(contract: Mapping[str, Any], document_class: str | None = None,
             template_id: str | None = None) -> dict[str, Any]:
    profile = contract["profile"]
    result = {"profile_id": profile["profile_id"], "version": profile["version"],
              "contract_sha256": contract["contract_sha256"]}
    if document_class is not None:
        result = {"profile_id": profile["profile_id"], "version": profile["version"],
                  "document_class": document_class, "template_id": template_id,
                  "contract_sha256": contract["contract_sha256"],
                  "contract_digest_version": contract["contract_digest_version"]}
    else:
        result["contract_digest_version"] = contract["contract_digest_version"]
    return result


def marker(effective_identity: Mapping[str, Any]) -> str:
    profile_id = effective_identity["profile_id"]
    version = effective_identity["version"]
    document_class = effective_identity["document_class"]
    if effective_identity["contract_digest_version"] == "1":
        return f"<!-- document-kit: profile={profile_id}@{version} class={document_class} -->"
    return (f"<!-- document-kit: profile={profile_id}@{version} class={document_class} "
            f"template={effective_identity['template_id']} "
            f"contract={effective_identity['contract_sha256']} "
            f"digest-version={effective_identity['contract_digest_version']} -->")


def registry_entry(effective_identity: Mapping[str, Any], relative: str) -> dict[str, Any]:
    if effective_identity["contract_digest_version"] == "1":
        return {"path": relative, "class": effective_identity["document_class"],
                "profile_id": effective_identity["profile_id"],
                "version": effective_identity["version"],
                "contract_sha256": effective_identity["contract_sha256"]}
    return {"path": relative, "class": effective_identity["document_class"],
            "profile_id": effective_identity["profile_id"],
            "version": effective_identity["version"],
            "contract_sha256": effective_identity["contract_sha256"],
            "template_id": effective_identity["template_id"],
            "contract_digest_version": effective_identity["contract_digest_version"]}


def digest_v1(resources: Mapping[str, bytes]) -> str:
    hasher = hashlib.sha256()
    for path in sorted(resources, key=lambda value: value.encode("utf-8")):
        data = resources[path]
        hasher.update(path.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(len(data)).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(data)
    return hasher.hexdigest()


def digest_v2(resources: Mapping[str, bytes]) -> str:
    hasher = hashlib.sha256()
    hasher.update(b"xdesign-doc-kit-contract\0")
    hasher.update(b"2\0")
    for path in sorted(resources, key=lambda value: value.encode("utf-8")):
        data = resources[path]
        hasher.update(path.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(len(data)).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(data)
    return hasher.hexdigest()


def selected_spec(contract: Mapping[str, Any], document_class: str,
                  template_id: str | None) -> tuple[dict[str, Any], dict[str, Any]]:
    from .catalog import select_template
    selection = select_template(contract, document_class, template_id)
    return selection["spec"], selection["effective_identity"]


def resolve_document(contract: Mapping[str, Any], entry: Mapping[str, Any], text: str,
                     registry_relative: str) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    zero = "0" * 64
    identity_keys = ("profile_id", "version", "class", "template_id", "contract_sha256",
                     "contract_digest_version")
    first_line = text.split("\n", 1)[0]
    if any(entry.get(key) == zero for key in identity_keys) or f"contract={zero}" in first_line:
        raise _error("registry-mismatch", "Target identity contains the stored-example sentinel",
                     "DOC-002", registry_relative, 1)
    document_class = entry.get("class")
    if not isinstance(document_class, str):
        raise _error("registry-mismatch", "Registered class is invalid", "DOC-002", registry_relative)
    profile = contract["profile"]
    selected_version = profile["version"]
    if profile.get("schema_version") == "1.0":
        if entry.get("profile_id") != profile["profile_id"] or entry.get("version") != selected_version \
                or entry.get("contract_sha256") != contract["contract_sha256"] \
                or "template_id" in entry or "contract_digest_version" in entry:
            raise _error("registry-mismatch", "Schema 1.0 registry identity differs from selected pack",
                         "DOC-002", registry_relative)
        effective = identity(contract, document_class, None)
        if document_class not in profile["document_classes"] or not text.startswith(marker(effective) + "\n"):
            raise _error("registry-mismatch", "Original v1 managed marker or class is absent", "DOC-002",
                         registry_relative, 1)
        return selected_spec(contract, document_class, None)[0], contract, effective

    if entry.get("profile_id") == profile["profile_id"] and entry.get("version") == selected_version \
            and entry.get("contract_sha256") == contract["contract_sha256"]:
        template_id = entry.get("template_id")
        digest_version = entry.get("contract_digest_version")
        if not isinstance(template_id, str) or digest_version != "2":
            raise _error("registry-mismatch", "Schema 1.1 template identity is incomplete", "DOC-002",
                         registry_relative)
        try:
            spec, effective = selected_spec(contract, document_class, template_id)
        except DocumentKitError as error:
            raise _error("registry-mismatch", "Registry template or class differs from the selected catalog",
                         "DOC-002", registry_relative) from error
        if not text.startswith(marker(effective) + "\n"):
            raise _error("registry-mismatch", "Managed marker differs from effective identity", "DOC-002",
                         registry_relative, 1)
        return spec, contract, effective

    if profile.get("schema_version") == "1.2":
        def children(c):
            for child in c.get("compatibility_contracts", []):
                yield child
                yield from children(child)
        for child in children(contract):
            if all(entry.get(k) == identity(child)[k] for k in ("profile_id", "version", "contract_sha256")):
                return resolve_document(child, entry, text, registry_relative)

    expected = {"profile_id": "xdesign-sysml-docs", "version": "1.0.0",
                "contract_sha256": "1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6"}
    legacy_fields = entry.get("template_id") is None and entry.get("contract_digest_version") in (None, "1")
    if all(entry.get(key) == value for key, value in expected.items()) and legacy_fields:
        for compatibility in contract.get("compatibility_contracts", []):
            candidate = compatibility["profile"]
            if candidate["profile_id"] == expected["profile_id"] and candidate["version"] == expected["version"] \
                    and compatibility["contract_sha256"] == expected["contract_sha256"]:
                effective = identity(compatibility, document_class, None)
                if document_class not in candidate["document_classes"] \
                        or not text.startswith(marker(effective) + "\n"):
                    raise _error("registry-mismatch", "Bundled-v1 marker or class differs from retained contract",
                                 "DOC-002", registry_relative, 1)
                return selected_spec(compatibility, document_class, None)[0], compatibility, effective
        raise _error("unsupported-legacy-identity", "Bundled v1 compatibility resources are unavailable",
                     "DOC-002", registry_relative)

    looks_v11 = (entry.get("contract_digest_version") == "2" or entry.get("template_id") is not None
                 or " digest-version=2" in first_line or " template=" in first_line)
    if looks_v11:
        raise _error("registry-mismatch", "Document belongs to a different 1.1 release or identity; select its original pack",
                     "DOC-002", registry_relative)
    raise _error("unsupported-legacy-identity", "Unknown legacy document identity; select its original pack",
                 "DOC-002", registry_relative)
