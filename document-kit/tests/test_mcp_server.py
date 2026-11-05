"""Process-level tests for the optional stdio MCP adapter."""

from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


KIT_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = KIT_ROOT.parent
CACHE = REPO_ROOT / "cache" / "document-kit-mcp-tests"
CACHE = Path(os.environ.get("XDESIGN_DOC_KIT_TEST_CACHE", str(CACHE)))


def _project(path: Path) -> None:
    (path / "mbse" / "model").mkdir(parents=True)
    (path / "mbse" / "wishes").mkdir(parents=True)
    (path / "mbse" / "model" / "sensor.sysml").write_text(
        "package Sensor {}\n", encoding="utf-8"
    )


def _payload(profile: dict) -> dict:
    sections = {}
    for index, section in enumerate(
        profile["document_classes"]["requirements"]["required_sections"], 900
    ):
        sections[section["key"]] = {
            "anchor": f"R{index}",
            "content": f"The sensor {section['key']} is described by its model.",
            "fields": {
                field: f"The sensor {field.replace('_', ' ')} is specified."
                for field in section["required_fields"]
            },
        }
    return {
        "metadata": {
            "title": "Sensor interface requirements",
            "status": "draft",
            "revision": "1",
            "model_sources": ["mbse/model/sensor.sysml"],
        },
        "sections": sections,
    }


def _content_text(result) -> str:
    return " ".join(getattr(block, "text", "") for block in result.content)


@unittest.skipUnless(importlib.util.find_spec("mcp"), "install the document-kit MCP extra")
class McpServerProcessTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        self.workspace = tempfile.TemporaryDirectory(prefix="mcp-", dir=CACHE)
        self.root = Path(self.workspace.name)
        self.project = self.root / "project"
        self.cli_project = self.root / "cli-project"
        self.project.mkdir()
        self.cli_project.mkdir()
        _project(self.project)
        _project(self.cli_project)
        self.environment = {
            **os.environ,
            "PYTHONPATH": str(KIT_ROOT),
            "PYTHONDONTWRITEBYTECODE": "1",
            "TMPDIR": str(CACHE),
            "TMP": str(CACHE),
            "TEMP": str(CACHE),
            "XDG_CACHE_HOME": str(CACHE),
        }

    def tearDown(self) -> None:
        self.workspace.cleanup()

    def _server(self, profile: Path | None = None):
        from mcp import StdioServerParameters

        return StdioServerParameters(
            command=sys.executable,
            args=[
                "-m",
                "xdesign_doc_kit.mcp_server",
                "--project-root",
                str(self.project),
                "--profile",
                str(profile or KIT_ROOT / "profile.json"),
            ],
            cwd=str(self.project),
            env=self.environment,
        )

    def _cli(self, project: Path, verb: str, *arguments: str, payload: dict | None = None,
             expected_returncode: int = 0, profile: Path = KIT_ROOT) -> dict:
        command = [
            sys.executable,
            "-m",
            "xdesign_doc_kit",
            verb,
            "--target-root",
            str(project),
            "--profile-root",
            str(profile),
            *arguments,
        ]
        if payload is not None:
            input_path = self.root / f"{verb}.json"
            input_path.write_text(json.dumps(payload), encoding="utf-8")
            command += ["--input", str(input_path)]
        process = subprocess.run(
            command,
            cwd=REPO_ROOT,
            env=self.environment,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(process.returncode, expected_returncode, process.stderr or process.stdout)
        return json.loads(process.stdout)

    async def test_tools_resources_cli_parity_and_project_binding(self) -> None:
        from mcp import Client

        async with Client(self._server()) as client:
            tools = await client.list_tools()
            self.assertEqual(
                {tool.name for tool in tools.tools},
                {
                    "get_contract",
                    "plan_document",
                    "create_document",
                    "update_section",
                    "attach_figure",
                    "check_documents",
                },
            )

            resources = await client.list_resources()
            self.assertEqual(
                {str(item.uri) for item in resources.resources},
                {"xdesign-doc-kit://contract", "xdesign-doc-kit://rules", "xdesign-doc-kit://catalog"},
            )
            templates = await client.list_resource_templates()
            template_uris = {str(item.uri_template) for item in templates.resource_templates}
            self.assertIn("xdesign-doc-kit://template/{document_class}", template_uris)
            self.assertIn("xdesign-doc-kit://template/{document_class}/{template_id}", template_uris)
            template = await client.read_resource("xdesign-doc-kit://template/requirements")
            self.assertIn("## 2. Requirements", template.contents[0].text)

            contract_result = await client.call_tool("get_contract")
            self.assertFalse(contract_result.is_error)
            contract = contract_result.structured_content
            self.assertIsInstance(contract, dict)
            self.assertEqual(contract["profile_id"], "xdesign-sysml-docs")
            self.assertEqual(contract["contract_digest_version"], "2")
            self.assertNotEqual(contract["contract_sha256"], "0" * 64)
            process = subprocess.run(
                [sys.executable, "-m", "xdesign_doc_kit", "get-contract", "--profile-root", str(KIT_ROOT)],
                cwd=REPO_ROOT, env=self.environment, text=True, capture_output=True, check=False,
            )
            self.assertEqual(process.returncode, 0, process.stdout + process.stderr)
            cli_contract = json.loads(process.stdout)
            self.assertEqual(contract, cli_contract)
            catalog = await client.read_resource("xdesign-doc-kit://catalog")
            self.assertEqual(json.loads(catalog.contents[0].text), cli_contract["catalog"])
            selected = await client.read_resource("xdesign-doc-kit://template/design/generic-design")
            self.assertEqual(selected.contents[0].text, cli_contract["templates"]["generic-design"])
            self.assertNotEqual(selected.contents[0].text,
                                (KIT_ROOT / "catalog/examples/generic-design.md").read_text())
            payload = _payload(contract["profile"])

            plan_result = await client.call_tool(
                "plan_document", {"document_class": "requirements", "name": "sensor"}
            )
            self.assertFalse(plan_result.is_error, plan_result.content)
            cli_plan = self._cli(
                self.cli_project,
                "plan-document",
                "--class",
                "requirements",
                "--name",
                "sensor",
            )
            plan = plan_result.structured_content
            self.assertEqual(plan, cli_plan)
            self.assertFalse((self.project / plan["path"]).exists())

            created_result = await client.call_tool(
                "create_document",
                {
                    "document_class": "requirements",
                    "name": "sensor",
                    **payload,
                },
            )
            self.assertFalse(created_result.is_error, created_result.content)
            created = created_result.structured_content
            cli_created = self._cli(
                self.cli_project,
                "create-document",
                "--class",
                "requirements",
                "--name",
                "sensor",
                payload=payload,
            )
            self.assertEqual(created, cli_created)
            self.assertEqual(
                (self.project / created["path"]).read_bytes(),
                (self.cli_project / cli_created["path"]).read_bytes(),
            )

            checked_result = await client.call_tool("check_documents")
            self.assertFalse(checked_result.is_error, checked_result.content)
            checked = checked_result.structured_content
            cli_checked = self._cli(self.cli_project, "check-documents")
            self.assertTrue(checked["ok"], checked["diagnostics"])
            self.assertEqual(checked, cli_checked)

            updated = await client.call_tool(
                "update_section",
                {
                    "document_class": "requirements",
                    "name": "sensor",
                    "section_key": "purpose",
                    "content": "The sensor boundary includes its bus and controller.",
                    "expected_sha256": created["sha256"],
                },
            )
            self.assertFalse(updated.is_error, updated.content)
            cli_updated = self._cli(
                self.cli_project, "update-section", "--class", "requirements", "--name", "sensor",
                "--section", "purpose", payload={"content": "The sensor boundary includes its bus and controller.",
                                                  "expected_sha256": created["sha256"]},
            )
            self.assertEqual(updated.structured_content, cli_updated)
            stale = await client.call_tool(
                "update_section",
                {
                    "document_class": "requirements",
                    "name": "sensor",
                    "section_key": "purpose",
                    "content": "A stale overwrite attempt.",
                    "expected_sha256": created["sha256"],
                },
            )
            self.assertTrue(stale.is_error)
            self.assertIn("stale-document", _content_text(stale))

            invalid = await client.call_tool(
                "plan_document",
                {"document_class": "requirements", "name": "../escape"},
            )
            self.assertTrue(invalid.is_error)
            self.assertIn("invalid-document-name", _content_text(invalid))

            for key in ("target_root", "project_root", "profile_root", "profile"):
                redirected = await client.call_tool(
                    "plan_document",
                    {
                        "document_class": "requirements",
                        "name": "redirect",
                        key: str(self.cli_project),
                    },
                )
                self.assertTrue(redirected.is_error, key)
                self.assertIn("invalid-request", _content_text(redirected))
            self.assertFalse((self.cli_project / "docs" / "requirements" / "redirect.md").exists())
            swapped = await client.call_tool(
                "get_contract", {"profile_root": str(self.root)}
            )
            self.assertTrue(swapped.is_error)

    async def test_template_selection_and_concern_cli_parity(self) -> None:
        from mcp import Client
        import hashlib

        fixture = json.loads((KIT_ROOT / "catalog/fixtures/system-context.json").read_text())
        for project in (self.project, self.cli_project):
            for item in fixture["models"] + fixture["figures"]:
                destination = project / item["target"]
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes((KIT_ROOT / item["pack"]).read_bytes())
        payload = {"metadata": fixture["metadata"], "sections": fixture["sections"]}

        def strip_figure(project: Path, relative: str) -> str:
            path = project / relative
            text = path.read_text()
            start = text.index("\n\n![Aster boundary and exchanges]")
            end = text.index("<!-- document-kit:content:boundary:end -->", start)
            path.write_text(text[:start] + "\n" + text[end:])
            return hashlib.sha256(path.read_bytes()).hexdigest()

        item = fixture["figures"][0]
        figure = {"asset": "assets/system-context.svg", "title": item["title"]}
        for key in ("Question", "Model view", "Companions", "Status", "Reading", "Limits"):
            figure[key] = item[key]
        async with Client(self._server()) as client:
            created_result = await client.call_tool("create_document", {
                "document_class": "architecture", "name": fixture["name"],
                "template_id": "system-context", **payload,
            })
            self.assertFalse(created_result.is_error, created_result.content)
            created = created_result.structured_content
            cli_created = self._cli(self.cli_project, "create-document", "--class", "architecture",
                                    "--name", fixture["name"], "--template-id", "system-context", payload=payload)
            self.assertEqual(created, cli_created)
            self.assertEqual(created["contract_digest_version"], "2")
            expected = strip_figure(self.project, created["path"])
            cli_expected = strip_figure(self.cli_project, cli_created["path"])
            self.assertEqual(expected, cli_expected)
            base = {"document_class": "architecture", "name": fixture["name"],
                    "section_key": "boundary", "figure": figure, "expected_sha256": expected}
            omitted = await client.call_tool("attach_figure", base)
            self.assertTrue(omitted.is_error)
            self.assertIn("missing-diagram-decision", _content_text(omitted))
            cli_omitted = self._cli(self.cli_project, "attach-figure", "--class", "architecture",
                                    "--name", fixture["name"], "--section", "boundary",
                                    payload={"figure": figure, "expected_sha256": cli_expected}, expected_returncode=1)
            self.assertEqual(cli_omitted["error"]["code"], "missing-diagram-decision")
            mismatched = await client.call_tool("attach_figure", {**base, "concern": "unknown"})
            self.assertTrue(mismatched.is_error)
            self.assertIn("invalid-diagram-decision", _content_text(mismatched))
            cli_mismatched = self._cli(self.cli_project, "attach-figure", "--class", "architecture",
                                       "--name", fixture["name"], "--section", "boundary", "--concern", "unknown",
                                       payload={"figure": figure, "expected_sha256": cli_expected}, expected_returncode=1)
            self.assertEqual(cli_mismatched["error"]["code"], "invalid-diagram-decision")
            attached_result = await client.call_tool("attach_figure", {**base, "concern": "system-boundary"})
            self.assertFalse(attached_result.is_error, attached_result.content)
            attached = attached_result.structured_content
            cli_attached = self._cli(self.cli_project, "attach-figure", "--class", "architecture",
                                     "--name", fixture["name"], "--section", "boundary",
                                     "--concern", "system-boundary",
                                     payload={"figure": figure, "expected_sha256": cli_expected})
            self.assertEqual(attached, cli_attached)
            self.assertEqual((self.project / created["path"]).read_bytes(),
                             (self.cli_project / cli_created["path"]).read_bytes())
            checked = await client.call_tool("check_documents")
            self.assertFalse(checked.is_error, checked.content)
            self.assertTrue(checked.structured_content["ok"], checked.structured_content["diagnostics"])

    async def test_v1_projection_and_original_template_resource(self) -> None:
        from mcp import Client

        profile = KIT_ROOT / "compatibility/v1.0.0"
        async with Client(self._server(profile)) as client:
            result = await client.call_tool("get_contract")
            self.assertFalse(result.is_error, result.content)
            contract = result.structured_content
            self.assertEqual(contract["contract_digest_version"], "1")
            self.assertEqual(contract["contract_sha256"],
                             "1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6")
            self.assertTrue(all(entry["template_id"] is None for entry in contract["catalog"]["entries"]))
            self.assertTrue(all(entry["scenario"] is None for entry in contract["catalog"]["entries"]))
            resource = await client.read_resource("xdesign-doc-kit://template/requirements")
            self.assertEqual(resource.contents[0].text,
                             (profile / "templates/requirements.md").read_text())
            process = subprocess.run(
                [sys.executable, "-m", "xdesign_doc_kit", "get-contract", "--profile-root", str(profile)],
                cwd=REPO_ROOT, env=self.environment, text=True, capture_output=True, check=False,
            )
            self.assertEqual(process.returncode, 0, process.stdout + process.stderr)
            self.assertEqual(contract, json.loads(process.stdout))
            payload = _payload(contract["profile"])
            plan_result = await client.call_tool("plan_document", {
                "document_class": "requirements", "name": "sensor",
            })
            cli_plan = self._cli(self.cli_project, "plan-document", "--class", "requirements",
                                 "--name", "sensor", profile=profile)
            self.assertEqual(plan_result.structured_content, cli_plan)
            created_result = await client.call_tool("create_document", {
                "document_class": "requirements", "name": "sensor", **payload,
            })
            self.assertFalse(created_result.is_error, created_result.content)
            created = created_result.structured_content
            cli_created = self._cli(self.cli_project, "create-document", "--class", "requirements",
                                    "--name", "sensor", payload=payload, profile=profile)
            self.assertEqual(created, cli_created)
            self.assertIsNone(created["template_id"])
            self.assertEqual(created["contract_digest_version"], "1")
            updated_result = await client.call_tool("update_section", {
                "document_class": "requirements", "name": "sensor", "section_key": "purpose",
                "content": "The v1 boundary remains pinned.", "expected_sha256": created["sha256"],
            })
            self.assertFalse(updated_result.is_error, updated_result.content)
            cli_updated = self._cli(self.cli_project, "update-section", "--class", "requirements",
                                    "--name", "sensor", "--section", "purpose",
                                    payload={"content": "The v1 boundary remains pinned.",
                                             "expected_sha256": created["sha256"]}, profile=profile)
            self.assertEqual(updated_result.structured_content, cli_updated)
            for project in (self.project, self.cli_project):
                asset = project / "docs/requirements/assets/v1-context.svg"
                asset.parent.mkdir(parents=True, exist_ok=True)
                asset.write_text('<svg xmlns="http://www.w3.org/2000/svg"><title>V1</title></svg>\n')
            figure = {"asset": "assets/v1-context.svg", "title": "V1 context",
                      "Question": "What remains compatible?", "Model view": "Sensor::V1",
                      "Companions": "The v1 template.", "Status": "renderable-now; designed-only",
                      "Reading": "The retained context is visible.", "Limits": "No migration is claimed."}
            attached_result = await client.call_tool("attach_figure", {
                "document_class": "requirements", "name": "sensor", "section_key": "purpose",
                "figure": figure, "expected_sha256": cli_updated["sha256"],
            })
            self.assertFalse(attached_result.is_error, attached_result.content)
            cli_attached = self._cli(self.cli_project, "attach-figure", "--class", "requirements",
                                     "--name", "sensor", "--section", "purpose",
                                     payload={"figure": figure, "expected_sha256": cli_updated["sha256"]}, profile=profile)
            self.assertEqual(attached_result.structured_content, cli_attached)
            checked_result = await client.call_tool("check_documents")
            self.assertFalse(checked_result.is_error, checked_result.content)
            cli_checked = self._cli(self.cli_project, "check-documents", profile=profile)
            self.assertEqual(checked_result.structured_content, cli_checked)
            rejected = await client.call_tool("plan_document", {
                "document_class": "requirements", "name": "sensor", "template_id": "generic-requirements",
            })
            self.assertTrue(rejected.is_error)
            self.assertIn("unknown-template", _content_text(rejected))

    async def test_contract_drift_refuses_calls(self) -> None:
        from mcp import Client

        profile_copy = self.root / "profile-copy"
        shutil.copytree(KIT_ROOT, profile_copy)
        async with Client(self._server(profile_copy)) as client:
            first = await client.call_tool("get_contract")
            self.assertFalse(first.is_error)
            original_rules = (profile_copy / "RULES.md").read_bytes()
            with (profile_copy / "RULES.md").open("a", encoding="utf-8") as output:
                output.write("\nA changed rule pack.\n")
            drift = await client.call_tool("get_contract")
            self.assertTrue(drift.is_error)
            self.assertIn("contract-drift", _content_text(drift))
            with self.assertRaisesRegex(Exception, "contract-drift"):
                await client.read_resource("xdesign-doc-kit://rules")
            refused = await client.call_tool(
                "plan_document", {"document_class": "requirements", "name": "sensor"}
            )
            self.assertTrue(refused.is_error)
            self.assertFalse((self.project / "docs" / "requirements" / "sensor.md").exists())
            (profile_copy / "RULES.md").write_bytes(original_rules)
            compatibility = profile_copy / "compatibility/v1.0.0/RULES.md"
            compatibility.write_bytes(compatibility.read_bytes() + b"\nCompatibility drift.\n")
            pinned = await client.call_tool("get_contract")
            self.assertTrue(pinned.is_error)
            self.assertIn("contract-drift", _content_text(pinned))


if __name__ == "__main__":
    unittest.main()
