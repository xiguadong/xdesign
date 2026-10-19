"""Conservative view guards from the Rust parser's concrete syntax tree.

Only the compiler's debug-tree format is decoded here. SysML names come from
the compiler's UTF-8 byte spans; no source-text pattern determines syntax.
"""
from __future__ import annotations

from dataclasses import dataclass, field
import re


_ROW = re.compile(r"^( *)([A-Z_][A-Z_0-9]*)@(\d+)\.\.(\d+)(.*)$")
_SCOPES = {"PACKAGE", "DEFINITION", "USAGE"}


@dataclass
class _Node:
    kind: str
    start: int
    end: int
    token: bool
    children: list = field(default_factory=list)


def _decode_tree(tree_text, source_bytes):
    roots, stack = [], []
    for line in tree_text.splitlines():
        if not line.strip():
            continue
        match = _ROW.fullmatch(line)
        if not match:
            raise ValueError("unrecognized compiler CST row")
        indent, kind, start, end, tail = match.groups()
        if len(indent) % 2:
            raise ValueError("invalid compiler CST indentation")
        depth, start, end = len(indent) // 2, int(start), int(end)
        if depth > len(stack) or not 0 <= start <= end <= len(source_bytes):
            raise ValueError("compiler CST nesting or source span is invalid")
        while len(stack) > depth:
            stack.pop()
        node = _Node(kind, start, end, bool(tail.strip()))
        if stack:
            parent = stack[-1]
            if parent.token or not parent.start <= start <= end <= parent.end:
                raise ValueError("compiler CST child lies outside its parent")
            parent.children.append(node)
        else:
            roots.append(node)
        stack.append(node)
    if (len(roots) != 1 or roots[0].kind != "SOURCE_FILE"
            or roots[0].start != 0 or roots[0].end != len(source_bytes)):
        raise ValueError("expected one compiler SOURCE_FILE matching the source bytes")
    return roots[0]


def unsupported_view_syntax(tree_text: str, source: str) -> dict[str, str]:
    """Return qualified view names with unsupported filter/import syntax.

    ``tree_text`` is stderr from one pinned ``sysml parse --tree`` invocation;
    ``source`` must be its exact source snapshot. Both ViewUsage and
    ViewDefinition scopes are recorded, so callers can apply definition guards
    to usage instances through the native type/subclassification graph.

    A ``*`` entry means a filter or CST cannot be mapped safely and callers must
    decline declared-view rendering. Normal imports outside views are allowed.
    Quoted names retain their original spelling, as native qualifiedName does.
    """
    if not isinstance(tree_text, str) or not isinstance(source, str):
        return {"*": "Compiler syntax tree and source must both be text."}
    try:
        source_bytes = source.encode("utf-8")
        root = _decode_tree(tree_text, source_bytes)
    except (ValueError, UnicodeError) as error:
        return {"*": f"Compiler syntax tree cannot be mapped safely: {error}."}
    unsupported = {}

    def record(key, reason):
        previous = unsupported.get(key)
        if previous is None:
            unsupported[key] = reason
        elif reason not in previous:
            unsupported[key] = previous + " " + reason

    # Keep traversal iterative: valid models may nest more deeply than the
    # host Python recursion limit. Each branch owns immutable scope/context.
    pending = [(root, (), True, None)]
    while pending:
        node, scope, reliable, view_name = pending.pop()
        next_scope, next_reliable, next_view = scope, reliable, view_name
        if node.kind in _SCOPES and not node.token:
            names = [child for child in node.children if child.kind == "NAME" and not child.token]
            if len(names) == 1:
                try:
                    name = source_bytes[names[0].start:names[0].end].decode("utf-8")
                except UnicodeError:
                    name = ""
                if name:
                    next_scope = scope + (name,)
                else:
                    next_reliable = False
            else:
                # Anonymous or effective names require semantic rules; never
                # invent their qualified spelling from NAME_REF tokens.
                next_reliable = False
            is_view = any(child.kind == "VIEW_KW" and child.token for child in node.children)
            if is_view:
                next_view = "::".join(next_scope) if next_reliable else "*"
        if node.kind == "FILTER" and not node.token:
            record(next_view or "*", "View filters are unsupported by this compiler projection.")
        elif node.kind == "IMPORT" and not node.token and next_view is not None:
            record(next_view, "Ordinary imports inside views cannot be separated from exposed memberships.")
        for child in reversed(node.children):
            if not child.token:
                pending.append((child, next_scope, next_reliable, next_view))
    return unsupported
