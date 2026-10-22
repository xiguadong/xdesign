"""Regression checks for final-figure evidence and its input bindings."""
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

from tools.render_docs import FIGURES
from tools.verify_diagrams import FIGURE_PATHS, inspect_svg, verify
from tools.sysml_web.tests.test_compiler import PROJECT_ROOT


TEST_CACHE = PROJECT_ROOT / "cache/evidence/wish-0007/verification-tests"
SIMPLE_SVG = '''<svg xmlns="http://www.w3.org/2000/svg" width="600" height="300" viewBox="0 0 600 300" data-layout-engine="graphviz-dot">
<style>text{font-size:14px}.edge-label{font-size:13px}</style>
<g id="n_a" class="node" data-native-id="a"><rect x="20" y="30" width="100" height="50"/><text x="30" y="60">Alpha</text></g>
<g id="n_b" class="node" data-native-id="b"><text x="300" y="60">Beta</text></g>
<g data-native-id="relation" data-source-id="a" data-target-id="b"><text class="edge-label" x="180" y="45">connects</text></g>
</svg>'''


def write_png(path, width, height, shade=255):
    def chunk(kind, value):
        return struct.pack(">I", len(value)) + kind + value + struct.pack(">I", zlib.crc32(kind + value))
    pixels = (b"\x00" + bytes([shade, shade, shade]) * width) * height
    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
    data += chunk(b"IDAT", zlib.compress(pixels)) + chunk(b"IEND", b"")
    path.write_bytes(data)


class FinalDiagramEvidenceTests(unittest.TestCase):
    def test_render_and_verification_paths_use_document_local_assets(self):
        rendered = {f"docs/{name}.svg" for name, _, _ in FIGURES}
        self.assertEqual(len(rendered), len(FIGURES))
        self.assertTrue(all(Path(path).parent.name == "assets" for path in rendered))
        self.assertTrue(set(FIGURE_PATHS).issubset(rendered))

    def setUp(self):
        TEST_CACHE.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(prefix="run-", dir=TEST_CACHE)
        self.addCleanup(self.directory.cleanup)
        self.repo = Path(self.directory.name)
        self.figure = self.repo / "docs/architecture/assets/figure.svg"
        self.figure.parent.mkdir(parents=True)
        self.figure.write_text(SIMPLE_SVG)
        self.model = self.repo / "mbse/model/system.sysml"
        self.model.parent.mkdir(parents=True)
        self.model.write_text("package Example { part def System; }")
        self.evidence = self.repo / "cache/evidence"

    def rasterizer(self, source, target, width, node, modules):
        write_png(target, width, width // 2)
        return {"width": width, "height": width // 2, "nodeVersion": "test", "sharpVersion": "test"}

    def run_verification(self, rasterizer=None):
        return verify(self.repo, self.evidence, figures=[self.figure], rasterizer=rasterizer or self.rasterizer)

    def test_css_font_sizes_and_article_scaling_are_measured(self):
        result = inspect_svg(self.figure)
        self.assertEqual(result["minSVGFont"], 13)
        self.assertEqual(result["displayedWidthPx"], 600)
        self.assertEqual(result["minFontAtArticleWidthPx"], 13)
        self.assertEqual(result["minFontAtPreviewWidthPx"], 19.5)
        self.assertEqual(result["failures"], [])
        self.figure.write_text(SIMPLE_SVG.replace('width="600"', 'width="1200pt"').replace('0 0 600 300', '0 0 1200 600'))
        result = inspect_svg(self.figure)
        self.assertEqual(result["displayedWidthPx"], 900)
        self.assertEqual(result["minFontAtArticleWidthPx"], 9.75)
        self.assertTrue(any("required minimum" in failure for failure in result["failures"]))

    def test_ellipsis_and_duplicate_nodes_fail_but_relation_annotations_can_repeat(self):
        self.figure.write_text(SIMPLE_SVG.replace("Alpha</text>", "Alpha…</text>"))
        self.assertTrue(any("ellipsis" in failure for failure in inspect_svg(self.figure)["failures"]))
        self.figure.write_text(SIMPLE_SVG.replace('data-native-id="b"', 'data-native-id="a"'))
        self.assertTrue(any("drawn more than once" in failure for failure in inspect_svg(self.figure)["failures"]))
        self.figure.write_text(SIMPLE_SVG.replace("</svg>", '<g data-native-id="relation"><title>Additional relation annotation…</title></g></svg>'))
        self.assertEqual(inspect_svg(self.figure)["failures"], [])

    def test_missing_native_source_identity_is_reported(self):
        self.figure.write_text(SIMPLE_SVG.replace(' data-native-id="a"', ""))
        result = inspect_svg(self.figure)
        self.assertTrue(any("missing native model identity" in failure for failure in result["failures"]))
        self.assertTrue(any("undrawn endpoint" in failure for failure in result["failures"]))

    def test_manifest_binds_sources_svg_and_png_and_repeated_run(self):
        first = self.run_verification()
        self.assertTrue(first["passed"], first)
        figure = first["figures"][0]
        self.assertEqual(len(figure["svgSha256"]), 64)
        self.assertEqual(len(figure["pngSha256"]), 64)
        self.assertEqual((figure["pngWidth"], figure["pngHeight"]), (900, 450))
        self.assertIn("mbse/model/system.sysml", first["sourceBundle"]["files"])
        self.assertIsNone(figure["consistentWithPreviousPreview"])
        self.assertTrue(figure["rerenderConsistent"])
        self.assertEqual(figure["pngSha256"], figure["rerenderPngSha256"])
        first["figures"][0]["articleScreenshot"] = {"path": "cache/article.png", "sha256": "reviewed"}
        (self.evidence / "figure-manifest.json").write_text(json.dumps(first))
        second = self.run_verification()
        self.assertTrue(second["passed"], second)
        self.assertTrue(second["figures"][0]["consistentWithPreviousPreview"])
        self.assertEqual(second["figures"][0]["articleScreenshot"]["sha256"], "reviewed")
        self.model.write_text("package Changed { part def System; }")
        third = self.run_verification()
        self.assertNotEqual(third["sourceBundle"]["sha256"], second["sourceBundle"]["sha256"])
        self.assertNotIn("articleScreenshot", third["figures"][0])

    def test_mutation_during_preview_cannot_be_reported_as_a_pass(self):
        def mutate(source, target, width, node, modules):
            self.rasterizer(source, target, width, node, modules)
            source.write_text(SIMPLE_SVG.replace("Alpha", "Changed"))
            self.model.write_text("package Changed;")
            return {"width": width, "height": width // 2}
        result = self.run_verification(mutate)
        self.assertFalse(result["passed"])
        self.assertTrue(any("SVG changed" in failure for failure in result["figures"][0]["failures"]))
        self.assertTrue(any("inputs changed" in failure for failure in result["failures"]))

    def test_evidence_outside_project_cache_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "under this project's cache"):
            verify(self.repo, self.repo / "external-evidence", figures=[self.figure], rasterizer=self.rasterizer)

    def test_nondeterministic_preview_is_rejected(self):
        calls = []
        def changing_preview(source, target, width, node, modules):
            calls.append(target)
            write_png(target, width, width // 2, shade=255 - len(calls))
            return {"width": width, "height": width // 2}
        result = self.run_verification(changing_preview)
        self.assertFalse(result["passed"])
        self.assertFalse(result["figures"][0]["rerenderConsistent"])
        self.assertTrue(any("immediate rerender" in failure for failure in result["figures"][0]["failures"]))


if __name__ == "__main__":
    unittest.main()
