from __future__ import annotations

import hashlib
import json
import shutil
import sys
from pathlib import Path

DISTRIBUTION = Path(__file__).resolve().parents[1]
KIT = DISTRIBUTION / "compatibility/v1.1.0"
REPO = DISTRIBUTION.parent
sys.path.insert(0, str(DISTRIBUTION))

from xdesign_doc_kit.core import create_document, get_contract


def copy_pack(destination: Path) -> Path:
    shutil.copytree(KIT, destination, dirs_exist_ok=True)
    return destination


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def materialize_inputs(target: Path, items: list[dict], pack: Path = KIT) -> None:
    for item in items:
        destination = target / item["target"]
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes((pack / item["pack"]).read_bytes())


def create_from_fixture(target: Path, fixture: dict, pack: Path = KIT,
                        expected_digest: str | None = None) -> dict:
    target.mkdir(parents=True, exist_ok=True)
    materialize_inputs(target, fixture.get("models", []) + fixture.get("figures", []), pack)
    return create_document(
        target,
        fixture["document_class"],
        fixture["name"],
        fixture["metadata"],
        fixture["sections"],
        profile_root=pack,
        expected_contract_sha256=expected_digest,
        template_id=fixture["template_id"],
    )


def changed_release(destination: Path, version: str = "1.1.1") -> Path:
    copy_pack(destination)
    for path in destination.rglob("*"):
        if not path.is_file() or "compatibility" in path.parts:
            continue
        if path.suffix not in {".json", ".md"}:
            continue
        data = path.read_bytes()
        data = data.replace(b'"version": "1.1.0"', f'"version": "{version}"'.encode("ascii"))
        data = data.replace(b"xdesign-sysml-docs@1.1.0", f"xdesign-sysml-docs@{version}".encode("ascii"))
        path.write_bytes(data)
    (destination / "README.md").write_bytes((destination / "README.md").read_bytes() + b"\nReviewed release fixture.\n")
    return destination


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def registry_path(target: Path) -> Path:
    return target / "docs/.document-kit-managed.json"


def selected_digest(pack: Path = KIT) -> str:
    return get_contract(pack)["contract_sha256"]
