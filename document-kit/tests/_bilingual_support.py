from __future__ import annotations

import copy
import hashlib
import json
import os
import re
import tempfile
from pathlib import Path

from xdesign_doc_kit import DocumentKitError, create_document

KIT = Path(__file__).resolve().parents[1]
REPO = KIT.parent
CACHE = Path(os.environ.get("XDESIGN_DOC_KIT_TEST_CACHE", str(REPO / "cache/document-kit-bilingual-tests")))
EVIDENCE = Path(os.environ.get("XDESIGN_DOC_KIT_BILINGUAL_EVIDENCE", str(CACHE / "evidence")))
ZERO = "0" * 64
MARKER = re.compile(
    rb"<!-- document-kit: profile=([^ @]+)@([^ ]+) class=([^ ]+) template=([^ ]+) language=([^ ]+) "
    rb"contract=([0-9a-f]{64}) digest-version=3 -->"
)


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_path(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def materialize(target: Path, fixture: dict, pack: Path = KIT) -> None:
    for item in fixture.get("models", []) + fixture.get("figures", []):
        destination = target / item["target"]
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes((pack / item["pack"]).read_bytes())


def companions(target: Path, fixture: dict) -> None:
    if fixture["document_class"] != "flow-index":
        return
    for role in ("definition", "implementation", "views"):
        path = target / f"docs/design/flows/{fixture['name']}/{role}.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"# Fictional {role}\n\nThis navigation target is intentionally unmanaged.\n", encoding="utf-8")


def create_from_fixture(target: Path, fixture: dict, pack: Path = KIT, digest: str | None = None,
                        include_companions: bool = True) -> dict:
    target.mkdir(parents=True, exist_ok=True)
    materialize(target, fixture, pack)
    if include_companions:
        companions(target, fixture)
    return create_document(
        target, fixture["document_class"], fixture["name"], fixture["metadata"], fixture["sections"],
        profile_root=pack, expected_contract_sha256=digest, template_id=fixture["template_id"],
        layer=fixture.get("layer"), language=fixture.get("language"),
    )


def selection(fixture: dict) -> dict:
    return {key: fixture.get(key) for key in ("document_class", "name", "layer")}


def expected_bytes(stored: bytes, digest: str, identity: dict) -> bytes:
    first, remainder = stored.split(b"\n", 1)
    match = MARKER.fullmatch(first)
    if match is None or stored.count(ZERO.encode("ascii")) != 1 or digest.encode("ascii") in stored:
        raise DocumentKitError("fixture-mismatch", "Stored example marker or sentinel is invalid")
    if match.group(6) != ZERO.encode("ascii"):
        raise DocumentKitError("fixture-mismatch", "Stored example contract is not the single sentinel")
    actual = {
        "profile_id": match.group(1).decode(), "version": match.group(2).decode(),
        "document_class": match.group(3).decode(), "template_id": match.group(4).decode(),
        "language": match.group(5).decode(), "contract_digest_version": "3",
    }
    wanted = {key: identity[key] for key in actual}
    if actual != wanted:
        raise DocumentKitError("fixture-mismatch", f"Stored example identity differs: {actual} != {wanted}")
    start, end = match.span(6)
    return first[:start] + digest.encode("ascii") + first[end:] + b"\n" + remainder


def assert_created_identity(case, target: Path, created: dict, fixture: dict, digest: str) -> None:
    case.assertEqual(created["language"], fixture["language"])
    expected = {
        "profile_id": "xdesign-sysml-docs", "version": "1.3.0",
        "document_class": fixture["document_class"], "template_id": fixture["template_id"],
        "language": fixture["language"], "contract_sha256": digest, "contract_digest_version": "3",
    }
    case.assertEqual(created["effective_identity"], expected)
    registry = load_json(target / "docs/.document-kit-managed.json")
    entry = next(item for item in registry["entries"] if item["path"] == created["path"])
    case.assertEqual(entry, {
        "path": created["path"], "class": fixture["document_class"],
        "profile_id": "xdesign-sysml-docs", "version": "1.3.0",
        "contract_sha256": digest, "template_id": fixture["template_id"],
        "contract_digest_version": "3", "language": fixture["language"],
    })
    document = (target / created["path"]).read_bytes()
    match = MARKER.fullmatch(document.split(b"\n", 1)[0])
    case.assertIsNotNone(match)
    case.assertEqual(match.group(5).decode(), fixture["language"])
    case.assertEqual(match.group(6).decode(), digest)
    case.assertNotIn(ZERO.encode("ascii"), document)


def compare_created_bytes(stored: bytes, created: bytes, digest: str, identity: dict) -> None:
    expected = expected_bytes(stored, digest, identity)
    if created != expected:
        raise DocumentKitError("fixture-mismatch", "Created bytes differ after exact one-field in-memory substitution")


def snapshot(root: Path) -> dict[str, str]:
    if not root.exists():
        return {}
    return {
        path.relative_to(root).as_posix(): sha_path(path)
        for path in root.rglob("*") if path.is_file() and not path.is_symlink() and "cache" not in path.relative_to(root).parts
    }


def registry_bytes(target: Path) -> bytes:
    return (target / "docs/.document-kit-managed.json").read_bytes()


def fixture_for(entry: dict, language: str, pack: Path = KIT) -> dict:
    return load_json(pack / entry["variants"][language]["resources"]["fixture"])


def updated_input(fixture: dict) -> tuple[str, str, dict[str, str]]:
    key = next(iter(fixture["sections"]))
    section = fixture["sections"][key]
    if fixture["language"] == "zh-CN":
        content = section["content"] + "\n\n本次修订仅补充边界说明，不改变模型身份或既有验收条件。"
        field = next(iter(section["fields"]))
        if field == "diagram_decisions":
            field = next((name for name in section["fields"] if name != "diagram_decisions"), field)
        fields = {field: section["fields"][field] + " 本字段在修订后继续保持原有语义。"}
    else:
        content = section["content"] + "\n\nThis revision clarifies the boundary without changing model identity or acceptance."
        field = next(iter(section["fields"]))
        if field == "diagram_decisions":
            field = next((name for name in section["fields"] if name != "diagram_decisions"), field)
        fields = {field: section["fields"][field] + " The field retains its original engineering meaning."}
    return key, content, fields


def error_code(case, code: str, operation, *args, **kwargs):
    with case.assertRaises(DocumentKitError) as rejected:
        operation(*args, **kwargs)
    case.assertEqual(rejected.exception.code, code, rejected.exception.as_dict())
    return rejected.exception


def temporary_directory():
    CACHE.mkdir(parents=True, exist_ok=True)
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    return tempfile.TemporaryDirectory(dir=CACHE)
