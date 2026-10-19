#!/usr/bin/env python3
"""Verify final documentation SVGs and create hash-bound article-size previews.

Run after rendering the final figures, passing the configured Node package
runtime when sharp is not on NODE_PATH::

    python3 tools/verify_diagrams.py --evidence-dir cache/evidence/wish-0007 \
        --node /path/to/node --node-modules /path/to/node_modules

This utility checks measurable presentation properties. Human inspection of
the saved previews and the built article is still required for engineering
notation, sensible composition and remaining visual collisions.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
from xml.etree import ElementTree as ET


REPO = Path(__file__).resolve().parents[1]
SVG = "http://www.w3.org/2000/svg"
FIGURE_PATHS = (
    "docs/requirements/assets/01_requirements.svg",
    "docs/arch/assets/02_simple_flow_packages.svg",
    "docs/details/simple_flow/assets/05_block_definition.svg",
    "docs/details/simple_flow/assets/05_internal_block.svg",
    "docs/details/simple_flow/assets/05_parametric.svg",
    "docs/requirements/assets/01_use_cases.svg",
    "docs/details/simple_flow/assets/05_activity.svg",
    "docs/details/simple_flow/assets/05_sequence.svg",
    "docs/details/simple_flow/assets/05_state_machine.svg",
)
SOURCE_PATTERNS = (
    "mbse/model/**/*.sysml", "mbse/wishes/**/*.sysml", "tools/render_docs.py",
    "tools/verify_diagrams.py", "tools/sysml_web/*.py", "tools/sysml_web/toolchain.json",
    "tools/sysml_web/bin/sysml", "tools/docs_site/theme/**/*.css",
    "tools/docs_site/theme/**/*.js", "tools/docs_site/requirements.txt", "mkdocs.yml",
    "docs/details/simple_flow/04_sysml_diagrams.md",
    "docs/details/simple_flow/05_module_design.md",
    "docs/requirements/01_baseline.md", "docs/arch/02_design.md",
)
SHARP_SCRIPT = r"""
const sharp = require('sharp');
const [source, target, width] = process.argv.slice(1);
(async () => {
  const info = await sharp(source, { density: 96 })
    .resize({ width: Number(width), withoutEnlargement: false })
    .flatten({ background: '#ffffff' })
    .png({ compressionLevel: 9, adaptiveFiltering: false })
    .toFile(target);
  process.stdout.write(JSON.stringify({ width: info.width, height: info.height,
    sharpVersion: sharp.versions.sharp, svgRenderer: sharp.versions.rsvg,
    nodeVersion: process.version }));
})().catch(error => { process.stderr.write(String(error)); process.exit(1); });
"""


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def _path(path, repo):
    try:
        return str(path.resolve().relative_to(repo.resolve()))
    except ValueError:
        return str(path.resolve())


def source_snapshot(repo):
    files = sorted({path for pattern in SOURCE_PATTERNS for path in repo.glob(pattern) if path.is_file()})
    hashes = {_path(path, repo): sha256(path.read_bytes()) for path in files}
    digest = sha256(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode())
    return {"sha256": digest, "files": hashes}


def _number(value, inherited=16):
    match = re.fullmatch(r"\s*([\d.]+)\s*(px|pt|pc|in|cm|mm|em|rem|%)?\s*", value or "")
    if not match:
        raise ValueError(f"Unsupported SVG length {value!r}.")
    number, unit = float(match[1]), match[2] or "px"
    return number * {"px": 1, "pt": 4 / 3, "pc": 16, "in": 96,
                     "cm": 96 / 2.54, "mm": 96 / 25.4, "em": inherited,
                     "rem": 16, "%": inherited / 100}[unit]


def _rules(root):
    rules = []
    for element in root.iter(f"{{{SVG}}}style"):
        for selectors, declarations in re.findall(r"([^{}]+)\{([^{}]+)\}", element.text or ""):
            match = re.search(r"(?:^|;)\s*font-size\s*:\s*([^;!]+)", declarations)
            if match:
                for selector in selectors.split(","):
                    rules.append((selector.strip(), match[1].strip()))
    return rules


def _matches(element, selector):
    # Final generated figures use simple tag, class and id rules. Fail rather
    # than report a misleading computed size for an unsupported CSS selector.
    if not re.fullmatch(r"(?:[\w-]+)?(?:[.#][\w-]+)*", selector):
        raise ValueError(f"Unsupported font-size selector {selector!r}.")
    tag = re.match(r"^[\w-]+", selector)
    if tag and tag[0] != element.tag.rsplit("}", 1)[-1]:
        return False
    classes = set(element.get("class", "").split())
    return all((token[1:] in classes if token[0] == "." else element.get("id") == token[1:])
               for token in re.findall(r"[.#][\w-]+", selector))


def _transform_scale(value):
    scale = 1.0
    for operation, arguments in re.findall(r"([a-zA-Z]+)\(([^)]*)\)", value or ""):
        numbers = [float(number) for number in re.split(r"[ ,]+", arguments.strip()) if number]
        if operation == "scale":
            scale *= abs(numbers[-1])
        elif operation == "matrix":
            scale *= math.hypot(numbers[2], numbers[3])
        elif operation not in {"translate", "rotate"}:
            raise ValueError(f"Unsupported SVG transform {operation!r}.")
    return scale


def inspect_svg(path, article_width=900, min_font=11):
    data = path.read_bytes()
    root = ET.fromstring(data)
    box = [float(number) for number in re.split(r"[ ,]+", root.get("viewBox", "").strip()) if number]
    if len(box) != 4 or box[2] <= 0 or box[3] <= 0:
        raise ValueError("SVG needs a positive viewBox width and height.")
    failures, fonts, labels = [], [], []
    rules = _rules(root)

    def visit(element, inherited=16, parent_scale=1):
        size = _number(element.get("font-size"), inherited) if element.get("font-size") else inherited
        matched = [(selector.count("#") * 100 + selector.count(".") * 10 + bool(re.match(r"^\w", selector)), number, value)
                   for number, (selector, value) in enumerate(rules) if _matches(element, selector)]
        if matched:
            size = _number(max(matched)[2], inherited)
        inline = re.search(r"(?:^|;)\s*font-size\s*:\s*([^;!]+)", element.get("style", ""))
        if inline:
            size = _number(inline[1], inherited)
        scale = parent_scale * _transform_scale(element.get("transform"))
        if element.tag in {f"{{{SVG}}}text", f"{{{SVG}}}tspan"}:
            visible = (element.text or "").strip()
            if visible:
                fonts.append(size * scale)
                labels.append(visible)
        for child in element:
            visit(child, size, scale)

    visit(root)
    if not fonts:
        failures.append("No visible text with measurable font size.")
    if any("…" in label for label in labels):
        failures.append("Visible labels contain an ellipsis elision (…).")
    natural_width = _number(root.get("width")) if root.get("width") and not root.get("width").endswith("%") else article_width
    displayed_width = min(article_width, natural_width)
    minimum = min(fonts) if fonts else 0
    article_minimum = minimum * displayed_width / box[2]
    preview_minimum = minimum * article_width / box[2]
    if article_minimum + 1e-6 < min_font:
        failures.append(f"Smallest article text is {article_minimum:.2f}px; required minimum is {min_font:g}px.")
    groups = list(root.iter(f"{{{SVG}}}g"))
    native_ids = {group.get("data-native-id") for group in groups if group.get("data-native-id")}
    nodes, edges = [], []
    for group in groups:
        classes = set(group.get("class", "").split())
        if group.get("data-source-id") is not None:
            edges.append(group)
            if not all(group.get(field) for field in ("data-native-id", "data-source-id", "data-target-id")):
                failures.append("A relationship is missing native source, target or relation identity.")
            for field in ("data-source-id", "data-target-id"):
                if group.get(field) not in native_ids:
                    failures.append(f"A relationship references an undrawn endpoint {group.get(field)!r}.")
        elif not (classes & {"diagram-port", "diagram-group", "cluster"}) and (
                "node" in classes or group.get("data-shape") or (group.get("id", "").startswith("n") and group.get("data-native-id"))):
            nodes.append(group)
            if not group.get("data-native-id"):
                failures.append("A diagram node is missing native model identity.")
    node_ids = [node.get("data-native-id") for node in nodes if node.get("data-native-id")]
    if len(node_ids) != len(set(node_ids)):
        failures.append("A native diagram-node identity is drawn more than once.")
    if not native_ids or not nodes or not edges:
        failures.append("Figure must retain native model node and relationship source IDs.")
    identities_valid = bool(nodes and edges and all(node.get("data-native-id") for node in nodes)
                            and all(all(edge.get(field) for field in ("data-native-id", "data-source-id", "data-target-id"))
                                    and edge.get("data-source-id") in native_ids and edge.get("data-target-id") in native_ids
                                    for edge in edges))
    return {
        "svgSha256": sha256(data), "viewBox": box, "minSVGFont": round(minimum, 4),
        "articleWidthPx": article_width, "displayedWidthPx": round(displayed_width, 4),
        "minFontAtArticleWidthPx": round(article_minimum, 4),
        "minFontAtPreviewWidthPx": round(preview_minimum, 4), "visibleTextCount": len(fonts),
        "engine": root.get("data-layout-engine", "unknown"),
        "nativeSourceIds": sorted(native_ids), "nodeCount": len(nodes), "relationshipCount": len(edges),
        "sourceIdsPresent": identities_valid, "failures": failures,
    }


def rasterize_svg(source, target, width, node=None, node_modules=None):
    executable = node or os.environ.get("SYSML_PREVIEW_NODE") or shutil.which("node")
    if not executable:
        raise RuntimeError("Node.js is required for PNG previews; pass --node or set SYSML_PREVIEW_NODE.")
    environment = dict(os.environ, TMPDIR=str(target.parent), TMP=str(target.parent), TEMP=str(target.parent),
                       XDG_CACHE_HOME=str(target.parent.parent / "runtime-cache"))
    if node_modules:
        environment["NODE_PATH"] = str(node_modules)
    completed = subprocess.run([str(executable), "-e", SHARP_SCRIPT, str(source), str(target), str(width)],
                               capture_output=True, text=True, cwd=target.parent,
                               env=environment, timeout=45)
    if completed.returncode:
        raise RuntimeError("PNG preview rendering failed. Configure sharp with --node-modules or NODE_PATH. " + completed.stderr.strip())
    return json.loads(completed.stdout)


def _png_dimensions(data):
    if data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise ValueError("Preview renderer did not write a PNG.")
    return struct.unpack(">II", data[16:24])


def verify(repo, evidence_dir, article_width=900, min_font=11, node=None, node_modules=None,
           figures=None, rasterizer=None):
    repo, evidence_dir = repo.resolve(), evidence_dir.resolve()
    if not evidence_dir.is_relative_to(repo / "cache"):
        raise ValueError("Diagram evidence must be stored under this project's cache directory.")
    if article_width <= 0 or min_font <= 0:
        raise ValueError("Article width and minimum font must be positive.")
    evidence_dir.mkdir(parents=True, exist_ok=True)
    previews = evidence_dir / "previews"
    previews.mkdir(exist_ok=True)
    rerenders = previews / "rerender"
    rerenders.mkdir(exist_ok=True)
    manifest_path = evidence_dir / "figure-manifest.json"
    old = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    before = source_snapshot(repo)
    previous = {item["path"]: item for item in old.get("figures", [])}
    figures = list(figures) if figures is not None else [repo / name for name in FIGURE_PATHS]
    manifest = {"schemaVersion": 1, "createdAt": datetime.now(timezone.utc).isoformat(),
                "articleWidthPx": article_width, "requiredMinFontPx": min_font,
                "sourceBundle": before, "expectedFigureCount": len(figures), "figures": [], "failures": []}
    executable = shutil.which(os.environ.get("SYSML_GRAPHVIZ_DOT", "dot"))
    if executable:
        version = subprocess.run([executable, "-V"], capture_output=True, text=True, timeout=10)
        manifest["graphvizVersion"] = (version.stderr or version.stdout).strip()
    rasterizer = rasterizer or rasterize_svg
    for path in figures:
        record = {"path": _path(path, repo), "failures": []}
        try:
            record.update(inspect_svg(path, article_width, min_font))
            if record["engine"] == "graphviz-dot":
                record["engineInfo"] = {"name": "Graphviz dot", "version": manifest.get("graphvizVersion", "unavailable")}
                if not manifest.get("graphvizVersion"):
                    record["failures"].append("Graphviz version could not be recorded; configure SYSML_GRAPHVIZ_DOT.")
            elif record["engine"] == "sequence":
                record["engineInfo"] = {"name": "Model-derived sequence renderer", "source": "tools/sysml_web/documentation_views.py",
                                        "sourceSha256": before["files"].get("tools/sysml_web/documentation_views.py")}
            else:
                record["failures"].append("Figure does not identify its layout renderer.")
            target = previews / (path.name + ".png")
            record["previewRenderer"] = rasterizer(path.resolve(), target, article_width, node, node_modules)
            png = target.read_bytes()
            width, height = _png_dimensions(png)
            record.update(pngPath=_path(target, repo), pngSha256=sha256(png), pngWidth=width, pngHeight=height)
            expected_height = round(article_width * record["viewBox"][3] / record["viewBox"][2])
            if width != article_width or abs(height - expected_height) > 1:
                record["failures"].append("PNG dimensions do not preserve the full SVG aspect ratio at article width.")
            rerender = rerenders / target.name
            rasterizer(path.resolve(), rerender, article_width, node, node_modules)
            rerender_hash = sha256(rerender.read_bytes())
            record.update(rerenderPngPath=_path(rerender, repo), rerenderPngSha256=rerender_hash,
                          rerenderConsistent=rerender_hash == record["pngSha256"])
            if not record["rerenderConsistent"]:
                record["failures"].append("An immediate rerender of the same SVG produced different PNG bytes.")
            if sha256(path.read_bytes()) != record["svgSha256"]:
                record["failures"].append("SVG changed during preview generation; rerun verification.")
            previous_record = previous.get(record["path"], {})
            comparable = (old.get("sourceBundle", {}).get("sha256") == before["sha256"]
                          and previous_record.get("svgSha256") == record["svgSha256"]
                          and old.get("articleWidthPx") == article_width
                          and previous_record.get("previewRenderer") == record["previewRenderer"])
            record["consistentWithPreviousPreview"] = (previous_record.get("pngSha256") == record["pngSha256"]) if comparable else None
            if record["consistentWithPreviousPreview"] is False:
                record["failures"].append("The same SVG and source bundle produced different preview bytes.")
            if comparable and record["consistentWithPreviousPreview"]:
                record.update({key: value for key, value in previous_record.items() if key.startswith("articleScreenshot")})
        except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired, ET.ParseError) as exc:
            record["failures"].append(str(exc))
        record["passed"] = not record["failures"]
        manifest["figures"].append(record)
    if before != source_snapshot(repo):
        manifest["failures"].append("Rendering inputs changed during verification; rerun against final sources.")
    unchanged_figures = all(previous.get(item["path"], {}).get("svgSha256") == item.get("svgSha256")
                            and previous.get(item["path"], {}).get("pngSha256") == item.get("pngSha256")
                            for item in manifest["figures"])
    if old.get("sourceBundle", {}).get("sha256") == before["sha256"] and unchanged_figures:
        manifest.update({key: value for key, value in old.items() if key.startswith("articleScreenshot")})
    manifest["passed"] = not manifest["failures"] and all(item["passed"] for item in manifest["figures"])
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence-dir", type=Path, default=REPO / "cache/docs-render/review")
    parser.add_argument("--article-width", type=int, default=900)
    parser.add_argument("--min-font", type=float, default=11)
    parser.add_argument("--node", help="Configured Node.js executable")
    parser.add_argument("--node-modules", type=Path, help="Configured package directory containing sharp")
    args = parser.parse_args(argv)
    try:
        result = verify(REPO, args.evidence_dir, args.article_width, args.min_font, args.node, args.node_modules)
    except (ValueError, OSError) as exc:
        parser.error(str(exc))
    for figure in result["figures"]:
        status = "PASS" if figure["passed"] else "FAIL"
        print(f"{status} {figure['path']}" + (": " + "; ".join(figure["failures"]) if figure["failures"] else ""))
    for failure in result["failures"]:
        print("FAIL " + failure)
    print("Manifest: " + str(args.evidence_dir.resolve() / "figure-manifest.json"))
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
