"""Original real-event grain versus family memberships;never causal/rate proof."""
import copy
import hashlib
import itertools
import json
import unittest
from unittest.mock import patch
from audits import contact_proxy_joint_cohort as audit
from audits import contact_proxy_qualification as proxy
from audits import contact_consequence_support as literal
from audits import joint_contact as joint
from audits import runner_transitions as runner
from tests.test_contact_proxy_qualification import evidence, no_play_control, source_pair
from tests.test_contact_consequence_support import fixture
from tests.test_joint_contact import fixture as ordinary
from tests.test_runner_only_steals import fixture as runner_only
from tests.test_runner_transitions import attached, child


def inputs(pairs, supported=None):
    rows, outcomes, games, proof = evidence(pairs)
    if supported is not None:
        games = list(supported)
        census = runner.inventory(rows, outcomes, games)
        groups = {json.dumps({k: v for k, v in cell.items() if k != 'rows'}, sort_keys=True):
                  cell['rows'] for cell in census['raw_child_cells']}
        cohort = {name: copy.deepcopy(census[name]) for name in runner.COHORT}
        cohort['raw_totals'] = {name: census['raw_totals'][name] for name in runner.COHORT['raw_totals']}
        proof.update(native_groups=groups, expected_census=census, cohort=cohort)
        proof['expected_support'] = literal.inventory(rows, outcomes, games,
            **{name: proof[name] for name in ('native_groups', 'expected_census', 'native_joint', 'cohort')})
    proof['expected_proxy'] = proxy.inventory(rows, outcomes, games, **proof)
    proof['expected_joint'] = joint.inventory(rows, outcomes)
    return rows, outcomes, games, proof


def run(pairs, supported=None):
    rows, outcomes, games, proof = inputs(pairs, supported)
    return audit.inventory(rows, outcomes, games, **proof), proof


class JointCohort(unittest.TestCase):
    def test_source_events_and_memberships_conserve_complete_prior_reports(self):
        value, proof = run([fixture(f, key=i+1) for i, f in enumerate(('SF', 'SH', 'GIDP'))])
        self.assertEqual(value['format'], '2019-contact-proxy-joint-cohort-v1')
        self.assertEqual(value['causal_certification'], 'unavailable')
        self.assertEqual(value['proxy_support'], proof['expected_proxy'])
        self.assertEqual(value['joint_support'], proof['expected_joint'])
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 3)
        self.assertEqual(sum(c['labeled_events'] for c in value['source_contexts']), 3)
        self.assertEqual(sum(c['overlap_events'] for c in value['source_contexts']), 0)
        self.assertEqual(sum(c['events'] for c in value['memberships']), 3)

    def test_geometry_matches_keep_exact_existing_residual_first_reasons(self):
        value, proof = run([fixture(f, key=i+1) for i, f in enumerate(('SF', 'SH', 'GIDP'))])
        self.assertEqual(len(value['memberships']), 3)
        expected = dict(SF='non_single_scoreless_out', SH='runner_effect_or_unknown',
                        GIDP='non_single_scoreless_out')
        original = {c['family']: c for c in proof['expected_proxy']['contexts']}
        for cell in value['memberships']:
            with self.subTest(family=cell['family']):
                self.assertIs(cell['event_geometry_proxy'], True)
                self.assertEqual((cell['joint_category'], cell['contact_bin'], cell['residual_reason']),
                                 ('contact_out', 'residual', expected[cell['family']]))
                self.assertEqual(cell['literal_shape'], original[cell['family']]['shape'])
                self.assertEqual(cell['literal_reasons'], original[cell['family']]['reasons'])

    def test_conflicting_labels_are_two_memberships_but_one_distinct_source_event(self):
        pair = fixture('SH', mask=1)
        pair[0]['raw_play'] = '13/SH/SF'
        value, _ = run([pair])
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 1)
        self.assertEqual(sum(c['labeled_events'] for c in value['source_contexts']), 1)
        self.assertEqual(sum(c['overlap_events'] for c in value['source_contexts']), 1)
        self.assertEqual(sum(c['events'] for c in value['memberships']), 2)
        self.assertEqual({c['family'] for c in value['memberships']}, {'SF', 'SH'})
        for cell in value['memberships']:
            self.assertIs(cell['event_geometry_proxy'], False)
            self.assertIn('conflicting_labels', cell['literal_reasons'])

    def test_no_play_labels_remain_in_prior_report_not_new_event_units(self):
        value, proof = run(no_play_control())
        self.assertEqual(value['proxy_support'], proof['expected_proxy'])
        self.assertEqual(value['proxy_support']['literal_support']['no_play_label_counts']['SF'], 1)
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 1)
        self.assertEqual(sum(c['labeled_events'] for c in value['source_contexts']), 0)
        self.assertEqual(value['memberships'], [])

    def test_all_ordinary_candidate_bins_are_source_frames_not_family_memberships(self):
        value, _ = run([ordinary(tag, key=i+1) for i, tag in
                        enumerate(('GroundBall', 'Fly', 'LineDrive', 'PopUp'))])
        self.assertEqual(len(value['source_contexts']), 4)
        self.assertEqual({c['contact_bin'] for c in value['source_contexts']},
                         set(joint.BINS)-{'residual'})
        self.assertTrue(all(c['residual_reason'] == '' for c in value['source_contexts']))
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 4)
        self.assertEqual(value['memberships'], [])

    def test_every_reconciliation_evidence_object_is_required_before_projection(self):
        rows, outcomes, games, proof = inputs([fixture('SH')])
        for name in proof:
            with self.subTest(name=name):
                changed = dict(proof, **{name: None})
                with patch.object(audit, '_project', create=True,
                                  side_effect=AssertionError('projection started')):
                    with self.assertRaises(ValueError):
                        audit.inventory(rows, outcomes, games, **changed)

    def test_whole_accepted_baselines_are_type_strict_and_nonfinite_is_refused(self):
        rows, outcomes, games, proof = inputs([fixture('SH')])
        for name, path in [('expected_support', ('families', 'SH', 'events')),
                           ('expected_proxy', ('families', 'SH', 'events')),
                           ('expected_joint', ('counts', 'contact_out'))]:
            for alias in (True, 1.0, float('nan')):
                with self.subTest(name=name, alias=alias):
                    changed = copy.deepcopy(proof)
                    target = changed[name]
                    for key in path[:-1]:
                        target = target[key]
                    target[path[-1]] = alias
                    with patch.object(audit, '_project', create=True,
                                      side_effect=AssertionError('projection started')):
                        with self.assertRaises(ValueError):
                            audit.inventory(rows, outcomes, games, **changed)

    def test_native_mismatch_cannot_reach_new_projection(self):
        rows, outcomes, games, proof = inputs([fixture('GIDP')])
        changed = copy.deepcopy(proof)
        changed['native_joint']['1']['event_runs'] = 1
        with patch.object(audit, '_project', create=True,
                          side_effect=AssertionError('projection started')):
            with self.assertRaises(ValueError):
                audit.inventory(rows, outcomes, games, **changed)

    def test_all_supported_source_masks_outs_have_the_original_four_axis_descriptor(self):
        cases = [(f, mask, outs) for f, mask, outs in
                 itertools.product(('SF', 'SH', 'GIDP'), range(8), range(3))
                 if f != 'GIDP' or outs < 2]
        self.assertEqual(len(cases), 64)
        for family, mask, outs in cases:
            with self.subTest(family=family, mask=mask, outs=outs):
                pair = source_pair(family, mask, outs)
                value, _ = run([pair])
                self.assertEqual(len(value['memberships']), 1)
                expected = (mask in ((4, 5, 6, 7) if family == 'SF' else (1, 5))
                            and outs < 2 and pair[0]['runs'] == str(int(family == 'SF')))
                self.assertIs(value['memberships'][0]['event_geometry_proxy'], expected)
                self.assertEqual(sum(c['events'] for c in value['source_contexts']), 1)

    def test_non_pa_labels_have_no_contact_bin_and_keep_null_distinct_from_zero(self):
        pairs = [runner_only(key=1), runner_only(key=2)]
        for row, context in pairs:
            row['raw_play'] = 'SB2/SF'
        pairs[1][1]['pa'] = '0'
        value, proof = run(pairs)
        self.assertEqual(len(value['source_contexts']), 2)
        self.assertEqual({(type(c['pa']), c['pa']) for c in value['source_contexts']},
                         {(type(None), None), (int, 0)})
        self.assertTrue(all(c['result'] is None and c['joint_category'] == 'non_pa'
                            and c['contact_bin'] is None and c['residual_reason'] is None
                            for c in value['source_contexts']))
        self.assertEqual(sum(c['events'] for c in value['memberships']), 2)
        self.assertTrue(all(not c['event_geometry_proxy'] and
                            'not_terminal_pa' in c['literal_reasons'] for c in value['memberships']))
        self.assertEqual(value['proxy_support'], proof['expected_proxy'])

    def test_labeled_noncontact_and_original_exclusions_are_not_silently_contact_out(self):
        pairs = [fixture('SH', mask=1, key=i+1) for i in range(3)]
        for pair, result in zip(pairs, ('Walk', 'IntentionalWalk', 'Interference')):
            pair[0]['result'] = pair[1]['result'] = result
        value, _ = run(pairs)
        self.assertEqual(len(value['memberships']), 3)
        self.assertEqual({c['joint_category'] for c in value['memberships']},
                         {'walk', 'excluded_ibb', 'excluded_interference'})
        self.assertTrue(all(c['contact_bin'] is None and c['residual_reason'] is None
                            for c in value['memberships']))
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 3)

    def test_training_heldout_and_excluded_game_memberships_use_unchanged_split(self):
        games = ['ANA201904040', 'ANA201904050', 'BOS201908070']
        self.assertNotIn(hashlib.sha256(games[0].encode()).hexdigest()[0], '012')
        self.assertIn(hashlib.sha256(games[1].encode()).hexdigest()[0], '012')
        pairs = [fixture('SH', key=i+1) for i in range(3)]
        for pair, game in zip(pairs, games):
            pair[0]['game_id'] = pair[1]['game_id'] = game
        value, proof = run(pairs, supported=games[:2])
        self.assertEqual({c['split'] for c in value['source_contexts']},
                         {'training', 'heldout', 'excluded'})
        self.assertEqual({c['split'] for c in value['memberships']},
                         {'training', 'heldout', 'excluded'})
        self.assertEqual(value['proxy_support'], proof['expected_proxy'])

    def test_weighted_duplicate_shapes_and_independent_iterable_order_are_exact(self):
        pairs = [fixture('SF', key=i+1) for i in range(3)]
        rows, outcomes, games, proof = inputs(pairs)
        value = audit.inventory(rows, outcomes, games, **proof)
        self.assertEqual(len(value['source_contexts']), 1)
        self.assertEqual(value['source_contexts'][0]['events'], 3)
        self.assertEqual(value['source_contexts'][0]['labeled_events'], 3)
        self.assertEqual(len(value['memberships']), 1)
        self.assertEqual(value['memberships'][0]['events'], 3)
        self.assertEqual(value, audit.inventory(iter(reversed(rows)), iter(outcomes), games, **proof))
        self.assertEqual(value, audit.inventory(iter(rows), iter(reversed(outcomes)), games, **proof))

    def test_source_and_baselines_are_not_mutated_or_aliased_by_output(self):
        args = inputs([fixture('SF')])
        before = copy.deepcopy(args)
        value = audit.inventory(*args[:3], **args[3])
        self.assertEqual(args, before)
        value['proxy_support']['families']['SF']['events'] = 999
        value['joint_support']['counts']['contact_out'] = 999
        self.assertEqual(args, before)

    def test_missing_batter_extra_native_child_and_unknown_flag_are_preserved(self):
        row, context = fixture('SF')
        children = [c for c in json.loads(row['runners']) if c['baserunner'] != 'Batter']
        children.append(child('First', 'EXTRA', is_advance_attempt=None))
        pair = attached((row, context), children)
        value, proof = run([pair])
        self.assertEqual(value['proxy_support'], proof['expected_proxy'])
        prior = value['proxy_support']['literal_support']
        self.assertEqual(prior['source_totals']['extra_child_rows'], 1)
        self.assertEqual(prior['source_totals']['opportunities'], 2)
        self.assertEqual(sum(c['events'] for c in value['source_contexts']), 1)
        self.assertEqual(sum(c['events'] for c in value['memberships']), 1)
        self.assertIs(value['memberships'][0]['event_geometry_proxy'], True)
        self.assertIn('incomplete_vector', value['memberships'][0]['literal_reasons'])
        original = value['proxy_support']['contexts'][0]['runners']
        self.assertEqual({c['baserunner'] for c in original}, {'First', 'Third'})
        self.assertIsNone(next(c for c in original if c['baserunner'] == 'First')['is_advance_attempt'])

    def test_late_nested_baseline_mismatches_fail_before_projection_not_only_marginals(self):
        rows, outcomes, games, proof = inputs([fixture('SF')])
        changes = []
        changed = copy.deepcopy(proof)
        changed['expected_proxy']['literal_support']['before_state_pa_frames'][0]['frames'] += 1
        changes.append(changed)
        changed = copy.deepcopy(proof)
        changed['expected_joint']['examples'][0]['event']['raw_play'] = 'not the original example'
        changes.append(changed)
        for changed in changes:
            with self.subTest(name=next(k for k in proof if changed[k] != proof[k])):
                with patch.object(audit, '_project', create=True,
                                  side_effect=AssertionError('projection started')):
                    with self.assertRaises(ValueError):
                        audit.inventory(rows, outcomes, games, **changed)

    def test_duplicates_join_disagreement_and_missing_native_map_fail_before_projection(self):
        rows, outcomes, games, proof = inputs([fixture('SH')])
        changed = copy.deepcopy(outcomes)
        changed[0]['angle'] = 'Changed'
        missing_native = dict(proof, native_joint={})
        for r, o, evidence_map in [(rows+rows, outcomes, proof),
                                   (rows, outcomes+outcomes, proof),
                                   (rows, changed, proof), (rows, outcomes, missing_native)]:
            with self.subTest(rows=len(r), outcomes=len(o), native=len(evidence_map['native_joint'])):
                with patch.object(audit, '_project', create=True,
                                  side_effect=AssertionError('projection started')):
                    with self.assertRaises(ValueError):
                        audit.inventory(r, o, games, **evidence_map)

    def test_legacy_publication_provenance_is_preserved_but_never_republished_as_fresh(self):
        rows, outcomes, games, proof = inputs([fixture('SH')])
        proof['expected_joint']['provenance'] = dict(marker='original accepted publication')
        original = copy.deepcopy(proof)
        value = audit.inventory(rows, outcomes, games, **proof)
        self.assertEqual(value['joint_support'], original['expected_joint'])
        self.assertEqual(proof, original)
        proof['expected_joint']['provenance']['invalid'] = float('nan')
        with patch.object(audit, '_project', create=True,
                          side_effect=AssertionError('projection started')):
            with self.assertRaises(ValueError):
                audit.inventory(rows, outcomes, games, **proof)
