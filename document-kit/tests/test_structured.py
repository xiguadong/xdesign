"""SDK-01..13 behavior, byte preservation and actual consumer contracts."""

from __future__ import annotations

import copy
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

KIT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(KIT))
from xdesign_doc_kit import *
from xdesign_doc_kit import core
from xdesign_doc_kit.catalog import load_catalog_contract
from xdesign_doc_kit.compatibility import marker, registry_entry

CACHE = Path(
    os.environ.get(
        "XDESIGN_DOC_KIT_TEST_CACHE", KIT.parent / "cache/document-kit-structured-tests"
    )
)
CACHE.mkdir(parents=True, exist_ok=True)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def readj(path):
    return json.loads(path.read_text())


def writej(path, data):
    path.write_text(json.dumps(data, indent=2) + "\n")


def snap(root):
    return {
        p.relative_to(root).as_posix(): p.read_bytes()
        for p in root.rglob("*")
        if p.is_file()
        and not p.is_symlink()
        and "cache" not in p.relative_to(root).parts
    }


def fixture(cls="api-design", pack=KIT):
    return readj(pack / f"catalog/fixtures/generic-{cls}.json")


def materialize(target, f, pack=KIT):
    for item in f.get("models", []) + f.get("figures", []):
        p = target / item["target"]
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes((pack / item["pack"]).read_bytes())


def create(target, f, pack=KIT):
    return create_document(
        target,
        f["document_class"],
        f["name"],
        f["metadata"],
        f["sections"],
        pack,
        template_id=f["template_id"],
        layer=f.get("layer"),
    )


def selection(f):
    return {k: f.get(k) for k in ("document_class", "name", "layer")}


def figure(target, path):
    asset = target / Path(path).parent / "assets/context.svg"
    asset.parent.mkdir(exist_ok=True)
    asset.write_text(
        '<svg xmlns="http://www.w3.org/2000/svg"><title>Fictional boundary</title></svg>\n'
    )
    return {
        "asset": "assets/context.svg",
        "title": "Fictional boundary",
        "Question": "What is the boundary?",
        "Model view": "ExampleContext",
        "Companions": "The contract prose.",
        "Status": "external-tool; designed-only",
        "Reading": "Only the boundary is named.",
        "Limits": "No behavior or execution is shown.",
    }


def error(test, code, fn, *args, **kwargs):
    with test.assertRaises(DocumentKitError) as caught:
        fn(*args, **kwargs)
    test.assertEqual(caught.exception.code, code, caught.exception.as_dict())
    return caught.exception


def derived(path):
    shutil.copytree(KIT, path)
    for p in path.rglob("*"):
        if (
            p.is_file()
            and "compatibility" not in p.relative_to(path).parts
            and p.suffix in (".json", ".md")
        ):
            p.write_bytes(
                p.read_bytes().replace(b"xdesign-sysml-docs", b"derived-docs")
            )
    p = readj(path / "profile.json")
    p["compatibility_packs"] = []
    writej(path / "profile.json", p)
    return path


class StructuredTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=CACHE)
        self.root = Path(self.tmp.name)
        self.target = self.root / "project"
        self.target.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def prepare(self, cls="api-design"):
        f = fixture(cls)
        materialize(self.target, f)
        return f

    def assert_preserved(self, before):
        self.assertEqual(snap(self.target), before)

    def test_sdk01_07_all_examples_exact_bytes_and_mixed_scope(self):
        contract = get_contract()
        self.assertEqual(len(contract["catalog"]["entries"]), 21)
        evidence = []
        for entry in contract["catalog"]["entries"]:
            f = readj(KIT / entry["fixture"])
            project = self.root / entry["template_id"]
            project.mkdir()
            before = snap(project)
            plan = plan_document(
                project,
                f["document_class"],
                f["name"],
                layer=f["layer"],
                template_id=f["template_id"],
            )
            self.assertEqual(snap(project), before)
            self.assertEqual(
                plan["required_metadata"],
                ["title", "status", "revision", "model_sources"],
            )
            materialize(project, f)
            # A landing page has real navigation dependencies supplied by the caller.
            if f["document_class"] == "flow-index":
                for role in ("definition", "implementation", "views"):
                    p = project / f'docs/design/flows/{f["name"]}/{role}.md'
                    p.parent.mkdir(parents=True, exist_ok=True)
                    p.write_text("# Fictional " + role + "\n")
            manifest = {
                "expected_contract_sha256": contract["contract_sha256"],
                "fixture_path": entry["fixture"],
                "fixture_sha256": sha((KIT / entry["fixture"]).read_bytes()),
            }
            mpath = CACHE / (entry["template_id"] + "-manifest.json")
            writej(mpath, manifest)
            created = create(project, f)
            data = (project / created["path"]).read_bytes()
            stored = (KIT / entry["example"]).read_bytes()
            self.assertEqual(
                data.split(b"\n")[0].decode(), marker(created["effective_identity"])
            )
            registry = readj(project / "docs/.document-kit-managed.json")
            self.assertIn(
                registry_entry(created["effective_identity"], created["path"]),
                registry["entries"],
            )
            first, body = stored.split(b"\n", 1)
            self.assertEqual(stored.count(b"0" * 64), 1)
            expected = (
                first.replace(
                    b"contract=" + b"0" * 64,
                    b"contract=" + contract["contract_sha256"].encode(),
                    1,
                )
                + b"\n"
                + body
            )
            self.assertEqual(data, expected)
            self.assertEqual(created["sha256"], sha(data))
            self.assertTrue(check_documents(project)["ok"])
            evidence.append(
                dict(
                    template=entry["template_id"],
                    path=created["path"],
                    sha256=sha(data),
                    manifest_sha256=sha(mpath.read_bytes()),
                )
            )
        writej(CACHE / "structured-examples-evidence.json", evidence)
        mixed = readj(KIT / "catalog/mixed/fixture.json")
        materialize(self.target, mixed)
        coverage = self.target / mixed["coverage"]["target"]
        coverage.parent.mkdir(parents=True, exist_ok=True)
        coverage.write_bytes((KIT / mixed["coverage"]["pack"]).read_bytes())
        mixed_manifest = {
            "expected_contract_sha256": contract["contract_sha256"],
            "fixture_path": "catalog/mixed/fixture.json",
            "fixture_sha256": sha((KIT / "catalog/mixed/fixture.json").read_bytes()),
        }
        manifest_path = CACHE / "structured-mixed-manifest.json"
        writej(manifest_path, mixed_manifest)
        mixed_evidence = []
        for f in mixed["documents"]:
            result = create(self.target, f)
            stored = (KIT / f["expected_document"]).read_bytes()
            first, body = stored.split(b"\n", 1)
            self.assertEqual(stored.count(b"0" * 64), 1)
            expected = (
                first.replace(
                    b"contract=" + b"0" * 64,
                    b"contract=" + contract["contract_sha256"].encode(),
                    1,
                )
                + b"\n"
                + body
            )
            created = (self.target / result["path"]).read_bytes()
            self.assertEqual(created, expected)
            self.assertEqual(
                created.split(b"\n")[0].decode(), marker(result["effective_identity"])
            )
            self.assertIn(
                registry_entry(result["effective_identity"], result["path"]),
                readj(self.target / "docs/.document-kit-managed.json")["entries"],
            )
            mixed_evidence.append(
                dict(
                    path=result["path"],
                    sha256=sha(created),
                    stored_sha256=sha(stored),
                    effective_identity=result["effective_identity"],
                )
            )
        report = check_documents(self.target)
        self.assertTrue(report["ok"], report)
        self.assertEqual(len(report["checked_paths"]), 12)
        self.assertEqual(len(report["effective_identity_by_path"]), 11)
        writej(
            CACHE / "structured-mixed-evidence.json",
            dict(
                manifest_sha256=sha(manifest_path.read_bytes()),
                documents=mixed_evidence,
                report=report,
            ),
        )
        for cls in (
            "api-design",
            "flow-index",
            "flow-definition",
            "flow-implementation",
            "flow-views",
        ):
            r = check_documents(self.target, cls)
            self.assertTrue(r["ok"], r)
            self.assertEqual(len(r["checked_paths"]), 1)

    def test_sdk02_every_role_update_attach_and_stale_preserve_identity(self):
        documents = []
        for cls, layer, name in [
            ("api-design", "core", "create-run"),
            ("api-design", "storage", "rs-store-save"),
            ("flow-definition", None, "simple-flow"),
            ("flow-implementation", None, "simple-flow"),
            ("flow-views", None, "simple-flow"),
            ("flow-index", None, "simple-flow"),
        ]:
            f = self.prepare(cls)
            f.update(layer=layer, name=name)
            if cls == "api-design":
                f["sections"]["purpose"]["fields"]["layer"] = layer
                if layer == "storage":
                    for s in f["sections"].values():
                        s["anchor"] = "U" + str(int(s["anchor"][1:]) + 100)
                    f["sections"]["purpose"]["fields"].update(
                        api_identity="Example::RunId",
                        api_kind="type",
                        purpose="Identify one run.",
                    )
                    f["sections"]["contract"]["fields"].update(
                        declaration="`struct RunId(String)`",
                        inputs="Not applicable: this type has no callable inputs.",
                        outputs="Not applicable: values are created by a separate factory.",
                    )
            created = create(self.target, f)
            documents.append((f, created))
        for f, created in documents:
            cls, name, layer = f["document_class"], f["name"], f["layer"]
            key = next(iter(f["sections"]))
            path = self.target / created["path"]
            registry = (self.target / "docs/.document-kit-managed.json").read_bytes()
            old = path.read_bytes()
            current = created["sha256"]
            update = update_section(
                self.target,
                cls,
                name,
                key,
                "Caller revised the boundary.",
                current,
                layer=layer,
            )
            self.assertEqual(
                (self.target / "docs/.document-kit-managed.json").read_bytes(), registry
            )
            self.assertEqual(old.split(b"\n")[0], path.read_bytes().split(b"\n")[0])
            self.assertEqual(
                created["effective_identity"], update["effective_identity"]
            )
            fig = figure(self.target, created["path"])
            before = snap(self.target)
            error(
                self,
                "stale-document",
                attach_figure,
                self.target,
                cls,
                name,
                key,
                fig,
                current,
                layer=layer,
                concern="unknown",
            )
            self.assert_preserved(before)
            error(
                self,
                "invalid-diagram-decision",
                attach_figure,
                self.target,
                cls,
                name,
                key,
                fig,
                update["sha256"],
                layer=layer,
                concern="unknown",
            )
            self.assert_preserved(before)
            attached = attach_figure(
                self.target, cls, name, key, fig, update["sha256"], layer=layer
            )
            self.assertIn(attached["figure_number"], path.read_text())
            self.assertEqual(
                (self.target / "docs/.document-kit-managed.json").read_bytes(), registry
            )
            self.assertTrue(check_documents(self.target, selection=selection(f))["ok"])
        # Same API basename in a second layer is a separate owner.
        f = self.prepare()
        f["layer"] = "storage"
        f["sections"]["purpose"]["fields"]["layer"] = "storage"
        for s in f["sections"].values():
            s["anchor"] = "U" + str(int(s["anchor"][1:]) + 300)
        self.assertEqual(
            create(self.target, f)["path"], "docs/design/api/storage/create-run.md"
        )
        self.assertTrue(check_documents(self.target)["ok"])

    def test_sdk03_routes_collisions_and_symlinks(self):
        f = self.prepare()
        original = snap(self.target)
        for name in (
            "",
            "Upper",
            "under_score",
            "slash/name",
            "back\\slash",
            ".",
            "..",
            "../outside",
            "/absolute",
            "index",
        ):
            for fn in (
                lambda: plan_document(self.target, "api-design", name, layer="core"),
                lambda: create_document(
                    self.target,
                    "api-design",
                    name,
                    f["metadata"],
                    f["sections"],
                    layer="core",
                ),
            ):
                error(self, "invalid-document-name", fn)
                self.assert_preserved(original)
        for layer in ("", "Upper", "under_score", "bad/name", "..", "unknown", None):
            error(
                self,
                "invalid-request",
                plan_document,
                self.target,
                "api-design",
                "name",
                layer=layer,
            )
        error(
            self,
            "invalid-request",
            plan_document,
            self.target,
            "flow-index",
            "flow",
            layer="core",
        )
        self.assertEqual(
            plan_document(self.target, "flow-index", "flow")["path"],
            "docs/design/flows/flow/index.md",
        )
        outside = self.root / "outside"
        outside.mkdir()
        (outside / "sentinel").write_bytes(b"protected")
        for dest in (outside, self.target / "mbse/model"):
            (self.target / "docs").symlink_to(dest, target_is_directory=True)
            error(self, "unsafe-path", create, self.target, f)
            (self.target / "docs").unlink()
        self.assertEqual((outside / "sentinel").read_bytes(), b"protected")
        created = create(self.target, f)
        before = snap(self.target)
        error(self, "document-exists", create, self.target, f)
        self.assert_preserved(before)
        (self.target / created["path"]).unlink()
        before = snap(self.target)
        error(self, "registry-mismatch", create, self.target, f)
        self.assert_preserved(before)

    def test_sdk04_candidates_model_context_links_layer_and_u_anchors(self):
        f = self.prepare()
        model = self.target / f["metadata"]["model_sources"][0]
        model.write_text(
            'package Context { @DocSection { document = "docs/design/api/core/create-run.md"; section = "U2000"; } }\n'
        )
        created = create(self.target, f)
        self.assertTrue(check_documents(self.target)["ok"])
        before = snap(self.target)
        for code, content, fields in [
            ("unresolved-placeholder", "TODO", None),
            ("invalid-link", "[missing](missing.md)", None),
            ("invalid-request", "New prose", {"layer": "storage"}),
        ]:
            error(
                self,
                code,
                update_section,
                self.target,
                "api-design",
                "create-run",
                "purpose",
                content,
                created["sha256"],
                fields,
                layer="core",
            )
            self.assert_preserved(before)
        for change, expected in [
            ("anchor", "missing-anchor"),
            ("field", "missing-field"),
            ("content", "empty-content"),
        ]:
            other = copy.deepcopy(f)
            other["name"] = "other"
            other["sections"]["purpose"][
                {"anchor": "anchor", "field": "fields", "content": "content"}[change]
            ] = (
                {"purpose": "incomplete"}
                if change == "field"
                else "D42" if change == "anchor" else ""
            )
            # Avoid unrelated duplicate anchors, so the predicate under test is visible.
            for sec in other["sections"].values():
                sec["anchor"] = "U" + str(int(sec["anchor"][1:]) + 500)
            if change == "anchor":
                other["sections"]["purpose"]["anchor"] = "D42"
            error(self, expected, create, self.target, other)
            self.assert_preserved(before)
        model.write_text(model.read_text().replace("U2000", "U99999"))
        before = snap(self.target)
        error(
            self,
            "invalid-docsection",
            update_section,
            self.target,
            "api-design",
            "create-run",
            "purpose",
            "Changed.",
            created["sha256"],
            layer="core",
        )
        self.assert_preserved(before)

    def test_sdk10_unmanaged_and_orphan_are_not_adopted(self):
        f = self.prepare()
        path = self.target / "docs/design/api/core/create-run.md"
        path.parent.mkdir(parents=True)
        path.write_text("# Ordinary page\n")
        before = snap(self.target)
        p = plan_document(self.target, "api-design", "create-run", layer="core")
        self.assertTrue(p["exists"])
        self.assertEqual(p["sha256"], sha(path.read_bytes()))
        self.assertTrue(check_documents(self.target)["ok"])
        error(self, "document-exists", create, self.target, f)
        error(
            self,
            "registry-mismatch",
            check_documents,
            self.target,
            selection=selection(f),
        )
        error(
            self,
            "registry-mismatch",
            update_section,
            self.target,
            "api-design",
            "create-run",
            "purpose",
            "text",
            p["sha256"],
            layer="core",
        )
        self.assert_preserved(before)
        path.unlink()
        r = create(self.target, f)
        (self.target / "docs/.document-kit-managed.json").unlink()
        self.assertIn(
            "registry-mismatch",
            {d["code"] for d in check_documents(self.target)["diagnostics"]},
        )

    def test_sdk11_selected_collision_context_and_class_filters(self):
        f = self.prepare()
        created = create(self.target, f)

        def selected():
            return check_documents(self.target, selection=selection(f))

        for path in ("docs/aaa.md", "docs/zzz.md"):
            p = self.target / path
            p.write_text('<a id="U2000"></a>\n# Duplicate\n')
            self.assertIn(
                "duplicate-anchor", {d["code"] for d in selected()["diagnostics"]}
            )
            p.unlink()
        for name in ("a", "b"):
            (self.target / f"docs/{name}.md").write_text(
                '<a id="R123"></a>\n# Unrelated\n'
            )
        self.assertTrue(selected()["ok"])
        self.assertFalse(check_documents(self.target)["ok"])
        (self.target / "docs/b.md").write_text(
            '```html\n<a id="U2000"></a>\n```\n# Fenced\n'
        )
        self.assertTrue(selected()["ok"])
        model = self.target / f["metadata"]["model_sources"][0]
        model.write_text(
            'package C { @DocSection { document = "docs/no.md"; section = "R9"; } }'
        )
        self.assertTrue(selected()["ok"])
        self.assertFalse(check_documents(self.target, "api-design")["ok"])
        model.write_text("package C { @DocSection { invalid; } }")
        self.assertFalse(selected()["ok"])
        # A copied foreign marker does not make unrelated selected checks claim its identity.
        (self.target / "docs/a.md").write_text(
            "<!-- document-kit: profile=foreign@9 -->\n# Foreign\n"
        )
        model.write_text("package C {}")
        self.assertTrue(selected()["ok"])

    def test_sdk05_11_exact_retained_closures_coexist_and_update(self):
        packs = [KIT / "compatibility/v1.0.0", KIT / "compatibility/v1.1.0", KIT]
        expected = [
            "1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6",
            "e2b827c6e929f4a8d03344a91ca72f2eb837962e916d5e7f0487e315e1440cef",
        ]
        results = []
        for i, pack in enumerate(packs):
            if i < 2:
                self.assertEqual(get_contract(pack)["contract_sha256"], expected[i])
            f = fixture("requirements", KIT)
            materialize(self.target, f)
            f["name"] = "release-" + str(i)
            for s in f["sections"].values():
                s["anchor"] = "R" + str(int(s["anchor"][1:]) + i * 100)
            result = create_document(
                self.target,
                "requirements",
                f["name"],
                f["metadata"],
                f["sections"],
                pack,
            )
            results.append((f, result))
        report = check_documents(self.target)
        self.assertTrue(report["ok"], report)
        registry = (self.target / "docs/.document-kit-managed.json").read_bytes()
        for i, (f, result) in enumerate(results):
            r = check_documents(self.target, selection=selection(f))
            self.assertTrue(r["ok"], r)
            self.assertEqual(
                r["effective_identity_by_path"][result["path"]],
                result["effective_identity"],
            )
            updated = update_section(
                self.target,
                "requirements",
                f["name"],
                "purpose",
                "Updated through 1.2.",
                result["sha256"],
            )
            self.assertEqual(
                updated["effective_identity"], result["effective_identity"]
            )
            self.assertEqual(
                (self.target / "docs/.document-kit-managed.json").read_bytes(), registry
            )
            if i < 2:
                old = update_section(
                    self.target,
                    "requirements",
                    f["name"],
                    "purpose",
                    "Updated by original pack.",
                    updated["sha256"],
                    profile_root=packs[i],
                )
                self.assertEqual(
                    old["effective_identity"], result["effective_identity"]
                )
        # A corrupted unrelated identity is omitted only by selected checking.
        data = readj(self.target / "docs/.document-kit-managed.json")
        data["entries"][0]["contract_sha256"] = "f" * 64
        writej(self.target / "docs/.document-kit-managed.json", data)
        self.assertFalse(check_documents(self.target)["ok"])
        self.assertTrue(
            check_documents(self.target, selection=selection(results[2][0]))["ok"]
        )

    def test_sdk12_per_role_model_metadata_rejections(self):
        for cls in (
            "api-design",
            "flow-index",
            "flow-definition",
            "flow-implementation",
            "flow-views",
        ):
            f = self.prepare(cls)
            before = snap(self.target)
            if cls == "flow-index":
                f["sections"]["scope"]["fields"][
                    "navigation_links"
                ] = "Navigation will be allocated after the owning pages exist."
            for value in (
                None,
                [],
                ["missing.sysml"],
                ["/absolute.sysml"],
                ["../outside.sysml"],
                ["mbse/model/wrong.txt"],
                ["mbse/model/missing.sysml"],
                ["mbse/model/example.sysml"] * 2,
                [12],
            ):
                bad = copy.deepcopy(f)
                bad["metadata"]["model_sources"] = value
                with self.assertRaises(DocumentKitError):
                    create(self.target, bad)
                self.assert_preserved(before)
            for key in ("title", "status", "revision"):
                bad = copy.deepcopy(f)
                bad["metadata"][key] = ""
                error(self, "missing-metadata", create, self.target, bad)
                self.assert_preserved(before)
            link = self.target / "mbse/model/link.sysml"
            link.symlink_to(self.target / "mbse/model/example.sysml")
            with self.assertRaises(DocumentKitError):
                create(self.target, f)
            link.unlink()
            self.assert_preserved(before)
            r = create(self.target, f)
            path = self.target / r["path"]
            model = self.target / "mbse/model/example.sysml"
            model.unlink()
            (self.target / "mbse/model/another.sysml").write_text("package Another {}")
            before = snap(self.target)
            error(
                self,
                "missing-model-source",
                update_section,
                self.target,
                cls,
                f["name"],
                next(iter(f["sections"])),
                "Revised.",
                r["sha256"],
                layer=f["layer"],
            )
            self.assert_preserved(before)
            self.assertFalse(check_documents(self.target, selection=selection(f))["ok"])
            path.unlink()
            (self.target / "docs/.document-kit-managed.json").unlink()
            (self.target / "mbse/model/another.sysml").unlink()

    def test_sdk03_11_namespace_and_nested_sentinel(self):
        pack = self.root / "pack"
        shutil.copytree(KIT, pack)
        p = readj(pack / "profile.json")
        p["compatibility_packs"][1]["contract_sha256"] = "0" * 64
        writej(pack / "profile.json", p)
        error(self, "contract-drift", get_contract, pack)
        p = readj(KIT / "profile.json")
        writej(pack / "profile.json", p)
        child = pack / "compatibility/v1.1.0/catalog/examples/generic-design.md"
        child.write_bytes(child.read_bytes() + b"0" * 64)
        error(self, "invalid-catalog", get_contract, pack)
        shutil.copyfile(
            KIT / "compatibility/v1.1.0/catalog/examples/generic-design.md", child
        )
        p["project"]["docs_root"] = "manual"
        for spec in p["document_classes"].values():
            spec["path_template"] = spec["path_template"].replace("docs/", "manual/", 1)
        writej(pack / "profile.json", p)
        error(self, "invalid-profile", get_contract, pack)
        p = readj(KIT / "profile.json")
        p["document_classes"]["flow-definition"]["path_template"] = p[
            "document_classes"
        ]["flow-views"]["path_template"]
        writej(pack / "profile.json", p)
        error(self, "invalid-profile", get_contract, pack)
        with patch("xdesign_doc_kit.catalog.digest_v2", return_value="0" * 64):
            error(self, "invalid-catalog", get_contract, KIT)

    def test_sdk02_optional_decision_lifecycle(self):
        pack = derived(self.root / "pack")
        p = readj(pack / "profile.json")
        m = readj(pack / "catalog/manifest.json")
        e = next(e for e in m["entries"] if e["template_id"] == "generic-api-design")
        e["required_sections"][0]["required_fields"].append("diagram_decisions")
        p["document_classes"]["api-design"]["required_sections"] = copy.deepcopy(
            e["required_sections"]
        )
        e["view_concerns"] = [
            dict(
                concern="boundary",
                section_key="purpose",
                applicability="optional",
                trigger="When the author needs a visual boundary.",
                alternatives=["figure", "prose", "table", "not-applicable"],
            )
        ]
        t = pack / e["template"]
        t.write_text(
            t.read_text().replace(
                "{{content:purpose}}",
                "**Diagram decisions:** {{field:diagram_decisions}}\n\n{{content:purpose}}",
            )
        )
        for file in [pack / e["fixture"], pack / "catalog/mixed/fixture.json"]:
            d = readj(file)
            for f in d.get("documents", [d]):
                if f["document_class"] == "api-design":
                    f["sections"]["purpose"]["fields"]["diagram_decisions"] = "[]"
            writej(file, d)
        writej(pack / "profile.json", p)
        writej(pack / "catalog/manifest.json", m)
        f = fixture(pack=pack)
        materialize(self.target, f, pack)
        r = create(self.target, f, pack)
        fig = figure(self.target, r["path"])
        before = snap(self.target)
        error(
            self,
            "missing-diagram-decision",
            attach_figure,
            self.target,
            "api-design",
            f["name"],
            "purpose",
            fig,
            r["sha256"],
            pack,
            layer="core",
            concern="boundary",
        )
        self.assert_preserved(before)
        decision = dict(
            concern="boundary",
            applies=True,
            choice="prose",
            rationale="Prose suffices.",
            asset=None,
            model_view=None,
        )
        r = update_section(
            self.target,
            "api-design",
            f["name"],
            "purpose",
            "Caller prose.",
            r["sha256"],
            {"diagram_decisions": json.dumps([decision])},
            pack,
            layer="core",
        )
        for choice in ("prose", "table", "not-applicable"):
            decision.update(choice=choice, applies=choice != "not-applicable")
            r = update_section(
                self.target,
                "api-design",
                f["name"],
                "purpose",
                "Caller prose.",
                r["sha256"],
                {"diagram_decisions": json.dumps([decision])},
                pack,
                layer="core",
            )
            before = snap(self.target)
            error(
                self,
                "invalid-diagram-decision",
                attach_figure,
                self.target,
                "api-design",
                f["name"],
                "purpose",
                fig,
                r["sha256"],
                pack,
                layer="core",
                concern="boundary",
            )
            self.assert_preserved(before)
        r = update_section(
            self.target,
            "api-design",
            f["name"],
            "purpose",
            "Caller prose.",
            r["sha256"],
            {"diagram_decisions": "[]"},
            pack,
            layer="core",
        )
        decision.update(
            choice="figure",
            applies=True,
            rationale="Show the context boundary.",
            asset=fig["asset"],
            model_view=fig["Model view"],
        )
        r = attach_figure(
            self.target,
            "api-design",
            f["name"],
            "purpose",
            fig,
            r["sha256"],
            pack,
            layer="core",
            concern="boundary",
            decision=decision,
        )
        before = snap(self.target)
        error(
            self,
            "invalid-diagram-decision",
            update_section,
            self.target,
            "api-design",
            f["name"],
            "purpose",
            "Caller prose.",
            r["sha256"],
            {"diagram_decisions": "[]"},
            pack,
            layer="core",
        )
        self.assert_preserved(before)
        decision["rationale"] = "Revised caller rationale."
        r = update_section(
            self.target,
            "api-design",
            f["name"],
            "purpose",
            "Caller prose.",
            r["sha256"],
            {"diagram_decisions": json.dumps([decision])},
            pack,
            layer="core",
        )
        self.assertTrue(check_documents(self.target, profile_root=pack)["ok"])

    def test_sdk11_positive_custom_binding_and_effective_model_roots(self):
        # N2: a separately verified custom 1.1 release is bound exactly, without
        # changing original pack interpretation or silently adopting its pages.
        parent = derived(self.root / "parent")
        child = parent / "compatibility/custom"
        shutil.copytree(KIT / "compatibility/v1.1.0", child)
        for path in child.rglob("*"):
            if (
                path.is_file()
                and "compatibility" not in path.relative_to(child).parts
                and path.suffix in (".json", ".md")
            ):
                path.write_bytes(
                    path.read_bytes().replace(b"xdesign-sysml-docs", b"custom-docs")
                )
        child_profile = readj(child / "profile.json")
        child_profile["compatibility_packs"] = []
        child_profile["project"]["model_roots"] = ["mbse/custom"]
        child_profile["project"]["optional_model_roots"] = []
        writej(child / "profile.json", child_profile)
        identity = core.get_contract(child)
        parent_profile = readj(parent / "profile.json")
        parent_profile["compatibility_packs"] = [
            dict(
                profile_id=identity["profile_id"],
                version=identity["version"],
                root="compatibility/custom",
                contract_sha256=identity["contract_sha256"],
                contract_digest_version=identity["contract_digest_version"],
            )
        ]
        writej(parent / "profile.json", parent_profile)
        f = fixture("requirements", child)
        f["metadata"]["model_sources"] = ["mbse/custom/context.sysml"]
        model = self.target / "mbse/custom/context.sysml"
        model.parent.mkdir(parents=True)
        model.write_text("package CustomContext {}")
        r = create_document(
            self.target, "requirements", f["name"], f["metadata"], f["sections"], child
        )
        materialize(self.target, fixture())
        before = (self.target / "docs/.document-kit-managed.json").read_bytes()
        self.assertTrue(check_documents(self.target, profile_root=parent)["ok"])
        report = check_documents(
            self.target, profile_root=parent, selection=selection(f)
        )
        self.assertEqual(
            report["effective_identity_by_path"][r["path"]], r["effective_identity"]
        )
        updated = update_section(
            self.target,
            "requirements",
            f["name"],
            "purpose",
            "Caller revision.",
            r["sha256"],
            profile_root=parent,
        )
        self.assertEqual(updated["effective_identity"], r["effective_identity"])
        self.assertEqual(
            (self.target / "docs/.document-kit-managed.json").read_bytes(), before
        )
        model.unlink()
        error(
            self,
            "missing-model-root",
            check_documents,
            self.target,
            profile_root=parent,
            selection=selection(f),
        )

    def test_sdk03_scopes_offroute_registry_and_inside_symlinks(self):
        f = self.prepare()
        r = create(self.target, f)
        path = self.target / r["path"]
        # An off-route registered path is still included by its recorded class.
        wrong = path.with_name("other.md")
        path.rename(wrong)
        registry = readj(self.target / "docs/.document-kit-managed.json")
        registry["entries"][0]["path"] = wrong.relative_to(self.target).as_posix()
        writej(self.target / "docs/.document-kit-managed.json", registry)
        text = wrong.read_text().replace("**Layer:** core", "**Layer:** storage")
        wrong.write_text(text)
        report = check_documents(self.target, "api-design")
        self.assertEqual(report["checked_paths"], ["docs/design/api/core/other.md"])
        self.assertIn("invalid-request", {d["code"] for d in report["diagnostics"]})
        wrong.rename(path)
        registry["entries"][0]["path"] = r["path"]
        writej(self.target / "docs/.document-kit-managed.json", registry)
        elsewhere = self.target / "copy.md"
        path.rename(elsewhere)
        path.symlink_to(elsewhere)
        error(
            self,
            "unsafe-path",
            plan_document,
            self.target,
            "api-design",
            f["name"],
            layer="core",
        )
        error(self, "unsafe-path", check_documents, self.target, selection=selection(f))
        path.unlink()
        elsewhere.rename(path)
        pack = self.root / "pack"
        shutil.copytree(KIT, pack)
        resource = pack / "catalog/templates/generic-api-design.md"
        resource.rename(resource.with_suffix(".saved"))
        resource.symlink_to(resource.with_suffix(".saved"))
        with self.assertRaises(DocumentKitError):
            get_contract(pack)

    def test_sdk04_selected_same_file_duplicate_and_link_target_context(self):
        f = self.prepare()
        r = create(self.target, f)
        path = self.target / r["path"]
        original = path.read_bytes()
        path.write_bytes(original + b'\n<a id="U2000"></a>\n')
        report = check_documents(self.target, selection=selection(f))
        self.assertIn("duplicate-anchor", {d["code"] for d in report["diagnostics"]})
        path.write_bytes(original)
        (self.target / "docs/context.md").write_text('# Context\n<a id="D90"></a>\n')
        before = snap(self.target)
        error(
            self,
            "invalid-link",
            update_section,
            self.target,
            "api-design",
            f["name"],
            "purpose",
            "[Wrong file](../../../context.md#U2000)",
            r["sha256"],
            layer="core",
        )
        self.assert_preserved(before)
        # A malformed registry aborts the check; it is not a partial report.
        (self.target / "docs/.document-kit-managed.json").write_text("{broken")
        error(
            self,
            "registry-mismatch",
            check_documents,
            self.target,
            selection=selection(f),
        )


if __name__ == "__main__":
    unittest.main()
