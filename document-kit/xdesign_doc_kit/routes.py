"""Decidable component routes for schema 1.2 and its bound namespace."""

from __future__ import annotations

import re
from itertools import combinations
from typing import Any, Mapping
from .compatibility import _error, _safe_rel

SLUG = r"^[a-z0-9]+(?:-[a-z0-9]+)*$"


def request_selection(document_class: Any, name: Any, layer: Any) -> dict[str, Any]:
    if not isinstance(document_class, str) or not document_class:
        raise _error("invalid-request", "document_class must be nonempty text")
    if not isinstance(name, str) or re.fullmatch(SLUG, name) is None:
        raise _error(
            "invalid-document-name", f"Invalid document name: {name}", "DOC-001"
        )
    if layer is not None and (
        not isinstance(layer, str) or re.fullmatch(SLUG, layer) is None
    ):
        raise _error(
            "invalid-request", "layer must be a lowercase semantic slug or null"
        )
    return dict(document_class=document_class, name=name, layer=layer)


def validate_routes(profile: Mapping[str, Any]) -> None:
    classes = profile["document_classes"]
    root = _safe_rel(
        profile["project"]["docs_root"], "Documentation root", "invalid-profile"
    )
    for name, spec in classes.items():
        kind = spec.get("selector_kind", "flat")
        route = spec.get("path_template")
        if kind not in ("flat", "api", "flow") or spec.get("filename_pattern") != SLUG:
            raise _error(
                "invalid-profile", f"Invalid selector kind or slug pattern: {name}"
            )
        if not isinstance(route, str):
            raise _error("invalid-profile", f"Invalid route: {name}")
        parts = _safe_rel(route, "Class route", "invalid-profile").parts
        if parts[: len(root.parts)] != root.parts or len(parts) <= len(root.parts):
            raise _error("invalid-profile", f"Route escapes docs_root: {name}")
        if route.count("{name}") != 1 or route.count("{layer}") != (
            1 if kind == "api" else 0
        ):
            raise _error("invalid-profile", f"Wrong route placeholders: {name}")
        if kind in ("flat", "api") and parts[-1] != "{name}.md":
            raise _error(
                "invalid-profile", f"API/flat route needs a name filename: {name}"
            )
        if kind == "flow" and (
            parts[-1]
            not in ("index.md", "definition.md", "implementation.md", "views.md")
            or "{name}" not in parts[:-1]
        ):
            raise _error(
                "invalid-profile",
                f"Flow route needs a flow directory and role filename: {name}",
            )
        for i, part in enumerate(parts):
            if i == len(parts) - 1:
                if part not in (
                    "{name}.md",
                    "index.md",
                    "definition.md",
                    "implementation.md",
                    "views.md",
                ):
                    raise _error("invalid-profile", f"Invalid route filename: {name}")
            elif part not in ("{name}", "{layer}") and not re.fullmatch(SLUG, part):
                raise _error("invalid-profile", f"Invalid route component: {name}")
        reserved = spec.get("reserved_names", [])
        layers = spec.get("allowed_layers", [])
        for values in (reserved, layers):
            if (
                not isinstance(values, list)
                or any(
                    not isinstance(v, str) or not re.fullmatch(SLUG, v) for v in values
                )
                or len(set(values)) != len(values)
            ):
                raise _error("invalid-profile", f"Invalid route vocabulary: {name}")
        if (kind == "api" and not layers) or (
            kind != "api" and "allowed_layers" in spec
        ):
            raise _error("invalid-profile", f"Invalid allowed_layers: {name}")
    for (a, first), (b, second) in combinations(classes.items(), 2):
        if overlap(first, second):
            raise _error("invalid-profile", f"Overlapping routes: {a} and {b}")


def _accepts(component, spec, value):
    if component == "{layer}":
        return value in spec["allowed_layers"]
    if component in ("{name}", "{name}.md"):
        if component.endswith(".md"):
            if not value.endswith(".md"):
                return False
            value = value[:-3]
        return bool(re.fullmatch(SLUG, value)) and value not in spec.get(
            "reserved_names", []
        )
    return component == value


def overlap(first, second):
    a, b = first["path_template"].split("/"), second["path_template"].split("/")
    if len(a) != len(b):
        return False
    for x, y in zip(a, b):
        if x in ("{name}", "{name}.md") and y in ("{name}", "{name}.md"):
            if x != y:
                return False
        elif x == "{layer}":
            if not any(_accepts(y, second, v) for v in first["allowed_layers"]):
                return False
        elif y == "{layer}":
            if not any(_accepts(x, first, v) for v in second["allowed_layers"]):
                return False
        elif not (_accepts(x, first, y) or _accepts(y, second, x)):
            return False
    return True


def resolve(profile, selection):
    spec = profile["document_classes"].get(selection["document_class"])
    if spec is None:
        raise _error(
            "invalid-request", f"Unknown document class: {selection['document_class']}"
        )
    layer, name = selection["layer"], selection["name"]
    if spec.get("selector_kind", "flat") == "api":
        if layer not in spec["allowed_layers"]:
            raise _error(
                "invalid-request", "layer must select one configured API layer"
            )
    elif layer is not None:
        raise _error("invalid-request", "layer is only valid for an API class")
    if name in spec.get("reserved_names", []):
        raise _error(
            "invalid-document-name", f"Reserved document name: {name}", "DOC-001"
        )
    return spec["path_template"].replace("{name}", name).replace("{layer}", layer or "")


def reverse(spec, relative, document_class):
    route, parts = spec["path_template"].split("/"), relative.split("/")
    if len(route) != len(parts):
        return None
    selection = dict(document_class=document_class, name=None, layer=None)
    for component, value in zip(route, parts):
        if not _accepts(component, spec, value):
            return None
        if component == "{layer}":
            selection["layer"] = value
        elif component in ("{name}", "{name}.md"):
            selection["name"] = value[:-3] if component.endswith(".md") else value
    return selection


def validate_family(profile, children):
    """Children have already passed their independent byte/descriptor checks."""
    all_packs = []

    def visit(c):
        all_packs.append(c)
        for child in c.get("compatibility_contracts", []):
            visit(child)

    for child in children:
        visit(child)
    releases, classes = {}, dict(profile["document_classes"])
    own_key = (profile["profile_id"], profile["version"])
    for pack in all_packs:
        p = pack["profile"]
        key = (p["profile_id"], p["version"])
        validate_routes(p)
        if (
            p["project"]["docs_root"] != profile["project"]["docs_root"]
            or key == own_key
        ):
            raise _error(
                "invalid-profile",
                "Compatibility namespace or release conflicts with selected pack",
            )
        if key in releases:
            previous = releases[key]
            closure = lambda c: {
                k: v[1] if isinstance(v, tuple) else v
                for k, v in c["resources"].items()
            }
            if (
                key != ("xdesign-sysml-docs", "1.0.0")
                or pack["contract_sha256"]
                != "1fc588f652dec5e36504b9a2c9046539857094177cdfb790d88626174d2926a6"
                or closure(previous) != closure(pack)
            ):
                raise _error(
                    "invalid-profile", "Duplicate or conflicting compatibility release"
                )
        else:
            releases[key] = pack
        for name, spec in p["document_classes"].items():
            if name in classes:
                old = classes[name]
                if old["path_template"] != spec["path_template"] or old.get(
                    "selector_kind", "flat"
                ) != spec.get("selector_kind", "flat"):
                    raise _error(
                        "invalid-profile", f"Conflicting bound route for {name}"
                    )
            else:
                classes[name] = spec
    validate_routes({**profile, "document_classes": classes})
    declarations = list(profile["document_classes"].items())
    for pack in all_packs:
        declarations.extend(pack["profile"]["document_classes"].items())
    for (a, first), (b, second) in combinations(declarations, 2):
        if a != b and overlap(first, second):
            raise _error("invalid-profile", f"Overlapping bound routes: {a} and {b}")
