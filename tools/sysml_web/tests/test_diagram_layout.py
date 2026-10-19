"""Verify usable diagram geometry without losing compiler identities."""
from pathlib import Path
import json
import subprocess
import unittest
from unittest.mock import patch
from xml.etree import ElementTree as ET

from tools.sysml_web.diagram_layout import (
    DiagramEdge, DiagramGroup, DiagramNode, DiagramPort, render_diagram,
)
from tools.sysml_web.tests.test_compiler import PROJECT_ROOT, SVG_NS


EVIDENCE = PROJECT_ROOT / "cache" / "evidence" / "wish-0007" / "layout-tests"


class DiagramLayoutTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        EVIDENCE.mkdir(parents=True, exist_ok=True)
        cls.cache_patch = patch("tools.sysml_web.diagram_layout.LAYOUT_CACHE", EVIDENCE)
        cls.cache_patch.start()
        cls.addClassCleanup(cls.cache_patch.stop)

    def render(self, nodes, edges=(), **options):
        return ET.fromstring(render_diagram(nodes, edges, "Readable engineering view", **options))

    def test_graphviz_measures_full_labels_and_preserves_native_endpoints(self):
        label = "An engineering element whose complete long name must remain readable"
        svg = self.render(
            [DiagramNode("native-short", "Unit"), DiagramNode("native-long", label,
              lines=("part members : Component [4..*]",))],
            [DiagramEdge("native-typing", "native-short", "native-long", "owns members [4..*]", "composition")],
            question="What does the system contain?", direction="LR",
        )
        self.assertEqual(svg.get("data-layout-engine"), "graphviz-dot")
        self.assertEqual(svg.get("role"), "img")
        self.assertIsNotNone(svg.find("svg:title[@id='diagram-title']", SVG_NS))
        nodes = svg.findall(".//svg:g[@data-shape]", SVG_NS)
        self.assertEqual({node.get("data-native-id") for node in nodes}, {"native-short", "native-long"})
        widths = [float(node.find("svg:rect[@class='node']", SVG_NS).get("width")) for node in nodes]
        self.assertGreater(max(widths), min(widths) * 1.5)
        full_node = next(node for node in nodes if node.get("data-native-id") == "native-long")
        self.assertIn(label, " ".join(" ".join(full_node.itertext()).split()))
        edge = svg.find(".//svg:g[@data-source-id]", SVG_NS)
        self.assertEqual((edge.get("data-native-id"), edge.get("data-source-id"), edge.get("data-target-id")),
                         ("native-typing", "native-short", "native-long"))
        self.assertEqual(edge.find("svg:text[@class='edge-label']", SVG_NS).text, "owns members [4..*]")
        self.assertEqual(edge.get("data-edge-kind"), "composition")
        self.assertTrue(edge.findall("svg:polygon", SVG_NS), "Composition must retain its Graphviz diamond marker.")

    def test_containment_and_visible_port_terminals_retain_native_ids(self):
        svg = self.render(
            [DiagramNode("source", "Sensor", group="system", ports=(DiagramPort("out", "measurement", "E"),)),
             DiagramNode("target", "Controller", group="subsystem", ports=(DiagramPort("in", "sample", "W"),))],
            [DiagramEdge("wire", "source", "target", "measurement", "connection", "out", "in")],
            groups=(DiagramGroup("system", "System boundary"), DiagramGroup("subsystem", "Control subsystem", "system")),
            direction="LR",
        )
        groups = svg.findall(".//svg:g[@class='cluster diagram-group']", SVG_NS)
        self.assertEqual({group.get("data-native-id") for group in groups}, {"system", "subsystem"})
        ports = svg.findall(".//svg:g[@class='diagram-port']", SVG_NS)
        self.assertEqual({(port.get("data-native-id"), port.get("data-owner-id")) for port in ports},
                         {("out", "source"), ("in", "target")})
        for port in ports:
            self.assertIsNotNone(port.find("svg:rect", SVG_NS), "A port must have a visible terminal.")
            owner = next(node for node in svg.findall(".//svg:g[@data-shape]", SVG_NS)
                         if node.get("data-native-id") == port.get("data-owner-id"))
            boundary = owner.find("svg:rect[@class='node']", SVG_NS)
            terminal = port.find("svg:rect", SVG_NS)
            terminal_center = float(terminal.get("x")) + float(terminal.get("width")) / 2
            boundary_x = float(boundary.get("x"))
            expected = boundary_x if port.get("data-side") == "W" else boundary_x + float(boundary.get("width"))
            self.assertAlmostEqual(terminal_center, expected, places=2,
                                   msg="Port terminals must straddle the owning part boundary.")
        edge = svg.find(".//svg:g[@data-source-id]", SVG_NS)
        self.assertEqual((edge.get("data-source-port-id"), edge.get("data-target-port-id")), ("out", "in"))
        self.assertFalse(edge.findall("svg:polygon", SVG_NS), "An undirected connector must not acquire an arrowhead.")
        self.assertTrue(all(float(text.get("font-size", "14")) >= 14
                            for node in svg.findall(".//svg:g[@data-shape]", SVG_NS)
                            for text in node.findall(".//svg:text", SVG_NS)))

    def test_layout_separates_connected_node_boundaries(self):
        svg = self.render(
            [DiagramNode(str(number), "Step " + str(number), shape="state") for number in range(5)],
            [DiagramEdge("edge-" + str(number), str(number), str(number + 1), "continue") for number in range(4)],
        )
        positions = []
        for node in svg.findall(".//svg:g[@data-shape]", SVG_NS):
            texts = node.findall(".//svg:text", SVG_NS)
            positions.append(min(float(text.get("y")) for text in texts))
        self.assertEqual(len(set(positions)), 5)
        self.assertGreater(min(abs(a - b) for a in positions for b in positions if a != b), 50)

    def test_specialization_ranks_base_above_subtype_without_reversing_semantics(self):
        svg = self.render(
            [DiagramNode("base", "Base type"), DiagramNode("subtype", "Specific type")],
            [DiagramEdge("specializes", "subtype", "base", "specializes", "specialization"),
             DiagramEdge("reference", "base", "subtype", "current", "reference", constraint=False)],
        )
        nodes = {node.get("data-native-id"): node for node in svg.findall(".//svg:g[@data-shape]", SVG_NS)}
        top = {key: float(node.find("svg:rect[@class='node']", SVG_NS).get("y"))
               for key, node in nodes.items()}
        self.assertLess(top["base"], top["subtype"])
        edge = next(edge for edge in svg.findall(".//svg:g[@data-source-id]", SVG_NS)
                    if edge.get("data-native-id") == "specializes")
        self.assertEqual((edge.get("data-source-id"), edge.get("data-target-id")), ("subtype", "base"))
        self.assertTrue(edge.findall("svg:polygon", SVG_NS), "The specialization triangle must remain visible.")

    def test_document_labels_are_data_and_cannot_inject_dot_or_svg(self):
        label = '<script>alert(1)</script> & "quoted"; x -> y;'
        svg = self.render([DiagramNode("native\"; unsafe", label)])
        self.assertFalse(any(element.tag.rsplit("}", 1)[-1] == "script" for element in svg.iter()))
        node = svg.find(".//svg:g[@data-shape]", SVG_NS)
        self.assertEqual(node.get("data-native-id"), "native\"; unsafe")
        self.assertIn(label, " ".join(" ".join(node.itertext()).split()))

    def test_invalid_projection_fails_before_layout(self):
        with self.assertRaisesRegex(ValueError, "endpoint absent"):
            self.render([DiagramNode("one", "One")], [DiagramEdge("missing", "one", "two")])
        with self.assertRaisesRegex(ValueError, "Unknown port"):
            self.render([DiagramNode("one", "One")], [DiagramEdge("missing", "one", "one", source_port="missing")])
        with self.assertRaisesRegex(ValueError, "parent cycle"):
            self.render([DiagramNode("one", "One")], groups=(DiagramGroup("a", "A", "b"), DiagramGroup("b", "B", "a")))

    def test_missing_graphviz_has_an_actionable_error(self):
        with patch("tools.sysml_web.diagram_layout.shutil.which", return_value=None):
            with self.assertRaisesRegex(RuntimeError, "Graphviz 'dot' is required"):
                self.render([DiagramNode("one", "One")])

    def test_layout_failure_preserves_stderr_and_source_in_cache(self):
        failed_cache = EVIDENCE / "failed-layout"
        responses = [subprocess.CompletedProcess(["dot", "-V"], 0, "", "graphviz test"),
                     subprocess.CompletedProcess(["dot", "-Tsvg"], 1, "", "layout diagnostic")]
        with patch("tools.sysml_web.diagram_layout.LAYOUT_CACHE", failed_cache), \
             patch("tools.sysml_web.diagram_layout.subprocess.run", side_effect=responses):
            with self.assertRaisesRegex(RuntimeError, "layout diagnostic"):
                self.render([DiagramNode("one", "One")])
        diagnostics = list(failed_cache.rglob("diagnostics.json"))
        self.assertEqual(len(diagnostics), 1)
        self.assertEqual(json.loads(diagnostics[0].read_text())["stderr"], "layout diagnostic")
        self.assertTrue((diagnostics[0].parent / "diagram.dot").is_file())


if __name__ == "__main__":
    unittest.main()
