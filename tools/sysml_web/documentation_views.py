"""Documentation projections of the nine traditional SysML diagram questions.

The declared view's exposed native IDs select the content. This adapter reads
compiler interchange only; it neither parses SysML nor invents engineering
relationships. Sequence messages remain the pinned compiler renderer's job.
"""

from __future__ import annotations

from textwrap import wrap
from html import escape
import re

from .declared_views import DeclaredViews
from .renderer import CONNECTORS, Edge, Projection, _ids, _name, _named, _one, _svg_id
from .diagram_layout import DiagramNode, DiagramPort, DiagramEdge, DiagramGroup, render_diagram


TITLES = {
    "requirements": "Requirement diagram",
    "packages": "Package diagram",
    "block_definition": "Block definition diagram",
    "internal_block": "Internal block diagram",
    "parametric": "Parametric diagram",
    "use_cases": "Use case diagram",
    "activity": "Activity diagram",
    "state_machine": "State machine diagram",
    "sequence": "Sequence diagram",
}


class DocumentationProjection(Projection):
    def __init__(self, raw, kind):
        super().__init__(raw)
        self.kind = kind
        self.presentation_annotations = {}
        for key, element in self.local.items():
            if element["@type"] == "MetadataUsage":
                for target in _ids(element.get("annotatedElement")):
                    self.presentation_annotations.setdefault(target, []).append(key)

    def presentation(self, key):
        """Read model-owned display text; it never replaces semantic identity."""
        result = {}
        for child in dict.fromkeys(self.presentation_annotations.get(key, []) + self.children[key]):
            annotation = self.index[child]
            if annotation["@type"] != "MetadataUsage" or not any(
                self.index.get(kind, {}).get("qualifiedName") == "XDesignCommon::DiagramPresentation"
                for kind in _ids(annotation.get("type"))
            ):
                continue
            for feature in self.children[child]:
                element = self.index[feature]
                if element["@type"] in {"AttributeUsage", "ReferenceUsage"} and _name(element) in {"label", "summary", "question"}:
                    values = [self.index[item].get("value") for item in self.descendants(feature)
                              if self.index[item]["@type"] == "LiteralString"]
                    if values:
                        result[_name(element)] = values[0]
        return result

    def caption(self, key):
        own = self.presentation(key)
        if own.get("label"):
            return own["label"]
        return _words(_name(self.index[key]))

    def summary(self, key):
        own = self.presentation(key)
        if own.get("summary"):
            return own["summary"]
        for typed in _ids(self.index[key].get("type")):
            inherited = self.presentation(typed)
            if inherited.get("summary"):
                return inherited["summary"]
        return ""

    def tooltip(self, key):
        element = self.index[key]
        return "\n".join([element.get("qualifiedName") or _name(element), key,
                           *super().compartments(key), *self.text(key)])

    def text(self, key, kind="TextualRepresentation"):
        """Retain outer expression evidence once, in native ownership order."""
        result, pending = [], [key]
        while pending:
            current = pending.pop()
            element = self.index[current]
            if element["@type"] == kind and element.get("body"):
                result.append(element["body"])
                continue
            representations = [self.index[item] for item in _ids(element.get("textualRepresentation"))
                               if self.index.get(item, {}).get("@type") == kind and self.index[item].get("body")]
            if representations:
                result.extend(item["body"] for item in representations)
                # The full expression already contains its nested lambda and
                # operands; rendering those again would repeat fragments.
                continue
            pending.extend(reversed(self.children[current]))
        return result

    def compartments(self, key):
        element = self.index[key]
        kind = element["@type"]
        if self.kind == "packages":
            members = [self.index[child] for child in self.children[key]
                       if _named(self.index[child]) and not self.index[child]["@type"].endswith("Membership")]
            return [f'{child["@type"]}: {_name(child)}' for child in members[:6]] + (
                [f"… {len(members) - 6} more direct declarations"] if len(members) > 6 else [])
        lines = []
        if kind in {"RequirementDefinition", "RequirementUsage", "VerificationCaseDefinition", "VerificationCaseUsage"}:
            # Documentation is the requirement/case description; metadata values
            # and inherited library requirements are deliberately not expanded.
            for child in self.children[key]:
                if self.index[child]["@type"] == "Documentation":
                    lines.extend(wrap(" ".join(self.index[child].get("body", "").split()), 48))
        elif self.kind in {"activity", "state_machine", "use_cases"}:
            types = self.type_names(element)
            if types:
                lines.append("type: " + ", ".join(types))
        elif self.kind == "requirements":
            types = self.type_names(element)
            lines = ["type: " + ", ".join(types)] if types else []
        else:
            lines = super().compartments(key)
        if self.kind == "parametric":
            types = _ids(element.get("type"))
            expression_owners = [key] + [item for item in types if item in self.local]
            expressions = []
            for owner in expression_owners:
                expressions.extend(self.text(owner))
            for expression in dict.fromkeys(expressions):
                lines.extend(wrap(" ".join(expression.split()), 48))
        return lines

    def ancestor(self, key, visible):
        seen = set()
        while key and key not in seen:
            if key in visible:
                return key
            seen.add(key)
            key = _one(self.index.get(key, {}).get("owner"))
        return None

    def referenced(self, key, visible):
        """Resolve a verification/role proxy through its native specialization."""
        element = self.index.get(key, {})
        candidates = []
        for relation in _ids(element.get("ownedReferenceSubsetting")) + _ids(element.get("ownedSubsetting")):
            entry = self.index.get(relation, {})
            candidates.extend(_ids(entry.get("referencedFeature") or entry.get("subsettedFeature")))
        return next((item for item in candidates if item in visible), None)

    def end(self, key, visible):
        direct = self.endpoint(key, visible)
        if direct:
            return direct
        element = self.index.get(key, {})
        chain = _ids(element.get("chainingFeature"))
        if chain:
            ancestor = self.ancestor(chain[0], visible)
            if ancestor:
                return ancestor, ".".join(_name(self.index[item]) for item in chain if item in self.index)
        ancestor = self.ancestor(key, visible)
        return (ancestor, _name(element)) if ancestor else None

    def relationships(self, visible, scope, allowed=None):
        edges, notes = [], []
        for relation in self.local.values():
            kind, native_id = relation["@type"], relation["@id"]
            if allowed is not None and kind not in allowed:
                continue
            if kind == "SatisfyRequirementUsage":
                source = self.ancestor(_one(relation.get("satisfyingFeature")), visible)
                target = self.ancestor(_one(relation.get("satisfiedRequirement")), visible)
                if source and target:
                    edges.append(Edge(source, target, "dependency", "«satisfy»", native_id))
            elif kind == "RequirementVerificationMembership":
                source = self.ancestor(_one(relation.get("owner")), visible)
                target = self.referenced(_one(relation.get("verifiedRequirement") or relation.get("memberElement")), visible)
                if source and target:
                    edges.append(Edge(source, target, "dependency", "«verify»", native_id))
            elif kind in {"NamespaceImport", "MembershipImport"}:
                source = self.ancestor(_one(relation.get("owner")), visible)
                target = _one(relation.get("importedNamespace") or relation.get("importedElement"))
                if kind == "MembershipImport":
                    target = _one(self.index.get(target, {}).get("memberElement")) or target
                target = self.ancestor(target, visible)
                if source and target and source != target:
                    edges.append(Edge(source, target, "dependency", f'«{relation.get("visibility", "public")} import»', native_id))
            elif kind == "Dependency":
                for source in _ids(relation.get("client")):
                    for target in _ids(relation.get("supplier")):
                        if source in visible and target in visible:
                            edges.append(Edge(source, target, "dependency", _name(relation) if _named(relation) else "«dependency»", native_id))
            elif kind in CONNECTORS and native_id in scope:
                endpoints = _ids(relation.get("endFeature") or relation.get("relatedFeature"))
                if kind == "TransitionUsage":
                    endpoints = _ids(relation.get("source")) + _ids(relation.get("target"))
                    if not endpoints:
                        continue  # Its native owned succession supplies the ends.
                ends = [self.end(endpoint, visible) for endpoint in endpoints]
                if len(ends) != 2 or not all(ends):
                    # A native start/end library feature may lie outside a view.
                    # Keep the limitation visible rather than add a synthetic ID.
                    notes.append(f"{kind} {native_id}: an endpoint is outside the selected named elements.")
                    continue
                source, target = ends
                owner = self.index.get(_one(relation.get("owner")), {})
                named = owner if kind == "SuccessionAsUsage" and owner.get("@type") == "TransitionUsage" else relation
                label = _name(named) if _named(named) else ""
                texts = self.text(named["@id"])
                if texts:
                    label += " [" + " ".join(texts[0].split()) + "]"
                binding = kind.startswith("Binding")
                edge_kind = "binding" if binding else "directed" if kind in {
                    "FlowUsage", "SuccessionAsUsage", "Succession", "TransitionUsage", "AllocationUsage"} else "connection"
                if binding:
                    label = source[1] + " = " + target[1]
                edges.append(Edge(source[0], target[0], edge_kind, label, native_id,
                                  source[1] + " → " + target[1]))
        return list(dict.fromkeys(edges)), notes


def _words(value):
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", value).replace("_", " ")
    return value[:1].upper() + value[1:]


def _node(projection, key, *, shape="box", group=None, ports=(), lines=None):
    element = projection.index[key]
    kind = {"PartDefinition": "part def", "PartUsage": "part", "ItemDefinition": "item def",
            "RequirementUsage": "requirement", "RequirementDefinition": "requirement",
            "VerificationCaseDefinition": "verification", "MetadataDefinition": "metadata",
            "UseCaseUsage": "use case", "UseCaseDefinition": "use case", "ConstraintUsage": "constraint",
            "StateUsage": "state", "AttributeUsage": "value", "Package": "package",
            "DecisionNode": ""}.get(element["@type"], "")
    if shape == "actor":
        kind = "actor"
    description = projection.summary(key)
    return DiagramNode(key, projection.caption(key), kind=kind,
                       lines=tuple(wrap(description, 29)) if lines is None and description else tuple(lines or ()),
                       shape=shape, group=group, ports=tuple(ports), tooltip=projection.tooltip(key))


def _edge(projection, edge, **changes):
    label = edge.label
    if edge.kind in {"composition", "reference"}:
        prefix, _, feature = label.partition(" ")
        label = ("owns " if prefix == "owns" else "ref ") + _words(feature).lower()
    if edge.kind == "specialization":
        label = ""
    values = dict(id=edge.native_id, source=edge.source, target=edge.target,
                  label=label, kind=edge.kind, tooltip=edge.detail)
    values.update(changes)
    return DiagramEdge(**values)


def _annotation_relationships(svg, relations):
    """Containment and ports are graphical relationships, not extra connector lines."""
    from xml.etree import ElementTree as ET
    namespace = "http://www.w3.org/2000/svg"
    ET.register_namespace("", namespace)
    root = ET.fromstring(svg)
    for native_id, source, target, meaning in relations:
        group = ET.SubElement(root, f"{{{namespace}}}g", {
            "data-native-id": native_id, "data-source-id": source,
            "data-target-id": target, "data-representation": meaning})
        ET.SubElement(group, f"{{{namespace}}}title").text = meaning
    return ET.tostring(root, encoding="unicode")


def _sequence(projection, timeline, title, question):
    """A sequence has semantic time rows, not a general graph layout."""
    participants = [key for key in projection.children[timeline]
                    if projection.index[key]["@type"] == "PartUsage" and _named(projection.index[key])]
    messages = []
    for key in projection.children[timeline]:
        element = projection.index[key]
        if element["@type"] != "FlowUsage":
            continue
        ends = [projection.end(item, set(participants)) for item in _ids(element.get("endFeature"))]
        if len(ends) != 2 or not all(ends):
            raise ValueError("Sequence message has no complete native participant pair.")
        messages.append((key, ends[0][0], ends[1][0]))
    if not participants or not messages:
        raise ValueError("The sequence view has no native participants or messages.")
    width, row, top = max(720, len(participants) * 130 + 30), 46, 168
    height = top + len(messages) * row + 28
    x = {key: 80 + number * 130 for number, key in enumerate(participants)}
    body = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" data-layout-engine="sequence" aria-labelledby="diagram-title diagram-desc">',
            f'<title id="diagram-title">{escape(title)}</title><desc id="diagram-desc">{escape(question)}</desc>',
            '<rect width="100%" height="100%" fill="#fff"/>',
            '<defs><marker id="seq-arrow" markerWidth="10" markerHeight="8" refX="9" refY="4" orient="auto"><path d="M1 1 L9 4 L1 7" fill="none" stroke="#334e68" stroke-width="1.5"/></marker></defs>',
            '<style>text{font-family:Arial,sans-serif;fill:#243b53}.message{font-size:15px}.head{font-size:16px;font-weight:600}</style>',
            f'<text x="24" y="32" font-size="22" font-weight="700">{escape(title)}</text>',
            f'<text x="24" y="59" font-size="14">{escape(question)}</text>']
    for key in participants:
        cx = x[key]
        body.extend([f'<g id="{_svg_id(key)}" data-native-id="{key}"><title>{escape(projection.tooltip(key))}</title>',
                     f'<rect x="{cx-60}" y="88" width="120" height="46" rx="6" fill="#edf4fb" stroke="#9fb3c8"/>',
                     f'<text x="{cx}" y="116" text-anchor="middle" class="head">{escape(projection.caption(key))}</text>',
                     f'<path d="M{cx} 134 V{height-16}" stroke="#9fb3c8" stroke-dasharray="4 5"/></g>'])
    for number, (key, source, target) in enumerate(messages):
        y = top + number * row
        label = projection.caption(key)
        mid, label_width = (x[source]+x[target])/2, min(width-48, len(label)*7.5+16)
        body.extend([f'<g data-native-id="{key}" data-source-id="{source}" data-target-id="{target}"><title>{escape(projection.tooltip(key))}</title>',
                     f'<path d="M{x[source]} {y} H{x[target]}" stroke="#334e68" stroke-width="1.5" marker-end="url(#seq-arrow)"/>',
                     f'<rect x="{mid-label_width/2}" y="{y-22}" width="{label_width}" height="20" fill="#fff"/>',
                     f'<text x="{mid}" y="{y-7}" text-anchor="middle" class="message">{escape(label)}</text></g>'])
    return "".join(body) + "</svg>"


def render_documentation_view(raw, guards, target, kind):
    """Compose one engineering question before asking Graphviz for its layout."""
    if kind not in TITLES:
        raise ValueError(f"Unsupported documentation diagram kind: {kind}")
    declared = DeclaredViews(raw, guards)
    key = declared.projection.target(target)
    resolved = declared.selection(key)
    projection = DocumentationProjection(raw, kind)
    scope = declared._scope(resolved["exposed"])
    selected = {item for item in scope if item in projection.local and _named(projection.index[item])}
    view = projection.presentation(key)
    title = view.get("label") or TITLES[kind]
    question = view.get("question") or view.get("summary") or ""
    nodes, edges, groups, relations, notes = [], [], [], [], []
    direction = "TB"
    if kind == "sequence":
        timelines = [item for item in resolved["exposed"] if projection.index[item]["@type"] == "OccurrenceDefinition"]
        if len(timelines) != 1:
            raise ValueError("The sequence view must expose one occurrence definition.")
        svg = _sequence(projection, timelines[0], title, question)
    else:
        if kind == "packages":
            visible = {item for item in resolved["exposed"] if projection.index[item]["@type"] == "Package"}
            native_edges, notes = projection.relationships(visible, scope, {"NamespaceImport", "MembershipImport", "Dependency"})
            nodes = [_node(projection, item, shape="package", lines=()) for item in projection.ordered(visible)]
            edges = [_edge(projection, edge, label="imports") for edge in native_edges]
        elif kind == "block_definition":
            all_nodes, all_edges, _, _ = projection.definitions(None)
            visible = set(all_nodes).intersection(resolved["exposed"])
            nodes = [_node(projection, item) for item in projection.ordered(visible)]
            for edge in all_edges:
                if edge.source not in visible or edge.target not in visible:
                    continue
                # Show declared run ownership once, at the specialization that
                # fixes the four-stage cardinality. Nested subflow execution and
                # current-record selection are documented in the record view.
                if any(role in edge.label for role in ("currentRecord", "currentAssessments", "subFlowExecution")):
                    continue
                specialized_sources = {other.source for other in all_edges
                                       if other.kind == "specialization" and other.target == edge.source}
                if edge.kind == "composition" and any(
                    other.kind == "composition" and other.source in specialized_sources and other.target == edge.target
                    for other in all_edges
                ):
                    continue
                edges.append(_edge(projection, edge, constraint=edge.kind != "reference"))
            # An operation's output typing is a real factory-to-product relation.
            # Resolve it from native ownership rather than invent a creation arrow.
            for relation in projection.local.values():
                if relation["@type"] != "FeatureTyping":
                    continue
                feature = projection.index.get(_one(relation.get("typedFeature")), {})
                owner = projection.index.get(_one(feature.get("owner")), {})
                source = _one(owner.get("owner"))
                target_type = _one(relation.get("type"))
                if source in visible and target_type in visible and feature.get("direction") == "out" and owner.get("@type") == "PerformActionUsage":
                    edges.append(DiagramEdge(relation["@id"], source, target_type, label="returns run", kind="reference"))
        elif kind == "requirements":
            visible = {item for item in resolved["exposed"] if projection.index[item]["@type"] not in {"Package", "SatisfyRequirementUsage"}}
            native_edges, notes = projection.relationships(visible, scope, {"SatisfyRequirementUsage", "RequirementVerificationMembership", "Dependency"})
            nodes = [_node(projection, item) for item in projection.ordered(visible)]
            edges = [_edge(projection, edge) for edge in native_edges]
        elif kind == "use_cases":
            goals = {item for item in selected if projection.index[item]["@type"] == "UseCaseUsage"}
            goals = goals or {item for item in selected if projection.index[item]["@type"] == "UseCaseDefinition"}
            actors, subjects = set(), set()
            role_links = []
            for goal in projection.ordered(goals):
                owners = [goal] + [typed for typed in _ids(projection.index[goal].get("type")) if typed in selected]
                for child in [child for owner in owners for child in projection.children[owner]]:
                    role = projection.index[child]
                    membership = projection.index.get(_one(role.get("owningMembership")), {})
                    role_kind = membership.get("@type")
                    if role_kind not in {"SubjectMembership", "ActorMembership"}:
                        continue
                    typed = [item for item in _ids(role.get("type")) if item in resolved["exposed"]]
                    endpoint = projection.referenced(child, selected) or (typed[0] if len(typed) == 1 else child)
                    (actors if role_kind == "ActorMembership" else subjects).add(endpoint)
                    role_links.append((membership["@id"], endpoint, goal, role_kind))
            if len(subjects) != 1:
                raise ValueError("A use-case diagram must select one explicit system boundary.")
            subject = next(iter(subjects))
            groups = [DiagramGroup(subject, projection.caption(subject))]
            nodes = [_node(projection, item, shape="actor", lines=()) for item in projection.ordered(actors)]
            nodes += [_node(projection, item, shape="ellipse", group=subject, lines=()) for item in projection.ordered(goals)]
            for native_id, endpoint, goal, role_kind in role_links:
                if role_kind == "SubjectMembership":
                    relations.append((native_id, endpoint, goal, "system boundary contains goal"))
                else:
                    edges.append(DiagramEdge(native_id, endpoint, goal, kind="connection"))
            direction = "LR"
        elif kind == "internal_block":
            contexts = [item for item in resolved["exposed"] if projection.index[item]["@type"].endswith("Definition")]
            if len(contexts) != 1:
                raise ValueError("Internal block view must select one automation context.")
            context = contexts[0]
            visible, _, _, notes = projection.internal(context)
            visible = set(visible)
            native_edges, _ = projection.relationships(visible, scope, {"InterfaceUsage", "ConnectionUsage"})
            # Connected participants answer this view's interface question. Creation
            # and the referenced plan are explained by their own diagrams.
            connected = {endpoint for edge in native_edges for endpoint in (edge.source, edge.target)}
            groups = [DiagramGroup(context, projection.caption(context))]
            ports = {}
            for item in sorted(connected):
                ports[item] = []
                for typed in _ids(projection.index[item].get("type")):
                    for child in projection.children[typed]:
                        if projection.index[child]["@type"] == "PortUsage":
                            side = "W" if all(edge.source != item for edge in native_edges) else "E"
                            ports[item].append(DiagramPort(child, projection.caption(child), side))
                nodes.append(_node(projection, item, group=context, ports=ports[item], lines=()))
            for edge in native_edges:
                endpoints = _ids(projection.index[edge.native_id].get("endFeature"))
                ends = []
                for endpoint in endpoints:
                    anonymous = projection.index.get(endpoint, {})
                    chain = _ids(anonymous.get("chainingFeature"))
                    if not chain:
                        for subset in _ids(anonymous.get("ownedReferenceSubsetting")):
                            referred = _one(projection.index.get(subset, {}).get("referencedFeature"))
                            chain = _ids(projection.index.get(referred, {}).get("chainingFeature"))
                            if chain:
                                break
                    ends.append(chain[-1] if len(chain) > 1 else None)
                caption = projection.caption(edge.native_id)
                edges.append(_edge(projection, edge, label=caption,
                                   source_port=ends[0] if ends else None,
                                   target_port=ends[1] if len(ends) > 1 else None))
            direction = "LR"
        elif kind == "parametric":
            constraints = {item for item in selected if projection.index[item]["@type"] == "ConstraintUsage"}
            contexts = {item for item in resolved["exposed"] if projection.index[item]["@type"] in {"PartDefinition", "PartUsage"}}
            values = {item for item in selected if projection.index[item]["@type"] == "AttributeUsage"
                      and projection.ancestor(item, contexts) and not projection.ancestor(item, constraints)}
            visible = constraints | values
            native_edges, notes = projection.relationships(visible, scope, {"BindingConnectorAsUsage", "BindingConnector"})
            for constraint in constraints:
                terminals = []
                for typed in _ids(projection.index[constraint].get("type")):
                    for child in projection.children[typed]:
                        if projection.index[child]["@type"] == "AttributeUsage":
                            terminals.append(DiagramPort(child, projection.caption(child), "W"))
                nodes.append(_node(projection, constraint, shape="constraint", ports=terminals))
            nodes += [_node(projection, item) for item in projection.ordered(values)]
            for edge in native_edges:
                endpoints = _ids(projection.index[edge.native_id].get("endFeature"))
                ends = []
                for endpoint in endpoints:
                    element = projection.index.get(endpoint, {})
                    chain = _ids(element.get("chainingFeature"))
                    if not chain:
                        for subset in _ids(element.get("ownedReferenceSubsetting")):
                            referred = _one(projection.index.get(subset, {}).get("referencedFeature"))
                            chain = _ids(projection.index.get(referred, {}).get("chainingFeature"))
                            if chain:
                                break
                    ends.append(chain[-1] if len(chain) > 1 else None)
                edges.append(_edge(projection, edge, label="=", source_port=ends[0] if ends else None,
                                   target_port=ends[1] if len(ends)>1 else None))
            direction = "LR"
        else:
            allowed = {"ActionUsage", "PerformActionUsage", "DecisionNode", "MergeNode", "ForkNode", "JoinNode"} if kind == "activity" else {"StateUsage"}
            visible = {item for item in selected if projection.index[item]["@type"] in allowed}
            native_edges, notes = projection.relationships(visible, scope, {"SuccessionAsUsage", "Succession", "TransitionUsage"})
            for item in projection.ordered(visible):
                shape = "diamond" if projection.index[item]["@type"] in {"DecisionNode", "MergeNode"} else "state"
                nodes.append(_node(projection, item, shape=shape))
            for edge in native_edges:
                own = projection.index.get(_one(projection.index[edge.native_id].get("owner")), {})
                semantic = own["@id"] if own.get("@type") == "TransitionUsage" else edge.native_id
                presentation = projection.presentation(semantic)
                label = presentation.get("label") or projection.caption(semantic) if _named(projection.index[semantic]) else ""
                summary = presentation.get("summary", "")
                label += ("\n" + summary) if summary else ""
                edges.append(_edge(projection, edge, label=label, tooltip=edge.label))
        if not nodes:
            raise ValueError(f"The declared {kind} selection has no matching native elements.")
        svg = render_diagram(nodes, edges, title, question=question, groups=groups, direction=direction)
        if relations:
            svg = _annotation_relationships(svg, relations)
    return svg, {"name": projection.index[key].get("qualifiedName"), "rendering": TITLES[kind],
                 "origin": "model selection and presentation metadata", "exposedIds": sorted(resolved["exposed"]),
                 "omissions": notes}
