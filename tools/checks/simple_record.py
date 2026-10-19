#!/usr/bin/env python3
"""Apply the seven model-declared SimpleFlow run-record gate rules."""

from __future__ import annotations

from pathlib import Path

from common import (CheckInputError, Context, execute, read_bytes,
                    repository_path, require_hex, result, sha256, snapshot_digest)


CHECKER_ID = "simpleRecordCheck"
RULES = (
    "flow-is-supported",
    "every-stage-visited",
    "completed-visits-authorized",
    "assessments-name-exit-gates",
    "passed-assessments-carry-evidence",
    "evidence-exists",
    "designed-runs-are-typed-by-the-design-class",
)


def _assessment_outcome(assessment: dict) -> str | None:
    value = assessment.get("outcome")
    return value.lower() if isinstance(value, str) else None


def _visit_id(visit: dict) -> str:
    return f"{visit.get('stage_id')}[{visit.get('visit_index')}]"


def _gate_map(plan: dict) -> dict[str, list[str]]:
    stages = plan.get("stages")
    if not isinstance(stages, list) or any(not isinstance(stage, dict) for stage in stages):
        raise CheckInputError("frozen plan stages are malformed")
    gates: dict[str, list[str]] = {}
    for stage in stages:
        stage_id = stage.get("stage_id")
        exits = stage.get("exit_checkpoint_ids")
        if not isinstance(stage_id, str) or not isinstance(exits, list) or any(not isinstance(item, str) for item in exits):
            raise CheckInputError("frozen plan stage exits are malformed")
        if stage_id in gates:
            raise CheckInputError("frozen plan repeats a stage")
        gates[stage_id] = exits
    if not gates:
        raise CheckInputError("frozen plan has no stages")
    return gates


def _record_visits(record: dict) -> list[dict]:
    visits = record.get("visits")
    if not isinstance(visits, list) or any(not isinstance(visit, dict) for visit in visits):
        raise CheckInputError("run record visits are malformed")
    return visits


def _completed_snapshot_failures(context: Context, visit: dict) -> list[str]:
    """Catch out-of-band edits to previously authorized visit artifacts."""
    visit_id = _visit_id(visit)
    artifacts = visit.get("artifact_digests")
    if not isinstance(artifacts, list) or any(not isinstance(item, dict) for item in artifacts):
        return [f"Completed visit {visit_id} has malformed artifact digests."]
    failures = []
    canonical = []
    seen: set[str] = set()
    for item in artifacts:
        uri = item.get("uri")
        digest = item.get("sha256")
        if not isinstance(uri, str) or not isinstance(digest, str):
            failures.append(f"Completed visit {visit_id} has malformed artifact digest entry.")
            continue
        if uri in seen:
            failures.append(f"Completed visit {visit_id} repeats artifact {uri}.")
            continue
        seen.add(uri)
        canonical.append({"uri": uri, "sha256": digest})
        try:
            require_hex(digest, f"completed artifact {uri} digest")
            actual = sha256(read_bytes(context.root, uri))
            if actual != digest:
                failures.append(f"Completed visit {visit_id} artifact {uri} changed after assessment.")
        except CheckInputError:
            failures.append(f"Completed visit {visit_id} artifact {uri} is missing or malformed.")
    try:
        digest = snapshot_digest(canonical)
        if digest != visit.get("artifact_snapshot_digest"):
            failures.append(f"Completed visit {visit_id} artifact snapshot digest differs from its recorded value.")
    except (CheckInputError, UnicodeError):
        failures.append(f"Completed visit {visit_id} artifact snapshot cannot be recomputed.")
    return failures


def evaluate(context: Context) -> list[dict]:
    gates = _gate_map(context.plan)
    visits = _record_visits(context.record)
    results = []
    plan_hash = sha256(read_bytes(context.root, context.record["plan_uri"]))
    identity = {"plan_id": context.record.get("plan_id"),
                "plan_sha256": context.record.get("plan_sha256"),
                "computed_plan_sha256": plan_hash}
    identity_failures = []
    if identity["plan_id"] != "designedSimpleFlow":
        identity_failures.append("Run record names another flow plan.")
    if identity["plan_sha256"] != plan_hash:
        identity_failures.append("Run record plan SHA-256 differs from retained frozen plan serialization.")
    results.append(result(RULES[0], [context.args.record, context.record["plan_uri"]], identity, identity_failures))

    stage_ids = list(gates)
    links = [{"visit_id": _visit_id(visit), "stage_id": visit.get("stage_id")}
             for visit in visits]
    missing = [stage for stage in stage_ids if not any(link["stage_id"] == stage for link in links)]
    outside = [link for link in links if link["stage_id"] not in gates]
    failures = [f"Flow stage {stage} lacks a visit." for stage in missing]
    failures.extend(f"Visit {link['visit_id']} names a stage outside the plan." for link in outside)
    results.append(result(RULES[1], [context.args.record],
                          {"planned_stage_ids": stage_ids, "visit_stage_links": links,
                           "missing_stage_ids": missing}, failures))

    completed = []
    completion_failures = []
    for visit in visits:
        if visit.get("status") != "completed":
            continue
        visit_id = _visit_id(visit)
        stage_id = visit.get("stage_id")
        required = gates.get(stage_id, [])
        assessments = visit.get("assessments")
        if not isinstance(assessments, list) or any(not isinstance(entry, dict) for entry in assessments):
            completion_failures.append(f"Completed visit {visit_id} has malformed assessments.")
            continue
        latest = {}
        for assessment in assessments:
            latest[assessment.get("checkpoint_id")] = assessment
        facts = []
        for gate in required:
            assessment = latest.get(gate)
            fact = {"gate": gate, "latest": assessment}
            facts.append(fact)
            if (assessment is None or _assessment_outcome(assessment) != "passed"
                    or assessment.get("evaluated_revision") != visit.get("artifact_revision")
                    or assessment.get("evaluated_snapshot_digest") != visit.get("artifact_snapshot_digest")):
                completion_failures.append(f"Completed visit {visit_id} has a missing, failed, unevaluated, or stale latest assessment of exit gate {gate}.")
        completion_failures.extend(_completed_snapshot_failures(context, visit))
        completed.append({"visit_id": visit_id, "revision": visit.get("artifact_revision"),
                          "required_gates": required, "latest_assessments": facts,
                          "artifact_snapshot_digest": visit.get("artifact_snapshot_digest")})
    results.append(result(RULES[2], [context.args.record] + [item["uri"] for visit in visits if visit.get("status") == "completed"
                                                     for item in visit.get("artifact_digests", []) if isinstance(item, dict) and "uri" in item],
                          {"completed_visits": completed}, completion_failures))

    passed = []
    gate_failures = []
    evidence_failures = []
    evidence_existence_failures = []
    evidence_inspected = []
    for visit in visits:
        visit_id = _visit_id(visit)
        allowed = gates.get(visit.get("stage_id"), [])
        assessments = visit.get("assessments")
        if not isinstance(assessments, list):
            raise CheckInputError(f"visit {visit_id} assessments are malformed")
        for index, assessment in enumerate(assessments):
            if not isinstance(assessment, dict):
                raise CheckInputError(f"visit {visit_id} assessment is malformed")
            if _assessment_outcome(assessment) != "passed":
                continue
            assessment_id = f"{visit_id}/assessment[{index}]"
            checkpoint = assessment.get("checkpoint_id")
            evidence = assessment.get("evidence_uris")
            passed.append({"assessment_id": assessment_id, "checkpoint_id": checkpoint,
                           "exit_gate_ids": allowed, "evidence_uris": evidence})
            if checkpoint not in allowed:
                gate_failures.append(f"Passed assessment {assessment_id} names gate {checkpoint} outside visit {visit_id}'s stage exits.")
            if not isinstance(evidence, list) or not evidence or any(not isinstance(uri, str) or not uri.strip() for uri in evidence):
                evidence_failures.append(f"Passed assessment {assessment_id} has no evidence URI.")
                continue
            if visit.get("status") in {"completed", "active", "blocked"}:
                for uri in evidence:
                    try:
                        exists = repository_path(context.root, uri).is_file()
                    except CheckInputError:
                        exists = False
                    evidence_inspected.append({"assessment_id": assessment_id, "uri": uri, "exists": exists})
                    if not exists:
                        evidence_existence_failures.append(f"Passed assessment {assessment_id} on visit {visit_id} cites missing evidence {uri}.")
    results.append(result(RULES[3], [context.args.record], {"passed_assessments": passed}, gate_failures))
    results.append(result(RULES[4], [context.args.record], {"passed_assessments": passed}, evidence_failures))
    results.append(result(RULES[5], [context.args.record] + [entry["uri"] for entry in evidence_inspected],
                          {"evidence": evidence_inspected}, evidence_existence_failures))

    execution_type = context.record.get("execution_type")
    type_failures = [] if execution_type == "SimpleFlowRun" else [
        "Run on designedSimpleFlow lacks SimpleFlowRun typing."
    ]
    results.append(result(RULES[6], [context.args.record], {
        "plan_id": context.record.get("plan_id"), "execution_type": execution_type,
    }, type_failures))
    return results


if __name__ == "__main__":
    raise SystemExit(execute(CHECKER_ID, RULES, evaluate))
