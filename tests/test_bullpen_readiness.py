"""Synthetic personnel/counts only; no real source rosters committed."""
import copy
import hashlib
import json
from pathlib import Path
import random
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from audits import bullpen_readiness as audit
from test_lineup_readiness import fixture

KEYS=('strikeout','walk','hbp','single','double','triple','home_run','roe','contact_out')
def parts():
    games,people,rows=fixture()
    for side,prefix in [('Away','a'),('Home','h')]:
        people.append(dict(player_id=prefix+'r',first_name=prefix+'r',last_name='Source',bats=None,throws='Unknown'))
        rows.append(dict(game_id='G1',player_id=prefix+'r',side=side,fielding_position=1,start_event_id=60,end_event_id=90))
    profiles=[dict(player_id=p,events=3,games=1,selected_pa=3,non_pa=0,included_pa=3,counts=dict(zip(KEYS,[1,0,0,1,0,0,0,0,1])),exclusions=dict(excluded_ibb=0,excluded_interference=0)) for p in ['ap','hp','ar','hr']]
    report=dict(format='2019-player-opportunity-support-v1',counts=dict(zip(KEYS,[100]*9)),roles=dict(pitcher=dict(profiles=profiles)))
    return games,people,rows,report

def run(p):return audit.inventory(*p,target='G1')
def team(r,side='Away'):return next(t for t in r['teams'] if t['side']==side)

class InventoryTests(unittest.TestCase):
    def test_exact_pitching_cast_not_bench_or_eligibility(self):
        p=parts();before=copy.deepcopy(p);r=run(p);t=team(r)
        self.assertEqual([x['player_id'] for x in t['observed_pitchers']],['ap','ar'])
        self.assertEqual(t['starter_id'],'ap');self.assertEqual(t['observed_nonstarter_pitcher_ids'],['ar'])
        self.assertEqual(r['pregame_bullpen_status'],'unsupported')
        self.assertTrue(all(x['eligible_at_start'] is None for x in t['observed_pitchers']))
        self.assertEqual(p,before)
    def test_nonpitching_roles_remain_literal_not_relief_candidates(self):
        p=parts();p[1].append(dict(player_id='pinch',first_name='Pinch',last_name='Source',bats='Left',throws='Right'))
        p[2].append(dict(game_id='G1',player_id='pinch',side='Away',fielding_position=11,start_event_id=80,end_event_id=90))
        t=team(run(p));self.assertEqual(t['observed_nonstarter_pitcher_ids'],['ar'])
        row=next(x for x in t['nonstarting_participants'] if x['player_id']=='pinch')
        self.assertEqual(row['appearances'][0]['fielding_position'],11)
        self.assertNotIn('missing_pitcher_profile',row['issues'])
    def test_raw_unknown_and_reversed_ranges_are_preserved(self):
        p=parts();a=p[2][-2];a['start_event_id']=255;a['end_event_id']=0
        t=team(run(p));row=next(x for x in t['observed_pitchers'] if x['player_id']=='ar')
        self.assertEqual(row['appearances'],[a]);self.assertIsNone(row['bats']);self.assertEqual(row['throws'],'Unknown')
        a['start_event_id']=None;a['end_event_id']=None
        self.assertIsNone(next(x for x in team(run(p))['observed_pitchers'] if x['player_id']=='ar')['appearances'][0]['start_event_id'])
    def test_prior_same_date_future_and_multi_team_are_not_eligibility(self):
        p=parts();template=copy.deepcopy(p[2])
        for gid,day in [('E','2019-03-19'),('S','2019-03-20'),('F','2019-09-01')]:
            g=copy.deepcopy(p[0][0]);g['game_id']=gid;g['date']=day;p[0].append(g)
            rows=[dict(x,game_id=gid) for x in template];p[2].extend(rows)
        p[2].append(dict(game_id='F',player_id='ar',side='Home',fielding_position=1,start_event_id=100,end_event_id=120))
        row=next(x for x in team(run(p))['season_pitchers'] if x['player_id']=='ar')
        self.assertEqual(row['pitching_games'],4);self.assertEqual(row['date_partition'],dict(before=1,on=2,after=1,unknown=0))
        self.assertEqual(row['observed_team_ids'],['A','H']);self.assertIsNone(row['eligible_at_start'])
    def test_zero_missing_and_invalid_profile_are_distinct(self):
        p=parts();profile=p[3]['roles']['pitcher']['profiles'][2]
        profile['selected_pa']=profile['included_pa']=0;profile['non_pa']=3
        profile['counts']=dict.fromkeys(KEYS,0)
        row=next(x for x in team(run(p))['observed_pitchers'] if x['player_id']=='ar')
        self.assertEqual(row['profile']['included_pa'],0)
        p[3]['roles']['pitcher']['profiles'].remove(profile)
        row=next(x for x in team(run(p))['observed_pitchers'] if x['player_id']=='ar')
        self.assertIsNone(row['profile']);self.assertIn('missing_pitcher_profile',row['issues'])
        for mutation in [lambda x:x['counts'].update(strikeout=True),lambda x:x.update(included_pa=99),lambda x:x['counts'].update(strikeout=-1)]:
            bad=parts();mutation(bad[3]['roles']['pitcher']['profiles'][0])
            with self.assertRaises(ValueError):run(bad)
    def test_missing_and_ambiguous_bio_are_findings(self):
        for duplicate in [False,True]:
            p=parts();bio=next(x for x in p[1] if x['player_id']=='ar')
            if duplicate:p[1].append(copy.deepcopy(bio))
            else:p[1].remove(bio)
            row=next(x for x in team(run(p))['observed_pitchers'] if x['player_id']=='ar')
            self.assertIsNone(row['name']);self.assertIn('person_lookup',row['issues'])
    def test_target_unknown_fields_are_counted_without_zero_filling(self):
        p=parts();p[2].append(dict(game_id='G1',player_id='unknown',side=None,fielding_position=0,start_event_id=None,end_event_id=None))
        r=run(p);self.assertEqual(r['target_unknown_fields'],dict(side=1,position=1,start=1,end=1,reversed_range=0))
        bad=parts();bad[3]['counts']['strikeout']=True
        with self.assertRaises(ValueError):run(bad)
    def test_shuffle_property_and_guards(self):
        p=parts();expected=run(p);rng=random.Random(19)
        for _ in range(20):
            for rows in p[:3]:rng.shuffle(rows)
            rng.shuffle(p[3]['roles']['pitcher']['profiles']);self.assertEqual(run(p),expected)
        for kind in ['duplicate','orphan','unsupported','profile_duplicate']:
            p=parts()
            if kind=='duplicate':p[2].append(copy.deepcopy(p[2][0]))
            elif kind=='orphan':p[2][0]['game_id']='missing'
            elif kind=='unsupported':p[0][0]['use_dh']=None
            else:p[3]['roles']['pitcher']['profiles'].append(copy.deepcopy(p[3]['roles']['pitcher']['profiles'][0]))
            with self.subTest(kind=kind),self.assertRaises(ValueError):run(p)

class PublicationTests(unittest.TestCase):
    def test_early_refusals_do_not_query_or_publish(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);missing=root/'missing';out=root/'report.json';out.write_text('keep')
            with self.assertRaisesRegex(ValueError,'overwrite'):audit.publish(missing,missing,missing,out)
            self.assertEqual(out.read_text(),'keep');out.unlink();missing.write_text('bad')
            with self.assertRaisesRegex(ValueError,'digest'):audit.publish(missing,missing,missing,out)
            self.assertFalse(out.exists())
        with self.assertRaisesRegex(ValueError,'outside'):audit.publish(Path('/missing'),Path('/missing'),Path('/missing'),Path(__file__).resolve().parents[1]/'.bullpen-refusal.json')
    def test_native_publisher_repeat_provenance_and_final_drift(self):
        from audits import lineup_readiness
        p=parts();p[0][0]['game_id']=audit.TARGET
        for row in p[2]:row['game_id']=audit.TARGET
        def sql(v):
            if v is None:return 'NULL'
            if type(v) is bool:return str(v).lower()
            if type(v) is int:return str(v)
            if isinstance(v,dict):return 'map(['+','.join(v)+'],['+','.join(sql(x) for x in v.values())+'])'
            return "'"+v.replace("'","''")+"'"
        def table(name,rows):return 'CREATE TABLE '+name+' AS '+' UNION ALL '.join('SELECT '+','.join(sql(v)+' AS '+k for k,v in row.items()) for row in rows)+';'
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);snapshot=root/'main.db';detail=root/'detail.db';counts=root/'counts.json'
            g=dict(p[0][0]);g.pop('results')
            manifest=dict(schema_version='f03-spike-1',query_version='2019-filtered-extract-v1',catalog_sha256=lineup_readiness.CATALOG,source_url='local synthetic',extracted_at='synthetic',attribution='notice')
            for path,script in [(snapshot,table('games',[g])+table('results',[dict(game_id=audit.TARGET,**p[0][0]['results'][0])])+table('people',p[1])+table('audit_manifest',[manifest])),(detail,table('stg_game_fielding_appearances',p[2]))]:
                result=subprocess.run(['duckdb',str(path),'-init','/dev/null','-bail','-batch','-c',script],capture_output=True,text=True,check=True);self.assertEqual(result.stderr,'')
            p[3]['provenance']=dict(snapshot_sha256=audit.digest(snapshot),opportunity_basis=audit.BASIS,attribution='notice');counts.write_text(json.dumps(p[3]))
            pins=(audit.digest(snapshot),audit.digest(detail))
            with patch.object(lineup_readiness,'INPUT_SHA',pins),patch.object(audit,'REPORT_SHA',audit.digest(counts)):
                first=audit.publish(snapshot,detail,counts,root/'first.json');repeat=audit.publish(snapshot,detail,counts,root/'repeat.json')
                self.assertEqual(first,repeat);self.assertEqual((root/'first.json').read_bytes(),(root/'repeat.json').read_bytes())
                self.assertEqual((root/'first.native.csv').read_bytes(),(root/'repeat.native.csv').read_bytes());self.assertEqual((root/'first.native.stderr').read_bytes(),b'')
                self.assertEqual(first['provenance']['native_export_sha256'],audit.digest(root/'first.native.csv'))
                self.assertEqual((audit.digest(snapshot),audit.digest(detail)),pins)
                original=audit.inventory
                def drift(*a,**kw):
                    result=original(*a,**kw);counts.write_text('changed');return result
                with patch.object(audit,'inventory',side_effect=drift),self.assertRaisesRegex(ValueError,'changed'):audit.publish(snapshot,detail,counts,root/'drift.json')
                self.assertFalse((root/'drift.json').exists());self.assertTrue((root/'drift.native.csv').exists())
    def test_writer_rereads_and_is_exclusive(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);source=root/'source';source.write_text('before');pins=(hashlib.sha256(source.read_bytes()).hexdigest(),);out=root/'out'
            source.write_text('after')
            with self.assertRaisesRegex(ValueError,'changed'):audit.write_report({},[source],pins,out)
            self.assertFalse(out.exists());source.write_text('before');audit.write_report({'a':1},[source],pins,out)
            with self.assertRaises(FileExistsError):audit.write_report({},[source],pins,out)
            self.assertEqual(json.loads(out.read_text()),{'a':1})
