"""Synthetic outing workloads; no historical rosters or fabricated pitch counts."""
import copy
import csv
import hashlib
import io
import json
from pathlib import Path
import random
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace
from audits import pitcher_workload as audit
from test_lineup_readiness import fixture
from test_player_opportunities import row


def parts():
    games,people,appearances=fixture()
    for side,prefix in [('Away','a'),('Home','h')]:
        people.append(dict(player_id=prefix+'r',first_name=prefix+'r',last_name='Source',bats=None,throws='Unknown'))
        appearances.append(dict(game_id='G1',player_id=prefix+'r',side=side,fielding_position=1,start_event_id=60,end_event_id=90))
    second=copy.deepcopy(games[0]);second['game_id']='G2';second['date']='2019-03-21'
    second['away_starting_pitcher_id']='ar';second['lineup_map_away']['0']='ar';second['fielding_map_away']['1']='ar'
    second_rows=[dict(a,game_id='G2') for a in appearances]
    for a in second_rows:
        if a['side']=='Away' and a['fielding_position']==1:a['player_id']='ar' if a['player_id']=='ap' else 'ap'
    third=copy.deepcopy(games[0]);third['game_id']='G3';third['date']='2019-03-22';third['results'][0]['suspension_flag']=True
    games.extend([second,third]);appearances.extend(second_rows+[dict(a,game_id='G3') for a in appearances])
    events=[]
    def add(game,pitcher,result='InPlayOut'):
        events.append(row(len(events)+1,result,game_id=game,pitcher_id=pitcher,batter_id='h1'))
    for result in ['Walk','IntentionalWalk','Interference','SacrificeFly','']:add('G1','ap',result)
    for _ in range(6):add('G2','ar')
    for result in ['HomeRun','StrikeOut']:add('G2','ap',result)
    add('G3','ap')
    return games,people,appearances,events


def run(p,selected=('ap','ar','hr')):return audit.inventory(*p,selected=selected)
def profile(report,player='ap'):return next(p for p in report['players'] if p['player_id']==player)


class WorkloadTests(unittest.TestCase):
    def test_start_and_relief_means_do_not_mix_roles(self):
        r=run(parts());self.assertEqual([p['player_id'] for p in r['players']],['ap','ar','hr'])
        p=profile(r);self.assertEqual(p['start'],dict(outings=1,pa_sum=4,mean_pa=4.0,limit=4))
        self.assertEqual(p['relief'],dict(outings=1,pa_sum=2,mean_pa=2.0,limit=2))
        self.assertEqual(p['source_pa'],7);self.assertEqual(p['excluded_pa'],1)
        self.assertEqual(profile(r,'ar')['start']['limit'],6)
        self.assertEqual(r['training_game_ids'],['G1','G2'])
    def test_workload_includes_exclusions_and_sacrifices_not_runner_only(self):
        p=profile(run(parts()));first=next(o for o in p['outing_rows'] if o['game_id']=='G1')
        self.assertEqual(first['terminal_pa'],4);self.assertEqual(first['non_pa'],1)
        self.assertEqual(first['role'],'start');self.assertTrue(first['training'])
    def test_zero_and_missing_role_are_distinct(self):
        p=profile(run(parts()),'hr')
        self.assertEqual(p['relief'],dict(outings=2,pa_sum=0,mean_pa=0.0,limit=1))
        self.assertEqual(p['start'],dict(outings=0,pa_sum=0,mean_pa=None,limit=None))
        self.assertIsNone(p['ability_profile'])
    def test_multiple_mound_stints_are_one_game_outing(self):
        p=parts();p[2].append(dict(game_id='G1',player_id='ap',side='Away',fielding_position=1,start_event_id=100,end_event_id=110))
        r=profile(run(p));self.assertEqual(r['start']['outings'],1)
        first=next(o for o in r['outing_rows'] if o['game_id']=='G1');self.assertEqual(first['position1_rows'],2)
        self.assertEqual(first['terminal_pa'],4)
    def test_unjoined_terminal_participants_are_not_invented_outings(self):
        p=parts();p[3].append(row(99,'StrikeOut',game_id='G1',pitcher_id='missing',batter_id='h1'))
        r=run(p,selected=('missing',));self.assertEqual(r['unmatched_terminal_pa'],1)
        self.assertEqual(profile(r,'missing')['start']['outings'],0)
        self.assertEqual(profile(r,'missing')['relief']['outings'],0)
        self.assertEqual(profile(r,'missing')['source_pa'],1)
        self.assertIn('missing_appearance',profile(r,'missing')['issues'])
    def test_shuffle_property_and_nonmutation(self):
        p=parts();before=copy.deepcopy(p);expected=run(p);self.assertEqual(p,before)
        rng=random.Random(19)
        for _ in range(20):
            for rows in p:rng.shuffle(rows)
            self.assertEqual(run(p),expected)
    def test_raw_ranges_and_nonpitching_roles_survive_without_becoming_workload(self):
        p=parts();extra=dict(game_id='G1',player_id='ap',side=None,fielding_position=11,start_event_id=255,end_event_id=0)
        p[2].append(extra);p[2].append(dict(game_id='G2',player_id='ap',side='Away',fielding_position=1,start_event_id=100,end_event_id=None))
        r=profile(run(p));self.assertEqual(r.get('appearance_rows'),sorted([a for a in p[2] if a['player_id']=='ap'],key=lambda a:json.dumps(a,sort_keys=True)))
        self.assertEqual(r['start']['pa_sum'],4);self.assertEqual(r['relief']['pa_sum'],2)
        second=next(o for o in r['outing_rows'] if o['game_id']=='G2')
        self.assertEqual(second.get('appearances'),sorted([a for a in p[2] if a['game_id']=='G2' and a['player_id']=='ap' and a['fielding_position']==1],key=lambda a:json.dumps(a,sort_keys=True)))
    def test_unmatched_non_pa_does_not_vanish_or_create_an_outing(self):
        p=parts();p[3].append(row(100,'',game_id='G1',pitcher_id='missing',batter_id='h1'))
        r=run(p,selected=('missing',));self.assertEqual(r.get('unmatched_non_pa'),1)
        q=profile(r,'missing');self.assertEqual(q.get('unmatched_non_pa'),1);self.assertEqual(q['source_pa'],0)
        self.assertEqual(q['relief']['outings'],0);self.assertIn('missing_appearance',q['issues'])
    def test_fractional_mean_ceil_sparse_bios_and_unknown_roles(self):
        p=parts();p[0][2]['results'][0]['suspension_flag']=False
        r=profile(run(p));self.assertEqual(r['start'],dict(outings=2,pa_sum=5,mean_pa=2.5,limit=3))
        p=parts()
        for a in p[2]:
            if a['player_id']=='ar' and a['game_id']=='G1':a['side']=None
        r=profile(run(p),'ar');self.assertEqual(r['relief']['outings'],0);self.assertIn('unknown_appearance_role',r['issues'])
        p[1].append(copy.deepcopy(next(b for b in p[1] if b['player_id']=='ar')))
        r=profile(run(p),'ar');self.assertIsNone(r['name']);self.assertIn('person_lookup',r['issues'])
    def test_existing_full_validators_and_orphan_guards_are_preserved(self):
        for kind in ['duplicate_event','bad_pa','duplicate_appearance','orphan_event','orphan_appearance']:
            p=parts()
            if kind=='duplicate_event':p[3].append(copy.deepcopy(p[3][0]))
            elif kind=='bad_pa':p[3][0]['pa']='0'
            elif kind=='duplicate_appearance':p[2].append(copy.deepcopy(p[2][0]))
            elif kind=='orphan_event':p[3][0]['game_id']='missing'
            else:p[2][0]['game_id']='missing'
            with self.subTest(kind=kind),self.assertRaises(ValueError):run(p)


class PublicationTests(unittest.TestCase):
    def test_early_refusals_prevent_query_and_publication(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);missing=root/'missing';out=root/'out.json';out.write_text('keep')
            with self.assertRaisesRegex(ValueError,'overwrite'):audit.publish(missing,missing,missing,missing,missing,out)
            self.assertEqual(out.read_text(),'keep');out.unlink();missing.write_text('bad')
            with self.assertRaisesRegex(ValueError,'digest'):audit.publish(missing,missing,missing,missing,missing,out)
            self.assertFalse(out.exists())
        with self.assertRaisesRegex(ValueError,'outside'):audit.publish(*([Path('/missing')]*5),Path(__file__).resolve().parents[1]/'.workload-refusal.json')
    def test_writer_rechecks_input_bytes_and_exclusive_output(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);source=root/'source';source.write_text('before');pins=(hashlib.sha256(source.read_bytes()).hexdigest(),);out=root/'out.json'
            source.write_text('after')
            with self.assertRaisesRegex(ValueError,'changed'):audit.write_report({'a':1},[source],pins,out)
            self.assertFalse(out.exists());source.write_text('before');audit.write_report({'a':1},[source],pins,out)
            self.assertEqual(out.read_text(),'{\n  "a": 1\n}\n')
            with self.assertRaises(FileExistsError):audit.write_report({},[source],pins,out)


class CliTests(unittest.TestCase):
    def test_help_and_refusal_exit_are_explicit(self):
        import sys
        r=subprocess.run([sys.executable,'-Werror','-m','audits.pitcher_workload','--help'],capture_output=True,text=True)
        self.assertEqual(r.returncode,0,r.stderr);self.assertIn('snapshot',r.stdout);self.assertIn('bullpen',r.stdout)
        with tempfile.TemporaryDirectory() as d:
            out=Path(d)/'out.json';out.write_text('keep')
            r=subprocess.run([sys.executable,'-Werror','-m','audits.pitcher_workload',*(['/missing']*5),str(out)],capture_output=True,text=True)
            self.assertNotEqual(r.returncode,0);self.assertIn('overwrite',r.stderr);self.assertEqual(out.read_text(),'keep')


class NativePublisherTests(unittest.TestCase):
    def test_repeat_source_conservation_and_final_drift(self):
        from audits import bullpen_readiness as bullpen
        p=parts();op=audit.player.inventory(p[3]);lr=audit.lineup.inventory(*p[:3]);br=bullpen.inventory(*p[:3],op,target='G1')
        manifest=dict(schema_version='f03-spike-1',query_version='2019-filtered-extract-v1',catalog_sha256=audit.lineup.CATALOG,source_url='local synthetic',extracted_at='synthetic',attribution='notice')
        def csv_text(fields,rows):
            s=io.StringIO();w=csv.DictWriter(s,fieldnames=fields);w.writeheader();w.writerows(rows);return s.getvalue()
        records=[]
        for kind,rows in [('game',p[0]),('person',p[1]),('appearance',p[2]),('manifest',[manifest])]:
            records.extend(dict(record_type=kind,payload=json.dumps(r,sort_keys=True)) for r in rows)
        groups={}
        for r in p[3]:
            cell=groups.setdefault((r['game_id'],r['pitcher_id']),dict(terminal_pa=0,non_pa=0));cell['terminal_pa' if r['result'] else 'non_pa']+=1
        native=[dict(game_id=g,pitcher_id=pid,**n) for (g,pid),n in sorted(groups.items())]
        outputs={'2019-lineup-readiness.sql':csv_text(['record_type','payload'],records),'2019-outcomes.sql':csv_text(list(p[3][0]),p[3]),'2019-pitcher-workload.sql':csv_text(['game_id','pitcher_id','terminal_pa','non_pa'],native)}
        def query(args,**kw):
            self.assertTrue(all(x in args for x in ['-init','/dev/null','-readonly','-bail','-batch','-csv']))
            self.assertIn('enable_external_access=false',' '.join(args).lower())
            self.assertIn('-nullvalue',args,'Canonical outcome CSV must preserve SQL NULL as empty, not a literal player ID')
            self.assertEqual(args[args.index('-nullvalue')+1],'')
            kw['stdout'].write(outputs[Path(args[args.index('-f')+1]).name]);return SimpleNamespace(returncode=0)
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);db=root/'snapshot';detail=root/'detail';db.write_bytes(b'synthetic main');detail.write_bytes(b'synthetic detail')
            op['provenance']=dict(snapshot=manifest,snapshot_sha256=hashlib.sha256(db.read_bytes()).hexdigest(),opportunity_basis='literal terminal-event participants; not official credits or PA/BF',attribution='notice')
            for r in [lr,br]:r['provenance']=dict(snapshot=manifest,snapshot_sha256=op['provenance']['snapshot_sha256'],supplement_sha256=hashlib.sha256(detail.read_bytes()).hexdigest(),attribution='notice')
            paths=[db,detail]
            for name,r in [('op.json',op),('lineup.json',lr),('bullpen.json',br)]:
                path=root/name;path.write_text(json.dumps(r,sort_keys=True));paths.append(path)
            pins=tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in paths)
            with patch.object(audit,'INPUT_SHA',pins),patch.object(audit,'TARGET','G1'),patch.object(audit.subprocess,'run',side_effect=query):
                first=audit.publish(*paths,root/'first.json');self.assertEqual(len(first['players']),4)
                repeat=audit.publish(*paths,root/'repeat.json');self.assertEqual(first,repeat)
                self.assertEqual((root/'first.json').read_bytes(),(root/'repeat.json').read_bytes())
                self.assertEqual(tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in paths),pins)
                for label,change in [('sum',lambda n:n[0].update(terminal_pa=n[0]['terminal_pa']+1)),
                                     ('duplicate',lambda n:n.append(copy.deepcopy(n[0]))),
                                     ('negative',lambda n:n[0].update(non_pa=-1))]:
                    bad=copy.deepcopy(native);change(bad);saved=outputs['2019-pitcher-workload.sql']
                    outputs['2019-pitcher-workload.sql']=csv_text(['game_id','pitcher_id','terminal_pa','non_pa'],bad)
                    out=root/(label+'.json')
                    with self.subTest(label=label),patch.object(audit,'inventory',wraps=audit.inventory) as measured,self.assertRaises(ValueError):audit.publish(*paths,out)
                    self.assertEqual(measured.call_count,0,'Reconcile native source before measuring outing means')
                    self.assertFalse(out.exists());outputs['2019-pitcher-workload.sql']=saved
                saved=outputs['2019-outcomes.sql'];outputs['2019-outcomes.sql']=csv_text(list(p[3][0]),p[3]+[p[3][0]])
                with self.assertRaisesRegex(ValueError,'duplicate event'):audit.publish(*paths,root/'duplicate-event.json')
                self.assertFalse((root/'duplicate-event.json').exists());outputs['2019-outcomes.sql']=saved
                def failed(args,**kw):
                    query(args,**kw);kw['stderr'].write('synthetic native failure\n');return SimpleNamespace(returncode=9)
                with patch.object(audit.subprocess,'run',side_effect=failed),self.assertRaisesRegex(RuntimeError,'Native workload query failed'):
                    audit.publish(*paths,root/'failed.json')
                self.assertFalse((root/'failed.json').exists());self.assertEqual((root/'failed.lineup.stderr').read_text(),'synthetic native failure\n')
                op['provenance']['opportunity_basis']='not the accepted basis';paths[2].write_text(json.dumps(op,sort_keys=True))
                changed_pins=tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in paths)
                with patch.object(audit,'INPUT_SHA',changed_pins),self.assertRaisesRegex(ValueError,'basis'):audit.publish(*paths,root/'basis.json')
                self.assertFalse((root/'basis.json').exists());op['provenance']['opportunity_basis']='literal terminal-event participants; not official credits or PA/BF';paths[2].write_text(json.dumps(op,sort_keys=True))
                self.assertEqual(tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in paths),pins)
                original=audit.inventory
                def drift(*a,**kw):
                    r=original(*a,**kw);paths[2].write_text('changed');return r
                with patch.object(audit,'inventory',side_effect=drift),self.assertRaisesRegex(ValueError,'changed'):
                    audit.publish(*paths,root/'drift.json')
                self.assertFalse((root/'drift.json').exists())


class NativeProjectionTests(unittest.TestCase):
    def query(self,change=''):
        sql=(Path(__file__).resolve().parents[1]/'audits/2019-pitcher-workload.sql').read_text()
        with tempfile.TemporaryDirectory() as d:
            db=Path(d)/'source.duckdb'
            setup="""CREATE TABLE events(event_key INTEGER,game_id VARCHAR,pitcher_id VARCHAR,plate_appearance_result VARCHAR,no_play_flag BOOLEAN,season INTEGER);
CREATE TABLE batting(event_key INTEGER,plate_appearances INTEGER);
INSERT INTO events VALUES (1,'G','ap','IntentionalWalk',false,2019),(2,'G','ap',NULL,false,2019),(3,'G','ar','Interference',false,2019),(4,'G','ap','Walk',true,2019),(5,'OLD','ar','Walk',false,2018);
INSERT INTO batting VALUES (1,1),(3,1),(4,1),(5,1);"""+change
            subprocess.run(['duckdb','-init','/dev/null','-bail','-batch',str(db),'-c',setup],check=True,capture_output=True,text=True)
            before=db.read_bytes()
            result=subprocess.run(['duckdb','-init','/dev/null','-readonly','-bail','-batch','-csv',str(db),'-c','SET enable_external_access=false;'+sql],capture_output=True,text=True)
            self.assertEqual(db.read_bytes(),before)
        return result
    def test_readonly_native_groups_keep_all_terminal_pas_and_drop_no_play(self):
        r=self.query();self.assertEqual(r.returncode,0,r.stderr);self.assertEqual(r.stderr,'')
        self.assertEqual(list(csv.DictReader(io.StringIO(r.stdout))),[
            dict(game_id='G',pitcher_id='ap',terminal_pa='1',non_pa='1'),
            dict(game_id='G',pitcher_id='ar',terminal_pa='1',non_pa='0')])
    def test_native_empty_null_encoding_is_not_the_literal_null_identity(self):
        cmd=['duckdb','-init','/dev/null','-bail','-batch','-csv']
        sql="SELECT NULL::VARCHAR AS first,0 AS base_mask,'NULL' AS literal_identity;"
        r=subprocess.run([*cmd,'-nullvalue','','-c',sql],capture_output=True,text=True,check=True)
        self.assertEqual(list(csv.DictReader(io.StringIO(r.stdout))),[dict(first='',base_mask='0',literal_identity='NULL')])
        r=subprocess.run([*cmd,'-c',sql],capture_output=True,text=True,check=True)
        self.assertEqual(list(csv.DictReader(io.StringIO(r.stdout)))[0]['first'],'NULL')
    def test_native_guard_refuses_missing_pa_and_join_fanout(self):
        for change in ['UPDATE batting SET plate_appearances=0 WHERE event_key=1;',
                       'INSERT INTO batting VALUES (1,1);',
                       'UPDATE events SET no_play_flag=NULL WHERE event_key=1;']:
            with self.subTest(change=change):
                r=self.query(change);self.assertNotEqual(r.returncode,0)
                self.assertIn('workload',r.stderr)
