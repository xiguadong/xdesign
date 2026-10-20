"""Relationship map for Flow definitions and xdesign's selectable usages.

The pinned compiler export supplies the definition specializations, feature
types, subsets, redefinition and default selection. Coordinates only arrange
those native relationships so the two different meanings of `:>` are visible.
"""

from __future__ import annotations

from html import escape

from .renderer import Projection, _ids, _name, _one


FLOW = "XDesignArchitecture::Flow"
XDESIGN = "XDesignComposition::xdesign"
ORIGIN = "XDesignArchitecture::XDesign::'flow'"
AVAILABLE = "XDesignArchitecture::XDesign::availableFlows"
SELECTED = "XDesignComposition::xdesign::selectedFlow"
WORKFLOW_FEATURE = "XDesignWorkflow::Workflow::'flow'"
WORKFLOW_SELECTED = "XDesignWorkflow::Workflow::selectedFlow"

# This layout includes every current descendant of Flow and every current
# portfolio usage. A changed family fails rendering until the figure is reviewed.
DEFINITION_ORDER = (
    "Flow", "VModel", "SimpleFlow", "WorkflowFlow",
    "DesignedSimpleFlow", "NestedFlowExample", "WishVerifyFlow", "DesignFlow",
    "DesignedWishVerifyFlow",
)
ROWS = {
    "Flow": 145,
    "VModel": 245,
    "SimpleFlow": 330,
    "WorkflowFlow": 415,
    "DesignedSimpleFlow": 500,
    "NestedFlowExample": 585,
    "WishVerifyFlow": 670,
    "DesignFlow": 755,
    "DesignedWishVerifyFlow": 840,
}
WIDTH = 1140
HEIGHT = 1055
CARD_HEIGHT = 54


def _single_type(model: Projection, feature: str) -> str:
    types = [key for key in _ids(model.index[feature].get("type")) if key in model.local]
    if len(types) != 1:
        raise ValueError(f"Expected one local type for {model.index[feature].get('qualifiedName')}.")
    return types[0]


def _relation(model: Projection, feature: str, kind: str, general: str) -> str:
    matches = []
    for key in _ids(model.index[feature].get("ownedSpecialization")):
        item = model.index.get(key, {})
        if item.get("@type") == kind and _one(item.get("general")) == general:
            matches.append(key)
    if len(matches) != 1:
        raise ValueError(
            f"Expected one {kind} from {model.index[feature].get('qualifiedName')} "
            f"to {model.index[general].get('qualifiedName')}."
        )
    return matches[0]


def _reference(model: Projection, feature: str) -> str:
    targets = {
        target for key in model.descendants(feature)
        if model.index[key]["@type"] == "FeatureReferenceExpression"
        for target in _ids(model.index[key].get("referent"))
    }
    if len(targets) != 1:
        raise ValueError(f"Expected one reference from {model.index[feature].get('qualifiedName')}.")
    return targets.pop()


def _model(raw: list) -> tuple[Projection, dict, dict, str, dict]:
    model = Projection(raw)
    root = model.target(FLOW)
    system = model.target(XDESIGN)
    origin = model.target(ORIGIN)
    available = model.target(AVAILABLE)
    selected_feature = model.target(SELECTED)
    workflow_feature = model.target(WORKFLOW_FEATURE)
    workflow_selected = model.target(WORKFLOW_SELECTED)

    parents = {}
    inheritance = {}
    for relation in model.local.values():
        if relation["@type"] != "Subclassification":
            continue
        specific = _one(relation.get("specific"))
        general = _one(relation.get("general"))
        if specific in model.local and general in model.local:
            if model.index[specific]["@type"] == "PartDefinition":
                parents[specific] = general
                inheritance[specific] = relation["@id"]

    descendants = {root}
    while True:
        added = {child for child, parent in parents.items() if parent in descendants}
        if added <= descendants:
            break
        descendants |= added
    definitions = {_name(model.index[key]): key for key in descendants}
    if set(definitions) != set(DEFINITION_ORDER):
        raise ValueError(
            "Flow definition family changed; review the relationship figure: "
            f"found {sorted(definitions)}."
        )
    if len(definitions) != len(descendants):
        raise ValueError("Flow definitions have duplicate short names; use qualified labels.")

    usages = {}
    for key in model.children[system]:
        if model.index[key]["@type"] != "PartUsage":
            continue
        subsets_origin = [item for item in _ids(model.index[key].get("ownedSubsetting"))
                          if model.index.get(item, {}).get("@type") == "Subsetting"
                          and _one(model.index[item].get("subsettedFeature")) == origin]
        if not subsets_origin:
            continue
        origin_relation = _relation(model, key, "Subsetting", origin)
        portfolio_relation = _relation(model, key, "Subsetting", available)
        typed = _single_type(model, key)
        _relation(model, key, "FeatureTyping", typed)
        if typed not in descendants:
            raise ValueError(f"A selectable usage is typed outside the Flow family: {key}.")
        usages[typed] = (key, origin_relation, portfolio_relation)
    selectable = descendants - {root, definitions["SimpleFlow"]}
    if set(usages) != selectable:
        raise ValueError("xdesign portfolio differs from the seven shown usages; review the figure.")

    default = _reference(model, selected_feature)
    if default not in {usage[0] for usage in usages.values()}:
        raise ValueError("xdesign selectedFlow does not reference a portfolio usage.")

    wrapper_type = _single_type(model, workflow_feature)
    _relation(model, workflow_feature, "Redefinition", origin)
    if wrapper_type != definitions["WorkflowFlow"]:
        raise ValueError("Workflow.'flow' no longer has the WorkflowFlow type.")
    if _reference(model, workflow_selected) != workflow_feature:
        raise ValueError("Workflow.selectedFlow no longer refers to Workflow.'flow'.")
    return model, definitions, usages, default, {"origin": origin, "available": available,
                                                  "system": system, "wrapper": workflow_feature,
                                                  "selected": selected_feature,
                                                  "inheritance": inheritance,
                                                  "parents": parents}


def render_flow_relationships(raw: list) -> tuple[str, dict]:
    """Render the compiler-resolved definition tree and one xdesign portfolio."""
    model, definitions, usages, default, links = _model(raw)
    origin = links["origin"]
    by_id = {key: name for name, key in definitions.items()}
    default_name = _name(model.index[default])

    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-labelledby="figure-title figure-desc">',
        '<title id="figure-title">Flow definitions and selectable flows owned by xdesign</title>',
        '<desc id="figure-desc">On the left, three definition branches inherit from Flow. On the right, seven usages belong to the xdesign portfolio and each has one definition type. A separate Workflow configuration feature is identified below.</desc>',
        '<defs><marker id="type-arrow" markerWidth="11" markerHeight="11" refX="10" refY="5.5" orient="auto"><path d="M 1 1 L 10 5.5 L 1 10" fill="none" stroke="#476e9a" stroke-width="1.8"/></marker></defs>',
        '<style>text{font-family:Arial,Helvetica,sans-serif;fill:#1b2e44}.title{font-size:26px;font-weight:700}.subtitle{font-size:16px;fill:#52657a}.lane{font-size:17px;font-weight:700;fill:#365e80}.hint{font-size:14px;fill:#52657a}.definition{fill:#f2f7fc;stroke:#527496;stroke-width:1.5}.base{fill:#e2eef9;stroke:#315b81;stroke-width:2}.usage{fill:#f1f8f1;stroke:#609268;stroke-width:1.5}.default{fill:#e3f3e4;stroke:#39804b;stroke-width:2}.card-text{font-size:18px;font-weight:600}.secondary{font-size:14px;fill:#4d6278}.inherit{fill:none;stroke:#6d8297;stroke-width:2}.portfolio{fill:none;stroke:#6c9975;stroke-width:2.1}.typing{fill:none;stroke:#476e9a;stroke-width:1.8;marker-end:url(#type-arrow)}.legend{font-size:14px;fill:#52657a}.note{fill:#fbfbf6;stroke:#aaa991;stroke-width:1.2}.note-title{font-size:17px;font-weight:700}.note-text{font-size:15px;fill:#3f5060}</style>',
        '<rect width="100%" height="100%" fill="#fff"/>',
        '<text x="54" y="47" class="title">One Flow family; selectable plans belong to xdesign</text>',
        '<text x="54" y="75" class="subtitle">Definitions describe the plan shape. Usages are the plans a wish can select.</text>',
        '<text x="54" y="116" class="lane">FLOW DEFINITIONS</text>',
        '<text x="690" y="116" class="lane">XDESIGN PORTFOLIO USAGES</text>',
        '<text x="54" y="927" class="legend">Gray branches: :&gt; definition inheritance</text>',
        '<text x="385" y="927" class="legend">Blue arrows: usage typed by definition</text>',
        '<text x="746" y="927" class="legend">Green branches: subsets availableFlows and :&gt; xdesign.\'flow\'</text>',
    ]

    # The native Subclassification relations determine which branch connects
    # which definition. The fixed coordinates keep peer types on one row.
    coords = {}
    for name in DEFINITION_ORDER:
        x = 64 if name == "Flow" else 260 if name in {"VModel", "SimpleFlow", "WishVerifyFlow"} else 350
        width = 160 if name == "Flow" else 200 if x == 260 else 250
        coords[name] = (x, ROWS[name], width)

    # Root-to-branch connections share a left trunk; descendant branches sit
    # immediately before their child cards. Every drawn edge is verified above.
    for name in DEFINITION_ORDER[1:]:
        child = definitions[name]
        parent = links["parents"].get(child)
        if parent not in by_id:
            raise ValueError(f"Flow definition {name} has an unshown parent.")
        parent_name = by_id[parent]
        px, py, pw = coords[parent_name]
        cx, cy, _ = coords[name]
        if parent_name == "Flow":
            path = f"M {px + pw} {py + CARD_HEIGHT // 2} H 241 V {cy + CARD_HEIGHT // 2} H {cx - 3}"
        elif parent_name == "SimpleFlow":
            path = f"M {px + 100} {py + CARD_HEIGHT} H 331 V {cy + CARD_HEIGHT // 2} H {cx - 3}"
        else:
            path = f"M {px + 100} {py + CARD_HEIGHT} H 331 V {cy + CARD_HEIGHT // 2} H {cx - 3}"
        relation = links["inheritance"][child]
        svg.append(f'<path class="inherit" d="{path}" data-native-id="{relation}"><title>{escape(model.index[child]["qualifiedName"])} specializes {escape(model.index[parent]["qualifiedName"])}</title></path>')

    # The xdesign backbone starts from the inherited abstract Flow feature.
    svg.append('<path class="portfolio" d="M 1050 172 H 1081 V 867"/>')
    for name in DEFINITION_ORDER:
        key = definitions[name]
        x, y, width = coords[name]
        element = model.index[key]
        klass = "base" if name == "Flow" else "definition"
        svg.extend([
            f'<g data-native-id="{key}"><title>{escape(element["qualifiedName"])}&#10;Native ID: {key}</title>',
            f'<rect class="{klass}" x="{x}" y="{y}" width="{width}" height="{CARD_HEIGHT}" rx="9"/>',
            f'<text class="card-text" x="{x + 15}" y="{y + 34}">{escape(name)}</text>',
            '</g>',
        ])
        if key not in usages:
            continue
        usage, origin_relation, portfolio_relation = usages[key]
        use = model.index[usage]
        uy = y
        ux, uw = 690, 360
        label = "xdesign." + _name(use)
        selected = usage == default
        svg.extend([
            f'<path class="typing" d="M {x + width + 8} {y + 27} H {ux - 9}" data-native-id="{_relation(model, usage, "FeatureTyping", key)}"><title>{escape(use["qualifiedName"])} is typed by {escape(element["qualifiedName"])}</title></path>',
            f'<path class="portfolio" d="M {ux + uw} {uy + 27} H 1081" data-native-id="{origin_relation}" data-portfolio-subsetting-id="{portfolio_relation}"><title>{escape(use["qualifiedName"])} subsets xdesign.\'flow\' and availableFlows</title></path>',
            f'<g data-native-id="{usage}"><title>{escape(use["qualifiedName"])}&#10;Type: {escape(element["qualifiedName"])}&#10;Native ID: {usage}</title>',
            f'<rect class="{"default" if selected else "usage"}" x="{ux}" y="{uy}" width="{uw}" height="{CARD_HEIGHT}" rx="9"/>',
            f'<text class="card-text" x="{ux + 15}" y="{uy + 33}">{escape(label)}</text>',
            *([f'<text class="secondary" x="{ux + uw - 16}" y="{uy + 33}" text-anchor="end">default</text>'] if selected else []),
            '</g>',
        ])

    origin_element = model.index[origin]
    svg.extend([
        f'<g data-native-id="{origin}"><title>{escape(origin_element["qualifiedName"])}&#10;Abstract Flow origin inherited by xdesign</title>',
        '<rect class="base" x="690" y="145" width="360" height="54" rx="9"/>',
        '<text class="card-text" x="705" y="179">xdesign.\'flow\' : Flow</text>',
        '</g>',
        f'<text x="690" y="224" class="hint" data-native-id="{links["selected"]}">selectedFlow → xdesign.{escape(default_name)} (default)</text>',
        '<rect class="note" x="54" y="960" width="1032" height="75" rx="10"/>',
        '<text x="72" y="987" class="note-title">Why Workflow.\'flow\' also appears in the model</text>',
        f'<text x="72" y="1012" class="note-text" data-native-id="{links["wrapper"]}">It redefines XDesign.\'flow\' in the separate Workflow configuration and is typed WorkflowFlow; xdesign.workflowFlow is the portfolio usage.</text>',
        '</svg>',
    ])
    return "\n".join(svg) + "\n", {
        "name": XDESIGN,
        "rendering": "native Flow subclassification, feature typing, and portfolio subsetting projection",
        "definitions": [definitions[name] for name in DEFINITION_ORDER],
        "usages": [usages[definitions[name]][0] for name in DEFINITION_ORDER if definitions[name] in usages],
        "default": default,
        "origin": origin,
        "workflowConfigurationFeature": links["wrapper"],
    }
