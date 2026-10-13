"""Source participant exposure is not an official box credit or a fitted rating."""
import copy
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from audits import player_opportunities as player
from audits.outcome_inventory import FIELDS
from audits.player_opportunities import inventory
from tests.test_outcome_inventory import event

CATEGORIES = ('strikeout', 'walk', 'hbp', 'single', 'double', 'triple', 'home_run', 'roe', 'contact_out')


def row(key, result='InPlayOut', **changes):
    fields = dict(event_key=key, event_id=key, result=result)
    if result not in ('InPlayOut', 'FieldersChoice', 'SacrificeFly', 'SacrificeHit'):
        fields.update(outs=0, state_outs=0, batting_outs=0, putouts=0,
                      after_outs=0, assists=0, after_base_mask=1)
    if result in ('', 'StrikeOut', 'Walk', 'IntentionalWalk', 'HitByPitch', 'Interference'):
        fields.update(bip='', trajectory='', fielder='')
    if result == 'StrikeOut':
        fields.update(outs=1, state_outs=1, batting_outs=1, putouts=1,
                      after_outs=1, after_base_mask=0)
    if result in ('Double', 'GroundRuleDouble'):
        fields['after_base_mask'] = 2
    if result == 'Triple':
        fields['after_base_mask'] = 4
    if result in ('HomeRun', 'InsideTheParkHomeRun'):
        fields.update(runs=1, state_runs=1, after_base_mask=0,
                      runner_rows=1, runner_runs=1, runner_outs=0, unforced_runner_outs=0,
                      steals=0, cs=0, pickoffs=0, wp=0, pb=0, balk=0, unspecified=0,
                      indifference=0, error_advances=0, force_exposure=0,
                      steal_opportunities=0, extra_base_chances=0)
    if result == '':
        # Literal runner-only pickoff; the current batter has no terminal PA.
        fields.update(pa=0, batting_outs='', outs=1, state_outs=1, putouts=1,
                      after_outs=1, after_base_mask=0, base_mask=1, first='RUNNER',
                      runner_rows=1, runner_runs=0, runner_outs=1, unforced_runner_outs=0,
                      steals=0, cs=0, pickoffs=1, wp=0, pb=0, balk=0, unspecified=0,
                      indifference=0, error_advances=0, force_exposure=0,
                      steal_opportunities=0, extra_base_chances=0)
    return event(**{**fields, **changes})


def mixed():
    return [row(1, 'Walk', batter_id='A', pitcher_id='B'),
            row(2, 'StrikeOut', batter_id='A', pitcher_id='B'),
            row(3, '', batter_id='A', pitcher_id='B'),
            row(4, 'IntentionalWalk', batter_id='C', pitcher_id='A', game_id='GAME2'),
            row(5, 'Interference', batter_id='C', pitcher_id='A', game_id='GAME2')]


def profile(player_id, included):
    counts = dict.fromkeys(CATEGORIES, 0)
    if included:
        counts.update(strikeout=1, walk=1)
    return dict(player_id=player_id, events=3 if included else 2, games=1,
                selected_pa=2, non_pa=1 if included else 0, counts=counts,
                included_pa=2 if included else 0,
                exclusions=dict(excluded_ibb=0 if included else 1,
                                excluded_interference=0 if included else 1),
                zero_categories=7 if included else 9)


class PlayerOpportunityTests(unittest.TestCase):
    def test_global_cohort_retains_exclusions_and_runner_only_events(self):
        report = inventory(mixed())
        counts = dict.fromkeys(CATEGORIES, 0)
        counts.update(strikeout=1, walk=1)
        self.assertEqual(report.get('counts'), counts)
        self.assertEqual(report.get('exclusions'), dict(excluded_ibb=1, excluded_interference=1))
        self.assertEqual(report.get('outcome_totals'), dict(events=5, pa=4, non_pa=1,
                         outs=2, runs=0, putouts=2, runner_run_units=0))
        self.assertEqual(report.get('games'), 2)
        self.assertEqual(report.get('format'), '2019-player-opportunity-support-v1')

    def test_role_profiles_are_literal_source_participants_not_shared_box_credits(self):
        roles = inventory(mixed()).get('roles', {})
        self.assertEqual(roles.get('batter', {}).get('profiles'),
                         [profile('A', True), profile('C', False)])
        self.assertEqual(roles.get('pitcher', {}).get('profiles'),
                         [profile('A', False), profile('B', True)])
        # A occurs in both roles, but an exclusion as pitcher cannot become a batting PA.

    def test_zero_observed_counts_are_reported_without_probabilities(self):
        summary = inventory(mixed()).get('roles', {}).get('batter', {}).get('summary', {})
        self.assertEqual(summary.get('players'), 2)
        self.assertEqual(summary.get('zero_exposure_players'), 1)
        zeros = dict.fromkeys(CATEGORIES, 2)
        zeros.update(strikeout=1, walk=1)
        self.assertEqual(summary.get('zero_categories'), zeros)
        self.assertEqual(summary.get('exposure_bands'), [
            dict(band='0', players=1, included_pa=0),
            dict(band='1-9', players=1, included_pa=2),
            dict(band='10-49', players=0, included_pa=0),
            dict(band='50-199', players=0, included_pa=0),
            dict(band='200-499', players=0, included_pa=0),
            dict(band='500+', players=0, included_pa=0)])
        report = inventory(mixed())
        self.assertNotIn('probabilities', report)
        self.assertNotIn('ratings', report)
        self.assertNotIn('credited_batters_faced', report)

    def test_exposure_bands_include_every_fixed_boundary_and_zero_included_player(self):
        rows = [row(1, 'Interference', batter_id='P0')]
        key = 2
        for exposure in [1, 9, 10, 49, 50, 199, 200, 499, 500]:
            for _ in range(exposure):
                rows.append(row(key, batter_id=f'P{exposure}'))
                key += 1
        bands = inventory(rows).get('roles', {}).get('batter', {}).get('summary', {}).get('exposure_bands')
        self.assertEqual(bands, [dict(band='0', players=1, included_pa=0),
            dict(band='1-9', players=2, included_pa=10),
            dict(band='10-49', players=2, included_pa=59),
            dict(band='50-199', players=2, included_pa=249),
            dict(band='200-499', players=2, included_pa=699),
            dict(band='500+', players=1, included_pa=500)])

    def test_all_original_result_coarsenings_reconcile_per_role(self):
        results = ['StrikeOut', 'Walk', 'HitByPitch', 'Single', 'Double', 'GroundRuleDouble',
                   'Triple', 'HomeRun', 'InsideTheParkHomeRun', 'ReachedOnError', 'InPlayOut',
                   'FieldersChoice', 'SacrificeFly', 'SacrificeHit', 'IntentionalWalk', 'Interference']
        rows = [row(i, result) for i, result in enumerate(results, 1)]
        report = inventory(rows)
        expected = dict(strikeout=1, walk=1, hbp=1, single=1, double=2, triple=1,
                        home_run=2, roe=1, contact_out=4)
        self.assertEqual(report.get('counts'), expected)
        for role in ['batter', 'pitcher']:
            profiles = report.get('roles', {}).get(role, {}).get('profiles', [])
            self.assertEqual(len(profiles), 1)
            self.assertEqual(profiles[0]['counts'], expected)
            self.assertEqual(profiles[0]['events'], 16)
            self.assertEqual(profiles[0]['included_pa'], 14)
            self.assertEqual(profiles[0]['selected_pa'], 16)

    def test_complete_existing_validation_is_not_replaced_by_a_counting_parser(self):
        for field, value in [('batter_id', ''), ('pitcher_id', ''), ('season', '2020'),
                             ('state_outs', '0'), ('pa', '0'), ('outs', '-1')]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                inventory([row(1, **{field: value})])
        with self.assertRaises(ValueError):
            inventory([row(1), row(1)])
        with self.assertRaises(ValueError):
            inventory([])
        invalid = row(1)
        del invalid['runner_rows']
        with self.assertRaises(ValueError):
            inventory([invalid])

    def test_unknown_pa_result_is_rejected_instead_of_losing_the_opportunity(self):
        with self.assertRaisesRegex(ValueError, 'Unknown PA classification'):
            inventory([row(1, 'NotAResult')])

    def test_input_order_invariance_distinct_games_and_no_row_mutation(self):
        rows = mixed()
        rows.append(row(6, batter_id='A', pitcher_id='B', game_id='GAME2'))
        frozen = copy.deepcopy(rows)
        first = inventory(iter(rows))
        self.assertEqual(first, inventory(reversed(rows)))
        self.assertEqual(rows, frozen)
        profiles = first.get('roles', {}).get('batter', {}).get('profiles', [])
        self.assertTrue(profiles, 'role profiles required')
        self.assertEqual(profiles[0]['games'], 2)
        self.assertEqual(profiles[0]['events'], 4)


class PublicationTests(unittest.TestCase):
    def test_existing_output_is_refused_before_reading_missing_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / 'existing.json'
            output.write_bytes(b'keep exactly')
            result = subprocess.run(['python3', '-Werror', '-m', 'audits.player_opportunities',
                str(root/'missing.csv'), str(root/'missing-manifest.csv'),
                str(root/'missing-certificate.json'), str(output)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'Refusing to overwrite', result.stderr)
            self.assertEqual(result.stdout, b'')
            self.assertEqual(output.read_bytes(), b'keep exactly')

    def test_tiny_valid_source_data_cannot_publish_as_the_complete_accepted_pilot(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            events = root/'events.csv'
            with events.open('w', newline='') as stream:
                writer = csv.DictWriter(stream, fieldnames=sorted(FIELDS))
                writer.writeheader()
                writer.writerows(mixed())
            manifest = root/'manifest.csv'
            manifest.write_text('schema_version,query_version,catalog_sha256,source_url,extracted_at,attribution\n'
                'f03-spike-1,2019-filtered-extract-v1,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583,https://example.invalid/source,2026-10-12,Retrosheet\n')
            certificate = root/'counts.json'
            certificate.write_text(json.dumps(dict(format='2019-joint-contact-proxies-v1',
                counts=dict(strikeout=42823,walk=15142,hbp=1984,single=25947,double=8531,
                            triple=785,home_run=6776,roe=1470,contact_out=82245),
                exclusions=dict(excluded_ibb=753,excluded_interference=61))))
            output = root/'report.json'
            result = subprocess.run(['python3', '-Werror', '-m', 'audits.player_opportunities',
                str(events), str(manifest), str(certificate), str(output)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'Unsupported input digest', result.stderr)
            self.assertEqual(result.stdout, b'')
            self.assertFalse(output.exists())

    def test_publication_rechecks_real_input_bytes_before_creating_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root/'source', root/'output.json'
            source.write_bytes(b'original')
            before = (hashlib.sha256(source.read_bytes()).hexdigest(),)
            source.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'inputs changed'):
                player._write_report({'validated': True}, (source,), before, output)
            self.assertFalse(output.exists())

    def test_post_validation_writer_is_deterministic_and_exclusive(self):
        # This tests the writer boundary, not a certified pilot CLI success.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root/'source', root/'output.json'
            source.write_bytes(b'unchanged')
            before = (hashlib.sha256(source.read_bytes()).hexdigest(),)
            report = {'z': 0, 'a': {'observed': 0}}
            player._write_report(report, (source,), before, output)
            expected = (json.dumps(report, sort_keys=True, indent=2)+'\n').encode()
            self.assertEqual(output.read_bytes(), expected)
            with self.assertRaises(FileExistsError):
                player._write_report(report, (source,), before, output)
            self.assertEqual(output.read_bytes(), expected)


if __name__ == '__main__':
    unittest.main()
