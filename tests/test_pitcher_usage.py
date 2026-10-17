import copy
import unittest
from pathlib import Path
import tempfile
from audits import pitcher_usage


def fixture():
    # Away bats first; starter returns for a second defensive half before relief.
    events = [dict(event_key=str(i), game_id='g', event_id=str(i), pitcher_id=p,
                   result=result, outs=str(outs), runs=str(runs))
              for i, p, result, outs, runs in [
                  (1, 'a', 'Single', 0, 0), (2, 'b', 'HomeRun', 0, 2),
                  (3, 'a', 'StrikeOut', 1, 0), (4, 'c', '', 0, 1),
                  (5, 'c', 'StrikeOut', 1, 0)]]
    contexts = [dict(event_key=str(i), game_id='g', event_id=str(i), inning=str(inning), batting_side=side)
                for i, inning, side in [(1, 1, 'Away'), (2, 1, 'Home'), (3, 2, 'Away'),
                                         (4, 2, 'Away'), (5, 2, 'Away')]]
    players = []
    for pid, role, pa, np in [('a', 'start', 2, 0), ('c', 'relief', 1, 1), ('z', 'relief', 0, 0)]:
        outing = dict(game_id='g', side='Home', role=role, training=True,
                      terminal_pa=pa, non_pa=np, position1_rows=1)
        players.append(dict(player_id=pid, outing_rows=[outing]))
    return contexts, events, dict(players=players)


class PitcherUsageTests(unittest.TestCase):
    def test_decisions_use_current_context_not_eventual_outing_and_censor_game_end(self):
        args = fixture(); before = copy.deepcopy(args)
        report = pitcher_usage.inventory(*args)
        self.assertEqual(args, before)
        a, c, z = report['outings']
        self.assertEqual((a['pa'], a['outs'], a['hits'], a['runs']), (2, 1, 1, 0))
        self.assertEqual([d['status'] for d in a['decisions']], ['continue', 'change'])
        self.assertEqual([d['lead'] for d in a['decisions']], [0, 2])
        self.assertEqual([d['pa'] for d in c['decisions']], [0, 1])
        self.assertEqual([d['runs'] for d in c['decisions']], [1, 1])
        self.assertEqual(c['decisions'][-1]['status'], 'game_end_censored')
        self.assertFalse(c['decisions'][0]['terminal_pa'])
        self.assertEqual(z['decisions'], [])
        self.assertEqual(report['totals'], dict(outings=3, events=4, pa=3, zero_event_outings=1,
                                               continue_=2, change=1, game_end_censored=1))
        # Input order cannot influence chronology or summaries.
        self.assertEqual(report, pitcher_usage.inventory(list(reversed(args[0])), list(reversed(args[1])), args[2]))

    def test_context_grain_and_dimensions_fail_closed(self):
        for kind in ['duplicate', 'missing', 'orphan', 'game', 'event_id', 'side', 'inning']:
            contexts, events, workload = fixture()
            if kind == 'duplicate': contexts.append(copy.deepcopy(contexts[0]))
            elif kind == 'missing': contexts.pop()
            elif kind == 'orphan': contexts.append(dict(contexts[0], event_key='99'))
            elif kind == 'game': contexts[0]['game_id'] = 'other'
            elif kind == 'event_id': contexts[0]['event_id'] = '99'
            elif kind == 'side': contexts[0]['batting_side'] = ''
            elif kind == 'inning': contexts[0]['inning'] = '0'
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                pitcher_usage.inventory(contexts, events, workload)

    def test_outing_reconciliation_exclusions_and_multiple_stints(self):
        contexts, events, workload = fixture()
        workload['players'][0]['outing_rows'][0]['terminal_pa'] += 1
        with self.assertRaisesRegex(ValueError, 'outing'):
            pitcher_usage.inventory(contexts, events, workload)
        contexts, events, workload = fixture()
        workload['players'][0]['outing_rows'][0]['training'] = False
        workload['players'][1]['outing_rows'][0]['position1_rows'] = 2
        report = pitcher_usage.inventory(contexts, events, workload)
        self.assertEqual(len(report['outings']), 2)
        self.assertTrue(report['outings'][0]['multiple_appearance_rows'])
        self.assertEqual(report['excluded_outings'], 1)

    def test_publication_refuses_repository_existing_and_unpinned_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); bad = root / 'bad.json'; output = root / 'report.json'
            bad.write_text('{}')
            for destination in [Path(__file__).resolve(), Path('forbidden-report.json')]:
                with self.assertRaises(ValueError):
                    pitcher_usage.publish(bad, bad, bad, destination)
            output.write_text('preserve')
            with self.assertRaises(ValueError):
                pitcher_usage.publish(bad, bad, bad, output)
            self.assertEqual(output.read_text(), 'preserve')
            output.unlink()
            with self.assertRaisesRegex(ValueError, 'provenance'):
                pitcher_usage.publish(bad, bad, bad, output)
            self.assertFalse(output.exists())

    def test_csv_boundary_preserves_missing_values_but_rejects_malformed_shape(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'rows.csv'
            path.write_text('a,b\nx,\n')
            self.assertEqual(pitcher_usage.read_csv(path, ['a', 'b']), [dict(a='x', b='')])
            for text in ['b,a\nx,y\n', 'a,b\nx\n', 'a,b\nx,y,z\n']:
                path.write_text(text)
                with self.assertRaises(ValueError):
                    pitcher_usage.read_csv(path, ['a', 'b'])


if __name__ == '__main__': unittest.main()
