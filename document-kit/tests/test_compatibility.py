from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from _support import (
    KIT, REPO, changed_release, copy_pack, create_from_fixture, load_json,
    materialize_inputs, registry_path, sha256,
)
from xdesign_doc_kit.compatibility import DocumentKitError
from xdesign_doc_kit.core import attach_figure, check_documents, create_document, get_contract, plan_document, update_section

CACHE = REPO / "cache/document-kit-compatibility-tests"
V1 = KIT / "compatibility/v1.0.0"
V1_DIGEST = "1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6"
ZERO = "0" * 64


def codes(report: dict) -> set[str]:
    return {item["code"] for item in report["diagnostics"]}


class CompatibilityTests(unittest.TestCase):
    def setUp(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=CACHE)
        self.root = Path(self.temporary.name)
        self.fixture = load_json(KIT / "catalog/fixtures/generic-requirements.json")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def target(self, name: str = "target") -> Path:
        path = self.root / name
        path.mkdir(parents=True, exist_ok=True)
        materialize_inputs(path, self.fixture["models"] + self.fixture["figures"])
        return path

    def create_v1(self, target: Path, pack: Path = V1) -> dict:
        return create_document(target, "requirements", self.fixture["name"], self.fixture["metadata"],
                               self.fixture["sections"], profile_root=pack,
                               expected_contract_sha256=get_contract(pack)["contract_sha256"])

    def test_s13_v1_digest_projection_creation_update_and_figure(self) -> None:
        contract = get_contract(V1)
        self.assertEqual(contract["contract_sha256"], V1_DIGEST)
        self.assertEqual(contract["contract_digest_version"], "1")
        self.assertTrue(all(entry["template_id"] is None for entry in contract["catalog"]["entries"]))
        self.assertTrue(all(entry["scenario"] is None for entry in contract["catalog"]["entries"]))
        for relative in contract["profile"]["document_classes"].values():
            self.assertNotIn(ZERO, (V1 / relative["template"]).read_text())
        self.assertFalse(any((V1 / "catalog").exists() for _ in [0]))
        with self.assertRaises(DocumentKitError) as rejected:
            plan_document(self.root, "requirements", "sensor", profile_root=V1,
                          template_id="generic-requirements")
        self.assertEqual(rejected.exception.code, "unknown-template")
        target = self.target()
        created = self.create_v1(target)
        self.assertIsNone(created["template_id"])
        self.assertEqual(created["effective_identity"]["contract_digest_version"], "1")
        self.assertEqual(created["effective_identity"]["contract_sha256"], V1_DIGEST)
        self.assertTrue((target / created["path"]).read_text().startswith(
            "<!-- document-kit: profile=xdesign-sysml-docs@1.0.0 class=requirements -->\n"))
        registry = registry_path(target)
        entry = load_json(registry)["entries"][0]
        self.assertEqual(set(entry), {"path", "class", "profile_id", "version", "contract_sha256"})
        before = registry.read_bytes()
        updated = update_section(target, "requirements", self.fixture["name"], "purpose",
                                 "The retained v1 boundary is revised.", created["sha256"], profile_root=V1,
                                 expected_contract_sha256=V1_DIGEST)
        self.assertEqual(registry.read_bytes(), before)
        self.assertEqual(updated["effective_identity"]["contract_digest_version"], "1")
        asset = target / "docs/requirements/assets/legacy-context.svg"
        asset.parent.mkdir(parents=True, exist_ok=True)
        asset.write_text('<svg xmlns="http://www.w3.org/2000/svg"><title>Legacy</title></svg>\n')
        figure = {"asset": "assets/legacy-context.svg", "title": "Legacy context",
                  "Question": "What is retained?", "Model view": "Sensor::Context",
                  "Companions": "The requirements table.", "Status": "renderable-now; designed-only",
                  "Reading": "The fictional legacy context is visible.", "Limits": "No acceptance is claimed."}
        attached = attach_figure(target, "requirements", self.fixture["name"], "purpose", figure,
                                 updated["sha256"], profile_root=V1, expected_contract_sha256=V1_DIGEST)
        self.assertEqual(registry.read_bytes(), before)
        report = check_documents(target, profile_root=V1)
        self.assertTrue(report["ok"], report["diagnostics"])
        self.assertIsNone(report["effective_identity_by_path"][created["path"]]["template_id"])
        self.assertEqual(report["effective_identity_by_path"][created["path"]]["contract_digest_version"], "1")

    def test_doc13_bundled_v1_mixed_routing_precedes_selected_identity_mismatch(self) -> None:
        target = self.target()
        created = self.create_v1(target)
        registry = registry_path(target)
        before = registry.read_bytes()
        selected = get_contract(KIT)
        report = check_documents(target, profile_root=KIT)
        self.assertTrue(report["ok"], report["diagnostics"])
        self.assertEqual(report["contract_sha256"], selected["contract_sha256"])
        effective = report["effective_identity_by_path"][created["path"]]
        self.assertEqual(effective["contract_sha256"], V1_DIGEST)
        self.assertEqual(effective["contract_digest_version"], "1")
        updated = update_section(target, "requirements", self.fixture["name"], "purpose",
                                 "The bundled-v1 page remains under its original rules.", created["sha256"],
                                 profile_root=KIT, expected_contract_sha256=selected["contract_sha256"])
        self.assertEqual(updated["contract_sha256"], selected["contract_sha256"])
        self.assertEqual(updated["effective_identity"]["contract_sha256"], V1_DIGEST)
        self.assertEqual(registry.read_bytes(), before)
        asset = target / "docs/requirements/assets/mixed-legacy.svg"
        asset.parent.mkdir(parents=True, exist_ok=True)
        asset.write_text('<svg xmlns="http://www.w3.org/2000/svg"><title>Mixed legacy</title></svg>\n')
        figure = {"asset": "assets/mixed-legacy.svg", "title": "Mixed legacy",
                  "Question": "What remains pinned?", "Model view": "Sensor::Legacy",
                  "Companions": "The selected catalog.", "Status": "renderable-now; designed-only",
                  "Reading": "The legacy page keeps its identity.", "Limits": "No migration is performed."}
        attached = attach_figure(target, "requirements", self.fixture["name"], "purpose", figure,
                                 updated["sha256"], profile_root=KIT,
                                 expected_contract_sha256=selected["contract_sha256"])
        self.assertEqual(attached["effective_identity"]["contract_sha256"], V1_DIGEST)
        self.assertEqual(registry.read_bytes(), before)

    def test_doc13_custom_v1_original_pack_and_unknown_under_catalog(self) -> None:
        custom = copy_pack(self.root / "custom-v1") / "compatibility/v1.0.0"
        for path in custom.rglob("*"):
            if path.is_file() and path.suffix in {".json", ".md"}:
                path.write_bytes(path.read_bytes().replace(b"xdesign-sysml-docs", b"custom-docs"))
        contract = get_contract(custom)
        self.assertEqual(contract["profile_id"], "custom-docs")
        self.assertNotEqual(contract["contract_sha256"], V1_DIGEST)
        target = self.target("custom-target")
        created = self.create_v1(target, custom)
        self.assertEqual(created["profile_id"], "custom-docs")
        report = check_documents(target, profile_root=KIT)
        self.assertFalse(report["ok"])
        self.assertIn("unsupported-legacy-identity", codes(report))
        versioned = copy_pack(self.root / "custom-v1-versioned") / "compatibility/v1.0.0"
        for path in versioned.rglob("*"):
            if path.is_file() and path.suffix in {".json", ".md"}:
                data = path.read_bytes().replace(b"xdesign-sysml-docs", b"custom-v12")
                data = data.replace(b'"version": "1.0.0"', b'"version": "1.2.0"')
                path.write_bytes(data.replace(b"@1.0.0", b"@1.2.0"))
        versioned_target = self.target("custom-v12-target")
        created = self.create_v1(versioned_target, versioned)
        self.assertEqual(created["version"], "1.2.0")
        report = check_documents(versioned_target, profile_root=KIT)
        self.assertFalse(report["ok"])
        self.assertIn("unsupported-legacy-identity", codes(report))

    def test_doc13_original_11_release_and_different_release_refusal(self) -> None:
        target = self.target()
        created = create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        registry = registry_path(target)
        before = registry.read_bytes()
        release = changed_release(self.root / "release-1.1.1")
        report = check_documents(target, profile_root=release)
        self.assertFalse(report["ok"])
        self.assertIn("registry-mismatch", codes(report))
        updated = update_section(target, "requirements", self.fixture["name"], "purpose",
                                 "The original 1.1.0 pack remains explicitly selectable.", created["sha256"],
                                 profile_root=KIT, expected_contract_sha256=get_contract(KIT)["contract_sha256"])
        self.assertTrue(updated["ok"])
        self.assertEqual(registry.read_bytes(), before)

    def test_doc13_unmanaged_pages_are_not_adopted_or_rewritten(self) -> None:
        target = self.target()
        page = target / "docs/design/unmanaged.md"
        page.parent.mkdir(parents=True, exist_ok=True)
        page.write_text("# Unmanaged design\n\nThe existing page is outside the kit registry.\n")
        before = page.read_bytes()
        self.assertTrue(check_documents(target, profile_root=KIT)["ok"])
        with self.assertRaises(DocumentKitError) as rejected:
            update_section(target, "design", "unmanaged", "purpose", "A refused write.", "0" * 64,
                           profile_root=KIT)
        self.assertEqual(rejected.exception.code, "registry-mismatch")
        self.assertEqual(page.read_bytes(), before)
        self.assertFalse(registry_path(target).exists())

    def test_s07_target_sentinel_copied_example_and_forged_registry(self) -> None:
        target = self.target()
        created = create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        document = target / created["path"]
        registry = registry_path(target)
        original_document = document.read_bytes()
        original_registry = registry.read_bytes()
        document.write_bytes(original_document.replace(b"contract=" + created["contract_sha256"].encode(),
                                                       b"contract=" + ZERO.encode(), 1))
        report = check_documents(target, profile_root=KIT)
        self.assertIn("registry-mismatch", codes(report))
        self.assertEqual(registry.read_bytes(), original_registry)
        document.write_bytes(original_document)
        value = load_json(registry)
        value["entries"][0]["contract_sha256"] = ZERO
        registry.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
        report = check_documents(target, profile_root=KIT)
        self.assertIn("registry-mismatch", codes(report))
        self.assertEqual(document.read_bytes(), original_document)
        registry.write_bytes(original_registry)
        copied = target / "docs/requirements/copied-example.md"
        copied.write_bytes((KIT / self.fixture["expected_document"]).read_bytes())
        before = registry.read_bytes()
        report = check_documents(target, profile_root=KIT)
        self.assertIn("registry-mismatch", codes(report))
        self.assertEqual(registry.read_bytes(), before)
        value = load_json(registry)
        forged = dict(value["entries"][0])
        forged["path"] = "docs/requirements/copied-example.md"
        forged["contract_sha256"] = ZERO
        value["entries"].append(forged)
        registry.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
        report = check_documents(target, profile_root=KIT)
        self.assertIn("registry-mismatch", codes(report))
        self.assertNotIn("docs/requirements/copied-example.md", report["effective_identity_by_path"])
    def test_doc10_each_own_identity_mismatch_fails_without_rewrite(self) -> None:
        target = self.target()
        created = create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        document = target / created["path"]
        registry = registry_path(target)
        original_document = document.read_bytes()
        original_registry = registry.read_bytes()
        for field in ("class", "template_id", "profile_id", "version", "contract_sha256", "contract_digest_version"):
            with self.subTest(registry_field=field):
                value = load_json(registry)
                value["entries"][0][field] = {"contract_sha256": "f" * 64, "contract_digest_version": "1"}.get(field, "wrong-identity")
                registry.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
                report = check_documents(target, profile_root=KIT)
                self.assertIn("registry-mismatch", codes(report))
                self.assertEqual(document.read_bytes(), original_document)
                registry.write_bytes(original_registry)
        marker = original_document.decode()
        replacements = {
            "class": ("class=requirements", "class=design"),
            "template": ("template=generic-requirements", "template=generic-design"),
            "profile": ("profile=xdesign-sysml-docs", "profile=other-docs"),
            "version": ("@1.1.0", "@1.1.1"),
            "digest": (created["contract_sha256"], "f" * 64),
            "digest-version": ("digest-version=2", "digest-version=1"),
        }
        for field, (old, new) in replacements.items():
            with self.subTest(marker_field=field):
                document.write_text(marker.replace(old, new, 1))
                report = check_documents(target, profile_root=KIT)
                self.assertIn("registry-mismatch", codes(report))
                self.assertEqual(registry.read_bytes(), original_registry)
                document.write_bytes(original_document)

    def test_doc10_changed_default_does_not_reselect_original_document(self) -> None:
        fixture = load_json(KIT / "catalog/fixtures/generic-design.json")
        target = self.target()
        created = create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        before = registry_path(target).read_bytes()
        derived = changed_release(self.root / "changed-default")
        manifest_path = derived / "catalog/manifest.json"
        manifest = load_json(manifest_path)
        alternate = copy.deepcopy(next(entry for entry in manifest["entries"] if entry["template_id"] == "generic-design"))
        alternate["template_id"] = "generic-alternate-design"
        for key in ("template", "guide", "example", "fixture"):
            source = derived / alternate[key]
            destination = derived / alternate[key].replace("generic-design", "generic-alternate-design")
            destination.write_bytes(source.read_bytes())
            alternate[key] = destination.relative_to(derived).as_posix()
        example = derived / alternate["example"]
        example.write_bytes(example.read_bytes().replace(b"template=generic-design", b"template=generic-alternate-design", 1))
        fixture_path = derived / alternate["fixture"]
        alternate_fixture = load_json(fixture_path)
        alternate_fixture["template_id"] = "generic-alternate-design"
        alternate_fixture["expected_document"] = alternate["example"]
        fixture_path.write_text(json.dumps(alternate_fixture, indent=2) + "\n")
        manifest["entries"].append(alternate)
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
        profile_path = derived / "profile.json"
        profile = load_json(profile_path)
        profile["document_classes"]["design"]["default_template_id"] = "generic-alternate-design"
        profile["document_classes"]["design"]["template"] = alternate["template"]
        profile_path.write_text(json.dumps(profile, indent=2) + "\n")
        plan = plan_document(target, "design", "new-default", profile_root=derived)
        self.assertEqual(plan["template_id"], "generic-alternate-design")
        updated = update_section(target, "design", fixture["name"], "purpose",
                                 "The original generic template remains effective.", created["sha256"],
                                 profile_root=KIT, expected_contract_sha256=get_contract(KIT)["contract_sha256"])
        self.assertEqual(updated["effective_identity"]["template_id"], "generic-design")
        self.assertEqual(registry_path(target).read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
