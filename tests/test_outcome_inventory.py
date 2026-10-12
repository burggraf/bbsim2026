"""Outcome audits must preserve unknowns, event grain and supplied accounting."""
import copy
import itertools
from pathlib import Path
import subprocess
import tempfile
import unittest
from audits.outcome_inventory import inventory, read_manifest
import csv
import io



def event(**changes):
    row = dict(event_key='1', game_id='GAME1', event_id='1', season='2019',
               result='InPlayOut', pa='1', trajectory='GroundBall', location='Shortstop',
               depth='Default', angle='Default', fielder='6', derived_known='1', derived_unknown='0',
               derived_fly='0', derived_ground='1', derived_line='0', derived_pop='0',
               derived_bunts='0', derived_location_known='1', derived_fielder_known='1', bip='1',
               outs_before='0', base_mask='0', outs='1', runs='0',
               state_outs='1', state_runs='0', batting_outs='1', putouts='1',
               assists='1', errors='0', dp='0', tp='0', gidp='0',
               frame_end='false', after_outs='1', after_base_mask='0',
               batter_id='BATTER', pitcher_id='PITCHER', first='', second='', third='',
               personnel_key='42', runner_rows='0', runner_runs='', runner_outs='',
               unforced_runner_outs='', steals='', cs='', pickoffs='', wp='', pb='',
               balk='', unspecified='', indifference='', error_advances='',
               force_exposure='', steal_opportunities='', extra_base_chances='')
    if changes.get('bip') == '':
        row.update({key: '' for key in row if key.startswith('derived_')})
    row.update({key: str(value) for key, value in changes.items()})
    return row


class OutcomeInventoryTests(unittest.TestCase):
    def test_accounts_for_every_event_result_and_out_multiplicity(self):
        rows = [event(), event(event_key=2, event_id=2, result='StrikeOut', trajectory='',
                              bip='', fielder='', outs=0,
                              state_outs=0, batting_outs=0, putouts=0, after_outs=0),
                event(event_key=3, event_id=3, result='', pa='', batting_outs='',
                      trajectory='', bip='', fielder='', outs=2,
                      state_outs=2, putouts=2, after_outs=2, dp=1)]
        report = inventory(rows)
        self.assertEqual(report.get('totals'), dict(events=3, pa=2, non_pa=1, outs=3,
                                                   runs=0, putouts=3, runner_run_units=0))
        self.assertEqual(report.get('results'), {'<non-PA>': 1, 'InPlayOut': 1, 'StrikeOut': 1})
        self.assertEqual(report.get('out_multiplicity'), {'0': 1, '1': 1, '2': 1})
        self.assertEqual(report.get('signals', {}).get('strikeout_events_without_out'), 1)
        self.assertEqual(report.get('signals', {}).get('double_play_events'), 1)
        self.assertEqual(report.get('coverage', {}).get('events_without_runner_rows'), 3)

    def test_unknown_raw_contact_is_not_fabricated_from_derived_tags(self):
        rows = [event(trajectory='Unknown', derived_ground=0, derived_fly=1, location='Unknown', fielder=''),
                event(event_key=2, event_id=2, trajectory='', derived_known=0, derived_unknown=1,
                      derived_ground=0, location='', fielder='')]
        report = inventory(rows)
        self.assertEqual(report.get('raw_trajectories'), {'<missing>': 1, 'Unknown': 1})
        self.assertEqual(report.get('derived_contact_flags', {}).get('derived_fly'), 1)
        self.assertEqual(report.get('derived_contact_flags', {}).get('derived_unknown'), 1)
        self.assertEqual(report.get('coverage', {}).get('bip_raw_trajectory_unknown'), 2)
        self.assertEqual(report.get('coverage', {}).get('bip_derived_known_raw_unknown'), 1)
        self.assertEqual(report.get('coverage', {}).get('bip_raw_location_unknown'), 2)
        self.assertEqual(report.get('coverage', {}).get('bip_fielder_unknown'), 2)
        known_fielder = inventory([event(trajectory='Unknown')])
        self.assertEqual(known_fielder['coverage']['bip_fielder_unknown'], 0)

    def test_recorded_foul_angle_is_preserved_without_fabricating_ball_status(self):
        rows = [event(event_key=i, event_id=i, angle=tag) for i, tag in
                enumerate(['Foul', 'FoulLine', 'Default', 'Unknown'], 1)]
        report = inventory(rows)
        self.assertEqual(report.get('raw_angles'), {'Foul': 1, 'FoulLine': 1, 'Default': 1, 'Unknown': 1})
        self.assertEqual(report.get('coverage', {}).get('recorded_foul_angle_events'), 1)
        self.assertNotIn('foul_outs', report)
        self.assertNotIn('fair_balls', report)

    def test_opportunity_units_are_supplied_exposures_not_force_outs_or_rates(self):
        row = event(runner_rows=1, runner_runs=0, runner_outs=0, unforced_runner_outs=0,
                    steals=0, cs=0, pickoffs=0, wp=0, pb=0, balk=0, unspecified=0,
                    indifference=0, error_advances=0, force_exposure=1,
                    steal_opportunities=1, extra_base_chances=0)
        report = inventory([row])
        self.assertEqual(report['supplied_opportunity_units'],
                         {'force_exposure': 1, 'steal_opportunities': 1, 'extra_base_chances': 0})
        self.assertEqual(report['opportunity_contexts'][0]['force_exposure'], 1)
        self.assertEqual(report['opportunity_contexts'][0]['events'], 1)
        self.assertNotIn('force_outs', report)

    def test_frame_end_nulls_are_conventions_not_missing_active_after_states(self):
        row = event(outs_before=2, frame_end='true', after_outs='', after_base_mask='')
        report = inventory([row])
        self.assertEqual(report.get('coverage', {}).get('frame_end_events'), 1)
        for field in ['after_outs', 'after_base_mask']:
            with self.subTest(field=field), self.assertRaises(ValueError):
                inventory([event(**{field: ''})])

    def test_rejects_duplicate_missing_malformed_or_inconsistent_facts(self):
        for field, value in [('season', '2020'), ('event_key', ''), ('outs', '4'),
                             ('outs_before', '3'), ('base_mask', '8'), ('state_outs', '0'),
                             ('state_runs', '1'), ('batting_outs', '0'), ('putouts', '0'),
                             ('runs', '-1'), ('pa', ''), ('frame_end', 'maybe'),
                             ('batter_id', ''), ('fielder', '10'), ('runner_rows', '1'),
                             ('runner_runs', '1'), ('first', 'RUNNER')]:
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                inventory([event(**{field: value})])
        with self.assertRaises(ValueError):
            inventory([event(), event()])
        with self.assertRaises(ValueError):
            inventory([])
        broken = event()
        del broken['state_outs']
        with self.assertRaises(ValueError):
            inventory([broken])

    def test_supplied_runner_runs_reconcile_and_do_not_identify_force_out_methods(self):
        fields = dict(runner_rows=1, runner_runs=1, runner_outs=0, unforced_runner_outs=0,
                      steals=0, cs=0, pickoffs=0, wp=0, pb=0, balk=0, unspecified=0,
                      indifference=0, error_advances=0, force_exposure=0,
                      steal_opportunities=0, extra_base_chances=0)
        row = event(result='Single', outs=0, state_outs=0, batting_outs=0, putouts=0,
                    runs=1, state_runs=1, after_outs=0, **fields)
        report = inventory([row])
        self.assertEqual(report.get('totals', {}).get('runner_run_units'), 1)
        with self.assertRaises(ValueError):
            inventory([event(**{**row, 'runner_runs': '0'})])
        self.assertNotIn('force_outs', report)

    def test_context_partitions_preserve_all_masks_outs_and_multi_out_signatures(self):
        rows = []
        for key, (mask, before, made) in enumerate(itertools.product(range(8), range(3), range(4)), 1):
            if before + made > 3:
                continue
            row = event(event_key=key, event_id=key, base_mask=mask, outs_before=before,
                        outs=made, state_outs=made, batting_outs=made, putouts=made,
                        after_outs=before+made, after_base_mask=mask)
            for index, name in enumerate(['first', 'second', 'third']):
                row[name] = f'RUNNER-{index}' if mask & (1 << index) else ''
            if before + made == 3:
                row.update(frame_end='true', after_outs='', after_base_mask='')
            rows.append(row)
        frozen = copy.deepcopy(rows)
        report = inventory(rows)
        self.assertEqual(sum(cell['events'] for cell in report.get('contexts', [])), len(rows))
        self.assertEqual(sum(cell['events'] for cell in report.get('signatures', [])), len(rows))
        self.assertEqual(report.get('totals', {}).get('outs'), sum(int(r['outs']) for r in rows))
        self.assertEqual(rows, frozen)
        self.assertEqual(inventory(reversed(rows)), report)
        for name in ('fielders', 'opportunity_contexts'):
            self.assertEqual(sum(cell['events'] for cell in report[name]), len(rows))
        for name in ('raw_locations', 'raw_depths'):
            self.assertEqual(sum(report[name].values()), len(rows))


class OutcomeSqlTests(unittest.TestCase):
    def test_native_sql_exports_event_grain_and_rejects_fanout_or_missing_units(self):
        sql = Path('audits/2019-outcomes.sql').read_text()
        create = """
CREATE TABLE audit_manifest AS SELECT '2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583' AS catalog_sha256;
CREATE TABLE events AS SELECT 1 AS event_key,'GAME1' AS game_id,1 AS event_id,2019 AS season,FALSE AS no_play_flag,'InPlayOut' AS plate_appearance_result,'GroundBall' AS batted_trajectory,'Shortstop' AS batted_location_general,'Default' AS batted_location_depth,'FoulLine' AS batted_location_angle,6 AS batted_to_fielder,0 AS outs,0 AS base_state,1 AS outs_on_play,0 AS runs_on_play,'BATTER' AS batter_id,'PITCHER' AS pitcher_id;
CREATE TABLE states AS SELECT 1 AS event_key,1 AS outs_on_play,0 AS runs_on_play,FALSE AS frame_end_flag,1 AS outs_end,0 AS base_state_end,NULL::VARCHAR AS runner_first_id_start,NULL::VARCHAR AS runner_second_id_start,NULL::VARCHAR AS runner_third_id_start;
CREATE TABLE batting AS SELECT 1 AS event_key,1 AS plate_appearances,1 AS outs_on_play,0 AS grounded_into_double_plays;
CREATE TABLE fielding AS SELECT 1 AS event_key,1 AS personnel_fielding_key,1 AS putouts,1 AS assists,0 AS errors,0 AS double_plays,0 AS triple_plays;
CREATE TABLE batted AS SELECT 1 AS event_key,0 AS trajectory_fly_ball,1 AS trajectory_ground_ball,0 AS trajectory_line_drive,0 AS trajectory_pop_up,0 AS bunts,1 AS trajectory_known,0 AS trajectory_unknown,1 AS batted_location_known,1 AS fielded_by_known,1 AS balls_in_play;
CREATE TABLE running AS SELECT 1 AS event_key,'BATTER' AS runner_id,0 AS runs,0 AS outs_on_basepaths,0 AS unforced_outs_on_basepaths,0 AS stolen_bases,0 AS caught_stealing,0 AS picked_off,0 AS advances_on_wild_pitches,0 AS advances_on_passed_balls,0 AS advances_on_balks,0 AS advances_on_unspecified_plays,0 AS advances_on_defensive_indifference,0 AS advances_on_errors,0 AS times_force_on_runner,0 AS stolen_base_opportunities,0 AS extra_base_chances;
"""
        bunt = "UPDATE events SET batted_trajectory='GroundBallBunt'; UPDATE batted SET trajectory_ground_ball=0,bunts=1;"
        bunt_without_stat = bunt.replace('bunts=1', 'bunts=0')
        mutations = [None, bunt, bunt_without_stat, 'INSERT INTO fielding SELECT * FROM fielding;',
                     'INSERT INTO running SELECT * FROM running;',
                     "INSERT INTO running SELECT * REPLACE ('SECOND' AS runner_id,NULL AS runs) FROM running;",
                     'DELETE FROM states;']
        for mutation in mutations:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as folder:
                db = str(Path(folder) / 'fixture.duckdb')
                subprocess.run(['duckdb', db, '-c', create + (mutation or '')], check=True,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                result = subprocess.run(['duckdb', '-readonly', '-csv', '-nullvalue', '', db],
                                        input=sql, capture_output=True, text=True)
                if mutation == 'DELETE FROM states;':
                    self.assertEqual(result.returncode, 0)
                    with self.assertRaises(ValueError):
                        inventory(csv.DictReader(io.StringIO(result.stdout)))
                elif mutation in (bunt, bunt_without_stat):
                    self.assertEqual(result.returncode, 0, result.stderr)
                    report = inventory(csv.DictReader(io.StringIO(result.stdout)))
                    self.assertEqual(report['raw_trajectories'], {'GroundBallBunt': 1})
                    self.assertEqual(report['derived_contact_flags']['derived_bunts'], int(mutation == bunt))
                    self.assertEqual(report['coverage']['bip_known_without_narrow_flag'], 1)
                    self.assertEqual(report['derived_contact_flags']['derived_known'], 1)
                    self.assertEqual(report['derived_contact_flags']['derived_ground'], 0)
                elif mutation is not None:
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                else:
                    self.assertEqual(result.returncode, 0, result.stderr)
                    report = inventory(csv.DictReader(io.StringIO(result.stdout)))
                    self.assertEqual(report['totals']['events'], 1)
                    self.assertEqual(report['coverage']['bip_events'], 1)
                    self.assertEqual(report.get('raw_angles'), {'FoulLine': 1})
                    self.assertEqual(report.get('coverage', {}).get('recorded_foul_angle_events'), 0)

    def test_manifest_requires_one_supported_snapshot_and_attribution(self):
        valid = dict(schema_version='f03-spike-1', catalog_sha256='2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583',
                     query_version='2019-filtered-extract-v1', source_url='https://example.test/catalog',
                     extracted_at='2026-10-09', attribution='Source attribution')
        for rows in [[valid], [], [valid, valid], [{**valid, 'catalog_sha256': 'bad'}], [{**valid, 'attribution': ''}]]:
            with self.subTest(rows=rows), tempfile.TemporaryDirectory() as folder:
                path = Path(folder) / 'manifest.csv'
                with path.open('w', newline='') as stream:
                    writer = csv.DictWriter(stream, fieldnames=list(valid))
                    writer.writeheader()
                    writer.writerows(rows)
                if rows == [valid]:
                    self.assertEqual(read_manifest(path), valid)
                else:
                    with self.assertRaises(ValueError):
                        read_manifest(path)


if __name__ == '__main__':
    unittest.main()
