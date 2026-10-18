import copy
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from audits import season_usage_targets as audit


def fixture():
    games = [dict(game_id='OAK201904010', date='2019-04-01', away_team_id='SEA',
                  home_team_id='OAK', starting_pitcher_ids={'SEA': 'away', 'OAK': 'home'},
                  suspended=False, shortened=False, duration_outs=6)]
    people = [dict(player_id=p) for p in ('away', 'home', 'credit', 'current', 'relief')]
    events = []
    for key, team, pitcher, outs, result in [(1, 'OAK', 'home', 2, 'StrikeOut'),
            (2, 'OAK', 'relief', 1, None), (3, 'SEA', 'away', 3, 'IntentionalWalk'),
            (4, 'SEA', 'away', 0, 'Interference')]:
        events.append(dict(event_key=key, game_id=games[0]['game_id'], pitcher_id=pitcher,
                           batter_id='current', fielding_team_id=team,
                           batting_team_id='SEA' if team == 'OAK' else 'OAK',
                           no_play_flag=False, outs_on_play=outs, result=result))
    batting = [dict(event_key=e['event_key'], game_id=e['game_id'], batter_id='credit',
                    batting_team_id=e['batting_team_id'], plate_appearances=1,
                    pitcher_id='relief')
               for e in events if e['result']]
    teams = [dict(game_id=games[0]['game_id'], team_id=t, putouts=3) for t in ('SEA', 'OAK')]
    return games, people, batting, events, teams


class SeasonUsageTargetsTests(unittest.TestCase):
    def test_credit_not_participant_includes_ibb_ci_and_non_pa_outs(self):
        r = audit.inventory(*fixture())
        self.assertEqual(r['totals'], dict(pa=3, outs=6))
        self.assertEqual(r['team_games'], dict(OAK=1, SEA=1))
        rows = {(x['team_id'], x['player_id'], x['role']): x for x in r['records']}
        self.assertEqual(rows['SEA', 'credit', 'batter']['usage'], 1)
        self.assertEqual(rows['OAK', 'credit', 'batter']['usage'], 2)
        self.assertEqual(rows['OAK', 'relief', 'pitcher']['usage'], 1)
        self.assertFalse(any(x['player_id'] == 'current' for x in r['records']))
        self.assertEqual(rows['OAK', 'credit', 'batter']['source_rows'], 2)

    def test_unknown_zero_and_absent_are_distinct(self):
        args = fixture(); args[2][0]['plate_appearances'] = None
        args[2][1]['plate_appearances'] = None
        args[3][0]['outs_on_play'] = None
        args[3].append(dict(args[3][0], event_key=5, pitcher_id='current',
                            outs_on_play=0, result=None))
        r = audit.inventory(*args)
        rows = {(x['team_id'], x['player_id'], x['role']): x for x in r['records']}
        unknown = rows['OAK', 'home', 'pitcher']
        self.assertIsNone(unknown['usage']); self.assertEqual(unknown['unknown_rows'], 1)
        self.assertIsNone(rows['SEA', 'credit', 'batter']['usage'])
        self.assertIsNone(rows['OAK', 'credit', 'batter']['usage'])
        self.assertEqual(rows['OAK', 'credit', 'batter']['source_rows'], 2)
        self.assertEqual(rows['OAK', 'credit', 'batter']['unknown_rows'], 1)
        self.assertEqual(rows['OAK', 'current', 'pitcher']['usage'], 0)
        self.assertNotIn(('SEA', 'relief', 'pitcher'), rows)
        self.assertEqual(r['totals'], dict(pa=None, outs=None))

    def test_observed_zero_batting_credit_is_not_an_absent_target(self):
        args = fixture(); args[2][0]['plate_appearances'] = 0
        rows = audit.inventory(*args)['records']
        zero = next(r for r in rows if r['role'] == 'batter' and r['team_id'] == 'SEA')
        self.assertEqual((zero['usage'], zero['unknown_rows'], zero['source_rows']), (0, 0, 1))

    def test_no_play_is_not_workload_and_input_order_is_irrelevant(self):
        args = fixture(); before = copy.deepcopy(args)
        expected = audit.inventory(*args)
        self.assertEqual(args, before)
        args[3].append(dict(args[3][0], event_key=99, no_play_flag=True, result=None,
                            pitcher_id='relief', outs_on_play=0))
        self.assertEqual(audit.inventory(*args), expected)
        self.assertEqual(audit.inventory(*(list(reversed(a)) for a in args)), expected)
        self.assertEqual(expected['records'], sorted(expected['records'], key=lambda x:
                         (x['game_id'], x['team_id'], x['player_id'], x['role'])))

    def test_trade_team_splits_and_suspension_flags_preserved(self):
        args = fixture()
        g = dict(args[0][0], game_id='SEA201904020', date='2019-04-02',
                 home_team_id='SEA', away_team_id='OAK', suspended=True, shortened=True)
        args[0].append(g)
        for index in (2, 3, 4):
            args[index].extend([dict(x, game_id=g['game_id'], **(
                {'event_key': x['event_key'] + 10} if index in (2, 3) else {}))
                for x in list(args[index])])
        r = audit.inventory(*args)
        self.assertEqual(r['totals'], dict(pa=6, outs=12))
        self.assertEqual(r['team_games'], dict(OAK=2, SEA=2))
        self.assertTrue(r['games'][1]['suspended']); self.assertTrue(r['games'][1]['shortened'])
        self.assertEqual(len(r['records']), 10)

    def test_duplicates_refuse_at_every_source_grain(self):
        for index in range(5):
            args = fixture(); args[index].append(copy.deepcopy(args[index][0]))
            with self.subTest(index=index), self.assertRaises(ValueError):
                audit.inventory(*args)

    def test_orphans_wrong_teams_missing_people_and_terminal_binding_refuse(self):
        changes = [(2, 'game_id', 'orphan'), (3, 'game_id', 'orphan'),
                   (4, 'game_id', 'orphan'), (2, 'batting_team_id', 'NYA'),
                   (3, 'fielding_team_id', 'NYA'), (4, 'team_id', 'NYA'),
                   (2, 'batter_id', 'missing'), (3, 'pitcher_id', 'missing'),
                   (2, 'event_key', 2), (3, 'no_play_flag', None)]
        for index, key, value in changes:
            args = fixture(); args[index][0][key] = value
            with self.subTest(index=index, key=key), self.assertRaises(ValueError):
                audit.inventory(*args)
        for index in (2, 4):
            args = fixture(); args[index].pop()
            with self.subTest(missing=index), self.assertRaises(ValueError):
                audit.inventory(*args)

    def test_duration_and_team_putouts_reconcile_individually(self):
        for index, key, value in [(0, 'duration_outs', 7), (4, 'putouts', 4)]:
            args = fixture(); args[index][0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):audit.inventory(*args)
        args = fixture(); args[4][0]['putouts'] = 2; args[4][1]['putouts'] = 4
        with self.assertRaises(ValueError):audit.inventory(*args)

    def test_invalid_types_negative_counts_and_game_metadata_refuse(self):
        changes = [(2, 'plate_appearances', -1), (2, 'plate_appearances', True),
                   (3, 'outs_on_play', 4), (3, 'outs_on_play', '1'),
                   (0, 'date', '2020-04-01'), (0, 'suspended', None),
                   (0, 'starting_pitcher_ids', {'SEA': 'missing', 'OAK': 'home'}),
                   (4, 'putouts', None)]
        for index, key, value in changes:
            args = fixture(); args[index][0][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                audit.inventory(*args)

    def test_publication_refuses_repo_existing_and_unpinned_without_receipt(self):
        with self.assertRaises(ValueError):audit.publish('missing', Path('bad-targets'))
        with tempfile.TemporaryDirectory() as folder:
            p = Path(folder); source = p/'fake.db'; source.write_text('fake')
            output = p/'new'
            with self.assertRaises(ValueError):audit.publish(source, output)
            self.assertFalse(output.exists())
            with self.assertRaises(ValueError):audit.publish(source, p)
            self.assertEqual(source.read_text(), 'fake')

    def test_sql_recipe_is_pinned_and_drift_refuses_before_native_read(self):
        sql = Path(audit.__file__).with_name('2019-season-usage-targets.sql')
        self.assertEqual(audit._digest(sql), audit.SQL_SHA256)
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)/'new'
            with patch.object(audit, '_digest', side_effect=[audit.SNAPSHOT_SHA256, 'drift', 'recipe']), \
                    patch.object(audit.subprocess, 'run') as run:
                with self.assertRaises(ValueError):audit.publish('fake.db', output)
            run.assert_not_called(); self.assertFalse(output.exists())

    def test_native_failure_retains_evidence_but_never_completes(self):
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)/'new'
            with patch.object(audit, '_digest', side_effect=[audit.SNAPSHOT_SHA256, audit.SQL_SHA256, 'recipe']), \
                    patch.object(audit.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'duckdb')) as run:
                with self.assertRaises(subprocess.CalledProcessError):audit.publish('fake.db', output)
            command = run.call_args.args[0]
            self.assertEqual(command[:3], ['duckdb', '-init', '/dev/null'])
            self.assertIn('-readonly', command)
            self.assertIn('SET enable_external_access=false', command)
            self.assertTrue((output/'native.json').exists())
            self.assertTrue((output/'native.stderr').exists())
            self.assertFalse((output/'targets.json').exists())
            self.assertFalse((output/'receipt.json').exists())
            with self.assertRaises(ValueError):audit.publish('fake.db', output)


if __name__ == '__main__':unittest.main()
