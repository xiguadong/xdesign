"""Stdio MCP adapter for one selected project and document contract.

The adapter contains no authoring rules. It delegates to the shared core and
refuses calls if the versioned profile, rules, or templates change after launch.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
from typing import Any, Callable

from mcp.server import MCPServer
from mcp.server.mcpserver.exceptions import ResourceError, ResourceNotFoundError, ToolError

from . import core


@dataclass(frozen=True)
class _Binding:
    project_root: Path
    profile_root: Path
    contract_sha256: str


_binding: _Binding | None = None

mcp = MCPServer(
    "xdesign-doc-kit",
    version="0.1.0",
    instructions=(
        "Read the selected contract before authoring. Plan each document, supply "
        "engineering content, then check the project. Structural results do not "
        "replace SysML compiler checks or engineering review."
    ),
)


def _error_envelope(code: str, message: str, diagnostics: list[dict[str, Any]] | None = None) -> str:
    findings = diagnostics or [
        {"rule": "input", "code": code, "file": "", "line": None, "detail": message}
    ]
    return json.dumps(
        {
            "ok": False,
            "error": {"code": code, "message": message},
            "diagnostics": findings,
        },
        ensure_ascii=False,
        sort_keys=True,
    )


def _tool_error(error: core.DocumentKitError) -> ToolError:
    return ToolError(_error_envelope(error.code, error.message, error.diagnostics))


def _selected() -> _Binding:
    if _binding is None:
        raise RuntimeError("MCP server has no selected project/profile binding")
    return _binding


def _reject_selectors(
    target_root: str | None,
    project_root: str | None,
    profile_root: str | None,
    profile: str | None,
) -> None:
    if any(value is not None for value in (target_root, project_root, profile_root, profile)):
        raise ToolError(
            _error_envelope(
                "invalid-request",
                "This MCP process is bound to one project and contract at launch; root/profile tool arguments are refused",
            )
        )


def _live_contract() -> dict[str, Any]:
    selected = _selected()
    result = core.get_contract(selected.profile_root)
    if result["contract_sha256"] != selected.contract_sha256:
        raise core.DocumentKitError(
            "contract-drift",
            "The selected profile, rules, or templates changed after MCP launch; restart with a reviewed contract",
        )
    return result


def _tool_call(operation: Callable[..., dict[str, Any]], *args: Any, **kwargs: Any) -> dict[str, Any]:
    try:
        selected = _selected()
        _live_contract()
        result = operation(*args, profile_root=selected.profile_root, **kwargs)
        if result["contract_sha256"] != selected.contract_sha256:
            raise core.DocumentKitError(
                "contract-drift",
                "The selected contract changed during this operation; inspect the project before retrying",
            )
        return result
    except core.DocumentKitError as error:
        raise _tool_error(error) from error


@mcp.tool()
def get_contract(
    target_root: str | None = None,
    project_root: str | None = None,
    profile_root: str | None = None,
    profile: str | None = None,
) -> dict[str, Any]:
    """Read the pinned profile, rules, templates, and contract digest."""
    _reject_selectors(target_root, project_root, profile_root, profile)
    try:
        return _live_contract()
    except core.DocumentKitError as error:
        raise _tool_error(error) from error


@mcp.tool()
def plan_document(
    document_class: str,
    name: str,
    target_root: str | None = None,
    project_root: str | None = None,
    profile_root: str | None = None,
    profile: str | None = None,
) -> dict[str, Any]:
    """Resolve a class route, required metadata, and ordered sections without writing."""
    _reject_selectors(target_root, project_root, profile_root, profile)
    return _tool_call(core.plan_document, _selected().project_root, document_class, name)


@mcp.tool()
def create_document(
    document_class: str,
    name: str,
    metadata: dict[str, Any],
    sections: dict[str, dict[str, Any]],
    target_root: str | None = None,
    project_root: str | None = None,
    profile_root: str | None = None,
    profile: str | None = None,
) -> dict[str, Any]:
    """Write a complete new document from caller-supplied engineering content."""
    _reject_selectors(target_root, project_root, profile_root, profile)
    selected = _selected()
    return _tool_call(
        core.create_document,
        selected.project_root,
        document_class,
        name,
        metadata,
        sections,
        expected_contract_sha256=selected.contract_sha256,
    )


@mcp.tool()
def update_section(
    document_class: str,
    name: str,
    section_key: str,
    content: str,
    expected_sha256: str,
    fields: dict[str, str] | None = None,
    target_root: str | None = None,
    project_root: str | None = None,
    profile_root: str | None = None,
    profile: str | None = None,
) -> dict[str, Any]:
    """Update one anchored section only when its document hash is current."""
    _reject_selectors(target_root, project_root, profile_root, profile)
    selected = _selected()
    return _tool_call(
        core.update_section,
        selected.project_root,
        document_class,
        name,
        section_key,
        content,
        expected_sha256,
        fields=fields,
        expected_contract_sha256=selected.contract_sha256,
    )


@mcp.tool()
def attach_figure(
    document_class: str,
    name: str,
    section_key: str,
    figure: dict[str, Any],
    expected_sha256: str,
    target_root: str | None = None,
    project_root: str | None = None,
    profile_root: str | None = None,
    profile: str | None = None,
) -> dict[str, Any]:
    """Embed an existing canonical SVG with the six required figure fields."""
    _reject_selectors(target_root, project_root, profile_root, profile)
    selected = _selected()
    return _tool_call(
        core.attach_figure,
        selected.project_root,
        document_class,
        name,
        section_key,
        figure,
        expected_sha256,
        expected_contract_sha256=selected.contract_sha256,
    )


@mcp.tool()
def check_documents(
    document_class: str | None = None,
    target_root: str | None = None,
    project_root: str | None = None,
    profile_root: str | None = None,
    profile: str | None = None,
) -> dict[str, Any]:
    """Return deterministic findings for the bound project's document set."""
    _reject_selectors(target_root, project_root, profile_root, profile)
    return _tool_call(core.check_documents, _selected().project_root, document_class=document_class)


def _resource_contract() -> dict[str, Any]:
    try:
        return _live_contract()
    except core.DocumentKitError as error:
        raise ResourceError(_error_envelope(error.code, error.message, error.diagnostics)) from error


@mcp.resource("xdesign-doc-kit://contract", mime_type="application/json")
def contract_resource() -> str:
    """Complete selected contract as JSON."""
    return json.dumps(_resource_contract(), ensure_ascii=False, sort_keys=True)


@mcp.resource("xdesign-doc-kit://rules", mime_type="text/markdown")
def rules_resource() -> str:
    """Human-readable rule explanation for the selected profile."""
    return str(_resource_contract()["rules"])


@mcp.resource("xdesign-doc-kit://template/{document_class}", mime_type="text/markdown")
def template_resource(document_class: str) -> str:
    """Markdown template for one document class."""
    templates = _resource_contract()["templates"]
    if document_class not in templates:
        raise ResourceNotFoundError(f"Unknown document class: {document_class}")
    return str(templates[document_class])


def main(argv: list[str] | None = None) -> int:
    """Run the server using only launch-selected project and contract paths."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project-root", required=True, help="Selected project root")
    parser.add_argument("--profile", required=True, help="Pinned profile directory or profile.json")
    arguments = parser.parse_args(argv)
    try:
        project_root = Path(arguments.project_root).expanduser().resolve(strict=True)
        profile_path = Path(arguments.profile).expanduser().resolve(strict=True)
    except OSError as error:
        parser.error(f"selected project or profile is unavailable: {error}")
    if profile_path.is_file() and profile_path.name != "profile.json":
        parser.error("--profile must name a profile directory or profile.json")
    profile_root = profile_path.parent if profile_path.is_file() else profile_path
    if not project_root.is_dir() or not profile_root.is_dir():
        parser.error("project root and profile directory must exist")
    try:
        contract = core.get_contract(profile_root)
    except core.DocumentKitError as error:
        parser.error(f"{error.code}: {error.message}")
    global _binding
    _binding = _Binding(project_root, profile_root, contract["contract_sha256"])
    mcp.run(transport="stdio")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
