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

    def _cli(self, project: Path, verb: str, *arguments: str, payload: dict | None = None) -> dict:
        command = [
            sys.executable,
            "-m",
            "xdesign_doc_kit",
            verb,
            "--target-root",
            str(project),
            "--profile-root",
            str(KIT_ROOT),
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
        self.assertEqual(process.returncode, 0, process.stderr or process.stdout)
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
                {"xdesign-doc-kit://contract", "xdesign-doc-kit://rules"},
            )
            templates = await client.list_resource_templates()
            self.assertIn(
                "xdesign-doc-kit://template/{document_class}",
                {str(item.uri_template) for item in templates.resource_templates},
            )
            template = await client.read_resource("xdesign-doc-kit://template/requirements")
            self.assertIn("## 2. Requirements", template.contents[0].text)

            contract_result = await client.call_tool("get_contract")
            self.assertFalse(contract_result.is_error)
            contract = contract_result.structured_content
            self.assertIsInstance(contract, dict)
            self.assertEqual(contract["profile_id"], "xdesign-sysml-docs")
            self.assertEqual(len(contract["contract_sha256"]), 64)
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
            self.assertEqual(plan["path"], cli_plan["path"])
            self.assertEqual(plan["required_sections"], cli_plan["required_sections"])
            self.assertEqual(plan["contract_sha256"], cli_plan["contract_sha256"])
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
            self.assertEqual(created["sha256"], cli_created["sha256"])
            self.assertEqual(created["contract_sha256"], cli_created["contract_sha256"])
            self.assertEqual(
                (self.project / created["path"]).read_bytes(),
                (self.cli_project / cli_created["path"]).read_bytes(),
            )

            checked_result = await client.call_tool("check_documents")
            self.assertFalse(checked_result.is_error, checked_result.content)
            checked = checked_result.structured_content
            cli_checked = self._cli(self.cli_project, "check-documents")
            self.assertTrue(checked["ok"], checked["diagnostics"])
            self.assertEqual(checked["document_hashes"], cli_checked["document_hashes"])
            self.assertEqual(checked["diagnostics"], cli_checked["diagnostics"])
            self.assertEqual(checked["contract_sha256"], cli_checked["contract_sha256"])

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

    async def test_contract_drift_refuses_calls(self) -> None:
        from mcp import Client

        profile_copy = self.root / "profile-copy"
        profile_copy.mkdir()
        shutil.copy2(KIT_ROOT / "profile.json", profile_copy / "profile.json")
        shutil.copy2(KIT_ROOT / "RULES.md", profile_copy / "RULES.md")
        shutil.copytree(KIT_ROOT / "templates", profile_copy / "templates")
        async with Client(self._server(profile_copy)) as client:
            first = await client.call_tool("get_contract")
            self.assertFalse(first.is_error)
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


if __name__ == "__main__":
    unittest.main()
