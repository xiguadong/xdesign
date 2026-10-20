"""Regression cases for detailed-design member and annotation discovery.

Temporary models and checker inputs stay under the ignored cache directory.
"""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import shutil
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools" / "checks"))
import common  # noqa: E402
import simple_design  # noqa: E402


class DetailedDesignLayoutCases(unittest.TestCase):
    def setUp(self) -> None:
        cache = ROOT / "cache" / "tests"
        cache.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="design-layout-", dir=cache)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.inputs = simple_design.list_inputs(ROOT)
        for uri in self.inputs:
            destination = self.root / uri
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / uri, destination)
        self.design = self.root / simple_design.DESIGN
        self.original = self.design.read_text(encoding="utf-8")

    def evaluate_doc_rule(self, source: str) -> dict:
        self.design.write_text(source, encoding="utf-8")
        hashes = {uri: hashlib.sha256((self.root / uri).read_bytes()).hexdigest()
                  for uri in self.inputs}
        context = common.Context(self.root, argparse.Namespace(), {}, {}, {}, hashes, {})
        return next(item for item in simple_design.evaluate(context)
                    if item["rule_id"] == "design-elements-carry-doc-sections")

    def without_run_doc_section(self, header: str) -> str:
        original_header = "    part def SimpleFlowRun :> FlowExecution {"
        original_section = ('        @XDesignCommon::DocSection { '
                            'document = "docs/details/simple_flow/01_run_design.md"; '
                            'section = "D02"; }')
        self.assertIn(original_header, self.original)
        self.assertIn(original_section, self.original)
        return self.original.replace(original_header, header, 1).replace(original_section, "", 1)

    def test_two_space_tab_and_multiline_headers_are_inspected(self) -> None:
        headers = (
            "  part def SimpleFlowRun :> FlowExecution {",
            "\tpart def SimpleFlowRun :> FlowExecution {",
            "\tpart\n\tdef\n\tSimpleFlowRun :> FlowExecution\n\t{",
        )
        for header in headers:
            with self.subTest(header=header):
                result = self.evaluate_doc_rule(self.without_run_doc_section(header))
                self.assertEqual(result["verdict"], "failed", result)
                self.assertTrue(any("SimpleFlowRun" in failure for failure in result["failures"]))
                self.assertIn("SimpleFlowRun", [item["element"] for item in result["observed"]["sections"]])

    def test_attribute_calc_and_abstract_definitions_are_inspected(self) -> None:
        declarations = (
            "  attribute def MissingAttribute { }\n",
            "\tcalc def MissingCalculation { }\n",
            "  abstract part def MissingAbstractPart { }\n",
        )
        for declaration, name in zip(declarations, ("MissingAttribute", "MissingCalculation", "MissingAbstractPart")):
            with self.subTest(name=name):
                source = self.original.rstrip()[:-1] + declaration + "}\n"
                result = self.evaluate_doc_rule(source)
                self.assertEqual(result["verdict"], "failed", result)
                self.assertTrue(any(name in failure for failure in result["failures"]))

    def test_nested_annotation_does_not_document_outer_definition(self) -> None:
        source = self.without_run_doc_section("    part def SimpleFlowRun :> FlowExecution {")
        run = common.find_blocks(source, r"\bpart\s+def\s+SimpleFlowRun\s*:>\s*FlowExecution\s*\{")
        self.assertEqual(len(run), 1)
        opening = source.index("{", run[0][0].start(), run[0][0].end())
        closing = opening + len(run[0][1]) - 1
        nested = ('        part nested : SimpleFlowCheckpoint {\n'
                  '            @XDesignCommon::DocSection { '
                  'document = "docs/details/simple_flow/01_run_design.md"; '
                  'section = "D02"; }\n'
                  '        }\n')
        source = source[:closing] + nested + source[closing:]
        elements = dict(simple_design.top_level(source))
        self.assertEqual(simple_design.direct_annotations(elements["SimpleFlowRun"], simple_design.DOC_SECTION), [])
        result = self.evaluate_doc_rule(source)
        self.assertEqual(result["verdict"], "failed", result)
        self.assertTrue(any("SimpleFlowRun" in failure for failure in result["failures"]))

    def test_command_rule_reports_each_checked_command_as_an_input(self) -> None:
        self.design.write_text(self.original, encoding="utf-8")
        hashes = {uri: hashlib.sha256((self.root / uri).read_bytes()).hexdigest()
                  for uri in self.inputs}
        context = common.Context(self.root, argparse.Namespace(), {}, {}, {}, hashes, {})
        result = next(item for item in simple_design.evaluate(context)
                      if item["rule_id"] == "checker-commands-exist")
        self.assertEqual(result["verdict"], "passed", result)
        commands = {item["command_uri"] for item in result["observed"]["commands"]}
        self.assertEqual(commands, set(result["inputs"]) - {simple_design.DESIGN})


if __name__ == "__main__":
    unittest.main()
