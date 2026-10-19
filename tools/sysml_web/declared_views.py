"""Resolve model-defined views from the pinned compiler's semantic graph."""

from __future__ import annotations

from collections import defaultdict

if __package__:
    from .renderer import CONNECTORS, Edge, Projection, _graph, _ids, _name, _named, _one
else:
    from renderer import CONNECTORS, Edge, Projection, _graph, _ids, _name, _named, _one


# Native identities belong to the pinned v0.1.4 embedded 2026-05 library.
# Matching identities prevents local declarations with similar names from
# masquerading as standard renderings or standard view definitions.
RENDERINGS = {
    "19ead289-7f29-52c6-a417-904b35f6187e": ("Views::asInterconnectionDiagram", "interconnection"),
    "707bc92b-8882-51b1-a1a7-bb125fd3213d": ("Views::asTreeDiagram", "tree"),
    "be34ba7e-c0a4-5371-8ead-888ac6fe6b95": ("Views::asTextualNotation", None),
    "24b9faef-74fb-57eb-9acf-b90401e12082": ("Views::asElementTable", None),
}
STANDARD_VIEWS = {
    "d571047d-d792-5be6-b362-c01793e18663": ("StandardViewDefinitions::GeneralView", "general"),
    "38da8b9f-be68-509f-9bf9-8e6bae8f4a64": ("StandardViewDefinitions::BrowserView", "tree"),
    "99bb2090-4a78-582b-8f8c-24f3e1e772e2": ("StandardViewDefinitions::InterconnectionView", "interconnection"),
    "5aa231fd-4d00-5443-a16b-79a5e48ac80f": ("StandardViewDefinitions::ActionFlowView", None),
    "c0575334-e37c-594e-a477-fa67de03467b": ("StandardViewDefinitions::StateTransitionView", None),
    "e418d298-50ab-5552-82d7-bbb3a2d88ff8": ("StandardViewDefinitions::SequenceView", None),
}


class DeclaredViews:
    def __init__(self, raw: list, unsupported_syntax: dict | None = None) -> None:
        self.projection = Projection(raw)
        self.index = self.projection.index
        self.guards = unsupported_syntax
        self.parents = defaultdict(set)
        for key, element in self.index.items():
            if element["@type"] == "ViewUsage":
                self.parents[key].update(_ids(element.get("type")))
            if element["@type"] in {"Subclassification", "Specialization"}:
                source = _one(element.get("specific"))
                target = _one(element.get("general"))
                if source and target:
                    self.parents[source].add(target)

    def ancestry(self, key: str) -> set[str]:
        found, pending = set(), [key]
        while pending:
            current = pending.pop()
            if current in found:
                continue
            found.add(current)
            pending.extend(self.parents[current] - found)
        return found

    def _rendering(self, key: str, visited=None) -> set[str]:
        visited = set() if visited is None else set(visited)
        if key in visited:
            return set()
        visited.add(key)
        direct = {
            _one(self.index[child].get("referencedRendering"))
            for child in self.projection.children[key]
            if self.index[child]["@type"] == "ViewRenderingMembership"
        } - {None}
        if direct:
            return direct
        inherited = set()
        for parent in self.parents[key]:
            inherited.update(self._rendering(parent, visited))
        return inherited

    def _exposed(self, key: str) -> set[str]:
        result = set()
        for member in _ids(self.index[key].get("importedMembership")):
            membership = self.index.get(member, {})
            member_ids = _ids(membership.get("memberElement"))
            if not member_ids:
                raise ValueError("The compiler did not resolve an exposed membership in this view.")
            for element in member_ids:
                if element not in self.projection.local:
                    raise ValueError("Exposing elements outside the exported local model is not supported.")
                result.add(element)
        if not result:
            raise ValueError("The view exposes no local elements. Add an expose declaration to its view usage.")
        return result

    def selection(self, key: str) -> dict:
        """Validate exposed IDs independently of a presentation adapter."""
        element = self.index[key]
        if element["@type"] != "ViewUsage":
            raise ValueError("A view definition is a template. Select a view usage with expose declarations.")
        ancestry = self.ancestry(key)
        if self.guards is None:
            raise ValueError("Compiler syntax evidence is required to check unsupported view filters.")
        if "*" in self.guards:
            raise ValueError(self.guards["*"])
        for ancestor in ancestry:
            model_element = self.index.get(ancestor, {})
            qualified = model_element.get("qualifiedName")
            if qualified in self.guards:
                raise ValueError(self.guards[qualified])
            for child in self.projection.children[ancestor]:
                kind = self.index[child]["@type"]
                if kind == "ElementFilterMembership":
                    raise ValueError("View filters are not supported by this rendering adapter, including inherited filters.")
                if kind in {"MembershipImport", "NamespaceImport"}:
                    raise ValueError("Imports inside a view cannot be separated reliably from exposed elements. Move imports to its enclosing package.")
        return {"ancestry": ancestry, "exposed": self._exposed(key)}

    def resolve(self, key: str) -> dict:
        selection = self.selection(key)
        ancestry = selection["ancestry"]
        renderings = self._rendering(key)
        if len(renderings) > 1:
            raise ValueError("The view inherits ambiguous rendering declarations.")
        if renderings:
            rendering_id = next(iter(renderings))
            if rendering_id not in RENDERINGS:
                renderer = self.index.get(rendering_id, {})
                name = renderer.get("qualifiedName") or rendering_id
                raise ValueError(f"Custom rendering {name!r} is not supported.")
            rendering, mode = RENDERINGS[rendering_id]
            if mode is None:
                raise ValueError(f"Rendering {rendering} is not supported by the SVG adapter.")
            origin = "render declaration"
        else:
            types = ancestry.intersection(STANDARD_VIEWS)
            supported = {kind for kind in types if STANDARD_VIEWS[kind][1] is not None}
            if types - supported:
                names = ", ".join(STANDARD_VIEWS[kind][0] for kind in types - supported)
                raise ValueError(f"Standard view rendering for {names} is not supported.")
            if len(supported) != 1:
                raise ValueError("The view has no supported render declaration or unambiguous standard view type.")
            rendering, mode = STANDARD_VIEWS[next(iter(supported))]
            origin = "standard view type"
        return {"id": key, "rendering": rendering, "mode": mode, "origin": origin, "exposed": selection["exposed"]}

    def catalog(self) -> list[dict]:
        views = []
        for key, element in self.projection.local.items():
            if element["@type"] not in {"ViewUsage", "ViewDefinition"} or not _named(element):
                continue
            metadata = {
                "id": key, "name": _name(element),
                "qualifiedName": element.get("qualifiedName") or _name(element),
                "rendering": None, "canRender": False, "reason": "",
            }
            try:
                resolved = self.resolve(key)
                metadata.update({"rendering": resolved["rendering"], "canRender": True, "origin": resolved["origin"]})
            except ValueError as error:
                metadata["reason"] = str(error)
            views.append(metadata)
        return sorted(views, key=lambda view: view["qualifiedName"])

    def _scope(self, exposed: set[str]) -> set[str]:
        scope = set()
        for key in exposed:
            scope.update(self.projection.descendants(key))
        return scope

    def _general(self, scope: set[str], title: str) -> str:
        visible = {
            key for key in scope if key in self.projection.local
            and _named(self.index[key]) and self.index[key]["@type"] not in CONNECTORS
            and self.index[key]["@type"] != "Dependency"
        }
        if not visible:
            raise ValueError("The exposed selection contains no named elements to render.")
        edges, notes = [], []
        for key in visible:
            element = self.index[key]
            owner = _one(element.get("owner"))
            membership = _one(element.get("owningMembership"))
            if owner in visible and membership in self.index:
                edges.append(Edge(owner, key, "containment", "contains", membership))
        for relation in self.projection.local.values():
            kind, native_id = relation["@type"], relation["@id"]
            if kind in {"Subclassification", "Specialization", "FeatureTyping"}:
                source = _one(relation.get("specific") or relation.get("typedFeature"))
                target = _one(relation.get("general") or relation.get("type"))
                if source in visible and target in visible:
                    specialization = kind != "FeatureTyping"
                    edges.append(Edge(source, target, "specialization" if specialization else "typing", "specializes" if specialization else "typed by", native_id))
            elif kind == "Dependency":
                for source in _ids(relation.get("client")):
                    for target in _ids(relation.get("supplier")):
                        if source in visible and target in visible:
                            edges.append(Edge(source, target, "dependency", "depends", native_id))
            elif kind in CONNECTORS and native_id in scope:
                endpoints = _ids(relation.get("endFeature") or relation.get("relatedFeature"))
                if kind == "TransitionUsage":
                    endpoints = _ids(relation.get("source")) + _ids(relation.get("target"))
                    if not endpoints and any(self.index[child]["@type"] == "SuccessionAsUsage" for child in self.projection.children[native_id]):
                        # The pinned export carries transition endpoints on
                        # its owned succession, which is projected separately.
                        continue
                ends = [self.projection.endpoint(endpoint, visible) for endpoint in endpoints]
                if len(ends) == 2 and all(ends):
                    source, target = ends
                    label = _name(relation) if _named(relation) else kind
                    owner = self.index.get(_one(relation.get("owner")), {})
                    if kind == "SuccessionAsUsage" and owner.get("@type") == "TransitionUsage" and _named(owner):
                        label = _name(owner)
                    edge_kind = "directed" if kind in {"TransitionUsage", "FlowUsage", "SuccessionAsUsage", "Succession"} else "binding" if kind.startswith("Binding") else "connection"
                    edges.append(Edge(source[0], target[0], edge_kind, label, native_id, source[1] + " → " + target[1]))
                else:
                    notes.append(f"{kind} {native_id}: exported endpoints cannot be projected as named nodes.")
        return _graph(self.projection, self.projection.ordered(visible), list(dict.fromkeys(edges)), title, notes)

    def render(self, target: str) -> tuple[str, dict]:
        key = self.projection.target(target)
        if not key or self.index[key]["@type"] not in {"ViewUsage", "ViewDefinition"}:
            raise ValueError("Select a declared SysML view by its qualified name.")
        resolved = self.resolve(key)
        qualified = self.index[key].get("qualifiedName") or _name(self.index[key])
        title = qualified + " · " + resolved["rendering"]
        scope = self._scope(resolved["exposed"])
        if resolved["mode"] == "tree":
            svg = self.projection.tree(None, selection=scope, title=title)
        elif resolved["mode"] == "general":
            svg = self._general(scope, title)
        else:
            nodes, edges, notes = set(), [], []
            for exposed in resolved["exposed"]:
                element = self.index[exposed]
                if element["@type"] in CONNECTORS:
                    continue
                children = self.projection.children[exposed]
                context = element["@type"].endswith("Definition") or element["@type"] == "Package"
                if context and any(self.index[child]["@type"] in {"PartUsage", "ItemUsage", "PortUsage"} and _named(self.index[child]) for child in children):
                    selected, selected_edges, _, selected_notes = self.projection.internal(exposed)
                    nodes.update(selected)
                    edges.extend(selected_edges)
                    notes.extend(selected_notes)
                elif _named(element):
                    nodes.add(exposed)
            if not nodes:
                raise ValueError("The exposed selection contains no renderable structural elements.")
            # Connect separately exposed parts using their actual exported
            # connector endpoints, even when their owner is not a drawn node.
            for relation in self.projection.local.values():
                if relation["@type"] not in CONNECTORS:
                    continue
                endpoints = _ids(relation.get("endFeature") or relation.get("relatedFeature"))
                ends = [self.projection.endpoint(endpoint, nodes) for endpoint in endpoints]
                if len(ends) == 2 and all(ends):
                    source, target_node = ends
                    kind = relation["@type"]
                    label = _name(relation) if _named(relation) else kind
                    edge_kind = "directed" if kind in {"FlowUsage", "SuccessionAsUsage", "Succession", "AllocationUsage"} else "binding" if kind.startswith("Binding") else "connection"
                    edges.append(Edge(source[0], target_node[0], edge_kind, label, relation["@id"], source[1] + " → " + target_node[1]))
            unique = {}
            for edge in edges:
                unique.setdefault(edge.native_id, edge)
            svg = _graph(self.projection, self.projection.ordered(nodes), list(unique.values()), title, notes, internal=True)
        info = {"name": qualified, "rendering": resolved["rendering"], "origin": resolved["origin"], "exposedIds": sorted(resolved["exposed"])}
        return svg, info
