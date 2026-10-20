#!/usr/bin/env python3
"""Apply the two model-declared SimpleFlow requirements gate rules."""

from __future__ import annotations

import re
from pathlib import Path

from common import (CheckInputError, Context, attribute_string, code_mask, execute,
                    find_blocks, require_sysml_parse, result)


CHECKER_ID = "simpleRequirementsCheck"
RULES = (
    "requirements-carry-rationale",
    "satisfy-links-resolve",
)
INPUTS = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml")
IDENTIFIER = re.compile(r"[A-Za-z_]\w*\b")
REQUIREMENT_KEYWORD = re.compile(r"\brequirement\b")
SATISFY_LINK = re.compile(
    r"\bsatisfy\s+((?:::)?[A-Za-z_]\w*(?:(?:::|\.)[A-Za-z_]\w*)*)\s+by\s+([^;]+);"
)


def _matching_brace(masked: str, opening: int) -> int:
    depth = 0
    for cursor in range(opening, len(masked)):
        if masked[cursor] == "{":
            depth += 1
        elif masked[cursor] == "}":
            depth -= 1
            if depth == 0:
                return cursor
    raise CheckInputError("unclosed requirement body")


def _requirements_in_package(package: str) -> list[tuple[str, str | None]]:
    """Enumerate usages by keyword, including modifiers and specialization forms."""
    masked = code_mask(package)
    nested_packages = []
    for match, body, _ in find_blocks(package, r"\bpackage\s+([A-Za-z_]\w*)\s*\{"):
        opening = masked.find("{", match.start(), match.end())
        nested_packages.append((opening, opening + len(body) - 1, match.group(1)))
    declarations = []
    for keyword in REQUIREMENT_KEYWORD.finditer(masked):
        cursor = keyword.end()
        while cursor < len(masked) and masked[cursor].isspace():
            cursor += 1
        name_match = IDENTIFIER.match(masked, cursor)
        if name_match is None:
            raise CheckInputError(f"unrecognized requirement declaration near offset {keyword.start()}")
        if name_match.group() == "def":
            continue
        name = name_match.group()
        terminator = re.search(r"[;{}]", masked[name_match.end():])
        if terminator is None:
            raise CheckInputError(f"unterminated requirement declaration {name}")
        opening = name_match.end() + terminator.start()
        if masked[opening] == "}":
            raise CheckInputError(f"unrecognized requirement declaration {name}")
        body = package[opening:_matching_brace(masked, opening) + 1] if masked[opening] == "{" else None
        scope = [entry for entry in nested_packages if entry[0] < keyword.start() < entry[1]]
        scope.sort(key=lambda entry: entry[0])
        qualified = "::".join([entry[2] for entry in scope] + [name])
        declarations.append((qualified, body))
    return declarations


def _satisfy_links(source: str) -> list[tuple[str, str]]:
    """Inspect every link in the declared traceability file, regardless of package."""
    masked = code_mask(source)
    links = []
    for keyword in re.finditer(r"\bsatisfy\b", masked):
        match = SATISFY_LINK.match(masked, keyword.start())
        if match is None:
            raise CheckInputError(f"unrecognized satisfy link near offset {keyword.start()}")
        links.append((match.group(1), source[match.start(2):match.end(2)].strip()))
    return links


def resolves_target(target: str, declared: set[str]) -> bool:
    absolute = target.startswith("::")
    parts = re.split(r"::|\.", target.removeprefix("::"))
    if parts[0] == "XDesignRequirements":
        parts = parts[1:]
    elif absolute or len(parts) > 1:
        return False
    return bool(parts) and "::".join(parts) in declared


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
    declarations = _requirements_in_package(package)
    names = [name for name, _ in declarations]
    if len(names) != len(set(names)):
        raise CheckInputError("XDesignRequirements repeats a requirement name")
    missing = []
    rationale: dict[str, str | None] = {}
    for name, body in declarations:
        value = attribute_string(body, "rationale") if body is not None else None
        rationale[name] = value
        if value is None or not value.strip():
            missing.append(f"Requirement {name} has no rationale attribute.")
    if not names:
        missing.append("XDesignRequirements declares no requirements.")
    first = result(RULES[0], [INPUTS[0]], {
        "requirement_ids": names,
        "missing_rationale_ids": [name for name, value in rationale.items() if not value or not value.strip()],
    }, missing)

    links = _satisfy_links(traceability_text)
    failed = []
    if not links:
        failed.append("The declared traceability source has no satisfy links.")
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
