"""Native synthetic coverage for the read-only source-fact export, not a resolver."""
import csv
import hashlib
import io
from itertools import product
from pathlib import Path
import subprocess
import tempfile
import unittest

QUERY = Path(__file__).resolve().parents[1] / 'audits' / '2019-terminal-hit-source.sql'
EVENT_COLUMNS = ('event_key', 'game_id', 'event_id', 'no_play_flag', 'plate_appearance_result',
    'inning', 'frame', 'outs', 'base_state', 'outs_on_play', 'runs_on_play', 'runs_batted_in')
STATE_COLUMNS = ('event_key', 'inning_start', 'frame_start', 'outs_start', 'base_state_start',
    'outs_on_play', 'runs_on_play', 'outs_end', 'base_state_end', 'score_home_start',
    'score_away_start', 'score_home_end', 'score_away_end', 'frame_end_flag',
    'game_end_flag', 'truncated_frame_flag')
OUTPUT = ('event_key', 'game_id', 'event_id', 'result', 'inning', 'frame', 'outs_before',
    'base_mask', 'event_outs', 'event_runs', 'event_rbi', 'after_outs', 'after_base_mask',
    'home_start', 'away_start', 'home_end', 'away_end', 'frame_end', 'game_end', 'truncated_frame')
SCHEMA = '''SET enable_external_access=FALSE;
CREATE TABLE events(event_key UINTEGER,game_id VARCHAR,event_id UINTEGER,
 no_play_flag BOOLEAN,plate_appearance_result VARCHAR,inning UTINYINT,frame VARCHAR,
 outs UTINYINT,base_state UTINYINT,outs_on_play UTINYINT,runs_on_play UTINYINT,runs_batted_in UTINYINT);
CREATE TABLE states(event_key UINTEGER,inning_start UTINYINT,frame_start VARCHAR,
 outs_start UTINYINT,base_state_start UTINYINT,outs_on_play UTINYINT,runs_on_play UTINYINT,
 outs_end UTINYINT,base_state_end UTINYINT,score_home_start UTINYINT,score_away_start UTINYINT,
 score_home_end UTINYINT,score_away_end UTINYINT,frame_end_flag BOOLEAN,
 game_end_flag BOOLEAN,truncated_frame_flag BOOLEAN);
'''


def fixture(key=1, **changes):
    event = dict(zip(EVENT_COLUMNS, (key, 'SYNTHETIC', key, False, 'Single', 9,
        'Bottom', 0, 4, 0, 1, 1)))
    state = dict(zip(STATE_COLUMNS, (key, 9, 'Bottom', 0, 4, 0, 1, 0, None,
        2, 2, 3, 2, True, True, True)))
    for name, value in changes.items():
        target, field = name.split('__', 1)
        (event if target == 'e' else state)[field] = value
    return event, state


def inserts(table, rows, columns):
    def literal(value):
        if value is None:
            return 'NULL'
        if isinstance(value, str):
            return "'" + value.replace("'", "''") + "'"
        return str(value).upper()
    return ''.join('INSERT INTO ' + table + ' VALUES (' +
        ','.join(literal(row[name]) for name in columns) + ');\n' for row in rows)


class TerminalHitSourceTests(unittest.TestCase):
    def native(self, prefix, query, database=':memory:'):
        return subprocess.run(['duckdb', '-init', '/dev/null', '-bail', '-batch',
            '-csv', '-nullvalue', '', database], input=prefix + query, text=True,
            capture_output=True, check=False, timeout=30)

    def run_rows(self, fixtures, *, extra_events=(), extra_states=(), expected_error=False):
        events = [item[0] for item in fixtures] + list(extra_events)
        states = [item[1] for item in fixtures] + list(extra_states)
        prefix = SCHEMA + inserts('events', events, EVENT_COLUMNS) + inserts('states', states, STATE_COLUMNS)
        result = self.native(prefix, QUERY.read_text())
        if expected_error:
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('terminal source: conflicting or incomplete event/state join', result.stderr)
            return None
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, '')
        reader = csv.DictReader(io.StringIO(result.stdout), strict=True)
        self.assertEqual(reader.fieldnames, list(OUTPUT))
        return list(reader)

    def test_literal_walkoff_keeps_boundary_null_and_credit_without_inventing_clock(self):
        row, = self.run_rows([fixture()])
        self.assertEqual(row, dict(zip(OUTPUT, ('1','SYNTHETIC','1','Single','9','Bottom',
            '0','4','0','1','1','0','','2','2','3','2','true','true','true'))))
        self.assertNotIn('winning_runner', row)
        self.assertNotIn('stop_time', row)

    def test_cartesian_contexts_and_reversed_input_order_are_literal(self):
        fixtures = []
        for key, (value, mask, outs, inning, frame) in enumerate(product(
                ('Single','Double'), range(8), range(3), (1,9,10), ('Top','Bottom')), 1):
            fixtures.append(fixture(key, e__plate_appearance_result=value,
                e__base_state=mask, s__base_state_start=mask, e__outs=outs,
                s__outs_start=outs, s__outs_end=outs, e__inning=inning,
                s__inning_start=inning, e__frame=frame, s__frame_start=frame,
                s__score_home_end=3 if frame=='Bottom' else 2,
                s__score_away_end=2 if frame=='Bottom' else 3))
        rows = self.run_rows(list(reversed(fixtures)))
        self.assertEqual(len(rows), 288)
        for (event, state), row in zip(fixtures, rows):
            self.assertEqual(int(row['event_key']), event['event_key'])
            for output, original in (('result','plate_appearance_result'), ('base_mask','base_state'),
                    ('outs_before','outs'), ('inning','inning'), ('frame','frame')):
                self.assertEqual(row[output], str(event[original]))
            self.assertEqual(row['after_outs'], str(state['outs_end']))

    def test_outs_early_endings_and_nonwinning_late_hits_are_not_filtered(self):
        cases = [fixture(1, e__outs_on_play=1, s__outs_on_play=1, s__outs_end=1),
            fixture(2, e__inning=7, s__inning_start=7),
            fixture(3, s__score_away_start=9, s__score_away_end=9),
            fixture(4, s__game_end_flag=False, s__truncated_frame_flag=False,
                s__frame_end_flag=False, s__base_state_end=1)]
        self.assertEqual(len(self.run_rows(cases)), 4)

    def test_new_unknowns_are_preserved_instead_of_zero_filled(self):
        for field in ('inning_start','frame_start','outs_start','base_state_start',
                'score_home_start','score_away_start','score_home_end','score_away_end',
                'frame_end_flag','game_end_flag','truncated_frame_flag','outs_end','base_state_end'):
            with self.subTest(field=field):
                rows = self.run_rows([fixture(**{'s__'+field:None})])
                self.assertEqual(len(rows), 1)
                projected = {'score_home_start':'home_start','score_away_start':'away_start',
                    'score_home_end':'home_end','score_away_end':'away_end',
                    'frame_end_flag':'frame_end','game_end_flag':'game_end',
                    'truncated_frame_flag':'truncated_frame','outs_end':'after_outs',
                    'base_state_end':'after_base_mask'}
                if field in projected:
                    self.assertEqual(rows[0][projected[field]], '')
        row, = self.run_rows([fixture(e__inning=None, e__frame=None,
            e__runs_batted_in=None, s__game_end_flag=None, s__score_home_start=None)])
        self.assertEqual([row[k] for k in ('inning','frame','event_rbi','game_end','home_start')], ['']*5)

    def test_no_play_and_other_results_are_excluded_not_coarsened(self):
        fixtures = [fixture(1)]
        for key, value in enumerate(('GroundRuleDouble','Triple','HomeRun','Error',None),2):
            fixtures.append(fixture(key,e__plate_appearance_result=value))
        fixtures.append(fixture(7,e__no_play_flag=True))
        self.assertEqual([r['event_key'] for r in self.run_rows(fixtures)], ['1'])

    def test_duplicate_event_or_state_keys_refuse(self):
        event, state = fixture()
        self.run_rows([fixture()], extra_events=[event], expected_error=True)
        self.run_rows([fixture()], extra_states=[state], expected_error=True)

    def test_missing_or_orphan_state_refuses_even_outside_hit_candidates(self):
        event, state = fixture(2,e__plate_appearance_result='Triple')
        self.run_rows([fixture()], extra_events=[event], expected_error=True)
        self.run_rows([fixture()], extra_states=[state], expected_error=True)

    def test_nonnull_start_and_unit_conflicts_refuse(self):
        for field, value in (('inning_start',10),('frame_start','Top'),('outs_start',1),
                ('base_state_start',3),('outs_on_play',1),('runs_on_play',2)):
            with self.subTest(field=field):
                self.run_rows([fixture(**{'s__'+field:value})], expected_error=True)

    def test_known_score_deltas_must_match_batting_half_and_event_runs(self):
        for changes in ({'s__score_home_end':4}, {'s__score_away_end':3},
                {'e__frame':'Top','s__frame_start':'Top'},
                {'s__score_home_start':4}):
            with self.subTest(changes=changes):
                self.run_rows([fixture(**changes)], expected_error=True)
        row, = self.run_rows([fixture(e__frame='Top',s__frame_start='Top',
            s__score_home_end=2,s__score_away_end=3)])
        self.assertEqual(row['frame'], 'Top')

    def test_readonly_database_bytes_do_not_change(self):
        event, state = fixture()
        with tempfile.TemporaryDirectory() as directory:
            database = str(Path(directory)/'synthetic.duckdb')
            result = self.native(SCHEMA, inserts('events',[event],EVENT_COLUMNS) +
                inserts('states',[state],STATE_COLUMNS),database)
            self.assertEqual(result.returncode,0,result.stderr)
            before = hashlib.sha256(Path(database).read_bytes()).hexdigest()
            result = subprocess.run(['duckdb','-init','/dev/null','-readonly','-bail',
                '-batch','-csv',database],input=QUERY.read_text(),capture_output=True,
                text=True,check=False,timeout=30)
            self.assertEqual(result.returncode,0,result.stderr)
            self.assertEqual(result.stderr,'')
            self.assertEqual(hashlib.sha256(Path(database).read_bytes()).hexdigest(),before)


if __name__ == '__main__':
    unittest.main()
