"""Run the pinned Rust SysML compiler against retained source snapshots."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import threading
import time
import uuid
import xml.etree.ElementTree as ET

if __package__:
    from .renderer import render_model
    from .declared_views import DeclaredViews
    from .cst import unsupported_view_syntax
else:
    from renderer import render_model
    from declared_views import DeclaredViews
    from cst import unsupported_view_syntax


DEFAULT_SHA256 = "674fce28512a772d1886ff2397f5f02dda66265299c67c01f5cc36eaf00207f6"
DEFAULT_COMPILER = Path(__file__).resolve().parent / "bin" / "sysml"
MAX_FILES = 128
MAX_SOURCE_BYTES = 2 * 1024 * 1024
MAX_TOTAL_BYTES = 10 * 1024 * 1024
MAX_OUTPUT_BYTES = 64 * 1024 * 1024
VIEWS = {"definitions", "internal", "tree", "sequence", "declared"}
ARTIFACTS = {
    "report.json", "parse.json", "check.json", "export.json", "diagram.svg",
    "parse.stderr.log", "check.stderr.log", "export.stderr.log", "diagram.stderr.log",
}


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def reference_id(value: object) -> str | None:
    if isinstance(value, dict):
        return value.get("@id")
    if isinstance(value, list) and value:
        return reference_id(value[0])
    return value if isinstance(value, str) else None


def normalize_model(raw: list, unsupported_syntax: dict | None = None) -> dict:
    """Expose named declarations from the compiler's native semantic model.

    The raw interchange export remains downloadable, including unnamed
    expressions, memberships and references to the embedded standard library.
    This function does not parse SysML source or infer executable behavior.
    """
    index = {node["@id"]: node for node in raw if isinstance(node, dict) and "@id" in node}
    named = {
        identifier: node for identifier, node in index.items()
        if not node.get("isLibraryElement") and (node.get("declaredName") or node.get("name"))
    }
    documentation: dict[str, list[str]] = {}
    for node in index.values():
        if node.get("@type") == "Documentation" and node.get("body"):
            owner = reference_id(node.get("owner"))
            if owner:
                documentation.setdefault(owner, []).append(str(node["body"]))
    elements = []
    definitions = []
    for identifier, node in named.items():
        owner = reference_id(node.get("owner"))
        visited = {identifier}
        while owner and owner not in named:
            if owner in visited or owner not in index:
                owner = None
                break
            visited.add(owner)
            owner = reference_id(index[owner].get("owner"))
        name = node.get("declaredName") or node.get("name")
        kind = node.get("@type", "Element")
        element = {
            "id": identifier, "kind": kind, "name": name,
            "qualifiedName": node.get("qualifiedName", name), "owner": owner,
            "documentation": "\n\n".join(documentation.get(identifier, [])),
        }
        elements.append(element)
        if kind.endswith("Definition") or kind in {"Class", "DataType", "Enumeration"}:
            definitions.append({key: element[key] for key in ("id", "kind", "name", "qualifiedName")})
    return {
        "elements": elements,
        "roots": [element["id"] for element in elements if element["owner"] not in named],
        "definitions": sorted(definitions, key=lambda node: node["qualifiedName"]),
        "views": DeclaredViews(raw, unsupported_syntax).catalog(),
        "totalElements": len(raw),
        "format": "SysML v2 native semantic interchange",
    }


class SysMLCompiler:
    def __init__(
        self, project_root: Path, executable: Path | None = None,
        cache_root: Path | None = None, timeout: float = 30,
        expected_sha256: str = DEFAULT_SHA256,
    ) -> None:
        self.project_root = Path(project_root).resolve()
        self.cache_root = Path(cache_root or self.project_root / "cache/sysml-web").resolve()
        if not self.cache_root.is_relative_to(self.project_root / "cache"):
            raise ValueError("The tool cache must be inside the project's cache/ directory.")
        if not 0 < timeout <= 120:
            raise ValueError("Compiler timeout must be between 0 and 120 seconds.")
        if not re.fullmatch(r"[a-fA-F0-9]{64}", expected_sha256):
            raise ValueError("The expected compiler SHA-256 must contain 64 hexadecimal characters.")
        self.timeout = timeout
        original = Path(executable or os.environ.get("SYSML_COMPILER", str(DEFAULT_COMPILER))).expanduser().resolve()
        if not original.is_file():
            raise ValueError(f"Rust SysML compiler not found: {original}")
        compiler_bytes = original.read_bytes()
        digest = hashlib.sha256(compiler_bytes).hexdigest()
        if digest != expected_sha256.lower():
            raise ValueError(f"Compiler SHA-256 does not match the selected toolchain: {original}")
        compiler_dir = self.cache_root / "compiler"
        compiler_dir.mkdir(parents=True, exist_ok=True)
        self.executable = compiler_dir / ("sysml-" + digest)
        if not self.executable.is_file() or hashlib.sha256(self.executable.read_bytes()).hexdigest() != digest:
            temporary = compiler_dir / ("install-" + uuid.uuid4().hex)
            temporary.write_bytes(compiler_bytes)
            temporary.chmod(0o755)
            temporary.replace(self.executable)
        (compiler_dir / "tmp").mkdir(exist_ok=True)
        version = subprocess.run(
            [str(self.executable), "--version"], capture_output=True, text=True,
            timeout=timeout, env=self._environment(compiler_dir / "tmp"),
        )
        (compiler_dir / "version.stdout.log").write_text(version.stdout, encoding="utf-8")
        (compiler_dir / "version.stderr.log").write_text(version.stderr, encoding="utf-8")
        if version.returncode or not version.stdout.startswith("sysml 0.1.4 "):
            raise ValueError("The tool requires the selected sysml-v2-rs v0.1.4 compiler.")
        self.info = {"path": str(original), "sha256": digest, "version": version.stdout.strip()}
        self._slots = threading.BoundedSemaphore(2)

    @staticmethod
    def _environment(temporary: Path) -> dict:
        env = os.environ.copy()
        env.update({"TMPDIR": str(temporary), "TMP": str(temporary), "TEMP": str(temporary), "NO_COLOR": "1"})
        return env

    def project_files(self) -> list[dict]:
        model_root = self.project_root / "mbse"
        result = []
        if model_root.is_dir():
            for path in sorted(model_root.rglob("*")):
                if path.is_file() and path.suffix in {".sysml", ".kerml"}:
                    if not path.resolve().is_relative_to(self.project_root):
                        continue
                    if path.stat().st_size > MAX_SOURCE_BYTES:
                        raise ValueError(f"Project model exceeds the per-file limit: {path.name}")
                    result.append({"name": path.relative_to(self.project_root).as_posix(), "content": path.read_text(encoding="utf-8")})
        return self._validate_files(result) if result else []

    @staticmethod
    def _validate_files(files: object) -> list[dict]:
        if not isinstance(files, list) or not 1 <= len(files) <= MAX_FILES:
            raise ValueError(f"Load between 1 and {MAX_FILES} SysML or KerML files.")
        names = set()
        total = 0
        validated = []
        for file in files:
            if not isinstance(file, dict) or not isinstance(file.get("name"), str) or not isinstance(file.get("content"), str):
                raise ValueError("Each file requires a name and UTF-8 text content.")
            name = file["name"]
            path = PurePosixPath(name)
            if (
                not name or len(name) > 512 or path.is_absolute() or "\\" in name
                or re.match(r"^[A-Za-z]:", name)
                or any(part in {"", ".", ".."} for part in name.split("/"))
                or any(ord(char) < 32 for char in name) or path.suffix not in {".sysml", ".kerml"}
            ):
                raise ValueError("File names must be relative paths ending in .sysml or .kerml, without traversal.")
            if name in names:
                raise ValueError(f"Duplicate file name: {name}")
            names.add(name)
            size = len(file["content"].encode("utf-8"))
            total += size
            if size > MAX_SOURCE_BYTES or total > MAX_TOTAL_BYTES:
                raise ValueError("Source limit exceeded: 2 MiB per file and 10 MiB per workspace.")
            validated.append({"name": name, "content": file["content"]})
        for name in names:
            if any(parent.as_posix() in names for parent in PurePosixPath(name).parents):
                raise ValueError("A source file cannot also be another source file's directory.")
        return validated

    def _invoke(self, run: Path, phase: str, arguments: list[str]) -> tuple[dict, object]:
        syntax = phase.startswith("syntax-")
        filename = phase + ".txt" if syntax else "diagram.svg" if phase == "diagram" else phase + ".json"
        output_path = run / filename
        error_path = run / (phase + ".stderr.log")
        command = [str(self.executable), "--format", "json", "--color", "never", "parse" if syntax else phase, *arguments]
        started = time.monotonic()
        status = "completed"
        with output_path.open("wb") as stdout, error_path.open("wb") as stderr:
            process = subprocess.Popen(command, cwd=run / "sources", env=self._environment(run / "tmp"), stdout=stdout, stderr=stderr)
            while process.poll() is None:
                if time.monotonic() - started > self.timeout:
                    status = "timedOut"
                    process.kill()
                    break
                if output_path.stat().st_size + error_path.stat().st_size > MAX_OUTPUT_BYTES:
                    status = "outputLimit"
                    process.kill()
                    break
                time.sleep(0.025)
            process.wait()
        observation = {
            "command": phase, "arguments": command[1:], "exitCode": process.returncode,
            "status": status, "durationMs": round((time.monotonic() - started) * 1000),
            "stdoutArtifact": filename, "stderrArtifact": error_path.name, "ok": False,
        }
        if output_path.stat().st_size + error_path.stat().st_size > MAX_OUTPUT_BYTES:
            observation["status"] = "outputLimit"
        data = None
        if observation["status"] == "completed":
            try:
                if syntax:
                    # v0.1.4 emits its JSON parse report to stdout and its CST
                    # to stderr. Retain both original outputs plus a named tree.
                    native_stdout = run / (phase + ".parse.json")
                    output_path.replace(native_stdout)
                    observation["stdoutArtifact"] = native_stdout.name
                    data = error_path.read_text(encoding="utf-8")
                    output_path.write_text(data, encoding="utf-8")
                    valid = bool(re.search(r"(?m)^SOURCE_FILE@", data))
                elif phase == "diagram":
                    root = ET.fromstring(output_path.read_bytes())
                    valid = root.tag == "{http://www.w3.org/2000/svg}svg"
                else:
                    data = json.loads(output_path.read_text(encoding="utf-8"))
                    valid = isinstance(data, list) if phase == "export" else isinstance(data, dict) and data.get("ok") is True
                observation["ok"] = process.returncode == 0 and valid
            except (ValueError, UnicodeError, ET.ParseError):
                observation["error"] = "The compiler did not produce the expected output format."
        return observation, data

    @staticmethod
    def _diagnostics(parse: object, check: object, source_names: dict[str, str]) -> list[dict]:
        result = []
        seen = set()

        def add(phase: str, entry: object, path: str = "", category: str = "") -> None:
            if not isinstance(entry, dict):
                entry = {"message": str(entry)}
            raw_path = entry.get("path") or entry.get("file") or path
            original = source_names.get(str(raw_path), str(raw_path))
            message = entry.get("message") or entry.get("reason") or entry.get("text") or entry.get("name") or json.dumps(entry, ensure_ascii=False)
            if category:
                message = category + ": " + str(message)
            diagnostic = {
                "phase": phase, "severity": "error", "file": original,
                "line": entry.get("line"), "column": entry.get("column"), "message": str(message),
            }
            key = (original, diagnostic["line"], diagnostic["column"], diagnostic["message"])
            if key not in seen:
                seen.add(key)
                result.append(diagnostic)

        if isinstance(parse, dict):
            for file in parse.get("files", []):
                for error in file.get("errors", []):
                    add("parse", error, file.get("path", ""))
            for path in parse.get("unreadable", []):
                add("parse", path, category="Unreadable source")
        if isinstance(check, dict):
            for error in check.get("parseErrors", []):
                add("check", error)
            for field, category in (("unresolved", "Unresolved reference"), ("collisions", "Name collision"), ("violations", "Rule violation"), ("unreadable", "Unreadable source")):
                for entry in check.get(field, []):
                    add("check", entry, category=category)
        return result

    def analyze(self, files: list[dict], operation: str = "check", view: str = "definitions", target: str = "") -> dict:
        files = self._validate_files(files)
        if not isinstance(operation, str) or operation not in {"check", "render"}:
            raise ValueError("Operation must be check or render.")
        if not isinstance(view, str) or view not in VIEWS:
            raise ValueError("Unknown diagram view.")
        if not isinstance(target, str) or len(target) > 1024 or any(ord(char) < 32 for char in target):
            raise ValueError("The diagram target must be a model name of at most 1024 characters.")
        if operation == "render" and view in {"internal", "sequence", "declared"} and not target.strip():
            raise ValueError("Choose a model target for an internal, sequence, or declared SysML view.")
        run_id = uuid.uuid4().hex
        run = self.cache_root / "runs" / run_id
        (run / "sources").mkdir(parents=True)
        (run / "tmp").mkdir()
        paths = []
        source_names = {}
        source_records = []
        for file in files:
            snapshot = run / "sources" / file["name"]
            snapshot.parent.mkdir(parents=True, exist_ok=True)
            snapshot.write_text(file["content"], encoding="utf-8")
            paths.append(str(snapshot))
            source_names[str(snapshot)] = file["name"]
            source_records.append({"name": file["name"], "sha256": hashlib.sha256(file["content"].encode("utf-8")).hexdigest()})
        base_url = f"/api/runs/{run_id}/"
        response = {
            "runId": run_id, "ok": False, "compiler": self.info,
            "parse": None, "check": None, "diagnostics": [],
            "rules": {"held": 0, "unevaluated": 0}, "model": None,
            "diagram": None, "error": None, "commands": [],
            "sources": source_records, "operation": operation, "view": view, "target": target,
            "artifacts": {"report": base_url + "report.json", "parse": None, "check": None, "export": None, "syntax": []},
        }
        with self._slots:
            try:
                for phase in ("parse", "check"):
                    observation, report = self._invoke(run, phase, paths)
                    response["commands"].append(observation)
                    response[phase] = report
                    response["artifacts"][phase] = base_url + phase + ".json"
                response["diagnostics"] = self._diagnostics(response["parse"], response["check"], source_names)
                if isinstance(response["check"], dict):
                    response["rules"].update(response["check"].get("rules", {}))
                if all(command["ok"] for command in response["commands"]):
                    observation, exported = self._invoke(run, "export", paths)
                    response["commands"].append(observation)
                    response["artifacts"]["export"] = base_url + "export.json"
                    if observation["ok"]:
                        syntax_guards = {}
                        if any(element.get("@type") in {"ViewUsage", "ViewDefinition"} for element in exported):
                            for index, (path, file) in enumerate(zip(paths, files)):
                                observation, syntax_tree = self._invoke(run, f"syntax-{index}", [path, "--tree"])
                                response["commands"].append(observation)
                                response["artifacts"]["syntax"].append(base_url + f"syntax-{index}.txt")
                                if not observation["ok"]:
                                    raise ValueError("The compiler could not retain syntax evidence needed to check view filters.")
                                syntax_guards.update(unsupported_view_syntax(syntax_tree, file["content"]))
                        response["model"] = normalize_model(exported, syntax_guards)
                        if operation == "render":
                            if view == "declared":
                                svg, declared = DeclaredViews(exported, syntax_guards).render(target.strip())
                                (run / "diagram.svg").write_text(svg, encoding="utf-8")
                                response["diagram"] = {"url": base_url + "diagram.svg", "view": view, "engine": "Declared SysML view projection", "declaredView": declared}
                            elif view == "sequence":
                                observation, _ = self._invoke(run, "diagram", [*paths, f"--sequence={target.strip()}"])
                                response["commands"].append(observation)
                                if observation["ok"]:
                                    response["diagram"] = {"url": base_url + "diagram.svg", "view": view, "engine": "Rust native sequence renderer"}
                            else:
                                svg = render_model(exported, view=view, target=target.strip())
                                (run / "diagram.svg").write_text(svg, encoding="utf-8")
                                response["diagram"] = {"url": base_url + "diagram.svg", "view": view, "engine": "Compiler semantic model projection"}
                response["ok"] = all(command["ok"] for command in response["commands"])
                for command in response["commands"]:
                    if command["ok"]:
                        continue
                    stderr_path = run / command["stderrArtifact"]
                    stderr = stderr_path.read_bytes()[:8192].decode("utf-8", errors="replace").strip()
                    message = command.get("error") or stderr or f"{command['command']} failed ({command['status']}, exit {command['exitCode']})."
                    if command["status"] != "completed":
                        message = f"{command['command']} stopped: {command['status']}."
                    if not any(entry["phase"] == command["command"] for entry in response["diagnostics"]):
                        response["diagnostics"].append({"phase": command["command"], "severity": "error", "file": "", "line": None, "column": None, "message": message})
                if not response["ok"]:
                    response["error"] = "The model did not pass all requested compiler operations. See diagnostics and the retained report."
            except (OSError, ValueError, TypeError) as error:
                response["ok"] = False
                response["error"] = str(error)
                response["diagnostics"].append({"phase": "tool", "severity": "error", "file": "", "line": None, "column": None, "message": str(error)})
        write_json(run / "report.json", response)
        return response

    def artifact_path(self, run_id: str, filename: str) -> Path | None:
        syntax_artifact = bool(re.fullmatch(r"syntax-(?:[0-9]|[1-9][0-9]|1[01][0-9]|12[0-7])\.(?:txt|parse\.json|stderr\.log)", filename))
        if not re.fullmatch(r"[a-f0-9]{32}", run_id) or (filename not in ARTIFACTS and not syntax_artifact):
            return None
        path = self.cache_root / "runs" / run_id / filename
        return path if path.is_file() and path.resolve().is_relative_to(self.cache_root) else None
