"""Verify the browser API and access to retained compiler artifacts."""

import json
from pathlib import Path
import tempfile
import threading
import unittest
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from tools.sysml_web.compiler import SysMLCompiler
from tools.sysml_web.server import create_server
from tools.sysml_web.tests.test_compiler import TEST_CACHE, VALID_FILES


class BrowserAPIIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        TEST_CACHE.mkdir(parents=True, exist_ok=True)
        cls.workspace = Path(tempfile.mkdtemp(prefix="http-", dir=TEST_CACHE))
        project = cls.workspace / "project"
        (project / "mbse").mkdir(parents=True)
        (project / "mbse/types.sysml").write_text(VALID_FILES[0]["content"])
        cls.compiler = SysMLCompiler(project, cache_root=project / "cache" / "reports")
        cls.server = create_server(cls.compiler, port=0)
        cls.server_thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.server_thread.start()
        host, port = cls.server.server_address[:2]
        cls.base_url = f"http://{host}:{port}"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.server_thread.join(timeout=5)

    def request(self, path, data=None):
        headers = {"Content-Type": "application/json"} if data is not None else {}
        body = json.dumps(data).encode("utf-8") if data is not None else None
        request = Request(self.base_url + path, data=body, headers=headers)
        with urlopen(request, timeout=30) as response:
            return response.status, response.headers, response.read()

    def test_project_endpoint_loads_editable_sources(self):
        status, headers, body = self.request("/api/project")
        self.assertEqual(status, 200)
        self.assertIn("application/json", headers["Content-Type"])
        project = json.loads(body)
        self.assertEqual(len(project["files"]), 1)
        self.assertEqual(project["files"][0]["content"], VALID_FILES[0]["content"])

    def test_browser_can_render_then_download_retained_svg(self):
        status, _, body = self.request("/api/analyze", {
            "files": VALID_FILES,
            "operation": "render",
            "view": "definitions",
        })
        self.assertEqual(status, 200)
        result = json.loads(body)
        self.assertTrue(result["ok"], result)
        run = self.compiler.cache_root / "runs" / result["runId"]
        svg_path = next(run.rglob("*.svg"))
        status, headers, svg = self.request(f"/api/runs/{result['runId']}/{svg_path.name}")
        self.assertEqual(status, 200)
        self.assertIn("image/svg+xml", headers["Content-Type"])
        self.assertEqual(svg, svg_path.read_bytes())
        self.assertIn(b"System", svg)

    def test_bad_upload_is_a_json_client_error(self):
        with self.assertRaises(HTTPError) as raised:
            self.request("/api/analyze", {
                "files": [{"name": "../escape.sysml", "content": "package Unsafe;"}],
                "operation": "check",
            })
        error = raised.exception
        self.assertEqual(error.code, 400)
        self.assertIn("application/json", error.headers["Content-Type"])
        self.assertTrue(json.loads(error.read())["error"])

    def test_invalid_sysml_returns_diagnostics_without_http_failure(self):
        status, _, body = self.request("/api/analyze", {
            "files": [{"name": "broken.sysml", "content": "package Bad { part x : ; }"}],
            "operation": "check",
        })
        self.assertEqual(status, 200)
        result = json.loads(body)
        self.assertFalse(result["ok"])
        self.assertTrue(result["diagnostics"])
        self.assertEqual(result["diagnostics"][0]["file"], "broken.sysml")

    def test_artifact_route_does_not_read_files_outside_a_run(self):
        with self.assertRaises(HTTPError) as raised:
            self.request("/api/runs/%2E%2E/%2E%2E/AGENTS.md")
        self.assertIn(raised.exception.code, (400, 404))


if __name__ == "__main__":
    unittest.main()
