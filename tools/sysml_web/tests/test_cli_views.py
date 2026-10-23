"""Reject misleading CLI figures when model order, guards or bindings drift."""

import json
import unittest
from xml.etree import ElementTree

from tools.render_docs import LOGS, TMP, exported_model
from tools.sysml_web.cli_views import render_cli_view
from tools.sysml_web.renderer import _ids


TIMELINE = "XDesignDetailedDesign::wishVerifyCliTimelineView"
SWIMLANE = "XDesignDetailedDesign::wishVerifyCliSwimlaneView"
PACKAGES = "XDesignComposition::stageCliPackageView"
NS = {"svg": "http://www.w3.org/2000/svg"}


class CliViewTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        LOGS.mkdir(parents=True, exist_ok=True)
        TMP.mkdir(parents=True, exist_ok=True)
        cls.raw, cls.guards = exported_model()

    def test_preserves_source_identity_and_projection_limits(self):
        for target, kind in ((TIMELINE, "cli_timeline"), (SWIMLANE, "cli_swimlane")):
            with self.subTest(kind=kind):
                svg, info = render_cli_view(self.raw, self.guards, target, kind)
                root = ElementTree.fromstring(svg)
                embedded = json.loads(root.find("svg:metadata", NS).text)
                self.assertEqual(info, embedded)
                self.assertFalse(info["native_sequence_messages"])
                self.assertFalse(info["native_allocations"])
                self.assertTrue(any(e["qualifiedName"] is None for e in info["source_elements"]))
                self.assertIn("wishAccepted", svg)
                self.assertIn("outcomeAccepted", svg)

    def test_rejects_a_changed_completion_bypass_guard(self):
        raw = [dict(e, body="observe.completed")
               if e["@type"] == "TextualRepresentation" and e.get("body") == "not observe.completed"
               else e for e in self.raw]
        with self.assertRaisesRegex(ValueError, "Expected CLI guard"):
            render_cli_view(raw, self.guards, SWIMLANE, "cli_swimlane")

    def test_rejects_reversed_record_and_assess_order(self):
        _, info = render_cli_view(self.raw, self.guards, TIMELINE, "cli_timeline")
        actions = {e["declaredName"]: e["native_id"] for e in info["source_elements"]
                   if e["source_context"] == "XDesignWorkflow::StageCli::check"}
        endpoints = [actions["record"], actions["assessAndAdvance"]]
        raw = [dict(e, relatedElement=list(reversed(e["relatedElement"])))
               if e["@type"] == "SuccessionAsUsage" and _ids(e.get("relatedElement")) == endpoints
               else e for e in self.raw]
        with self.assertRaisesRegex(ValueError, "native succession"):
            render_cli_view(raw, self.guards, TIMELINE, "cli_timeline")

    def test_rejects_a_different_composition_binding(self):
        by_id = {e["@id"]: e for e in self.raw}
        cli = next(e["@id"] for e in self.raw
                   if e.get("qualifiedName") == "XDesignComposition::xdesign::cli")
        ends = {k for e in self.raw if e["@type"] == "BindingConnectorAsUsage"
                for k in _ids(e.get("endFeature"))
                if _ids(by_id[k].get("chainingFeature"))[:1] == [cli]}
        self.assertEqual(len(ends), 1)
        raw = [dict(e, chainingFeature=list(reversed(e["chainingFeature"])))
               if e["@id"] in ends else e for e in self.raw]
        with self.assertRaisesRegex(ValueError, "cli.controller = controller binding"):
            render_cli_view(raw, self.guards, PACKAGES, "cli_packages")


if __name__ == "__main__":
    unittest.main()
