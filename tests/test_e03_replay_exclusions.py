from collections import Counter
from contextlib import closing
import csv
from itertools import product
from pathlib import Path
import sqlite3
import unittest
from unittest.mock import patch


QUERY = Path(__file__).resolve().parents[1] / 'audits' / '2019-replay-exclusions.sql'
CATEGORIES = {'candidate', 'unknown_metadata', 'forfeit', 'suspended', 'tied',
              'shortened', 'unsupported_finish'}


def expected_reason(forfeit, suspension, tie, outs, home, away):
    # Independent necessary-condition oracle; not the engine's legal reducer.
    if None in (forfeit, suspension, tie, outs, home, away):
        return 'unknown_metadata'
    if forfeit:
        return 'forfeit'
    if suspension:
        return 'suspended'
    if tie:
        return 'tied'
    if outs < 51:
        return 'shortened'
    if home > away and outs % 6 in (3, 4, 5):
        return 'candidate'
    if away > home and outs % 6 == 0:
        return 'candidate'
    return 'unsupported_finish'


def property_facts():
    return [(f, s, t, outs, home, away)
            for f, s, t in product((None, 0, 1), repeat=3)
            for outs in (None, 48, 51, 52, 53, 54, 55, 57, 60, 66)
            for home, away in ((1, 0), (0, 1), (1, 1), (None, 1), (1, None))]


def native_property_sql():
    """Emit synthetic native-type cases; CLI verification supplements, not replaces, unittest."""
    facts = property_facts()
    games = ','.join(f"('SYN{i}',2019,TRUE,'AL','NL')" for i in range(len(facts)))
    rows = ','.join("('SYN" + str(i) + "'," +
                    ','.join('NULL' if v is None else str(v) for v in fact) + ',TRUE)'
                    for i, fact in enumerate(facts))
    query = QUERY.read_text().replace('SELECT metric,n FROM metrics ORDER BY metric;',
                                     'SELECT game_id,reason FROM classified ORDER BY game_id;')
    prefix = '''SET enable_external_access=false;
CREATE TABLE games(game_id VARCHAR,season SMALLINT,is_regular_season BOOLEAN,home_league VARCHAR,away_league VARCHAR);
CREATE TABLE results(game_id VARCHAR,forfeit_flag BOOLEAN,suspension_flag BOOLEAN,tie_flag BOOLEAN,duration_outs UTINYINT,home_runs_scored UTINYINT,away_runs_scored UTINYINT,is_nine_inning_game BOOLEAN);
'''
    return prefix + 'INSERT INTO games VALUES ' + games + ';\nINSERT INTO results VALUES ' + rows + ';\n' + query


def verify_native_property_csv(path):
    with Path(path).open(newline='') as f:
        reader = csv.DictReader(f, strict=True)
        if reader.fieldnames != ['game_id', 'reason']:
            raise AssertionError('unexpected native property output header')
        rows = list(reader)
    expected = {f'SYN{i}': expected_reason(*fact) for i, fact in enumerate(property_facts())}
    actual = {row['game_id']: row['reason'] for row in rows}
    if len(rows) != len(expected) or actual != expected:
        raise AssertionError('native individual classifications differ from independent oracle')
    return len(rows)


class ReplayExclusionTests(unittest.TestCase):
    def run_query(self, facts, *, games=None, extra_results=()):
        sql = QUERY.read_text()
        # Keep the decision body identical; only DuckDB's session-only network
        # guard is inapplicable to this standard-library SQLite oracle harness.
        self.assertEqual(sql.count('SET enable_external_access=false;'), 1)
        body = sql.split('SET enable_external_access=false;', 1)[1]
        with closing(sqlite3.connect(':memory:')) as db:
            db.executescript('''CREATE TABLE games (game_id TEXT, season INTEGER,
                is_regular_season INTEGER, home_league TEXT, away_league TEXT);
                CREATE TABLE results (game_id TEXT, forfeit_flag INTEGER,
                suspension_flag INTEGER, tie_flag INTEGER, duration_outs INTEGER,
                home_runs_scored INTEGER, away_runs_scored INTEGER,
                is_nine_inning_game INTEGER);''')
            if games is None:
                games = [(f'SYN{i}', 2019, 1, 'AL', 'NL') for i in range(len(facts))]
            db.executemany('INSERT INTO games VALUES (?,?,?,?,?)', games)
            rows = [(f'SYN{i}', *fact, 1) for i, fact in enumerate(facts)]
            db.executemany('INSERT INTO results VALUES (?,?,?,?,?,?,?,?)', rows + list(extra_results))
            result = dict(db.execute(body).fetchall())
            projection = 'SELECT metric,n FROM metrics ORDER BY metric;'
            self.assertEqual(body.count(projection), 1)
            case_query = body.replace(projection, 'SELECT game_id,reason FROM classified;')
            self.classifications = dict(db.execute(case_query).fetchall())
        self.assertEqual(set(result), CATEGORIES | {'selected_games'})
        self.assertEqual(sum(result[k] for k in CATEGORIES), result['selected_games'])
        return result

    def test_query_harness_closes_its_database_connection(self):
        connection = sqlite3.connect(':memory:')
        try:
            with patch('sqlite3.connect', return_value=connection):
                self.run_query([(0, 0, 0, 54, 1, 2)])
            with self.assertRaisesRegex(sqlite3.ProgrammingError, 'closed database'):
                connection.execute('SELECT 1')
        finally:
            connection.close()

    def test_regular_home_wins_walkoffs_and_extra_inning_endpoints_are_candidates(self):
        facts = [(0, 0, 0, outs, 2, 1) for outs in (51, 52, 53, 57, 58, 59)]
        facts += [(0, 0, 0, outs, 1, 2) for outs in (54, 60, 66)]
        self.assertEqual(self.run_query(facts)['candidate'], len(facts))

    def test_unknown_duration_is_not_promoted_by_default_nine_inning_hint(self):
        counts = self.run_query([(0, 0, 0, None, 2, 1)])
        self.assertEqual(counts['unknown_metadata'], 1)
        self.assertEqual(counts['candidate'], 0)

    def test_unknown_flags_and_scores_are_not_zero_filled(self):
        facts = [(None, 0, 0, 54, 1, 2), (0, None, 0, 54, 1, 2),
                 (0, 0, None, 54, 1, 2), (0, 0, 0, 54, None, 2),
                 (0, 0, 0, 54, 1, None)]
        self.assertEqual(self.run_query(facts)['unknown_metadata'], len(facts))

    def test_flagged_shortened_and_incomplete_endings_are_excluded(self):
        facts = [(1, 0, 0, 54, 1, 2), (0, 1, 0, 54, 1, 2),
                 (0, 0, 1, 54, 1, 1), (0, 0, 0, 48, 2, 1),
                 (0, 0, 0, 52, 1, 2), (0, 0, 0, 54, 2, 1),
                 (0, 0, 0, 54, 1, 1)]
        counts = self.run_query(facts)
        self.assertEqual(counts['candidate'], 0)
        self.assertEqual(counts['unsupported_finish'], 3)
        for reason in ['forfeit', 'suspended', 'tied', 'shortened']:
            self.assertEqual(counts[reason], 1)

    def test_multiple_flags_have_one_documented_primary_reason(self):
        counts = self.run_query([(1, 1, 1, 48, 1, 1)])
        self.assertEqual(counts['forfeit'], 1)
        self.assertEqual(counts['selected_games'], 1)

    def test_missing_result_and_duplicate_source_grains_are_unknown(self):
        valid = (0, 0, 0, 54, 1, 2)
        missing = self.run_query([], games=[('SYN0', 2019, 1, 'AL', 'NL')])
        self.assertEqual(missing['unknown_metadata'], 1)
        duplicate_result = self.run_query([valid], extra_results=[('SYN0', *valid, 1)])
        self.assertEqual(duplicate_result['unknown_metadata'], 1)
        duplicate_game = self.run_query([valid], games=[('SYN0', 2019, 1, 'AL', 'NL')] * 2)
        self.assertEqual(duplicate_game['unknown_metadata'], 1)
        self.assertEqual(duplicate_game['candidate'], 0)

    def test_out_of_selection_duplicate_identity_cannot_hide_ambiguous_game_grain(self):
        selected = ('SYN0', 2019, 1, 'AL', 'NL')
        for other in [('SYN0', 2020, 1, 'AL', 'NL'),
                      ('SYN0', 2019, 0, 'AL', 'NL'),
                      ('SYN0', 2019, 1, 'AAA', 'NL'),
                      ('SYN0', 2019, 1, 'AL', None)]:
            with self.subTest(other=other):
                counts = self.run_query([(0, 0, 0, 54, 1, 2)], games=[selected, other])
                self.assertEqual(counts['selected_games'], 1)
                self.assertEqual(counts['unknown_metadata'], 1)
                self.assertEqual(counts['candidate'], 0)

    def test_empty_and_null_source_identity_are_not_candidates(self):
        for key in [None, '', ' ']:
            with self.subTest(key=key):
                counts = self.run_query([], games=[(key, 2019, 1, 'AL', 'NL')],
                                        extra_results=[(key, 0, 0, 0, 54, 1, 2, 1)])
                self.assertEqual(counts['unknown_metadata'], 1)

    def test_only_selected_2019_mlb_regular_season_games_are_classified(self):
        games = [('SYN0', 2020, 1, 'AL', 'NL'), ('SYN1', 2019, 0, 'AL', 'NL'),
                 ('SYN2', 2019, 1, 'AAA', 'NL'), ('SYN3', 2019, 1, 'AL', None)]
        self.assertEqual(self.run_query([(0, 0, 0, 54, 1, 2)] * 4, games=games)['selected_games'], 0)

    def test_property_cartesian_metadata_cases_match_independent_oracle(self):
        facts = property_facts()
        expected = Counter(expected_reason(*fact) for fact in facts)
        counts = self.run_query(facts)
        for reason in CATEGORIES:
            self.assertEqual(counts[reason], expected[reason], reason)
        self.assertEqual(counts['selected_games'], 1350)
        self.assertEqual(len(self.classifications), len(facts))
        for i, fact in enumerate(facts):
            self.assertEqual(self.classifications[f'SYN{i}'], expected_reason(*fact), fact)


if __name__ == '__main__':
    unittest.main()
