#!/usr/bin/env python3
"""Apply the two model-declared SimpleFlow implementation gate rules."""

from __future__ import annotations

from common import Context, execute, result


CHECKER_ID = "simpleImplementationCheck"
RULES = (
    "visit-exists",
    "artifacts-of-current-visit-exist",
)


def evaluate(context: Context) -> list[dict]:
    visit = context.visit
    visit_id = f"{visit['stage_id']}[{visit.get('visit_index')}]"
    matches = [f"{candidate.get('stage_id')}[{candidate.get('visit_index')}]"
               for candidate in context.record.get("visits", [])
               if isinstance(candidate, dict) and candidate.get("stage_id") == context.args.stage]
    current_matches = visit.get("stage_id") == context.args.stage
    visit_failures = [] if current_matches else [f"Checked stage {context.args.stage} has no current visit."]
    first = result(RULES[0], [context.args.record], {
        "checked_stage_id": context.args.stage,
        "matching_visit_ids": matches,
        "current_visit_index": context.record["current_visit"],
    }, visit_failures)
    artifacts = visit["artifact_digests"]
    uris = [item["uri"] for item in artifacts]
    failures = [] if uris else [f"Current implementation visit {visit_id} records no artifact URI."]
    second = result(RULES[1], [context.args.record] + uris, {
        "visit_id": visit_id,
        "artifact_uris": uris,
        "existence": {uri: True for uri in uris},
    }, failures)
    return [first, second]


if __name__ == "__main__":
    raise SystemExit(execute(CHECKER_ID, RULES, evaluate))
