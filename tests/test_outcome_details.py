"""Native SQL regression tests for the bounded factual supplement (no network)."""
from pathlib import Path
import csv
import io
import subprocess
import unittest

FIXTURE = """
CREATE TABLE selected_games AS SELECT 'GAME1' game_id;
CREATE TABLE selected_events AS SELECT 1::UINTEGER event_key,'GAME1' game_id,1::UTINYINT event_id,
 FALSE no_play_flag,'Away' batting_side,'B1' batter_id,'P1' pitcher_id,
 1::UTINYINT outs_on_play,0::UTINYINT runs_on_play;
CREATE TABLE source_events AS SELECT * FROM selected_events;
CREATE TABLE stg_event_audit AS SELECT 1::UINTEGER event_key,'GAME1' game_id,1::UTINYINT event_id,
 '2019.EVN' filename,100::UINTEGER line_number,'63' raw_play;
CREATE TABLE stg_event_flags AS SELECT 1::UINTEGER event_key,1::UTINYINT sequence_id,'GroundBall' flag;
CREATE TABLE stg_event_baserunners AS SELECT 1::UINTEGER event_key,'GAME1' game_id,1::UTINYINT event_id,
 'Batter' baserunner,'B1' runner_id,TRUE is_out,FALSE run_scored_flag;
CREATE TABLE expected_running AS SELECT event_key,baserunner,runner_id,
 run_scored_flag::INTEGER runs,is_out::INTEGER outs_on_basepaths FROM stg_event_baserunners;
CREATE TABLE stg_event_fielding_plays AS SELECT 1::UINTEGER event_key,'GAME1' game_id,1::UTINYINT event_id,
 1::UTINYINT sequence_id,6::UTINYINT fielding_position,'Assist' fielding_play
 UNION ALL SELECT 1,'GAME1',1,2,3,'Putout';
CREATE TABLE stg_game_fielding_appearances AS SELECT 'GAME1' game_id,'SS1' player_id,'Home' side,
 6::UTINYINT fielding_position,1::UTINYINT start_event_id,10::UTINYINT end_event_id
 UNION ALL SELECT 'GAME1','1B1','Home',3,1,10;
CREATE TABLE expected_fielding AS SELECT 1::UINTEGER event_key,1 putouts,1 assists,0 errors;
"""

class OutcomeDetailTests(unittest.TestCase):
    def run_sql(self, mutation=''):
        sql=Path('audits/2019-outcome-details-verify.sql').read_text()
        return subprocess.run(['duckdb','-bail','-batch','-init','/dev/null','-csv',':memory:'],
                              input=FIXTURE+mutation+sql,text=True,capture_output=True)

    def test_literal_facts_and_named_credits_reconcile(self):
        result=self.run_sql()
        self.assertEqual(result.returncode,0,result.stderr)
        values={r['metric']:int(r['n']) for r in csv.DictReader(io.StringIO(result.stdout))}
        self.assertEqual(values['events'],1)
        self.assertEqual(values['putouts'],1)
        self.assertEqual(values['assists'],1)
        self.assertEqual(values['unresolved_credit_actors'],0)

    def test_repeated_assists_are_deduplicated_within_one_credit_segment(self):
        mutation="UPDATE stg_event_fielding_plays SET sequence_id=3 WHERE fielding_play='Putout'; INSERT INTO stg_event_fielding_plays VALUES (1,'GAME1',1,2,6,'Assist');"
        result=self.run_sql(mutation)
        self.assertEqual(result.returncode,0,result.stderr)
        values={r['metric']:int(r['n']) for r in csv.DictReader(io.StringIO(result.stdout))}
        self.assertEqual(values['assists'],1)

    def test_missing_duplicate_or_foreign_facts_do_not_promote(self):
        mutations=[
            'DELETE FROM stg_event_audit;',
            'UPDATE source_events SET outs_on_play=2;',
            'INSERT INTO source_events SELECT * FROM source_events;',
            'INSERT INTO stg_game_fielding_appearances SELECT * FROM stg_game_fielding_appearances;',
            'INSERT INTO stg_event_audit SELECT * FROM stg_event_audit;',
            "UPDATE stg_event_audit SET raw_play='';",
            "UPDATE stg_event_audit SET game_id='WRONG';",
            'UPDATE stg_event_audit SET event_id=2;',
            'INSERT INTO stg_event_flags SELECT * FROM stg_event_flags;',
            'UPDATE stg_event_flags SET event_key=99;',
            'INSERT INTO stg_event_baserunners SELECT * FROM stg_event_baserunners;',
            'INSERT INTO stg_event_fielding_plays SELECT * FROM stg_event_fielding_plays;',
            'UPDATE stg_event_fielding_plays SET sequence_id=NULL;',
            'UPDATE expected_fielding SET assists=2;',
            "UPDATE expected_running SET runner_id='OTHER';",
            'INSERT INTO expected_running SELECT * FROM expected_running;',
            'DELETE FROM stg_game_fielding_appearances WHERE fielding_position=3;',
            "INSERT INTO stg_game_fielding_appearances VALUES ('GAME1','OTHER','Home',3,1,10);",
            "UPDATE stg_game_fielding_appearances SET side='Away' WHERE fielding_position=3;",
        ]
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                result=self.run_sql(mutation)
                self.assertNotEqual(result.returncode,0,'Invalid source accepted: '+mutation)

    def test_no_play_raw_runners_are_preserved_without_fabricating_derived_statistics(self):
        mutation="""
INSERT INTO selected_events SELECT 2,game_id,2,TRUE,batting_side,batter_id,pitcher_id,0,0 FROM selected_events;
INSERT INTO source_events SELECT * FROM selected_events WHERE event_key=2;
INSERT INTO stg_event_audit VALUES(2,'GAME1',2,'2019.EVN',101,'NP');
INSERT INTO stg_event_baserunners VALUES(2,'GAME1',2,'First','F1',FALSE,FALSE);
"""
        result=self.run_sql(mutation)
        self.assertEqual(result.returncode,0,result.stderr)

    def test_terminal_empty_appearance_remains_recorded_but_not_activated(self):
        result=self.run_sql("INSERT INTO stg_game_fielding_appearances VALUES ('GAME1','UNPLAYED','Home',3,2,1);")
        self.assertEqual(result.returncode,0,result.stderr)
        values={r['metric']:int(r['n']) for r in csv.DictReader(io.StringIO(result.stdout))}
        self.assertEqual(values['terminal_empty_appearances'],1)
        self.assertEqual(values['unresolved_credit_actors'],0)

    def test_nonterminal_reversed_and_skipped_terminal_ranges_still_fail(self):
        for start,end in [(3,1),(3,2)]:
            result=self.run_sql(f"INSERT INTO stg_game_fielding_appearances VALUES ('GAME1','UNPLAYED','Home',3,{start},{end});")
            self.assertNotEqual(result.returncode,0)

    def test_unknown_appearance_position_does_not_invalidate_known_credits(self):
        result=self.run_sql("INSERT INTO stg_game_fielding_appearances VALUES ('GAME1','UNKNOWN','Home',0,1,10);")
        self.assertEqual(result.returncode,0,result.stderr)

    def test_unknown_credit_position_is_reported_not_given_a_fictional_actor(self):
        result=self.run_sql('UPDATE stg_event_fielding_plays SET fielding_position=0 WHERE fielding_play=\'Putout\';')
        self.assertEqual(result.returncode,0,result.stderr)
        values={r['metric']:int(r['n']) for r in csv.DictReader(io.StringIO(result.stdout))}
        self.assertEqual(values['unknown_position_credits'],1)
        self.assertEqual(values['unresolved_credit_actors'],0)

if __name__=='__main__': unittest.main()
