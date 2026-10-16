"""Authenticated toy diagnostics;not actual-data scoring or native execution proof.

Isolated summaries exercise arithmetic,not whole-source trust. Toy witnesses use
existing complete synthetic fixtures;the private independent reference is separate.
"""
import copy
from fractions import Fraction
import hashlib
import itertools
import json
import math
import struct
import unittest
from unittest.mock import patch
from audits import contact_proxy_probability_diagnostics as subject
from audits import contact_proxy_probability_review as review
from audits import contact_proxy_training_counts as codec
from tests.test_contact_proxy_training_counts import source
from tests.test_contact_proxy_probability_review import witness, support
from tests.test_contact_consequence_support import fixture

CLASSES = ('SF', 'SH', 'GIDP', 'remaining')
SPLITS = ('training', 'heldout', 'excluded')
LIMITS = dict(optimized_fitting=False, model_selection=False, sampling=False,
              integration=False, causal_certification=False, predictive_validation=False,
              calibration=False, uncertainty_coverage=False)
SUMMARY_FIELDS = {'parent_events', 'response_events', 'expected_events', 'frequency',
    'frequency_residual', 'count_residual', 'brier_loss_sum', 'mean_brier',
    'nll_loss_sum', 'nll_loss_sum_bits', 'mean_nll', 'mean_nll_bits', 'undefined_reason'}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2, allow_nan=False)+'\n').encode('ascii')


def bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def pair(value):
    value = Fraction(value)
    return [value.numerator, value.denominator]


def inputs():
    raw, report = source(); sh = digest(raw); counts = codec.encode(raw, sh)
    native = witness(codec.parse(counts, sh))
    proposal = review.review(raw, sh, (counts, counts), digest(counts), native)
    parameters = encoded(proposal)
    args = (raw, sh, (counts, counts), digest(counts), (parameters, parameters),
            digest(parameters), native, digest(native))
    return args, report, proposal


def toy(mask, outs, training, observed, split='training'):
    legal = support(mask, outs); denominator = 2*sum(training)+sum(legal)
    nums = [2*c+1 if ok else 0 for c, ok in zip(training, legal)]
    proposal = dict(base_mask=mask, outs_before=outs, parent_events=sum(training),
        response_events=dict(zip(CLASSES, training)), supported=legal,
        prior_only=not sum(training), numerators=nums, denominator=denominator,
        probability_bits=[bits(a/denominator) for a in nums])
    observed_row = dict(split=split, base_mask=mask, outs_before=outs,
                        parent_events=sum(observed), response_events=dict(zip(CLASSES, observed)))
    return observed_row, proposal


class ProbabilityDiagnostics(unittest.TestCase):
    def assert_exact_summary(self, result, rows):
        self.assertTrue(SUMMARY_FIELDS <= set(result))
        n = sum(row['parent_events'] for row, _ in rows)
        observed = [sum(row['response_events'][f] for row, _ in rows) for f in CLASSES]
        expected = [sum((Fraction(p['numerators'][j], p['denominator'])*row['parent_events']
                         for row, p in rows), Fraction()) for j in range(4)]
        loss = Fraction()
        for row, p in rows:
            probabilities = [Fraction(a, p['denominator']) for a in p['numerators']]
            squared = sum(x*x for x in probabilities)
            loss += sum(row['response_events'][f]*(1-2*probabilities[j]+squared)
                        for j, f in enumerate(CLASSES))
        self.assertEqual(result['parent_events'], n)
        self.assertEqual(result['response_events'], dict(zip(CLASSES, observed)))
        self.assertEqual(result['expected_events'], list(map(pair, expected)))
        self.assertEqual(result['count_residual'], [pair(c-e) for c, e in zip(observed, expected)])
        self.assertEqual(result['brier_loss_sum'], pair(loss))
        self.assertEqual(result['mean_brier'], pair(loss/n) if n else None)
        self.assertEqual(result['frequency'], [pair(Fraction(c, n)) for c in observed] if n else [None]*4)
        self.assertEqual(result['frequency_residual'],
                         [pair((c-e)/n) for c, e in zip(observed, expected)] if n else [None]*4)
        self.assertEqual(result['undefined_reason'], None if n else 'no_observed_parent_events')
        if not n:
            self.assertIsNone(result['mean_nll']); self.assertIsNone(result['mean_nll_bits'])
            self.assertEqual(result['nll_loss_sum'], 0.0)
            self.assertEqual(result['nll_loss_sum_bits'], bits(0.0))
        else:
            self.assertTrue(math.isfinite(result['mean_nll']))
            self.assertGreaterEqual(result['mean_nll'], 0.0)
            self.assertEqual(result['mean_nll_bits'], bits(result['mean_nll']))
            self.assertEqual(result['nll_loss_sum_bits'], bits(result['nll_loss_sum']))

    def test_public_complete72_contexts_three_splits_and_unchanged_proposal(self):
        args, report, proposal = inputs(); result = subject.diagnose(*args)
        self.assertEqual(len(result.get('contexts', [])), 72)
        self.assertEqual(len(result.get('splits', [])), 3)
        self.assertEqual(set(result), {'scope', 'model_id', 'source_sha256', 'counts_sha256',
            'parameters_sha256', 'native_sha256', 'proper_scores_computed', 'proposal',
            'contexts', 'splits', 'limits'})
        self.assertEqual(result['scope'], 'geometry_proxy_retrospective_diagnostics')
        self.assertEqual(result['model_id'], proposal['model_id'])
        self.assertEqual([result[k] for k in ('source_sha256', 'counts_sha256',
                         'parameters_sha256', 'native_sha256')], [args[i] for i in (1, 3, 5, 7)])
        self.assertIs(result['proper_scores_computed'], True)
        self.assertEqual(result['proposal'], proposal); self.assertEqual(result['limits'], LIMITS)
        for source_row, row in zip(report['contexts'], result['contexts']):
            p = proposal['contexts'][source_row['base_mask']*3+source_row['outs_before']]
            self.assertEqual(set(row), SUMMARY_FIELDS | {'split', 'base_mask', 'outs_before',
                'training_parent_events', 'prior_only', 'supported', 'numerators',
                'denominator', 'probability_bits'})
            self.assertEqual([row[k] for k in ('split', 'base_mask', 'outs_before')],
                             [source_row[k] for k in ('split', 'base_mask', 'outs_before')])
            self.assertEqual(row['training_parent_events'], p['parent_events'])
            for k in ('prior_only', 'supported', 'numerators', 'denominator', 'probability_bits'):
                self.assertEqual(row[k], p[k])
            self.assert_exact_summary(row, [(source_row, p)])
        for split, row in zip(SPLITS, result['splits']):
            self.assertEqual(set(row), SUMMARY_FIELDS | {'split'}); self.assertEqual(row['split'], split)
            self.assert_exact_summary(row, [(r, proposal['contexts'][r['base_mask']*3+r['outs_before']])
                                           for r in report['contexts'] if r['split'] == split])

    def test_public_nontraining_exposure_never_refits_training_proposal(self):
        pairs = [fixture('SH', mask=1, key=i+1) for i in range(3)]
        games = ('ANA201904040', 'ANA201904050', 'BOS201908070')
        for value, game in zip(pairs, games):
            value[0]['game_id'] = value[1]['game_id'] = game
        raw, _ = source(pairs, supported=games[:2]); sh = digest(raw)
        counts = codec.encode(raw, sh); native = witness(codec.parse(counts, sh))
        proposal = review.review(raw, sh, (counts, counts), digest(counts), native)
        parameters = encoded(proposal)
        result = subject.diagnose(raw, sh, (counts, counts), digest(counts),
            (parameters, parameters), digest(parameters), native, digest(native))
        self.assertEqual(result['proposal']['training_events'], 1)
        self.assertEqual([r['parent_events'] for r in result['splits']], [1, 1, 1])
        for row in result['splits']:
            self.assertEqual(row['response_events'], dict(SF=0, SH=1, GIDP=0, remaining=0))
            self.assertEqual(row['mean_brier'], [6, 25])
            self.assertEqual(row['mean_nll'], -math.log(3/5))
        self.assertEqual(result['proposal'], proposal)

    def test_isolated_row_permutation_and_observation_scaling_preserve_means(self):
        rows = [toy(5, 0, [0, 0, 0, 0], [1, 0, 0, 0]),
                toy(5, 1, [0, 0, 0, 9], [0, 0, 0, 9]),
                toy(4, 0, [2, 0, 0, 3], [2, 0, 0, 3])]
        baseline = subject._summarize(rows)
        for ordered in itertools.permutations(rows):
            result = subject._summarize(list(ordered))
            self.assertEqual(result, baseline)
        for scale in (2, 7, 1000):
            changed = copy.deepcopy(rows)
            for row, _ in changed:
                row['parent_events'] *= scale
                row['response_events'] = {f: c*scale for f, c in row['response_events'].items()}
            result = subject._summarize(changed); self.assert_exact_summary(result, changed)
            for field in ('frequency', 'frequency_residual', 'mean_brier', 'mean_nll_bits'):
                self.assertEqual(result[field], baseline[field])
            self.assertEqual(Fraction(*result['brier_loss_sum']),
                             Fraction(*baseline['brier_loss_sum'])*scale)

    def test_authentication_returns_no_token_and_never_calls_log(self):
        args, _, _ = inputs()
        with patch('math.log', side_effect=AssertionError('Authentication must not score')):
            self.assertIsNone(subject.authenticate(*args))

    def test_every_external_authority_and_immutable_pair_boundary_refuses(self):
        args, _, _ = inputs(); variants = []
        for i in (1, 3, 5, 7):
            for bad in (None, '', 'F'*64, 'z'*64, '0'*63, True, 'b'*64):
                changed = list(args); changed[i] = bad; variants.append(changed)
        for i in (0, 6):
            changed = list(args); changed[i] = bytearray(args[i]); variants.append(changed)
        for i in (2, 4):
            for bad in (None, (), (args[i][0],), list(args[i]),
                        (args[i][0], bytearray(args[i][1])), (args[i][0], b'changed')):
                changed = list(args); changed[i] = bad; variants.append(changed)
        for index, changed in enumerate(variants):
            with self.subTest(variant=index):
                for function in (subject.authenticate, subject.diagnose):
                    with self.assertRaises(ValueError): function(*changed)

    def test_whole_nontraining_corruption_refuses_even_with_matching_forged_metadata(self):
        args, report, proposal = inputs(); changed = copy.deepcopy(report)
        row = next(r for r in changed['contexts'] if r['split'] == 'heldout')
        row['parent_events'] += 1; row['response_events']['remaining'] += 1
        raw = encoded(changed); sh = digest(raw)
        counts = args[2][0].replace(args[1].encode(), sh.encode())
        native = args[6].replace(args[1].encode(), sh.encode())
        forged = copy.deepcopy(proposal); forged.update(source_sha256=sh,
            counts_sha256=digest(counts), native_sha256=digest(native))
        parameters = encoded(forged)
        bad = (raw, sh, (counts, counts), digest(counts), (parameters, parameters),
               digest(parameters), native, digest(native))
        with patch('math.log', side_effect=AssertionError('Poison must refuse before scoring')):
            for function in (subject.authenticate, subject.diagnose):
                with self.assertRaises(ValueError): function(*bad)

    def test_parameter_canonical_bytes_and_complete_native_witness_cannot_self_authorize(self):
        args, _, proposal = inputs()
        changed = copy.deepcopy(proposal)
        changed['contexts'][0]['prior_only'] = not changed['contexts'][0]['prior_only']
        for raw in (encoded(changed), args[4][0]+b' ', b'\xef\xbb\xbf'+args[4][0], b'{}\n'):
            bad = list(args); bad[4] = (raw, raw); bad[5] = digest(raw)
            for function in (subject.authenticate, subject.diagnose):
                with self.subTest(size=len(raw), function=function.__name__):
                    with self.assertRaises(ValueError): function(*bad)
        for native in (args[6]+b'\n', args[6][:-1], args[6].replace(b'\t0\t', b'\t1\t', 1)):
            bad = list(args); bad[6] = native; bad[7] = digest(native)
            with self.assertRaises(ValueError): subject.diagnose(*bad)

    def test_isolated_summary_unequal_exposure_and_nulls_are_not_mean_of_means(self):
        rows = [toy(5, 0, [0, 0, 0, 0], [1, 0, 0, 0]),
                toy(5, 1, [0, 0, 0, 9], [0, 0, 0, 9])]
        result = subject._summarize(rows); self.assert_exact_summary(result, rows)
        expected = math.fsum([.1*(-math.log(.25)), .9*(-math.log(19/22))])
        self.assertEqual(result['mean_nll'], expected)
        self.assertNotEqual(result['mean_nll'], (-math.log(.25)-math.log(19/22))/2)
        self.assert_exact_summary(subject._summarize([toy(5, 0, [0]*4, [0]*4)]),
                                  [toy(5, 0, [0]*4, [0]*4)])

    def test_isolated_small1716_supported_partitions_retain_exact_rationals(self):
        cases = 0
        for mask in range(8):
            for outs in range(3):
                selected = [i for i, ok in enumerate(support(mask, outs)) if ok]
                for n in range(9):
                    for prefix in itertools.product(range(n+1), repeat=len(selected)-1):
                        if sum(prefix) > n: continue
                        values = [0]*4
                        for i, v in zip(selected, (*prefix, n-sum(prefix))): values[i] = v
                        rows = [toy(mask, outs, values, values)]
                        with self.subTest(mask=mask, outs=outs, counts=values):
                            self.assert_exact_summary(subject._summarize(rows), rows)
                        cases += 1
        self.assertEqual(cases, 1716)

    def test_isolated_cap_rotations_and_structural_positive_refusal_before_log(self):
        for mask, outs, n in itertools.product((0, 4, 5), (0, 1, 2),
                (1, 2, 8, 32768, 65536, 1073741824, 2147483647)):
            for i, ok in enumerate(support(mask, outs)):
                if not ok: continue
                counts = [0]*4; counts[i] = n; rows = [toy(mask, outs, counts, counts)]
                with self.subTest(mask=mask, outs=outs, count=n, response=i):
                    self.assert_exact_summary(subject._summarize(rows), rows)
        for split in SPLITS:
            bad = [toy(0, 0, [0, 0, 0, 1], [1, 0, 0, 0], split)]
            with patch('math.log', side_effect=AssertionError('Structural contradiction before log')):
                with self.assertRaises(ValueError): subject._summarize(bad)

    def test_nonfinite_or_negative_primary_log_results_refuse(self):
        args, _, _ = inputs()
        for value in (math.nan, math.inf, -math.inf, 1.0):
            with self.subTest(log_result=value), patch('math.log', return_value=value):
                with self.assertRaises(ValueError): subject.diagnose(*args)

    def test_repeat_returns_fresh_containers_and_preserves_original_byte_inputs(self):
        args, _, _ = inputs(); saved = copy.deepcopy(args); first = subject.diagnose(*args)
        self.assertEqual(encoded(first), encoded(subject.diagnose(*args)))
        self.assertEqual(args, saved)
        self.assertIn('proposal', first); self.assertIn('contexts', first)
        first['proposal']['contexts'][0]['parent_events'] += 1
        first['contexts'].clear()
        self.assertEqual(len(subject.diagnose(*args)['contexts']), 72)
        self.assertEqual(args, saved)


if __name__ == '__main__':
    unittest.main(verbosity=2)
