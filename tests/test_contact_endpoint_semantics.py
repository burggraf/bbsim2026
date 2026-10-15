"""Literal source-representation cells; no inferred retirement or game reducer."""
import csv
import hashlib
import io
import json
from pathlib import Path
import unittest

import test_contact_consequence_support as previous


ROOT = Path(__file__).resolve().parents[1]
QUERY = ROOT / 'audits/2019-contact-endpoint-semantics.sql'
SHARED = ROOT / 'audits/2019-contact-consequence-support.sql'
FIELDS = {'split', 'pa', 'result', 'role', 'expected_identity_present',
          'child_present', 'observed_identity_present', 'identity_matches',
          'is_out', 'base_end', 'attempted_advance_to_base', 'is_advance_attempt'}


class NativeRepresentation(unittest.TestCase):
    def setUp(self):
        self.source = previous.NativeSQLRegressions()
        self.source.setUp()
        self.addCleanup(self.source.doCleanups)
        self.snapshot, self.detail = self.source.snapshot, self.source.detail
        self.source.query = self.source.directory / 'effective-semantics.sql'
        # This fixture represents a source child vector, not a clean causal episode.
        self.source.sql(self.detail, 'UPDATE stg_event_baserunners SET baserunning_play_type=NULL;')

    def execute(self):
        old = SHARED.read_text()
        marker = 'WITH flag_vectors AS ('
        self.assertEqual(old.count(marker), 1)
        prefix = old[:old.index(marker)]
        self.source.query.write_text(prefix + QUERY.read_text())
        return self.source.execute()

    def cells(self):
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, '')
        reader = csv.DictReader(io.StringIO(result.stdout))
        self.assertEqual(reader.fieldnames, ['grain', 'facts', 'units'])
        cells = []
        seen = set()
        for row in reader:
            facts = json.loads(row['facts'])
            self.assertEqual(set(facts), FIELDS)
            self.assertIn(row['grain'], ('event_batter', 'before_runner', 'runner_child'))
            units = int(row['units'])
            self.assertGreater(units, 0)
            key = (row['grain'], json.dumps(facts, sort_keys=True))
            self.assertNotIn(key, seen)
            seen.add(key)
            cells.append((row['grain'], facts, units))
        return cells

    def one(self, grain, role):
        rows = [(f, n) for g, f, n in self.cells() if g == grain and f['role'] == role]
        self.assertEqual(len(rows), 1, rows)
        return rows[0]

    def totals(self):
        cells = self.cells()
        return {grain: sum(n for g, _, n in cells if g == grain)
                for grain in ('event_batter', 'before_runner', 'runner_child')}

    def test_absent_batter_is_not_fabricated_out_or_false_child_evidence(self):
        self.source.sql(self.detail, "DELETE FROM stg_event_baserunners WHERE baserunner='Batter';")
        facts, units = self.one('event_batter', 'Batter')
        self.assertEqual(units, 1)
        self.assertEqual((facts['pa'], facts['result']), (1, 'InPlayOut'))
        self.assertTrue(facts['expected_identity_present'])
        self.assertIs(facts['child_present'], False)
        for field in ('observed_identity_present', 'identity_matches', 'is_out',
                      'base_end', 'attempted_advance_to_base', 'is_advance_attempt'):
            with self.subTest(field=field):
                self.assertIsNone(facts[field])
        self.assertEqual(self.totals(), dict(event_batter=1, before_runner=1, runner_child=1))

    def test_literal_present_batter_is_kept_without_normalization(self):
        for is_out, endpoint in [('false', "'First'"), ('true', "'Out'"), ('true', 'NULL')]:
            with self.subTest(is_out=is_out, endpoint=endpoint):
                self.source.sql(self.detail, f"UPDATE stg_event_baserunners SET is_out={is_out},base_end={endpoint} WHERE baserunner='Batter';")
                facts, units = self.one('event_batter', 'Batter')
                self.assertEqual(units, 1)
                self.assertIs(facts['child_present'], True)
                self.assertIs(facts['is_out'], is_out == 'true')
                self.assertEqual(facts['base_end'], None if endpoint == 'NULL' else endpoint.strip("'"))

    def test_missing_occupied_before_child_still_has_a_frame_not_a_hold(self):
        self.source.sql(self.detail, "DELETE FROM stg_event_baserunners WHERE baserunner='Third';")
        facts, units = self.one('before_runner', 'Third')
        self.assertEqual(units, 1)
        self.assertIs(facts['child_present'], False)
        self.assertIsNone(facts['is_out'])
        self.assertIsNone(facts['base_end'])
        self.assertEqual(self.totals(), dict(event_batter=1, before_runner=1, runner_child=1))

    def test_retired_origin_base_null_and_literal_out_remain_distinct(self):
        self.source.sql(self.detail, "UPDATE stg_event_baserunners SET is_out=true WHERE baserunner='Third';")
        for endpoint in ("'Third'", 'NULL', "'Out'", "'NULL'"):
            with self.subTest(endpoint=endpoint):
                self.source.sql(self.detail, f"UPDATE stg_event_baserunners SET base_end={endpoint} WHERE baserunner='Third';")
                for grain in ('before_runner', 'runner_child'):
                    facts, units = self.one(grain, 'Third')
                    self.assertEqual(units, 1)
                    self.assertIs(facts['is_out'], True)
                    self.assertEqual(facts['base_end'], None if endpoint == 'NULL' else endpoint.strip("'"))

    def test_unknown_identity_literal_null_and_mismatch_are_not_aliased(self):
        for identity, available, match in [('NULL', False, None), ("'NULL'", True, False),
                                            ("'OTHER'", True, False), ("'T1'", True, True)]:
            with self.subTest(identity=identity):
                self.source.sql(self.detail, f"UPDATE stg_event_baserunners SET runner_id={identity} WHERE baserunner='Third';")
                facts, _ = self.one('before_runner', 'Third')
                self.assertIs(facts['child_present'], True)
                self.assertIs(facts['expected_identity_present'], True)
                self.assertIs(facts['observed_identity_present'], available)
                self.assertIs(facts['identity_matches'], match)

    def test_nullable_attempt_bit_and_target_are_separate_literal_fields(self):
        for target, bit in [('NULL', 'NULL'), ('NULL', 'false'), ("'Third'", 'true'), ("'NULL'", 'NULL')]:
            with self.subTest(target=target, bit=bit):
                self.source.sql(self.detail, f"UPDATE stg_event_baserunners SET attempted_advance_to_base={target},is_advance_attempt={bit} WHERE baserunner='Third';")
                facts, _ = self.one('runner_child', 'Third')
                self.assertEqual(facts['attempted_advance_to_base'], None if target == 'NULL' else target.strip("'"))
                self.assertIs(facts['is_advance_attempt'], {'NULL': None, 'false': False, 'true': True}[bit])

    def test_extra_known_role_record_is_not_silently_dropped(self):
        self.source.sql(self.detail, "INSERT INTO stg_event_baserunners VALUES(1,'ANA201904040',1,'First','EXTRA',NULL,true,false,NULL,NULL,NULL,'First',false);")
        self.assertEqual(self.totals(), dict(event_batter=1, before_runner=1, runner_child=3))
        facts, units = self.one('runner_child', 'First')
        self.assertEqual(units, 1)
        self.assertIs(facts['expected_identity_present'], False)
        self.assertIs(facts['child_present'], True)
        self.assertIsNone(facts['identity_matches'])

    def test_empty_children_conserve_frames_and_retain_missing_evidence(self):
        self.source.sql(self.detail, 'DELETE FROM stg_event_baserunners;')
        self.assertEqual(self.totals(), dict(event_batter=1, before_runner=1, runner_child=0))

    def test_supplied_non_pa_unit_is_not_promoted_to_plate_appearance(self):
        self.source.sql(self.snapshot, "UPDATE batting SET plate_appearances=NULL; UPDATE events SET plate_appearance_result=NULL,outs_on_play=0,batted_trajectory=NULL WHERE event_key=1; UPDATE states SET outs_end=0;")
        self.source.sql(self.detail, "UPDATE selected_events SET plate_appearance_result=NULL,outs_on_play=0,batted_trajectory=NULL WHERE event_key=1; UPDATE stg_event_audit SET raw_play='WP.3-H' WHERE event_key=1; DELETE FROM stg_event_flags; DELETE FROM stg_event_baserunners WHERE baserunner='Batter'; UPDATE stg_event_baserunners SET baserunning_play_type='WildPitch' WHERE event_key=1;")
        self.assertTrue(self.cells())
        self.assertTrue(all(f['pa'] is None and f['result'] is None for _, f, _ in self.cells()))
        self.assertEqual(self.totals(), dict(event_batter=1, before_runner=1, runner_child=1))

    def test_all_before_masks_conserve_occupied_frames_without_batter_inference(self):
        self.source.sql(self.snapshot, 'UPDATE events SET runs_on_play=0,batted_trajectory=NULL WHERE event_key=1;')
        self.source.sql(self.detail, "UPDATE selected_events SET runs_on_play=0,batted_trajectory=NULL WHERE event_key=1; UPDATE stg_event_audit SET raw_play='63' WHERE event_key=1; DELETE FROM stg_event_flags;")
        for mask in range(8):
            with self.subTest(mask=mask):
                ids = ["'F1'" if mask & 1 else 'NULL', "'S1'" if mask & 2 else 'NULL', "'T1'" if mask & 4 else 'NULL']
                self.source.sql(self.snapshot, f"UPDATE events SET base_state={mask} WHERE event_key=1; UPDATE states SET base_state_end={mask},runner_first_id_start={ids[0]},runner_second_id_start={ids[1]},runner_third_id_start={ids[2]};")
                self.source.sql(self.detail, 'DELETE FROM stg_event_baserunners;')
                for bit, role, identity in ((1, 'First', 'F1'), (2, 'Second', 'S1'), (4, 'Third', 'T1')):
                    if mask & bit:
                        self.source.sql(self.detail, f"INSERT INTO stg_event_baserunners VALUES(1,'ANA201904040',1,'{role}','{identity}',NULL,false,false,NULL,NULL,NULL,'{role}',false);")
                self.assertEqual(self.totals(), dict(event_batter=1, before_runner=mask.bit_count(), runner_child=mask.bit_count()))
                batter, _ = self.one('event_batter', 'Batter')
                self.assertIs(batter['child_present'], False)
                self.assertIsNone(batter['is_out'])
                for grain, facts, units in self.cells():
                    self.assertEqual(units, 1)
                    if grain == 'before_runner':
                        self.assertIs(facts['identity_matches'], True)
                        self.assertIs(facts['is_out'], False)
                        self.assertEqual(facts['base_end'], facts['role'])

    def test_identical_cells_aggregate_units_without_parent_or_child_fanout(self):
        self.source.sql(self.snapshot, "INSERT INTO events SELECT 3,game_id,3,season,no_play_flag,plate_appearance_result,outs,base_state,outs_on_play,runs_on_play,batted_trajectory,batter_id FROM events WHERE event_key=1; INSERT INTO states SELECT 3,frame_end_flag,outs_end,base_state_end,runner_first_id_start,runner_second_id_start,runner_third_id_start FROM states WHERE event_key=1; INSERT INTO batting SELECT 3,plate_appearances,grounded_into_double_plays FROM batting WHERE event_key=1; INSERT INTO fielding SELECT 3,errors FROM fielding WHERE event_key=1;")
        self.source.sql(self.detail, "INSERT INTO selected_events SELECT 3,game_id,3,no_play_flag,plate_appearance_result,outs_on_play,runs_on_play,batted_trajectory FROM selected_events WHERE event_key=1; INSERT INTO stg_event_audit SELECT 3,game_id,3,raw_play FROM stg_event_audit WHERE event_key=1; INSERT INTO stg_event_flags SELECT 3,sequence_id,flag FROM stg_event_flags WHERE event_key=1; INSERT INTO stg_event_baserunners SELECT 3,game_id,3,baserunner,runner_id,baserunning_play_type,is_out,run_scored_flag,explicit_out_flag,advanced_on_error_flag,attempted_advance_to_base,base_end,is_advance_attempt FROM stg_event_baserunners WHERE event_key=1;")
        self.assertEqual(self.totals(), dict(event_batter=2, before_runner=2, runner_child=4))
        self.assertTrue(all(units == 2 for _, _, units in self.cells()))

    def test_split_is_original_whole_game_sha256_and_exclusions(self):
        for game in ('ANA201904040', 'SYNTHETIC_A', 'SYNTHETIC_HELDOUT_7', 'BOS201908070'):
            with self.subTest(game=game):
                self.source.sql(self.snapshot, f"UPDATE events SET game_id='{game}';")
                for table in ('selected_events', 'stg_event_audit', 'stg_event_baserunners'):
                    self.source.sql(self.detail, f"UPDATE {table} SET game_id='{game}';")
                expected = 'excluded' if game == 'BOS201908070' else 'heldout' if hashlib.sha256(game.encode()).hexdigest()[0] in '012' else 'training'
                self.assertTrue(self.cells())
                self.assertTrue(all(f['split'] == expected for _, f, _ in self.cells()))

    def test_row_permutation_repeat_and_readonly_bytes_are_exact(self):
        original = self.execute()
        self.assertEqual(original.returncode, 0, original.stderr)
        self.assertTrue(self.cells())
        self.source.sql(self.detail, 'CREATE TEMP TABLE r AS SELECT * FROM stg_event_baserunners ORDER BY baserunner DESC; DELETE FROM stg_event_baserunners; INSERT INTO stg_event_baserunners SELECT * FROM r;')
        before = [p.read_bytes() for p in (self.snapshot, self.detail)]
        again = self.execute()
        self.assertEqual(again.returncode, 0, again.stderr)
        self.assertEqual(again.stdout, original.stdout)
        self.assertEqual(self.execute().stdout, original.stdout)
        self.assertEqual([p.read_bytes() for p in (self.snapshot, self.detail)], before)

    def test_query_cannot_run_without_frozen_shared_assertion_prefix(self):
        self.source.query.write_text(QUERY.read_text())
        result = self.source.execute()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, '')
        self.assertIn('contact_consequence_assertions', result.stderr)

    def test_shared_parent_guard_refuses_before_cells_even_without_children(self):
        self.source.sql(self.detail, 'DELETE FROM stg_event_baserunners; UPDATE selected_events SET outs_on_play=2 WHERE event_key=1;')
        result = self.execute()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, '')
        self.assertIn('unsupported contact consequence joint facts', result.stderr)

    def test_shared_duplicate_and_orphan_guards_are_not_bypassed(self):
        self.source.sql(self.detail, "INSERT INTO stg_event_baserunners SELECT * FROM stg_event_baserunners WHERE baserunner='Third';")
        duplicate = self.execute()
        self.assertNotEqual(duplicate.returncode, 0)
        self.assertEqual(duplicate.stdout, '')
        self.source.sql(self.detail, "DELETE FROM stg_event_baserunners WHERE baserunner='Third'; INSERT INTO stg_event_baserunners VALUES(99,'ANA201904040',99,'Third','ORPHAN',NULL,true,false,NULL,NULL,NULL,NULL,NULL);")
        orphan = self.execute()
        self.assertNotEqual(orphan.returncode, 0)
        self.assertEqual(orphan.stdout, '')


if __name__ == '__main__':
    unittest.main()
