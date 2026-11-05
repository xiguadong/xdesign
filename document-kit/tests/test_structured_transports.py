"""SDK-06/08/13: real CLI and stdio MCP, including typed failure outcomes."""

from __future__ import annotations
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from test_structured import (
    KIT,
    CACHE,
    fixture,
    materialize,
    create,
    selection,
    figure,
    readj,
    writej,
    sha,
    snap,
)
from xdesign_doc_kit import core, DocumentKitError
from _faults import fault
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


def envelope(result):
    if not result.is_error:
        return result.structured_content
    text = " ".join(getattr(c, "text", "") for c in result.content)
    return json.loads(text[text.index("{") :])


class StructuredTransportTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=CACHE)
        self.root = Path(self.tmp.name)
        self.projects = []
        for name in ("python", "cli", "mcp"):
            p = self.root / name
            p.mkdir()
            self.projects.append(p)
        self.env = {
            **os.environ,
            "PYTHONPATH": str(KIT),
            "PYTHONDONTWRITEBYTECODE": "1",
            "TMPDIR": str(CACHE),
        }
        self.transcript = []

    def tearDown(self):
        writej(CACHE / (self._testMethodName + "-transcript.json"), self.transcript)
        self.tmp.cleanup()

    def cli(self, project, operation, args, payload=None, pack=KIT, mode=None, code=0):
        cmd = [sys.executable]
        if mode:
            cmd += [str(KIT / "tests/_fault_driver.py"), mode, "cli"]
        else:
            cmd += ["-m", "xdesign_doc_kit"]
        cmd += [operation.replace("_", "-"), "--profile-root", str(pack)]
        if operation != "get_contract":
            cmd += ["--target-root", str(project)]
        for key, value in args.items():
            if value is not None:
                cmd += [
                    "--"
                    + {"document_class": "class", "section_key": "section"}.get(
                        key, key.replace("_", "-")
                    ),
                    str(value),
                ]
        if payload is not None:
            cmd += ["--input", json.dumps(payload)]
        r = subprocess.run(
            cmd,
            env={**self.env, "PYTHONPATH": str(pack)},
            text=True,
            capture_output=True,
        )
        self.assertEqual(r.returncode, code, r.stdout + r.stderr)
        data = json.loads(r.stdout)
        self.transcript.append(
            dict(adapter="cli", command=cmd, exit=r.returncode, result=data)
        )
        return data

    def server(self, project, pack=KIT, mode=None):
        args = (
            [str(KIT / "tests/_fault_driver.py"), mode, "mcp"]
            if mode
            else ["-m", "xdesign_doc_kit.mcp_server"]
        )
        args += ["--project-root", str(project), "--profile", str(pack)]
        return StdioServerParameters(
            command=sys.executable, args=args, env={**self.env, "PYTHONPATH": str(pack)}
        )

    async def call(self, client, name, args):
        result = await client.call_tool(name, args)
        data = envelope(result)
        self.transcript.append(
            dict(
                adapter="mcp",
                tool=name,
                arguments=args,
                is_error=result.is_error,
                result=data,
            )
        )
        self.assertEqual(result.is_error, "error" in data)
        return data

    async def test_sdk06_08_copied_distribution_all_operations(self):
        pack = self.root / "copied-distribution"
        shutil.copytree(KIT, pack)
        f = fixture()
        args = selection(f)
        examples = [
            json.loads(block)
            for block in re.findall(
                r"```json\n(.*?)\n```", (pack / "API.md").read_text(), re.S
            )
        ]
        payload = next(
            value
            for value in examples
            if isinstance(value, dict) and set(value) == {"metadata", "sections"}
        )
        py, cli, mcp = self.projects
        for p in self.projects:
            materialize(p, f, pack)
            for source in payload["metadata"]["model_sources"]:
                (p / source).write_text("package CallerContext {}\n")
        contract = core.get_contract(pack)
        self.assertTrue((pack / "API.md").is_file())
        self.assertIn("publication-unknown", (pack / "API.md").read_text())
        async with (
            stdio_client(self.server(mcp, pack)) as (read, write),
            ClientSession(read, write) as client,
        ):
            await client.initialize()
            self.assertEqual(await self.call(client, "get_contract", {}), contract)
            self.assertEqual(self.cli(cli, "get_contract", {}, pack=pack), contract)
            for uri, expected in [
                ("xdesign-doc-kit://rules", contract["rules"]),
                (
                    "xdesign-doc-kit://template/api-design",
                    contract["templates"]["generic-api-design"],
                ),
                (
                    "xdesign-doc-kit://template/api-design/generic-api-design",
                    contract["templates"]["generic-api-design"],
                ),
            ]:
                r = await client.read_resource(uri)
                self.assertEqual(r.contents[0].text, expected)
            plan = core.plan_document(py, **args, profile_root=pack)
            self.assertEqual(await self.call(client, "plan_document", args), plan)
            self.assertEqual(self.cli(cli, "plan_document", args, pack=pack), plan)
            created = core.create_document(py, **args, **payload, profile_root=pack)
            self.assertEqual(
                await self.call(client, "create_document", {**args, **payload}), created
            )
            self.assertEqual(
                self.cli(cli, "create_document", args, payload, pack), created
            )
            for selected in (False, True):
                opts = {"selection": args} if selected else {}
                report = core.check_documents(py, profile_root=pack, **opts)
                self.assertEqual(
                    await self.call(client, "check_documents", opts), report
                )
                self.assertEqual(
                    self.cli(
                        cli, "check_documents", args if selected else {}, pack=pack
                    ),
                    report,
                )
            update = dict(
                section_key="purpose",
                content="Revised boundary.",
                expected_sha256=created["sha256"],
            )
            updated = core.update_section(py, **args, **update, profile_root=pack)
            self.assertEqual(
                await self.call(client, "update_section", {**args, **update}), updated
            )
            self.assertEqual(
                self.cli(
                    cli,
                    "update_section",
                    {**args, "section_key": "purpose"},
                    {k: v for k, v in update.items() if k != "section_key"},
                    pack,
                ),
                updated,
            )
            for p in self.projects:
                fig = figure(p, created["path"])
            attach = dict(
                section_key="purpose", figure=fig, expected_sha256=updated["sha256"]
            )
            attached = core.attach_figure(py, **args, **attach, profile_root=pack)
            self.assertEqual(
                await self.call(client, "attach_figure", {**args, **attach}), attached
            )
            self.assertEqual(
                self.cli(
                    cli,
                    "attach_figure",
                    {**args, "section_key": "purpose"},
                    {k: v for k, v in attach.items() if k != "section_key"},
                    pack,
                ),
                attached,
            )
            for p in self.projects:
                self.assertEqual(
                    (p / created["path"]).read_bytes(),
                    (py / created["path"]).read_bytes(),
                )
            stale = await self.call(
                client, "attach_figure", {**args, **attach, "concern": "unknown"}
            )
            self.assertEqual(stale["error"]["code"], "stale-document")
            for selector in ("target_root", "project_root", "profile_root", "profile"):
                self.assertEqual(
                    (
                        await self.call(
                            client, "plan_document", {**args, selector: str(cli)}
                        )
                    )["error"]["code"],
                    "invalid-request",
                )
            self.assertEqual(
                (
                    await self.call(
                        client,
                        "create_document",
                        {**args, **payload, "output_path": "elsewhere"},
                    )
                )["error"]["code"],
                "invalid-request",
            )
            self.assertEqual(
                (
                    await self.call(
                        client,
                        "create_document",
                        {**args, **payload, "expected_contract_sha256": "0" * 64},
                    )
                )["error"]["code"],
                "invalid-request",
            )
            self.assertEqual(
                (
                    await self.call(
                        client,
                        "create_document",
                        {**args, **payload, "expected_contract_sha256": "f" * 64},
                    )
                )["error"]["code"],
                "contract-drift",
            )
            (pack / "README.md").write_text(
                (pack / "README.md").read_text() + "\nChanged after launch.\n"
            )
            self.assertEqual(
                (await self.call(client, "plan_document", {**args, "name": "Bad"}))[
                    "error"
                ]["code"],
                "invalid-document-name",
            )
            self.assertEqual(
                (
                    await self.call(
                        client, "attach_figure", {**args, **attach, "figure": []}
                    )
                )["error"]["code"],
                "invalid-request",
            )
            self.assertEqual(
                (await self.call(client, "plan_document", args))["error"]["code"],
                "contract-drift",
            )
        before = snap(cli)
        self.assertEqual(
            self.cli(
                cli,
                "create_document",
                args,
                {**payload, "output_path": "elsewhere"},
                pack,
                code=1,
            )["error"]["code"],
            "invalid-request",
        )
        self.assertEqual(snap(cli), before)
        self.assertEqual(
            self.cli(cli, "check_documents", {"layer": "core"}, pack=pack, code=1)[
                "error"
            ]["code"],
            "invalid-request",
        )

    async def test_sdk13_denied_io_and_publication_outcomes_all_surfaces(self):
        # Each phase is faulted in a fresh equivalent project. Injection is test-only;
        # the server still uses actual MCP stdio and the real product operations.
        modes = [
            "pack-read",
            "model-read",
            "document-read",
            "registry-read",
            "route",
            "lock",
            "staging",
            "page-publish",
            "registry-publish",
            "orphan",
            "unknown-cleanup",
            "after-publication",
        ]
        f = fixture()
        args = selection(f)
        payload = {k: f[k] for k in ("metadata", "sections")}
        for mode in modes:
            projects = []
            created = []
            for base in self.projects:
                p = base / mode
                p.mkdir()
                materialize(p, f)
                projects.append(p)
                if mode in ("document-read", "registry-read"):
                    created.append(create(p, f))
            operation = "update_section" if created else "create_document"
            request = {
                **args,
                **(
                    dict(
                        section_key="purpose",
                        content="Changed.",
                        expected_sha256=created[0]["sha256"],
                    )
                    if created
                    else payload
                ),
            }
            before = [snap(p) for p in projects]
            with self.subTest(mode=mode):
                try:
                    with fault(mode):
                        getattr(core, operation)(projects[0], **request)
                except DocumentKitError as e:
                    py = e.as_dict()
                else:
                    self.fail("Fault did not reject Python operation")
                cli_args = {**args, **({"section_key": "purpose"} if created else {})}
                cli_payload = {
                    k: v for k, v in request.items() if k not in (*args, "section_key")
                }
                cli = self.cli(
                    projects[1], operation, cli_args, cli_payload, mode=mode, code=1
                )
                async with (
                    stdio_client(self.server(projects[2], mode=mode)) as (read, write),
                    ClientSession(read, write) as client,
                ):
                    await client.initialize()
                    mcp = await self.call(client, operation, request)
                self.assertEqual(py, cli)
                self.assertEqual(py, mcp)
                self.assertIn("outcome", py)
                outcome = py["outcome"]
                self.assertEqual(
                    set(outcome), {"operation", "phase", "publication_state", "path"}
                )
                if mode in ("orphan", "unknown-cleanup", "after-publication"):
                    self.assertEqual(
                        py["error"]["code"],
                        (
                            "publication-incomplete"
                            if mode == "orphan"
                            else "publication-unknown"
                        ),
                    )
                    for p in projects:
                        self.assertTrue(
                            (p / "docs/design/api/core/create-run.md").is_file()
                        )
                        self.assertFalse(core.check_documents(p)["ok"])
                else:
                    self.assertEqual(outcome["publication_state"], "none")
                    for p, b in zip(projects, before):
                        self.assertEqual(snap(p), b)

    async def test_sdk02_06_flow_roles_actual_mcp(self):
        project = self.projects[2]
        async with (
            stdio_client(self.server(project)) as (read, write),
            ClientSession(read, write) as client,
        ):
            await client.initialize()
            for cls in (
                "flow-definition",
                "flow-implementation",
                "flow-views",
                "flow-index",
            ):
                f = fixture(cls)
                materialize(project, f)
                args = selection(f)
                plan = await self.call(client, "plan_document", args)
                self.assertEqual(
                    plan["path"],
                    "docs/design/flows/sample-flow/"
                    + cls.removeprefix("flow-")
                    + ".md",
                )
                created = await self.call(
                    client,
                    "create_document",
                    {**args, "metadata": f["metadata"], "sections": f["sections"]},
                )
                self.assertTrue(created["ok"], created)
                key = next(iter(f["sections"]))
                update = {
                    **args,
                    "section_key": key,
                    "content": "Caller revised this flow role.",
                    "expected_sha256": created["sha256"],
                }
                updated = await self.call(client, "update_section", update)
                self.assertTrue(updated["ok"], updated)
                fig = figure(project, created["path"])
                attached = await self.call(
                    client,
                    "attach_figure",
                    {
                        **args,
                        "section_key": key,
                        "figure": fig,
                        "expected_sha256": updated["sha256"],
                    },
                )
                self.assertTrue(attached["ok"], attached)
                selected = await self.call(
                    client, "check_documents", {"selection": args}
                )
                self.assertTrue(selected["ok"], selected)
                self.assertEqual(selected["checked_paths"], [created["path"]])
            self.assertTrue((await self.call(client, "check_documents", {}))["ok"])

    async def test_sdk13_update_attach_denials_preserve_pages(self):
        for operation in ("update_section", "attach_figure"):
            for mode in (
                "model-read",
                "document-read",
                "registry-read",
                "staging",
                "page-publish",
            ):
                f = fixture()
                args = selection(f)
                projects = []
                for base in self.projects:
                    project = base / (operation + "-" + mode)
                    project.mkdir()
                    materialize(project, f)
                    r = create(project, f)
                    fig = figure(project, r["path"])
                    projects.append(project)
                request = {
                    **args,
                    "section_key": "purpose",
                    "expected_sha256": r["sha256"],
                    **(
                        {"content": "Revised."}
                        if operation == "update_section"
                        else {"figure": fig}
                    ),
                }
                before = [snap(p) for p in projects]
                with self.subTest(operation=operation, mode=mode):
                    try:
                        with fault(mode):
                            getattr(core, operation)(projects[0], **request)
                    except DocumentKitError as e:
                        py = e.as_dict()
                    else:
                        self.fail("Fault did not reject operation")
                    cli = self.cli(
                        projects[1],
                        operation,
                        {**args, "section_key": "purpose"},
                        {
                            k: v
                            for k, v in request.items()
                            if k not in (*args, "section_key")
                        },
                        mode=mode,
                        code=1,
                    )
                    async with (
                        stdio_client(self.server(projects[2], mode=mode)) as (
                            read,
                            write,
                        ),
                        ClientSession(read, write) as client,
                    ):
                        await client.initialize()
                        mcp = await self.call(client, operation, request)
                    self.assertEqual(py, cli)
                    self.assertEqual(py, mcp)
                    self.assertEqual(py["outcome"]["publication_state"], "none")
                    for project, initial in zip(projects, before):
                        self.assertEqual(snap(project), initial)


if __name__ == "__main__":
    unittest.main()
