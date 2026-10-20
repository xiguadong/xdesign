"""Compact wish-to-delivery projection of the model's default VModel.

The pinned compiler export supplies every stage, successor, exit checkpoint,
checkpoint kind, criterion, and verification correspondence. The fixed layout
only chooses where those native elements appear on the page.
"""

from __future__ import annotations

from html import escape
import re

from .renderer import Projection, _ids, _name


FLOW = "XDesignArchitecture::VModel"
STAGE = "XDesignArchitecture::Stage"
CHECKPOINT = "XDesignArchitecture::Checkpoint"
WIDTH = 1280
CARD_WIDTH = 370
CARD_HEIGHT = 104
LEFT_X = 72
RIGHT_X = WIDTH - LEFT_X - CARD_WIDTH
TOP_Y = 150
ROW_STEP = 157


def _words(name: str) -> str:
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", " ", name).capitalize()


def _child(projection: Projection, owner: str, name: str) -> str:
    matching = [key for key in projection.children[owner]
                if _name(projection.index[key]) == name]
    if len(matching) != 1:
        raise ValueError(f"Expected one {name} feature on {projection.index[owner].get('qualifiedName')}.")
    return matching[0]


def _references(projection: Projection, owner: str, feature: str, allowed: set[str]) -> list[str]:
    child = _child(projection, owner, feature)
    return list(dict.fromkeys(
        target for key in projection.descendants(child)
        if projection.index[key]["@type"] == "FeatureReferenceExpression"
        for target in _ids(projection.index[key].get("referent")) if target in allowed
    ))


def _literal(projection: Projection, owner: str, feature: str, kind: str):
    child = _child(projection, owner, feature)
    values = [projection.index[key]["value"] for key in projection.descendants(child)
              if projection.index[key]["@type"] == kind]
    if len(values) != 1:
        raise ValueError(f"Expected one {kind} value for {projection.index[owner].get('qualifiedName')}::{feature}.")
    return values[0]


def _model(raw: list) -> tuple[Projection, list[dict]]:
    projection = Projection(raw)
    flow = projection.target(FLOW)
    stage_type = projection.target(STAGE)
    checkpoint_type = projection.target(CHECKPOINT)
    if not all((flow, stage_type, checkpoint_type)):
        raise ValueError("The compiler export lacks the VModel or its base types.")

    owned = [key for key in projection.children[flow]
             if projection.index[key]["@type"] == "PartUsage"]
    stages = {key for key in owned if stage_type in _ids(projection.index[key].get("type"))}
    checkpoints = {key for key in owned if checkpoint_type in _ids(projection.index[key].get("type"))}
    if not stages or not checkpoints:
        raise ValueError("The VModel has no exported stages or checkpoints.")

    initial = _references(projection, flow, "initialStage", stages)
    if len(initial) != 1:
        raise ValueError("The VModel must have one initial stage for this overview.")
    ordered: list[str] = []
    current = initial[0]
    while current not in ordered:
        ordered.append(current)
        successors = _references(projection, current, "nextStages", stages)
        if len(successors) > 1:
            raise ValueError("This overview needs a branching layout for the changed VModel.")
        if not successors:
            break
        current = successors[0]
    if current in ordered[:-1] or set(ordered) != stages:
        raise ValueError("The VModel successor chain cycles or omits a stage.")

    result: list[dict] = []
    for stage in ordered:
        gates = _references(projection, stage, "exitCheckpoints", checkpoints)
        if len(gates) != 1:
            raise ValueError("This overview expects one exit checkpoint per VModel stage.")
        gate = gates[0]
        sources = _references(projection, gate, "sourceStage", stages)
        if sources != [stage]:
            raise ValueError("A VModel checkpoint has a source stage different from its exit stage.")
        kind_feature = _child(projection, gate, "kind")
        kinds = [projection.index[target].get("name") for key in projection.descendants(kind_feature)
                 if projection.index[key]["@type"] == "FeatureReferenceExpression"
                 for target in _ids(projection.index[key].get("referent"))
                 if projection.index.get(target, {}).get("name") in {"syntactic", "semantic"}]
        if len(kinds) != 1:
            raise ValueError("A VModel checkpoint lacks one exported kind.")
        verifies = _references(projection, stage, "verifiesStage", stages) if any(
            _name(projection.index[key]) == "verifiesStage" for key in projection.children[stage]
        ) else []
        if len(verifies) > 1:
            raise ValueError("A VModel stage verifies more than one stage; update the overview layout.")
        result.append({
            "id": stage,
            "name": _name(projection.index[stage]),
            "qualified": projection.index[stage]["qualifiedName"],
            "gate": _name(projection.index[gate]),
            "gate_id": gate,
            "kind": kinds[0],
            "criterion": _literal(projection, gate, "criteria", "LiteralString"),
            "verifies": verifies[0] if verifies else None,
        })
    return projection, result


def render_vmodel_overview(raw: list) -> tuple[str, dict]:
    """Return a readable SVG and source summary from compiler-exported model IDs."""
    _projection, stages = _model(raw)
    first_verify = next((index for index, stage in enumerate(stages) if stage["verifies"]), None)
    if first_verify is None or first_verify < 2 or first_verify >= len(stages) - 1:
        raise ValueError("The VModel no longer has the expected development/verification shape.")
    implementation = stages[first_verify - 1]
    development = stages[:first_verify - 1]
    verification = stages[first_verify:-1]
    delivery = stages[-1]
    by_id = {stage["id"]: stage for stage in stages}
    if _name(_projection.index[implementation["id"]]) != "implementation" or delivery["name"] != "delivery":
        raise ValueError("The VModel's implementation or delivery stage changed; review this projection.")
    if len(development) != len(verification) + 1:
        raise ValueError("The VModel's verification correspondence needs a new layout.")
    if {stage["verifies"] for stage in verification} != {stage["id"] for stage in development[1:]}:
        raise ValueError("The VModel's verification pairs need a new layout.")

    positions = {stage["id"]: (LEFT_X, TOP_Y + index * ROW_STEP)
                 for index, stage in enumerate(development)}
    for stage in verification:
        matching = by_id[stage["verifies"]]
        positions[stage["id"]] = (RIGHT_X, positions[matching["id"]][1])
    positions[delivery["id"]] = (RIGHT_X, TOP_Y)
    implementation_y = TOP_Y + len(development) * ROW_STEP + 16
    positions[implementation["id"]] = ((WIDTH - CARD_WIDTH) // 2, implementation_y)
    height = implementation_y + CARD_HEIGHT + 114

    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{height}" viewBox="0 0 {WIDTH} {height}" role="img" aria-labelledby="figure-title figure-desc">',
        '<title id="figure-title">Wish to delivery: default VModel stage and checkpoint overview</title>',
        '<desc id="figure-desc">Follow the solid arrows down the left side to implementation, then up the right side to delivery. Dashed cross links identify which development stage each verification stage checks.</desc>',
        '<defs><marker id="flow-arrow" markerWidth="10" markerHeight="10" refX="8" refY="5" orient="auto"><path d="M 1 1 L 9 5 L 1 9" fill="none" stroke="#365e80" stroke-width="1.8"/></marker><marker id="verify-arrow" markerWidth="9" markerHeight="9" refX="7" refY="4.5" orient="auto"><path d="M 1 1 L 8 4.5 L 1 8" fill="none" stroke="#87909b" stroke-width="1.5"/></marker></defs>',
        '<style>text{font-family:Arial,Helvetica,sans-serif;fill:#172b43}.title{font-size:26px;font-weight:700}.subtitle{font-size:15px;fill:#52657a}.lane{font-size:16px;font-weight:700;fill:#365e80;letter-spacing:.05em}.stage{fill:#fff;stroke:#647a91;stroke-width:1.5}.stage.delivery{fill:#eef7ef;stroke:#599a68}.stage.implementation{fill:#fff7e6;stroke:#b88e44}.number{font-size:13px;font-weight:700;fill:#52657a}.name{font-size:21px;font-weight:700}.gate{font-size:15px;fill:#334b64}.kind{font-size:13px;fill:#52657a}.flow{fill:none;stroke:#365e80;stroke-width:2.5;marker-end:url(#flow-arrow)}.pair{fill:none;stroke:#87909b;stroke-width:1.7;stroke-dasharray:6 6;marker-end:url(#verify-arrow)}.pair-label{font-size:12px;fill:#667582}.footer{font-size:14px;fill:#52657a}</style>',
        '<rect width="100%" height="100%" fill="#fff"/>',
        '<text x="72" y="53" class="title">From wish to implementation and delivery</text>',
        '<text x="72" y="80" class="subtitle">Default VModel definition · solid arrows: progression after a passed exit gate · dashed arrows: verifies stage</text>',
        f'<text x="{LEFT_X}" y="125" class="lane">DEFINITION AND BUILD ↓</text>',
        f'<text x="{RIGHT_X}" y="125" class="lane">VERIFICATION AND DELIVERY ↑</text>',
    ]

    # Progression edges are native nextStages references, traversed above.
    for source, target in zip(stages, stages[1:]):
        sx, sy = positions[source["id"]]
        tx, ty = positions[target["id"]]
        if sx == tx and sy < ty:
            x = sx + CARD_WIDTH // 2
            path = f"M {x} {sy + CARD_HEIGHT + 5} L {x} {ty - 8}"
        elif sx == tx and sy > ty:
            x = sx + CARD_WIDTH // 2
            path = f"M {x} {sy - 5} L {x} {ty + CARD_HEIGHT + 8}"
        elif sx < tx and sy < ty:
            path = (f"M {sx + CARD_WIDTH - 45} {sy + CARD_HEIGHT + 5} "
                    f"L {tx + 46} {ty - 8}")
        elif sx < tx and sy > ty:
            path = (f"M {sx + CARD_WIDTH - 46} {sy - 8} "
                    f"L {tx + 46} {ty + CARD_HEIGHT + 8}")
        else:
            path = (f"M {sx + 46} {sy - 5} "
                    f"L {tx + CARD_WIDTH - 46} {ty + CARD_HEIGHT + 8}")
        svg.append(f'<path class="flow" d="{path}"><title>{escape(source["qualified"])} → {escape(target["qualified"])}</title></path>')

    # Each dashed arrow is the native verifiesStage reference from a right lane
    # verification stage to the corresponding development stage on the left.
    for stage in verification:
        rx, ry = positions[stage["id"]]
        lx, ly = positions[stage["verifies"]]
        y = ry + CARD_HEIGHT // 2
        svg.append(f'<path class="pair" d="M {rx - 8} {y} L {lx + CARD_WIDTH + 8} {y}"><title>{escape(stage["qualified"])} verifies {escape(by_id[stage["verifies"]]["qualified"])}</title></path>')
        svg.append(f'<text class="pair-label" x="{WIDTH // 2}" y="{y - 10}" text-anchor="middle">verifies</text>')

    for number, stage in enumerate(stages, start=1):
        x, y = positions[stage["id"]]
        role = "delivery" if stage is delivery else "implementation" if stage is implementation else ""
        svg.extend([
            f'<g id="stage-{stage["id"]}" aria-label="{escape(stage["name"])}; exit checkpoint {escape(stage["gate"])}">',
            f'<title>{escape(stage["qualified"])}\nExit: {escape(stage["gate"])} ({escape(stage["kind"])})\n{escape(stage["criterion"])}</title>',
            f'<rect class="stage {role}" x="{x}" y="{y}" width="{CARD_WIDTH}" height="{CARD_HEIGHT}" rx="11"/>',
            f'<text class="number" x="{x + 17}" y="{y + 22}">{number:02d}</text>',
            f'<text class="name" x="{x + 17}" y="{y + 50}">{escape(_words(stage["name"]))}</text>',
            f'<text class="gate" x="{x + 17}" y="{y + 76}">Exit: {escape(stage["gate"])}</text>',
            f'<text class="kind" x="{x + 17}" y="{y + 95}">{escape(stage["kind"])} · {"fixed checker" if stage["kind"] == "syntactic" else "permitted judge"}</text>',
            '</g>',
        ])
    svg.append(f'<text class="footer" x="72" y="{height - 45}">Definition view. Each wish records its own visits, assessments, and evidence; this figure is not a completed run or a calendar plan.</text>')
    svg.append('</svg>')
    return "\n".join(svg) + "\n", {
        "name": FLOW,
        "rendering": "native VModel stage/checkpoint projection",
        "stageIds": [stage["id"] for stage in stages],
        "checkpointIds": [stage["gate_id"] for stage in stages],
    }
