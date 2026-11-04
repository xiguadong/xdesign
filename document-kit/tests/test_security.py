from __future__ import annotations

import copy
import errno
import inspect
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from _support import KIT, REPO, copy_pack, create_from_fixture, load_json, materialize_inputs, registry_path, sha256
from xdesign_doc_kit.compatibility import DocumentKitError
from xdesign_doc_kit.core import check_documents, create_document, get_contract, plan_document, update_section

CACHE = REPO / "cache/document-kit-security-tests"
ZERO = "0" * 64


def codes(report: dict) -> set[str]:
    return {item["code"] for item in report["diagnostics"]}


class SecurityTests(unittest.TestCase):
    def setUp(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=CACHE)
        self.root = Path(self.temporary.name)
        self.fixture = load_json(KIT / "catalog/fixtures/generic-requirements.json")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def target(self, name="target") -> Path:
        target = self.root / name
        target.mkdir(parents=True, exist_ok=True)
        materialize_inputs(target, self.fixture["models"] + self.fixture["figures"])
        return target

    def test_doc14_unsafe_markdown_urls_and_html_fail_before_writes(self) -> None:
        cases = {
            "script": ("<script>alert(1)</script>", "unsafe-markdown"),
            "event": ('<a id="R999" onclick="run()"></a>', "unsafe-markdown"),
            "html": ("<iframe src=\"https://example.com\"></iframe>", "unsafe-markdown"),
            "javascript": ("[bad](javascript:alert(1))", "unsafe-url"),
            "file": ("[bad](file:///etc/passwd)", "unsafe-url"),
            "data": ("[bad](data:text/plain,hello)", "unsafe-url"),
            "protocol-relative": ("[bad](//example.com/source)", "unsafe-url"),
            "credentials": ("[bad](https://user:secret@example.com/source)", "unsafe-url"),
            "control": ("[bad](https://example.com/\nsource)", "unsafe-url"),
            "absolute": ("[bad](/etc/passwd)", "unsafe-path"),
            "backslash": ("[bad](..\\outside.md)", "unsafe-path"),
        }
        for mode, (addition, expected) in cases.items():
            with self.subTest(mode=mode):
                fixture = copy.deepcopy(self.fixture)
                fixture["sections"]["purpose"]["content"] += "\n" + addition
                target = self.target(mode)
                with self.assertRaises(DocumentKitError) as rejected:
                    create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
                self.assertEqual(rejected.exception.code, expected, rejected.exception.diagnostics)
                self.assertFalse(registry_path(target).exists())

    def test_doc14_catalog_markdown_and_license_sentinel_values(self) -> None:
        mutations = [
            ("html", "unsafe-markdown"), ("url", "unsafe-url"),
            ("unknown-license-url", "invalid-license"), ("unlicensed-license-url", "invalid-license"),
        ]
        for mode, expected in mutations:
            with self.subTest(mode=mode):
                pack = copy_pack(self.root / mode)
                if mode in ("html", "url"):
                    guide = pack / "catalog/guides/generic-design.md"
                    addition = "<script>alert(1)</script>" if mode == "html" else "[bad](javascript:alert(1))"
                    guide.write_bytes(guide.read_bytes() + b"\n" + addition.encode() + b"\n")
                else:
                    sources_path = pack / "catalog/sources.json"
                    sources = load_json(sources_path)
                    record = sources["sources"][4]
                    record["license_url"] = "unknown" if mode == "unknown-license-url" else "unlicensed"
                    sources_path.write_text(json.dumps(sources, indent=2) + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, expected)

    def test_doc14_malformed_resources_conflicting_roles_and_symlinks(self) -> None:
        for mode, expected in (("utf8", "invalid-catalog"), ("json", "invalid-catalog"),
                               ("profile-json", "invalid-profile"), ("conflict", "invalid-profile"),
                               ("descriptor-version", "contract-drift"), ("symlink", "invalid-catalog"),
                               ("source-traversal", "unsafe-path")):
            with self.subTest(mode=mode):
                pack = copy_pack(self.root / mode)
                if mode == "utf8":
                    (pack / "catalog/guides/generic-design.md").write_bytes(b"\xff\xfe")
                elif mode == "json":
                    (pack / "catalog/manifest.json").write_text("{not-json}\n")
                elif mode == "profile-json":
                    path = pack / "profile.json"
                    path.write_text(path.read_text().replace('"version": "1.1.0"', '"version": "1.1.0", "version": "1.1.0"', 1))
                elif mode == "conflict":
                    path = pack / "profile.json"
                    profile = load_json(path)
                    profile["rules_file"] = "catalog/templates/generic-design.md"
                    path.write_text(json.dumps(profile, indent=2) + "\n")
                elif mode == "descriptor-version":
                    path = pack / "profile.json"
                    profile = load_json(path)
                    profile["compatibility_packs"][0]["contract_digest_version"] = "2"
                    path.write_text(json.dumps(profile, indent=2) + "\n")
                elif mode == "symlink":
                    path = pack / "catalog/guides/generic-design.md"
                    path.unlink()
                    path.symlink_to(pack / "RULES.md")
                else:
                    path = pack / "catalog/sources.json"
                    sources = load_json(path)
                    sources["sources"][0]["url"] = None
                    sources["sources"][0]["local_path"] = "../outside.md"
                    path.write_text(json.dumps(sources, indent=2) + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, expected)

    def test_doc14_unlicensed_control_and_no_network_or_execution(self) -> None:
        pack = copy_pack(self.root / "unlicensed")
        sources_path = pack / "catalog/sources.json"
        sources = load_json(sources_path)
        sources["sources"][4]["license_name"] = "unlicensed"
        sources["sources"][4]["license_version"] = "unlicensed"
        sources["sources"][4]["license_url"] = None
        sources_path.write_text(json.dumps(sources, indent=2) + "\n")
        with patch("socket.create_connection", side_effect=AssertionError("network access")):
            self.assertTrue(get_contract(pack)["ok"])
        for value in ("file:///tmp/license", "data:text/plain,license", "//example.com/license"):
            with self.subTest(license_url=value):
                pack = copy_pack(self.root / ("unsafe-" + value.split(":", 1)[0].replace("/", "")))
                sources_path = pack / "catalog/sources.json"
                sources = load_json(sources_path)
                sources["sources"][0]["license_url"] = value
                sources_path.write_text(json.dumps(sources, indent=2) + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, "unsafe-url")

    def test_model_root_configuration_precedes_existence(self) -> None:
        malformed = copy_pack(self.root / "malformed-roots")
        profile_path = malformed / "profile.json"
        profile = load_json(profile_path)
        profile["project"]["model_roots"] = ["mbse/model", "mbse/model"]
        profile_path.write_text(json.dumps(profile, indent=2) + "\n")
        with self.assertRaises(DocumentKitError) as rejected:
            get_contract(malformed)
        self.assertEqual(rejected.exception.code, "invalid-profile")
        missing = copy_pack(self.root / "missing-roots")
        profile_path = missing / "profile.json"
        profile = load_json(profile_path)
        profile["project"]["model_roots"] = ["mbse/missing"]
        profile_path.write_text(json.dumps(profile, indent=2) + "\n")
        self.assertTrue(get_contract(missing)["ok"])
        with self.assertRaises(DocumentKitError) as rejected:
            create_from_fixture(self.target("missing-model"), self.fixture, pack=missing)
        self.assertEqual(rejected.exception.code, "missing-model-root")

    def test_update_ownership_readability_stale_then_effective_identity(self) -> None:
        target = self.target()
        created = create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        document = target / created["path"]
        registry = registry_path(target)
        original = document.read_bytes()
        registry_before = registry.read_bytes()
        document.write_bytes(original + b"\n")
        externally_changed = document.read_bytes()
        with self.assertRaises(DocumentKitError) as rejected:
            update_section(target, "requirements", self.fixture["name"], "purpose", "Changed",
                           created["sha256"], profile_root=KIT)
        self.assertEqual(rejected.exception.code, "stale-document")
        self.assertEqual(document.read_bytes(), externally_changed)
        document.write_bytes(b"\xff\xfe")
        with self.assertRaises(DocumentKitError) as rejected:
            update_section(target, "requirements", self.fixture["name"], "purpose", "Changed",
                           created["sha256"], profile_root=KIT)
        self.assertEqual(rejected.exception.code, "unreadable-input")
        document.write_bytes(original.replace(created["contract_sha256"].encode(), b"f" * 64, 1))
        changed_hash = sha256(document)
        before = document.read_bytes()
        with self.assertRaises(DocumentKitError) as rejected:
            update_section(target, "requirements", self.fixture["name"], "purpose", "Changed",
                           changed_hash, profile_root=KIT)
        self.assertEqual(rejected.exception.code, "registry-mismatch")
        self.assertEqual(document.read_bytes(), before)
        self.assertEqual(registry.read_bytes(), registry_before)

    def test_create_existing_destination_precedes_duplicate_registry_owner(self) -> None:
        target = self.target()
        created = create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        document = target / created["path"]
        registry = registry_path(target)
        before_document = document.read_bytes()
        before_registry = registry.read_bytes()
        with self.assertRaises(DocumentKitError) as rejected:
            create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        self.assertEqual(rejected.exception.code, "document-exists")
        self.assertEqual(document.read_bytes(), before_document)
        self.assertEqual(registry.read_bytes(), before_registry)

    def test_create_registry_failure_removes_only_new_page(self) -> None:
        target = self.target()
        with patch("xdesign_doc_kit.core.os.replace", side_effect=OSError(errno.EXDEV, "cross-device")):
            with self.assertRaises(DocumentKitError) as rejected:
                create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        self.assertEqual(rejected.exception.code, "atomic-replace-failed")
        self.assertFalse((target / "docs/requirements" / (self.fixture["name"] + ".md")).exists())
        self.assertFalse(registry_path(target).exists())

    def test_independent_figure_findings_and_deterministic_sort(self) -> None:
        target = self.target()
        created = create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        document = target / created["path"]
        text = document.read_text()
        text = text.replace("<!-- document-kit:content:purpose:start -->",
                            "<!-- document-kit:content:purpose:start -->\n![Missing](assets/missing.svg)", 1)
        document.write_text(text)
        model = target / "mbse/model/example.sysml"
        model.write_text(model.read_text() +
                         f'\n@XDesignCommon::DocFigure {{ document = "{created["path"]}"; figure = "docs/requirements/assets/missing.svg"; }}\n')
        report = check_documents(target, profile_root=KIT)
        self.assertFalse(report["ok"])
        self.assertTrue({"missing-figure", "invalid-docfigure"}.issubset(codes(report)), report["diagnostics"])
        keys = [(item["file"], item["line"] or 0, item["code"], item["detail"]) for item in report["diagnostics"]]
        self.assertEqual(keys, sorted(keys))

    def test_s14_production_adapters_have_no_zero_provider_input(self) -> None:
        from xdesign_doc_kit import __main__ as cli
        from xdesign_doc_kit import core
        self.assertNotIn("digest_provider", inspect.signature(core._contract).parameters)
        self.assertNotIn("digest_provider", inspect.signature(cli.main).parameters)
        environment = {"PYTHONPATH": str(KIT), "PYTHONDONTWRITEBYTECODE": "1"}
        process = subprocess.run([sys.executable, "-B", "-m", "xdesign_doc_kit", "get-contract",
                                  "--profile-root", str(KIT)], cwd=REPO, env=environment,
                                 text=True, capture_output=True, check=False)
        self.assertEqual(process.returncode, 0, process.stdout + process.stderr)
        result = json.loads(process.stdout)
        self.assertEqual(result["contract_sha256"], get_contract(KIT)["contract_sha256"])
        self.assertNotEqual(result["contract_sha256"], ZERO)
        self.assertNotIn("digest_provider", process.stdout)
        self.assertNotIn("digest_provider", (KIT / "xdesign_doc_kit/mcp_server.py").read_text())

    def test_doc14_reviewed_pack_markdown_attack_matrix(self) -> None:
        cases = [
            ("[bad](vbscript:msgbox(1))", "unsafe-url"),
            ("[bad](JavaScript:alert(1))", "unsafe-url"),
            ("[bad](FILE:///etc/passwd)", "unsafe-url"),
            ("[bad](Data:text/plain,hello)", "unsafe-url"),
            ("[bad](blob:https://example.com/id)", "unsafe-url"),
            ("[bad](chrome://settings)", "unsafe-url"),
            ("[bad](/etc/passwd)", "unsafe-path"),
            ("[bad](../../outside.md)", "unsafe-path"),
            ("![bad](javascript:alert(1))", "unsafe-url"),
            ("[bad][x]\n\n[x]: javascript:alert(1)", "unsafe-url"),
            ("{% include \"x.md\" %}", "unsafe-markdown"),
            ("{% import \"x.md\" %}", "unsafe-markdown"),
            ("{% from \"x.md\" import thing %}", "unsafe-markdown"),
            ("[bad](HTTPS://user:pass@example.com/x)", "unsafe-url"),
            ("[bad](http://[::1)", "unsafe-url"),
        ]
        for index, (addition, expected) in enumerate(cases):
            with self.subTest(input=addition):
                pack = copy_pack(self.root / f"attack-pack-{index}")
                guide = pack / "catalog/guides/generic-design.md"
                guide.write_text(guide.read_text() + "\n" + addition + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, expected, rejected.exception.diagnostics)
                self.assertEqual(set(rejected.exception.diagnostics[0]),
                                 {"rule", "code", "file", "line", "detail"})

    def test_doc14_reviewed_managed_markdown_attack_matrix(self) -> None:
        cases = [
            ("![bad](javascript:alert(1))", "unsafe-url"),
            ("[bad][x]\n\n[x]: javascript:alert(1)", "unsafe-url"),
            ("![bad][x]\n\n[x]: https://evil.example/pixel.png", "unsafe-url"),
            ("![x]\n\n[x]: https://evil.example/pixel.png", "unsafe-url"),
            ("![bad](https://evil.example/pixel.png)", "unsafe-url"),
            ("[bad](../../outside.md)", "unsafe-path"),
            ("[bad](/etc/passwd)", "unsafe-path"),
            ("{% include \"x.md\" %}", "unsafe-markdown"),
            ("[bad](http://[::1)", "unsafe-url"),
        ]
        for index, (addition, expected) in enumerate(cases):
            with self.subTest(input=addition):
                fixture = copy.deepcopy(self.fixture)
                fixture["sections"]["purpose"]["content"] += "\n" + addition
                target = self.target(f"managed-attack-{index}")
                with self.assertRaises(DocumentKitError) as rejected:
                    create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
                self.assertEqual(rejected.exception.code, expected, rejected.exception.diagnostics)
                self.assertFalse(registry_path(target).exists())

    def test_doc14_wrong_types_never_escape_as_python_exceptions(self) -> None:
        for mode, expected in (("document-class", "invalid-catalog"), ("source-id", "invalid-source"),
                               ("section-key", "invalid-catalog"), ("ordered-fields", "invalid-profile"),
                               ("mixed-template", "invalid-catalog")):
            with self.subTest(mode=mode):
                pack = copy_pack(self.root / f"type-{mode}")
                if mode == "ordered-fields":
                    path = pack / "profile.json"
                    value = load_json(path)
                    value["figure_block"]["ordered_fields"] = [["Question"]]
                else:
                    path = pack / ("catalog/mixed/fixture.json" if mode == "mixed-template" else "catalog/manifest.json")
                    value = load_json(path)
                    if mode == "document-class":
                        value["entries"][0]["document_class"] = ["requirements"]
                    elif mode == "source-id":
                        value["entries"][0]["sources"][0]["source_id"] = ["DPR"]
                    elif mode == "section-key":
                        entry = next(item for item in value["entries"] if item["template_id"] == "system-context")
                        entry["view_concerns"][0]["section_key"] = ["boundary"]
                    else:
                        value["documents"][0]["template_id"] = ["system-requirements"]
                path.write_text(json.dumps(value, indent=2) + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, expected)
        with self.assertRaises(DocumentKitError) as rejected:
            check_documents(self.root, document_class=["design"])
        self.assertEqual(rejected.exception.code, "invalid-request")
    def test_doc14_commonmark_whitespace_entities_and_liquid_variants(self) -> None:
        cases = [
            ("[bad]( javascript:alert(1))", "unsafe-url"),
            ("[bad](\n javascript:alert(1))", "unsafe-url"),
            ("[bad]( /etc/passwd)", "unsafe-path"),
            ("[bad]( ../outside.md)", "unsafe-path"),
            ("![bad]( javascript:alert(1))", "unsafe-url"),
            ("[bad]( javascript&colon;alert(1))", "unsafe-url"),
            ("[bad](javascript&#58;alert(1))", "unsafe-url"),
            ("[bad](&#106;avascript:alert(1))", "unsafe-url"),
            ("{%- include \"x.md\" -%}", "unsafe-markdown"),
            ("{%- include_relative \"x.md\" -%}", "unsafe-markdown"),
            ("{% include_relative \"x.md\" %}", "unsafe-markdown"),
            ("{% render \"x.md\" %}", "unsafe-markdown"),
            ("{% layout \"x.md\" %}", "unsafe-markdown"),
        ]
        for index, (payload, expected) in enumerate(cases):
            with self.subTest(payload=payload, surface="pack"):
                pack = copy_pack(self.root / f"semantic-pack-{index}")
                guide = pack / "catalog/guides/generic-design.md"
                guide.write_text(guide.read_text() + "\n" + payload + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, expected)
            with self.subTest(payload=payload, surface="managed"):
                fixture = copy.deepcopy(self.fixture)
                fixture["sections"]["purpose"]["content"] += "\n" + payload
                target = self.target(f"semantic-managed-{index}")
                with self.assertRaises(DocumentKitError) as rejected:
                    create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
                self.assertEqual(rejected.exception.code, expected)
                self.assertFalse(registry_path(target).exists())

    def test_doc14_safe_whitespace_entities_and_local_paths_remain_valid(self) -> None:
        fixture = copy.deepcopy(self.fixture)
        fixture["sections"]["purpose"]["content"] += (
            "\n[Self]( #R100)\n[Reference][x]\n\n[x]: #R100\n"
            "[Source]( HTTPS://example.com/docs?a=1&amp;b=2)"
        )
        target = self.target("safe-markdown")
        created = create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        self.assertTrue(created["ok"])
        self.assertTrue(check_documents(target)["ok"])
        context = load_json(KIT / "catalog/fixtures/system-context.json")
        context["sections"]["boundary"]["content"] = context["sections"]["boundary"]["content"].replace(
            "](assets/system-context.svg)", "]( assets/system-context.svg)")
        target = self.target("safe-spaced-image")
        created = create_from_fixture(target, context, expected_digest=get_contract(KIT)["contract_sha256"])
        self.assertTrue(created["ok"])
        self.assertTrue(check_documents(target)["ok"])

    def test_doc14_reviewer_markdown_rendering_cases_are_rejected(self) -> None:
        renderer = REPO / "cache/docs-site/venv/bin/python"
        if not renderer.is_file():
            self.skipTest("repository MkDocs renderer is unavailable")
        from xdesign_doc_kit.catalog import _markdown_security_findings
        for payload, rendered_fragment in (
            ("[bad]( javascript:alert(1))", 'href="javascript:alert(1)"'),
            ("![bad]( javascript:alert(1))", 'src="javascript:alert(1)"'),
            ("[bad]( javascript&colon;alert(1))", 'href="javascript:alert(1)"'),
            ("[bad]( /etc/passwd)", 'href="/etc/passwd"'),
            ("[bad]( ../outside.md)", 'href="../outside.md"'),
        ):
            with self.subTest(payload=payload):
                rendered = subprocess.check_output(
                    [str(renderer), "-c", "import markdown,sys; print(markdown.markdown(sys.stdin.read()))"],
                    input=payload, text=True,
                )
                self.assertIn(rendered_fragment, __import__("html").unescape(rendered))
                self.assertTrue(any(item["code"] in ("unsafe-url", "unsafe-path")
                                    for item in _markdown_security_findings(payload, "review.md")))
    def test_doc14_nested_labels_attr_lists_and_existing_assets_fail_closed(self) -> None:
        cases = [
            ("[a [b]](javascript:alert(1))", "unsafe-url"),
            ("[a [b]]( javascript:alert(1))", "unsafe-url"),
            ("[a [b]](javascript&colon;alert(1))", "unsafe-url"),
            ("[a `]` b](javascript:alert(1))", "unsafe-url"),
            ("[a [b] [c]](javascript:alert(1))", "unsafe-url"),
            ("![a [b]](javascript:alert(1))", "unsafe-url"),
            ("![![x](safe.png)](javascript:alert(1))", "unsafe-url"),
            ("[x](https://example.com){: href=\"javascript:alert(1)\" }", "unsafe-markdown"),
            ("[x](https://example.com){: href=javascript:alert(1) }", "unsafe-markdown"),
            ("[x](https://example.com){: href=\"//evil.example/x\" }", "unsafe-markdown"),
            ("![x](assets/a.svg){: src=\"javascript:alert(1)\" }", "unsafe-markdown"),
        ]
        for index, (payload, expected) in enumerate(cases):
            with self.subTest(payload=payload, surface="pack"):
                pack = copy_pack(self.root / f"nested-pack-{index}")
                guide = pack / "catalog/guides/generic-design.md"
                guide.write_text(guide.read_text() + "\n" + payload + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, expected)
            with self.subTest(payload=payload, surface="managed"):
                fixture = copy.deepcopy(self.fixture)
                fixture["sections"]["purpose"]["content"] += "\n" + payload
                target = self.target(f"nested-managed-{index}")
                with self.assertRaises(DocumentKitError) as rejected:
                    create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
                self.assertEqual(rejected.exception.code, expected)
                self.assertFalse(registry_path(target).exists())
        for payload, expected in (
            ("![x](assets/system-context.svg){: src=\"javascript:alert(1)\" }", "unsafe-markdown"),
            ("[![x](assets/system-context.svg)](javascript:alert(1))", "unsafe-url"),
        ):
            with self.subTest(existing_asset=payload):
                fixture = load_json(KIT / "catalog/fixtures/system-context.json")
                fixture["sections"]["interfaces"]["content"] += "\n" + payload
                target = self.target("existing-asset-bypass")
                with self.assertRaises(DocumentKitError) as rejected:
                    create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
                self.assertEqual(rejected.exception.code, expected)
                self.assertFalse(registry_path(target).exists())

    def test_doc14_nested_and_attr_update_attach_preserve_document_and_registry(self) -> None:
        target = self.target()
        created = create_from_fixture(target, self.fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        document = target / created["path"]
        registry = registry_path(target)
        before_document = document.read_bytes()
        before_registry = registry.read_bytes()
        with self.assertRaises(DocumentKitError) as rejected:
            from xdesign_doc_kit.core import update_section
            update_section(target, "requirements", self.fixture["name"], "purpose",
                           "[a [b]](javascript:alert(1))", created["sha256"])
        self.assertEqual(rejected.exception.code, "unsafe-url")
        self.assertEqual(document.read_bytes(), before_document)
        self.assertEqual(registry.read_bytes(), before_registry)
        asset = document.parent / "assets/override.svg"
        asset.parent.mkdir(parents=True, exist_ok=True)
        asset.write_text('<svg xmlns="http://www.w3.org/2000/svg"><title>Safe</title></svg>\n')
        figure = {"asset": "assets/override.svg", "title": "Safe title",
                  "Question": "What is safe?", "Model view": "Example::SystemContext",
                  "Companions": "The context table.", "Status": "renderable-now; designed-only",
                  "Reading": "A safe figure.", "Limits": "No execution result."}
        figure["title"] = "[x](https://example.com){: href=\"javascript:alert(1)\" }"
        with self.assertRaises(DocumentKitError) as rejected:
            from xdesign_doc_kit.core import attach_figure
            attach_figure(target, "requirements", self.fixture["name"], "purpose", figure, created["sha256"])
        self.assertEqual(rejected.exception.code, "unsafe-markdown")
        self.assertEqual(document.read_bytes(), before_document)
        self.assertEqual(registry.read_bytes(), before_registry)

    def test_doc14_every_digest_markdown_resource_is_security_checked(self) -> None:
        for relative in ("README.md", "RULES.md", "SKILL.md", "templates/figure-block.md",
                         "compatibility/v1.0.0/templates/design.md"):
            with self.subTest(resource=relative):
                pack = copy_pack(self.root / ("resource-" + relative.replace("/", "-")))
                path = pack / relative
                path.write_text(path.read_text() + "\n[a [b]](javascript:alert(1))\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, "unsafe-url")

    def test_doc14_safe_nested_pointy_and_reference_syntax_is_preserved(self) -> None:
        fixture = copy.deepcopy(self.fixture)
        fixture["sections"]["purpose"]["content"] += (
            "\n[Pointy](<https://example.com/a b>)\n"
            "[Nested](https://example.com/a)\n"
            "[a [b]](https://example.com/b)\n"
            "[a `]` b](https://example.com/c)\n"
            "[Reference][x]\n\n[x]: <https://example.com/d>"
        )
        target = self.target("safe-parser-syntax")
        created = create_from_fixture(target, fixture, expected_digest=get_contract(KIT)["contract_sha256"])
        self.assertTrue(created["ok"])
        self.assertTrue(check_documents(target)["ok"])

    def test_doc14_nul_destination_and_actual_mkdocs_nested_rendering(self) -> None:
        for payload in ("[bad](java\x00script:alert(1))", "[bad](java&#0;script:alert(1))"):
            with self.subTest(payload=payload):
                pack = copy_pack(self.root / "nul-destination")
                guide = pack / "catalog/guides/generic-design.md"
                guide.write_text(guide.read_text() + "\n" + payload + "\n")
                with self.assertRaises(DocumentKitError) as rejected:
                    get_contract(pack)
                self.assertEqual(rejected.exception.code, "unsafe-url")
        renderer = REPO / "cache/docs-site/venv/bin/python"
        if not renderer.is_file():
            self.skipTest("repository MkDocs renderer is unavailable")
        from xdesign_doc_kit.catalog import _markdown_security_findings
        script = "import markdown,sys; print(markdown.markdown(sys.stdin.read(), extensions=['tables','admonition','attr_list','pymdownx.superfences']))"
        for payload in (
            "[a [b]](javascript:alert(1))", "[a `]` b](javascript:alert(1))",
            "![a [b]](javascript:alert(1))",
            "[x](https://example.com){: href=javascript:alert(1) }",
            "![x](assets/a.svg){: src=\"javascript:alert(1)\" }",
        ):
            with self.subTest(rendered_payload=payload):
                rendered = subprocess.check_output([str(renderer), "-c", script], input=payload, text=True)
                self.assertIn("javascript:alert(1)", rendered)
                self.assertTrue(_markdown_security_findings(payload, "review.md"))


if __name__ == "__main__":
    unittest.main()
