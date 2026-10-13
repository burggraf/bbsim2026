"""Lossless notation inventory; no legal retirement/timing inference."""
import csv
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from audits.raw_mechanisms import lex, inventory, read_export, root_family, check_scope, SOURCE_TOTALS, SUPPLEMENT_SHA
from audits.outcome_details import SOURCE_COMMIT, CATALOG_SHA, SNAPSHOT_SHA
from tests.test_outcome_details import FIXTURE

class RawMechanismTests(unittest.TestCase):
    def test_simple_base_modifiers_and_advances_reconstruct_exactly(self):
        value=lex('S9/L9S.2-H;1-3')
        self.assertEqual(value['base_atoms'],['S9'])
        self.assertEqual(value['base_separators'],[])
        self.assertEqual(value['modifiers'],['L9S'])
        self.assertEqual(value['advances'],['2-H','1-3'])
        self.assertEqual(value['raw'],'S9/L9S.2-H;1-3')
        self.assertEqual(value['status'],'split')

    def test_explicit_compound_and_double_steal_are_not_one_atom(self):
        for raw,atoms,separators in [('K+CS2(24)/DP',['K','CS2(24)'],['+']),
                                    ('SB3;SB2',['SB3','SB2'],[';'])]:
            value=lex(raw)
            self.assertEqual(value['base_atoms'],atoms)
            self.assertEqual(value['base_separators'],separators)

    def test_nested_annotation_delimiters_stay_inside_advance(self):
        raw='K.BX1(23+E3/TH)(UR);1-2(WP)'
        value=lex(raw)
        self.assertEqual(value['base_atoms'],['K'])
        self.assertEqual(value['modifiers'],[])
        self.assertEqual(value['advances'],['BX1(23+E3/TH)(UR)','1-2(WP)'])
        self.assertEqual(value['status'],'split')

    def test_nested_base_credit_annotation_is_not_an_outer_modifier_or_compound(self):
        value=lex('CS2(2E4/TH).1-3')
        self.assertEqual(value['base_atoms'],['CS2(2E4/TH)'])
        self.assertEqual(value['modifiers'],[])
        value=lex('PO1(13+E3/TH).1-2')
        self.assertEqual(value['base_atoms'],['PO1(13+E3/TH)'])
        self.assertEqual(value['base_separators'],[])

    def test_flat_and_nested_forms_round_trip_without_reordering(self):
        for base in ['K','S7','5(2)4(1)3','K+WP','SBH;SB2','PO1(13+E3/TH)']:
            for modifier in ['', '/DP','/G56/MREV']:
                for advance in ['', '.3-H;1-3;B-1','.BX2(8434)',' .1-2(E4/TH)(UR)(NR)']:
                    raw=base+modifier+advance
                    value=lex(raw)
                    self.assertEqual(value['status'],'split')
                    head=value['base_atoms'][0]
                    for delimiter,atom in zip(value['base_separators'],value['base_atoms'][1:]):
                        head+=delimiter+atom
                    head+=''.join('/'+v for v in value['modifiers'])
                    if value['advances']:head+='.'+';'.join(value['advances'])
                    self.assertEqual(head,raw)

    def test_unbalanced_and_multiple_advance_sections_remain_unknown_not_illegal(self):
        for raw in ['K.BX1(23','S7.1-2.2-3','K)']:
            value=lex(raw)
            self.assertEqual(value['raw'],raw)
            self.assertEqual(value['status'],'structural_unknown')
            self.assertTrue(value['unknown_reason'])

def row(**changes):
    value=dict(event_key='1',game_id='GAME1',event_id='1',no_play_flag='false',
               result='StrikeOut',outs='0',runs='0',trajectory='',angle='',
               raw_play='K+PB.B-1',flags='[]',credits='[]',runners='[]')
    value.update(changes)
    return value

class RawInventoryTests(unittest.TestCase):
    def test_unknown_symbols_are_preserved_without_legal_or_out_inference(self):
        self.assertEqual(root_family('UNKNOWN123'),'unknown_symbol')
        self.assertEqual(root_family('5(2)4(1)3'),'numeric_fielding_notation')
        self.assertEqual(root_family('5E3'),'error_notation')
        self.assertEqual(root_family('PO1(1E3/TH)'),'pickoff_notation')
        self.assertEqual(root_family('DGR'),'ground_rule_double_notation')
        report=inventory([row(raw_play='UNKNOWN123')])
        self.assertEqual(report['totals']['outs'],0)
        self.assertEqual(report['unknown_roots'],{'UNKNOWN123':1})
        self.assertEqual(report['examples'][0]['event']['raw_play'],'UNKNOWN123')

    def test_each_structural_unknown_keeps_its_full_keyed_record_and_reason(self):
        report=inventory([row(raw_play='K.'),row(event_key='2',event_id='2',raw_play='S7..1-2')])
        self.assertEqual(len(report['structural_unknown_events']),2)
        self.assertEqual(report['structural_unknown_reasons'],{'empty_outer_segment':1,'multiple_advance_delimiters':1})
        self.assertEqual([v['event']['raw_play'] for v in report['structural_unknown_events']],['K.','S7..1-2'])

    def test_no_play_and_nested_notation_are_separate_from_real_compound_counts(self):
        report=inventory([row(),row(event_key='2',event_id='2',no_play_flag='true',result='',raw_play='NP'),
                          row(event_key='3',event_id='3',raw_play='K.BX1(23+E3/TH)')])
        self.assertEqual(report['totals']['events'],3)
        self.assertEqual(report['totals']['real_events'],2)
        self.assertEqual(report['totals']['no_play_events'],1)
        self.assertEqual(report['totals']['compound_real_events'],1)
        self.assertEqual(report['root_families']['passed_ball_notation'],1)
        self.assertEqual(report['totals']['outs'],0)

    def test_child_credit_segments_and_runner_out_flags_not_literal_x_drive_units(self):
        credits=[dict(sequence_id=1,fielding_position=6,fielding_play='Assist'),
                 dict(sequence_id=2,fielding_position=6,fielding_play='Assist'),
                 dict(sequence_id=3,fielding_position=3,fielding_play='Putout'),
                 dict(sequence_id=4,fielding_position=6,fielding_play='Assist'),
                 dict(sequence_id=5,fielding_position=4,fielding_play='Error')]
        runners=[dict(baserunner='Batter',runner_id='B1',baserunning_play_type='Advance',
                      is_out=False,run_scored_flag=False,explicit_out_flag=True,
                      advanced_on_error_flag=True,attempted_advance_to_base='Second',
                      base_end='Second',is_advance_attempt=True)]
        report=inventory([row(outs='1',raw_play='K.BX2(6E4)',credits=json.dumps(credits),runners=json.dumps(runners))])
        self.assertEqual(report['supplied_credit_units'],dict(putouts=1,assists=2,errors=1))
        self.assertEqual(report['totals']['runner_outs'],0)
        self.assertEqual(report['runner_labels'],{'Advance':1})

    def test_fielders_choice_record_is_preserved_but_not_charged_as_error_or_out(self):
        credits=json.dumps([dict(sequence_id=1,fielding_position=6,fielding_play='FieldersChoice')])
        report=inventory([row(raw_play='FC6/G6S',credits=credits)])
        self.assertEqual(report['supplied_credit_units'],dict(putouts=0,assists=0,errors=0))
        self.assertEqual(report['source_fielding_plays'],{'FieldersChoice':1})
        self.assertEqual(report['totals']['credit_rows'],1)

    def test_source_label_order_and_unknown_contact_tags_are_preserved(self):
        flags=json.dumps([dict(sequence_id=1,flag='NoDoublePlay'),dict(sequence_id=2,flag='UnknownLabel')])
        report=inventory([row(flags=flags,trajectory='Unknown',angle='FoulLine')])
        self.assertEqual(report['source_flags'],{'NoDoublePlay':1,'UnknownLabel':1})
        self.assertEqual(report['raw_trajectories'],{'Unknown':1})
        self.assertEqual(report['raw_angles'],{'FoulLine':1})
        self.assertEqual(report['examples'][0]['event']['flags'],flags)

    def test_event_input_order_does_not_change_counts_or_keyed_examples(self):
        rows=[row(event_key='3',event_id='3'),row(event_key='1',event_id='1'),row(event_key='2',event_id='2')]
        self.assertEqual(inventory(rows),inventory(list(reversed(rows))))

    def test_invalid_grain_required_units_and_child_types_are_rejected(self):
        for rows in [[],[row(),row()], [row(raw_play='')], [row(outs='')],
                     [row(credits='null')], [row(flags='[{"sequence_id":1,"flag":null}]')],
                     [row(runners='[{"baserunner":"Batter","is_out":null}]')],
                     [row(no_play_flag='true',result='',outs='1',credits='[]')]]:
            with self.subTest(rows=rows),self.assertRaises(ValueError):inventory(rows)

    def test_duplicate_json_fields_and_invalid_supplied_label_types_are_rejected(self):
        runners=[dict(baserunner='Batter',runner_id='B1',baserunning_play_type=123,
                      is_out=False,run_scored_flag=False,explicit_out_flag=False,
                      advanced_on_error_flag=False,attempted_advance_to_base='First',
                      base_end='First',is_advance_attempt=True)]
        for changes in [dict(flags='[{"sequence_id":1,"flag":"ForceOut","flag":"AppealPlay"}]'),
                        dict(runners=json.dumps(runners))]:
            with self.subTest(changes=changes),self.assertRaises(ValueError):inventory([row(**changes)])

    def test_duplicate_missing_and_extra_csv_headers_fail(self):
        for header in ['event_key,event_key','event_key','event_key,extra']:
            with self.subTest(header=header),self.assertRaises(ValueError):
                list(read_export(io.StringIO(header+'\n1,1\n')))

EXPORT_FIXTURE=FIXTURE+"""
ALTER TABLE selected_events ADD COLUMN plate_appearance_result VARCHAR DEFAULT 'InPlayOut';
ALTER TABLE selected_events ADD COLUMN batted_trajectory VARCHAR DEFAULT 'GroundBall';
ALTER TABLE selected_events ADD COLUMN batted_location_angle VARCHAR DEFAULT 'Default';
ALTER TABLE stg_event_baserunners ADD COLUMN baserunning_play_type VARCHAR DEFAULT 'Advance';
ALTER TABLE stg_event_baserunners ADD COLUMN explicit_out_flag BOOLEAN DEFAULT FALSE;
ALTER TABLE stg_event_baserunners ADD COLUMN advanced_on_error_flag BOOLEAN DEFAULT FALSE;
ALTER TABLE stg_event_baserunners ADD COLUMN attempted_advance_to_base VARCHAR DEFAULT 'First';
ALTER TABLE stg_event_baserunners ADD COLUMN base_end VARCHAR DEFAULT 'Out';
ALTER TABLE stg_event_baserunners ADD COLUMN is_advance_attempt BOOLEAN DEFAULT TRUE;
"""

class RawCliTests(unittest.TestCase):
    def test_existing_output_is_refused_before_inputs_are_read(self):
        with tempfile.TemporaryDirectory() as folder:
            output=Path(folder)/'existing.json';output.write_text('preserve')
            result=subprocess.run([sys.executable,'-m','audits.raw_mechanisms',
                str(Path(folder)/'absent.csv'),str(Path(folder)/'absent.json'),str(output)],
                text=True,capture_output=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('Refusing to overwrite an existing file',result.stderr)
            self.assertEqual(output.read_text(),'preserve')

    def test_cli_refuses_incomplete_cohort_without_publishing_report(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);events=root/'events.csv';manifest=root/'manifest.json';output=root/'report.json'
            with events.open('w',newline='') as f:
                writer=csv.DictWriter(f,fieldnames=row().keys());writer.writeheader();writer.writerow(row())
            manifest.write_text(json.dumps(dict(format='outcome-detail-2019-v1',source_commit=SOURCE_COMMIT,
                catalog_sha256=CATALOG_SHA,snapshot_sha256=SNAPSHOT_SHA,database_sha256=SUPPLEMENT_SHA,
                snapshot_id=857,attribution='Retrosheet')))
            result=subprocess.run([sys.executable,'-m','audits.raw_mechanisms',str(events),str(manifest),str(output)],
                                  text=True,capture_output=True)
            self.assertNotEqual(result.returncode,0)
            self.assertFalse(output.exists())

class RawScopeTests(unittest.TestCase):
    def test_pinned_manifest_and_complete_cohort_controls_are_required(self):
        report=dict(games=2429,totals=dict(SOURCE_TOTALS),supplied_credit_units=dict(putouts=130270,assists=43968,errors=2836))
        manifest=dict(format='outcome-detail-2019-v1',source_commit=SOURCE_COMMIT,
                      catalog_sha256=CATALOG_SHA,snapshot_sha256=SNAPSHOT_SHA,
                      database_sha256=SUPPLEMENT_SHA,snapshot_id=857,attribution='Retrosheet')
        self.assertIsNone(check_scope(report,manifest))
        for changes in [dict(source_commit='wrong'),dict(database_sha256='wrong'),
                        dict(catalog_sha256='wrong'),dict(snapshot_id=856),dict(attribution='')]:
            with self.subTest(changes=changes),self.assertRaises(ValueError):
                check_scope(report,dict(manifest,**changes))
        report['totals']['real_events']-=1
        with self.assertRaises(ValueError):check_scope(report,manifest)

class RawExportTests(unittest.TestCase):
    def run_export(self, mutation=''):
        sql=Path('audits/2019-raw-mechanisms.sql').read_text()
        return subprocess.run(['duckdb','-bail','-batch','-init','/dev/null','-csv','-nullvalue','',':memory:'],
            input='SET enable_external_access=FALSE;\n'+EXPORT_FIXTURE+mutation+sql+'SELECT * FROM mechanism_export ORDER BY event_key;',
            text=True,capture_output=True)

    def test_independent_child_aggregation_preserves_labels_without_fanout(self):
        result=self.run_export("INSERT INTO stg_event_flags VALUES (1,2,'UnknownLabel'); INSERT INTO stg_event_baserunners SELECT event_key,game_id,event_id,'First','F1',FALSE,FALSE,baserunning_play_type,FALSE,FALSE,attempted_advance_to_base,'First',FALSE FROM stg_event_baserunners;")
        self.assertEqual(result.returncode,0,result.stderr)
        rows=list(csv.DictReader(io.StringIO(result.stdout)))
        self.assertEqual(len(rows),1)
        self.assertEqual([x['flag'] for x in json.loads(rows[0]['flags'])],['GroundBall','UnknownLabel'])
        self.assertEqual(len(json.loads(rows[0]['credits'])),2)
        self.assertEqual(len(json.loads(rows[0]['runners'])),2)

    def test_missing_duplicate_mismatched_and_orphan_source_children_fail(self):
        for mutation in ['DELETE FROM stg_event_audit;',
                         'INSERT INTO stg_event_audit SELECT * FROM stg_event_audit;',
                         "UPDATE stg_event_audit SET game_id='WRONG';",
                         'UPDATE stg_event_flags SET event_key=99;',
                         'UPDATE stg_event_fielding_plays SET event_key=99;',
                         'UPDATE stg_event_baserunners SET event_key=99;',
                         'INSERT INTO selected_events SELECT * FROM selected_events;',
                         'INSERT INTO stg_event_flags SELECT * FROM stg_event_flags;',
                         'INSERT INTO stg_event_fielding_plays SELECT * FROM stg_event_fielding_plays;',
                         'INSERT INTO stg_event_baserunners SELECT * FROM stg_event_baserunners;',
                         'UPDATE stg_event_flags SET sequence_id=NULL;',
                         'UPDATE stg_event_fielding_plays SET sequence_id=NULL;',
                         "UPDATE stg_event_baserunners SET game_id='WRONG';",
                         "UPDATE stg_event_audit SET raw_play='';"]:
            with self.subTest(mutation=mutation):
                result=self.run_export(mutation)
                self.assertNotEqual(result.returncode,0)

if __name__=='__main__':unittest.main()
