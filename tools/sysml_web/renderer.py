"""Render the Rust compiler's native interchange graph as standalone SVG.

This is a documented projection of named declarations and resolved relations,
not a second SysML parser. All joins use the compiler's native element IDs.
The full export remains the authority for omitted details and library elements.
"""
from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from html import escape
import hashlib
try:
    from .diagram_layout import DiagramNode, DiagramEdge, render_diagram
except ImportError:
    from diagram_layout import DiagramNode, DiagramEdge, render_diagram


MAX_GRAPH_NODES = 160
MAX_TREE_NODES = 1000
MAX_FEATURE_LINES = 11
CONNECTORS = {
    "ConnectionUsage", "InterfaceUsage", "BindingConnectorAsUsage",
    "BindingConnector", "SuccessionAsUsage", "Succession", "FlowUsage",
    "AllocationUsage", "TransitionUsage",
}


def _ids(value):
    if isinstance(value, dict):
        value = [value]
    if not isinstance(value, list):
        return []
    return [item["@id"] for item in value
            if isinstance(item, dict) and isinstance(item.get("@id"), str)]


def _one(value):
    items = _ids(value)
    return items[0] if len(items) == 1 else None


def _text(value, limit=66):
    value = " ".join(str(value).split())
    return value if len(value) <= limit else value[:limit - 1] + "…"


def _xml(value):
    # XML 1.0 rejects control characters even inside escaped text.
    clean = "".join(c for c in str(value)
                    if c in "\t\n\r" or 0x20 <= ord(c) <= 0xD7FF
                    or 0xE000 <= ord(c) <= 0xFFFD or 0x10000 <= ord(c) <= 0x10FFFF)
    return escape(clean, quote=True)


def _name(element):
    return element.get("name") or element.get("declaredName") or element["@id"]


def _named(element):
    return bool(element.get("declaredName") or
                (element.get("name") and element.get("qualifiedName")))


def _svg_id(element_id):
    return "n" + hashlib.sha256(element_id.encode()).hexdigest()[:20]


@dataclass(frozen=True)
class Edge:
    source: str
    target: str
    kind: str
    label: str
    native_id: str
    detail: str = ""


class Projection:
    def __init__(self, raw):
        if not isinstance(raw, list) or any(
                not isinstance(e, dict) or not isinstance(e.get("@id"), str)
                or not isinstance(e.get("@type"), str) for e in raw):
            raise ValueError("Expected the Rust compiler's interchange array with @id and @type.")
        self.index = {element["@id"]: element for element in raw}
        if len(self.index) != len(raw):
            raise ValueError("Compiler export contains duplicate native element IDs.")
        self.local = {key: element for key, element in self.index.items()
                      if not element.get("isLibraryElement")}
        self.children = defaultdict(list)
        for key, element in self.local.items():
            parent = _one(element.get("owner"))
            if parent in self.local:
                self.children[parent].append(key)

    def ordered(self, ids):
        return sorted(ids, key=lambda key: (
            self.index[key].get("qualifiedName") or _name(self.index[key]), key))

    def target(self, asked):
        if not asked:
            return None
        matches = [key for key, e in self.local.items()
                   if key == asked or e.get("qualifiedName") == asked]
        if not matches:
            matches = [key for key, e in self.local.items()
                       if e.get("name") == asked or e.get("declaredName") == asked]
        if len(matches) != 1:
            reason = "ambiguous" if matches else "not found"
            raise ValueError(f"Target {asked!r} is {reason}; use a qualified name or native ID.")
        return matches[0]

    def descendants(self, root):
        result, pending = {root}, [root]
        while pending:
            current = pending.pop()
            for child in self.children[current]:
                if child not in result:
                    result.add(child)
                    pending.append(child)
        return result

    def type_names(self, element):
        return [_name(self.index[key]) for key in _ids(element.get("type"))
                if key in self.index and _named(self.index[key])]

    def multiplicity(self, element):
        value = self.index.get(_one(element.get("multiplicity")), {})
        def literal(field):
            bound = self.index.get(_one(value.get(field)), {})
            if bound.get("@type") == "LiteralInfinity":
                return "*"
            if bound.get("@type") == "LiteralInteger":
                return str(bound.get("value"))
            return None
        lower, upper = literal("lowerBound"), literal("upperBound")
        if lower is None or upper is None:
            return ""
        return f" [{lower}]" if lower == upper else f" [{lower}..{upper}]"

    def member_label(self, element):
        kind = element["@type"]
        prefix = kind.removesuffix("Usage").removesuffix("Definition")
        if prefix == "Enumeration":
            prefix = "enum"
        if not element.get("isComposite") and kind in {"PartUsage", "ItemUsage"}:
            prefix = "ref " + prefix.lower()
        types = self.type_names(element)
        typed = " : " + ", ".join(types) if types else ""
        return f"{prefix.lower()} {_name(element)}{typed}{self.multiplicity(element)}"

    def compartments(self, key):
        features = [self.index[child] for child in self.children[key]
                    if _named(self.index[child]) and
                    self.index[child]["@type"].endswith(("Usage", "Definition"))]
        return [self.member_label(e) for e in features]

    def definitions(self, target):
        selected = {key for key, e in self.local.items()
                    if e["@type"].endswith("Definition") and _named(e)}
        if target:
            selected &= self.descendants(target)
        if not selected:
            raise ValueError("No named definitions in this selection; use the membership tree view.")
        edges = []
        for key in selected:
            element = self.index[key]
            owner = _one(element.get("owner"))
            membership = _one(element.get("owningMembership"))
            if owner in selected and membership in self.index:
                edges.append(Edge(owner, key, "containment", "contains definition", membership,
                                  f"Native owningMembership of {key}"))
        for relation in self.local.values():
            kind, native_id = relation["@type"], relation["@id"]
            if kind in {"Subclassification", "Specialization"}:
                source = _one(relation.get("specific") or relation.get("subclassifier"))
                dest = _one(relation.get("general") or relation.get("superclassifier"))
                if source in selected and dest in selected:
                    edges.append(Edge(source, dest, "specialization", "specializes", native_id))
            elif kind == "FeatureTyping":
                feature_id = _one(relation.get("typedFeature"))
                feature = self.local.get(feature_id, {})
                owner = _one(feature.get("owner"))
                dest = _one(relation.get("type"))
                if owner in selected and dest in selected and _named(feature):
                    composite = bool(feature.get("isComposite"))
                    label = ("owns " if composite else "references ") + _name(feature)
                    edges.append(Edge(owner, dest, "composition" if composite else "reference",
                                      label + self.multiplicity(feature), native_id,
                                      f"FeatureTyping of native feature {feature_id}"))
            elif kind == "Dependency":
                for source in _ids(relation.get("client")):
                    for dest in _ids(relation.get("supplier")):
                        if source in selected and dest in selected:
                            edges.append(Edge(source, dest, "dependency", "depends", native_id))
        return self.ordered(selected), list(dict.fromkeys(edges)), "Named definitions", []

    def endpoint(self, end_id, visible):
        """Select the direct part from a compiler-resolved endpoint path."""
        end = self.index.get(end_id, {})
        chain = _ids(end.get("chainingFeature"))
        if chain:
            if chain[0] in visible:
                names = [_name(self.index[key]) for key in chain if key in self.index]
                return chain[0], ".".join(names)
            return None
        if end_id in visible:
            return end_id, _name(end)
        # An anonymous end may reference the visible part through its native
        # ReferenceSubsetting. Never infer a usage from a type's owner.
        candidates = []
        for reference in _ids(end.get("ownedReferenceSubsetting")):
            candidates += _ids(self.index.get(reference, {}).get("referencedFeature"))
        candidates = list(dict.fromkeys(candidates))
        if len(candidates) == 1 and candidates[0] in visible:
            return candidates[0], _name(self.index[candidates[0]])
        return None

    def internal(self, target):
        if not target:
            raise ValueError("Internal view requires a named definition or usage target.")
        selected = {child for child in self.children[target]
                    if self.index[child]["@type"] in {"PartUsage", "ItemUsage", "PortUsage"}
                    and _named(self.index[child])}
        if not selected:
            raise ValueError("This target has no directly owned named parts, items or ports to render.")
        edges, notes = [], []
        for child in self.children[target]:
            relation = self.index[child]
            kind = relation["@type"]
            if kind not in CONNECTORS:
                continue
            end_ids = _ids(relation.get("endFeature"))
            if not end_ids:
                end_ids = _ids(relation.get("relatedFeature"))
            if kind == "TransitionUsage":
                end_ids = _ids(relation.get("source")) + _ids(relation.get("target"))
            endpoints = [self.endpoint(key, selected) for key in end_ids]
            if len(endpoints) != 2 or any(end is None for end in endpoints):
                notes.append(f"{kind} {relation['@id']}: no complete pair of direct-part endpoint paths.")
                continue
            a, b = endpoints
            if a[0] == b[0]:
                notes.append(f"{kind} {relation['@id']}: endpoints share a part; inspect the raw AST for its internal path.")
                continue
            edge_kind = "binding" if kind.startswith("Binding") else "connection"
            if kind in {"FlowUsage", "SuccessionAsUsage", "Succession", "TransitionUsage", "AllocationUsage"}:
                edge_kind = "directed"
            separator = " = " if edge_kind == "binding" else " → " if edge_kind == "directed" else " ↔ "
            edges.append(Edge(a[0], b[0], edge_kind, a[1] + separator + b[1], child,
                              f"{kind}; endpoint paths from compiler endFeature / chainingFeature / ReferenceSubsetting"))
        title = "Internal structure · " + (self.index[target].get("qualifiedName") or _name(self.index[target]))
        return self.ordered(selected), edges, title, notes

    def tree(self, target, selection=None, title="Membership tree"):
        selected = {key for key, e in self.local.items() if _named(e)}
        if selection is not None:
            selected &= set(selection)
        elif target:
            selected &= self.descendants(target)
        if not selected:
            raise ValueError("No named declarations to render in this selection.")
        if len(selected) > MAX_TREE_NODES:
            raise ValueError(f"Membership tree contains {len(selected)} nodes; select a target (limit {MAX_TREE_NODES}).")
        children, roots = defaultdict(list), []
        for key in selected:
            parent, seen = _one(self.index[key].get("owner")), {key}
            while parent in self.local and parent not in selected and parent not in seen:
                seen.add(parent)
                parent = _one(self.index[parent].get("owner"))
            if parent in selected and parent not in seen:
                children[parent].append(key)
            else:
                roots.append(key)
        rows, visited = [], set()
        def visit(key):
            pending = [(key, 0)]
            while pending:
                current, depth = pending.pop()
                if current in visited:
                    continue
                visited.add(current)
                rows.append((current, min(depth, 30)))
                pending.extend((child, depth + 1)
                               for child in reversed(self.ordered(children[current])))
        for key in self.ordered(roots):
            visit(key)
        for key in self.ordered(selected - visited):
            visit(key)
        width = max(960, max(depth for _, depth in rows) * 26 + 850)
        height = 128 + len(rows) * 34
        body = []
        for row, (key, depth) in enumerate(rows):
            element = self.index[key]
            x, y = 36 + depth * 26, 112 + row * 34
            qualified = element.get("qualifiedName") or _name(element)
            body.append(f'<g data-native-id="{_xml(key)}"><title>{_xml(qualified)} · {_xml(key)}</title>')
            if depth:
                body.append(f'<path d="M {x - 17} {y - 12} V {y} H {x - 5}" class="tree-line"/>')
            body.append(f'<text x="{x}" y="{y}" class="tree-name">{_xml(_text(_name(element), 76))}</text>')
            body.append(f'<text x="{width - 300}" y="{y}" class="kind">{_xml(_text(element["@type"], 34))}</text></g>')
        return _document(width, height, title, f"{len(rows)} named declarations · native owner relationships", "".join(body))


def _document(width, height, title, subtitle, body):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="diagram-title diagram-desc">
<title id="diagram-title">{_xml(title)}</title>
<desc id="diagram-desc">{_xml(subtitle)}. Generated from Rust compiler interchange JSON; named declarations and resolved native relationships.</desc>
<defs>
  <marker id="arrow" markerWidth="9" markerHeight="9" refX="8" refY="4.5" orient="auto"><path d="M1 1 L8 4.5 L1 8" fill="none" stroke="#64748b" stroke-width="1.4"/></marker>
  <marker id="triangle" markerWidth="12" markerHeight="12" refX="11" refY="6" orient="auto"><path d="M1 1 L11 6 L1 11 Z" fill="#ffffff" stroke="#64748b"/></marker>
  <marker id="diamond" markerWidth="12" markerHeight="9" refX="1" refY="4.5" orient="auto"><path d="M1 4.5 L6 1 L11 4.5 L6 8 Z" fill="#334155"/></marker>
</defs>
<style>text{{font-family:system-ui,-apple-system,sans-serif;fill:#172b43}}.title{{font-size:22px;font-weight:650}}.subtitle{{font-size:12px;fill:#63748a}}.node{{fill:#fff;stroke:#a9bbce;stroke-width:1.4}}.header{{fill:#edf4fb}}.name{{font-size:16px;font-weight:650}}.kind{{font-size:11px;fill:#61748b}}.feature{{font-size:12px;fill:#33465c}}.edge{{fill:none;stroke:#8192a8;stroke-width:1.2}}.edge-label{{font-size:10px;fill:#52677f;paint-order:stroke;stroke:#f6f9fc;stroke-width:4px;stroke-linejoin:round}}.tree-line{{fill:none;stroke:#c6d4e3}}.tree-name{{font-size:14px}}.note{{font-size:11px;fill:#755e31}}</style>
<rect width="100%" height="100%" fill="#f6f9fc"/>
<text x="30" y="39" class="title">{_xml(_text(title, 98))}</text>
<text x="30" y="62" class="subtitle">{_xml(_text(subtitle, 140))}</text>
{body}
</svg>'''


def _graph(projection, nodes, edges, title, notes, internal=False, notation=None, layout_rows=None):
    """Present a compiler projection using measured labels and routed edges.

    ``layout_rows`` is accepted for older callers; Graphviz now determines
    placement from the native relationships rather than arbitrary row indexes.
    Documentation compositions can pass explicit groups, ports and shapes to
    ``render_diagram`` directly without changing the model projection.
    """
    if len(nodes) > MAX_GRAPH_NODES:
        raise ValueError(f"Diagram contains {len(nodes)} nodes; select a narrower target (limit {MAX_GRAPH_NODES}).")
    displayed = []
    for key in nodes:
        element = projection.index[key]
        lines = projection.compartments(key)
        if internal:
            types = projection.type_names(element)
            if types:
                lines = ["type: " + ", ".join(types)] + lines
        visible = lines[:MAX_FEATURE_LINES]
        if len(lines) > MAX_FEATURE_LINES:
            visible.append(f"… {len(lines) - MAX_FEATURE_LINES} more direct features; see native export")
        kind = element["@type"]
        shape = "box"
        if notation == "use_cases":
            if key in getattr(projection, "actors", set()):
                shape = "actor"
                kind = "actor"
            elif kind in {"UseCaseDefinition", "UseCaseUsage"}:
                shape = "ellipse"
        elif notation == "packages":
            shape = "package"
        elif notation == "activity" and kind in {"DecisionNode", "MergeNode"}:
            shape = "diamond"
        elif notation in {"activity", "state_machine"}:
            shape = "state"
        if key in getattr(projection, "subjects", set()):
            kind = "subject"
        elif notation:
            kind = kind.removesuffix("Usage").removesuffix("Definition")
        qualified = element.get("qualifiedName") or _name(element)
        displayed.append(DiagramNode(key, _name(element), kind, tuple(visible), shape,
                                     tooltip=qualified + "\n" + "\n".join(lines)))
    relationships = [DiagramEdge(edge.native_id, edge.source, edge.target, edge.label,
                                 edge.kind, tooltip=edge.detail) for edge in edges]
    question = f"{len(nodes)} nodes · {len(edges)} resolved relationships · native element IDs retained"
    if notes:
        question += f" · {len(notes)} unresolved endpoint projections omitted; inspect native export"
    svg = render_diagram(displayed, relationships, title, question=question)
    if notes:
        # Preserve exact omission diagnostics in accessible SVG metadata.
        svg = svg.replace("</desc>", "\n" + _xml("\n".join(notes)) + "</desc>", 1)
    return svg


def render_model(raw: list, view="definitions", target="") -> str:
    """Return SVG from native compiler JSON; reject unsupported selections.

    ``definitions`` draws named reusable definitions, direct feature labels,
    native subclassification and declared feature typing relations. ``tree``
    draws named declarations using native ownership. ``internal`` draws direct
    named parts/items/ports and connector paths of a requested target. Endpoint
    paths that cannot be represented remain explicit omission notes in the SVG.
    """
    projection = Projection(raw)
    selected = projection.target(target)
    if view in {"tree", "browser"}:
        return projection.tree(selected)
    if view == "definitions":
        nodes, edges, title, notes = projection.definitions(selected)
    elif view == "internal":
        nodes, edges, title, notes = projection.internal(selected)
    else:
        raise ValueError(f"Unsupported AST view {view!r}; use definitions, tree or internal.")
    return _graph(projection, nodes, edges, title, notes, internal=view == "internal")
