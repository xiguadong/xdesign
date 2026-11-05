from __future__ import annotations

import os

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from _support import KIT, REPO, copy_pack, create_from_fixture, load_json, selected_digest
from xdesign_doc_kit import catalog
from xdesign_doc_kit.compatibility import DocumentKitError, digest_v2
from xdesign_doc_kit.core import check_documents, create_document, get_contract, plan_document

CACHE = REPO / "cache/document-kit-catalog-tests"
CACHE = Path(os.environ.get("XDESIGN_DOC_KIT_TEST_CACHE", str(CACHE)))
ZERO = "0" * 64


def expect_code(case: unittest.TestCase, code: str, operation, *args, **kwargs) -> DocumentKitError:
    with case.assertRaises(DocumentKitError) as rejected:
        operation(*args, **kwargs)
    case.assertEqual(rejected.exception.code, code, rejected.exception.diagnostics)
    case.assertEqual(set(rejected.exception.diagnostics[0]), {"rule", "code", "file", "line", "detail"})
    return rejected.exception


class CatalogTests(unittest.TestCase):
    def setUp(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=CACHE)
        self.root = Path(self.temporary.name)
        self.target = self.root / "target"
        self.target.mkdir()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def pack(self, name: str = "pack") -> Path:
        return copy_pack(self.root / name)

    def test_doc08_inventory_sources_and_derived_inventory(self) -> None:
        contract = get_contract(KIT)
        entries = contract["catalog"]["entries"]
        self.assertEqual(len(entries), 16)
        self.assertEqual(contract["contract_digest_version"], "2")
        self.assertNotEqual(contract["contract_sha256"], ZERO)
        cited = {usage["source_id"] for entry in entries for usage in entry["sources"]}
        self.assertEqual(cited, {"DPR", "GOOD-DOCS", "GITHUB-DOCS", "SPECD", "C4", "OPENSPEC", "ARC42", "REQ42"})
        for entry in entries:
            with self.subTest(template_id=entry["template_id"]):
                self.assertEqual(set(entry), catalog.ENTRY_FIELDS)
                self.assertTrue(entry["sources"])
                self.assertTrue(entry["checks"])
                for key in ("template", "guide", "example", "fixture"):
                    self.assertTrue((KIT / entry[key]).is_file())
        derived = self.pack("derived")
        manifest_path = derived / "catalog/manifest.json"
        manifest = load_json(manifest_path)
        manifest["profile_id"] = "derived-docs"
        manifest["entries"] = [entry for entry in manifest["entries"] if entry["template_id"] != "design-review"]
        profile_path = derived / "profile.json"
        profile = load_json(profile_path)
        profile["profile_id"] = "derived-docs"
        profile_path.write_text(json.dumps(profile, indent=2) + "\n")
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
        for fixture_path in (derived / "catalog/fixtures").glob("*.json"):
            fixture = load_json(fixture_path)
            fixture["profile_id"] = "derived-docs"
            fixture_path.write_text(json.dumps(fixture, indent=2) + "\n")
        mixed_path = derived / "catalog/mixed/fixture.json"
        mixed = load_json(mixed_path)
        mixed["profile_id"] = "derived-docs"
        mixed_path.write_text(json.dumps(mixed, indent=2) + "\n")
        for example in (derived / "catalog/examples").glob("*.md"):
            example.write_bytes(example.read_bytes().replace(b"profile=xdesign-sysml-docs@", b"profile=derived-docs@"))
        self.assertEqual(len(get_contract(derived)["catalog"]["entries"]), 15)

    def test_doc09_explicit_defaults_invalid_selection_and_no_writes(self) -> None:
        contract = get_contract(KIT)
        for entry in contract["catalog"]["entries"]:
            with self.subTest(template_id=entry["template_id"]):
                plan = plan_document(self.target, entry["document_class"], "sensor-system",
                                     profile_root=KIT, template_id=entry["template_id"])
                self.assertEqual(plan["template_id"], entry["template_id"])
                self.assertEqual(plan["effective_identity"]["contract_sha256"], contract["contract_sha256"])
                self.assertEqual(plan["path"], contract["profile"]["document_classes"][entry["document_class"]]["path_template"].replace("{name}", "sensor-system"))
                self.assertFalse((self.target / plan["path"]).exists())
        for entry in contract["catalog"]["entries"][:5]:
            with self.subTest(default=entry["template_id"]):
                omitted = plan_document(self.target, entry["document_class"], "sensor-system", profile_root=KIT)
                explicit = plan_document(self.target, entry["document_class"], "sensor-system",
                                         profile_root=KIT, template_id=entry["template_id"])
                self.assertEqual(omitted["effective_identity"], explicit["effective_identity"])
                self.assertEqual(omitted["required_sections"], explicit["required_sections"])
        for template_id, code in (("not-present", "unknown-template"), ("Bad_ID", "unknown-template"),
                                  ("generic-design", "template-class-mismatch")):
            with self.subTest(template_id=template_id, code=code):
                expect_code(self, code, plan_document, self.target, "requirements", "sensor-system",
                            profile_root=KIT, template_id=template_id)
        self.assertFalse((self.target / "docs").exists())

    def test_doc09_generic_creation_omission_equals_explicit(self) -> None:
        for entry in get_contract(KIT)["catalog"]["entries"][:5]:
            with self.subTest(template_id=entry["template_id"]):
                fixture = load_json(KIT / entry["fixture"])
                first = self.root / f"explicit-{entry['template_id']}"
                second = self.root / f"omitted-{entry['template_id']}"
                explicit = create_from_fixture(first, fixture, expected_digest=selected_digest())
                second.mkdir()
                from _support import materialize_inputs
                materialize_inputs(second, fixture["models"] + fixture["figures"])
                omitted = create_document(second, fixture["document_class"], fixture["name"],
                                          fixture["metadata"], fixture["sections"], profile_root=KIT)
                self.assertEqual(explicit["effective_identity"], omitted["effective_identity"])
                self.assertEqual((first / explicit["path"]).read_bytes(), (second / omitted["path"]).read_bytes())

    def test_doc08_default_errors_are_specific(self) -> None:
        for mode in ("missing", "unknown", "non-generic", "cross-class"):
            with self.subTest(mode=mode):
                pack = self.pack(mode)
                profile_path = pack / "profile.json"
                profile = load_json(profile_path)
                if mode == "missing":
                    del profile["document_classes"]["design"]["default_template_id"]
                elif mode == "unknown":
                    profile["document_classes"]["design"]["default_template_id"] = "not-present"
                elif mode == "non-generic":
                    profile["document_classes"]["design"]["default_template_id"] = "interaction-flow"
                else:
                    profile["document_classes"]["architecture"]["default_template_id"] = "generic-design"
                profile_path.write_text(json.dumps(profile, indent=2) + "\n")
                expect_code(self, "template-default", get_contract, pack)

    def test_doc08_doc14_source_and_license_precedence(self) -> None:
        cases = []
        for mode in ("missing-license", "primary-cardinality", "unsafe-primary", "malformed-url", "invalid-license",
                     "unsafe-license", "unknown-source", "copying-review"):
            pack = self.pack(mode)
            sources_path = pack / "catalog/sources.json"
            sources = load_json(sources_path)
            record = sources["sources"][0]
            if mode == "missing-license":
                del record["license_url"]
                record["url"] = "javascript:alert(1)"
            elif mode == "primary-cardinality":
                record["local_path"] = "catalog/routing.md"
                record["license_url"] = None
            elif mode == "unsafe-primary":
                record["url"] = "https://user:secret@example.com/source"
                record["license_url"] = None
            elif mode == "malformed-url":
                record["url"] = "http://[::1"
            elif mode == "invalid-license":
                record["license_url"] = None
            elif mode == "unsafe-license":
                record["license_url"] = "javascript:alert(1)"
            sources_path.write_text(json.dumps(sources, indent=2) + "\n")
            if mode == "unknown-source":
                manifest_path = pack / "catalog/manifest.json"
                manifest = load_json(manifest_path)
                manifest["entries"][0]["sources"][0]["source_id"] = "UNKNOWN"
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
            if mode == "copying-review":
                manifest_path = pack / "catalog/manifest.json"
                manifest = load_json(manifest_path)
                manifest["entries"][0]["sources"][0]["copying_decision"] = "copied"
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
            expected = {
                "missing-license": "invalid-source", "primary-cardinality": "invalid-source",
                "unsafe-primary": "unsafe-url", "malformed-url": "unsafe-url", "invalid-license": "invalid-license",
                "unsafe-license": "unsafe-url", "unknown-source": "invalid-source",
                "copying-review": "copying-review-required",
            }[mode]
            expect_code(self, expected, get_contract, pack)

    def test_s02_s03_s04_s05_example_and_pack_sentinel_defects(self) -> None:
        example_path = KIT / "catalog/examples/generic-requirements.md"
        original = example_path.read_bytes()
        variants = {
            "actual-digest": original.replace(ZERO.encode(), selected_digest().encode(), 1),
            "other-digest": original.replace(ZERO.encode(), b"f" * 64, 1),
            "uppercase": original.replace(ZERO.encode(), b"A" * 64, 1),
            "empty-contract": original.replace(b"contract=" + ZERO.encode(), b"contract=", 1),
            "short": original.replace(ZERO.encode(), b"0" * 63, 1),
            "long": original.replace(ZERO.encode(), b"0" * 65, 1),
            "prefixed": original.replace(b"contract=" + ZERO.encode(), b"contract=x" + ZERO.encode(), 1),
            "two-markers": original + original.split(b"\n", 1)[0] + b"\n",
            "two-contracts": original.replace(b" digest-version=", b" contract=" + ZERO.encode() + b" digest-version=", 1),
            "body-sentinel": original + ZERO.encode(),
            "noncontract-profile": original.replace(b"profile=xdesign-sysml-docs@", b"profile=" + ZERO.encode() + b"@", 1),
            "noncontract-class": original.replace(b"class=requirements", b"class=" + ZERO.encode(), 1),
            "noncontract-template": original.replace(b"template=generic-requirements", b"template=" + ZERO.encode(), 1),
        }
        for mode, data in variants.items():
            with self.subTest(mode=mode):
                pack = self.pack(mode)
                (pack / "catalog/examples/generic-requirements.md").write_bytes(data)
                expect_code(self, "invalid-catalog", get_contract, pack)
                self.assertFalse((self.target / "docs").exists())
        for relative in ("catalog/guides/generic-design.md", "catalog/templates/generic-design.md",
                         "catalog/fixtures/generic-design.json", "catalog/manifest.json", "catalog/sources.json",
                         "catalog/mixed/mbse/model/example.sysml", "catalog/mixed/assets/system-context.svg",
                         "catalog/mixed/fixture.json", "catalog/mixed/coverage.md", "RULES.md",
                         "compatibility/v1.0.0/RULES.md"):
            with self.subTest(resource=relative):
                pack = self.pack("sentinel-" + relative.replace("/", "-"))
                path = pack / relative
                if path.suffix == ".json":
                    value = load_json(path)
                    if relative.endswith("manifest.json"):
                        value["entries"][0]["title"] += " " + ZERO
                    elif relative.endswith("sources.json"):
                        value["sources"][0]["title"] += " " + ZERO
                    elif "mixed" in relative:
                        value["coverage"]["target"] += ZERO
                    else:
                        value["name"] += ZERO
                    path.write_text(json.dumps(value, indent=2) + "\n")
                else:
                    path.write_bytes(path.read_bytes() + b"\n" + ZERO.encode() + b"\n")
                expect_code(self, "invalid-catalog", get_contract, pack)

    def test_s06_zero_recomputation_descriptor_and_request_are_distinct(self) -> None:
        fixture = load_json(KIT / "catalog/fixtures/generic-requirements.json")
        with patch("xdesign_doc_kit.catalog.digest_v2", return_value=ZERO):
            expect_code(self, "invalid-catalog", get_contract, KIT)
            expect_code(self, "invalid-catalog", plan_document, self.target, "requirements", "sensor", profile_root=KIT)
            expect_code(self, "invalid-catalog", create_document, self.target, "requirements", "sensor",
                        fixture["metadata"], fixture["sections"], profile_root=KIT)
            expect_code(self, "invalid-catalog", check_documents, self.target, profile_root=KIT)
        descriptor = self.pack("zero-descriptor")
        profile_path = descriptor / "profile.json"
        profile = load_json(profile_path)
        profile["compatibility_packs"][0]["contract_sha256"] = ZERO
        profile_path.write_text(json.dumps(profile, indent=2) + "\n")
        expect_code(self, "contract-drift", get_contract, descriptor)
        expect_code(self, "invalid-request", create_document, self.target, "requirements", "sensor",
                    fixture["metadata"], fixture["sections"], profile_root=self.root / "missing-pack",
                    expected_contract_sha256=ZERO)
        self.assertFalse((self.target / "docs").exists())

    def test_s11_s12_raw_digest_changes_every_resource_category(self) -> None:
        from xdesign_doc_kit.core import _contract
        contract = _contract(KIT)
        resources = contract["resources"]
        original_digest = contract["contract_sha256"]
        paths = [
            "profile.json", "RULES.md", "catalog/manifest.json", "catalog/sources.json",
            "catalog/templates/generic-design.md", "catalog/guides/generic-design.md",
            "catalog/examples/generic-design.md", "catalog/fixtures/generic-design.json",
            "catalog/mixed/fixture.json", "catalog/mixed/coverage.md",
            "catalog/mixed/mbse/model/example.sysml", "catalog/mixed/assets/system-context.svg",
            "compatibility/v1.0.0/RULES.md",
        ]
        for relative in paths:
            with self.subTest(resource=relative):
                changed = dict(resources)
                changed[relative] = changed[relative] + b"\n"
                self.assertNotEqual(digest_v2(changed), original_digest)
        example = resources["catalog/examples/generic-design.md"]
        for mode, mutated in (
            ("noncontract-marker", example.replace(b"profile=xdesign-sysml-docs@1.1.0", b"profile=changed@1.1.0", 1)),
            ("sentinel", example.replace(ZERO.encode(), b"0" * 63, 1)),
        ):
            with self.subTest(raw_digest_mutation=mode):
                changed = dict(resources)
                changed["catalog/examples/generic-design.md"] = mutated
                self.assertNotEqual(digest_v2(changed), original_digest)
        for relative in ("catalog/guides/generic-design.md", "catalog/examples/generic-design.md",
                         "catalog/fixtures/generic-design.json", "catalog/manifest.json"):
            with self.subTest(valid_drift=relative):
                pack = self.pack("drift-" + relative.replace("/", "-"))
                path = pack / relative
                path.write_bytes(path.read_bytes() + b"\n")
                self.assertNotEqual(get_contract(pack)["contract_sha256"], original_digest)
        for marker_mutation in (b"profile=changed@1.1.0", b"0" * 63):
            pack = self.pack("marker-" + str(len(marker_mutation)))
            path = pack / "catalog/examples/generic-design.md"
            data = path.read_bytes()
            if marker_mutation.startswith(b"profile"):
                data = data.replace(b"profile=xdesign-sysml-docs@1.1.0", marker_mutation, 1)
            else:
                data = data.replace(ZERO.encode(), marker_mutation, 1)
            path.write_bytes(data)
            expect_code(self, "invalid-catalog", get_contract, pack)

    def test_doc08_every_manifest_field_is_required_and_default_cannot_duplicate(self) -> None:
        for field in sorted(catalog.ENTRY_FIELDS):
            with self.subTest(missing_entry_field=field):
                pack = self.pack(f"missing-{field}")
                manifest_path = pack / "catalog/manifest.json"
                manifest = load_json(manifest_path)
                del manifest["entries"][0][field]
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
                expect_code(self, "invalid-catalog", get_contract, pack)
        for field in ("schema_version", "profile_id", "version", "entries"):
            with self.subTest(missing_manifest_field=field):
                pack = self.pack(f"missing-manifest-{field}")
                manifest_path = pack / "catalog/manifest.json"
                manifest = load_json(manifest_path)
                del manifest[field]
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
                expect_code(self, "invalid-catalog", get_contract, pack)
        pack = self.pack("duplicate-default")
        manifest_path = pack / "catalog/manifest.json"
        manifest = load_json(manifest_path)
        duplicate = copy.deepcopy(next(entry for entry in manifest["entries"] if entry["template_id"] == "generic-design"))
        manifest["entries"].append(duplicate)
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
        expect_code(self, "invalid-catalog", get_contract, pack)

    def test_full_input_fixture_shape_and_profile_catalog_schema_precedence(self) -> None:
        for entry in get_contract(KIT)["catalog"]["entries"]:
            fixture_bytes = (KIT / entry["fixture"]).read_bytes()
            self.assertNotIn(ZERO.encode(), fixture_bytes)
            self.assertNotIn(selected_digest().encode(), fixture_bytes)
            fixture = json.loads(fixture_bytes)
            self.assertEqual(fixture["contract_digest_version"], "2")
            self.assertIn("models", fixture)
            self.assertIn("figures", fixture)
        pack = self.pack("missing-fixture-identity")
        fixture_path = pack / "catalog/fixtures/generic-design.json"
        fixture = load_json(fixture_path)
        del fixture["profile_id"]
        fixture_path.write_text(json.dumps(fixture, indent=2) + "\n")
        expect_code(self, "invalid-catalog", get_contract, pack)
        caller_pack = self.pack("missing-fixture-section")
        fixture_path = caller_pack / "catalog/fixtures/generic-design.json"
        fixture = load_json(fixture_path)
        del fixture["sections"]["purpose"]
        fixture_path.write_text(json.dumps(fixture, indent=2) + "\n")
        expect_code(self, "invalid-catalog", get_contract, caller_pack)
        digest_pack = self.pack("fixture-actual-digest")
        fixture_path = digest_pack / "catalog/fixtures/generic-design.json"
        fixture = load_json(fixture_path)
        fixture["metadata"]["title"] += " " + selected_digest()
        fixture_path.write_text(json.dumps(fixture, indent=2) + "\n")
        from xdesign_doc_kit.core import _contract
        expect_code(self, "invalid-catalog", catalog.load_catalog_contract, digest_pack, _contract,
                    digest_provider=lambda resources: selected_digest())
        schema_pack = self.pack("schema-precedence")
        profile_path = schema_pack / "profile.json"
        profile = load_json(profile_path)
        profile["schema_version"] = "2.0"
        profile["project"]["model_roots"] = []
        profile_path.write_text(json.dumps(profile, indent=2) + "\n")
        manifest_path = schema_pack / "catalog/manifest.json"
        manifest = load_json(manifest_path)
        manifest["schema_version"] = "2.0"
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
        expect_code(self, "invalid-profile", get_contract, schema_pack)
        roots_pack = self.pack("bad-roots")
        profile_path = roots_pack / "profile.json"
        profile = load_json(profile_path)
        profile["project"]["model_roots"] = []
        profile_path.write_text(json.dumps(profile, indent=2) + "\n")
        expect_code(self, "invalid-profile", get_contract, roots_pack)


if __name__ == "__main__":
    unittest.main()
