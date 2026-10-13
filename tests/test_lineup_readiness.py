"""Literal synthetic source facts; no real roster, outcome fitting or UI."""
import copy
import csv
import io
import json
import hashlib
import random
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from audits import lineup_readiness as audit


def fixture(game_id='G1', date='2019-03-20', dh=True):
    game = dict(game_id=game_id, date=date, season=2019, is_regular_season=True,
                home_league='AL', away_league='AL', use_dh=dh, bat_first_side='Away',
                park_id='P', home_team_id='H', away_team_id='A',
                home_team_name='Home source name', away_team_name='Away source name',
                results=[dict(home_team_id='H', away_team_id='A', forfeit_flag=False,
                              suspension_flag=False, tie_flag=False)])
    people, appearances = [], []
    for side, prefix in [('away', 'a'), ('home', 'h')]:
        pitcher = prefix+'p'
        bats = {str(i): prefix+str(i) for i in range(1, 10)}
        if dh:
            bats['0'] = pitcher
        else:
            bats['9'] = pitcher
        field = {'1': pitcher, **{str(i+1): prefix+str(i) for i in range(1, 9)}}
        if dh:
            field['10'] = prefix+'9'
        game[side+'_starting_pitcher_id'] = pitcher
        game['lineup_map_'+side] = bats
        game['fielding_map_'+side] = field
        for player in sorted(set(bats.values())):
            people.append(dict(player_id=player, first_name=player, last_name='Source',
                               bats='Right', throws='Left'))
        for position, player in field.items():
            appearances.append(dict(game_id=game_id, player_id=player, side=side.title(),
                                    fielding_position=int(position), start_event_id=1,
                                    end_event_id=90))
    return [game], people, appearances


def run(parts):
    return audit.inventory(*parts)


def side_issues(report, side='Away'):
    return next(s['issues'] for s in report['checks'][0]['sides'] if s['side']==side)


class LineupReadinessTests(unittest.TestCase):
    def test_dh_names_slots_and_pitcher_are_bound_to_literal_source_maps(self):
        parts=fixture(); before=copy.deepcopy(parts); report=run(parts)
        self.assertEqual(report['coverage']['candidate_games'],1)
        self.assertEqual(report['coverage']['sides'],2)
        team=report['witnesses']['primary']['teams'][0]
        self.assertEqual(team['side'],'Away')
        self.assertEqual(team['pitcher_id'],'ap')
        self.assertEqual(team['dh_slot'],9)
        self.assertEqual(team['engine_dh_slot'],8)
        self.assertEqual([p['player_id'] for p in team['batting_order']],['a'+str(i) for i in range(1,10)])
        self.assertEqual(team['batting_order'][0]['name'],'a1 Source')
        self.assertEqual(team['fielding'][0]['position'],1)
        self.assertEqual(team['fielding'][0]['player_id'],'ap')
        self.assertEqual(team['fielding'][-1]['position'],10)
        self.assertEqual(parts,before)

    def test_no_dh_pitcher_bats_once_and_is_not_an_extra_slot(self):
        report=run(fixture(dh=False))
        self.assertEqual(report['coverage']['candidate_games'],1)
        team=report['witnesses']['primary']['teams'][0]
        self.assertIsNone(team['dh_slot']);self.assertIsNone(team['engine_dh_slot'])
        self.assertEqual(team['batting_order'][-1]['player_id'],'ap')
        self.assertEqual(len(team['fielding']),9)
        self.assertIsNone(report['witnesses']['dh'])

    def test_wrong_keys_and_duplicate_ids_are_not_count_only_completeness(self):
        for mutate,reason in [
            (lambda g:g['lineup_map_away'].__setitem__('11',g['lineup_map_away'].pop('9')),'lineup_keys'),
            (lambda g:g['fielding_map_away'].__setitem__('11',g['fielding_map_away'].pop('9')),'fielding_keys'),
            (lambda g:g['lineup_map_away'].__setitem__('2','a1'),'lineup_ids'),
            (lambda g:g['fielding_map_away'].__setitem__('3','a1'),'fielding_ids'),
            (lambda g:g['lineup_map_away'].__setitem__('2',''),'lineup_ids')]:
            with self.subTest(reason=reason):
                parts=fixture();mutate(parts[0][0]);report=run(parts)
                self.assertIn(reason,side_issues(report));self.assertEqual(report['coverage']['candidate_games'],0)

    def test_pitcher_and_dh_relations_do_not_follow_batting_order_ordinals(self):
        for dh,mutate,reason in [
            (True,lambda g:g['fielding_map_away'].__setitem__('1','a1'),'pitcher_fielding'),
            (True,lambda g:g['lineup_map_away'].__setitem__('0','a1'),'pitcher_lineup'),
            (True,lambda g:g['lineup_map_away'].__setitem__('1','ap'),'role_sets'),
            (False,lambda g:g['lineup_map_away'].__setitem__('9','other'),'role_sets'),
            (True,lambda g:g['fielding_map_away'].__setitem__('10','other'),'role_sets')]:
            with self.subTest(dh=dh,reason=reason):
                parts=fixture(dh=dh);mutate(parts[0][0]);report=run(parts)
                self.assertIn(reason,side_issues(report));self.assertEqual(report['coverage']['candidate_games'],0)

    def test_all_failures_are_reported_not_hidden_by_priority(self):
        parts=fixture();g=parts[0][0];g['lineup_map_away']['2']='a1';g['fielding_map_away']['1']='a2'
        report=run(parts);issues=side_issues(report)
        self.assertIn('lineup_ids',issues);self.assertIn('pitcher_fielding',issues);self.assertIn('role_sets',issues)
        self.assertEqual(report['coverage']['side_issue_counts']['lineup_ids'],1)

    def test_unknown_dh_and_null_maps_remain_unknown_not_false_or_empty(self):
        for key in ('use_dh','lineup_map_away','fielding_map_away'):
            with self.subTest(key=key):
                parts=fixture();parts[0][0][key]=None;report=run(parts)
                self.assertEqual(report['coverage']['candidate_games'],0)
                self.assertTrue(report['checks'][0]['issues'] or side_issues(report))
                self.assertIsNone(report['witnesses']['primary'])

    def test_missing_ambiguous_or_unnamed_bios_cannot_fabricate_a_named_player(self):
        for mode in ('missing','duplicate','blank'):
            with self.subTest(mode=mode):
                parts=fixture();person=next(p for p in parts[1] if p['player_id']=='a1')
                if mode=='missing':parts[1].remove(person)
                elif mode=='duplicate':parts[1].append(copy.deepcopy(person))
                else:person['first_name']=' '
                report=run(parts);self.assertIn('person_lookup',side_issues(report))
                self.assertEqual(report['coverage']['candidate_games'],0)
                self.assertEqual(report['coverage']['missing_start_lookups'],1)

    def test_unknown_hands_are_counted_and_preserved_without_an_eligibility_cutoff(self):
        parts=fixture();person=next(p for p in parts[1] if p['player_id']=='a1')
        person['bats']=None;person['throws']='Unknown';report=run(parts)
        self.assertEqual(report['coverage']['candidate_games'],1)
        self.assertEqual(report['coverage']['unknown_hand_slots'],2)
        player=report['witnesses']['primary']['teams'][0]['batting_order'][0]
        self.assertIsNone(player['bats']);self.assertEqual(player['throws'],'Unknown')

    def test_missing_park_does_not_add_an_unpreregistered_lineup_cutoff(self):
        parts=fixture();parts[0][0]['park_id']=None
        report=run(parts)
        self.assertEqual(report['coverage']['candidate_games'],1)
        self.assertIsNone(report['witnesses']['primary']['park_id'])

    def test_same_player_on_both_starting_teams_is_not_admitted(self):
        parts=fixture();g=parts[0][0]
        g['lineup_map_home']['1']='a1';g['fielding_map_home']['2']='a1'
        report=run(parts);self.assertIn('cross_team_ids',report['checks'][0]['issues'])
        self.assertEqual(report['coverage']['candidate_games'],0)

    def test_result_flags_and_team_mismatch_are_explicit_unknowns_or_exclusions(self):
        for value in (None,True):
            for flag in ('forfeit_flag','suspension_flag','tie_flag'):
                with self.subTest(flag=flag,value=value):
                    parts=fixture();parts[0][0]['results'][0][flag]=value;report=run(parts)
                    self.assertIn('result_metadata',report['checks'][0]['issues'])
                    self.assertEqual(report['coverage']['candidate_games'],0)
        parts=fixture();parts[0][0]['results'][0]['home_team_id']='OTHER'
        self.assertIn('result_metadata',run(parts)['checks'][0]['issues'])

    def test_global_grains_are_not_silently_collapsed(self):
        parts=fixture();parts[0].append(copy.deepcopy(parts[0][0]))
        with self.assertRaisesRegex(ValueError,'Duplicate game'):run(parts)
        parts=fixture();parts[0][0]['results']*=2
        self.assertIn('result_metadata',run(parts)['checks'][0]['issues'])
        parts=fixture();parts[2].append(copy.deepcopy(parts[2][0]))
        with self.assertRaisesRegex(ValueError,'Duplicate appearance'):run(parts)

    def test_earliest_witness_and_rule_controls_ignore_input_order_and_outcomes(self):
        early=fixture('Z','2019-03-20',True);late=fixture('A','2019-03-28',False)
        parts=(early[0]+late[0],early[1],early[2]+late[2])
        # Same stable people occur in both games; add the no-DH pitcher's already-known lookup only once.
        report=run(parts)
        self.assertEqual(report['coverage']['candidate_games'],2)
        self.assertEqual(report['witnesses']['primary']['game_id'],'Z')
        self.assertEqual(report['witnesses']['dh']['game_id'],'Z')
        self.assertEqual(report['witnesses']['no_dh']['game_id'],'A')
        shuffled=copy.deepcopy(parts);rng=random.Random(19)
        for rows in shuffled:rng.shuffle(rows)
        self.assertEqual(run(shuffled),report)

    def test_appearance_windows_do_not_fill_roster_gaps_or_choose_an_opening_bench(self):
        early=fixture('E','2019-04-01');late=fixture('L','2019-09-01')
        parts=(early[0]+late[0],early[1],early[2]+late[2]);report=run(parts)
        self.assertEqual(len(report['membership']['profiles']),20)
        profile=next(p for p in report['membership']['profiles'] if p['team_id']=='A' and p['player_id']=='a1')
        self.assertEqual(profile,dict(team_id='A',player_id='a1',games=2,
            first_date='2019-04-01',last_date='2019-09-01',positions=[2]))
        self.assertEqual(report['membership']['basis'],'observed appearances only; not active-roster intervals or opening-bench eligibility')
        self.assertNotIn('bench',report['witnesses']['primary']['teams'][0])

    def test_unknown_position_and_reversed_terminal_range_are_retained_not_invented(self):
        parts=fixture();row=parts[2][0];row['fielding_position']=0;row['start_event_id']=92;row['end_event_id']=91
        report=run(parts)
        self.assertEqual(report['coverage']['unknown_position_appearances'],1)
        self.assertEqual(report['membership']['reversed_ranges'],[row])
        self.assertEqual(report['coverage']['candidate_games'],1)
        self.assertEqual(report['coverage']['uncorroborated_start_slots'],1)

    def test_appearance_batting_roles_and_unknown_byte_values_are_preserved(self):
        for position in (11,12,255):
            with self.subTest(position=position):
                parts=fixture();parts[2][0]['fielding_position']=position
                report=run(parts)
                self.assertEqual(report['coverage']['position_appearance_counts'][str(position)],1)
                self.assertEqual(report['coverage']['unknown_position_appearances'],int(position==255))
                self.assertEqual(report['coverage']['candidate_games'],1)
                self.assertEqual(report['coverage']['uncorroborated_start_slots'],1)
                profile=next(p for p in report['membership']['profiles'] if p['player_id']==parts[2][0]['player_id'])
                self.assertIn(position,profile['positions'])

    def test_missing_same_position_appearance_does_not_manufacture_start_confirmation(self):
        parts=fixture();parts[2].pop(0);report=run(parts)
        self.assertEqual(report['coverage']['start_appearance_slots'],20)
        self.assertEqual(report['coverage']['corroborated_start_slots'],19)
        self.assertEqual(report['coverage']['uncorroborated_start_slots'],1)
        self.assertEqual(report['coverage']['candidate_games'],1)

    def test_orphan_appearances_and_invalid_cohort_cannot_pollute_membership(self):
        parts=fixture();parts[2][0]['game_id']='orphan'
        with self.assertRaisesRegex(ValueError,'Orphan appearance'):run(parts)
        parts=fixture();parts[0][0]['season']=2020
        with self.assertRaisesRegex(ValueError,'2019 cohort'):run(parts)


class ExportTests(unittest.TestCase):
    maxDiff=None
    def test_literal_json_maps_and_nulls_survive_csv_decoding(self):
        game=fixture()[0][0];stream=io.StringIO();writer=csv.writer(stream)
        writer.writerow(['record_type','payload']);writer.writerow(['game',json.dumps(game)])
        decoded=audit.read_export(stream.getvalue())
        self.assertEqual(decoded,{'game':[game],'person':[],'appearance':[],'manifest':[]})

    def test_duplicate_json_keys_headers_and_unknown_records_are_not_silently_accepted(self):
        for text in ['record_type,payload\ngame,"{""game_id"":""A"",""game_id"":""B""}"\n',
                     'record_type,record_type,payload\ngame,game,{}\n',
                     'record_type,payload\nother,{}\n']:
            with self.subTest(text=text),self.assertRaises(ValueError):audit.read_export(text)


class PublicationTests(unittest.TestCase):
    def test_cli_help_exposes_local_snapshot_supplement_and_output(self):
        result=subprocess.run([sys.executable,'-Werror','-m','audits.lineup_readiness','--help'],capture_output=True,text=True,check=True)
        for arg in ('snapshot','supplement','output'):self.assertIn(arg,result.stdout)
        self.assertEqual(result.stderr,'')

    def test_existing_output_refuses_before_reading_missing_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);output=root/'report.json';output.write_text('keep')
            with self.assertRaisesRegex(ValueError,'Refusing to overwrite'):
                audit.publish(root/'missing-main',root/'missing-detail',output)
            self.assertEqual(output.read_text(),'keep')
            self.assertEqual(sorted(p.name for p in root.iterdir()),['report.json'])

    def test_unaccepted_databases_are_rejected_before_native_query_or_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);one=root/'one';two=root/'two';one.write_bytes(b'not the snapshot');two.write_bytes(b'not the supplement')
            with self.assertRaisesRegex(ValueError,'Unsupported.*digest'):
                audit.publish(one,two,root/'report.json')
            self.assertEqual(sorted(p.name for p in root.iterdir()),['one','two'])

    def test_final_input_reread_prevents_publication_after_change(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source';source.write_bytes(b'original')
            before=(hashlib.sha256(source.read_bytes()).hexdigest(),);source.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                audit._write_report({'literal':True},(source,),before,root/'out.json')
            self.assertFalse((root/'out.json').exists())

    def test_stable_writer_is_exclusive_and_sorted_without_mutating_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source';source.write_bytes(b'stable')
            before=(hashlib.sha256(source.read_bytes()).hexdigest(),);output=root/'out.json';report={'z':2,'a':1}
            audit._write_report(report,(source,),before,output)
            self.assertEqual(output.read_text(),json.dumps(report,sort_keys=True,indent=2)+'\n')
            self.assertEqual(report,{'z':2,'a':1})
            with self.assertRaises(FileExistsError):audit._write_report({'bad':1},(source,),before,output)
            self.assertEqual(json.loads(output.read_text()),report)


class NativeExportTests(unittest.TestCase):
    def test_real_duckdb_projection_preserves_maps_people_results_and_raw_ranges(self):
        games,people,appearances=fixture()
        game=games[0]
        def literal(value):
            if value is None:return 'NULL'
            if isinstance(value,bool):return str(value).lower()
            if isinstance(value,int):return str(value)
            if isinstance(value,dict):
                return 'map(['+','.join(value)+'],['+','.join(literal(v) for v in value.values())+'])'
            return "'"+value.replace("'","''")+"'"
        schema='CREATE TABLE games AS SELECT '+','.join(literal(value)+' AS '+key for key,value in game.items() if key!='results')+';'
        schema+='CREATE TABLE results AS SELECT '+literal(game['game_id'])+' AS game_id,'+','.join(literal(value)+' AS '+key for key,value in game['results'][0].items())+';'
        for name,rows in [('people',people)]:
            schema+='CREATE TABLE '+name+' AS '+ ' UNION ALL '.join('SELECT '+','.join(literal(value)+' AS '+key for key,value in row.items()) for row in rows)+';'
        schema+="ATTACH ':memory:' AS detail; CREATE TABLE detail.stg_game_fielding_appearances AS "+' UNION ALL '.join('SELECT '+','.join(literal(value)+' AS '+key for key,value in row.items()) for row in appearances)+';'
        schema+="CREATE TABLE audit_manifest AS SELECT 'f03-spike-1' AS schema_version,'2019-filtered-extract-v1' AS query_version,'"+audit.CATALOG+"' AS catalog_sha256,'notice' AS attribution; SET enable_external_access=false;"
        query=Path('audits/2019-lineup-readiness.sql').read_text()
        result=subprocess.run(['duckdb',':memory:','-init','/dev/null','-bail','-batch','-csv','-c',schema+query],capture_output=True,text=True,check=True)
        self.assertEqual(result.stderr,'')
        exported=audit.read_export(result.stdout)
        self.assertEqual(exported['game'],games)
        self.assertEqual(sorted(exported['person'],key=lambda p:p['player_id']),sorted(people,key=lambda p:p['player_id']))
        self.assertEqual(sorted(exported['appearance'],key=lambda p:(p['side'],p['fielding_position'])),sorted(appearances,key=lambda p:(p['side'],p['fielding_position'])))
        self.assertEqual(audit.inventory(exported['game'],exported['person'],exported['appearance']),run((games,people,appearances)))


if __name__=='__main__':unittest.main()
