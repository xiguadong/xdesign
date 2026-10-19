"""Check that exported SysML view declarations control SVG presentation."""

import json
from pathlib import Path
import tempfile
import unittest
from xml.etree import ElementTree

from tools.sysml_web.compiler import SysMLCompiler
from tools.sysml_web.tests.test_compiler import PROJECT_ROOT, SVG_NS, TEST_CACHE


DECLARED_SOURCE = """package Declared {
    private import Views::*;
    part def Unit;
    part def Alpha {
        part alphaLeft : Unit;
        part alphaRight : Unit;
        connect alphaLeft to alphaRight;
    }
    part def Beta {
        part betaLeft : Unit;
        part betaRight : Unit;
        connect betaLeft to betaRight;
    }
    view alphaView { expose Alpha; render asInterconnectionDiagram; }
    view betaView { expose Beta; render asInterconnectionDiagram; }
    view hierarchy { expose Alpha; render asTreeDiagram; }
    view def Architecture { render asInterconnectionDiagram; }
    view inheritedAlpha : Architecture { expose Alpha; }
    view wishLifecycle : StandardViewDefinitions::GeneralView {
        expose Alpha::alphaLeft;
        expose Alpha::alphaRight;
    }
}
"""
UNSUPPORTED_VIEWS = (
    (
        "plain local view without rendering",
        "package Demo { part def System; view noRendering { expose System; } }",
        "Demo::noRendering",
        "render",
    ),
    (
        "custom rendering",
        "package Demo { private import Views::*; part def System; "
        "rendering custom : GraphicalRendering; "
        "view customView { expose System; render custom; } }",
        "Demo::customView",
        "render",
    ),
    (
        "custom rendering named like a builtin",
        "package Demo { private import Views::*; part def System; "
        "rendering asTreeDiagram : GraphicalRendering; "
        "view customView { expose System; render asTreeDiagram; } }",
        "Demo::customView",
        "render",
    ),
    (
        "unknown expose filter",
        "package Demo { private import Views::*; "
        "package Models { part def System; part system : System; } "
        "view filtered { expose Models::**[1 > 0]; render asTreeDiagram; } }",
        "Demo::filtered",
        "filter",
    ),
    (
        "inherited filter omitted by the compiler export",
        "package Demo { private import Views::*; "
        "package Models { part def System; part system : System; } "
        "view def Filtered { filter @SysML::PartUsage; render asTreeDiagram; } "
        "view inherited : Filtered { expose Models::**; } }",
        "Demo::inherited",
        "filter",
    ),
)


class DeclaredViewIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        TEST_CACHE.mkdir(parents=True, exist_ok=True)
        cls.project = Path(tempfile.mkdtemp(prefix="views-", dir=TEST_CACHE))
        cls.cache_root = cls.project / "cache" / "reports"
        cls.compiler = SysMLCompiler(cls.project, cache_root=cls.cache_root)

    def analyze_view(self, content, target):
        files = [{"name": "declared_views.sysml", "content": content}]
        return self.compiler.analyze(files, operation="render", view="declared", target=target)

    def read_svg(self, result):
        self.assertTrue(result["ok"], result)
        run = self.cache_root / "runs" / result["runId"]
        native = {
            element["@id"]: element
            for element in json.loads((run / "export.json").read_text(encoding="utf-8"))
        }
        return native, ElementTree.parse(run / "diagram.svg").getroot()

    @staticmethod
    def drawn_names(native, svg):
        return {
            native[group.attrib["data-native-id"]].get("declaredName")
            for group in svg.findall(".//svg:g[@data-native-id]", SVG_NS)
            if group.attrib["data-native-id"] in native
            and native[group.attrib["data-native-id"]].get("declaredName")
        }

    def test_different_exposes_produce_different_scoped_diagrams(self):
        results = {}
        for view_name, expected, excluded in (
            ("alphaView", {"alphaLeft", "alphaRight"}, {"betaLeft", "betaRight", "Beta"}),
            ("betaView", {"betaLeft", "betaRight"}, {"alphaLeft", "alphaRight", "Alpha"}),
        ):
            with self.subTest(view=view_name):
                result = self.analyze_view(DECLARED_SOURCE, "Declared::" + view_name)
                native, svg = self.read_svg(result)
                drawn = self.drawn_names(native, svg)
                self.assertTrue(expected.issubset(drawn), drawn)
                self.assertFalse(drawn.intersection(excluded), drawn)
                results[view_name] = drawn
                self.assertEqual(result["diagram"]["view"], "declared")
        self.assertNotEqual(results["alphaView"], results["betaView"])

    def test_declared_rendering_selects_tree_or_interconnection(self):
        tree_result = self.analyze_view(DECLARED_SOURCE, "Declared::hierarchy")
        native, tree = self.read_svg(tree_result)
        self.assertTrue(tree.findall(".//svg:text[@class='tree-name']", SVG_NS))
        self.assertIn("Alpha", self.drawn_names(native, tree))
        self.assertFalse(tree.findall(".//svg:rect[@class='node']", SVG_NS))
        graph_result = self.analyze_view(DECLARED_SOURCE, "Declared::alphaView")
        _, graph = self.read_svg(graph_result)
        self.assertEqual(len(graph.findall(".//svg:rect[@class='node']", SVG_NS)), 2)
        self.assertEqual(len(graph.findall(".//svg:g[@data-source-id]", SVG_NS)), 1)
        self.assertFalse(graph.findall(".//svg:text[@class='tree-name']", SVG_NS))

    def test_view_usage_inherits_its_view_definition_rendering(self):
        result = self.analyze_view(DECLARED_SOURCE, "Declared::inheritedAlpha")
        native, svg = self.read_svg(result)
        names = self.drawn_names(native, svg)
        self.assertTrue({"alphaLeft", "alphaRight"}.issubset(names), names)
        self.assertFalse(names.intersection({"betaLeft", "betaRight"}), names)
        self.assertEqual(len(svg.findall(".//svg:g[@data-source-id]", SVG_NS)), 1)
        view = next(view for view in result["model"]["views"] if view["qualifiedName"] == "Declared::inheritedAlpha")
        self.assertTrue(view["canRender"], view)
        self.assertIn("asInterconnectionDiagram", json.dumps(view["rendering"]))

    def test_exported_views_metadata_retains_native_ids_and_rendering(self):
        result = self.compiler.analyze([{"name": "declared_views.sysml", "content": DECLARED_SOURCE}])
        self.assertTrue(result["ok"], result)
        run = self.cache_root / "runs" / result["runId"]
        native = {
            node["@id"]: node
            for node in json.loads((run / "export.json").read_text(encoding="utf-8"))
        }
        views = {view["qualifiedName"]: view for view in result["model"]["views"]}
        for view_name, rendering in (
            ("alphaView", "asInterconnectionDiagram"),
            ("betaView", "asInterconnectionDiagram"),
            ("hierarchy", "asTreeDiagram"),
            ("inheritedAlpha", "asInterconnectionDiagram"),
        ):
            with self.subTest(view=view_name):
                view = views["Declared::" + view_name]
                self.assertEqual(native[view["id"]]["@type"], "ViewUsage")
                self.assertEqual(view["name"], view_name)
                self.assertTrue(view["canRender"], view)
                self.assertIn(rendering, json.dumps(view["rendering"]))
                self.assertFalse(view["reason"], view)

    def test_standard_general_view_uses_its_default_without_a_render_clause(self):
        result = self.analyze_view(DECLARED_SOURCE, "Declared::wishLifecycle")
        native, svg = self.read_svg(result)
        drawn = self.drawn_names(native, svg)
        self.assertTrue({"alphaLeft", "alphaRight"}.issubset(drawn), drawn)
        self.assertFalse(drawn.intersection({"betaLeft", "betaRight", "Beta"}), drawn)
        view = next(view for view in result["model"]["views"] if view["qualifiedName"] == "Declared::wishLifecycle")
        self.assertTrue(view["canRender"], view)
        self.assertFalse(view["reason"], view)

    def test_standard_general_view_renders_named_lifecycle_transitions_as_edges(self):
        source = (PROJECT_ROOT / "tools/sysml_web/examples/declared_views.sysml").read_text(encoding="utf-8")
        result = self.analyze_view(source, "DeclaredViewExample::wishLifecycle")
        native, svg = self.read_svg(result)
        nodes = svg.findall(".//svg:g[@id][@data-native-id]", SVG_NS)
        self.assertEqual(
            {native[node.attrib["data-native-id"]]["declaredName"] for node in nodes},
            {"Lifecycle", "received", "active", "delivered"},
        )
        edges = svg.findall(".//svg:g[@data-source-id]", SVG_NS)
        successions = [edge for edge in edges if native[edge.attrib["data-native-id"]]["@type"] == "SuccessionAsUsage"]
        self.assertEqual(len(successions), 2)
        self.assertEqual(
            {
                (native[edge.attrib["data-source-id"]]["declaredName"],
                 native[edge.attrib["data-target-id"]]["declaredName"])
                for edge in successions
            },
            {("received", "active"), ("active", "delivered")},
        )
        labels = set()
        for edge in successions:
            succession = native[edge.attrib["data-native-id"]]
            transition = native[succession["owner"]["@id"]]
            self.assertEqual(transition["@type"], "TransitionUsage")
            label = edge.find("svg:text[@class='edge-label']", SVG_NS).text
            self.assertEqual(label, transition["declaredName"])
            labels.add(label)
        self.assertEqual(labels, {"beginWork", "acceptDelivery"})

    def test_interconnection_preserves_exposed_parts_with_redefined_ports(self):
        source = """package PortScope {
            private import Views::*;
            port def SignalPort;
            part def Component { port signal : SignalPort; }
            part left : Component { port redefines signal : SignalPort; }
            part right : Component { port redefines signal : SignalPort; }
            connect left.signal to right.signal;
            view interfaces {
                expose left;
                expose right;
                render asInterconnectionDiagram;
            }
        }
        """
        result = self.analyze_view(source, "PortScope::interfaces")
        native, svg = self.read_svg(result)
        nodes = svg.findall(".//svg:g[@id][@data-native-id]", SVG_NS)
        self.assertEqual(len(nodes), 2)
        self.assertEqual(
            {native[node.attrib["data-native-id"]]["declaredName"] for node in nodes},
            {"left", "right"},
        )
        self.assertTrue(all(native[node.attrib["data-native-id"]]["@type"] == "PartUsage" for node in nodes))
        self.assertTrue(any(element["@type"] == "PortUsage" and element.get("declaredName") == "signal" for element in native.values()))
        edges = svg.findall(".//svg:g[@data-source-id]", SVG_NS)
        self.assertEqual(len(edges), 1)
        edge = edges[0]
        connection = native[edge.attrib["data-native-id"]]
        self.assertEqual(connection["@type"], "ConnectionUsage")
        self.assertEqual(len(connection["endFeature"]), 2)
        self.assertEqual(native[edge.attrib["data-source-id"]]["declaredName"], "left")
        self.assertEqual(native[edge.attrib["data-target-id"]]["declaredName"], "right")
        for end, endpoint in zip(connection["endFeature"], ("data-source-id", "data-target-id")):
            chain = native[end["@id"]]["chainingFeature"]
            self.assertEqual(chain[0]["@id"], edge.attrib[endpoint])
            self.assertEqual(native[chain[1]["@id"]]["@type"], "PortUsage")
            self.assertEqual(native[chain[1]["@id"]]["name"], "signal")

    def test_unsupported_custom_rendering_and_filters_fail_without_fallback(self):
        for case, source, target, reason in UNSUPPORTED_VIEWS:
            with self.subTest(case=case):
                result = self.analyze_view(source, target)
                self.assertFalse(result["ok"], result)
                self.assertTrue(result["parse"]["ok"], result)
                self.assertTrue(result["check"]["ok"], result)
                self.assertTrue(result["error"], result)
                self.assertIsNone(result["diagram"])
                diagnostics = json.dumps(result["diagnostics"]) + " " + result["error"]
                self.assertIn(reason, diagnostics.lower())
                self.assertFalse((self.cache_root / "runs" / result["runId"] / "diagram.svg").exists())

    def test_unsupported_filter_does_not_block_a_different_supported_view(self):
        source = """package Scoped {
            private import Views::*;
            package Models { part def System; part system : System; }
            view supported { expose Models::System; render asTreeDiagram; }
            view filtered { expose Models::**[1 > 0]; render asTreeDiagram; }
        }
        """
        result = self.analyze_view(source, "Scoped::supported")
        native, svg = self.read_svg(result)
        drawn = self.drawn_names(native, svg)
        self.assertIn("System", drawn)
        self.assertNotIn("system", drawn)
        views = {view["qualifiedName"]: view for view in result["model"]["views"]}
        self.assertTrue(views["Scoped::supported"]["canRender"], views)
        self.assertFalse(views["Scoped::filtered"]["canRender"], views)
        self.assertIn("filter", views["Scoped::filtered"]["reason"].lower())


if __name__ == "__main__":
    unittest.main()
