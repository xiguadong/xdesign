#!/usr/bin/env python3
"""Apply the two model-declared SimpleFlow requirements gate rules."""

from __future__ import annotations

import re
from pathlib import Path

from common import (CheckInputError, Context, attribute_string, execute, find_blocks,
                    require_sysml_parse, result)


CHECKER_ID = "simpleRequirementsCheck"
RULES = (
    "requirements-carry-rationale",
    "satisfy-links-resolve",
)
INPUTS = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml")
REQUIREMENT_USAGE = r"\brequirement\s+(?!def\b)([A-Za-z_]\w*)(?:\s*:\s*[A-Za-z_]\w*(?:(?:::|\.)[A-Za-z_]\w*)*)?\s*\{"
REQUIREMENT_NO_BODY = re.compile(r"\brequirement\s+(?!def\b)([A-Za-z_]\w*)(?:\s*:\s*[A-Za-z_]\w*(?:(?:::|\.)[A-Za-z_]\w*)*)?\s*;")


def resolves_target(target: str, declared: set[str]) -> bool:
    absolute = target.startswith("::")
    parts = re.split(r"::|\.", target.removeprefix("::"))
    return (not absolute and len(parts) == 1 and parts[0] in declared) or (
        len(parts) == 2 and parts[0] == "XDesignRequirements" and parts[1] in declared
    )


def list_inputs(root: Path) -> list[str]:
    return list(INPUTS)


def evaluate(context: Context) -> list[dict]:
    requirements_text = context.read_rule_text(INPUTS[0])
    traceability_text = context.read_rule_text(INPUTS[1])
    require_sysml_parse(context.root, list(INPUTS))
    packages = find_blocks(requirements_text, r"\bpackage\s+XDesignRequirements\s*\{")
    if len(packages) != 1:
        raise CheckInputError("expected exactly one XDesignRequirements package")
    package = packages[0][1]
    declarations = find_blocks(package, REQUIREMENT_USAGE)
    no_body = [match.group(1) for match in REQUIREMENT_NO_BODY.finditer(packages[0][2])]
    names = [match.group(1) for match, _, _ in declarations] + no_body
    if len(names) != len(set(names)):
        raise CheckInputError("XDesignRequirements repeats a requirement name")
    missing = []
    rationale: dict[str, str | None] = {}
    for match, body, _ in declarations:
        name = match.group(1)
        value = attribute_string(body, "rationale")
        rationale[name] = value
        if value is None or not value.strip():
            missing.append(f"Requirement {name} has no rationale attribute.")
    for name in no_body:
        rationale[name] = None
        missing.append(f"Requirement {name} has no rationale attribute.")
    if not names:
        missing.append("XDesignRequirements declares no requirements.")
    first = result(RULES[0], [INPUTS[0]], {
        "requirement_ids": names,
        "missing_rationale_ids": [name for name, value in rationale.items() if not value or not value.strip()],
    }, missing)

    trace_packages = find_blocks(traceability_text, r"\bpackage\s+XDesignTraceability\s*\{")
    if len(trace_packages) != 1:
        raise CheckInputError("expected exactly one XDesignTraceability package")
    masked = trace_packages[0][2]
    link_pattern = r"\bsatisfy\s+((?:::)?[A-Za-z_]\w*(?:(?:::|\.)[A-Za-z_]\w*)*)\s+by\s+([^;]+);"
    links = [(match.group(1), match.group(2).strip()) for match in re.finditer(link_pattern, masked)]
    failed = []
    if not links:
        failed.append("XDesignTraceability declares no satisfy links.")
    for index, (target, _) in enumerate(links, 1):
        if not resolves_target(target, set(rationale)):
            failed.append(f"Satisfy link {index} targets undeclared requirement {target}.")
    second = result(RULES[1], list(INPUTS), {
        "satisfy_links": [
            {"ordinal": index, "target": target, "satisfier": satisfier,
             "resolved_requirement": resolves_target(target, set(rationale))}
            for index, (target, satisfier) in enumerate(links, 1)
        ],
        "declared_requirement_ids": names,
    }, failed)
    return [first, second]


if __name__ == "__main__":
    raise SystemExit(execute(CHECKER_ID, RULES, evaluate, list_inputs))
