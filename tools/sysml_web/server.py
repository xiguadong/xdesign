#!/usr/bin/env python3
"""Serve the local SysML v2 workbench without third-party dependencies."""

from __future__ import annotations

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import subprocess
import sys
from urllib.parse import urlparse

if __package__:
    from .compiler import DEFAULT_SHA256, MAX_TOTAL_BYTES, SysMLCompiler
else:
    from compiler import DEFAULT_SHA256, MAX_TOTAL_BYTES, SysMLCompiler


WEB_ROOT = Path(__file__).resolve().parent / "web"
PROJECT_ROOT = Path(__file__).resolve().parents[2]
STATIC_FILES = {"/": "index.html", "/index.html": "index.html", "/app.js": "app.js", "/styles.css": "styles.css"}
CONTENT_TYPES = {".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".json": "application/json; charset=utf-8", ".svg": "image/svg+xml", ".log": "text/plain; charset=utf-8", ".txt": "text/plain; charset=utf-8"}


def create_server(compiler: SysMLCompiler, host: str = "127.0.0.1", port: int = 0) -> ThreadingHTTPServer:
    if host not in {"127.0.0.1", "localhost"}:
        raise ValueError("The workbench binds to localhost.")

    class Handler(BaseHTTPRequestHandler):
        server_version = "SysMLWorkbench/1.0"

        def _trusted_request(self) -> bool:
            try:
                address = urlparse("http://" + self.headers.get("Host", ""))
                if address.hostname not in {"localhost", "127.0.0.1"} or address.port != self.server.server_port:
                    return False
                origin = self.headers.get("Origin")
                if origin:
                    parsed = urlparse(origin)
                    return parsed.scheme == "http" and parsed.hostname in {"localhost", "127.0.0.1"} and parsed.port == self.server.server_port
                return True
            except ValueError:
                return False

        def _send(self, status: int, body: bytes, content_type: str, filename: str = "") -> None:
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            if content_type == "image/svg+xml":
                self.send_header("Content-Security-Policy", "sandbox; default-src 'none'; style-src 'unsafe-inline'")
            else:
                self.send_header("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' blob: data:; object-src 'none'; base-uri 'none'; frame-ancestors 'none'")
            if filename:
                self.send_header("Content-Disposition", f'inline; filename="{filename}"')
            self.end_headers()
            self.wfile.write(body)

        def _json(self, status: int, value: object) -> None:
            self._send(status, json.dumps(value, ensure_ascii=False).encode("utf-8"), CONTENT_TYPES[".json"])

        def do_GET(self) -> None:
            if not self._trusted_request():
                self._json(403, {"error": "Only requests from the local workbench are accepted."})
                return
            path = urlparse(self.path).path
            try:
                if path == "/api/project":
                    self._json(200, {"files": compiler.project_files(), "compiler": compiler.info})
                elif path.startswith("/api/runs/"):
                    parts = path.split("/")
                    artifact = compiler.artifact_path(parts[3], parts[4]) if len(parts) == 5 else None
                    if artifact is None:
                        self._json(404, {"error": "Artifact not found."})
                    else:
                        self._send(200, artifact.read_bytes(), CONTENT_TYPES.get(artifact.suffix, "application/octet-stream"), artifact.name)
                elif path in STATIC_FILES:
                    asset = WEB_ROOT / STATIC_FILES[path]
                    self._send(200, asset.read_bytes(), CONTENT_TYPES[asset.suffix])
                else:
                    self._json(404, {"error": "Route not found."})
            except (OSError, ValueError, UnicodeError) as error:
                self._json(500, {"error": str(error)})

        def do_POST(self) -> None:
            if not self._trusted_request():
                self._json(403, {"error": "Only requests from the local workbench are accepted."})
                return
            if urlparse(self.path).path != "/api/analyze":
                self._json(404, {"error": "Route not found."})
                return
            if self.headers.get("Content-Type", "").split(";")[0].strip() != "application/json":
                self._json(415, {"error": "Send workspace content as application/json."})
                return
            try:
                size = int(self.headers.get("Content-Length", "0"))
                if not 0 < size <= MAX_TOTAL_BYTES * 6:
                    self._json(413, {"error": "Workspace request is empty or exceeds the upload limit."})
                    return
                payload = json.loads(self.rfile.read(size))
                if not isinstance(payload, dict):
                    raise ValueError("The request must be a JSON object.")
                result = compiler.analyze(
                    payload.get("files"), operation=payload.get("operation", "check"),
                    view=payload.get("view", "definitions"), target=payload.get("target", ""),
                )
                self._json(200, result)
            except (ValueError, UnicodeError, TypeError) as error:
                self._json(400, {"error": str(error)})
            except OSError as error:
                self._json(500, {"error": str(error)})

        def log_message(self, message: str, *args: object) -> None:
            sys.stderr.write(f"{self.log_date_time_string()} {message % args}\n")

    server = ThreadingHTTPServer((host, port), Handler)
    server.daemon_threads = True
    return server


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project-root", type=Path, default=PROJECT_ROOT)
    parser.add_argument("--compiler", type=Path, help="Path to the selected Rust sysml compiler")
    parser.add_argument("--compiler-sha256", default=DEFAULT_SHA256, help="SHA-256 pin for the selected v0.1.4 platform binary")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--timeout", type=float, default=30, help="Maximum seconds per compiler invocation")
    args = parser.parse_args()
    if not 0 <= args.port <= 65535:
        parser.error("--port must be between 0 and 65535")
    try:
        compiler = SysMLCompiler(args.project_root, args.compiler, timeout=args.timeout, expected_sha256=args.compiler_sha256)
        server = create_server(compiler, port=args.port)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Cannot start SysML workbench: {error}", file=sys.stderr)
        return 1
    print(f"SysML v2 workbench: http://127.0.0.1:{server.server_port}", flush=True)
    print(f"Compiler: {compiler.info['version']}", flush=True)
    print(f"Reports: {compiler.cache_root / 'runs'}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nWorkbench stopped.", flush=True)
    finally:
        server.server_close()
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
