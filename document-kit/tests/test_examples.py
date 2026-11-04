from __future__ import annotations

import copy
import hashlib
import json
import os
import re
import tempfile
import unittest
from pathlib import Path

from _support import KIT, REPO, changed_release, create_from_fixture, load_json, materialize_inputs, registry_path, sha256
from xdesign_doc_kit.compatibility import DocumentKitError
from xdesign_doc_kit.core import check_documents, create_document, get_contract

CACHE = Path(os.environ.get("XDESIGN_DOC_KIT_ACCEPTANCE_CACHE",
                            str(REPO / "cache/document-template-library-acceptance")))
ZERO = "0" * 64
MARKER = re.compile(
    rb"<!-- document-kit: profile=([^ @]+)@([^ ]+) class=([^ ]+) template=([^ ]+) "
    rb"contract=([0-9a-f]{64}) digest-version=2 -->"
)


def expected_bytes(stored: bytes, digest: str, identity: dict | None = None) -> bytes:
    first, remainder = stored.split(b"\n", 1)
    match = MARKER.fullmatch(first)
    if match is None or stored.count(ZERO.encode("ascii")) != 1 \
            or digest.encode("ascii") in stored or match.group(5) != ZERO.encode("ascii"):
        raise DocumentKitError("fixture-mismatch", "Stored example marker or sentinel is invalid")
    if identity is not None:
        actual = (match.group(1).decode(), match.group(2).decode(), match.group(3).decode(), match.group(4).decode())
        wanted = (identity["profile_id"], identity["version"], identity["document_class"], identity["template_id"])
        if actual != wanted:
            raise DocumentKitError("fixture-mismatch", "Stored example non-contract identity differs")
    start, end = match.span(5)
    return first[:start] + digest.encode("ascii") + first[end:] + b"\n" + remainder


def compare_created_bytes(stored: bytes, created: bytes, digest: str, identity: dict) -> None:
    expected = expected_bytes(stored, digest, identity)
    if created.split(b"\n", 1)[0] != expected.split(b"\n", 1)[0] or created != expected:
        raise DocumentKitError("fixture-mismatch", "Created bytes differ after exact one-field substitution")


def validate_manifest(path: Path, digest: str, fixture_path: str, fixture_sha256: str,
                      retained_manifest_sha256: str | None = None) -> str:
    value = load_json(path)
    if not isinstance(value, dict) or set(value) != {"expected_contract_sha256", "fixture_path", "fixture_sha256"}:
        raise DocumentKitError("fixture-mismatch", "Acceptance manifest must contain exactly three fields")
    if value != {"expected_contract_sha256": digest, "fixture_path": fixture_path,
                 "fixture_sha256": fixture_sha256}:
        raise DocumentKitError("fixture-mismatch", "Acceptance manifest is stale")
    actual_hash = sha256(path)
    if retained_manifest_sha256 is not None and actual_hash != retained_manifest_sha256:
        raise DocumentKitError("fixture-mismatch", "Retained acceptance manifest hash is stale")
    return actual_hash


def write_manifest(directory: Path, name: str, digest: str, fixture_path: Path,
                   extra_fields: dict | None = None, pack: Path = KIT) -> tuple[Path, str]:
    directory.mkdir(parents=True, exist_ok=True)
    value = {
        "expected_contract_sha256": digest,
        "fixture_path": fixture_path.relative_to(pack).as_posix(),
        "fixture_sha256": sha256(fixture_path),
    }
    if extra_fields:
        value.update(extra_fields)
    path = directory / f"{name}.json"
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if set(value) != {"expected_contract_sha256", "fixture_path", "fixture_sha256"}:
        raise DocumentKitError("fixture-mismatch", "Self or additional manifest hashes are forbidden")
    return path, sha256(path)


def assert_created_identity(case: unittest.TestCase, target: Path, created: dict,
                            fixture: dict, digest: str) -> None:
    document = target / created["path"]
    first = document.read_bytes().split(b"\n", 1)[0]
    match = MARKER.fullmatch(first)
    case.assertIsNotNone(match)
    case.assertEqual(match.group(5).decode(), digest)
    expected = {
        "profile_id": "xdesign-sysml-docs",
        "version": "1.1.0",
        "document_class": fixture["document_class"],
        "template_id": fixture["template_id"],
        "contract_sha256": digest,
        "contract_digest_version": "2",
    }
    case.assertEqual(created["effective_identity"], expected)
    entries = load_json(registry_path(target))["entries"]
    entry = next(item for item in entries if item["path"] == created["path"])
    case.assertEqual(entry, {
        "path": created["path"], "class": fixture["document_class"],
        "profile_id": expected["profile_id"], "version": expected["version"],
        "contract_sha256": digest, "template_id": fixture["template_id"],
        "contract_digest_version": "2",
    })
    case.assertNotIn(ZERO, document.read_text(encoding="utf-8"))
    case.assertNotIn(ZERO, registry_path(target).read_text(encoding="utf-8"))


class ExampleAcceptanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        cls.contract = get_contract(KIT)
        cls.digest = cls.contract["contract_sha256"]
        cls.entries = cls.contract["catalog"]["entries"]

    def test_s01_inventory_markers_and_mixed_references(self) -> None:
        mixed = load_json(KIT / "catalog/mixed/fixture.json")
        references = {document["expected_document"] for document in mixed["documents"]}
        self.assertEqual(len(references), 6)
        for entry in self.entries:
            with self.subTest(template_id=entry["template_id"]):
                stored = (KIT / entry["example"]).read_bytes()
                expected_bytes(stored, self.digest, {
                    "profile_id": "xdesign-sysml-docs", "version": "1.1.0",
                    "document_class": entry["document_class"], "template_id": entry["template_id"],
                })
                self.assertNotIn(self.digest.encode("ascii"), stored)
                if entry["example"] in references:
                    self.assertIn(entry["example"], {item["example"] for item in self.entries})

    def test_doc11_s09_all_isolated_examples_exact_bytes_and_checks(self) -> None:
        evidence = []
        for entry in self.entries:
            with self.subTest(template_id=entry["template_id"]), tempfile.TemporaryDirectory(dir=CACHE) as directory:
                target = Path(directory)
                fixture_path = KIT / entry["fixture"]
                fixture = load_json(fixture_path)
                manifest, manifest_hash = write_manifest(CACHE / "manifests", entry["template_id"],
                                                         self.digest, fixture_path)
                validate_manifest(manifest, self.digest, entry["fixture"], sha256(fixture_path), manifest_hash)
                created = create_from_fixture(target, fixture, expected_digest=self.digest)
                document = target / created["path"]
                registry = registry_path(target)
                stored_path = KIT / entry["example"]
                before = {sha256(stored_path), sha256(manifest)}
                stored_hash = sha256(stored_path)
                registry_hash = sha256(registry)
                assert_created_identity(self, target, created, fixture, self.digest)
                wanted = expected_bytes(stored_path.read_bytes(), self.digest, created["effective_identity"])
                self.assertEqual(document.read_bytes(), wanted)
                report = check_documents(target, profile_root=KIT)
                self.assertTrue(report["ok"], report["diagnostics"])
                self.assertEqual(report["effective_identity_by_path"][created["path"]], created["effective_identity"])
                self.assertEqual(before, {sha256(stored_path), sha256(manifest)})
                self.assertEqual(stored_hash, sha256(stored_path))
                self.assertEqual(registry_hash, sha256(registry))
                evidence.append({
                    "template_id": entry["template_id"], "fixture_path": entry["fixture"],
                    "fixture_sha256": sha256(fixture_path), "manifest_sha256": manifest_hash,
                    "actual_contract_sha256": self.digest, "stored_example_sha256": stored_hash,
                    "created_sha256": created["sha256"], "registry_sha256": registry_hash,
                    "fictional": True, "engineering_acceptance": False,
                })
        (CACHE / "isolated-evidence.json").write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")

    def test_doc10_s10_mixed_example_exact_bytes_coverage_and_one_check(self) -> None:
        fixture_path = KIT / "catalog/mixed/fixture.json"
        fixture = load_json(fixture_path)
        with tempfile.TemporaryDirectory(dir=CACHE) as directory:
            target = Path(directory)
            manifest, manifest_hash = write_manifest(CACHE / "manifests", "mixed", self.digest, fixture_path)
            materialize_inputs(target, fixture["models"] + fixture["figures"])
            materialize_inputs(target, [fixture["coverage"]])
            records = []
            for document_fixture in fixture["documents"]:
                with self.subTest(template_id=document_fixture["template_id"]):
                    created = create_from_fixture(target, document_fixture, expected_digest=self.digest)
                    document = target / created["path"]
                    registry_hash = sha256(registry_path(target))
                    assert_created_identity(self, target, created, document_fixture, self.digest)
                    stored_path = KIT / document_fixture["expected_document"]
                    stored_hash = sha256(stored_path)
                    wanted = expected_bytes(stored_path.read_bytes(), self.digest, created["effective_identity"])
                    self.assertEqual(document.read_bytes(), wanted)
                    self.assertEqual(stored_hash, sha256(stored_path))
                    self.assertEqual(registry_hash, sha256(registry_path(target)))
                    records.append({"template_id": document_fixture["template_id"],
                                    "created_sha256": created["sha256"], "registry_sha256": registry_hash})
            classes = [document["document_class"] for document in fixture["documents"]]
            self.assertEqual(set(classes), {"requirements", "architecture", "design", "interface", "verification"})
            self.assertEqual(classes.count("architecture"), 2)
            coverage_text = (target / fixture["coverage"]["target"]).read_text()
            rows = {line.split("|")[1].strip(): line.split("|")[2].strip()
                    for line in coverage_text.splitlines() if line.startswith("|") and not line.startswith("| ---")}
            expected_rows = {
                "System goals and acceptance": "system-requirements",
                "External boundary": "system-context",
                "Internal composition": "component-composition",
                "Runtime ordering": "interaction-flow",
                "API operations": "operation-contract",
                "Verification planning": "verification-test-plan",
                "Lifecycle state detail": "Omitted: linear procedure with no meaningful state machine",
                "Data relationship detail": "Omitted: schema table is sufficient",
            }
            for concern, selection in expected_rows.items():
                self.assertEqual(rows.get(concern), selection)
            self.assertEqual({document["template_id"] for document in fixture["documents"]},
                             {selection for selection in expected_rows.values() if not selection.startswith("Omitted:")})
            report = check_documents(target, profile_root=KIT)
            self.assertTrue(report["ok"], report["diagnostics"])
            self.assertEqual(len(report["effective_identity_by_path"]), 6)
            evidence = {"documents": records, "fixture_sha256": sha256(fixture_path),
                        "manifest_sha256": manifest_hash, "actual_contract_sha256": self.digest,
                        "coverage": fixture["coverage"], "fictional": True, "engineering_acceptance": False}
            (CACHE / "mixed-evidence.json").write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")

    def test_s08_only_exact_one_field_full_byte_comparison_is_accepted(self) -> None:
        entry = self.entries[0]
        identity = {"profile_id": "xdesign-sysml-docs", "version": "1.1.0",
                    "document_class": entry["document_class"], "template_id": entry["template_id"]}
        stored = (KIT / entry["example"]).read_bytes()
        created = expected_bytes(stored, self.digest, identity)
        compare_created_bytes(stored, created, self.digest, identity)
        altered_marker = created.replace(b"digest-version=2", b"digest-version=1", 1)
        self.assertEqual(altered_marker.split(b"\n", 1)[1], created.split(b"\n", 1)[1])
        negatives = [
            created.split(b"\n", 1)[1], created + b"\n", created.replace(b"\n", b"\r\n"),
            created.replace(b"Revision: 1", b"Revision: 2", 1), altered_marker,
        ]
        for changed in negatives:
            with self.subTest(change=changed[:30]):
                with self.assertRaises(DocumentKitError) as rejected:
                    compare_created_bytes(stored, changed, self.digest, identity)
                self.assertEqual(rejected.exception.code, "fixture-mismatch")
        with_self_digest = stored + self.digest.encode("ascii")
        globally_normalized = with_self_digest.replace(self.digest.encode("ascii"), ZERO.encode("ascii"))
        self.assertNotEqual(globally_normalized, with_self_digest)
        with self.assertRaises(DocumentKitError) as rejected:
            compare_created_bytes(globally_normalized, created, self.digest, identity)
        self.assertEqual(rejected.exception.code, "fixture-mismatch")

    def test_s12_external_manifest_shape_and_stale_hashes_fail_before_creation(self) -> None:
        fixture_path = KIT / self.entries[0]["fixture"]
        for field in ("manifest_sha256", "self_sha256", "own_hash"):
            with self.subTest(field=field), tempfile.TemporaryDirectory(dir=CACHE) as directory:
                target = Path(directory)
                with self.assertRaises(DocumentKitError) as rejected:
                    write_manifest(target, "bad", self.digest, fixture_path, {field: "0" * 64})
                self.assertEqual(rejected.exception.code, "fixture-mismatch")
                self.assertFalse((target / "docs").exists())
        with tempfile.TemporaryDirectory(dir=CACHE) as directory:
            root = Path(directory)
            manifest, retained = write_manifest(root, "fixture", self.digest, fixture_path)
            fixture_hash = sha256(fixture_path)
            valid = (self.digest, self.entries[0]["fixture"], fixture_hash, retained)
            validate_manifest(manifest, *valid)
            for index in range(4):
                with self.subTest(stale=index):
                    values = list(valid)
                    values[index] = "f" * 64 if index != 1 else "catalog/fixtures/change-requirements.json"
                    with self.assertRaises(DocumentKitError) as rejected:
                        validate_manifest(manifest, *values)
                    self.assertEqual(rejected.exception.code, "fixture-mismatch")
            self.assertEqual(sha256(manifest), retained)

    def test_s12_valid_release_recomputes_and_propagates_actual_digest(self) -> None:
        with tempfile.TemporaryDirectory(dir=CACHE) as directory:
            root = Path(directory)
            release = changed_release(root / "release")
            digest = get_contract(release)["contract_sha256"]
            self.assertNotEqual(digest, self.digest)
            fixture = load_json(release / "catalog/fixtures/generic-requirements.json")
            target = root / "target"
            created = create_from_fixture(target, fixture, pack=release, expected_digest=digest)
            self.assertEqual(created["effective_identity"]["version"], "1.1.1")
            self.assertEqual(created["effective_identity"]["contract_sha256"], digest)
            registry = load_json(registry_path(target))["entries"][0]
            self.assertEqual(registry["contract_sha256"], digest)
            stored_path = release / fixture["expected_document"]
            stored = stored_path.read_bytes()
            self.assertEqual(stored.count(ZERO.encode("ascii")), 1)
            self.assertNotIn(digest.encode("ascii"), stored)
            wanted = expected_bytes(stored, digest, created["effective_identity"])
            self.assertEqual((target / created["path"]).read_bytes(), wanted)
            manifest, retained = write_manifest(root, "release", self.digest,
                                                release / "catalog/fixtures/generic-requirements.json", pack=release)
            with self.assertRaises(DocumentKitError) as rejected:
                validate_manifest(manifest, digest, "catalog/fixtures/generic-requirements.json",
                                  sha256(release / "catalog/fixtures/generic-requirements.json"), retained)
            self.assertEqual(rejected.exception.code, "fixture-mismatch")
            self.assertEqual(stored, stored_path.read_bytes())

    def test_doc11_diagram_alternatives_and_decision_failures(self) -> None:
        table_fixture = load_json(KIT / "catalog/fixtures/component-composition.json")
        with tempfile.TemporaryDirectory(dir=CACHE) as directory:
            created = create_from_fixture(Path(directory), table_fixture, expected_digest=self.digest)
            self.assertTrue(created["ok"])
        prose_fixture = load_json(KIT / "catalog/fixtures/interaction-flow.json")
        flow = prose_fixture["sections"]["flow"]
        flow["content"] = "The fictional interaction is explained as an ordered prose list without a figure."
        flow["fields"]["diagram_decisions"] = json.dumps([{
            "concern": "interaction-ordering", "applies": True, "choice": "prose",
            "rationale": "Two ordered steps are clearer as prose than as a figure.",
            "asset": None, "model_view": None,
        }])
        with tempfile.TemporaryDirectory(dir=CACHE) as directory:
            created = create_from_fixture(Path(directory), prose_fixture, expected_digest=self.digest)
            self.assertTrue(check_documents(Path(directory), profile_root=KIT)["ok"])
        original = load_json(KIT / "catalog/fixtures/system-context.json")
        for mode in ("missing", "empty-rationale", "duplicate", "inapplicable-figure", "malformed-block"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(dir=CACHE) as directory:
                fixture = copy.deepcopy(original)
                if mode == "malformed-block":
                    fixture["sections"]["boundary"]["content"] = re.sub(
                        r"\*\*Question:\*\*[^\n]*", "**Question:** ",
                        fixture["sections"]["boundary"]["content"], count=1)
                else:
                    decisions = json.loads(fixture["sections"]["boundary"]["fields"]["diagram_decisions"])
                    if mode == "missing":
                        decisions = []
                    elif mode == "empty-rationale":
                        decisions[0]["rationale"] = ""
                    elif mode == "duplicate":
                        decisions.append(copy.deepcopy(decisions[0]))
                    else:
                        decisions[0]["applies"] = False
                    fixture["sections"]["boundary"]["fields"]["diagram_decisions"] = json.dumps(decisions)
                with self.assertRaises(DocumentKitError) as rejected:
                    create_from_fixture(Path(directory), fixture, expected_digest=self.digest)
                expected = {"missing": "missing-diagram-decision", "malformed-block": "figure-block"}.get(
                    mode, "invalid-diagram-decision")
                self.assertEqual(rejected.exception.code, expected)
                self.assertFalse(registry_path(Path(directory)).exists())
    def test_doc15_guides_manifests_and_examples_have_scenario_substance(self) -> None:
        markers = {
            "system-requirements": "greenhouse",
            "change-requirements": "heat alert",
            "system-context": "dashed boundary",
            "component-composition": "RingBuffer",
            "interaction-flow": "ReadRequest",
            "state-machine": "probeFault",
            "data-model": "sensorId",
            "operation-contract": "getCurrentReading",
            "data-contract": "quality",
            "verification-test-plan": "20.2",
            "design-review": "independent reviewer",
        }
        for entry in self.entries:
            with self.subTest(template_id=entry["template_id"]):
                guide = (KIT / entry["guide"]).read_text()
                self.assertIn("Worked field:", guide)
                self.assertNotIn("is not the main question", guide)
                self.assertNotIn("is not the main question", " ".join(entry["when_not_to_use"]))
                self.assertNotIn("Scenario purpose", entry["required_inputs"])
                self.assertGreaterEqual(len(entry["dependencies"]), 2)
                fixture = load_json(KIT / entry["fixture"])
                self.assertNotIn("The fictional sensor project's", fixture["metadata"]["title"])
                for section in fixture["sections"].values():
                    self.assertNotIn("engineering concern without claiming a real result", section["content"])
                    self.assertNotIn("is described explicitly", section["content"])
                    self.assertGreater(len(section["content"]), 60)
                if entry["template_id"] in markers:
                    self.assertIn(markers[entry["template_id"]], json.dumps(fixture))

    def test_doc15_model_views_figure_blocks_and_svg_geometry_are_faithful(self) -> None:
        import xml.etree.ElementTree as ET
        model = (KIT / "catalog/mixed/mbse/model/example.sysml").read_text()
        self.assertIn("view def SystemContext", model)
        self.assertIn("view def InteractionFlow", model)
        for template_id, expected in (
            ("system-context", ["AsterHub boundary", "Operator", "WeatherService", "readings", "calibration", "forecast", "stroke-dasharray"]),
            ("interaction-flow", ["ReadRequest", "validate", "ReadingResponse", "TIMEOUT", "Controller", "Sensor"]),
        ):
            with self.subTest(template_id=template_id):
                fixture = load_json(KIT / f"catalog/fixtures/{template_id}.json")
                figure = fixture["figures"][0]
                svg_path = KIT / figure["pack"]
                root = ET.fromstring(svg_path.read_bytes())
                visible = svg_path.read_text()
                for value in expected:
                    self.assertIn(value, visible)
                section = next(section for section in fixture["sections"].values()
                               if "diagram_decisions" in section["fields"])
                self.assertIn(figure["Model view"], section["content"])
                self.assertIn(figure["Reading"], section["content"])
                self.assertIn(figure["Question"], section["content"])
                self.assertIn(figure["Limits"], section["content"])
                self.assertGreaterEqual(int(root.attrib["width"]), 600)
                self.assertIn("font-size=\"12\"", visible)
        self.assertNotEqual((KIT / "catalog/mixed/assets/system-context.svg").read_bytes(),
                            (KIT / "catalog/mixed/assets/interaction-flow.svg").read_bytes())


if __name__ == "__main__":
    unittest.main()
