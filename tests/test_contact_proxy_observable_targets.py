"""Direct after descriptors versus supplied fictional canonical shapes,not causes."""
import copy
import itertools
import json
import unittest
from unittest.mock import patch
from audits import contact_proxy_observable_targets as audit
from audits import contact_proxy_joint_cohort as previous
from tests.test_contact_proxy_joint_cohort import inputs as inherited_inputs
from tests.test_contact_consequence_support import fixture
from tests.test_contact_proxy_qualification import no_play_control, source_pair
from tests.test_joint_contact import fixture as ordinary
from tests.test_runner_only_steals import fixture as runner_only
from tests.test_runner_transitions import attached, child


def shape_fixture():
    """Test data only;the real measurement reference must come from Rust."""
    rows = []
    for family in ('SF', 'SH', 'GIDP'):
        for mask, outs in itertools.product((4, 5, 6, 7) if family == 'SF' else (1, 5), (0, 1)):
            delta_outs = 2 if family == 'GIDP' else 1
            end = outs + delta_outs
            mask_after = mask & ~4 if family == 'SF' else (mask & ~1) | 2 if family == 'SH' else mask & ~1
            rows.append(dict(family=family, base_mask=mask, outs_before=outs,
                delta_outs=delta_outs, delta_runs=int(family == 'SF'), event_end_outs=end,
                normalized_after_outs=0 if end == 3 else end,
                normalized_after_base_mask=0 if end == 3 else mask_after,
                frame_changed=end == 3))
    return dict(format='contact-proxy-controlled-shapes-v1',
        policy='supplied-contact-consequence-projection-v1',
        control_state=dict(inning=2, half='Top', score=[0, 0]),
        assumptions='Fictional eligible actors and supplied clean contact facts;not historical causes',
        contexts=rows)


def inputs(pairs, supported=None):
    rows, outcomes, games, proof = inherited_inputs(pairs, supported)
    proof['expected_joint_cohort'] = previous.inventory(rows, outcomes, games, **proof)
    proof['canonical_shapes'] = shape_fixture()
    return rows, outcomes, games, proof


def run(pairs):
    args = inputs(pairs)
    return audit.inventory(*args[:3], **args[3]), args[3]


class ObservableTargets(unittest.TestCase):
    def test_complete_prior_reports_and_direct_fields_are_preserved(self):
        value, proof = run([fixture(f, key=i+1) for i, f in enumerate(('SF', 'SH', 'GIDP'))])
        self.assertEqual(value['format'], '2019-contact-proxy-observable-targets-v1')
        self.assertEqual(value['causal_certification'], 'unavailable')
        self.assertEqual(value['historical_canonical_eligibility'], 'unavailable')
        self.assertEqual(value['joint_cohort_support'], proof['expected_joint_cohort'])
        self.assertEqual(value['canonical_shapes'], proof['canonical_shapes'])
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 3)
        self.assertEqual(sum(c['events'] for c in value['memberships']), 3)
        for cell in value['memberships']:
            self.assertIs(cell['frame_end'], False)
            self.assertIs(cell['literal_after_check_reached'], True)
            self.assertEqual(cell['raw_out_sum_relation'], 'match')
            self.assertEqual(cell['controlled_mask_relation'], 'match')

    def test_missing_batter_does_not_hide_known_mask_difference(self):
        row, context = fixture('SF')
        row['runners'] = json.dumps([c for c in json.loads(row['runners']) if c['baserunner'] != 'Batter'])
        context.update(after_base_mask='1')
        value, _ = run([(row, context)])
        self.assertEqual(len(value['memberships']), 1)
        cell = value['memberships'][0]
        self.assertIs(cell['event_geometry_proxy'], True)
        self.assertIs(cell['literal_after_check_reached'], False)
        self.assertIn('incomplete_vector', cell['literal_reasons'])
        self.assertNotIn('after_state_evidence', cell['literal_reasons'])
        self.assertEqual(cell['after_outs'], 1)
        self.assertEqual(cell['after_base_mask'], 1)
        self.assertEqual(cell['raw_out_sum_relation'], 'match')
        self.assertEqual(cell['controlled_mask_relation'], 'different')

    def test_known_out_difference_and_unknown_mask_remain_separate_at_boundary(self):
        pair = fixture('SF'); pair[1].update(frame_end='true', after_outs='2', after_base_mask='')
        value, _ = run([pair])
        self.assertEqual(len(value['memberships']), 1)
        cell = value['memberships'][0]
        self.assertEqual(cell['after_outs'], 2)
        self.assertIsNone(cell['after_base_mask'])
        self.assertEqual(cell['raw_out_sum_relation'], 'different')
        self.assertEqual(cell['controlled_mask_relation'], 'not_compared_boundary')

    def test_boundary_unknown_outs_do_not_become_zero(self):
        pair = fixture('SF'); pair[1].update(frame_end='true', after_outs='', after_base_mask='1')
        value, _ = run([pair])
        self.assertEqual(len(value['memberships']), 1)
        cell = value['memberships'][0]
        self.assertIsNone(cell['after_outs'])
        self.assertEqual(cell['after_base_mask'], 1)
        self.assertEqual(cell['raw_out_sum_relation'], 'unknown')
        self.assertEqual(cell['controlled_mask_relation'], 'not_compared_boundary')

    def test_invalid_nonboundary_missing_fields_and_out_sum_refuse_before_projection(self):
        rows, outcomes, games, proof = inputs([fixture('SF')])
        for name, bad, message in [('after_outs', '', 'missing/invalid after_outs'),
                ('after_base_mask', '', 'missing/invalid after_base_mask'),
                ('after_outs', '2', 'active after-outs mismatch')]:
            changed = copy.deepcopy(outcomes); changed[0][name] = bad
            with self.subTest(name=name, bad=bad), patch.object(audit, '_project', create=True,
                    side_effect=AssertionError('new counters started')):
                with self.assertRaisesRegex(ValueError, message):
                    audit.inventory(rows, changed, games, **proof)

    def test_third_out_keeps_raw_three_and_null_not_normalized_zero(self):
        value, proof = run([fixture('GIDP', outs=1)])
        self.assertEqual(len(value['memberships']), 1)
        cell = value['memberships'][0]
        self.assertIs(cell['frame_end'], True)
        self.assertEqual(cell['after_outs'], 3)
        self.assertIsNone(cell['after_base_mask'])
        self.assertEqual(cell['raw_out_sum_relation'], 'match')
        self.assertEqual(cell['controlled_mask_relation'], 'not_compared_boundary')
        reference = next(c for c in proof['canonical_shapes']['contexts'] if
            c['family'] == 'GIDP' and c['base_mask'] == 5 and c['outs_before'] == 1)
        self.assertEqual(reference['normalized_after_outs'], 0)
        self.assertEqual(reference['normalized_after_base_mask'], 0)

    def test_source_frame_end_alone_prevents_mask_comparison(self):
        pair = fixture('SF'); pair[1].update(frame_end='true', after_base_mask='1')
        value, _ = run([pair])
        self.assertEqual(len(value['memberships']), 1)
        cell = value['memberships'][0]
        self.assertIs(cell['frame_end'], True)
        self.assertEqual(cell['after_base_mask'], 1)
        self.assertEqual(cell['controlled_mask_relation'], 'not_compared_boundary')
        self.assertEqual(cell['raw_out_sum_relation'], 'match')

    def test_controlled_half_change_alone_prevents_mask_comparison(self):
        pair = fixture('GIDP', outs=1)
        pair[1].update(frame_end='false', after_base_mask='4')
        value, _ = run([pair]); cell = value['memberships'][0]
        self.assertIs(cell['frame_end'], False)
        self.assertEqual(cell['after_outs'], 3)
        self.assertEqual(cell['after_base_mask'], 4)
        self.assertEqual(cell['raw_out_sum_relation'], 'match')
        self.assertEqual(cell['controlled_mask_relation'], 'not_compared_boundary')

    def test_new_whole_baseline_and_canonical_reference_are_required_before_projection(self):
        args = inputs([fixture('SH')])
        for name in ('expected_joint_cohort', 'canonical_shapes'):
            with self.subTest(name=name), patch.object(audit, '_project', create=True,
                    side_effect=AssertionError('new counters started')):
                with self.assertRaises(ValueError):
                    audit.inventory(*args[:3], **dict(args[3], **{name: None}))

    def test_every_inherited_evidence_object_is_required_before_new_counters(self):
        args = inputs([fixture('SH')])
        for name in args[3]:
            with self.subTest(name=name), patch.object(audit, '_project', create=True,
                    side_effect=AssertionError('new counters started')):
                with self.assertRaises(ValueError):
                    audit.inventory(*args[:3], **dict(args[3], **{name: None}))

    def test_whole_prior_cohort_is_type_strict_and_late_mismatches_refuse(self):
        args = inputs([fixture('SF')]); variants = []
        for alias in (True, 1.0, float('nan')):
            proof = copy.deepcopy(args[3]); proof['expected_joint_cohort']['source_contexts'][0]['events'] = alias
            variants.append(proof)
        proof = copy.deepcopy(args[3]); proof['expected_joint_cohort']['memberships'][0]['literal_reasons'] += ['new invented reason']
        variants.append(proof)
        proof = copy.deepcopy(args[3]); proof['expected_joint_cohort']['joint_support']['examples'][0]['event']['raw_play'] = 'changed late example'
        variants.append(proof)
        for i, proof in enumerate(variants):
            with self.subTest(proof=i), patch.object(audit, '_project', create=True,
                    side_effect=AssertionError('new counters started')):
                with self.assertRaises(ValueError):
                    audit.inventory(*args[:3], **proof)

    def test_canonical_reference_schema_contexts_types_and_units_refuse_before_counters(self):
        args = inputs([fixture('SH')]); variants = [[], {}, dict(shape_fixture(), policy='other')]
        for path, bad in [('missing', None), ('duplicate', None), ('base_mask', True),
                ('outs_before', 0.0), ('frame_changed', 0), ('delta_outs', 2),
                ('event_end_outs', 3), ('normalized_after_outs', 0),
                ('normalized_after_base_mask', float('nan')), ('family', 'aliased SF')]:
            ref = shape_fixture()
            if path == 'missing': ref['contexts'].pop()
            elif path == 'duplicate': ref['contexts'].append(copy.deepcopy(ref['contexts'][0]))
            else: ref['contexts'][0][path] = bad
            variants.append(ref)
        ref = shape_fixture(); ref['control_state']['score'] = [False, 0]; variants.append(ref)
        ref = shape_fixture(); ref['contexts'][-1]['normalized_after_base_mask'] = 4; variants.append(ref)
        for i, ref in enumerate(variants):
            with self.subTest(i=i), patch.object(audit, '_project', create=True,
                    side_effect=AssertionError('new counters started')):
                with self.assertRaises(ValueError):
                    audit.inventory(*args[:3], **dict(args[3], canonical_shapes=ref))

    def test_native_and_census_mismatch_cannot_start_new_projection(self):
        args = inputs([fixture('GIDP')])
        for path, bad in [('event_runs', 1), ('frame_end', 0), ('after_outs', True)]:
            proof = copy.deepcopy(args[3]); proof['native_joint']['1'][path] = bad
            with self.subTest(path=path), patch.object(audit, '_project', create=True,
                    side_effect=AssertionError('new counters started')):
                with self.assertRaises(ValueError):
                    audit.inventory(*args[:3], **proof)
        proof = copy.deepcopy(args[3]); proof['expected_census']['totals']['real_events'] += 1
        with patch.object(audit, '_project', create=True, side_effect=AssertionError('new counters started')):
            with self.assertRaises(ValueError):
                audit.inventory(*args[:3], **proof)

    def test_all64_source_contexts_keep_original_descriptor_and_finite_relations(self):
        cases = [(f, mask, outs) for f, mask, outs in
            itertools.product(('SF', 'SH', 'GIDP'), range(8), range(3)) if f != 'GIDP' or outs < 2]
        self.assertEqual(len(cases), 64)
        for family, mask, outs in cases:
            with self.subTest(family=family, mask=mask, outs=outs):
                value, proof = run([source_pair(family, mask, outs)])
                self.assertEqual(len(value['memberships']), 1)
                cell = value['memberships'][0]; old = proof['expected_joint_cohort']['memberships'][0]
                for name in old:
                    self.assertEqual(cell[name], old[name])
                if old['event_geometry_proxy']:
                    self.assertEqual(cell['raw_out_sum_relation'], 'match')
                    self.assertEqual(cell['controlled_mask_relation'],
                        'not_compared_boundary' if cell['frame_end'] else 'match')
                else:
                    self.assertEqual(cell['raw_out_sum_relation'], 'not_geometry_proxy')
                    self.assertEqual(cell['controlled_mask_relation'], 'not_geometry_proxy')

    def test_conflicting_memberships_never_double_count_source_events(self):
        pair = fixture('SH', mask=1); pair[0]['raw_play'] = '13/SH/SF'
        value, proof = run([pair])
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 1)
        self.assertEqual(sum(c['overlap_events'] for c in value['source_contexts']), 1)
        self.assertEqual(sum(c['events'] for c in value['memberships']), 2)
        self.assertEqual(value['joint_cohort_support'], proof['expected_joint_cohort'])
        for cell in value['memberships']:
            self.assertFalse(cell['event_geometry_proxy'])
            self.assertEqual(cell['controlled_mask_relation'], 'not_geometry_proxy')

    def test_no_play_and_non_pa_null_zero_remain_in_correct_grains(self):
        pairs = no_play_control() + [runner_only(key=3), runner_only(key=4)]
        for row, context in pairs[2:]: row['raw_play'] = 'SB2/SF'
        pairs[-1][1]['pa'] = '0'
        value, proof = run(pairs)
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 3)
        self.assertEqual(sum(c['events'] for c in value['memberships']), 2)
        self.assertEqual({(type(c['pa']), c['pa']) for c in value['memberships']},
                         {(type(None), None), (int, 0)})
        self.assertEqual(value['joint_cohort_support'], proof['expected_joint_cohort'])

    def test_legacy_bins_roe_and_excluded_categories_are_not_reclassified(self):
        pairs = [ordinary(t, key=i+1) for i, t in enumerate(('GroundBall', 'Fly', 'LineDrive', 'PopUp'))]
        for i, result in enumerate(('ReachedOnError', 'Walk', 'IntentionalWalk', 'Interference'), 5):
            pair = fixture('SH', mask=1, key=i); pair[0]['result'] = pair[1]['result'] = result; pairs.append(pair)
        value, proof = run(pairs)
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 8)
        self.assertEqual(sum(c['events'] for c in value['memberships']), 4)
        self.assertEqual({c['joint_category'] for c in value['memberships']},
                         {'roe', 'walk', 'excluded_ibb', 'excluded_interference'})
        for cell in value['memberships']: self.assertIsNone(cell['contact_bin'])
        self.assertEqual(value['joint_cohort_support'], proof['expected_joint_cohort'])

    def test_training_heldout_excluded_splits_are_preserved(self):
        games = ['ANA201904040', 'ANA201904050', 'BOS201908070']
        pairs = [fixture('SH', key=i+1) for i in range(3)]
        for pair, game in zip(pairs, games): pair[0]['game_id'] = pair[1]['game_id'] = game
        args = inputs(pairs, supported=games[:2]); value = audit.inventory(*args[:3], **args[3])
        self.assertEqual({c['split'] for c in value['source_contexts']}, {'training', 'heldout', 'excluded'})
        self.assertEqual({c['split'] for c in value['memberships']}, {'training', 'heldout', 'excluded'})

    def test_direct_refinements_fold_to_every_original_weight_and_are_order_stable(self):
        pairs = [fixture('SF', key=i+1) for i in range(4)]
        for pair in pairs[:3]: pair[1]['after_base_mask'] = '1'
        args = inputs(pairs); value = audit.inventory(*args[:3], **args[3]); old = args[3]['expected_joint_cohort']
        self.assertEqual(len(value['source_contexts']), 2)
        self.assertEqual(sorted(c['events'] for c in value['source_contexts']), [1, 3])
        for weight in ('events', 'labeled_events', 'overlap_events'):
            self.assertEqual(previous._fold(value['source_contexts'], previous.SOURCE_FIELDS, weight),
                             previous._fold(old['source_contexts'], previous.SOURCE_FIELDS, weight))
        axes = tuple(k for k in old['memberships'][0] if k != 'events')
        self.assertEqual(previous._fold(value['memberships'], axes), previous._fold(old['memberships'], axes))
        self.assertEqual(value, audit.inventory(iter(reversed(args[0])), iter(args[1]), args[2], **args[3]))
        self.assertEqual(value, audit.inventory(iter(args[0]), iter(reversed(args[1])), args[2], **args[3]))

    def test_raw_null_zero_and_one_are_never_boolean_aliases(self):
        pairs = [fixture('SF', key=i+1) for i in range(3)]
        for pair, out in zip(pairs, ('', '0', '1')):
            pair[1].update(frame_end='true', after_outs=out, after_base_mask='0')
        value, _ = run(pairs)
        self.assertEqual(len(value['source_contexts']), 3)
        observed = {(type(c['after_outs']), c['after_outs']): c['raw_out_sum_relation'] for c in value['memberships']}
        self.assertEqual(observed, {(type(None), None): 'unknown', (int, 0): 'different', (int, 1): 'match'})
        self.assertTrue(all(type(c['frame_end']) is bool for c in value['memberships']))

    def test_original_inputs_and_nested_reports_oracle_are_not_mutated_or_aliased(self):
        args = inputs([fixture('SF')]); before = copy.deepcopy(args)
        value = audit.inventory(*args[:3], **args[3]); self.assertEqual(args, before)
        self.assertEqual(value['joint_cohort_support'], args[3]['expected_joint_cohort'])
        self.assertEqual(value['canonical_shapes'], args[3]['canonical_shapes'])
        value['joint_cohort_support']['source_contexts'][0]['events'] = 999
        value['canonical_shapes']['contexts'][0]['normalized_after_base_mask'] = 999
        self.assertEqual(args, before)

    def test_missing_batter_extra_child_and_unknown_effect_stay_in_complete_report(self):
        row, context = fixture('SF')
        children = [c for c in json.loads(row['runners']) if c['baserunner'] != 'Batter']
        children.append(child('First', 'EXTRA', is_advance_attempt=None))
        value, proof = run([attached((row, context), children)])
        self.assertEqual(len(value['memberships']), 1)
        cell = value['memberships'][0]
        self.assertTrue(cell['event_geometry_proxy'])
        self.assertFalse(cell['literal_after_check_reached'])
        self.assertEqual(value['joint_cohort_support'], proof['expected_joint_cohort'])
        prior = value['joint_cohort_support']['proxy_support']['literal_support']
        self.assertEqual(prior['source_totals']['extra_child_rows'], 1)
        self.assertEqual(prior['source_totals']['opportunities'], 2)


if __name__ == '__main__':
    unittest.main()
