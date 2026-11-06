"""Focused regression tests for the shared document engine."""

from __future__ import annotations

import os

import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

KIT = Path(__file__).resolve().parents[1]
REPO = KIT.parent
CACHE = REPO / "cache" / "document-kit-core-tests"
CACHE = Path(os.environ.get("XDESIGN_DOC_KIT_TEST_CACHE", str(CACHE)))
sys.path.insert(0, str(KIT))

from xdesign_doc_kit.core import (  # noqa: E402
    DocumentKitError, attach_figure, check_documents, create_document,
    get_contract, plan_document, update_section,
)


def payload() -> tuple[dict, dict]:
    spec = get_contract()["profile"]["document_classes"]["design"]
    metadata = {"title": "Sensor design", "status": "draft", "revision": "1",
                "model_sources": ["mbse/model/sensor.sysml"]}
    sections = {
        item["key"]: {"anchor": f"D{100 + index}",
                      "content": f"The sensor {item['key']} is specified here.",
                      "fields": {field: f"The {field} is supplied by the engineer."
                                 for field in item["required_fields"]}}
        for index, item in enumerate(spec["required_sections"])
    }
    return metadata, sections


class CoreTests(unittest.TestCase):
    def setUp(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="consumer-", dir=CACHE)
        self.target = Path(self.temp.name)
        model = self.target / "mbse/model/sensor.sysml"
        model.parent.mkdir(parents=True)
        model.write_text("package Sensor {}\n", encoding="utf-8")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_update_preserves_attached_figure_and_other_sections(self) -> None:
        metadata, sections = payload()
        sections["purpose"]["content"] = (
            "The purpose is specified here.\n\n<a id=\"D200\"></a>\n\n### Detailed boundary\n"
            "The sensor boundary is detailed here."
        )
        created = create_document(self.target, "design", "sensor", metadata, sections)
        path = self.target / created["path"]
        asset = path.parent / "assets/sensor-context.svg"
        asset.parent.mkdir()
        asset.write_text('<svg xmlns="http://www.w3.org/2000/svg"/>\n', encoding="utf-8")
        figure = {"asset": "assets/sensor-context.svg", "title": "Sensor context",
                  "Question": "What is the sensor boundary?", "Model view": "Sensor::context",
                  "Companions": "The interface table.", "Status": "renderable-now; designed-only",
                  "Reading": "The sensor is inside its boundary.", "Limits": "Power is omitted."}
        attached = attach_figure(self.target, "design", "sensor", "purpose", figure,
                                 created["sha256"])
        updated = update_section(self.target, "design", "sensor", "purpose",
                                 "The revised boundary is explicit.", attached["sha256"])
        text = path.read_text(encoding="utf-8")
        self.assertIn("Figure D100-1", text)
        self.assertIn("The revised boundary is explicit.", text)
        self.assertIn("The sensor structure is specified here.", text)
        self.assertEqual(updated["sha256"], hashlib.sha256(path.read_bytes()).hexdigest())
        self.assertTrue(check_documents(self.target)["ok"])

    def test_interrupted_registry_update_is_reported(self) -> None:
        metadata, sections = payload()
        created = create_document(self.target, "design", "sensor", metadata, sections)
        registry = self.target / "docs/.document-kit-managed.json"
        registry.unlink()
        report = check_documents(self.target)
        self.assertFalse(report["ok"])
        self.assertIn("registry-mismatch", {item["code"] for item in report["diagnostics"]})
        with self.assertRaises(DocumentKitError) as caught:
            create_document(self.target, "design", "sensor", metadata, sections)
        self.assertEqual(caught.exception.code, "document-exists")
        self.assertTrue((self.target / created["path"]).exists())

    def test_invalid_contract_bytes_fail_with_structured_error(self) -> None:
        profile_copy = self.target / "cache/profile-copy"
        shutil.copytree(KIT, profile_copy, dirs_exist_ok=True)
        profile = json.loads((KIT / "profile.json").read_text(encoding="utf-8"))
        profile["document_classes"]["design"]["required_sections"][0]["key"] = []
        (profile_copy / "profile.json").write_text(json.dumps(profile), encoding="utf-8")
        with self.assertRaises(DocumentKitError) as caught:
            get_contract(profile_copy)
        self.assertEqual(caught.exception.code, "invalid-profile")
        self.assertEqual(set(caught.exception.diagnostics[0]),
                         {"rule", "code", "file", "line", "detail"})

    def test_missing_local_heading_fragment_is_reported(self) -> None:
        metadata, sections = payload()
        created = create_document(self.target, "design", "sensor", metadata, sections)
        path = self.target / created["path"]
        original = path.read_text(encoding="utf-8")
        path.write_text(original.replace("The sensor purpose is specified here.",
                                         "[missing](#never-written)"), encoding="utf-8")
        report = check_documents(self.target)
        self.assertFalse(report["ok"])
        self.assertIn("invalid-link", {item["code"] for item in report["diagnostics"]})

    def test_candidate_self_links_resolve_before_file_exists(self) -> None:
        metadata, sections = payload()
        sections["purpose"]["content"] = (
            "See [the purpose](#D100) and [its owner](sensor.md#D100)."
        )
        created = create_document(self.target, "design", "sensor", metadata, sections)
        self.assertTrue((self.target / created["path"]).is_file())
        self.assertTrue(check_documents(self.target)["ok"])

    def test_class_filter_hashes_only_fully_checked_pages(self) -> None:
        metadata, sections = payload()
        created = create_document(self.target, "design", "sensor", metadata, sections)
        legacy = self.target / "docs/requirements/legacy.md"
        legacy.parent.mkdir(parents=True)
        legacy.write_text("# Legacy requirements\n", encoding="utf-8")
        report = check_documents(self.target, "design")
        self.assertTrue(report["ok"])
        self.assertEqual(report["checked_paths"], [created["path"]])
        self.assertEqual(set(report["document_hashes"]), {created["path"]})
        self.assertIn("docs/requirements/legacy.md", report["scanned_paths"])
        self.assertEqual(report["scope"]["document_class"], "design")
        model = self.target / "mbse/model/sensor.sysml"
        model.write_text(
            '@XDesignCommon::DocSection { document = "docs/requirements/legacy.md"; section = "R999"; }\n',
            encoding="utf-8",
        )
        with_global = check_documents(self.target, "design")
        self.assertFalse(with_global["ok"])
        self.assertEqual(with_global["checked_paths"], [created["path"]])
        self.assertIn("invalid-docsection", {item["code"] for item in with_global["global_findings"]})

    def test_derived_profile_cannot_relax_name_or_repeat_fields(self) -> None:
        profile_copy = self.target / "cache/profile-copy"
        shutil.copytree(KIT, profile_copy, dirs_exist_ok=True)
        profile = json.loads((KIT / "profile.json").read_text(encoding="utf-8"))
        profile["document_classes"]["design"]["filename_pattern"] = ".*"
        (profile_copy / "profile.json").write_text(json.dumps(profile), encoding="utf-8")
        with self.assertRaises(DocumentKitError) as caught:
            plan_document(self.target, "design", "MixedCase", profile_copy)
        self.assertEqual(caught.exception.code, "invalid-document-name")
        with self.assertRaises(DocumentKitError):
            plan_document(self.target, "design", "nested/name", profile_copy)
        profile["document_classes"]["design"]["required_sections"][1]["required_fields"][0] = "design_subject"
        (profile_copy / "profile.json").write_text(json.dumps(profile), encoding="utf-8")
        with self.assertRaises(DocumentKitError) as caught:
            get_contract(profile_copy)
        self.assertEqual(caught.exception.code, "invalid-profile")

    def test_managed_anchor_obeys_profile_case(self) -> None:
        metadata, sections = payload()
        created = create_document(self.target, "design", "sensor", metadata, sections)
        path = self.target / created["path"]
        path.write_text(path.read_text(encoding="utf-8").replace(
            '<a id="D100"></a>', '<a id="d100"></a>'), encoding="utf-8")
        report = check_documents(self.target)
        self.assertFalse(report["ok"])
        self.assertIn("missing-anchor", {item["code"] for item in report["diagnostics"]})


if __name__ == "__main__":
    unittest.main()
