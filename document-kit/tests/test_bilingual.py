from __future__ import annotations

import copy
import json
import shutil
import unittest
from pathlib import Path

from _bilingual_support import (
    CACHE, EVIDENCE, KIT, ZERO, assert_created_identity, compare_created_bytes,
    create_from_fixture, error_code, expected_bytes, fixture_for, load_json,
    materialize, registry_bytes, selection, sha_bytes, sha_path, snapshot,
    temporary_directory, updated_input, write_json,
)
from xdesign_doc_kit import (
    DocumentKitError, attach_figure, check_documents, create_document, get_contract,
    plan_document, update_section,
)


class BilingualContractTests(unittest.TestCase):
    def assert_current_resource_inventory(self, pack, contract):
        declared = {item["path"] for item in contract["resource_index"] + contract["shared_resource_index"]}
        # The manifest is declared root metadata, rather than a served resource.
        metadata = {contract["profile"]["catalog"]["manifest"]}
        self.assertFalse(declared & metadata)
        discovered = {
            path.relative_to(pack).as_posix()
            for root in (pack / "catalog", pack / "templates")
            for path in root.rglob("*") if path.is_file()
        }
        self.assertEqual(discovered, declared | metadata)

    @classmethod
    def setUpClass(cls):
        CACHE.mkdir(parents=True, exist_ok=True)
        EVIDENCE.mkdir(parents=True, exist_ok=True)
        cls.contract = get_contract(KIT)
        cls.digest = cls.contract["contract_sha256"]
        cls.entries = cls.contract["catalog"]["entries"]

    def test_bdk01_inventory_hashes_and_declared_discovery(self):
        self.assertEqual(self.contract["profile"]["schema_version"], "1.3")
        self.assertEqual(self.contract["profile"]["version"], "1.3.0")
        self.assertEqual(self.contract["contract_digest_version"], "3")
        self.assertNotEqual(self.digest, ZERO)
        self.assertEqual(self.contract["language_policy"], {"supported_ids": ["en", "zh-CN"], "default": "en"})
        self.assertEqual(len(self.entries), 21)
        self.assertEqual(len(self.contract["resource_index"]), 168)
        self.assertEqual(len(self.contract["shared_resource_index"]), 9)
        self.assertEqual(len(self.contract["resource_bytes"]), 177)
        declared = set()
        combinations = set()
        for entry in self.entries:
            self.assertEqual(set(entry), {"template_id", "document_class", "sources", "checks", "variants"})
            self.assertEqual(set(entry["variants"]), {"en", "zh-CN"})
            en = entry["variants"]["en"]
            zh = entry["variants"]["zh-CN"]
            self.assertEqual(
                [(s["key"], s["required_fields"]) for s in en["required_sections"]],
                [(s["key"], s["required_fields"]) for s in zh["required_sections"]],
            )
            for language in ("en", "zh-CN"):
                variant = entry["variants"][language]
                for kind, path in variant["resources"].items():
                    declared.add(path)
                    combinations.add((entry["template_id"], language, kind))
                    data = (KIT / path).read_bytes()
                    index = next(item for item in self.contract["resource_index"] if item["path"] == path)
                    self.assertEqual(index["sha256"], sha_bytes(data))
                    self.assertEqual(self.contract["resource_bytes"][path].encode("utf-8"), data)
                    self.assertEqual(index["template_id"], entry["template_id"])
                    self.assertEqual(index["language"], language)
                    self.assertEqual(index["kind"], kind)
                    self.assertEqual(set(index), {"template_id", "document_class", "language", "kind", "path", "sha256", "mime_type"})
            self.assertEqual(en["resources"]["template"], f"catalog/entries/{entry['template_id']}/en/template.md")
            self.assertEqual(zh["resources"]["template"], f"catalog/entries/{entry['template_id']}/zh-CN/template.md")
            self.assertEqual(self.contract["templates"][entry["template_id"]], (KIT / en["resources"]["template"]).read_text(encoding="utf-8"))
        discovered = {
            path.relative_to(KIT).as_posix()
            for path in (KIT / "catalog/entries").rglob("*") if path.is_file()
        }
        self.assertEqual(discovered, declared)
        self.assert_current_resource_inventory(KIT, self.contract)
        self.assertEqual(len(combinations), 168)
        for legacy in ("templates", "guides", "examples", "fixtures"):
            self.assertFalse((KIT / "catalog" / legacy).exists())
        for item in self.contract["shared_resource_index"]:
            self.assertEqual(set(item), {"path", "sha256", "mime_type"})
            self.assertEqual(item["sha256"], sha_path(KIT / item["path"]))
            self.assertNotIn("compatibility/", item["path"])
        self.assertEqual(self.contract["templates"]["figure-block"], self.contract["figure_templates"]["en"])
        self.assertIn("**Figure ", self.contract["figure_templates"]["en"])
        self.assertIn("**图 ", self.contract["figure_templates"]["zh-CN"])

    def test_bdk01_inverse_inventory_rejects_unindexed_current_resources(self):
        records = []
        for orphan in ("catalog/routing.md", "catalog/sources.json",
                       "catalog/shared/unindexed.md", "catalog/entries/unindexed/en/guide.md",
                       "templates/unindexed.md"):
            with self.subTest(orphan=orphan), temporary_directory() as directory:
                pack = Path(directory) / "pack"
                shutil.copytree(KIT, pack)
                self.assert_current_resource_inventory(pack, get_contract(pack))
                path = pack / orphan
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("{}\n" if path.suffix == ".json" else "# Unindexed resource\n", encoding="utf-8")
                contract = get_contract(pack)
                # Runtime stays declaration-only; the test-only inverse check
                # must detect even a file that cannot change the runtime digest.
                self.assertEqual(contract["contract_sha256"], self.digest)
                with self.assertRaises(AssertionError) as rejected:
                    self.assert_current_resource_inventory(pack, contract)
                self.assertIn(orphan, str(rejected.exception))
                records.append({"injected_path": orphan, "runtime_digest_unchanged": True,
                                "inverse_check_rejected": True})
        write_json(EVIDENCE / "bdk01-inverse-orphan-negatives.json", records)

    def test_bdk01_faulty_declarations_fail_before_target_access(self):
        for mode in ("missing", "duplicate", "unsafe"):
            with self.subTest(mode=mode), temporary_directory() as directory:
                root = Path(directory)
                pack = root / "pack"
                shutil.copytree(KIT, pack)
                manifest_path = pack / "catalog/manifest.json"
                manifest = load_json(manifest_path)
                resources = manifest["entries"][0]["variants"]["zh-CN"]["resources"]
                if mode == "missing":
                    (pack / resources["guide"]).unlink()
                elif mode == "duplicate":
                    resources["template"] = manifest["entries"][0]["variants"]["en"]["resources"]["template"]
                else:
                    resources["guide"] = "../outside.md"
                write_json(manifest_path, manifest)
                target = root / "target"
                target.mkdir()
                before = snapshot(target)
                error_code(self, "invalid-catalog", plan_document, target, "requirements", "sensor", profile_root=pack)
                self.assertEqual(snapshot(target), before)

    def test_bdk02_default_and_language_error_matrix(self):
        with temporary_directory() as directory:
            target = Path(directory)
            before = snapshot(target)
            for entry in self.entries:
                with self.subTest(template_id=entry["template_id"], selection="default-en"):
                    omitted = plan_document(target, entry["document_class"], "sensor", profile_root=KIT,
                                            template_id=entry["template_id"], layer="core" if entry["document_class"] == "api-design" else None)
                    explicit = plan_document(target, entry["document_class"], "sensor", profile_root=KIT,
                                             template_id=entry["template_id"], layer="core" if entry["document_class"] == "api-design" else None,
                                             language="en")
                    self.assertEqual(omitted, explicit)
                    self.assertEqual(omitted["language"], "en")
                    self.assertEqual(omitted["effective_identity"]["language"], "en")
            for language in ("zh", "en-US", "ZH-CN", "zh_CN"):
                with self.subTest(language=language):
                    error_code(self, "unknown-language", plan_document, target, "requirements", "sensor", profile_root=KIT, language=language)
            for language in ("", " zh-CN", "zh-CN ", "zh--CN", "-zh", "zh-", "zh-abcdefghijk", "中文", 3, [], True):
                with self.subTest(language=repr(language)):
                    error_code(self, "invalid-request", plan_document, target, "requirements", "sensor", profile_root=KIT, language=language)
            error_code(self, "invalid-request", check_documents, target, profile_root=KIT, language="en")
            error_code(self, "invalid-request", check_documents, target, "requirements", profile_root=KIT, language="zh-CN")
            for old in ("v1.0.0", "v1.1.0", "v1.2.0"):
                for language in ("en", "zh-CN", "unsupported", ""):
                    with self.subTest(old=old, language=language):
                        error_code(self, "invalid-request", plan_document, target, "requirements", "sensor",
                                   profile_root=KIT / "compatibility" / old, language=language)
            self.assertEqual(snapshot(target), before)

    def test_bdk03_all_42_isolated_fixtures_exact_bytes_check_update(self):
        evidence = []
        for entry in self.entries:
            for language in ("en", "zh-CN"):
                template_id = entry["template_id"]
                with self.subTest(template_id=template_id, language=language), temporary_directory() as directory:
                    target = Path(directory)
                    fixture = fixture_for(entry, language)
                    before = snapshot(target)
                    planned = plan_document(target, fixture["document_class"], fixture["name"], profile_root=KIT,
                                            template_id=template_id, layer=fixture["layer"], language=language)
                    self.assertEqual(snapshot(target), before)
                    self.assertEqual(planned["language"], language)
                    self.assertEqual(planned["required_sections"], entry["variants"][language]["required_sections"])
                    created = create_from_fixture(target, fixture, digest=self.digest)
                    assert_created_identity(self, target, created, fixture, self.digest)
                    document = target / created["path"]
                    stored = (KIT / fixture["expected_document"]).read_bytes()
                    compare_created_bytes(stored, document.read_bytes(), self.digest, created["effective_identity"])
                    checked = check_documents(target, profile_root=KIT, selection=selection(fixture), language=language)
                    self.assertTrue(checked["ok"], checked["diagnostics"])
                    self.assertEqual(checked["scope"]["language"], language)
                    self.assertEqual(checked["effective_identity_by_path"][created["path"]], created["effective_identity"])
                    artifact_dir = EVIDENCE / "isolated" / template_id / language
                    artifact_dir.mkdir(parents=True, exist_ok=True)
                    (artifact_dir / "created.md").write_bytes(document.read_bytes())
                    (artifact_dir / "registry.json").write_bytes(registry_bytes(target))
                    write_json(artifact_dir / "plan.json", planned)
                    write_json(artifact_dir / "created.json", created)
                    write_json(artifact_dir / "check.json", checked)
                    write_json(artifact_dir / "acceptance-manifest.json", {
                        "expected_contract_sha256": self.digest,
                        "fixture_path": entry["variants"][language]["resources"]["fixture"],
                        "fixture_sha256": sha_path(KIT / entry["variants"][language]["resources"]["fixture"]),
                    })
                    registry_before = registry_bytes(target)
                    marker_before = document.read_bytes().split(b"\n", 1)[0]
                    key, content, fields = updated_input(fixture)
                    updated = update_section(target, fixture["document_class"], fixture["name"], key, content,
                                             created["sha256"], fields=fields, profile_root=KIT,
                                             layer=fixture["layer"], language=language)
                    self.assertEqual(updated["language"], language)
                    self.assertEqual(updated["effective_identity"], created["effective_identity"])
                    self.assertEqual(registry_bytes(target), registry_before)
                    self.assertEqual(document.read_bytes().split(b"\n", 1)[0], marker_before)
                    rechecked = check_documents(target, profile_root=KIT, selection=selection(fixture))
                    self.assertIsNone(rechecked["scope"]["language"])
                    self.assertTrue(rechecked["ok"], rechecked["diagnostics"])
                    (artifact_dir / "updated.md").write_bytes(document.read_bytes())
                    write_json(artifact_dir / "updated.json", updated)
                    write_json(artifact_dir / "updated-check.json", rechecked)
                    evidence.append({
                        "case": "BDK-03", "template_id": template_id, "language": language,
                        "path": created["path"], "created_sha256": created["sha256"],
                        "updated_sha256": updated["sha256"], "registry_sha256": sha_bytes(registry_before),
                        "stored_example_sha256": sha_bytes(stored), "exact_byte_comparison": True,
                        "selected_check": "passed", "updated_check": "passed",
                    })
        self.assertEqual(len(evidence), 42)
        write_json(EVIDENCE / "bdk03-isolated-matrix.json", evidence)

    def test_bdk03_mixed_20_documents_and_alternatives(self):
        fixture = load_json(KIT / "catalog/shared/mixed/fixture.json")
        self.assertEqual(len(fixture["documents"]), 20)
        with temporary_directory() as directory:
            target = Path(directory)
            materialize(target, fixture)
            coverage = target / fixture["coverage"]["target"]
            coverage.parent.mkdir(parents=True, exist_ok=True)
            coverage.write_bytes((KIT / fixture["coverage"]["pack"]).read_bytes())
            records = []
            for document_fixture in sorted(fixture["documents"], key=lambda item: item["document_class"] == "flow-index"):
                created = create_from_fixture(target, document_fixture, digest=self.digest, include_companions=False)
                assert_created_identity(self, target, created, document_fixture, self.digest)
                stored = (KIT / document_fixture["expected_document"]).read_bytes()
                compare_created_bytes(stored, (target / created["path"]).read_bytes(), self.digest, created["effective_identity"])
                artifact = EVIDENCE / "mixed" / created["path"]
                artifact.parent.mkdir(parents=True, exist_ok=True)
                artifact.write_bytes((target / created["path"]).read_bytes())
                records.append({"path": created["path"], "language": created["language"], "sha256": created["sha256"]})
            pairs = {(item["document_class"], item["language"]) for item in fixture["documents"]}
            self.assertEqual(len(pairs), 20)
            self.assertEqual({item["template_id"] for item in fixture["documents"] if item["document_class"] == "requirements"},
                             {"generic-requirements", "system-requirements"})
            report = check_documents(target, profile_root=KIT)
            self.assertTrue(report["ok"], report["diagnostics"])
            self.assertEqual(len(report["effective_identity_by_path"]), 20)
            self.assertIsNone(report["scope"]["language"])
            zh_context = next(item for item in fixture["documents"] if item["template_id"] == "system-context" and item["language"] == "zh-CN")
            context_text = (target / f"docs/architecture/{zh_context['name']}.md").read_text(encoding="utf-8")
            self.assertIn("| 边界要素 |", context_text)
            self.assertNotIn("**图 ", context_text)
            zh_flow = next(item for item in fixture["documents"] if item["template_id"] == "interaction-flow" and item["language"] == "zh-CN")
            flow_text = (target / f"docs/design/{zh_flow['name']}.md").read_text(encoding="utf-8")
            self.assertIn("第一", flow_text)
            self.assertNotIn("**图 ", flow_text)
            (EVIDENCE / "mixed/registry.json").write_bytes(registry_bytes(target))
            write_json(EVIDENCE / "mixed/check.json", report)
            write_json(EVIDENCE / "bdk03-mixed-matrix.json", {"documents": records, "whole_check": "passed", "contract_sha256": self.digest})

    def test_bdk03_localized_figure_attachment_and_update(self):
        entry = next(item for item in self.entries if item["template_id"] == "generic-requirements")
        fixture = fixture_for(entry, "zh-CN")
        with temporary_directory() as directory:
            target = Path(directory)
            created = create_from_fixture(target, fixture, digest=self.digest)
            document = target / created["path"]
            registry_before = registry_bytes(target)
            asset = document.parent / "assets/zh-context.svg"
            asset.parent.mkdir(parents=True, exist_ok=True)
            asset.write_bytes((KIT / "catalog/shared/mixed/assets/system-context.svg").read_bytes())
            figure = {
                "asset": "assets/zh-context.svg", "title": "虚构系统边界",
                "Question": "哪些要素位于系统边界内？", "Model view": "Example::SystemContext",
                "Companions": "需求与验收条件表。", "Status": "renderable-now; designed-only",
                "Reading": "AsterHub 位于边界内，Operator 位于边界外。", "Limits": "该图不证明内部组成或执行结果。",
            }
            attached = attach_figure(target, "requirements", fixture["name"], "purpose", figure,
                                     created["sha256"], profile_root=KIT, language="zh-CN")
            self.assertEqual(attached["language"], "zh-CN")
            text = document.read_text(encoding="utf-8")
            anchor = fixture["sections"]["purpose"]["anchor"]
            self.assertIn(f"**图 {anchor}-1 — 虚构系统边界.**", text)
            for label in ("问题", "模型视图", "配套材料", "状态", "解读", "限制"):
                self.assertIn(f"**{label}:**", text)
            self.assertEqual(registry_bytes(target), registry_before)
            self.assertTrue(check_documents(target, profile_root=KIT, selection=selection(fixture), language="zh-CN")["ok"])

    def test_bdk04_collisions_assertions_persisted_defects_and_prose_only(self):
        generic = next(item for item in self.entries if item["template_id"] == "generic-requirements")
        zh = fixture_for(generic, "zh-CN")
        en = copy.deepcopy(fixture_for(generic, "en"))
        en["name"] = zh["name"]
        with temporary_directory() as directory:
            target = Path(directory)
            created = create_from_fixture(target, zh, digest=self.digest)
            document = target / created["path"]
            before = snapshot(target)
            error_code(self, "document-exists", create_from_fixture, target, en, KIT, self.digest)
            self.assertEqual(snapshot(target), before)
            original = document.read_bytes()
            document.unlink()
            before_orphan = snapshot(target)
            error_code(self, "registry-mismatch", create_from_fixture, target, en, KIT, self.digest)
            self.assertEqual(snapshot(target), before_orphan)
            document.write_bytes(original)
            before = snapshot(target)
            error_code(self, "invalid-request", update_section, target, "requirements", zh["name"], "purpose",
                       "Conflicting assertion must not write.", created["sha256"], profile_root=KIT, language="en")
            self.assertEqual(snapshot(target), before)
            updated = update_section(target, "requirements", zh["name"], "purpose",
                                     "This arbitrary English prose does not reselect or convert the persisted Chinese variant.",
                                     created["sha256"], profile_root=KIT)
            self.assertEqual(updated["language"], "zh-CN")
            self.assertEqual(sha_bytes(registry_bytes(target)), before["docs/.document-kit-managed.json"])
            self.assertTrue(check_documents(target, profile_root=KIT, selection=selection(zh))["ok"])

        for mode in ("registry-missing", "registry-unsupported", "marker-missing", "disagreeing"):
            with self.subTest(mode=mode), temporary_directory() as directory:
                target = Path(directory)
                created = create_from_fixture(target, zh, digest=self.digest)
                document = target / created["path"]
                registry = target / "docs/.document-kit-managed.json"
                value = load_json(registry)
                if mode == "registry-missing":
                    del value["entries"][0]["language"]
                    write_json(registry, value)
                elif mode == "registry-unsupported":
                    value["entries"][0]["language"] = "zh"
                    write_json(registry, value)
                elif mode == "marker-missing":
                    document.write_text(document.read_text(encoding="utf-8").replace(" language=zh-CN", "", 1), encoding="utf-8")
                if mode == "disagreeing":
                    document.write_text(document.read_text(encoding="utf-8").replace(" language=zh-CN", " language=en", 1), encoding="utf-8")
                before = snapshot(target)
                error_code(self, "registry-mismatch", update_section, target, "requirements", zh["name"], "purpose",
                           "No persisted-language repair is allowed.", sha_path(document), profile_root=KIT)
                self.assertEqual(snapshot(target), before)

    def test_bdk04_changed_default_does_not_reselect_persisted_variant(self):
        generic = next(item for item in self.entries if item["template_id"] == "generic-requirements")
        fixture = fixture_for(generic, "zh-CN")
        with temporary_directory() as directory:
            root = Path(directory)
            target = root / "target"
            created = create_from_fixture(target, fixture, digest=self.digest)
            derived = root / "derived"
            shutil.copytree(KIT, derived)
            profile_path = derived / "profile.json"
            profile = load_json(profile_path)
            manifest_path = derived / "catalog/manifest.json"
            manifest = load_json(manifest_path)
            alternate = copy.deepcopy(next(e for e in manifest["entries"] if e["template_id"] == "generic-requirements"))
            alternate["template_id"] = "generic-alternate-requirements"
            for language, variant in alternate["variants"].items():
                for kind, old_path in variant["resources"].items():
                    new_path = old_path.replace("/generic-requirements/", "/generic-alternate-requirements/")
                    destination = derived / new_path
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    content = (derived / old_path).read_text(encoding="utf-8").replace("generic-requirements", "generic-alternate-requirements")
                    destination.write_text(content, encoding="utf-8")
                    variant["resources"][kind] = new_path
            manifest["entries"].append(alternate)
            manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            profile["document_classes"]["requirements"]["default_template_id"] = "generic-alternate-requirements"
            profile["profile_id"] = "derived-bdk-docs"
            profile["compatibility_packs"] = []
            write_json(profile_path, profile)
            for resource in derived.rglob("*"):
                if resource.is_file() and resource.suffix in (".json", ".md") and "compatibility" not in resource.relative_to(derived).parts:
                    resource.write_bytes(resource.read_bytes().replace(b"xdesign-sysml-docs", b"derived-bdk-docs"))
            shutil.copytree(KIT, derived / "compatibility/v1.3.0")
            profile = load_json(profile_path)
            profile["compatibility_packs"] = [{
                "profile_id": self.contract["profile_id"], "version": "1.3.0",
                "root": "compatibility/v1.3.0", "contract_sha256": self.digest,
                "contract_digest_version": "3",
            }]
            write_json(profile_path, profile)
            planned = plan_document(target, "requirements", "new-page", profile_root=derived)
            self.assertEqual(planned["template_id"], "generic-alternate-requirements")
            self.assertEqual(planned["language"], "en")
            before = registry_bytes(target)
            updated = update_section(target, "requirements", fixture["name"], "purpose",
                                     "持久化中文变体不因默认条目变化而重新选择。",
                                     created["sha256"], profile_root=derived)
            self.assertEqual(updated["effective_identity"], created["effective_identity"])
            self.assertEqual(registry_bytes(target), before)
            asset = (target / created["path"]).parent / "assets/context.svg"
            asset.parent.mkdir(parents=True, exist_ok=True)
            asset.write_bytes((KIT / "catalog/shared/mixed/assets/system-context.svg").read_bytes())
            attached = attach_figure(target, "requirements", fixture["name"], "purpose", {
                "asset": "assets/context.svg", "title": "虚构边界",
                "Question": "边界为何？", "Model view": "Example::SystemContext",
                "Companions": "文字。", "Status": "external-tool; designed-only",
                "Reading": "仅说明边界。", "Limits": "不证明执行。",
            }, updated["sha256"], profile_root=derived)
            self.assertEqual(attached["effective_identity"], created["effective_identity"])
            self.assertEqual(registry_bytes(target), before)
            checked = check_documents(target, profile_root=derived, selection=selection(fixture))
            self.assertTrue(checked["ok"], checked["diagnostics"])
            write_json(EVIDENCE / "bdk04-changed-selected-default.json", {
                "plan": planned, "updated": updated, "attached": attached,
                "check": checked, "registry_unchanged": True,
            })


if __name__ == "__main__":
    unittest.main()
