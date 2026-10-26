#!/usr/bin/env python3
"""Apply the seven model-declared SimpleFlow design gate rules."""

from __future__ import annotations

import ast
import json
import os
from pathlib import Path
import re

from common import (CheckInputError, Context, ROOT, code_mask, execute,
                    find_blocks, repository_path, require_sysml_parse, result)


CHECKER_ID = "simpleDesignCheck"
RULES = (
    "design-declarations-present",
    "every-stage-gated",
    "design-elements-carry-doc-sections",
    "checker-commands-exist",
    "checker-rules-are-declared",
    "rule-sets-agree-with-commands",
    "structure-graphs-are-carried",
)
DESIGN = "mbse/model/detailedDesign.sysml"
COMPOSITION = "mbse/model/composition.sysml"
STAGES = ("requirements", "design", "implement", "verify")
EXPECTED_GATE_CHECKERS = {
    "simpleRequirementsAccepted": ("requirements", "simpleRequirementsCheck"),
    "simpleDesignAccepted": ("design", "simpleDesignCheck"),
    "simpleImplementationAccepted": ("implement", "simpleImplementationCheck"),
    "simpleRecordAccepted": ("verify", "simpleRecordCheck"),
}
EXPECTED_EXITS = {
    "requirements": {"simpleRequirementsAccepted"},
    "design": {"simpleDesignAccepted"},
    "implement": {"simpleImplementationAccepted"},
    "verify": {"simpleRecordAccepted", "verificationAccepted"},
}
REQUIRED_DECLARATIONS = {
    "DesignedSimpleFlow": r"\bpart\s+def\s+DesignedSimpleFlow\s*:>\s*SimpleFlow\s*\{",
    "SimpleFlowRun": r"\bpart\s+def\s+SimpleFlowRun\s*:>\s*FlowExecution\s*\{",
    "SimpleFlowRunFactory": r"\bpart\s+def\s+SimpleFlowRunFactory\s*\{",
    "EvaluateSimpleFlowGate": r"\baction\s+def\s+EvaluateSimpleFlowGate\s*\{",
    "AdvanceSimpleFlowRun": r"\baction\s+def\s+AdvanceSimpleFlowRun\s*\{",
    "SimpleFlowCheckpoint": r"\bpart\s+def\s+SimpleFlowCheckpoint\s*:>\s*Checkpoint\s*\{",
    "SimpleFlowSyntacticCheckpoint": r"\bpart\s+def\s+SimpleFlowSyntacticCheckpoint\s*:>\s*SimpleFlowCheckpoint\s*\{",
    "SimpleFlowSemanticCheckpoint": r"\bpart\s+def\s+SimpleFlowSemanticCheckpoint\s*:>\s*SimpleFlowCheckpoint\s*\{",
    "EvaluateSimpleFlowCheckpoint": r"\baction\s+def\s+EvaluateSimpleFlowCheckpoint\s*\{",
}
REQUIRED_VIEWS = (
    "blockDefinitionDiagram", "internalBlockDiagram", "requirementsDiagram",
    "parametricDiagram", "useCaseDiagram", "activityDiagram", "sequenceDiagram",
    "stateMachineDiagram", "packageDiagram",
)
REQUIRED_VIEW_FIGURES = {
    "blockDefinitionDiagram": ("docs/design/flows/simple-flow/views.md", "docs/design/flows/simple-flow/assets/block-definition.svg"),
    "internalBlockDiagram": ("docs/design/flows/simple-flow/views.md", "docs/design/flows/simple-flow/assets/internal-block.svg"),
    "requirementsDiagram": ("docs/verification/requirements.md", "docs/verification/assets/requirements.svg"),
    "parametricDiagram": ("docs/design/flows/simple-flow/views.md", "docs/design/flows/simple-flow/assets/parametric.svg"),
    "useCaseDiagram": ("docs/requirements/requirements.md", "docs/requirements/assets/use-cases.svg"),
    "activityDiagram": ("docs/design/flows/simple-flow/views.md", "docs/design/flows/simple-flow/assets/activity.svg"),
    "sequenceDiagram": ("docs/design/flows/simple-flow/views.md", "docs/design/flows/simple-flow/assets/sequence.svg"),
    "stateMachineDiagram": ("docs/design/flows/simple-flow/views.md", "docs/design/flows/simple-flow/assets/state-machine.svg"),
    "packageDiagram": ("docs/architecture/system-composition.md", "docs/architecture/assets/simple-flow-packages.svg"),
}
DOC_SECTION = re.compile(r'@XDesignCommon::DocSection\s*\{\s*document\s*=\s*"([^"\n]+)"\s*;\s*section\s*=\s*"([^"\n]+)"\s*;\s*\}')
DOC_FIGURE = re.compile(r'@XDesignCommon::DocFigure\s*\{\s*document\s*=\s*"([^"\n]+)"\s*;\s*figure\s*=\s*"([^"\n]+)"\s*;\s*\}')
COMMAND_URI = re.compile(r'\battribute\s+redefines\s+commandUri\s*=\s*"([^"\n]+)"\s*;')
ELEMENT_KINDS = {
    "action", "allocation", "analysis", "attribute", "behavior", "calc",
    "connection", "constraint", "enum", "flow", "interface", "item",
    "metadata", "occurrence", "package", "part", "port", "requirement",
    "state", "succession", "verification", "view", "viewpoint",
}
DECLARATION_MODIFIERS = {"abstract", "individual", "private", "protected", "public", "ref", "variation"}
IDENTIFIER = re.compile(r"'?[A-Za-z_]\w*'?")


def live_matches(source: str, pattern: re.Pattern[str]) -> list[re.Match[str]]:
    mask = code_mask(source)
    return [match for match in pattern.finditer(source) if mask[match.start()] != " "]


def annotations(source: str, pattern: re.Pattern[str]) -> list[tuple[str, str]]:
    return [(match.group(1), match.group(2)) for match in live_matches(source, pattern)]


def direct_annotations(source: str, pattern: re.Pattern[str]) -> list[tuple[str, str]]:
    """Only annotations directly owned by a brace-delimited declaration."""
    masked = code_mask(source)
    return [(match.group(1), match.group(2)) for match in live_matches(source, pattern)
            if masked[:match.start()].count("{") - masked[:match.start()].count("}") == 1]


def _closing_brace(masked: str, opening: int) -> int:
    depth = 0
    for cursor in range(opening, len(masked)):
        if masked[cursor] == "{":
            depth += 1
        elif masked[cursor] == "}":
            depth -= 1
            if depth == 0:
                return cursor
    raise CheckInputError("unclosed top-level detailed design element")


def _element_name(header: str) -> str:
    words = [match.group().strip("'") for match in IDENTIFIER.finditer(header)]
    index = 0
    while index < len(words) and words[index] in DECLARATION_MODIFIERS:
        index += 1
    if words[index:index + 2] == ["use", "case"]:
        index += 2
    elif index < len(words) and words[index] in ELEMENT_KINDS:
        index += 1
    else:
        raise CheckInputError(f"unclassified top-level detailed design declaration: {header.strip()}")
    if index < len(words) and words[index] == "def":
        index += 1
    if index < len(words) and words[index] == "redefines":
        index += 1
    if index >= len(words):
        raise CheckInputError(f"unnamed top-level detailed design declaration: {header.strip()}")
    return words[index]


def top_level(source: str) -> list[tuple[str, str]]:
    """Scan immediate package members independently of line breaks and indentation.

    The pinned parser validates syntax before this scan. Unknown member forms raise
    an input error, so a newly introduced declaration cannot silently evade the
    documentation rule.
    """
    packages = find_blocks(source, r"\bpackage\s+XDesignDetailedDesign\s*\{")
    if len(packages) != 1:
        raise CheckInputError("expected exactly one XDesignDetailedDesign package")
    package = packages[0][1]
    masked = code_mask(package)
    elements = []
    cursor = 1
    while cursor < len(masked) - 1:
        while cursor < len(masked) - 1 and masked[cursor].isspace():
            cursor += 1
        if cursor >= len(masked) - 1:
            break
        if re.match(r"doc\b", masked[cursor:]):
            cursor += 3  # Its comment or string is blanked by code_mask.
            continue
        delimiter = next((index for index in range(cursor, len(masked))
                          if masked[index] in "{;}"), None)
        if delimiter is None or masked[delimiter] == "}":
            raise CheckInputError("unclosed top-level detailed design declaration")
        header = masked[cursor:delimiter]
        if masked[cursor] == "@":
            # Metadata usages annotate the package or the following member.
            cursor = _closing_brace(masked, delimiter) + 1 if masked[delimiter] == "{" else delimiter + 1
            continue
        if re.fullmatch(r"\s*(?:(?:private|protected|public)\s+)?import\b[^;{}]*", header):
            if masked[delimiter] != ";":
                raise CheckInputError("an import in detailed design lacks a semicolon")
            cursor = delimiter + 1
            continue
        name = _element_name(header)
        if masked[delimiter] == "{":
            ending = _closing_brace(masked, delimiter)
            elements.append((name, package[delimiter:ending + 1]))
            cursor = ending + 1
        else:
            elements.append((name, ""))
            cursor = delimiter + 1
    return elements


def attribute_expression(body: str, field: str) -> str | None:
    masked = code_mask(body)
    starts = list(re.finditer(r"\battribute\s+redefines\s+" + re.escape(field) + r"\s*=", masked))
    if len(starts) != 1:
        return None
    start = starts[0].end()
    while start < len(body) and body[start].isspace():
        start += 1
    end = masked.find(";", start)
    return body[start:end].strip() if end >= 0 else None


def checkers(source: str) -> dict[str, dict]:
    found: dict[str, dict] = {}
    for match, body, _ in find_blocks(source, r"\bitem\s+([A-Za-z_]\w*)\s*:\s*Checker\s+subsets\s+checkers\s*\{"):
        name = match.group(1)
        command = live_matches(body, COMMAND_URI)
        rules = []
        for rule_match, rule_body, _ in find_blocks(body, r"\bitem\s+([A-Za-z_]\w*)\s*:\s*CheckerRule\s+subsets\s+rules\s*\{"):
            fields = {key: attribute_expression(rule_body, key) for key in ("ruleId", "condition", "reads")}
            rule_id = re.fullmatch(r'"([^"\n]*)"', fields["ruleId"] or "")
            fields["ruleId"] = rule_id.group(1) if rule_id else None
            fields["name"] = rule_match.group(1)
            rules.append(fields)
        found[name] = {"command": command[0].group(1) if len(command) == 1 else None,
                       "rules": rules}
    return found


def _rule_ids_from_python(source: str) -> list[str] | None:
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return None
    for statement in tree.body:
        if isinstance(statement, ast.Assign) and any(
                isinstance(target, ast.Name) and target.id == "RULES" for target in statement.targets):
            try:
                value = ast.literal_eval(statement.value)
            except (ValueError, TypeError, SyntaxError):
                return None
            if isinstance(value, (tuple, list)) and all(isinstance(item, str) for item in value):
                return list(value)
    return None


def _existing_file(root: Path, uri: str) -> bool:
    try:
        return repository_path(root, uri).is_file()
    except CheckInputError:
        return False


def command_uri(uri: str) -> str:
    """Resolve logical built-in source names to this run's retained resources."""
    mapping = json.loads(os.environ.get("XDESIGN_CHECKER_COMMANDS_JSON", "{}"))
    return mapping.get(uri, uri)


def list_inputs(root: Path = ROOT) -> list[str]:
    """Enumerate rule-read files for the CLI to snapshot before design evaluation."""
    source = (root / DESIGN).read_text(encoding="utf-8")
    paths = {DESIGN, COMPOSITION}
    paths.update(document for document, _ in annotations(source, DOC_SECTION))
    for document, figure in annotations(source, DOC_FIGURE):
        paths.add(document)
        paths.add(figure)
    paths.update(command_uri(entry["command"]) for entry in checkers(source).values() if entry["command"])
    return sorted(path for path in paths if not path.startswith("cache/xdesign/") and _existing_file(root, path))


def evaluate(context: Context) -> list[dict]:
    source = context.read_rule_text(DESIGN)
    composition = context.read_rule_text(COMPOSITION)
    require_sysml_parse(context.root, [DESIGN, COMPOSITION])
    code = code_mask(source)
    flow_blocks = find_blocks(source, REQUIRED_DECLARATIONS["DesignedSimpleFlow"])
    flow = flow_blocks[0][1] if len(flow_blocks) == 1 else ""
    checker_map = checkers(flow)
    for entry in checker_map.values():
        if entry["command"]:
            entry["command"] = command_uri(entry["command"])
    top = top_level(source)
    results = []

    present = {name: bool(re.search(pattern, code)) for name, pattern in REQUIRED_DECLARATIONS.items()}
    present["xdesign.designedSimpleFlow"] = bool(re.search(
        r"\bpart\s+designedSimpleFlow\s*:\s*DesignedSimpleFlow\s*:>\s*'flow'\s+subsets\s+availableFlows\s*\{",
        code_mask(composition)))
    results.append(result(RULES[0], [DESIGN, COMPOSITION], {"declarations": present}, [
        f"Required design declaration {name} is missing." for name, exists in present.items() if not exists
    ]))

    gate_types = {
        match.group(1): match.group(2)
        for match in re.finditer(r"\bpart\s+([A-Za-z_]\w*)\s*:\s*([A-Za-z_]\w*)\s+subsets\s+checkpoints\s*\{", code_mask(flow))
    }
    inherited = re.search(r"\bpart\s+redefines\s+verificationAccepted\s*:\s*SimpleFlowSemanticCheckpoint\s*\{", code_mask(flow))
    if inherited:
        gate_types["verificationAccepted"] = "SimpleFlowSemanticCheckpoint"
    gate_map = {}
    gate_failures = []
    checker_declarations = find_blocks(flow, r"\bitem\s+([A-Za-z_]\w*)\s*:\s*Checker\s+subsets\s+checkers\s*\{")
    checker_names = [match.group(1) for match, _, _ in checker_declarations]
    expected_checkers = {checker for _, checker in EXPECTED_GATE_CHECKERS.values()}
    if set(checker_names) != expected_checkers or len(checker_names) != len(expected_checkers):
        gate_failures.append(f"Designed SimpleFlow checker IDs differ from the four fixed gate checkers: expected {sorted(expected_checkers)}, found {sorted(checker_names)}.")
    gate_bindings = {}
    for gate, (stage, checker) in EXPECTED_GATE_CHECKERS.items():
        declarations = find_blocks(flow, r"\bpart\s+" + re.escape(gate) + r"\s*:\s*SimpleFlowSyntacticCheckpoint\s+subsets\s+checkpoints\s*\{")
        binding = None
        source_stage = None
        if len(declarations) == 1:
            body = declarations[0][2]
            binding_match = re.findall(r"\bref\s+item\s+redefines\s+checkers\s*=\s*([A-Za-z_]\w*)\s*;", body)
            source_match = re.findall(r"\bref\s+part\s+redefines\s+sourceStage\s*=\s*'?([A-Za-z_]\w*)'?\s*;", body)
            binding = binding_match[0] if len(binding_match) == 1 else None
            source_stage = source_match[0] if len(source_match) == 1 else None
        gate_bindings[gate] = {"expected_stage": stage, "source_stage": source_stage,
                               "expected_checker": checker, "bound_checker": binding}
        if len(declarations) != 1 or binding != checker or source_stage != stage or checker not in checker_map:
            gate_failures.append(f"Syntactic checkpoint {gate} must belong to stage {stage} and bind declared checker {checker}.")
    for stage in STAGES:
        named = find_blocks(flow, r"\bpart\s+redefines\s+'?" + stage + r"'?\s*\{")
        exits: list[str] = []
        if len(named) == 1:
            match = re.search(r"\bref\s+part\s+redefines\s+exitCheckpoints\s*=\s*([^;]+);", named[0][2])
            if match:
                exits = re.findall(r"[A-Za-z_]\w*", match.group(1))
        gate_map[stage] = {gate: gate_types.get(gate) for gate in exits}
        if not exits:
            gate_failures.append(f"Stage {stage} has no exit checkpoint.")
        if set(exits) != EXPECTED_EXITS[stage] or len(exits) != len(EXPECTED_EXITS[stage]):
            gate_failures.append(f"Stage {stage} exit checkpoints differ from the designed SimpleFlow: expected {sorted(EXPECTED_EXITS[stage])}, found {exits}.")
        for gate in exits:
            if gate_types.get(gate) not in {"SimpleFlowCheckpoint", "SimpleFlowSyntacticCheckpoint", "SimpleFlowSemanticCheckpoint"}:
                gate_failures.append(f"Checkpoint {gate} of stage {stage} is not a declared SimpleFlowCheckpoint instance in this flow.")
    results.append(result(RULES[1], [DESIGN, COMPOSITION], {"stage_exit_gates": gate_map,
                                               "gate_bindings": gate_bindings,
                                               "declared_checker_ids": checker_names}, gate_failures))

    sections = []
    section_failures = []
    if not top:
        section_failures.append("No top-level detailed design elements were found.")
    for name, body in top:
        targets = direct_annotations(body, DOC_SECTION) if body else []
        if len(targets) != 1:
            section_failures.append(f"Design element {name} has no resolvable DocSection document and anchor.")
            sections.append({"element": name, "target": None, "resolved": False})
            continue
        document, anchor = targets[0]
        exists = _existing_file(context.root, document)
        if exists:
            content = context.read_rule_text(document)
            resolved = bool(re.search(r'<a\s+id="' + re.escape(anchor.lower()) + r'"\s*></a>', content, re.I))
        else:
            resolved = False
        sections.append({"element": name, "document": document, "anchor": anchor, "resolved": resolved})
        if not resolved:
            section_failures.append(f"Design element {name} has no resolvable DocSection document and anchor: {document}#{anchor}.")
    results.append(result(RULES[2], sorted({item["document"] for item in sections if item.get("document")} | {DESIGN}),
                          {"sections": sections}, section_failures))

    commands = []
    command_failures = []
    if not checker_map:
        command_failures.append("Designed SimpleFlow declares no checkers.")
    for name, entry in checker_map.items():
        command = entry["command"]
        exists = bool(command and _existing_file(context.root, command))
        commands.append({"checker": name, "command_uri": command, "exists": exists})
        if not exists:
            command_failures.append(f"Checker {name} names missing command {command}.")
    results.append(result(RULES[3], [DESIGN] + [entry["command"] for entry in checker_map.values()
                                               if entry["command"]],
                          {"commands": commands}, command_failures))

    declared = []
    declaration_failures = []
    if set(checker_names) != expected_checkers or len(checker_names) != len(expected_checkers):
        declaration_failures.append(f"Fixed SimpleFlow checker declarations must be exactly {sorted(expected_checkers)}; found {sorted(checker_names)}.")
    for name, entry in checker_map.items():
        if not entry["rules"]:
            declaration_failures.append(f"Checker {name} has no rule.")
        ids = []
        for rule in entry["rules"]:
            ids.append(rule["ruleId"])
            missing = []
            if not rule.get("ruleId") or not rule["ruleId"].strip():
                missing.append("ruleId")
            for field in ("condition", "reads"):
                value = rule.get(field)
                if not value or not any(literal.strip() for literal in re.findall(r'"([^"\n]*)"', value)):
                    missing.append(field)
            if missing:
                declaration_failures.append(f"Checker {name} rule {rule['name']} lacks {', '.join(missing)}.")
        if len(set(ids)) != len(ids):
            declaration_failures.append(f"Checker {name} repeats a rule ID.")
        declared.append({"checker": name, "rules": entry["rules"]})
    results.append(result(RULES[4], [DESIGN], {"checkers": declared}, declaration_failures))

    agreement = []
    agreement_failures = []
    if set(checker_names) != expected_checkers or len(checker_names) != len(expected_checkers):
        agreement_failures.append(f"Cannot establish all four fixed checker rule sets: expected {sorted(expected_checkers)}, found {sorted(checker_names)}.")
    for name, entry in checker_map.items():
        command = entry["command"]
        ids = None
        if command and _existing_file(context.root, command):
            ids = _rule_ids_from_python(context.read_rule_text(command))
        expected = [rule["ruleId"] for rule in entry["rules"] if rule["ruleId"]]
        difference = sorted(set(expected).symmetric_difference(ids or []))
        agreement.append({"checker": name, "declared_rule_ids": expected, "command_rule_ids": ids,
                          "symmetric_difference": difference})
        if ids is None or len(ids) != len(set(ids)) or set(expected) != set(ids):
            agreement_failures.append(f"Checker {name} declares rule IDs that differ from command {command}: {difference}.")
    results.append(result(RULES[5], [DESIGN] + [entry["command"] for entry in checker_map.values() if entry["command"]],
                          {"rule_sets": agreement}, agreement_failures))

    figures = []
    figure_failures = []
    view_figures = {}
    for name in REQUIRED_VIEWS:
        view = find_blocks(source, r"\bview\s+" + re.escape(name) + r"\s*:\s*[^\n{]+\{")
        targets = annotations(view[0][1], DOC_FIGURE) if len(view) == 1 else []
        view_figures[name] = targets
        if len(targets) != 1:
            figure_failures.append(f"Named view {name} lacks its DocFigure.")
        elif targets[0] != REQUIRED_VIEW_FIGURES[name]:
            figure_failures.append(f"Named view {name} maps to {targets[0]} instead of its required question figure {REQUIRED_VIEW_FIGURES[name]}.")
    primary_assets = [targets[0][1] for targets in view_figures.values() if len(targets) == 1]
    if len(primary_assets) != len(set(primary_assets)):
        figure_failures.append("The nine question views reuse a figure asset instead of carrying distinct figures.")
    all_figures = annotations(source, DOC_FIGURE)
    if len(all_figures) < len(REQUIRED_VIEWS):
        figure_failures.append("The design declares fewer figures than the nine required question views.")
    for document, figure in all_figures:
        document_exists = _existing_file(context.root, document)
        figure_exists = _existing_file(context.root, figure)
        embedded = False
        if document_exists:
            content = context.read_rule_text(document)
            relative = Path(os.path.relpath(context.root / figure, (context.root / document).parent)).as_posix()
            embedded = bool(re.search(r'!\[[^\]]*\]\(\s*<?' + re.escape(relative) + r'>?\s*\)', content))
        if figure_exists:
            context.read_rule_input(figure)
        figures.append({"document": document, "figure": figure, "document_exists": document_exists,
                        "asset_exists": figure_exists, "embedded": embedded})
        if not figure_exists or not embedded:
            figure_failures.append(f"DocFigure asset {figure} is missing or is not embedded in {document}.")
    results.append(result(RULES[6], [DESIGN] + sorted({item["document"] for item in figures} | {item["figure"] for item in figures}),
                          {"figures": figures, "required_view_figures": view_figures}, figure_failures))
    return results


if __name__ == "__main__":
    raise SystemExit(execute(CHECKER_ID, RULES, evaluate, list_inputs))
