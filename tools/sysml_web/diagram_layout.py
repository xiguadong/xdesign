"""Lay out an explicit model projection with Graphviz, preserving native IDs.

This module does not read SysML or invent model relationships. Callers choose
the nodes, compartments, port anchors and relationships from compiler output;
Graphviz measures their labels and places and routes the resulting diagram.
Every input, Graphviz diagnostic and SVG is retained under the project cache.
"""
from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from html import escape
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
from textwrap import wrap
from xml.etree import ElementTree as ET


PROJECT_ROOT = Path(__file__).resolve().parents[2]
LAYOUT_CACHE = PROJECT_ROOT / "cache" / "sysml_web" / "graphviz"
SVG = "http://www.w3.org/2000/svg"
XLINK = "http://www.w3.org/1999/xlink"
ET.register_namespace("", SVG)
ET.register_namespace("xlink", XLINK)


@dataclass(frozen=True)
class DiagramPort:
    id: str
    label: str
    side: str = "E"


@dataclass(frozen=True)
class DiagramNode:
    id: str
    label: str
    kind: str = ""
    lines: tuple[str, ...] = ()
    shape: str = "box"
    group: str | None = None
    ports: tuple[DiagramPort, ...] = ()
    tooltip: str = ""


@dataclass(frozen=True)
class DiagramEdge:
    id: str
    source: str
    target: str
    label: str = ""
    kind: str = "directed"
    source_port: str | None = None
    target_port: str | None = None
    tooltip: str = ""
    constraint: bool = True


@dataclass(frozen=True)
class DiagramGroup:
    id: str
    label: str
    parent: str | None = None


def _id(prefix, value):
    return prefix + hashlib.sha256(value.encode("utf-8")).hexdigest()[:20]


def _clean(value):
    """Exclude characters that XML 1.0 cannot represent, without elision."""
    return "".join(c for c in str(value) if c in "\t\n\r"
                   or 0x20 <= ord(c) <= 0xD7FF or 0xE000 <= ord(c) <= 0xFFFD
                   or 0x10000 <= ord(c) <= 0x10FFFF)


def _quote(value):
    return json.dumps(_clean(value), ensure_ascii=False)


def _html(value):
    return escape(_clean(value), quote=True)


def _wrapped(value, width=36):
    # Wrapping sets a preferred line length; Graphviz measures the complete
    # text and grows the node to fit. No identifier or expression is elided.
    rows = []
    for paragraph in _clean(value).splitlines() or [""]:
        rows.extend(wrap(paragraph, width=width, break_long_words=False,
                         break_on_hyphens=False) or [""])
    return rows


def _html_text(value, width=36, align="LEFT"):
    return (f'<BR ALIGN="{align}"/>').join(_html(row) for row in _wrapped(value, width))


def _table(node):
    """Build measured compartments and visible, individually identified ports."""
    centered = node.shape in {"actor", "ellipse", "diamond", "state", "constraint"}
    align = "CENTER" if centered else "LEFT"
    rows = []
    if node.shape == "actor":
        rows.append('<TR><TD HEIGHT="90"> </TD></TR>')
    if node.kind:
        rows.append(f'<TR><TD ALIGN="{align}"><FONT POINT-SIZE="14" COLOR="#52657A">{_html_text(node.kind, 36, align)}</FONT></TD></TR>')
    rows.append(f'<TR><TD ALIGN="{align}"><FONT POINT-SIZE="16"><B>{_html_text(node.label, 32, align)}</B></FONT></TD></TR>')
    if node.lines:
        if node.shape in {"box", "constraint", "package"}:
            rows.append('<HR/>')
        rows.extend(f'<TR><TD ALIGN="LEFT"><FONT POINT-SIZE="14">{_html_text(line, 40)}</FONT></TD></TR>' for line in node.lines)
    border = "1" if node.shape in {"box", "constraint"} else "0"
    palette = {"requirement": "#FFF7E6", "verification": "#EDF7EF", "metadata": "#F3F5F7"}
    color = "#F2F7FC" if node.shape == "constraint" else palette.get(node.kind.lower(), "white")
    bgcolor = f' BGCOLOR="{color}"'
    center = f'<TABLE BORDER="{border}" COLOR="#647A91" CELLBORDER="0" CELLSPACING="0" CELLPADDING="8"{bgcolor}>{"".join(rows)}</TABLE>'
    if not node.ports:
        return center
    by_side = defaultdict(list)
    for port in node.ports:
        by_side[port.side.upper()].append(port)

    def square(port):
        port_id = _id("p", node.id + "\0" + port.id)
        return (f'<TD PORT="{port_id}" ID="{port_id}" HREF="#{port_id}" TOOLTIP="{_html(port.label)}" '
                'BORDER="1" COLOR="#334B64" BGCOLOR="#D9EAF7" WIDTH="12" HEIGHT="12" FIXEDSIZE="TRUE"> </TD>')

    def side_table(side):
        port_rows = []
        for port in by_side[side]:
            # Place the name inside the part, immediately beside the outer
            # terminal. Connectors leave the terminal away from that text.
            label = f'<TD><FONT POINT-SIZE="14">{_html_text(port.label, 20)}</FONT></TD>'
            cells = square(port) + label if side == "W" else label + square(port)
            port_rows.append(f'<TR>{cells}</TR>')
        return '<TABLE BORDER="0" CELLBORDER="0" CELLSPACING="4" CELLPADDING="0">' + ''.join(port_rows) + '</TABLE>'

    outer = []
    for side in ("N",):
        if by_side[side]:
            cells = ''.join(square(port) + f'<TD><FONT POINT-SIZE="14">{_html(port.label)}</FONT></TD>' for port in by_side[side])
            outer.append(f'<TR><TD COLSPAN="3"><TABLE BORDER="0" CELLSPACING="4"><TR>{cells}</TR></TABLE></TD></TR>')
    left = side_table("W") if by_side["W"] else " "
    right = side_table("E") if by_side["E"] else " "
    outer.append(f'<TR><TD>{left}</TD><TD>{center}</TD><TD>{right}</TD></TR>')
    if by_side["S"]:
        cells = ''.join(square(port) + f'<TD><FONT POINT-SIZE="14">{_html(port.label)}</FONT></TD>' for port in by_side["S"])
        outer.append(f'<TR><TD COLSPAN="3"><TABLE BORDER="0" CELLSPACING="4"><TR>{cells}</TR></TABLE></TD></TR>')
    return '<TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="0">' + ''.join(outer) + '</TABLE>'


def _dot_source(nodes, edges, title, question, groups, direction):
    if direction not in {"TB", "BT", "LR", "RL"}:
        raise ValueError("Diagram direction must be TB, BT, LR or RL.")
    by_id = {node.id: node for node in nodes}
    by_group = {group.id: group for group in groups}
    if len(by_id) != len(nodes) or len(by_group) != len(groups):
        raise ValueError("Diagram node and group IDs must be unique within their kind.")
    if not nodes:
        raise ValueError("Cannot lay out a diagram with no nodes.")
    aliases = {node.id: _id("n", node.id) for node in nodes}
    group_aliases = {group.id: _id("cluster_", group.id) for group in groups}
    port_ids = {}
    for node in nodes:
        if node.group is not None and node.group not in by_group:
            raise ValueError(f"Unknown group {node.group!r} for node {node.id!r}.")
        known = set()
        for port in node.ports:
            if port.id in known or port.side.upper() not in {"N", "E", "S", "W"}:
                raise ValueError(f"Duplicate port ID or unsupported port side on {node.id!r}.")
            known.add(port.id)
            port_ids[node.id, port.id] = _id("p", node.id + "\0" + port.id)
    children = defaultdict(list)
    for group in groups:
        if group.parent is not None and group.parent not in by_group:
            raise ValueError(f"Unknown parent group {group.parent!r}.")
    for group in groups:
        seen, current = {group.id}, group.parent
        while current is not None:
            if current in seen:
                raise ValueError("Diagram groups contain a parent cycle.")
            seen.add(current)
            current = by_group[current].parent
        children[group.parent].append(group)
    heading = f'<TABLE BORDER="0" CELLBORDER="0" CELLPADDING="4"><TR><TD><FONT POINT-SIZE="22"><B>{_html(title)}</B></FONT></TD></TR>'
    if question:
        heading += f'<TR><TD><FONT POINT-SIZE="14" COLOR="#52657A">{_html_text(question, 95, "CENTER")}</FONT></TD></TR>'
    heading += '</TABLE>'
    dot = ['digraph Model {', f'graph [rankdir={direction}, bgcolor="white", pad="0.25", nodesep="0.45", ranksep="0.65", splines=polyline, overlap=false, outputorder=edgesfirst, newrank=true, compound=true, fontname="Arial", fontsize=16, labelloc=t, label=<{heading}>];',
           'node [fontname="Arial", fontsize=16, color="#647A91", fontcolor="#172B43", penwidth=1.3];',
           'edge [fontname="Arial", fontsize=13, color="#647A91", fontcolor="#334B64", penwidth=1.3, arrowsize=0.8];']

    def write_node(node):
        shape = {"box": "plain", "constraint": "plain", "actor": "box", "ellipse": "ellipse",
                 "diamond": "diamond", "state": "box", "package": "folder",
                 "initial": "circle", "final": "doublecircle", "fork": "rect", "join": "rect"}.get(node.shape)
        if not shape:
            raise ValueError(f"Unsupported diagram shape {node.shape!r}.")
        attributes = [f'id={_quote(aliases[node.id])}', f'shape={shape}',
                      f'tooltip={_quote(node.tooltip or node.label)}', 'margin="0.12,0.08"']
        if node.shape == "actor":
            attributes += ['color="transparent"', 'fillcolor="transparent"']
        elif node.shape in {"state", "ellipse", "diamond", "package"}:
            attributes += ['style="rounded,filled"' if node.shape == "state" else 'style="filled"', 'fillcolor="white"']
        if node.shape in {"initial", "final", "fork", "join"}:
            attributes += ['label=""', 'style=filled', 'fillcolor="#172B43"', 'width=0.16', 'height=0.16', 'fixedsize=true']
            if node.shape in {"fork", "join"}:
                attributes += ['width=0.65', 'height=0.08']
            if node.label:
                attributes += [f'xlabel={_quote(node.label)}']
        else:
            attributes.append(f'label=<{_table(node)}>')
        dot.append(f'{aliases[node.id]} [{", ".join(attributes)}];')

    def write_group(parent):
        for node in nodes:
            if node.group == parent:
                write_node(node)
        for group in children[parent]:
            dot.append(f'subgraph {group_aliases[group.id]} {{')
            dot.append(f'graph [id={_quote(group_aliases[group.id])}, label={_quote(group.label)}, labelloc=t, labeljust=l, fontsize=16, color="#91A9BD", penwidth=1.3, style="rounded", margin=20];')
            write_group(group.id)
            dot.append('}')

    write_group(None)
    edge_aliases = {}
    for number, edge in enumerate(edges):
        if edge.source not in by_id or edge.target not in by_id:
            raise ValueError(f"Edge {edge.id!r} has an endpoint absent from the diagram.")
        edge_alias = _id("e", f"{number}\0{edge.id}\0{edge.source}\0{edge.target}")
        edge_aliases[edge_alias] = edge
        endpoints = []
        for node_id, port_id in ((edge.source, edge.source_port), (edge.target, edge.target_port)):
            endpoint = aliases[node_id]
            if port_id is not None:
                if (node_id, port_id) not in port_ids:
                    raise ValueError(f"Unknown port {port_id!r} on edge {edge.id!r} endpoint {node_id!r}.")
                side = next(port.side.lower() for port in by_id[node_id].ports if port.id == port_id)
                endpoint += f':{port_ids[node_id, port_id]}:{side}'
            endpoints.append(endpoint)
        attributes = [f'id={_quote(edge_alias)}', f'label={_quote(chr(10).join(_wrapped(edge.label, 30)))}',
                      f'tooltip={_quote(edge.tooltip or edge.label)}']
        if edge.kind in {"binding", "connection", "association"}:
            attributes += ['dir=none']
        elif edge.kind == "specialization":
            # Rank the general type above the specific type, while preserving
            # the UML/SysML arrow pointing from specific to general. This is
            # layout direction only; native endpoint metadata stays unchanged.
            endpoints.reverse()
            attributes += ['dir=back', 'arrowtail=empty', 'arrowhead=none']
        elif edge.kind in {"composition", "containment"}:
            attributes += ['dir=both', 'arrowtail=diamond', 'arrowhead=none']
        elif edge.kind == "aggregation":
            attributes += ['dir=both', 'arrowtail=odiamond', 'arrowhead=none']
        else:
            attributes += ['arrowhead=vee']
        if edge.kind in {"dependency", "reference", "include", "extend"}:
            attributes += ['style=dashed']
        if not edge.constraint:
            attributes += ['constraint=false']
        dot.append(f'{endpoints[0]} -> {endpoints[1]} [{", ".join(attributes)}];')
    dot.append('}')
    return '\n'.join(dot), aliases, group_aliases, port_ids, edge_aliases


def _rectangle(polygon):
    """Turn an axis-aligned Graphviz table boundary into a semantic SVG rect."""
    points = [tuple(map(float, point.split(','))) for point in polygon.get("points", "").split()]
    xs, ys = {p[0] for p in points}, {p[1] for p in points}
    if len(xs) != 2 or len(ys) != 2:
        return None
    x, right, y, bottom = min(xs), max(xs), min(ys), max(ys)
    result = ET.Element(f"{{{SVG}}}rect", {key: value for key, value in polygon.attrib.items() if key != "points"})
    result.attrib.update(x=str(x), y=str(y), width=str(right - x), height=str(bottom - y))
    return result


def _decorate(svg, nodes, edges, title, question, groups, aliases, group_aliases, port_ids, edge_aliases):
    root = ET.fromstring(svg)
    preserved_port_groups = {"a_" + alias for alias in port_ids.values()}

    def unwrap_links(parent):
        for child in list(parent):
            unwrap_links(child)
            if child.tag == f"{{{SVG}}}a" or (child.tag == f"{{{SVG}}}g" and child.get("id", "").startswith("a_") and child.get("id") not in preserved_port_groups):
                position = list(parent).index(child)
                parent.remove(child)
                for grandchild in list(child):
                    parent.insert(position, grandchild)
                    position += 1

    # Graphviz tooltip wrappers carry no geometry. Unwrap them to retain a
    # simple SVG contract: semantic node/edge groups own their visible marks.
    unwrap_links(root)
    root.set("role", "img")
    root.set("aria-labelledby", "diagram-title diagram-desc")
    root.set("data-layout-engine", "graphviz-dot")
    accessible_title = ET.Element(f"{{{SVG}}}title", {"id": "diagram-title"})
    accessible_title.text = _clean(title)
    description = ET.Element(f"{{{SVG}}}desc", {"id": "diagram-desc"})
    description.text = _clean((question + ". " if question else "") + "Model-derived projection. Native element IDs and resolved relationship endpoints are retained; Graphviz measures labels and routes edges.")
    root.insert(0, accessible_title)
    root.insert(1, description)
    by_svg_id = {element.get("id"): element for element in root.iter() if element.get("id")}
    for node in nodes:
        group = by_svg_id[aliases[node.id]]
        group.set("data-native-id", node.id)
        group.set("data-shape", node.shape)
        if node.group is not None:
            group.set("data-group-id", node.group)
        group.set("class", "node diagram-node shape-" + node.shape)
        group.find(f"{{{SVG}}}title").text = _clean((node.tooltip or node.label) + "\n" + node.id)
        rectangles = []
        for parent in group.iter():
            for polygon in list(parent.findall(f"{{{SVG}}}polygon")):
                rectangle = _rectangle(polygon)
                if rectangle is not None:
                    position = list(parent).index(polygon)
                    parent.remove(polygon)
                    parent.insert(position, rectangle)
                    rectangles.append(rectangle)
        if rectangles:
            boundary = max(rectangles, key=lambda element: float(element.get("width")) * float(element.get("height")))
            if node.ports and node.shape in {"box", "constraint"}:
                # Graphviz's HTML terminals define routing anchors, while a
                # single encompassing outline passes through their centers.
                # This keeps ports on the part boundary instead of drawing
                # unrelated small boxes beside an independent body card.
                terminal_rectangles = []
                for port in node.ports:
                    terminal_group = by_svg_id["a_" + port_ids[node.id, port.id]]
                    terminal = terminal_group.find(f"{{{SVG}}}rect")
                    terminal_rectangles.append((port.side.upper(), terminal))
                x, y, width, height = (float(boundary.get(key)) for key in ("x", "y", "width", "height"))
                right, bottom = x + width, y + height
                for side, terminal in terminal_rectangles:
                    px, py, pw, ph = (float(terminal.get(key)) for key in ("x", "y", "width", "height"))
                    if side == "W":
                        x = min(x, px + pw / 2)
                    elif side == "E":
                        right = max(right, px + pw / 2)
                    elif side == "N":
                        y = min(y, py + ph / 2)
                    elif side == "S":
                        bottom = max(bottom, py + ph / 2)
                    if side in {"W", "E"}:
                        y, bottom = min(y, py - 8), max(bottom, py + ph + 8)
                    else:
                        x, right = min(x, px - 8), max(right, px + pw + 8)
                port_marks = {mark for port in node.ports
                              for mark in by_svg_id["a_" + port_ids[node.id, port.id]].iter(f"{{{SVG}}}rect")}
                for rectangle in rectangles:
                    if rectangle not in port_marks:
                        rectangle.set("fill", "none")
                        rectangle.set("stroke", "none")
                boundary = ET.Element(f"{{{SVG}}}rect", {"x": str(x), "y": str(y),
                                      "width": str(right - x), "height": str(bottom - y),
                                      "fill": "#F2F7FC" if node.shape == "constraint" else "white",
                                      "stroke": "#647A91", "stroke-width": "1.3"})
                group.insert(1, boundary)
            boundary.set("class", "node")
            if node.shape == "actor":
                x, y, width = (float(boundary.get(key)) for key in ("x", "y", "width"))
                cx, top = x + width / 2, y + 12
                boundary_parent = next(parent for parent in group.iter() if boundary in list(parent))
                boundary_parent.remove(boundary)
                circle = ET.Element(f"{{{SVG}}}circle", {"cx": str(cx), "cy": str(top + 12), "r": "11", "fill": "white", "stroke": "#334B64", "stroke-width": "1.5"})
                stick = ET.Element(f"{{{SVG}}}path", {"d": f"M {cx} {top + 23} V {top + 52} M {cx - 23} {top + 34} H {cx + 23} M {cx} {top + 52} L {cx - 21} {top + 76} M {cx} {top + 52} L {cx + 21} {top + 76}", "fill": "none", "stroke": "#334B64", "stroke-width": "1.7"})
                group.append(circle)
                group.append(stick)
        for text in group.iter(f"{{{SVG}}}text"):
            text.set("class", "name" if float(text.get("font-size", "14")) >= 16 else "feature")
        for port in node.ports:
            port_alias = port_ids[node.id, port.id]
            port_group = by_svg_id.get("a_" + port_alias)
            if port_group is not None:
                port_group.set("id", port_alias)
                port_group.set("class", "diagram-port")
                port_group.set("data-native-id", port.id)
                port_group.set("data-owner-id", node.id)
                port_group.set("data-side", port.side.upper())
    for group in groups:
        element = by_svg_id.get(group_aliases[group.id])
        if element is not None:
            element.set("data-native-id", group.id)
            element.set("class", "cluster diagram-group")
    for edge_alias, edge in edge_aliases.items():
        element = by_svg_id[edge_alias]
        # Existing consumers identify node groups by their SVG id and native
        # id, and relationship groups by explicit source/target attributes.
        element.attrib.pop("id", None)
        element.set("data-native-id", edge.id)
        element.set("data-source-id", edge.source)
        element.set("data-target-id", edge.target)
        element.set("data-edge-kind", edge.kind)
        if edge.source_port is not None:
            element.set("data-source-port-id", edge.source_port)
        if edge.target_port is not None:
            element.set("data-target-port-id", edge.target_port)
        element.find(f"{{{SVG}}}title").text = _clean(edge.label + "\n" + edge.tooltip + "\n" + edge.id)
        for text in element.iter(f"{{{SVG}}}text"):
            text.set("class", "edge-label")
    return ET.tostring(root, encoding="unicode")


def render_diagram(nodes, edges, title, question="", groups=(), direction="TB") -> str:
    """Return a standalone SVG for an explicit, native-ID model projection.

    ``shape`` supports box, constraint, ellipse, diamond, state, package and
    actor, plus initial/final/fork/join control nodes. Port IDs are native
    model IDs; edges can select them by ``source_port`` and ``target_port``.
    ``DiagramEdge.constraint=False`` retains a relationship without using it
    to determine ranks, which is useful for references back to an owning type.
    Graphviz is a required layout dependency; failures retain diagnostics and
    raise an error rather than falling back to an unreadable fixed grid.
    """
    nodes, edges, groups = tuple(nodes), tuple(edges), tuple(groups)
    source, aliases, group_aliases, port_ids, edge_aliases = _dot_source(nodes, edges, title, question, groups, direction)
    configured = os.environ.get("SYSML_GRAPHVIZ_DOT", "dot")
    executable = shutil.which(configured)
    if not executable:
        raise RuntimeError("Graphviz 'dot' is required for diagram layout. Install Graphviz or set SYSML_GRAPHVIZ_DOT to its executable path.")
    version = subprocess.run([executable, "-V"], capture_output=True, text=True, timeout=10)
    version_text = (version.stderr or version.stdout).strip()
    digest = hashlib.sha256((version_text + "\0" + source).encode()).hexdigest()[:24]
    run = LAYOUT_CACHE / digest
    run.mkdir(parents=True, exist_ok=True)
    (run / "diagram.dot").write_text(source, encoding="utf-8")
    environment = dict(os.environ, TMPDIR=str(run), TMP=str(run), TEMP=str(run),
                       XDG_CACHE_HOME=str(LAYOUT_CACHE / "system-cache"))
    try:
        result = subprocess.run([executable, "-Tsvg"], input=source, capture_output=True,
                                text=True, cwd=run, env=environment, timeout=45)
    except (OSError, subprocess.TimeoutExpired) as exc:
        (run / "diagnostics.json").write_text(json.dumps({"executable": executable, "version": version_text, "error": str(exc)}, indent=2), encoding="utf-8")
        raise RuntimeError(f"Graphviz layout failed; see {run / 'diagnostics.json'}: {exc}") from exc
    diagnostics = {"executable": executable, "version": version_text, "returncode": result.returncode, "stderr": result.stderr}
    (run / "diagnostics.json").write_text(json.dumps(diagnostics, indent=2), encoding="utf-8")
    (run / "graphviz.svg").write_text(result.stdout, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"Graphviz layout failed; see {run / 'diagnostics.json'}: {result.stderr.strip()}")
    svg = _decorate(result.stdout, nodes, edges, title, question, groups, aliases, group_aliases, port_ids, edge_aliases)
    (run / "diagram.svg").write_text(svg, encoding="utf-8")
    return svg
