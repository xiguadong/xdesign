"""Shared, fail-closed invocation and report contract for SimpleFlow checks.

The runtime record is an exchange snapshot. Its plan is authenticated by exact
bytes; a visit's artifacts are authenticated by URI, byte digest and canonical
snapshot digest. Rule inputs may not float outside that snapshot.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
from typing import Any, Callable


ROOT = Path(os.environ["XDESIGN_PROJECT_ROOT"]).resolve() if "XDESIGN_PROJECT_ROOT" in os.environ else Path(__file__).resolve().parents[2]
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
EXPECTED_STAGE = {
    "simpleRequirementsCheck": "requirements",
    "simpleDesignCheck": "design",
    "simpleImplementationCheck": "implement",
    "simpleRecordCheck": "verify",
}


class CheckInputError(ValueError):
    """An invocation cannot support a trustworthy rule report."""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_path(root: Path, uri: str) -> Path:
    if not isinstance(uri, str) or not uri.strip():
        raise CheckInputError("an input URI is empty")
    candidate = Path(uri)
    if candidate.is_absolute():
        path = candidate.resolve()
    else:
        pure = PurePosixPath(uri)
        if ".." in pure.parts or pure.as_posix() != uri:
            raise CheckInputError(f"input URI is not normalized: {uri}")
        path = (root / uri).resolve()
    if not path.is_relative_to(root.resolve()):
        raise CheckInputError(f"input URI escapes the repository: {uri}")
    return path


def relative_uri(root: Path, uri: str) -> str:
    return repository_path(root, uri).relative_to(root.resolve()).as_posix()


def read_bytes(root: Path, uri: str) -> bytes:
    path = repository_path(root, uri)
    if not path.is_file():
        raise CheckInputError(f"input file is absent: {uri}")
    try:
        return path.read_bytes()
    except OSError as error:
        raise CheckInputError(f"input file is unreadable: {uri}: {error}") from error


def read_json(root: Path, uri: str) -> dict[str, Any]:
    try:
        value = json.loads(read_bytes(root, uri).decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise CheckInputError(f"invalid JSON at {uri}: {error}") from error
    if not isinstance(value, dict):
        raise CheckInputError(f"JSON object required at {uri}")
    return value


def require_hex(value: Any, label: str) -> str:
    if not isinstance(value, str) or HEX64.fullmatch(value) is None:
        raise CheckInputError(f"{label} must be a lowercase SHA-256 hex digest")
    return value


def required_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise CheckInputError(f"{label} must be a nonempty string")
    return value


def object_list(value: Any, label: str) -> list[dict[str, Any]]:
    if not isinstance(value, list) or any(not isinstance(item, dict) for item in value):
        raise CheckInputError(f"{label} must be a list of objects")
    return value


def snapshot_digest(artifacts: list[dict[str, Any]]) -> str:
    rows = [(item["uri"], item["sha256"]) for item in artifacts]
    if len({uri for uri, _ in rows}) != len(rows):
        raise CheckInputError("artifact snapshot repeats a URI")
    canonical = b"".join(
        uri.encode("utf-8") + b"\0" + digest.encode("ascii") + b"\n"
        for uri, digest in sorted(rows)
    )
    return sha256(canonical)


@dataclass
class Context:
    root: Path
    args: argparse.Namespace
    record: dict[str, Any]
    plan: dict[str, Any]
    visit: dict[str, Any]
    artifact_hashes: dict[str, str]
    input_hashes: dict[str, str]

    def read_rule_input(self, uri: str) -> bytes:
        """Read only snapshot-bound rule input bytes."""
        normalized = relative_uri(self.root, uri)
        digest = self.artifact_hashes.get(normalized)
        if digest is None:
            # Executor resources are authenticated by the frozen plan, not stage outputs.
            matches = [f["sha256"] for gate in self.plan.get("checkpoints", [])
                       for f in ((gate.get("checker") or {}).get("bundle") or {}).get("files", [])
                       if f.get("uri") == normalized]
            if matches and len(set(matches)) == 1:
                digest = matches[0]
        if digest is None:
            raise CheckInputError(f"rule input is absent from current artifact snapshot: {normalized}")
        data = read_bytes(self.root, normalized)
        actual = sha256(data)
        if digest != actual:
            raise CheckInputError(f"rule input changed since snapshot: {normalized}")
        self.input_hashes[normalized] = actual
        return data

    def read_rule_text(self, uri: str) -> str:
        try:
            return self.read_rule_input(uri).decode("utf-8")
        except UnicodeError as error:
            raise CheckInputError(f"rule input is not UTF-8: {uri}") from error


def _current_visit(record: dict[str, Any], stage: str) -> dict[str, Any]:
    visits = object_list(record.get("visits"), "record.visits")
    index = record.get("current_visit")
    if isinstance(index, bool) or not isinstance(index, int) or not 0 <= index < len(visits):
        raise CheckInputError("record.current_visit is not a valid visit index")
    visit = visits[index]
    if visit.get("stage_id") != stage:
        raise CheckInputError("current visit stage differs from request")
    return visit


def load_context(args: argparse.Namespace, root: Path = ROOT) -> Context:
    root = root.resolve()
    if args.plan_id != "designedSimpleFlow":
        raise CheckInputError("unsupported plan ID")
    require_hex(args.plan_sha256, "requested plan_sha256")
    require_hex(args.snapshot_digest, "requested snapshot_digest")
    required_string(args.stage, "requested stage")
    if args.artifact_revision < 0:
        raise CheckInputError("artifact revision cannot be negative")
    wish_uri = relative_uri(root, args.wish_model)
    if not wish_uri.endswith(".sysml"):
        raise CheckInputError("wish model must be a SysML file")
    wish_bytes = read_bytes(root, wish_uri)
    record_bytes = read_bytes(root, args.record)
    try:
        record = json.loads(record_bytes.decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise CheckInputError(f"invalid JSON at {args.record}: {error}") from error
    if not isinstance(record, dict):
        raise CheckInputError(f"JSON object required at {args.record}")
    if record.get("wish_model_uri") is not None and relative_uri(root, record["wish_model_uri"]) != wish_uri:
        raise CheckInputError("record wish model differs from request")
    if record.get("plan_id") != args.plan_id or record.get("plan_sha256") != args.plan_sha256:
        raise CheckInputError("record plan identity differs from request")
    plan_uri = required_string(record.get("plan_uri"), "record.plan_uri")
    if relative_uri(root, plan_uri) != relative_uri(root, args.plan_uri):
        raise CheckInputError("record plan URI differs from request")
    plan_bytes = read_bytes(root, plan_uri)
    if sha256(plan_bytes) != args.plan_sha256:
        raise CheckInputError("frozen plan bytes differ from requested SHA-256")
    try:
        plan = json.loads(plan_bytes.decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise CheckInputError(f"frozen plan is invalid JSON: {error}") from error
    if not isinstance(plan, dict) or plan.get("plan_id") != args.plan_id:
        raise CheckInputError("frozen plan ID differs from request")
    stages = object_list(plan.get("stages"), "plan.stages")
    if args.stage not in {stage.get("stage_id") for stage in stages}:
        raise CheckInputError("requested stage does not belong to frozen plan")
    visit = _current_visit(record, args.stage)
    if visit.get("artifact_revision") != args.artifact_revision:
        raise CheckInputError("current visit revision differs from request")
    if visit.get("artifact_snapshot_digest") != args.snapshot_digest:
        raise CheckInputError("current visit snapshot digest differs from request")
    artifacts = object_list(visit.get("artifact_digests"), "visit.artifact_digests")
    hashes: dict[str, str] = {}
    normalized_artifacts: list[dict[str, str]] = []
    for item in artifacts:
        uri = relative_uri(root, required_string(item.get("uri"), "artifact.uri"))
        if uri != item["uri"]:
            raise CheckInputError(f"artifact URI must be repository-relative: {item['uri']}")
        digest = require_hex(item.get("sha256"), f"artifact {uri} SHA-256")
        if uri in hashes:
            raise CheckInputError(f"artifact snapshot repeats URI: {uri}")
        actual = sha256(read_bytes(root, uri))
        if actual != digest:
            raise CheckInputError(f"artifact bytes changed since snapshot: {uri}")
        hashes[uri] = digest
        normalized_artifacts.append({"uri": uri, "sha256": digest})
    if snapshot_digest(normalized_artifacts) != args.snapshot_digest:
        raise CheckInputError("canonical artifact snapshot differs from request")
    return Context(root, args, record, plan, visit, hashes, {
        wish_uri: sha256(wish_bytes),
        relative_uri(root, args.record): sha256(record_bytes),
        relative_uri(root, plan_uri): sha256(plan_bytes),
    })


def result(rule_id: str, inputs: list[str], observed: Any, failures: list[str]) -> dict[str, Any]:
    return {
        "rule_id": rule_id,
        "verdict": "failed" if failures else "passed",
        "inputs": inputs,
        "observed": observed,
        "failures": failures,
    }


def code_mask(source: str) -> str:
    """Blank comments and string literals while preserving source offsets."""
    output = list(source)
    index = 0
    state = "code"
    while index < len(source):
        char = source[index]
        next_char = source[index + 1] if index + 1 < len(source) else ""
        if state == "code":
            if char == "/" and next_char == "*":
                output[index] = output[index + 1] = " "
                state = "block"
                index += 2
                continue
            if char == "/" and next_char == "/":
                output[index] = output[index + 1] = " "
                state = "line"
                index += 2
                continue
            if char == '"':
                output[index] = " "
                state = "string"
        elif state == "block":
            if char == "*" and next_char == "/":
                output[index] = output[index + 1] = " "
                state = "code"
                index += 2
                continue
            if char != "\n":
                output[index] = " "
        elif state == "line":
            if char == "\n":
                state = "code"
            else:
                output[index] = " "
        else:
            if char == "\\" and next_char:
                output[index] = output[index + 1] = " "
                index += 2
                continue
            if char == '"':
                state = "code"
            if char != "\n":
                output[index] = " "
        index += 1
    if state in {"block", "string"}:
        raise CheckInputError("unterminated SysML comment or string")
    return "".join(output)


def find_blocks(source: str, pattern: str) -> list[tuple[re.Match[str], str, str]]:
    """Locate brace-matched declarations; return header, original body, masked body."""
    masked = code_mask(source)
    found: list[tuple[re.Match[str], str, str]] = []
    for match in re.finditer(pattern, masked, re.MULTILINE):
        opening = masked.find("{", match.start(), match.end())
        if opening < 0:
            continue
        depth = 0
        for cursor in range(opening, len(masked)):
            if masked[cursor] == "{":
                depth += 1
            elif masked[cursor] == "}":
                depth -= 1
                if depth == 0:
                    found.append((match, source[opening:cursor + 1], masked[opening:cursor + 1]))
                    break
        else:
            raise CheckInputError(f"unclosed SysML declaration near offset {match.start()}")
    return found


def attribute_string(body: str, name: str) -> str | None:
    mask = code_mask(body)
    pattern = r'\battribute\s+(?:redefines\s+)?' + re.escape(name) + r'\s*=\s*"([^"\\]*(?:\\.[^"\\]*)*)"\s*;'
    for found in re.finditer(pattern, body):
        # A nested requirement's attribute does not give its parent a rationale.
        depth = mask[:found.start()].count("{") - mask[:found.start()].count("}")
        if mask[found.start()] != " " and depth == 1:
            return found.group(1)
    return None


def require_sysml_parse(root: Path, uris: list[str]) -> None:
    """Use the repository-pinned parser before inspecting SysML declarations."""
    compiler = Path(os.environ["XDESIGN_SYSML_COMPILER"]) if "XDESIGN_SYSML_COMPILER" in os.environ else ROOT / "tools" / "sysml_web" / "bin" / "sysml"
    if not compiler.is_file():
        raise CheckInputError("the pinned SysML compiler is missing")
    paths = [str(repository_path(root, uri)) for uri in uris]
    try:
        completed = subprocess.run([str(compiler), "parse", *paths], cwd=root,
                                   capture_output=True, text=True, timeout=30, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise CheckInputError(f"SysML parse could not run: {error}") from error
    if completed.returncode != 0:
        diagnostic = (completed.stderr or completed.stdout).strip()
        raise CheckInputError(f"SysML parse failed: {diagnostic}")


def emit_report(context: Context, checker_id: str, rules: tuple[str, ...], results: list[dict[str, Any]]) -> None:
    returned = [item.get("rule_id") for item in results]
    if returned != list(rules) or len(set(returned)) != len(returned):
        raise CheckInputError("checker returned a missing, extra, duplicate, or reordered rule")
    if any(item.get("verdict") not in {"passed", "failed"} for item in results):
        raise CheckInputError("checker returned an invalid rule verdict")
    report_uri = relative_uri(context.root, context.args.report)
    if not report_uri.startswith("cache/"):
        raise CheckInputError("report must be retained under cache/")
    input_hashes = dict(context.input_hashes)
    for item in results:
        for uri in item["inputs"]:
            try:
                normalized = relative_uri(context.root, uri)
                if normalized != uri:
                    raise CheckInputError(f"rule input URI is not normalized: {uri}")
                if repository_path(context.root, uri).is_file():
                    input_hashes[uri] = sha256(read_bytes(context.root, uri))
                elif item["verdict"] == "passed":
                    raise CheckInputError(f"passed rule input is absent: {uri}")
            except CheckInputError:
                if item["verdict"] == "passed":
                    raise
    report = {
        "checker_id": checker_id,
        "stage": context.args.stage,
        "artifact_revision": context.args.artifact_revision,
        "snapshot_digest": context.args.snapshot_digest,
        "report_uri": report_uri,
        "overall_verdict": "passed" if all(item["verdict"] == "passed" for item in results) else "failed",
        "rules": results,
        "input_sha256": dict(sorted(input_hashes.items())),
    }
    destination = repository_path(context.root, report_uri)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def execute(checker_id: str, rules: tuple[str, ...], evaluate: Callable[[Context], list[dict[str, Any]]],
            list_inputs: Callable[[Path], list[str]] | None = None) -> int:
    parser = argparse.ArgumentParser(description=f"Run fixed SimpleFlow checker {checker_id}")
    parser.add_argument("--list-inputs", action="store_true", help="print rule input paths as a JSON array")
    parser.add_argument("--wish-model")
    parser.add_argument("--record")
    parser.add_argument("--checker-id")
    parser.add_argument("--plan-id")
    parser.add_argument("--plan-uri")
    parser.add_argument("--plan-sha256")
    parser.add_argument("--stage")
    parser.add_argument("--artifact-revision", type=int)
    parser.add_argument("--snapshot-digest")
    parser.add_argument("--report")
    args = parser.parse_args()
    try:
        if args.list_inputs:
            if list_inputs is None:
                raise CheckInputError("this checker has no static rule input list")
            print(json.dumps(sorted(set(list_inputs(ROOT)))))
            return 0
        for field in ("wish_model", "record", "checker_id", "plan_id", "plan_uri", "plan_sha256",
                      "stage", "artifact_revision", "snapshot_digest", "report"):
            if getattr(args, field) is None:
                raise CheckInputError(f"--{field.replace('_', '-')} is required")
        if args.checker_id != checker_id:
            raise CheckInputError(f"expected checker ID {checker_id}, received {args.checker_id}")
        if args.stage != EXPECTED_STAGE[checker_id]:
            raise CheckInputError(f"checker {checker_id} cannot evaluate stage {args.stage}")
        context = load_context(args)
        results = evaluate(context)
        emit_report(context, checker_id, rules, results)
        return 0
    except (CheckInputError, OSError, KeyError, TypeError, ValueError) as error:
        print(f"{checker_id}: unevaluated: {error}", file=sys.stderr)
        return 2
