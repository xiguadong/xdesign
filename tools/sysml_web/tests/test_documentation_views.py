"""Check diagram meaning and readability independently of SVG file existence."""

import unittest
from xml.etree import ElementTree

from tools.render_docs import LOGS, TMP, exported_model
from tools.sysml_web.documentation_views import DocumentationProjection, render_documentation_view
from tools.sysml_web.renderer import _ids, _one
from tools.sysml_web.tests.test_compiler import SVG_NS


VIEWS = {
    'requirements': 'requirementsDiagram', 'packages': 'packageDiagram',
    'block_definition': 'blockDefinitionDiagram', 'internal_block': 'internalBlockDiagram',
    'parametric': 'parametricDiagram', 'use_cases': 'useCaseDiagram',
    'activity': 'activityDiagram', 'state_machine': 'stateMachineDiagram', 'sequence': 'sequenceDiagram',
}


class DocumentationDiagramTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        LOGS.mkdir(parents=True, exist_ok=True)
        TMP.mkdir(parents=True, exist_ok=True)
        cls.raw, cls.guards = exported_model()
        cls.native = {item['@id']: item for item in cls.raw}

    def diagram(self, kind, view=None):
        svg, _ = render_documentation_view(self.raw, self.guards,
                                         'XDesignDetailedDesign::' + (view or VIEWS[kind]), kind)
        return ElementTree.fromstring(svg)

    def nodes(self, svg):
        return {node.attrib['data-native-id']: self.native[node.attrib['data-native-id']]
                for node in svg.findall('.//svg:g[@data-shape]', SVG_NS)}

    def edges(self, svg):
        return svg.findall('.//svg:g[@data-source-id][@data-target-id]', SVG_NS)

    def test_requirements_verify_original_obligations(self):
        svg = self.diagram('requirements')
        nodes = self.nodes(svg)
        requirements = {key for key, node in nodes.items() if node['@type'] == 'RequirementUsage'}
        self.assertEqual({nodes[key]['declaredName'] for key in requirements}, {
            'flowRunIsRealizedByDesign', 'designedFlowGatesEveryStage', 'structureGraphsAreDocumented'})
        edges = self.edges(svg)
        self.assertEqual(len(edges), 6)
        self.assertEqual({edge.attrib['data-target-id'] for edge in edges}, requirements)
        verifications = [edge for edge in edges
                         if self.native[edge.attrib['data-native-id']]['@type'] == 'RequirementVerificationMembership']
        self.assertEqual(len(verifications), 3)
        for edge in verifications:
            membership = self.native[edge.attrib['data-native-id']]
            proxy = self.native[_one(membership['memberElement'])]
            targets = {_one(self.native[item].get('subsettedFeature'))
                       for item in _ids(proxy.get('ownedSubsetting'))}
            self.assertIn(edge.attrib['data-target-id'], targets)
            self.assertEqual(nodes[edge.attrib['data-source-id']]['@type'], 'VerificationCaseDefinition')

    def test_parametric_values_bind_to_visible_native_terminals(self):
        svg = self.diagram('parametric')
        nodes = self.nodes(svg)
        self.assertEqual(len(nodes), 5)
        ports = svg.findall(".//svg:g[@class='diagram-port']", SVG_NS)
        self.assertEqual(len(ports), 4)
        self.assertTrue(all(self.native[port.attrib['data-native-id']]['@type'] == 'AttributeUsage' for port in ports))
        constraints = {key for key, node in nodes.items() if node['@type'] == 'ConstraintUsage'}
        self.assertEqual(len(constraints), 1)
        for edge in self.edges(svg):
            self.assertEqual(self.native[edge.attrib['data-native-id']]['@type'], 'BindingConnectorAsUsage')
            self.assertEqual(len({edge.attrib['data-source-id'], edge.attrib['data-target-id']} & constraints), 1)
        visible = ' '.join(text.text or '' for text in svg.findall('.//svg:text', SVG_NS))
        self.assertNotIn('->select', visible)
        self.assertIn('Gate authorization', visible)

    def test_use_case_subject_is_boundary_and_actors_remain_outside(self):
        svg = self.diagram('use_cases')
        nodes = self.nodes(svg)
        self.assertEqual({node['declaredName'] for node in nodes.values()}, {
            'progressWish', 'reviewEvidence', 'WishRequester', 'Judge'})
        groups = svg.findall(".//svg:g[@data-native-id][@class='cluster diagram-group']", SVG_NS)
        self.assertEqual(len(groups), 1)
        subject = groups[0].attrib['data-native-id']
        self.assertEqual(self.native[subject]['declaredName'], 'SimpleFlowAutomation')
        goals = {key for key, node in nodes.items() if node['@type'] == 'UseCaseUsage'}
        self.assertEqual(len(goals), 2)
        contained = {node.attrib['data-native-id'] for node in groups[0].findall('.//svg:g[@data-shape]', SVG_NS)}
        # Graphviz renders clusters and nodes as siblings; source group attributes
        # retain containment explicitly when SVG nesting is not used.
        for goal in goals:
            node = next(item for item in svg.findall('.//svg:g[@data-shape]', SVG_NS)
                        if item.attrib['data-native-id'] == goal)
            self.assertEqual(node.attrib.get('data-group-id'), subject)
        roles = [self.native[edge.attrib['data-native-id']]['@type'] for edge in self.edges(svg)]
        self.assertEqual(roles.count('ActorMembership'), 2)
        self.assertEqual(roles.count('SubjectMembership'), 2)

    def test_internal_interfaces_have_owning_boundary_and_four_ports(self):
        svg = self.diagram('internal_block')
        self.assertEqual(len(self.nodes(svg)), 3)
        ports = svg.findall(".//svg:g[@class='diagram-port']", SVG_NS)
        self.assertEqual(len(ports), 4)
        self.assertEqual(len(svg.findall(".//svg:g[@class='cluster diagram-group']", SVG_NS)), 1)
        for edge in self.edges(svg):
            self.assertEqual(self.native[edge.attrib['data-native-id']]['@type'], 'InterfaceUsage')
            self.assertIn('data-source-port-id', edge.attrib)
            self.assertIn('data-target-port-id', edge.attrib)

    def test_state_guards_remain_in_native_tooltips_with_short_visible_captions(self):
        svg = self.diagram('state_machine')
        nodes = self.nodes(svg)
        self.assertEqual({node['declaredName'] for node in nodes.values()}, {'pending', 'active', 'blocked', 'completed'})
        edges = self.edges(svg)
        self.assertEqual(len(edges), 6)
        self.assertTrue(any('stage.completesFlow' in ''.join(edge.itertext()) for edge in edges))
        visible = ' '.join(text.text or '' for text in svg.findall('.//svg:text', SVG_NS))
        self.assertNotIn('run.currentRecord', visible)
        self.assertIn('terminal visit', visible.lower())

    def test_bdd_keeps_cardinality_on_relationship_and_avoids_duplicate_inherited_ownership(self):
        svg = self.diagram('block_definition')
        edges = self.edges(svg)
        labels = [' '.join(text.text or '' for text in edge.findall('.//svg:text', SVG_NS)) for edge in edges]
        self.assertTrue(any('[4..*]' in label for label in labels))
        self.assertEqual(sum('stage records' in label for label in labels), 1)
        self.assertTrue(any('returns run' in label for label in labels))

    def test_all_nine_are_legible_at_900_pixels_without_elision(self):
        for kind in VIEWS:
            with self.subTest(kind=kind):
                svg = self.diagram(kind)
                width = float(svg.attrib['viewBox'].split()[2])
                texts = svg.findall('.//svg:text', SVG_NS)
                font_sizes = [float(text.attrib.get('font-size', 15)) for text in texts]
                self.assertGreaterEqual(min(font_sizes) * min(1, 900 / width), 11)
                self.assertFalse(any('…' in (text.text or '') for text in texts))
                self.assertTrue(svg.find('svg:title', SVG_NS).text)
                self.assertTrue(svg.find('svg:desc', SVG_NS).text)

    def test_sequence_retains_event_anchored_message_order(self):
        svg = self.diagram('sequence')
        edges = self.edges(svg)
        self.assertEqual(len(edges), 10)
        self.assertEqual([self.native[edge.attrib['data-native-id']]['declaredName'] for edge in edges][::9],
                         ['createRun', 'completeRun'])
        self.assertTrue(all(self.native[edge.attrib['data-native-id']]['@type'] == 'FlowUsage' for edge in edges))


if __name__ == '__main__':
    unittest.main()
