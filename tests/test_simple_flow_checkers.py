"""Executable SimpleFlow checker contract and targeted rule tests.

All generated records, reports and mutated fixtures stay under ignored cache/.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools" / "checks"))
import common  # noqa: E402
import simple_design  # noqa: E402
import simple_implementation  # noqa: E402
import simple_record  # noqa: E402
import simple_requirements  # noqa: E402


def uri(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def artifact(path: str) -> dict[str, str]:
    return {"uri": path, "sha256": hashlib.sha256((ROOT / path).read_bytes()).hexdigest()}


def visit(stage: str, artifacts: list[dict], status: str = "pending") -> dict:
    return {
        "stage_id": stage,
        "visit_index": 0,
        "status": status,
        "artifact_revision": 1 if artifacts else 0,
        "artifact_snapshot_digest": common.snapshot_digest(artifacts),
        "artifact_digests": artifacts,
        "activities": [],
        "assessments": [],
    }


class CheckerCases(unittest.TestCase):
    def setUp(self) -> None:
        (ROOT / "cache" / "tests").mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="checkers-", dir=ROOT / "cache" / "tests")
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.wish = "mbse/wishes/wish-0009.sysml"
        self.stages = ["requirements", "design", "implement", "verify"]
        self.gates = {
            "requirements": ["simpleRequirementsAccepted"],
            "design": ["simpleDesignAccepted"],
            "implement": ["simpleImplementationAccepted"],
            "verify": ["simpleRecordAccepted", "verificationAccepted"],
        }
        plan = {
            "plan_id": "designedSimpleFlow",
            "initial_stage_id": "requirements",
            "stages": [
                {"stage_id": stage, "exit_checkpoint_ids": self.gates[stage],
                 "next_stage_ids": self.stages[index + 1:index + 2],
                 "completes_flow": stage == "verify"}
                for index, stage in enumerate(self.stages)
            ],
            "checkpoints": [],
        }
        self.plan_path = self.directory / "plan.json"
        self.plan_path.write_text(json.dumps(plan, sort_keys=True), encoding="utf-8")
        self.plan_sha256 = hashlib.sha256(self.plan_path.read_bytes()).hexdigest()
        self.evidence_path = self.directory / "evidence.txt"
        self.evidence_path.write_text("retained gate evidence\n", encoding="utf-8")
        self.record_path = self.directory / "record.json"
        self.report_path = self.directory / "report.json"

    def make_record(self, current: int, artifacts: list[dict] | None = None) -> dict:
        default_artifacts = [artifact("mbse/model/requirements.sysml")]
        visits = [visit(stage, default_artifacts.copy() if index <= current else [], "completed" if index < current else
                        ("active" if index == current else "pending"))
                  for index, stage in enumerate(self.stages)]
        if artifacts is not None:
            visits[current] = visit(self.stages[current], artifacts, "active")
        for index in range(current):
            previous = visits[index]
            previous["assessments"] = [
                {"checkpoint_id": gate, "evaluated_revision": previous["artifact_revision"],
                 "evaluated_snapshot_digest": previous["artifact_snapshot_digest"],
                 "outcome": "passed", "judged_by": "script", "assessor": "test",
                 "rationale": "checked", "evidence_uris": [uri(self.evidence_path)]}
                for gate in self.gates[self.stages[index]]
            ]
        record = {
            "run_id": "fixture-run", "execution_type": "FlowRun",
            "plan_id": "designedSimpleFlow", "plan_uri": uri(self.plan_path),
            "plan_sha256": self.plan_sha256, "wish_model_uri": self.wish,
            "status": "active", "current_visit": current, "visits": visits,
        }
        return record

    def invoke(self, module: str, checker_id: str, record: dict) -> tuple[subprocess.CompletedProcess[str], dict | None]:
        self.record_path.write_text(json.dumps(record, indent=2), encoding="utf-8")
        self.report_path.unlink(missing_ok=True)
        current = record["visits"][record["current_visit"]]
        command = [sys.executable, str(ROOT / "tools" / "checks" / module),
                   "--wish-model", self.wish, "--record", uri(self.record_path),
                   "--checker-id", checker_id, "--plan-id", "designedSimpleFlow",
                   "--plan-uri", uri(self.plan_path), "--plan-sha256", self.plan_sha256,
                   "--stage", current["stage_id"], "--artifact-revision", str(current["artifact_revision"]),
                   "--snapshot-digest", current["artifact_snapshot_digest"],
                   "--report", uri(self.report_path)]
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        completed = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True, text=True)
        report = json.loads(self.report_path.read_text()) if self.report_path.exists() else None
        return completed, report

    def test_four_commands_report_exact_rules_on_valid_inputs(self) -> None:
        cases = [
            ("simple_requirements.py", "simpleRequirementsCheck", 0,
             simple_requirements.list_inputs(ROOT), simple_requirements.RULES),
            ("simple_design.py", "simpleDesignCheck", 1,
             simple_design.list_inputs(ROOT), simple_design.RULES),
            ("simple_implementation.py", "simpleImplementationCheck", 2,
             ["mbse/model/requirements.sysml"], simple_implementation.RULES),
            ("simple_record.py", "simpleRecordCheck", 3,
             ["mbse/model/requirements.sysml"], simple_record.RULES),
        ]
        for module, checker, index, inputs, declared in cases:
            with self.subTest(checker=checker):
                record = self.make_record(index, [artifact(path) for path in inputs])
                completed, report = self.invoke(module, checker, record)
                self.assertEqual(completed.returncode, 0, completed.stderr)
                self.assertIsNotNone(report)
                self.assertEqual(report["overall_verdict"], "passed", report)
                self.assertEqual([item["rule_id"] for item in report["rules"]], list(declared))
                self.assertTrue(all(item["verdict"] == "passed" and not item["failures"]
                                    and item["observed"] is not None for item in report["rules"]))
                self.assertEqual(report["artifact_revision"], record["visits"][index]["artifact_revision"])
                self.assertEqual(report["snapshot_digest"], record["visits"][index]["artifact_snapshot_digest"])
                self.assertEqual(report["input_sha256"][self.wish], artifact(self.wish)["sha256"])

    def test_missing_binding_and_stale_bytes_are_unevaluated(self) -> None:
        record = self.make_record(0, [artifact("mbse/model/requirements.sysml")])
        completed, report = self.invoke("simple_requirements.py", simple_requirements.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 2)
        self.assertIsNone(report)
        self.assertIn("absent from current artifact snapshot", completed.stderr)
        record = self.make_record(0, [artifact(path) for path in simple_requirements.INPUTS])
        record["visits"][0]["artifact_digests"][0]["sha256"] = "0" * 64
        completed, report = self.invoke("simple_requirements.py", simple_requirements.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 2)
        self.assertIsNone(report)
        self.assertIn("artifact bytes changed", completed.stderr)

    def test_plan_uri_and_hash_mismatch_are_unevaluated(self) -> None:
        record = self.make_record(2, [artifact("mbse/model/requirements.sysml")])
        record["plan_uri"] = uri(self.evidence_path)
        completed, report = self.invoke("simple_implementation.py", simple_implementation.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 2)
        self.assertIsNone(report)
        self.assertIn("plan URI differs", completed.stderr)
        record = self.make_record(2, [artifact("mbse/model/requirements.sysml")])
        self.plan_path.write_text("changed plan", encoding="utf-8")
        completed, report = self.invoke("simple_implementation.py", simple_implementation.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 2)
        self.assertIsNone(report)
        self.assertIn("plan bytes differ", completed.stderr)

    def test_empty_implementation_artifacts_fail_a_rule(self) -> None:
        record = self.make_record(2, [])
        completed, report = self.invoke("simple_implementation.py", simple_implementation.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(report["overall_verdict"], "failed")
        self.assertEqual(report["rules"][1]["rule_id"], "artifacts-of-current-visit-exist")
        self.assertTrue(report["rules"][1]["failures"])

    def test_record_checks_completed_bytes_and_evidence(self) -> None:
        record = self.make_record(3, [artifact("mbse/model/requirements.sysml")])
        record["visits"][0]["artifact_digests"][0]["sha256"] = "0" * 64
        record["visits"][0]["assessments"][0]["evidence_uris"] = [uri(self.directory / "missing.txt")]
        completed, report = self.invoke("simple_record.py", simple_record.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        failures = {item["rule_id"]: item["failures"] for item in report["rules"]}
        self.assertTrue(failures["completed-visits-authorized"])
        self.assertTrue(failures["evidence-exists"])
        self.assertEqual(report["overall_verdict"], "failed")

    def test_record_uses_retained_stage_versions_after_live_source_changes(self) -> None:
        source = self.directory / "source.py"
        source.write_text("version = 1\n", encoding="utf-8")
        assessed = artifact(uri(source))
        directory = self.plan_path.parent / "artifact-snapshots"
        directory.mkdir()
        retained = directory / assessed["sha256"]
        retained.write_bytes(source.read_bytes())
        record = self.make_record(3)
        record["artifact_snapshot_directory"] = uri(directory)
        for previous in record["visits"][:3]:
            previous["artifact_digests"] = [assessed]
            previous["artifact_snapshot_digest"] = common.snapshot_digest([assessed])
            previous["artifact_snapshots"] = {uri(source): uri(retained)}
            previous["assessments"][0]["evaluated_snapshot_digest"] = previous["artifact_snapshot_digest"]
        source.write_text("version = 2\n", encoding="utf-8")
        completed, report = self.invoke("simple_record.py", simple_record.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(report["overall_verdict"], "passed", report)
        self.assertEqual(report["input_sha256"][uri(retained)], assessed["sha256"])
        retained.write_text("tampered\n", encoding="utf-8")
        completed, report = self.invoke("simple_record.py", simple_record.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(report["overall_verdict"], "failed")

    def test_record_rejects_schema_and_lifecycle_bypasses(self) -> None:
        baseline = self.make_record(3, [artifact("mbse/model/requirements.sysml")])

        def all_visits_pending(record: dict) -> None:
            record["status"] = "completed"
            for item in record["visits"]:
                item.update(status="pending", artifact_revision=0,
                            artifact_snapshot_digest=common.snapshot_digest([]),
                            artifact_digests=[], activities=[], assessments=[])

        def current_pending_with_missing_evidence(record: dict) -> None:
            current = record["visits"][3]
            current["status"] = "pending"
            current["assessments"] = [{
                "checkpoint_id": "verificationAccepted", "evaluated_revision": current["artifact_revision"],
                "evaluated_snapshot_digest": current["artifact_snapshot_digest"],
                "outcome": "passed", "judged_by": "human", "assessor": "test",
                "rationale": "checked", "evidence_uris": [uri(self.directory / "missing.txt")],
            }]

        mutations = [
            ("completed run with pending visits", all_visits_pending, "historical but not completed"),
            ("unknown historical status", lambda item: item["visits"][0].__setitem__("status", "Completed"),
             "status is not a supported value"),
            ("pending historical visit", lambda item: item["visits"][0].__setitem__("status", "pending"),
             "historical but not completed"),
            ("current pending hides evidence", current_pending_with_missing_evidence,
             "pending but contains work"),
            ("missing visit revision", lambda item: item["visits"][0].pop("artifact_revision"),
             "artifact_revision must be"),
            ("missing assessment revision", lambda item: item["visits"][0]["assessments"][0].pop("evaluated_revision"),
             "evaluated_revision must be"),
            ("unknown assessment outcome", lambda item: item["visits"][0]["assessments"][0].__setitem__("outcome", "Passed"),
             "outcome is not a supported value"),
        ]
        for label, mutate, diagnostic in mutations:
            with self.subTest(case=label):
                record = copy.deepcopy(baseline)
                mutate(record)
                completed, report = self.invoke("simple_record.py", simple_record.CHECKER_ID, record)
                self.assertEqual(completed.returncode, 2, completed.stderr)
                self.assertIsNone(report)
                self.assertIn(diagnostic, completed.stderr)

    def test_record_accepts_completed_and_reopened_visit_history(self) -> None:
        record = self.make_record(3, [artifact("mbse/model/requirements.sysml")])

        def authorize(item: dict) -> None:
            item["assessments"] = [
                {"checkpoint_id": gate, "evaluated_revision": item["artifact_revision"],
                 "evaluated_snapshot_digest": item["artifact_snapshot_digest"],
                 "outcome": "passed", "judged_by": "script", "assessor": "test",
                 "rationale": "checked", "evidence_uris": [uri(self.evidence_path)]}
                for gate in self.gates[item["stage_id"]]
            ]

        record["status"] = "completed"
        record["visits"][3]["status"] = "completed"
        authorize(record["visits"][3])
        completed, report = self.invoke("simple_record.py", simple_record.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(report["overall_verdict"], "passed")

        for stage in self.stages:
            item = visit(stage, [artifact("mbse/model/requirements.sysml")],
                         "active" if stage == "verify" else "completed")
            item["visit_index"] = 1
            if stage != "verify":
                authorize(item)
            record["visits"].append(item)
        record["status"] = "active"
        record["current_visit"] = len(record["visits"]) - 1
        completed, report = self.invoke("simple_record.py", simple_record.CHECKER_ID, record)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(report["overall_verdict"], "passed")

    def test_requirements_rationale_and_link_targets_fail_independently(self) -> None:
        fixture = self.directory / "source"
        for path in simple_requirements.INPUTS:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        requirement_file = fixture / simple_requirements.INPUTS[0]
        source = requirement_file.read_text()
        source = source.replace('attribute redefines rationale = "A flow can route a check to the right executor, a script or a judging reader, only if the checkpoint says which kind it is.";', '')
        requirement_file.write_text(source)
        trace_file = fixture / simple_requirements.INPUTS[1]
        trace_file.write_text(trace_file.read_text() + '\nsatisfy nonexistentRequirement by Checkpoint.kind;\n')
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest() for path in simple_requirements.INPUTS}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        results = simple_requirements.evaluate(context)
        self.assertEqual([item["verdict"] for item in results], ["failed", "failed"])
        self.assertTrue(any("checkpointKindIsDeclared" in failure for failure in results[0]["failures"]))
        self.assertTrue(any("nonexistentRequirement" in failure for failure in results[1]["failures"]))
        trace_source = trace_file.read_text().replace('    satisfy checkpointKindIsDeclared by Checkpoint.kind;',
                                                       '    satisfy nonexistentRequirement by Checkpoint.kind;')
        trace_file.write_text(trace_source)
        context.artifact_hashes[simple_requirements.INPUTS[1]] = hashlib.sha256(trace_file.read_bytes()).hexdigest()
        results = simple_requirements.evaluate(context)
        self.assertTrue(any("nonexistentRequirement" in failure for failure in results[1]["failures"]))

    def test_requirements_reject_other_package_and_nonstandard_usage_without_rationale(self) -> None:
        fixture = self.directory / "requirements-scope"
        for path in simple_requirements.INPUTS:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        requirement_file = fixture / simple_requirements.INPUTS[0]
        trace_file = fixture / simple_requirements.INPUTS[1]
        original = requirement_file.read_text()
        marker = "    requirement checkpointKindIsDeclared : XDesignRequirement {"
        self.assertIn(marker, original)
        requirement_file.write_text(original.replace(marker,
            "    requirement extraRequirement : Requirements::Requirement {\n"
            "        doc /* This valid usage lacks its own rationale. */\n"
            "        requirement nestedRequirement : XDesignRequirement {\n"
            "            attribute redefines rationale = \"Nested reason\";\n"
            "        }\n"
            "    }\n\n" + marker, 1))
        trace_original = trace_file.read_text()
        self.assertIn("satisfy checkpointKindIsDeclared by Checkpoint.kind;", trace_original)
        trace_file.write_text(trace_original.replace(
            "satisfy checkpointKindIsDeclared by Checkpoint.kind;",
            "satisfy OtherPackage::checkpointKindIsDeclared by Checkpoint.kind;", 1))
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest()
                  for path in simple_requirements.INPUTS}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        results = simple_requirements.evaluate(context)
        self.assertEqual([entry["verdict"] for entry in results], ["failed", "failed"])
        self.assertTrue(any("extraRequirement" in failure for failure in results[0]["failures"]))
        self.assertTrue(any("OtherPackage::checkpointKindIsDeclared" in failure for failure in results[1]["failures"]))
        declared = {"checkpointKindIsDeclared"}
        self.assertTrue(simple_requirements.resolves_target("checkpointKindIsDeclared", declared))
        self.assertTrue(simple_requirements.resolves_target("XDesignRequirements::checkpointKindIsDeclared", declared))
        self.assertTrue(simple_requirements.resolves_target("::XDesignRequirements::checkpointKindIsDeclared", declared))
        self.assertTrue(simple_requirements.resolves_target("XDesignRequirements.checkpointKindIsDeclared", declared))
        self.assertFalse(simple_requirements.resolves_target("OtherPackage::checkpointKindIsDeclared", declared))
        self.assertFalse(simple_requirements.resolves_target("::checkpointKindIsDeclared", declared))

    def test_requirements_inspect_specialized_abstract_and_external_links(self) -> None:
        fixture = self.directory / "requirement-forms"
        for path in simple_requirements.INPUTS:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        requirement_file = fixture / simple_requirements.INPUTS[0]
        marker = "    requirement checkpointKindIsDeclared : XDesignRequirement {"
        original = requirement_file.read_text()
        self.assertIn(marker, original)
        requirement_file.write_text(original.replace(
            marker,
            "    requirement specialForm :> XDesignRequirement { }\n"
            "    abstract requirement abstractForm : XDesignRequirement { }\n\n" + marker, 1))
        trace_file = fixture / simple_requirements.INPUTS[1]
        trace_file.write_text(trace_file.read_text() +
                              "\npackage OtherTrace {\n    satisfy ghostRequirement by Checkpoint.kind;\n}\n")
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest()
                  for path in simple_requirements.INPUTS}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        results = simple_requirements.evaluate(context)
        self.assertEqual([item["verdict"] for item in results], ["failed", "failed"])
        self.assertIn("specialForm", results[0]["observed"]["requirement_ids"])
        self.assertIn("abstractForm", results[0]["observed"]["requirement_ids"])
        self.assertTrue(any("specialForm" in failure for failure in results[0]["failures"]))
        self.assertTrue(any("abstractForm" in failure for failure in results[0]["failures"]))
        self.assertTrue(any("ghostRequirement" in failure for failure in results[1]["failures"]))
        self.assertTrue(any(link["target"] == "ghostRequirement"
                            for link in results[1]["observed"]["satisfy_links"]))

    def test_empty_requirements_and_trace_links_do_not_pass(self) -> None:
        fixture = self.directory / "empty-requirement-sources"
        requirements = fixture / simple_requirements.INPUTS[0]
        traceability = fixture / simple_requirements.INPUTS[1]
        requirements.parent.mkdir(parents=True, exist_ok=True)
        requirements.write_text("package XDesignRequirements {}\n")
        traceability.write_text("package XDesignTraceability {}\n")
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest()
                  for path in simple_requirements.INPUTS}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        results = simple_requirements.evaluate(context)
        self.assertEqual([item["verdict"] for item in results], ["failed", "failed"])

    def test_nested_project_requirement_resolves_by_qualified_name(self) -> None:
        fixture = self.directory / "nested-requirement"
        for path in simple_requirements.INPUTS:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        requirement_file = fixture / simple_requirements.INPUTS[0]
        marker = "    requirement checkpointKindIsDeclared : XDesignRequirement {"
        requirement_file.write_text(requirement_file.read_text().replace(
            marker,
            "    package Nested {\n"
            "        requirement nestedRequirement : XDesignRequirement {\n"
            "            attribute redefines rationale = \"Nested reason\";\n"
            "        }\n"
            "    }\n\n" + marker, 1))
        trace_file = fixture / simple_requirements.INPUTS[1]
        trace_file.write_text(trace_file.read_text() +
                              "\npackage OtherTrace {\n"
                              "    satisfy XDesignRequirements::Nested::nestedRequirement by Checkpoint.kind;\n"
                              "}\n")
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest()
                  for path in simple_requirements.INPUTS}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        results = simple_requirements.evaluate(context)
        self.assertEqual([item["verdict"] for item in results], ["passed", "passed"])
        self.assertIn("Nested::nestedRequirement", results[0]["observed"]["requirement_ids"])
        self.assertTrue(results[1]["observed"]["satisfy_links"][-1]["resolved_requirement"])

    def test_outer_requirement_does_not_borrow_nested_rationale(self) -> None:
        body = '{\n  requirement child : XDesignRequirement { attribute redefines rationale = "Child reason"; }\n}'
        self.assertIsNone(common.attribute_string(body, "rationale"))
        body = '{\n  attribute redefines rationale = "Parent reason";\n' + body[1:]
        self.assertEqual(common.attribute_string(body, "rationale"), "Parent reason")

    def test_design_missing_figure_is_failed_rule(self) -> None:
        fixture = self.directory / "design-source"
        paths = simple_design.list_inputs(ROOT)
        for path in paths:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        missing = simple_design.REQUIRED_VIEW_FIGURES["sequenceDiagram"][1]
        (fixture / missing).unlink()
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest()
                  for path in paths if (fixture / path).is_file()}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        results = simple_design.evaluate(context)
        graph = next(item for item in results if item["rule_id"] == "structure-graphs-are-carried")
        self.assertEqual(graph["verdict"], "failed")
        self.assertTrue(any(missing in failure for failure in graph["failures"]))

    def test_nine_primary_views_cannot_share_one_generic_figure(self) -> None:
        fixture = self.directory / "collapsed-views"
        paths = simple_design.list_inputs(ROOT)
        for path in paths:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        design_file = fixture / simple_design.DESIGN
        source = design_file.read_text()
        generic = simple_design.REQUIRED_VIEW_FIGURES["blockDefinitionDiagram"]
        for document, figure in simple_design.REQUIRED_VIEW_FIGURES.values():
            original = f'@XDesignCommon::DocFigure {{ document = "{document}"; figure = "{figure}"; }}'
            replacement = f'@XDesignCommon::DocFigure {{ document = "{generic[0]}"; figure = "{generic[1]}"; }}'
            self.assertIn(original, source)
            source = source.replace(original, replacement, 1)
        design_file.write_text(source)
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest() for path in paths}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        graph = next(item for item in simple_design.evaluate(context)
                     if item["rule_id"] == "structure-graphs-are-carried")
        self.assertEqual(graph["verdict"], "failed")
        self.assertTrue(any("question figure" in failure or "distinct figures" in failure
                            for failure in graph["failures"]))

    def test_primary_view_asset_swap_fails_even_when_assets_are_distinct(self) -> None:
        fixture = self.directory / "swapped-views"
        paths = simple_design.list_inputs(ROOT)
        for path in paths:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        design_file = fixture / simple_design.DESIGN
        source = design_file.read_text()
        first = simple_design.REQUIRED_VIEW_FIGURES["blockDefinitionDiagram"]
        second = simple_design.REQUIRED_VIEW_FIGURES["internalBlockDiagram"]
        self.assertEqual(first[0], second[0])
        first_annotation = f'@XDesignCommon::DocFigure {{ document = "{first[0]}"; figure = "{first[1]}"; }}'
        second_annotation = f'@XDesignCommon::DocFigure {{ document = "{second[0]}"; figure = "{second[1]}"; }}'
        self.assertIn(first_annotation, source)
        self.assertIn(second_annotation, source)
        source = source.replace(first_annotation, "@SWAP@", 1).replace(second_annotation, first_annotation, 1).replace("@SWAP@", second_annotation, 1)
        design_file.write_text(source)
        hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest() for path in paths}
        context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
        graph = next(item for item in simple_design.evaluate(context)
                     if item["rule_id"] == "structure-graphs-are-carried")
        self.assertEqual(graph["verdict"], "failed")
        self.assertTrue(any("blockDefinitionDiagram" in failure for failure in graph["failures"]))
        self.assertTrue(any("internalBlockDiagram" in failure for failure in graph["failures"]))

    def test_design_declaration_gate_locator_command_and_rule_mutations(self) -> None:
        fixture = self.directory / "design-mutations"
        paths = simple_design.list_inputs(ROOT)
        for path in paths:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        original_hashes = {path: hashlib.sha256((fixture / path).read_bytes()).hexdigest() for path in paths}
        design = simple_design.DESIGN
        mutations = [
            (design, "part def SimpleFlowRun :> FlowExecution {",
             "part def RenamedFlowRun :> FlowExecution {", "design-declarations-present"),
            (design, "ref part redefines exitCheckpoints = simpleRequirementsAccepted;",
             "", "every-stage-gated"),
            (design, "item simpleImplementationCheck : Checker subsets checkers {",
             "item simpleImplementationCheck : AlternateChecker subsets checkers {", "every-stage-gated"),
            (design, "ref item redefines checkers = simpleImplementationCheck;",
             "ref item redefines checkers = simpleDesignCheck;", "every-stage-gated"),
            (design, 'part def SimpleFlowRun :> FlowExecution {\n        @XDesignCommon::DiagramPresentation { label = "Flow run"; summary = "Visits, gate assessments and advancement"; }\n        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D02"; }',
             'part def SimpleFlowRun :> FlowExecution {\n        @XDesignCommon::DiagramPresentation { label = "Flow run"; summary = "Visits, gate assessments and advancement"; }',
             "design-elements-carry-doc-sections"),
            (design, 'attribute redefines commandUri = "tools/checks/simple_implementation.py";',
             'attribute redefines commandUri = "tools/checks/missing.py";', "checker-commands-exist"),
            (design, 'attribute redefines reads = "mbse/model/requirements.sysml";',
             'attribute redefines reads = "";', "checker-rules-are-declared"),
            ("tools/checks/simple_implementation.py", '"visit-exists",',
             '"different-rule",', "rule-sets-agree-with-commands"),
        ]
        for path, before, after, target in mutations:
            with self.subTest(rule=target):
                source_file = fixture / path
                original = source_file.read_text()
                self.assertIn(before, original)
                try:
                    source_file.write_text(original.replace(before, after, 1))
                    hashes = original_hashes.copy()
                    hashes[path] = hashlib.sha256(source_file.read_bytes()).hexdigest()
                    context = common.Context(fixture, argparse.Namespace(), {}, {}, {}, hashes, {})
                    results = simple_design.evaluate(context)
                    selected = next(item for item in results if item["rule_id"] == target)
                    self.assertEqual(selected["verdict"], "failed", selected)
                    self.assertTrue(selected["failures"])
                    if "AlternateChecker" in after:
                        by_rule = {item["rule_id"]: item for item in results}
                        for related in ("checker-rules-are-declared", "rule-sets-agree-with-commands"):
                            self.assertEqual(by_rule[related]["verdict"], "failed", by_rule[related])
                            self.assertTrue(by_rule[related]["failures"])
                finally:
                    source_file.write_text(original)

    def test_record_rule_mutations_name_specific_failures(self) -> None:
        baseline = self.make_record(3, [artifact("mbse/model/requirements.sysml")])
        plan = json.loads(self.plan_path.read_text())
        def run(record: dict) -> dict[str, dict]:
            context = common.Context(ROOT, argparse.Namespace(record=uri(self.record_path)),
                                     record, plan, record["visits"][-1], {}, {})
            return {entry["rule_id"]: entry for entry in simple_record.evaluate(context)}

        cases = [
            ("flow-is-supported", lambda item: item.__setitem__("plan_id", "otherFlow")),
            ("every-stage-visited", lambda item: item["visits"][1].__setitem__("stage_id", "outsidePlan")),
            ("completed-visits-authorized", lambda item: item["visits"][0]["assessments"][0].__setitem__("outcome", "failed")),
            ("assessments-name-exit-gates", lambda item: item["visits"][0]["assessments"][0].__setitem__("checkpoint_id", "simpleDesignAccepted")),
            ("passed-assessments-carry-evidence", lambda item: item["visits"][0]["assessments"][0].__setitem__("evidence_uris", [])),
            ("evidence-exists", lambda item: item["visits"][0]["assessments"][0].__setitem__("evidence_uris", [uri(self.directory / "missing.txt")])),
            ("designed-runs-are-typed-by-the-design-class", lambda item: item.__setitem__("execution_type", "FlowExecution")),
        ]
        for target, mutate in cases:
            with self.subTest(rule=target):
                record = copy.deepcopy(baseline)
                mutate(record)
                selected = run(record)[target]
                self.assertEqual(selected["verdict"], "failed", selected)
                self.assertTrue(selected["failures"])


if __name__ == "__main__":
    unittest.main()
