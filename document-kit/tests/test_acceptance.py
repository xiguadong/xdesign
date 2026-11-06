"""Independent DOC-01 through DOC-05 acceptance cases.

Fixtures live in cache and represent a separate engineering project.
"""

from __future__ import annotations

import os

import hashlib
import errno
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

KIT = Path(__file__).resolve().parents[1]
REPO = KIT.parent
CACHE = REPO / "cache" / "document-kit-acceptance"
CACHE = Path(os.environ.get("XDESIGN_DOC_KIT_TEST_CACHE", str(CACHE)))
sys.path.insert(0, str(KIT))

from xdesign_doc_kit.core import (  # noqa: E402
    DocumentKitError,
    attach_figure,
    check_documents,
    create_document,
    get_contract,
    plan_document,
    update_section,
)


def payload(document_class="requirements", start=100):
    spec = get_contract()["profile"]["document_classes"][document_class]
    prefix = {"requirements": "R", "architecture": "B", "design": "D",
              "interface": "I", "verification": "V"}[document_class]
    metadata = {"title": "Sensor " + document_class, "status": "draft",
                "revision": "1", "model_sources": ["mbse/model/sensor.sysml"]}
    sections = {
        item["key"]: {
            "anchor": f"{prefix}{start + index}",
            "content": f"The sensor project's {item['key']} is explained here.",
            "fields": {field: f"Sensor {field.replace('_', ' ')} evidence is described here."
                       for field in item["required_fields"]},
        }
        for index, item in enumerate(spec["required_sections"])
    }
    return metadata, sections


def codes(report):
    return {item["code"] for item in report.get("diagnostics", [])}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class AcceptanceTests(unittest.TestCase):
    def setUp(self):
        CACHE.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="consumer-", dir=CACHE)
        self.project = Path(self.temp.name)
        model = self.project / "mbse/model/sensor.sysml"
        model.parent.mkdir(parents=True)
        model.write_text("package Sensor {}\n", encoding="utf-8")

    def tearDown(self):
        self.temp.cleanup()

    def create_requirements(self):
        metadata, sections = payload()
        created = create_document(
            self.project, "requirements", "sensor-interface", metadata, sections
        )
        document = self.project / created["path"]
        self.assertEqual(created["sha256"], digest(document))
        return document, created

    def test_doc01_all_class_routes_and_contract_digest_drift(self):
        contract = get_contract()
        original_digest = contract["contract_sha256"]
        self.assertEqual(len(original_digest), 64)
        for document_class, spec in contract["profile"]["document_classes"].items():
            with self.subTest(document_class=document_class):
                layer = "core" if spec.get("selector_kind") == "api" else None
                plan = plan_document(self.project, document_class, "sensor-system", layer=layer)
                self.assertEqual(plan["path"], spec["path_template"].replace("{name}", "sensor-system").replace("{layer}", layer or ""))
                self.assertEqual(plan["required_sections"], spec["required_sections"])
                self.assertEqual(plan["contract_sha256"], original_digest)
                self.assertFalse(plan["exists"])
                self.assertFalse((self.project / plan["path"]).exists())
        profile_copy = self.project / "cache/profile-copy"
        shutil.copytree(KIT, profile_copy)
        self.assertEqual(get_contract(profile_copy)["contract_sha256"], original_digest)
        changed = profile_copy / "catalog/templates/generic-requirements.md"
        changed.write_bytes(changed.read_bytes() + b"\n")
        self.assertNotEqual(get_contract(profile_copy)["contract_sha256"], original_digest)
        unsafe_profile = get_contract(profile_copy)["profile"]
        unsafe_profile["rules_file"] = "../outside.md"
        (profile_copy / "profile.json").write_text(
            json.dumps(unsafe_profile), encoding="utf-8"
        )
        with self.assertRaises(DocumentKitError):
            get_contract(profile_copy)

    def test_doc02_create_registry_overwrite_and_second_project(self):
        document, created = self.create_requirements()
        content = document.read_text(encoding="utf-8")
        self.assertIn('<a id="R100"></a>', content)
        headings = [line[3:] for line in content.splitlines() if line.startswith("## ")]
        expected = [part["heading"] for part in
                    get_contract()["profile"]["document_classes"]["requirements"]["required_sections"]]
        self.assertEqual(headings, expected)
        registry = self.project / "docs/.document-kit-managed.json"
        self.assertIn("docs/requirements/sensor-interface.md", registry.read_text(encoding="utf-8"))
        self.assertTrue(check_documents(self.project)["ok"])
        original = document.read_bytes()
        metadata, sections = payload()
        with self.assertRaises(DocumentKitError):
            create_document(self.project, "requirements", "sensor-interface", metadata, sections)
        self.assertEqual(document.read_bytes(), original)
        with tempfile.TemporaryDirectory(prefix="consumer-second-", dir=CACHE) as directory:
            second = Path(directory)
            model = second / "mbse/model/sensor.sysml"
            model.parent.mkdir(parents=True)
            model.write_text("package OtherSensor {}\n", encoding="utf-8")
            sections["purpose"]["content"] = "The second project has a different boundary."
            other = create_document(second, "requirements", "sensor-interface", metadata, sections)
            self.assertTrue(check_documents(second)["ok"])
            self.assertNotEqual((second / other["path"]).read_bytes(), original)
            self.assertEqual(document.read_bytes(), original)
            self.assertEqual(created["contract_sha256"], other["contract_sha256"])

    def test_doc03_template_and_locator_failures(self):
        document, _ = self.create_requirements()
        original = document.read_text(encoding="utf-8")
        changes = (
            (original.replace("## 2. Requirements", "## 2. Different requirements"), "section-headings"),
            (original.replace("## 2. Requirements", ""), "section-headings"),
            (original.replace("## 3. Acceptance conditions", "## 2. Requirements"), "section-headings"),
            (original.replace("**System boundary:** Sensor system boundary evidence is described here.",
                              "**System boundary:** "), "missing-field"),
            (original.replace("The sensor project's purpose is explained here.", ""), "empty-content"),
            (original.replace("The sensor project's purpose is explained here.", "TODO"), "unresolved-placeholder"),
            (original.replace("The sensor project's purpose is explained here.",
                              "[missing](missing.md)"), "invalid-link"),
            (original.replace('<a id="R100"></a>', '<a id="R101"></a>'), "duplicate-anchor"),
            (original.split("\n", 1)[1], "registry-mismatch"),
        )
        for changed, expected in changes:
            with self.subTest(expected=expected):
                document.write_text(changed, encoding="utf-8")
                report = check_documents(self.project)
                self.assertFalse(report["ok"])
                self.assertIn(expected, codes(report), report["diagnostics"])
        document.write_text(original, encoding="utf-8")
        registry = self.project / "docs/.document-kit-managed.json"
        registry_bytes = registry.read_bytes()
        registry.unlink()
        missing_registry = check_documents(self.project)
        self.assertFalse(missing_registry["ok"])
        self.assertIn("registry-mismatch", codes(missing_registry))
        registry.write_bytes(registry_bytes)
        metadata, duplicate_sections = payload()
        with self.assertRaises(DocumentKitError):
            create_document(self.project, "requirements", "duplicate-interface",
                            metadata, duplicate_sections)
        self.assertFalse((self.project / "docs/requirements/duplicate-interface.md").exists())
        tick = chr(96)
        source_token = f"{tick}mbse/model/sensor.sysml{tick}"
        for bad_source in ("mbse/model/missing.sysml", "../outside.sysml"):
            with self.subTest(bad_source=bad_source):
                corrupted = original.replace(source_token, f"{tick}{bad_source}{tick}")
                self.assertNotEqual(corrupted, original)
                document.write_text(corrupted, encoding="utf-8")
                report = check_documents(self.project)
                self.assertFalse(report["ok"])
                self.assertTrue(
                    {"missing-model-source", "unsafe-path"}.intersection(codes(report)),
                    report["diagnostics"],
                )
        document.write_text(original, encoding="utf-8")
        model = self.project / "mbse/model/sensor.sysml"
        model.write_text(
            'package Sensor {\n'
            ' @XDesignCommon::DocSection { document = "docs/requirements/sensor-interface.md"; section = "R999"; }\n'
            ' @XDesignCommon::DocFigure { document = "docs/requirements/sensor-interface.md"; figure = "docs/requirements/assets/missing.svg"; }\n'
            '}\n', encoding="utf-8"
        )
        broken = check_documents(self.project)
        self.assertFalse(broken["ok"])
        self.assertTrue({"invalid-docsection", "invalid-docfigure"}.issubset(codes(broken)),
                        broken["diagnostics"])
        model.write_text('package Sensor { @XDesignCommon::DocSection ("unsupported") }\n',
                         encoding="utf-8")
        unsupported = check_documents(self.project)
        self.assertFalse(unsupported["ok"])
        self.assertIn("unsupported-annotation", codes(unsupported))
        for diagnostic in unsupported["diagnostics"]:
            self.assertEqual(set(diagnostic), {"rule", "code", "file", "line", "detail"})

    def test_doc04_unsafe_inputs_stale_hash_and_preserved_bytes(self):
        metadata, sections = payload()
        for name in ("../escape", "/absolute", "UPPER_CASE", "a/b"):
            with self.subTest(name=name), self.assertRaises(DocumentKitError):
                plan_document(self.project, "requirements", name)
        bad_sources = (
            ["../escape.sysml"], ["mbse/model/missing.sysml"],
            ["mbse/model/sensor.sysml", "mbse/model/sensor.sysml"],
            ["mbse/model/" + chr(96) + "sensor" + chr(96) + ".sysml"],
        )
        for source in bad_sources:
            with self.subTest(source=source), self.assertRaises(DocumentKitError):
                create_document(self.project, "requirements", "bad-source",
                                {**metadata, "model_sources": source}, sections)
        with tempfile.TemporaryDirectory(prefix="no-model-", dir=CACHE) as directory:
            empty_project = Path(directory)
            self.assertTrue(plan_document(empty_project, "requirements", "sensor-system")["ok"])
            with self.assertRaises(DocumentKitError):
                create_document(empty_project, "requirements", "sensor-system", metadata, sections)
            self.assertFalse((empty_project / "docs/requirements/sensor-system.md").exists())
        with tempfile.TemporaryDirectory(prefix="external-", dir=CACHE) as directory:
            outside = Path(directory)
            (self.project / "docs").mkdir(exist_ok=True)
            link = self.project / "docs/architecture"
            link.symlink_to(outside, target_is_directory=True)
            with self.assertRaises(DocumentKitError):
                plan_document(self.project, "architecture", "sensor-system")
            self.assertFalse((outside / "sensor-system.md").exists())
            link.unlink()
        document, created = self.create_requirements()
        original = document.read_bytes()
        with self.assertRaises(DocumentKitError):
            update_section(self.project, "requirements", "sensor-interface", "purpose",
                           "A changed boundary.", "0" * 64)
        self.assertEqual(document.read_bytes(), original)
        with self.assertRaises(DocumentKitError):
            update_section(self.project, "requirements", "sensor-interface", "purpose",
                           "TODO", created["sha256"])
        self.assertEqual(document.read_bytes(), original)
        self.assertFalse(list(document.parent.glob("*.tmp")))
        self.assertFalse(list(document.parent.glob(".*.tmp")))

    def test_doc04_cross_filesystem_replace_failure_keeps_page_and_registry(self):
        document, created = self.create_requirements()
        registry = self.project / "docs/.document-kit-managed.json"
        before_document = document.read_bytes()
        before_registry = registry.read_bytes()
        with patch("xdesign_doc_kit.core.os.replace",
                   side_effect=OSError(errno.EXDEV, "cross-device link")):
            with self.assertRaises(DocumentKitError) as rejected:
                update_section(
                    self.project, "requirements", "sensor-interface", "purpose",
                    "The revised boundary is described here.", created["sha256"],
                )
        self.assertEqual(rejected.exception.code, "atomic-replace-failed")
        self.assertEqual(document.read_bytes(), before_document)
        self.assertEqual(registry.read_bytes(), before_registry)
        self.assertFalse(list(document.parent.glob("*.tmp")))
        self.assertFalse(list(document.parent.glob(".*.tmp")))

    def test_doc05_figure_block_and_asset_failures(self):
        document, created = self.create_requirements()
        figure = {
            "asset": "assets/sensor-context.svg", "title": "Sensor context",
            "Question": "Where is the sensor boundary?", "Model view": "Sensor::context",
            "Companions": "The interface table.", "Status": "renderable-now; designed-only",
            "Reading": "The sensor is inside its boundary. The bus connects it to the controller.",
            "Limits": "Electrical values are omitted.",
        }
        original = document.read_bytes()
        with self.assertRaises(DocumentKitError):
            attach_figure(self.project, "requirements", "sensor-interface", "purpose",
                          figure, created["sha256"])
        self.assertEqual(document.read_bytes(), original)
        asset = document.parent / "assets/sensor-context.svg"
        asset.parent.mkdir(parents=True)
        asset.write_text(
            '<svg xmlns="http://www.w3.org/2000/svg"><title>Sensor context</title>'
            '<rect x="0" y="0" width="10" height="10"/></svg>\n', encoding="utf-8"
        )
        attached = attach_figure(self.project, "requirements", "sensor-interface", "purpose",
                                 figure, created["sha256"])
        self.assertEqual(attached["sha256"], digest(document))
        self.assertIn("Figure R100-1", document.read_text(encoding="utf-8"))
        self.assertTrue(check_documents(self.project)["ok"])
        with self.assertRaises(DocumentKitError):
            attach_figure(self.project, "requirements", "sensor-interface", "purpose",
                          figure, attached["sha256"])
        with self.assertRaises(DocumentKitError):
            attach_figure(self.project, "requirements", "sensor-interface", "wrong",
                          figure, attached["sha256"])
        pristine = document.read_text(encoding="utf-8")
        document.write_text(pristine.replace("**Limits:** Electrical values are omitted.",
                                             "**Limits:** "), encoding="utf-8")
        invalid = check_documents(self.project)
        self.assertFalse(invalid["ok"])
        self.assertIn("figure-block", codes(invalid))
        document.write_text(pristine, encoding="utf-8")
        document.write_text(pristine.replace("renderable-now; designed-only",
                                             "unknown-status"), encoding="utf-8")
        invalid_status = check_documents(self.project)
        self.assertFalse(invalid_status["ok"])
        self.assertIn("figure-status", codes(invalid_status))
        document.write_text(pristine, encoding="utf-8")

        lines = pristine.splitlines(keepends=True)
        question = next(index for index, line in enumerate(lines) if line.startswith("> **Question:**"))
        model_view = next(index for index, line in enumerate(lines) if line.startswith("> **Model view:**"))
        lines[question], lines[model_view] = lines[model_view], lines[question]
        document.write_text("".join(lines), encoding="utf-8")
        reordered = check_documents(self.project)
        self.assertFalse(reordered["ok"])
        self.assertIn("figure-block", codes(reordered))
        document.write_text(pristine, encoding="utf-8")

        second_asset = document.parent / "assets/secondary-context.svg"
        second_asset.write_text(
            '<svg xmlns="http://www.w3.org/2000/svg"><title>Secondary context</title></svg>\n',
            encoding="utf-8",
        )
        start = pristine.index("![Sensor context]")
        end = pristine.find("\n<a id=", start)
        self.assertGreater(end, start)
        duplicate_block = pristine[start:end].replace(
            "sensor-context.svg", "secondary-context.svg"
        ).replace("Sensor context", "Secondary context")
        document.write_text(pristine[:end] + "\n" + duplicate_block + pristine[end:],
                            encoding="utf-8")
        duplicate = check_documents(self.project)
        self.assertFalse(duplicate["ok"])
        self.assertIn("duplicate-figure", codes(duplicate))
        document.write_text(pristine, encoding="utf-8")

        asset.unlink()
        broken = check_documents(self.project)
        self.assertFalse(broken["ok"])
        self.assertIn("missing-figure", codes(broken))


if __name__ == "__main__":
    unittest.main()
