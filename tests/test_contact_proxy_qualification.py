"""Separate four-axis descriptors;never repaired literal vectors or mechanisms."""
import copy
import itertools
import json
import unittest
from unittest.mock import patch
from audits import contact_proxy_qualification as audit
from audits import contact_consequence_support as literal
from audits import runner_transitions as runner
from tests.test_contact_consequence_support import fixture
from tests.test_runner_only_steals import fixture as runner_only_fixture
from tests.test_runner_transitions import attached, child


def source_pair(family, mask, outs):
    pair = fixture(family, mask=mask, outs=outs)
    if family == 'SF' and not mask & 4:
        # The predecessor positive-fixture helper scores only Third. With Third
        # absent,keep a source-valid zero-run SF-label counterexample,not an
        # invented scorer or a relaxed runner/event reconciliation.
        pair[0]['runs'] = '0'
        pair[1].update(runs='0', state_runs='0')
    return pair


def no_play_control():
    row, _ = fixture('SF')
    row.update(no_play_flag='true', raw_play='NP/SF', result='', outs='0', runs='0', credits='[]', runners='[]')
    real = fixture('SF', key=2); real[0]['raw_play'] = '7'
    # A genuine unlabeled real PA supplies the required nonempty outcome export.
    return [(row, None), real]


def evidence(pairs):
    rows, contexts = [p[0] for p in pairs], [p[1] for p in pairs if p[1] is not None]
    games = sorted({row['game_id'] for row in rows})
    census = runner.inventory(rows, contexts, games)
    groups = {}
    for cell in census['raw_child_cells']:
        facts = {name: value for name, value in cell.items() if name != 'rows'}
        groups[json.dumps(facts, sort_keys=True)] = cell['rows']
    cohort = {name: copy.deepcopy(census[name]) for name in runner.COHORT}
    cohort['raw_totals'] = {name: census['raw_totals'][name] for name in runner.COHORT['raw_totals']}
    joint = {row['event_key']: literal._joint_facts(row, context)
        for row, context in pairs if context is not None}
    kwargs = dict(native_groups=groups, expected_census=census, native_joint=joint, cohort=cohort)
    accepted = literal.inventory(rows, contexts, games, **kwargs)
    return rows, contexts, games, dict(kwargs, expected_support=accepted)


def reconstruct(value):
    result = copy.deepcopy(value['literal_support'])
    result['contexts'] = [{name: item for name, item in cell.items()
        if name != 'event_geometry_proxy'} for cell in value['contexts']]
    return result


class GeometryProjection(unittest.TestCase):
    def test_three_families_are_separate_descriptors_with_exact_conservation(self):
        args = evidence([fixture(family, key=i) for i, family in enumerate(literal.FAMILIES, 1)])
        value = audit.inventory(*args[:3], **args[3])
        self.assertEqual(value['format'], '2019-contact-proxy-qualification-v1')
        self.assertEqual(value['causal_certification'], 'unavailable')
        for family in literal.FAMILIES:
            self.assertEqual(value['families'][family], dict(events=1, geometry_proxy=1, not_geometry_proxy=0))
        self.assertTrue(all(cell['event_geometry_proxy'] is True for cell in value['contexts']))
        self.assertEqual(sum(cell['events'] for cell in value['contexts']), 3)
        self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_missing_batter_can_match_geometry_without_becoming_a_complete_vector(self):
        pair = fixture('SF')
        pair[0]['runners'] = json.dumps([v for v in json.loads(pair[0]['runners']) if v['baserunner'] != 'Batter'])
        args = evidence([pair])
        value = audit.inventory(*args[:3], **args[3]); cell = value['contexts'][0]
        self.assertIs(cell['event_geometry_proxy'], True)
        self.assertEqual(cell['shape'], 'contradicted_or_incomplete')
        self.assertIn('incomplete_vector', cell['reasons'])
        self.assertEqual([v['baserunner'] for v in cell['runners']], ['Third'])
        self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_retired_null_or_origin_endpoints_stay_literal_even_when_geometry_matches(self):
        for family, role, endpoint in (('SF', 'Batter', None), ('GIDP', 'First', 'First')):
            with self.subTest(family=family, endpoint=endpoint):
                pair = fixture(family); children = json.loads(pair[0]['runners'])
                next(v for v in children if v['baserunner'] == role)['base_end'] = endpoint
                pair[0]['runners'] = json.dumps(children); args = evidence([pair])
                value = audit.inventory(*args[:3], **args[3]); cell = value['contexts'][0]
                self.assertIs(cell['event_geometry_proxy'], True)
                observed = next(v for v in cell['runners'] if v['baserunner'] == role)
                self.assertEqual(observed['base_end'], endpoint)
                self.assertIs(observed['is_out'], True)
                self.assertNotEqual(cell['shape'], 'complete_literal')
                self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_complete_support_mismatch_refuses_before_projection(self):
        args = evidence([fixture('SF')]); args[3]['expected_support']['games'] += 1
        with patch.object(audit, '_project', create=True, side_effect=AssertionError('projection started')) as project:
            with self.assertRaises(ValueError): audit.inventory(*args[:3], **args[3])
            project.assert_not_called()

    def test_each_missing_reconciliation_input_refuses_before_projection(self):
        for name in ('native_groups', 'expected_census', 'native_joint', 'cohort', 'expected_support'):
            with self.subTest(name=name):
                args = evidence([fixture('SF')]); args[3][name] = None
                with patch.object(audit, '_project', create=True, side_effect=AssertionError('projection started')) as project:
                    with self.assertRaises(ValueError): audit.inventory(*args[:3], **args[3])
                    project.assert_not_called()

    def test_inputs_and_complete_literal_report_are_immutable(self):
        args = evidence([fixture('SF'), fixture('SH', key=2)])
        before = copy.deepcopy(args)
        value = audit.inventory(*args[:3], **args[3])
        self.assertEqual(args, before)
        self.assertEqual(reconstruct(value), before[3]['expected_support'])
        self.assertNotIn('contexts', value['literal_support'])
        self.assertEqual(value['literal_support']['eligible_before_state_pa_frames'],
            before[3]['expected_support']['eligible_before_state_pa_frames'])

    def test_source_matrix_and_no_play_fixture_evidence_is_fully_reconciled(self):
        cases = [source_pair(family, mask, outs)
            for family, mask, outs in itertools.product(literal.FAMILIES, range(8), range(3))
            if not (family == 'GIDP' and outs == 2)]
        for pair in cases:
            args = evidence([pair])
            self.assertEqual(args[3]['expected_support']['validation'],
                dict(census=True, native_child=True, native_joint=True, cohort=True))
        self.assertEqual(evidence(no_play_control())[3]['expected_support']['totals']['real_events'], 1)

    def test_cartesian_source_before_frames_have_exact_mask_and_out_vetoes(self):
        for family, mask, outs in itertools.product(literal.FAMILIES, range(8), range(3)):
            if family == 'GIDP' and outs == 2:
                continue  # after_outs=4 is invalid native source,covered by strict guard below.
            with self.subTest(family=family, mask=mask, outs=outs):
                args = evidence([source_pair(family, mask, outs)])
                value = audit.inventory(*args[:3], **args[3])
                expected = mask in ((4, 5, 6, 7) if family == 'SF' else (1, 5)) and outs in (0, 1)
                self.assertIs(value['contexts'][0]['event_geometry_proxy'], expected)
                self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_wrong_event_unit_pairs_never_match_geometry(self):
        cases = []
        sf = fixture('SH'); sf[0]['raw_play'] = '13/SF'; sf[1]['derived_bunts'] = '0'; cases.append(('SF', sf))
        sh = fixture('SF', mask=5); sh[0]['raw_play'] = '7/SH'; sh[1]['derived_bunts'] = '1'; cases.append(('SH', sh))
        dp = fixture('SH'); dp[0]['raw_play'] = '63/GDP'; dp[1].update(gidp='1', dp='1', derived_bunts='0'); cases.append(('GIDP', dp))
        for family, pair in cases:
            with self.subTest(family=family):
                args = evidence([pair]); value = audit.inventory(*args[:3], **args[3]); cell = value['contexts'][0]
                self.assertEqual(cell['family'], family)
                self.assertIn('event_units', cell['reasons'])
                self.assertIs(cell['event_geometry_proxy'], False)
                self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_conflicting_source_labels_keep_memberships_but_no_geometry_matches(self):
        pair = fixture('SF', mask=5); pair[0]['raw_play'] = '7/SF/SH'; pair[1]['derived_bunts'] = '1'
        args = evidence([pair]); value = audit.inventory(*args[:3], **args[3])
        self.assertEqual(value['literal_support']['totals']['candidate_events'], 1)
        self.assertEqual(value['literal_support']['totals']['overlap_events'], 1)
        self.assertEqual(sum(cell['events'] for cell in value['contexts']), 2)
        self.assertEqual({cell['family'] for cell in value['contexts']}, {'SF', 'SH'})
        self.assertTrue(all('conflicting_labels' in cell['reasons'] and cell['event_geometry_proxy'] is False for cell in value['contexts']))
        self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_no_label_alias_casefold_or_substring_inference(self):
        for notation in ('7', '7/sf', '7/XSF', '7/SFX'):
            with self.subTest(notation=notation):
                pair = fixture('SF'); pair[0]['raw_play'] = notation
                args = evidence([pair]); value = audit.inventory(*args[:3], **args[3])
                self.assertEqual(value['contexts'], [])
                self.assertTrue(all(v == dict(events=0, geometry_proxy=0, not_geometry_proxy=0) for v in value['families'].values()))
                self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_nonpa_label_keeps_missing_pa_and_cannot_be_a_geometry_proxy(self):
        pair = runner_only_fixture(key=1); pair[0]['raw_play'] = 'SB2/SF'
        args = evidence([pair]); value = audit.inventory(*args[:3], **args[3]); cell = value['contexts'][0]
        self.assertIsNone(cell['pa'])
        self.assertIn('not_terminal_pa', cell['reasons'])
        self.assertIs(cell['event_geometry_proxy'], False)
        self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_no_play_label_retains_original_diagnostics_without_a_proxy_cell(self):
        args = evidence(no_play_control()); value = audit.inventory(*args[:3], **args[3])
        self.assertEqual(value['contexts'], [])
        self.assertEqual(value['literal_support']['no_play_label_counts']['SF'], 1)
        self.assertEqual(value['literal_support']['totals']['real_events'], 1)
        self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_notation_uncertainty_and_nullable_child_effects_are_not_cleared(self):
        pair = fixture('SF'); pair[0]['raw_play'] = '7/SF/F7'
        children = json.loads(pair[0]['runners']); children[0]['explicit_out_flag'] = None
        pair[0]['runners'] = json.dumps(children); args = evidence([pair])
        value = audit.inventory(*args[:3], **args[3]); cell = value['contexts'][0]
        self.assertIs(cell['event_geometry_proxy'], True)
        self.assertIn('notation_uncertain', cell['reasons'])
        self.assertIn('runner_effect_or_unknown', cell['reasons'])
        self.assertIsNone(cell['runners'][0]['explicit_out_flag'])
        self.assertEqual(reconstruct(value), args[3]['expected_support'])

    def test_row_and_child_permutations_repeat_exact_complete_reports(self):
        pairs = [fixture(family, key=i) for i, family in enumerate(literal.FAMILIES, 1)]
        original_args = evidence(pairs); original = audit.inventory(*original_args[:3], **original_args[3])
        for order in itertools.permutations(pairs):
            args = evidence(copy.deepcopy(order))
            self.assertEqual(audit.inventory(*args[:3], **args[3]), original)
        for i, pair in enumerate(pairs):
            for children in itertools.permutations(json.loads(pair[0]['runners'])):
                varied = copy.deepcopy(pairs); varied[i][0]['runners'] = json.dumps(children)
                args = evidence(varied); self.assertEqual(audit.inventory(*args[:3], **args[3]), original)
        self.assertEqual(audit.inventory(*original_args[:3], **original_args[3]), original)

    def test_each_complete_reconciliation_mismatch_refuses_before_projection(self):
        for name in ('native_groups', 'expected_census', 'native_joint', 'cohort'):
            with self.subTest(name=name):
                args = evidence([fixture('SF')]); args[3][name] = {}
                with patch.object(audit, '_project', create=True, side_effect=AssertionError('projection started')) as project:
                    with self.assertRaises(ValueError): audit.inventory(*args[:3], **args[3])
                    project.assert_not_called()

    def test_native_bool_alias_and_impossible_after_outs_refuse_before_projection(self):
        for name, value in (('pa', True), ('event_runs', False), ('after_outs', 4)):
            with self.subTest(name=name, value=value):
                args = evidence([fixture('SF')]); args[3]['native_joint']['1'][name] = value
                with patch.object(audit, '_project', create=True, side_effect=AssertionError('projection started')) as project:
                    with self.assertRaises(ValueError): audit.inventory(*args[:3], **args[3])
                    project.assert_not_called()

    def test_only_publication_provenance_may_differ_in_expected_support(self):
        args = evidence([fixture('SF')]); args[3]['expected_support']['provenance'] = {'test': 'published separately'}
        value = audit.inventory(*args[:3], **args[3]); expected = copy.deepcopy(args[3]['expected_support']); expected.pop('provenance')
        self.assertEqual(reconstruct(value), expected)
        args[3]['expected_support']['extra'] = 'not provenance'
        with self.assertRaises(ValueError): audit.inventory(*args[:3], **args[3])

    def test_expected_support_bool_float_aliases_never_equal_integer_units(self):
        for kind in ('games_bool', 'candidate_float', 'context_bool'):
            with self.subTest(kind=kind):
                args = evidence([fixture('SF')]); expected = args[3]['expected_support']
                if kind == 'games_bool': expected['games'] = True
                elif kind == 'candidate_float': expected['totals']['candidate_events'] = 1.0
                else: expected['contexts'][0]['events'] = True
                with patch.object(audit, '_project', create=True, side_effect=AssertionError('projection started')) as project:
                    with self.assertRaises(ValueError): audit.inventory(*args[:3], **args[3])
                    project.assert_not_called()

    def test_nonfinite_expected_support_refuses_without_projection(self):
        for value in (float('nan'), float('inf'), -float('inf')):
            with self.subTest(value=value):
                args = evidence([fixture('SF')]); args[3]['expected_support']['games'] = value
                with patch.object(audit, '_project', create=True, side_effect=AssertionError('projection started')) as project:
                    with self.assertRaises(ValueError): audit.inventory(*args[:3], **args[3])
                    project.assert_not_called()

    def test_extra_native_child_is_preserved_without_becoming_a_before_opportunity(self):
        pair = fixture('SF')
        children = json.loads(pair[0]['runners']) + [child('Second', 'EXTRA', base_end='Second')]
        args = evidence([attached(pair, children)])
        self.assertEqual(args[3]['expected_census']['totals']['extra_child_rows'], 1)
        self.assertEqual(args[3]['expected_census']['totals']['opportunities'], 2)
        value = audit.inventory(*args[:3], **args[3]); cell = value['contexts'][0]
        self.assertIs(cell['event_geometry_proxy'], True)
        self.assertEqual(len(cell['runners']), len(children))
        observed = next(c for c in cell['runners'] if c['baserunner'] == 'Second')
        expected = next(c for c in args[3]['expected_support']['contexts'][0]['runners'] if c['baserunner'] == 'Second')
        self.assertEqual(observed, expected)
        self.assertIs(observed['identity_present'], True)
        self.assertNotIn('runner_id', observed)
        self.assertIn('incomplete_vector', cell['reasons'])
        self.assertNotEqual(cell['shape'], 'complete_literal')
        self.assertEqual(reconstruct(value), args[3]['expected_support'])
        self.assertEqual(value['literal_support']['eligible_before_state_pa_frames'],
            args[3]['expected_support']['eligible_before_state_pa_frames'])

    def test_original_inventory_receives_every_evidence_input_before_projection(self):
        args = evidence([fixture('SF')])
        with patch.object(literal, 'inventory', wraps=literal.inventory) as prior:
            value = audit.inventory(*args[:3], **args[3])
        prior.assert_called_once_with(*args[:3], **{k: v for k, v in args[3].items() if k != 'expected_support'})
        self.assertEqual(reconstruct(value), args[3]['expected_support'])


if __name__ == '__main__':
    unittest.main()
