"""Distinct residual-parent response counts,not fitted rates or causal facts."""
import copy
import itertools
import unittest
from unittest.mock import patch
from audits import contact_proxy_response_counts as audit
from audits import contact_proxy_observable_targets as previous
from audits import contact_proxy_joint_cohort as cohort_rules
from tests.test_contact_proxy_observable_targets import inputs as inherited_inputs
from tests.test_contact_consequence_support import fixture
from tests.test_contact_proxy_qualification import no_play_control, source_pair
from tests.test_joint_contact import fixture as ordinary

RESPONSES = ('SF', 'SH', 'GIDP', 'remaining')
SPLITS = ('training', 'heldout', 'excluded')


def inputs(pairs, supported=None):
    rows, outcomes, games, proof = inherited_inputs(pairs, supported)
    proof['expected_observable_targets'] = previous.inventory(rows, outcomes, games, **proof)
    return rows, outcomes, games, proof


def run(pairs, supported=None):
    args = inputs(pairs, supported)
    return audit.inventory(*args[:3], **args[3]), args[3]


class ResponseCounts(unittest.TestCase):
    def test_three_geometry_classes_partition_one_parent_event_each(self):
        pairs = [fixture(family, key=i+1) for i, family in enumerate(RESPONSES[:3])]
        value, _ = run(pairs)
        self.assertEqual(value['totals']['parent_events'], 3)
        self.assertEqual(value['totals']['response_events'], dict(SF=1, SH=1, GIDP=1, remaining=0))
        self.assertEqual({c['response'] for c in value['source_responses']}, set(RESPONSES[:3]))

    def test_complete_observable_report_remains_unchanged(self):
        value, proof = run([fixture('SH')])
        self.assertEqual(value['format'], '2019-contact-proxy-response-counts-v1')
        self.assertEqual(value['observable_support'], proof['expected_observable_targets'])
        self.assertEqual(value['causal_certification'], 'unavailable')
        self.assertEqual(value['historical_canonical_eligibility'], 'unavailable')

    def test_context_domain_is72_known_cells_not_estimated_rates(self):
        value, _ = run([fixture('SH')])
        expected = [(split, mask, outs) for split in SPLITS for mask in range(8) for outs in range(3)]
        self.assertEqual([(r['split'], r['base_mask'], r['outs_before']) for r in value['contexts']], expected)
        for row in value['contexts']:
            with self.subTest(context=(row['split'], row['base_mask'], row['outs_before'])):
                self.assertEqual(set(row), {'split', 'base_mask', 'outs_before', 'parent_events', 'response_events'})
                self.assertIs(type(row['parent_events']), int)
                self.assertEqual(tuple(row['response_events']), RESPONSES)
                self.assertTrue(all(type(n) is int and n >= 0 for n in row['response_events'].values()))
                self.assertEqual(row['parent_events'], sum(row['response_events'].values()))
        self.assertEqual(sum(r['parent_events'] for r in value['contexts']), 1)

    def test_unlabeled_and_nongeometry_residuals_are_in_parent_denominator(self):
        pairs = [fixture('SF', key=1), no_play_control()[1], fixture('SH', mask=0, key=3)]
        value, _ = run(pairs)
        self.assertEqual(value['totals']['parent_events'], 3)
        self.assertEqual(value['totals']['response_events'], dict(SF=1, SH=0, GIDP=0, remaining=2))
        self.assertEqual(value['totals']['labeled_parent_events'], 2)
        self.assertEqual(value['totals']['overlap_parent_events'], 0)

    def test_conflicting_memberships_are_one_remaining_source_event(self):
        pair = fixture('SH', mask=1); pair[0]['raw_play'] = '13/SH/SF'
        value, proof = run([pair])
        self.assertEqual(len(proof['expected_observable_targets']['memberships']), 2)
        self.assertEqual(value['totals']['parent_events'], 1)
        self.assertEqual(value['totals']['response_events'], dict(SF=0, SH=0, GIDP=0, remaining=1))
        self.assertEqual(value['totals']['labeled_parent_events'], 1)
        self.assertEqual(value['totals']['overlap_parent_events'], 1)
        self.assertEqual(len(value['source_responses']), 1)
        self.assertEqual({k: value['source_responses'][0][k] for k in
            ('response', 'events', 'labeled_events', 'overlap_events')},
            dict(response='remaining', events=1, labeled_events=1, overlap_events=1))

    def test_no_play_labels_never_enter_parent_counts(self):
        value, proof = run(no_play_control())
        self.assertEqual(value['totals']['parent_events'], 1)
        self.assertEqual(value['totals']['response_events'], dict(SF=0, SH=0, GIDP=0, remaining=1))
        self.assertEqual(value['observable_support'], proof['expected_observable_targets'])

    def test_every_supported_and_unsupported_before_context_retains_parent(self):
        for family, mask, outs in itertools.product(RESPONSES[:3], range(8), (0, 1)):
            with self.subTest(family=family, mask=mask, outs=outs):
                value, proof = run([source_pair(family, mask, outs)])
                old = proof['expected_observable_targets']
                expected = family if old['memberships'][0]['event_geometry_proxy'] else 'remaining'
                self.assertEqual(value['totals']['parent_events'], 1)
                self.assertEqual(value['totals']['response_events'],
                    {name: int(name == expected) for name in RESPONSES})
                active = [c for c in value['contexts'] if c['parent_events']]
                self.assertEqual(len(active), 1)
                self.assertEqual((active[0]['base_mask'], active[0]['outs_before']), (mask, outs))

    def test_descriptor_differences_do_not_select_the_count_population(self):
        pairs = [fixture('SF', key=i+1) for i in range(2)]
        pairs[0][1]['after_base_mask'] = '1'
        value, proof = run(pairs)
        self.assertEqual({c['controlled_mask_relation'] for c in
            proof['expected_observable_targets']['memberships']}, {'match', 'different'})
        self.assertEqual(value['totals']['response_events'], dict(SF=2, SH=0, GIDP=0, remaining=0))
        self.assertEqual(len(value['source_responses']), 2)

    def test_third_out_boundaries_are_counted_without_mask_comparison(self):
        value, proof = run([fixture('GIDP', mask=1, outs=1)])
        self.assertEqual(proof['expected_observable_targets']['memberships'][0]
            ['controlled_mask_relation'], 'not_compared_boundary')
        self.assertEqual(value['totals']['response_events'], dict(SF=0, SH=0, GIDP=1, remaining=0))
        self.assertIs(value['source_responses'][0]['frame_end'], True)
        self.assertEqual(value['source_responses'][0]['after_outs'], 3)
        self.assertIsNone(value['source_responses'][0]['after_base_mask'])
        for name in cohort_rules.SOURCE_FIELDS + previous.AFTER_FIELDS:
            self.assertEqual(value['source_responses'][0][name],
                proof['expected_observable_targets']['source_contexts'][0][name])

    def test_all_three_weights_fold_to_complete_parent_source_cells(self):
        pairs = [fixture('SF', key=i+1) for i in range(3)]
        pairs[0][1]['after_base_mask'] = '1'
        conflict = fixture('SH', key=4); conflict[0]['raw_play'] = '13/SH/SF'
        value, proof = run(pairs+[conflict, fixture('SH', mask=0, key=5)])
        axes = cohort_rules.SOURCE_FIELDS + previous.AFTER_FIELDS
        parent = [c for c in proof['expected_observable_targets']['source_contexts']
            if c['joint_category'] == 'contact_out' and c['contact_bin'] == 'residual']
        for weight in ('events', 'labeled_events', 'overlap_events'):
            self.assertEqual(cohort_rules._fold(value['source_responses'], axes, weight),
                cohort_rules._fold(parent, axes, weight))
        self.assertEqual(sum(c['parent_events'] for c in value['contexts']), 5)

    def test_original_splits_have_separate_complete_context_tables(self):
        games = ['ANA201904040', 'ANA201904050', 'BOS201908070']
        pairs = [fixture('SH', key=i+1) for i in range(3)]
        for pair, game in zip(pairs, games):
            pair[0]['game_id'] = pair[1]['game_id'] = game
        value, _ = run(pairs, games[:2])
        for split in SPLITS:
            rows = [c for c in value['contexts'] if c['split'] == split]
            self.assertEqual(len(rows), 24)
            self.assertEqual(sum(c['parent_events'] for c in rows), 1)
            self.assertEqual(sum(c['response_events']['SH'] for c in rows), 1)

    def test_other_legacy_bins_are_outside_parent_not_remaining(self):
        value, proof = run([ordinary('GroundBall', mask=0, outs=0)])
        self.assertNotEqual(proof['expected_observable_targets']['source_contexts'][0]['contact_bin'], 'residual')
        self.assertEqual(value['totals']['parent_events'], 0)
        self.assertEqual(value['source_responses'], [])
        self.assertEqual(len(value['contexts']), 72)
        self.assertTrue(all(c['parent_events'] == 0 for c in value['contexts']))

    def test_two_out_parent_rows_stay_remaining_not_dropped(self):
        value, _ = run([fixture('SF', outs=2, key=1), fixture('SH', mask=1, outs=2, key=2)])
        self.assertEqual(value['totals']['parent_events'], 2)
        self.assertEqual(value['totals']['response_events'], dict(SF=0, SH=0, GIDP=0, remaining=2))
        self.assertEqual(sum(c['parent_events'] for c in value['contexts'] if c['outs_before'] == 2), 2)

    def test_input_order_generators_and_caller_objects_are_preserved(self):
        args = inputs([fixture(f, key=i+1) for i, f in enumerate(RESPONSES[:3])])
        original = copy.deepcopy(args)
        value = audit.inventory(*args[:3], **args[3])
        self.assertEqual(value, audit.inventory(iter(reversed(args[0])), iter(args[1]), args[2], **args[3]))
        self.assertEqual(value, audit.inventory(iter(args[0]), iter(reversed(args[1])), args[2], **args[3]))
        self.assertEqual(args, original)
        self.assertEqual(value['observable_support'], original[3]['expected_observable_targets'])
        value['observable_support']['source_contexts'][0]['events'] += 99
        self.assertEqual(args, original)

    def test_all_evidence_is_mandatory_before_new_projection(self):
        args = inputs([fixture('SH')])
        for name in args[3]:
            with self.subTest(missing=name):
                proof = dict(args[3]); proof[name] = None
                with patch.object(audit, '_project', create=True) as counter:
                    with self.assertRaises(ValueError):
                        audit.inventory(*args[:3], **proof)
                    counter.assert_not_called()

    def test_whole_observable_baseline_must_be_exact_before_projection(self):
        args = inputs([fixture('SH')])
        for bad in ({}, [], 'source', 1):
            with self.subTest(bad=bad):
                proof = copy.deepcopy(args[3]); proof['expected_observable_targets'] = bad
                with patch.object(audit, '_project', create=True) as counter:
                    with self.assertRaises(ValueError):
                        audit.inventory(*args[:3], **proof)
                    counter.assert_not_called()
        proof = copy.deepcopy(args[3])
        proof['expected_observable_targets']['source_contexts'][-1]['events'] += 1
        with patch.object(audit, '_project', create=True) as counter:
            with self.assertRaisesRegex(ValueError, 'observable.*baseline'):
                audit.inventory(*args[:3], **proof)
            counter.assert_not_called()

    def test_bool_float_nonfinite_baseline_aliases_are_not_integer_evidence(self):
        args = inputs([fixture('SH')])
        for alias in (True, 1.0, float('nan'), float('inf')):
            with self.subTest(alias=repr(alias)):
                proof = copy.deepcopy(args[3])
                proof['expected_observable_targets']['source_contexts'][0]['events'] = alias
                with patch.object(audit, '_project', create=True) as counter:
                    with self.assertRaises(ValueError):
                        audit.inventory(*args[:3], **proof)
                    counter.assert_not_called()

    def test_canonical_type_and_native_census_failures_still_refuse(self):
        args = inputs([fixture('SH')])
        for name in ('canonical_shapes', 'native_joint', 'expected_census'):
            with self.subTest(name=name):
                proof = copy.deepcopy(args[3])
                if name == 'canonical_shapes':
                    proof[name]['contexts'][-1]['delta_outs'] = True
                elif name == 'native_joint':
                    next(iter(proof[name].values()))['after_outs'] = 2
                else:
                    proof[name]['totals']['real_events'] += 1
                with patch.object(audit, '_project', create=True) as counter:
                    with self.assertRaises(ValueError):
                        audit.inventory(*args[:3], **proof)
                    counter.assert_not_called()


if __name__ == '__main__':
    unittest.main()
