"""Exercise compiler results against the project's pinned Rust toolchain."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from xml.etree import ElementTree

from tools.sysml_web.compiler import SysMLCompiler


PROJECT_ROOT = Path(__file__).resolve().parents[3]
TEST_CACHE = PROJECT_ROOT / "cache" / "sysml_web_tests"
VALID_FILES = [
    {"name": "types.sysml", "content": "package Shared { part def Component; }\n"},
    {
        "name": "nested/system.sysml",
        "content": (
            "package Demo { private import Shared::*; "
            "part def System { part component : Component; } }\n"
        ),
    },
]
RELATION_FILES = [{
    "name": "relations.sysml",
    "content": (
        "package Diagram { part def Base; part def Specialized :> Base; "
        "part def System { part unit : Specialized; } }\n"
    ),
}]
SVG_NS = {"svg": "http://www.w3.org/2000/svg"}


class CompilerIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        TEST_CACHE.mkdir(parents=True, exist_ok=True)
        cls.workspace = Path(tempfile.mkdtemp(prefix="compiler-", dir=TEST_CACHE))
        cls.project_root = cls.workspace / "project"
        cls.cache_root = cls.project_root / "cache" / "reports"
        models = cls.project_root / "mbse" / "model"
        models.mkdir(parents=True)
        for source in VALID_FILES:
            path = models / source["name"]
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source["content"], encoding="utf-8")
        cls.compiler = SysMLCompiler(cls.project_root, cache_root=cls.cache_root)

    def rendered_artifacts(self, files, view="definitions", target=""):
        result = self.compiler.analyze(files, operation="render", view=view, target=target)
        self.assertTrue(result["ok"], result)
        run = self.cache_root / "runs" / result["runId"]
        export = json.loads((run / "export.json").read_text(encoding="utf-8"))
        svg = ElementTree.parse(run / "diagram.svg").getroot()
        return export, svg

    def test_default_compiler_uses_repository_bundle_for_unrelated_project(self):
        bundled = PROJECT_ROOT / "tools/sysml_web/bin/sysml"
        self.assertTrue(bundled.is_file())
        self.assertEqual(Path(self.compiler.info["path"]), bundled.resolve())
        self.assertNotEqual(self.compiler.project_root, PROJECT_ROOT)
        self.assertTrue(self.compiler.executable.is_relative_to(self.cache_root))
        self.assertEqual(
            self.compiler.info["sha256"],
            "674fce28512a772d1886ff2397f5f02dda66265299c67c01f5cc36eaf00207f6",
        )
        self.assertIn("sysml 0.1.4", self.compiler.info["version"])
        manifest = json.loads((bundled.parent.parent / "toolchain.json").read_text())
        self.assertEqual(manifest["version"], "v0.1.4")
        self.assertEqual((bundled.parent.parent / manifest["binary"]["path"]).resolve(), bundled.resolve())
        self.assertEqual(manifest["binary"]["sha256"], hashlib.sha256(bundled.read_bytes()).hexdigest())
        self.assertEqual(manifest["binary"]["sha256"], self.compiler.info["sha256"])

    def test_project_loads_all_related_files_without_changing_source(self):
        files = self.compiler.project_files()
        by_name = {source["name"]: source["content"] for source in files}
        self.assertEqual(len(by_name), 2)
        for source in VALID_FILES:
            matches = [name for name in by_name if name.endswith(source["name"])]
            self.assertEqual(len(matches), 1)
            self.assertEqual(by_name[matches[0]], source["content"])

    def test_cross_file_check_retains_reports_and_unevaluated_rules(self):
        result = self.compiler.analyze(VALID_FILES, operation="check")
        self.assertTrue(result["ok"], result)
        self.assertTrue(result["parse"]["ok"], result)
        self.assertTrue(result["check"]["ok"], result)
        self.assertEqual(result["diagnostics"], [])
        self.assertGreater(result["rules"]["held"], 0)
        self.assertGreater(result["rules"]["unevaluated"], 0)
        run = self.cache_root / "runs" / result["runId"]
        self.assertTrue(run.is_dir())
        self.assertTrue(result["artifacts"])
        self.assertTrue(any(path.suffix == ".json" for path in run.rglob("*")))
        self.assertEqual(
            (self.project_root / "mbse/model/types.sysml").read_text(),
            VALID_FILES[0]["content"],
        )

    def test_render_exports_compiler_model_and_svg(self):
        result = self.compiler.analyze(VALID_FILES, operation="render")
        self.assertTrue(result["ok"], result)
        model = result["model"]
        self.assertTrue(model["elements"])
        definitions = json.dumps(model["definitions"])
        self.assertIn("Component", definitions)
        self.assertIn("System", definitions)
        run = self.cache_root / "runs" / result["runId"]
        svg_paths = list(run.rglob("*.svg"))
        self.assertEqual(len(svg_paths), 1)
        svg = svg_paths[0].read_text(encoding="utf-8")
        self.assertEqual(ElementTree.fromstring(svg).tag, "{http://www.w3.org/2000/svg}svg")
        self.assertIn("Component", svg)
        self.assertIn("System", svg)
        export = json.loads((run / "export.json").read_text(encoding="utf-8"))
        by_id = {element["@id"]: element for element in export}
        self.assertTrue(any(element["@type"] == "PartDefinition" for element in export))
        for element in model["elements"]:
            self.assertIn(element["id"], by_id)
            self.assertEqual(element["kind"], by_id[element["id"]]["@type"])

    def test_internal_view_resolves_requested_definition(self):
        result = self.compiler.analyze(
            VALID_FILES, operation="render", view="internal", target="Demo::System"
        )
        self.assertTrue(result["ok"], result)
        run = self.cache_root / "runs" / result["runId"]
        svg = next(run.rglob("*.svg")).read_text(encoding="utf-8")
        self.assertIn("component", svg)

    def test_definition_edges_retain_native_specialization_and_typing(self):
        export, svg = self.rendered_artifacts(RELATION_FILES)
        native = {element["@id"]: element for element in export}
        named = {element.get("qualifiedName"): element for element in export}
        base = named["Diagram::Base"]["@id"]
        specialized = named["Diagram::Specialized"]["@id"]
        system = named["Diagram::System"]["@id"]
        edges = svg.findall(".//svg:g[@data-source-id]", SVG_NS)
        rendered = {
            (edge.attrib["data-source-id"], edge.attrib["data-target-id"]):
            native[edge.attrib["data-native-id"]]
            for edge in edges
        }
        self.assertEqual(rendered[specialized, base]["@type"], "Subclassification")
        self.assertEqual(rendered[system, specialized]["@type"], "FeatureTyping")
        self.assertEqual(rendered[system, specialized]["type"]["@id"], specialized)
        feature_id = rendered[system, specialized]["typedFeature"]["@id"]
        self.assertEqual(native[feature_id]["declaredName"], "unit")
        self.assertEqual(native[feature_id]["owner"]["@id"], system)

    def test_tree_places_native_children_below_and_inside_their_owners(self):
        export, svg = self.rendered_artifacts(RELATION_FILES, view="tree")
        named = {element.get("qualifiedName"): element for element in export}
        rows = {
            group.attrib["data-native-id"]: group.find("svg:text[@class='tree-name']", SVG_NS)
            for group in svg.findall(".//svg:g[@data-native-id]", SVG_NS)
        }
        for child_name, owner_name in (
            ("Diagram::Base", "Diagram"),
            ("Diagram::Specialized", "Diagram"),
            ("Diagram::System", "Diagram"),
            ("Diagram::System::unit", "Diagram::System"),
        ):
            with self.subTest(child=child_name):
                child, owner = named[child_name], named[owner_name]
                self.assertEqual(child["owner"]["@id"], owner["@id"])
                child_row, owner_row = rows[child["@id"]], rows[owner["@id"]]
                self.assertGreater(float(child_row.attrib["x"]), float(owner_row.attrib["x"]))
                self.assertGreater(float(child_row.attrib["y"]), float(owner_row.attrib["y"]))

    def test_internal_view_preserves_three_parts_and_two_native_bindings(self):
        files = [{
            "name": "bindings.sysml",
            "content": (
                "package Internal { part def Unit { ref part selected : Unit; } "
                "part def System { part unitA : Unit; part unitB : Unit; part unitC : Unit; "
                "bind unitA.selected = unitB; bind unitB.selected = unitC; } }\n"
            ),
        }]
        export, svg = self.rendered_artifacts(files, view="internal", target="Internal::System")
        native = {element["@id"]: element for element in export}
        nodes = svg.findall(".//svg:g[@id][@data-native-id]", SVG_NS)
        self.assertEqual(len(nodes), 3)
        self.assertEqual(
            {native[node.attrib["data-native-id"]]["declaredName"] for node in nodes},
            {"unitA", "unitB", "unitC"},
        )
        edges = svg.findall(".//svg:g[@data-source-id]", SVG_NS)
        self.assertEqual(len(edges), 2)
        for edge in edges:
            relation = native[edge.attrib["data-native-id"]]
            self.assertEqual(relation["@type"], "BindingConnectorAsUsage")
            self.assertEqual(len(relation["endFeature"]), 2)
        self.assertEqual(
            {
                (native[edge.attrib["data-source-id"]]["declaredName"],
                 native[edge.attrib["data-target-id"]]["declaredName"])
                for edge in edges
            },
            {("unitA", "unitB"), ("unitB", "unitC")},
        )

    def test_quoted_identifier_is_svg_text_and_cannot_create_script_elements(self):
        identifier = '<script>alert(1)</script> & "quoted"'
        files = [{"name": "escaped.sysml", "content": "package Safe { part def '" + identifier + "'; }\n"}]
        export, svg = self.rendered_artifacts(files)
        self.assertTrue(any(element.get("declaredName") == identifier for element in export))
        self.assertFalse(any(node.tag.rsplit("}", 1)[-1] == "script" for node in svg.iter()))
        self.assertIn(identifier, "".join(svg.itertext()))

    def test_syntax_error_points_back_to_uploaded_name_and_position(self):
        files = [{
            "name": "nested/broken.sysml",
            "content": "package Broken {\n    part def Thing {\n        part x : ;\n    }\n}\n",
        }]
        result = self.compiler.analyze(files, operation="check")
        self.assertFalse(result["ok"])
        self.assertFalse(result["parse"]["ok"])
        self.assertTrue(result["diagnostics"])
        diagnostic = result["diagnostics"][0]
        self.assertEqual(diagnostic["file"], "nested/broken.sysml")
        self.assertEqual(diagnostic["line"], 3)
        self.assertEqual(diagnostic["column"], 18)
        self.assertIn("expected a name", diagnostic["message"])
        self.assertTrue((self.cache_root / "runs" / result["runId"]).is_dir())

    def test_unresolved_reference_remains_failed_with_its_evidence(self):
        files = [{
            "name": "unresolved.sysml",
            "content": "package Broken { part def Thing { part component : MissingComponent; } }\n",
        }]
        result = self.compiler.analyze(files, operation="check")
        self.assertFalse(result["ok"])
        self.assertTrue(result["parse"]["ok"])
        self.assertFalse(result["check"]["ok"])
        self.assertIn("MissingComponent", json.dumps(result["diagnostics"]))
        self.assertEqual(result["diagnostics"][0]["file"], "unresolved.sysml")
        self.assertGreater(result["diagnostics"][0]["column"], 0)

    def test_upload_names_cannot_escape_the_run_directory(self):
        for name in ("/tmp/model.sysml", "../model.sysml", "a/../../model.sysml", "a\\model.sysml"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.compiler.analyze([{"name": name, "content": "package Safe;"}])

    def test_duplicate_names_are_rejected_before_the_compiler_runs(self):
        with self.assertRaises(ValueError):
            self.compiler.analyze([VALID_FILES[0], VALID_FILES[0]])

    def test_invalid_operation_and_view_are_rejected(self):
        for keyword, value in (("operation", "shell"), ("view", "unknown")):
            with self.subTest(keyword=keyword), self.assertRaises(ValueError):
                self.compiler.analyze(VALID_FILES, **{keyword: value})

    def test_hostile_target_is_argument_data(self):
        marker = self.workspace / "target-was-executed"
        target = "Demo::System; touch " + str(marker)
        result = self.compiler.analyze(
            VALID_FILES, operation="render", view="internal", target=target
        )
        self.assertFalse(marker.exists())
        self.assertFalse(result["ok"])
        self.assertTrue(result["error"] or result["diagnostics"])


if __name__ == "__main__":
    unittest.main()
