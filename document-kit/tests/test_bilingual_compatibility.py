from __future__ import annotations

import unittest
from pathlib import Path

from _bilingual_support import (
    EVIDENCE, KIT, create_from_fixture, error_code, fixture_for, load_json,
    materialize, registry_bytes, selection, temporary_directory, write_json,
)
from xdesign_doc_kit import check_documents, create_document, get_contract, update_section
from xdesign_doc_kit.core import _contract


EXPECTED = {
    "1.0.0": ("1", "1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6", 8),
    "1.1.0": ("2", "e2b827c6e929f4a8d03344a91ca72f2eb837962e916d5e7f0487e315e1440cef", 85),
    "1.2.0": ("2", "a4ab976f05769bff34f3b6d3acd0de1dafb728dcf372d099fc3ac938758cf698", 191),
}


def closure(contract):
    return {path: value[1] if isinstance(value, tuple) else value for path, value in contract["resources"].items()}


class BilingualCompatibilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.current = get_contract(KIT)
        cls.entries = cls.current["catalog"]["entries"]

    def test_bdk07_original_closures_digests_and_nested_bytes(self):
        contracts = {}
        evidence = []
        for version, (digest_version, digest, count) in EXPECTED.items():
            root = KIT / "compatibility" / f"v{version}"
            contract = get_contract(root)
            self.assertEqual(contract["contract_digest_version"], digest_version)
            self.assertEqual(contract["contract_sha256"], digest)
            internal = _contract(root)
            self.assertEqual(len(internal["resources"]), count)
            contracts[version] = internal
            evidence.append({"version": version, "resource_count": count, "contract_sha256": digest,
                             "contract_digest_version": digest_version, "all_declared_hashes_recomputed": True})
        v10 = closure(contracts["1.0.0"])
        v11 = closure(contracts["1.1.0"])
        v12 = closure(contracts["1.2.0"])
        nested_v11 = KIT / "compatibility/v1.1.0"
        nested_v12 = KIT / "compatibility/v1.2.0"
        self.assertEqual(v10, closure(_contract(nested_v11 / "compatibility/v1.0.0")))
        self.assertEqual(v10, closure(_contract(nested_v12 / "compatibility/v1.0.0")))
        self.assertEqual(v10, closure(_contract(nested_v12 / "compatibility/v1.1.0/compatibility/v1.0.0")))
        self.assertEqual(v11, closure(_contract(nested_v12 / "compatibility/v1.1.0")))
        self.assertEqual(v12, closure(_contract(KIT / "compatibility/v1.2.0")))
        current_v10 = _contract(KIT / "compatibility/v1.0.0")
        current_v11 = _contract(KIT / "compatibility/v1.1.0")
        current_v12 = _contract(KIT / "compatibility/v1.2.0")
        self.assertEqual(v10, closure(current_v10))
        self.assertEqual(v11, closure(current_v11))
        self.assertEqual(v12, closure(current_v12))
        write_json(EVIDENCE / "bdk07-retained-closures.json", evidence)

    def test_bdk07_old_positional_calls_and_mixed_old_new_operations(self):
        generic = next(entry for entry in self.entries if entry["template_id"] == "generic-requirements")
        with temporary_directory() as directory:
            target = Path(directory)
            created_records = []
            for index, version in enumerate(("1.0.0", "1.1.0", "1.2.0")):
                pack = KIT / "compatibility" / f"v{version}"
                # 1.0 predates canonical fixtures; use valid caller input retained in 1.1.
                input_pack = KIT / "compatibility/v1.1.0" if version == "1.0.0" else pack
                fixture = load_json(input_pack / "catalog/fixtures/generic-requirements.json")
                fixture["version"] = version
                fixture["name"] = f"retained-{version.replace('.', '-')}"
                for section_index, section in enumerate(fixture["sections"].values()):
                    section["anchor"] = f"R{7000 + index * 100 + section_index}"
                materialize(target, fixture, input_pack)
                digest = EXPECTED[version][1]
                if version == "1.0.0":
                    created = create_document(target, "requirements", fixture["name"], fixture["metadata"],
                                              fixture["sections"], pack, digest)
                else:
                    created = create_document(target, "requirements", fixture["name"], fixture["metadata"],
                                              fixture["sections"], pack, digest, fixture["template_id"])
                self.assertNotIn("language", created["effective_identity"])
                self.assertNotIn("language", created)
                old_check = check_documents(target, profile_root=KIT, selection=selection(fixture))
                self.assertTrue(old_check["ok"], old_check["diagnostics"])
                registry_before = registry_bytes(target)
                updated = update_section(target, "requirements", fixture["name"], "purpose",
                                         f"Original {version} positional update remains valid.", created["sha256"],
                                         None, pack, digest)
                self.assertNotIn("language", updated["effective_identity"])
                self.assertNotIn("language", updated)
                self.assertEqual(registry_bytes(target), registry_before)
                created_records.append((fixture, pack, updated))

            for language, offset in (("en", 9000), ("zh-CN", 9100)):
                fixture = fixture_for(generic, language)
                fixture["name"] = f"current-{language.lower()}"
                for section_index, section in enumerate(fixture["sections"].values()):
                    section["anchor"] = f"R{offset + section_index}"
                created = create_from_fixture(target, fixture, digest=self.current["contract_sha256"])
                self.assertEqual(created["language"], language)
                created_records.append((fixture, KIT, created))

            report = check_documents(target, profile_root=KIT)
            self.assertTrue(report["ok"], report["diagnostics"])
            self.assertEqual(len(report["effective_identity_by_path"]), 5)
            for fixture, pack, created in created_records:
                is_current = pack == KIT
                effective_version = fixture["version"] if not is_current else "1.3.0"
                selected_report = check_documents(target, profile_root=KIT, selection=selection(fixture))
                self.assertTrue(selected_report["ok"], selected_report["diagnostics"])
                effective = selected_report["effective_identity_by_path"][fixture["name"] if False else created["path"]]
                self.assertEqual(effective["version"], effective_version)
                if effective_version == "1.3.0":
                    self.assertEqual(effective["language"], fixture["language"])
                else:
                    self.assertNotIn("language", effective)
                    error_code(self, "invalid-request", check_documents, target, profile_root=KIT,
                               selection=selection(fixture), language="en")
                registry_before = registry_bytes(target)
                content = "Mixed-release update preserves the effective historical identity."
                if effective_version == "1.3.0" and fixture["language"] == "zh-CN":
                    content = "混合版本更新保留当前中文变体的有效身份。"
                updated = update_section(target, "requirements", fixture["name"], "purpose", content,
                                         created["sha256"], profile_root=KIT)
                self.assertEqual(registry_bytes(target), registry_before)
                self.assertEqual(updated["effective_identity"], effective)
                if effective_version == "1.3.0":
                    self.assertEqual(updated["language"], fixture["language"])
                else:
                    self.assertNotIn("language", updated)

            error_code(self, "registry-mismatch", check_documents, target,
                       profile_root=KIT / "compatibility/v1.2.0")
            write_json(EVIDENCE / "bdk07-mixed-release-operations.json", {
                "old_positional_versions": ["1.0.0", "1.1.0", "1.2.0"],
                "current_languages": ["en", "zh-CN"],
                "mixed_whole_check": "passed", "old_pack_foreign_mixed_registry": "registry-mismatch",
                "document_count": len(created_records),
            })


if __name__ == "__main__":
    unittest.main()
