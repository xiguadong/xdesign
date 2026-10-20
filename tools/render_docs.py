#!/usr/bin/env python3
"""Render the documentation's diagrams from the project model.

Every figure in a documentation subdirectory's assets/ folder is derived from
the pinned compiler's model.
Focused documentation views use diagram-specific composition and Graphviz;
native compiler drawings remain available for reference overviews and journeys.
Run it after a model change:

    python3 tools/render_docs.py                          # render every figure
    python3 tools/render_docs.py details/assets/01_controller  # render one figure by name

Figure kinds, matching the views SysML v2 and this model declare:
  definitions  the definitions and their specializations, or one file's definitions
               with the rest of the model loaded as library
  internal     the internal structure of one definition
  sequence     the interaction one occurrence definition declares
  declared     a view the model itself declares, projected from the export
  file_plan    repository files and responsibilities selected by a declared view
  vmodel_overview  stage sequence, exit gates, and verification pairs of VModel
  flow_relationships  Flow definition family and xdesign's selectable usages

The nine stage-owned figures use documentation projections of exposed native IDs:
requirements, packages, block_definition, internal_block, parametric,
use_cases, activity, declared_sequence, and state_machine. Their familiar
diagram shapes are a presentation of the SysML v2 model; they do not introduce
a second model or an additional parser.

A figure's name is its path under docs/, with assets/ beside its owning document.

Command logs and the exported model stay under cache/docs-render/. The exit
status is nonzero when a figure fails; the other figures are still written.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from functools import lru_cache
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO))

from tools.sysml_web.cst import unsupported_view_syntax  # noqa: E402
from tools.sysml_web.declared_views import DeclaredViews  # noqa: E402
from tools.sysml_web.documentation_views import TITLES, render_documentation_view  # noqa: E402
from tools.sysml_web.file_plan_view import render_file_plan  # noqa: E402
from tools.sysml_web.flow_relationships import render_flow_relationships  # noqa: E402
from tools.sysml_web.renderer import _name  # noqa: E402
from tools.sysml_web.vmodel_overview import render_vmodel_overview  # noqa: E402

COMPILER = REPO / "tools" / "sysml_web" / "bin" / "sysml"
MODEL_DIR = REPO / "mbse" / "model"
WISHES_DIR = REPO / "mbse" / "wishes"
DOCS = REPO / "docs"
LOGS = REPO / "cache" / "docs-render"
TMP = REPO / "cache" / "sysml" / "tmp"

# name, kind, target. The name becomes docs/<owner>/assets/<file>.svg.
FIGURES = [
    ("arch/assets/00_model_definitions", "definitions", ""),
    ("arch/assets/00_lifecycle", "declared", "XDesignWorkflow::wishLifecycle"),
    ("arch/assets/03_wish_to_delivery", "vmodel_overview", "XDesignArchitecture::VModel"),
    ("arch/assets/01_composition", "internal", "XDesign"),
    ("arch/assets/01_interfaces", "declared", "XDesignWorkflow::workflowInterfaces"),
    ("details/assets/01_wish_verify_flow", "internal", "WishVerifyFlow"),
    ("details/assets/01_flow_relationships", "flow_relationships", "XDesignComposition::xdesign"),
    ("details/assets/01_controller", "internal", "Controller"),
    ("details/assets/02_records", "declared", "XDesignArchitecture::recordStructure"),
    ("details/assets/02_journey_rework", "sequence", "Wish0002Journey"),
    ("review/assets/01_journey_wish0005", "sequence", "Wish0005Journey"),
    ("review/assets/01_journey_wish0006", "sequence", "Wish0006Journey"),
    ("details/simple_flow/assets/01_designed_flow", "declared", "XDesignDetailedDesign::designedFlowTree"),
    ("details/simple_flow/assets/01_design_classes", "declared", "XDesignDetailedDesign::designTree"),
    ("details/simple_flow/assets/01_factory", "declared", "XDesignDetailedDesign::factoryFlow"),
    ("details/simple_flow/assets/01_gate_evaluation", "declared", "XDesignDetailedDesign::gateEvaluationFlow"),
    ("details/simple_flow/assets/02_checkpoint_evaluation", "declared", "XDesignDetailedDesign::checkpointEvaluationFlow"),
    ("details/simple_flow/assets/03_timeline", "sequence", "SimpleFlowRunTimeline"),
    ("details/simple_flow/assets/03_logical", "definitions", "mbse/model/detailedDesign.sysml"),
    ("details/simple_flow/assets/03_interfaces", "declared", "XDesignDetailedDesign::interfaceStructure"),
    ("requirements/assets/01_requirements", "requirements", "XDesignDetailedDesign::requirementsDiagram"),
    ("arch/assets/02_packages", "packages", "XDesignRustUml::projectPackageIntegration"),
    ("arch/assets/02_simple_flow_packages", "packages", "XDesignDetailedDesign::packageDiagram"),
    ("details/simple_flow/assets/05_block_definition", "block_definition", "XDesignDetailedDesign::blockDefinitionDiagram"),
    ("details/simple_flow/assets/05_internal_block", "internal_block", "XDesignDetailedDesign::internalBlockDiagram"),
    ("details/simple_flow/assets/05_parametric", "parametric", "XDesignDetailedDesign::parametricDiagram"),
    ("requirements/assets/01_use_cases", "use_cases", "XDesignDetailedDesign::useCaseDiagram"),
    ("details/simple_flow/assets/05_activity", "activity", "XDesignDetailedDesign::activityDiagram"),
    ("details/simple_flow/assets/05_sequence", "declared_sequence", "XDesignDetailedDesign::sequenceDiagram"),
    ("details/simple_flow/assets/05_state_machine", "state_machine", "XDesignDetailedDesign::stateMachineDiagram"),
    ("details/simple_flow/assets/06_crate_modules", "packages", "XDesignRustUml::rustModuleLayout"),
    ("details/simple_flow/assets/06_class_structure", "block_definition", "XDesignRustUml::rustClassStructure"),
    ("details/simple_flow/assets/06_gate_sequence", "declared_sequence", "XDesignRustUml::rustGateSequence"),
    ("details/simple_flow/assets/06_run_states", "state_machine", "XDesignRustUml::rustRunStates"),
    ("api/assets/01_crate_layers", "packages", "XDesignRustUml::rustCrateLayers"),
    ("prj_manage/assets/06_file_plan", "file_plan", "XDesignDetailedDesign::simpleFlowFilePlanView"),
    ("review/assets/01_journey_wish0009", "sequence", "Wish0009Journey"),
    ("details/wish_verify_flow/assets/01_designed_flow", "declared", "XDesignDetailedDesign::wishVerifyFlowTree"),
    ("details/wish_verify_flow/assets/01_design_classes", "declared", "XDesignDetailedDesign::wishVerifyDesignTree"),
    ("details/wish_verify_flow/assets/01_factory", "declared", "XDesignDetailedDesign::wishVerifyFactoryFlow"),
    ("details/wish_verify_flow/assets/01_gate_evaluation", "declared", "XDesignDetailedDesign::wishVerifyGateEvaluationFlow"),
    ("details/wish_verify_flow/assets/02_checkpoint_evaluation", "activity", "XDesignDetailedDesign::wishVerifyActivityDiagram"),
    ("details/wish_verify_flow/assets/04_packages", "packages", "XDesignDetailedDesign::wishVerifyPackageDiagram"),
    ("details/wish_verify_flow/assets/04_block_definition", "block_definition", "XDesignDetailedDesign::wishVerifyBlockDefinitionDiagram"),
    ("details/wish_verify_flow/assets/04_internal_block", "internal_block", "XDesignDetailedDesign::wishVerifyInternalBlockDiagram"),
    ("details/wish_verify_flow/assets/04_sequence", "declared_sequence", "XDesignDetailedDesign::wishVerifySequenceDiagram"),
    ("details/wish_verify_flow/assets/04_state_machine", "state_machine", "XDesignDetailedDesign::wishVerifyStateMachineDiagram"),
    ("details/wish_verify_flow/assets/04_requirements", "requirements", "XDesignDetailedDesign::wishVerifyRequirementsDiagram"),
    ("details/wish_verify_flow/assets/04_parametric", "parametric", "XDesignDetailedDesign::wishVerifyParametricDiagram"),
    ("details/wish_verify_flow/assets/04_use_cases", "use_cases", "XDesignDetailedDesign::wishVerifyUseCaseDiagram"),
    ("details/wish_verify_flow/assets/05_modules", "packages", "XDesignRustUml::wishVerifyRustModules"),
    ("details/wish_verify_flow/assets/05_class_structure", "block_definition", "XDesignRustUml::wishVerifyRustTypes"),
    ("prj_manage/assets/09_file_plan", "file_plan", "XDesignDetailedDesign::wishVerifyFlowFilePlanView"),
]


def sources() -> list[Path]:
    return sorted(MODEL_DIR.glob("*.sysml")) + sorted(WISHES_DIR.glob("*.sysml"))


def run_compiler(arguments: list[str]) -> subprocess.CompletedProcess:
    environment = {**os.environ, "TMPDIR": str(TMP), "TMP": str(TMP), "TEMP": str(TMP)}
    return subprocess.run(
        [str(COMPILER), "--color", "never", *arguments],
        capture_output=True, text=True, env=environment,
    )


def render_native(name: str, kind: str, target: str) -> tuple[bool, str]:
    """Render definitions, internal structure or a sequence with the Rust compiler."""
    arguments = ["diagram"]
    if kind == "internal":
        arguments.append(f"--internal={target}")
    elif kind == "sequence":
        arguments.append(f"--sequence={target}")
    if kind == "definitions" and target:
        # Scoped definitions: draw only the named file's definitions, with the
        # rest of the model loaded so names resolve without being drawn.
        arguments.extend(["--library", str(MODEL_DIR), "--library", str(WISHES_DIR), target])
    else:
        arguments.append(str(MODEL_DIR))
        if kind == "sequence":
            arguments.append(str(WISHES_DIR))
    completed = run_compiler(arguments)
    if completed.returncode or not completed.stdout.startswith("<svg"):
        return False, completed.stderr.strip() or f"compiler exit {completed.returncode}"
    figure = DOCS / f"{name}.svg"
    figure.parent.mkdir(parents=True, exist_ok=True)
    figure.write_text(completed.stdout, encoding="utf-8")
    return True, completed.stderr.strip().splitlines()[0] if completed.stderr.strip() else ""


@lru_cache(maxsize=1)
def exported_model() -> tuple[list, dict[str, str]]:
    """Retain and reuse one compiler export and its view syntax evidence."""
    guards: dict[str, str] = {}
    for path in sources():
        completed = run_compiler(["--format", "json", "parse", str(path), "--tree"])
        syntax = LOGS / "syntax" / path.relative_to(REPO)
        syntax.parent.mkdir(parents=True, exist_ok=True)
        syntax.with_suffix(".parse.json").write_text(completed.stdout, encoding="utf-8")
        syntax.with_suffix(".tree.txt").write_text(completed.stderr, encoding="utf-8")
        if completed.returncode:
            raise ValueError(f"Compiler parse failed for {path.relative_to(REPO)}; see {syntax.parent}.")
        guards.update(unsupported_view_syntax(completed.stderr, path.read_text(encoding="utf-8")))
        if "*" in guards:
            raise ValueError(guards["*"])
    completed = run_compiler(["--format", "json", "export", str(MODEL_DIR), str(WISHES_DIR)])
    (LOGS / "export.json").write_text(completed.stdout, encoding="utf-8")
    (LOGS / "export.stderr.log").write_text(completed.stderr, encoding="utf-8")
    if completed.returncode:
        raise ValueError(completed.stderr.strip() or f"export exit {completed.returncode}")
    return json.loads(completed.stdout), guards


def render_declared(name: str, target: str, kind="declared") -> tuple[bool, str]:
    """Project a view the model declares, from the compiler's semantic export."""
    exported, guards = exported_model()
    if kind == "declared_sequence":
        kind = "sequence"
    svg, declared = (render_file_plan(exported, guards, target) if kind == "file_plan"
                     else render_flow_relationships(exported) if kind == "flow_relationships"
                     else render_vmodel_overview(exported) if kind == "vmodel_overview"
                     else render_documentation_view(exported, guards, target, kind) if kind in TITLES
                     else DeclaredViews(exported, guards).render(target))
    figure = DOCS / f"{name}.svg"
    figure.parent.mkdir(parents=True, exist_ok=True)
    figure.write_text(svg, encoding="utf-8")
    label = "model projection" if kind in {"vmodel_overview", "flow_relationships"} else "declared view"
    return True, f"{label} {declared['name']} ({declared['rendering']})"


def main() -> int:
    requested = set(sys.argv[1:])
    DOCS.mkdir(parents=True, exist_ok=True)
    LOGS.mkdir(parents=True, exist_ok=True)
    TMP.mkdir(parents=True, exist_ok=True)
    failed = 0
    for name, kind, target in FIGURES:
        if requested and name not in requested:
            continue
        try:
            ok, note = (render_declared(name, target, kind) if kind != "sequence" and kind in {"declared", "declared_sequence", "file_plan", "vmodel_overview", "flow_relationships", *TITLES}
                        else render_native(name, kind, target))
        except (OSError, ValueError) as error:
            ok, note = False, str(error)
        report = LOGS / f"{name}.log"
        report.parent.mkdir(parents=True, exist_ok=True)
        report.write_text(note + "\n", encoding="utf-8")
        print(f"{'ok  ' if ok else 'FAIL'} {name}.svg  {note}")
        failed += 0 if ok else 1
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
