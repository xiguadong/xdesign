#!/usr/bin/env python3
"""Build or preview the documentation, keeping local state under cache/."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import venv


REPO = Path(__file__).resolve().parents[1]
CACHE = REPO / "cache" / "docs-site"
REQUIREMENTS = REPO / "tools" / "docs_site" / "requirements.txt"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("serve", "build"), nargs="?", default="serve")
    parser.add_argument("--port", type=int, default=8766, help="Local preview port (default: 8766)")
    arguments = parser.parse_args()
    if not 1 <= arguments.port <= 65535:
        parser.error("--port must be between 1 and 65535")

    for directory in (CACHE / "tmp", CACHE / "pycache", REPO / "cache" / "pip"):
        directory.mkdir(parents=True, exist_ok=True)
    environment = {
        **os.environ,
        "TMPDIR": str(CACHE / "tmp"),
        "TEMP": str(CACHE / "tmp"),
        "TMP": str(CACHE / "tmp"),
        "PIP_CACHE_DIR": str(REPO / "cache" / "pip"),
        "PYTHONPYCACHEPREFIX": str(CACHE / "pycache"),
        "PYTHONDONTWRITEBYTECODE": "1",
        "XDG_CACHE_HOME": str(REPO / "cache"),
    }
    os.environ.update(environment)
    environment_path = CACHE / "venv"
    python = environment_path / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not python.exists():
        venv.EnvBuilder(with_pip=True).create(environment_path)

    fingerprint = hashlib.sha256(REQUIREMENTS.read_bytes()).hexdigest()
    stamp = environment_path / ".docs-requirements.sha256"
    if not stamp.exists() or stamp.read_text().strip() != fingerprint:
        subprocess.run(
            [str(python), "-m", "pip", "install", "--disable-pip-version-check",
             "-r", str(REQUIREMENTS)],
            cwd=REPO, env=environment, check=True,
        )
        stamp.write_text(fingerprint + "\n", encoding="utf-8")

    command = [str(python), "-m", "mkdocs", arguments.command,
               "--config-file", str(REPO / "mkdocs.yml"), "--strict"]
    if arguments.command == "serve":
        command.extend(["--dev-addr", f"127.0.0.1:{arguments.port}"])
    # Replace this process so Ctrl+C reaches the live-reload server directly.
    os.chdir(REPO)
    os.execve(str(python), command, environment)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"Documentation command failed: {error}", file=sys.stderr)
        raise SystemExit(1)
