"""A compact file-plan projection from a declared SysML view's compiler export."""

from __future__ import annotations

from html import escape

from .declared_views import DeclaredViews
from .renderer import Projection, _name


def _attribute(projection: Projection, entry: str, name: str) -> str:
    for child in projection.children[entry]:
        if _name(projection.index[child]) != name:
            continue
        values = [projection.index[item].get("value") for item in projection.descendants(child)
                  if projection.index[item].get("@type") == "LiteralString"]
        if len(values) == 1 and isinstance(values[0], str) and values[0]:
            return values[0]
    raise ValueError(f"File-plan entry {_name(projection.index[entry])} has no {name} value")


def _file_plan_groups(projection: Projection, root: str) -> tuple[list[tuple[str, list[tuple[str, str]]]], set[str]]:
    groups: list[tuple[str, list[tuple[str, str]]]] = []
    paths: set[str] = set()
    for group in projection.children[root]:
        element = projection.index[group]
        if element.get("@type") != "ItemUsage" or not element.get("qualifiedName"):
            continue
        files: list[tuple[str, str]] = []
        for entry in projection.children[group]:
            member = projection.index[entry]
            if member.get("@type") != "ItemUsage" or not member.get("qualifiedName"):
                continue
            path = _attribute(projection, entry, "repositoryPath")
            responsibility = _attribute(projection, entry, "responsibility")
            if path in paths:
                raise ValueError(f"Duplicate file-plan path: {path}")
            paths.add(path)
            files.append((path, responsibility))
        if files:
            groups.append((_name(element), files))
    if not groups or not paths:
        raise ValueError("File-plan view selected no repository files")
    return groups, paths


def render_file_plan(raw: list, guards: dict[str, str], target: str) -> tuple[str, dict]:
    """Render only file usages exposed through the requested native BrowserView."""
    declared = DeclaredViews(raw, guards)
    key = declared.projection.target(target)
    selection = declared.selection(key)
    projection = Projection(raw)
    roots = [item for item in selection["exposed"]
             if projection.index[item].get("qualifiedName")]
    if len(roots) != 1:
        raise ValueError("File-plan view must expose exactly one file-plan usage")
    groups, paths = _file_plan_groups(projection, roots[0])
    if projection.index[roots[0]].get("qualifiedName") == "XDesignDetailedDesign::simpleFlowFilePlan":
        return _render_simple_flow_file_plan(projection, key, groups, paths)
    return _render_file_plan_stacked(projection, key, roots[0], groups, paths)


def _render_simple_flow_file_plan(projection: Projection, key: str,
                                  groups: list[tuple[str, list[tuple[str, str]]]],
                                  paths: set[str]) -> tuple[str, dict]:
    """The reviewed SimpleFlow layout: fixed groups, card titles and positions."""
    by_name = dict(groups)
    expected = {"application", "checkCommands", "testCases"}
    if set(by_name) != expected:
        raise ValueError(f"File-plan groups differ from {sorted(expected)}")

    width = 1420
    height = max(138 + 66 + 55 * len(by_name["application"]),
                 515 + 66 + 55 * len(by_name["testCases"])) + 20
    body = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
            f'viewBox="0 0 {width} {height}" role="img" aria-labelledby="title desc">',
            '<title id="title">xdesign SimpleFlow implementation file plan</title>',
            '<desc id="desc">Compiler-derived repository source, checker, and verification file groups.</desc>',
            '<style>text{font-family:Arial,Helvetica,sans-serif;fill:#193044}'
            '.title{font-size:29px;font-weight:700;fill:#fff}'
            '.subtitle{font-size:15px;fill:#dbe9f1}'
            '.group{font-size:19px;font-weight:700;fill:#183a4a}'
            '.path{font-family:Menlo,Consolas,monospace;font-size:15px;font-weight:600}'
            '.detail{font-size:13px;fill:#526677}'
            '.count{font-size:13px;font-weight:700;fill:#246d73}</style>',
            f'<rect x="0" y="0" width="{width}" height="{height}" fill="#f5f8fa"/>',
            '<rect x="0" y="0" width="1420" height="112" fill="#153b50"/>',
            '<text x="40" y="51" class="title">xdesign SimpleFlow implementation file plan</text>',
            f'<text x="40" y="82" class="subtitle">{len(paths)} planned files · SysML view XDesignDetailedDesign::simpleFlowFilePlanView</text>']

    def card(x: int, y: int, w: int, title: str, files: list[tuple[str, str]], accent: str) -> None:
        row_height = 55
        h = 66 + row_height * len(files)
        body.extend([f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="12" fill="#fff" stroke="#cad8df"/>',
                     f'<rect x="{x}" y="{y}" width="{w}" height="8" rx="4" fill="{accent}"/>',
                     f'<text x="{x + 20}" y="{y + 39}" class="group">{escape(title)}</text>',
                     f'<text x="{x + w - 24}" y="{y + 39}" text-anchor="end" class="count">{len(files)} files</text>'])
        for number, (path, purpose) in enumerate(files):
            top = y + 60 + number * row_height
            if number % 2:
                body.append(f'<rect x="{x + 1}" y="{top}" width="{w - 2}" height="{row_height}" fill="#f5f9fa"/>')
            body.extend([f'<text x="{x + 20}" y="{top + 20}" class="path">{escape(path)}</text>',
                         f'<text x="{x + 20}" y="{top + 40}" class="detail">{escape(purpose)}</text>'])

    card(34, 138, 735, "xdesign crate and SimpleFlow modules", by_name["application"], "#20a496")
    card(793, 138, 593, "Fixed checker commands", by_name["checkCommands"], "#507fbc")
    card(793, 515, 593, "Executable verification", by_name["testCases"], "#b97948")
    body.append('</svg>')
    return "".join(body), {"name": projection.index[key].get("qualifiedName"),
                           "rendering": "File plan projection"}


def _render_file_plan_stacked(projection: Projection, key: str, root: str,
                              groups: list[tuple[str, list[tuple[str, str]]]],
                              paths: set[str]) -> tuple[str, dict]:
    """A generic vertical stack of group cards for any other file-plan usage."""
    width = 1420
    row_height = 55
    gap = 24
    cards: list[tuple[str, list[tuple[str, str]], int]] = []
    y = 138
    for name, files in groups:
        cards.append((name, files, y))
        y += 66 + row_height * len(files) + gap
    height = y - gap + 20

    root_name = _name(projection.index[root])
    view_name = projection.index[key].get("qualifiedName")
    accents = ["#20a496", "#507fbc", "#b97948", "#8a6fb8"]
    body = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
            f'viewBox="0 0 {width} {height}" role="img" aria-labelledby="title desc">',
            f'<title id="title">xdesign file plan: {escape(root_name)}</title>',
            '<desc id="desc">Compiler-derived repository file groups and their responsibilities.</desc>',
            '<style>text{font-family:Arial,Helvetica,sans-serif;fill:#193044}'
            '.title{font-size:29px;font-weight:700;fill:#fff}'
            '.subtitle{font-size:15px;fill:#dbe9f1}'
            '.group{font-size:19px;font-weight:700;fill:#183a4a}'
            '.path{font-family:Menlo,Consolas,monospace;font-size:15px;font-weight:600}'
            '.detail{font-size:13px;fill:#526677}'
            '.count{font-size:13px;font-weight:700;fill:#246d73}</style>',
            f'<rect x="0" y="0" width="{width}" height="{height}" fill="#f5f8fa"/>',
            f'<rect x="0" y="0" width="{width}" height="112" fill="#153b50"/>',
            f'<text x="40" y="51" class="title">xdesign file plan: {escape(root_name)}</text>',
            f'<text x="40" y="82" class="subtitle">{len(paths)} planned files · SysML view {escape(str(view_name))}</text>']

    for number, (name, files, top_y) in enumerate(cards):
        w = width - 68
        h = 66 + row_height * len(files)
        accent = accents[number % len(accents)]
        body.extend([f'<rect x="34" y="{top_y}" width="{w}" height="{h}" rx="12" fill="#fff" stroke="#cad8df"/>',
                     f'<rect x="34" y="{top_y}" width="{w}" height="8" rx="4" fill="{accent}"/>',
                     f'<text x="54" y="{top_y + 39}" class="group">{escape(name)}</text>',
                     f'<text x="{34 + w - 24}" y="{top_y + 39}" text-anchor="end" class="count">{len(files)} files</text>'])
        for index, (path, purpose) in enumerate(files):
            row = top_y + 60 + index * row_height
            if index % 2:
                body.append(f'<rect x="35" y="{row}" width="{w - 2}" height="{row_height}" fill="#f5f9fa"/>')
            body.extend([f'<text x="54" y="{row + 20}" class="path">{escape(path)}</text>',
                         f'<text x="54" y="{row + 40}" class="detail">{escape(purpose)}</text>'])

    body.append('</svg>')
    return "".join(body), {"name": view_name, "rendering": "File plan projection"}
