#!/usr/bin/env python3
"""Check documentation layout, model locators, and model-derived figure blocks.

Run from any directory with ``python3 tools/check_documentation.py --json``.
The SysML compiler remains responsible for parsing and resolving model names;
this command checks the repository paths and Markdown targets that it cannot.
"""

from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path, PurePosixPath
import re


REPO = Path(__file__).resolve().parents[1]
ANNOTATION = re.compile(
    r"@(?:XDesignCommon::)?(?P<kind>DocSection|DocFigure)\s*\{(?P<body>[^}]*)\}",
    re.DOTALL,
)
ANNOTATION_START = re.compile(r"@(?:XDesignCommon::)?(?:DocSection|DocFigure)\b")
FIELD = re.compile(r'(?P<name>[A-Za-z]\w*)\s*=\s*"(?P<value>[^"]*)"')
ANCHOR = re.compile(r'<a\s+id="([^"]+)"\s*></a>', re.IGNORECASE)
SECTION_HEADING = re.compile(r"^#{2,4}\s+([A-Z][0-9]+)\b")
IMAGE = re.compile(r'!\[[^\]]*\]\(\s*(?:<(?P<bracketed>[^>]+)>|(?P<plain>[^\s)]+))')
IMAGE_START = re.compile(r'!\[')
HTML_IMAGE_START = re.compile(r'<img\b', re.IGNORECASE)
EXTERNAL_IMAGE = re.compile(r'^(?:https?://|data:|//)', re.IGNORECASE)
INLINE_CODE = re.compile(r'(?P<ticks>`+).*?(?P=ticks)')
BLOCK_LABELS = ("Question", "Model view", "Companions", "Status", "Reading", "Limits")
STATUSES = {
    "renderable-now", "planned", "table-substitute", "external-tool",
    "designed-only", "implemented",
}
RULE_CODES = {
    "directory-layout": {
        "missing-root-entry", "missing-overview", "invalid-document-name", "reserved-overview-index",
        "duplicate-document-index", "document-index-gap", "markdown-in-assets",
    },
    "section-target": {
        "invalid-document-path", "missing-document", "missing-section-id",
        "missing-section-anchor", "duplicate-anchor", "anchor-heading-mismatch",
    },
    "figure-target": {
        "invalid-figure-path", "missing-figure", "figure-directory-mismatch",
        "figure-prefix-mismatch", "figure-embed-count", "figure-section-placement",
    },
    "figure-block": {
        "missing-figure-block", "duplicate-figure-block", "figure-section-mismatch",
        "duplicate-figure-number", "figure-field-order", "missing-figure-field",
        "invalid-figure-status",
    },
    "embed-source": {
        "invalid-image-path", "missing-image", "image-directory-mismatch", "duplicate-image",
    },
}


def _visible_markdown(content: str) -> str:
    """Mask fenced examples while retaining line numbers for real anchors and images."""
    output: list[str] = []
    fence: tuple[str, int] | None = None
    for line in content.splitlines(keepends=True):
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            run = marker.group(1)
            if fence is None:
                fence = (run[0], len(run))
            elif run[0] == fence[0] and len(run) >= fence[1]:
                fence = None
        if fence is None and not marker:
            output.append(INLINE_CODE.sub(lambda match: " " * len(match.group()), line))
        else:
            output.append("".join("\n" if char == "\n" else " " for char in line))
    return "".join(output)


def _target(root: Path, value: str, directory: str, suffix: str) -> Path | None:
    """Accept only repository-relative, normalized paths in the expected directory."""
    posix = PurePosixPath(value)
    if (not value or posix.is_absolute() or ".." in posix.parts
            or posix.as_posix() != value or not value.startswith(directory + "/")
            or posix.suffix != suffix):
        return None
    target = root.joinpath(*posix.parts)
    resolved = target.resolve()
    if (not resolved.is_relative_to(root.resolve())
            or not resolved.is_relative_to((root / directory).resolve())):
        return None
    return target


def _readable(path: Path) -> bool:
    try:
        with path.open("rb") as source:
            source.read(1)
    except OSError:
        return False
    return True


def _in_assets(path: Path, docs_root: Path) -> bool:
    """Recognize an assets directory at any depth below docs."""
    return "assets" in path.parent.relative_to(docs_root).parts


def _figure_block(lines: list[str], image_line: int) -> tuple[str, list[tuple[str, str]], int, bool] | None:
    """Read the compact blockquote immediately following an embedded figure."""
    cursor = image_line + 1
    while cursor < len(lines) and not lines[cursor].strip():
        cursor += 1
    block: list[str] = []
    while cursor < len(lines) and lines[cursor].lstrip().startswith(">"):
        block.append(lines[cursor].lstrip()[1:].strip())
        cursor += 1
    if not block:
        return None
    title = re.match(r"\*\*Figure\s+([A-Z][0-9]+-[0-9]+)\s+[—-]\s+[^*]+\*\*", block[0])
    if not title:
        return None
    values: list[tuple[str, str]] = []
    repeated_title = False
    for line in block[1:]:
        if re.match(r"\*\*Figure\s+", line):
            repeated_title = True
            continue
        match = re.match(r"\*\*(Question|Model view|Companions|Status|Reading|Limits):\*\*\s*(.*)", line)
        if match:
            values.append((match.group(1), match.group(2).strip()))
        elif values:
            # Markdown continuation of a long field, most often the reading note.
            label, value = values[-1]
            values[-1] = (label, f"{value} {line}".strip())
    return title.group(1), values, cursor, repeated_title


def inspect(root: Path = REPO) -> dict:
    root = root.resolve()
    diagnostics: list[dict[str, object]] = []
    counts: Counter[str] = Counter()

    def report(code: str, source: Path, line: int, detail: str) -> None:
        rule = next((name for name, codes in RULE_CODES.items() if code in codes), "input")
        diagnostics.append({
            "rule": rule,
            "code": code,
            "file": source.relative_to(root).as_posix(),
            "line": line,
            "detail": detail,
        })

    documents = sorted((root / "docs").rglob("*.md"))
    if not documents:
        report("missing-document-set", root / "docs", 1, "No Markdown documents found")
    markdown: dict[Path, str] = {}
    for path in documents:
        if _in_assets(path, root / "docs"):
            report("markdown-in-assets", path, 1, "Asset directories contain generated figures, not Markdown documents")
            continue
        if path.name == "AGENTS.md":
            continue
        try:
            markdown[path] = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            report("unreadable-document", path, 1, str(error))

    if root / "docs" / "00_index.md" not in markdown:
        report("missing-root-entry", root / "docs", 1, "The documentation root needs 00_index.md")

    by_directory: dict[Path, list[Path]] = {}
    for document in markdown:
        if document.parent != root / "docs":
            by_directory.setdefault(document.parent, []).append(document)
    counts["documentation_directories"] = len(by_directory)
    for directory, members in sorted(by_directory.items()):
        overview = directory / "00_overview.md"
        if overview not in members:
            report("missing-overview", directory, 1, "Every documentation subdirectory needs 00_overview.md")
        indices: dict[int, list[Path]] = {}
        for document in members:
            match = re.fullmatch(r"(\d{2,})_[a-z0-9]+(?:_[a-z0-9]+)*\.md", document.name)
            if match is None:
                report("invalid-document-name", document, 1, document.name)
                continue
            index = int(match.group(1))
            indices.setdefault(index, []).append(document)
            if index == 0 and document.name != "00_overview.md":
                report("reserved-overview-index", document, 1, document.name)
        for index, collisions in indices.items():
            if len(collisions) > 1:
                for document in collisions:
                    report("duplicate-document-index", document, 1, f"index {index:02d}")
        missing = sorted(set(range(max(indices, default=-1) + 1)) - set(indices))
        if missing:
            report("document-index-gap", directory, 1, f"missing indices: {missing}")
    anchors: dict[str, list[tuple[Path, int]]] = {}
    images: dict[Path, list[tuple[Path, int]]] = {}
    section_points: dict[Path, list[tuple[int, str]]] = {}
    figure_labels: dict[str, tuple[Path, int]] = {}
    for document, content in markdown.items():
        lines = content.splitlines()
        visible = _visible_markdown(content)
        visible_lines = visible.splitlines()
        points: list[tuple[int, str]] = []
        for match in ANCHOR.finditer(visible):
            anchor = match.group(1).lower()
            line = visible.count("\n", 0, match.start()) + 1
            anchors.setdefault(anchor, []).append((document, line))
            counts["anchors"] += 1
            heading_line = line
            while heading_line < len(visible_lines) and not visible_lines[heading_line].strip():
                heading_line += 1
            heading = (SECTION_HEADING.match(visible_lines[heading_line])
                       if heading_line < len(visible_lines) else None)
            if heading is None or heading.group(1).lower() != anchor:
                report("anchor-heading-mismatch", document, line, anchor)
            else:
                points.append((heading_line + 1, anchor))
        section_points[document] = points

        def section_at(line_number: int) -> str | None:
            return next((section for start, section in reversed(points) if start <= line_number), None)

        document_images: list[tuple[Path, int]] = []
        for line_number, line in enumerate(visible_lines, 1):
            inline_images = list(IMAGE.finditer(line))
            inline_starts = {image.start() for image in inline_images}
            for start in IMAGE_START.finditer(line):
                if start.start() not in inline_starts:
                    report("unsupported-image-syntax", document, line_number, line.strip())
            for start in HTML_IMAGE_START.finditer(line):
                report("unsupported-image-syntax", document, line_number, line.strip())
            for image in inline_images:
                raw = image.group("bracketed") or image.group("plain")
                counts["images"] += 1
                if EXTERNAL_IMAGE.match(raw):
                    report("external-image-link", document, line_number, raw)
                    continue
                target = (document.parent / raw).resolve()
                document_images.append((target, line_number))
                if not target.is_relative_to(root / "docs") or target.suffix != ".svg":
                    report("invalid-image-path", document, line_number, raw)
                elif target.parent != document.parent / "assets":
                    report("image-directory-mismatch", document, line_number, raw)
                elif not target.is_file():
                    report("missing-image", document, line_number, raw)
                elif not _readable(target):
                    report("unreadable-image", document, line_number, raw)
                block = _figure_block(lines, line_number - 1)
                if block is None:
                    report("missing-figure-block", document, line_number, raw)
                else:
                    counts["figure_blocks"] += 1
                    label, fields, block_end, repeated_title = block
                    following = block_end
                    while following < len(lines) and not lines[following].strip():
                        following += 1
                    if (repeated_title or (following < len(lines)
                            and re.match(r"\s*>\s*\*\*Figure\s+", lines[following]))):
                        report("duplicate-figure-block", document, line_number, raw)
                    section = section_at(line_number)
                    if section is None or not label.lower().startswith(section + "-"):
                        report("figure-section-mismatch", document, line_number, label)
                    if label.lower() in figure_labels:
                        report("duplicate-figure-number", document, line_number, label)
                    else:
                        figure_labels[label.lower()] = (document, line_number)
                    labels = [name for name, _ in fields]
                    if labels != list(BLOCK_LABELS):
                        report("figure-field-order", document, line_number, f"{raw}: {labels}")
                    values = dict(fields)
                    for label in BLOCK_LABELS:
                        if not values.get(label):
                            report("missing-figure-field", document, line_number, f"{raw}: {label}")
                    status = values.get("Status", "").split(";", 1)[0].strip().rstrip(".").lower()
                    if status and status not in STATUSES:
                        report("invalid-figure-status", document, line_number, f"{raw}: {status}")
        images[document] = document_images
        for target, multiplicity in Counter(target for target, _ in document_images).items():
            if multiplicity > 1:
                report("duplicate-image", document, 1, f"{target}: {multiplicity} embeds")

    for anchor, occurrences in anchors.items():
        if len(occurrences) > 1:
            for document, line in occurrences:
                report("duplicate-anchor", document, line, anchor)

    def section_at(document: Path, line_number: int) -> str | None:
        return next((section for start, section in reversed(section_points[document]) if start <= line_number), None)

    sources = sorted((root / "mbse").rglob("*.sysml"))
    if not sources:
        report("missing-model-set", root / "mbse", 1, "No SysML model sources found")
    for source in sources:
        try:
            content = source.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            report("unreadable-model", source, 1, str(error))
            continue
        annotations = list(ANNOTATION.finditer(content))
        recognized_starts = {annotation.start() for annotation in annotations}
        for start in ANNOTATION_START.finditer(content):
            if start.start() not in recognized_starts:
                line = content.count("\n", 0, start.start()) + 1
                report("unsupported-annotation", source, line, start.group(0))
        preceding = None
        for annotation in annotations:
            kind = annotation.group("kind")
            body = annotation.group("body")
            parsed_fields = list(FIELD.finditer(body))
            fields = {m.group("name"): m.group("value") for m in parsed_fields}
            line = content.count("\n", 0, annotation.start()) + 1
            counts[kind] += 1
            remainder = FIELD.sub("", body).replace(";", "").strip()
            if remainder or len(parsed_fields) != len(fields):
                report("unsupported-annotation-fields", source, line, body.strip())
            document_name = fields.get("document", "")
            document = _target(root, document_name, "docs", ".md")
            if document is None:
                report("invalid-document-path", source, line, document_name)
                preceding = annotation
                continue
            if document not in markdown:
                report("missing-document", source, line, document_name)
                preceding = annotation
                continue
            if kind == "DocSection":
                section = fields.get("section", "").lower()
                if not section:
                    report("missing-section-id", source, line, document_name)
                elif not any(path == document for path, _ in anchors.get(section, [])):
                    report("missing-section-anchor", source, line, f"{document_name}#{section}")
            else:
                figure_name = fields.get("figure", "")
                figure = _target(root, figure_name, "docs", ".svg")
                if figure is None:
                    report("invalid-figure-path", source, line, figure_name)
                else:
                    if figure.parent != document.parent / "assets":
                        report("figure-directory-mismatch", source, line, figure_name)
                    if not figure.is_file():
                        report("missing-figure", source, line, figure_name)
                        preceding = annotation
                        continue
                    if not _readable(figure):
                        report("unreadable-figure", source, line, figure_name)
                    prefix = re.match(r"([0-9]+)_", document.name)
                    if prefix is None or not figure.name.startswith(prefix.group(1) + "_"):
                        report("figure-prefix-mismatch", source, line, f"{figure_name} for {document_name}")
                    embed_lines = [image_line for target, image_line in images[document] if target == figure]
                    if len(embed_lines) != 1:
                        report("figure-embed-count", source, line, f"{figure_name} in {document_name}: {len(embed_lines)}")
                    elif (preceding is not None and preceding.group("kind") == "DocSection"
                          and not content[preceding.end():annotation.start()].strip()):
                        previous_fields = {m.group("name"): m.group("value")
                                           for m in FIELD.finditer(preceding.group("body"))}
                        if previous_fields.get("document") == document_name:
                            expected = previous_fields.get("section", "").lower()
                            actual = section_at(document, embed_lines[0])
                            if expected != actual:
                                report("figure-section-placement", source, line,
                                       f"{figure_name}: expected {expected}, found {actual}")
            preceding = annotation

    failures = Counter(finding["rule"] for finding in diagnostics)
    return {
        "ok": not diagnostics,
        "counts": dict(sorted(counts.items())),
        "rules": {name: {"passed": failures[name] == 0, "failures": failures[name]}
                  for name in (*RULE_CODES, "input")},
        "diagnostics": sorted(diagnostics, key=lambda item: (item["file"], item["line"], item["code"])),
        "scope": "Directory overviews and numbering, model locator paths, unique headed anchors, figures in each carrying document directory's assets child, embeds, and adjacent numbered figure blocks. SysML name resolution and semantic view/document agreement require separate checks.",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="Print the full machine-readable report")
    args = parser.parse_args()
    result = inspect()
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"Documentation check: {'passed' if result['ok'] else 'failed'}; {result['counts']}")
        for finding in result["diagnostics"]:
            print(f"{finding['file']}:{finding['line']}: {finding['code']}: {finding['detail']}")
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
