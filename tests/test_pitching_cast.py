"""Synthetic closed cast/counts; availability is an explicit exhibition assumption."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from preview import pitching_cast as cast
from preview import player_ratings as ratings
from preview import real_lineup as real
from audits import pitcher_workload as workload
from test_real_lineup import witness
from test_player_ratings import report


def parts():
    w=witness();r=report();teams=[];players=[]
    def measured(pid,name,profile,s,g,role):
        summaries={};rows=[]
        for kind in ['start','relief']:
            total,n=(s,g) if kind==role else (0,0)
            summaries[kind]=dict(pa_sum=total,outings=n,mean_pa=total/n if n else None,limit=max(1,(total+n-1)//n) if n else None)
            rows.extend(dict(game_id=f'{pid}{kind}{i}',role=kind,training=True,terminal_pa=total if i==0 else 0) for i in range(n))
        players.append(dict(player_id=pid,name=name,ability_profile=copy.deepcopy(profile),outing_rows=rows,issues=[],**summaries))
        return dict(player_id=pid,name=name,eligible_at_start=None)
    for t in w['teams']:
        starter=t['pitcher_id'];p=next(p for p in r['roles']['pitcher']['profiles'] if p['player_id']==starter)
        name=next(f['name'] for f in t['fielding'] if f['player_id']==starter)
        facts=[measured(starter,name,p,26,1,'start')]
        values=[(8,2),(4,1),(6,2),(4,1)] if t['side']=='Away' else [(5,2),(9,2),(8,2),(0,1),(3,1),(7,1)]
        for i,(s,g) in enumerate(values):
            pid=('ar' if t['side']=='Away' else 'hr')+str(i);counts=dict.fromkeys(ratings.KEYS,0);counts['strikeout']=s
            p=dict(player_id=pid,included_pa=s,counts=counts);r['roles']['pitcher']['profiles'].append(p)
            facts.append(measured(pid,pid+' Synthetic',p,s,g,'relief'))
        teams.append(dict(side=t['side'],team_id=t['team_id'],starter_id=starter,observed_pitchers=facts,observed_nonstarter_pitcher_ids=[f['player_id'] for f in facts[1:]]))
    b=dict(format='2019-bullpen-readiness-v1',game_id=w['game_id'],pregame_bullpen_status='unsupported',teams=teams)
    d=dict(format='2019-pitcher-workload-v1',game_id=w['game_id'],basis=workload.BASIS,players=players,
        provenance=dict(snapshot_sha256=real.SNAPSHOT_SHA,supplement_sha256=real.SUPPLEMENT_SHA,opportunity_report_sha256=ratings.REPORT_SHA,lineup_report_sha256=real.REPORT_SHA,bullpen_report_sha256=cast.BULLPEN_SHA,attribution=real.ATTRIBUTION))
    return r,w,b,d


class CastTests(unittest.TestCase):
    def test_exact_cast_raw_role_workloads_and_thirty_ability_vectors(self):
        p=parts();before=copy.deepcopy(p);text=cast.encode(*p)
        self.assertIn('format\tclosed-pitching-cast-v1\n',text)
        self.assertEqual(len([x for x in text.splitlines() if x.startswith(('batter:','pitcher:'))]),30)
        self.assertIn('workload:start:ap\t26,1\n',text);self.assertIn('workload:relief:ap\t0,0\n',text)
        self.assertIn('workload:relief:hr3\t0,1\n',text);self.assertIn('pitcher:hr3\t0,0,0,0,0,0,0,0,0\n',text)
        self.assertIn('availability_basis\trecorded nonstarters assumed available at start; not verified eligibility\n',text)
        self.assertNotIn('probabilities\t',text);self.assertEqual(p,before)
    def test_exact_rational_longer_first_order_and_stable_id_ties(self):
        text=cast.encode(*parts());rows=[line.split('\t')[0].split(':')[-1] for line in text.splitlines() if line.startswith('cast:Away:')]
        self.assertEqual(rows,['ap','ar0','ar1','ar3','ar2'])
        rows=[line.split('\t')[0].split(':')[-1] for line in text.splitlines() if line.startswith('cast:Home:')]
        self.assertEqual(rows,['hp','hr5','hr1','hr2','hr4','hr0','hr3'])
        p=parts()
        for t in p[2]['teams']:t['observed_pitchers'].reverse();t['observed_nonstarter_pitcher_ids'].reverse()
        p[2]['teams'].reverse();p[3]['players'].reverse()
        for v in p[0]['roles'].values():v['profiles'].reverse()
        self.assertEqual(cast.encode(*p),text)
    def test_missing_sparse_unknown_and_malformed_support_are_not_borrowed(self):
        changes=[lambda p:p[0]['roles']['pitcher']['profiles'].pop(),
                 lambda p:p[0]['roles']['pitcher']['profiles'].append(p[0]['roles']['pitcher']['profiles'][-1]),
                 lambda p:p[3]['players'].append(p[3]['players'][0]),
                 lambda p:p[3]['players'][2]['relief'].update(outings=0),
                 lambda p:p[3]['players'][2]['relief'].update(pa_sum=True),
                 lambda p:p[3]['players'][2]['relief'].update(limit=999),
                 lambda p:p[3]['players'][-3]['relief'].update(limit=True),
                 lambda p:p[3]['players'][-3]['relief'].update(mean_pa=False),
                 lambda p:p[3]['players'][1]['ability_profile']['counts'].update(hbp=False),
                 lambda p:p[3]['players'][2].update(name='bad\nname'),
                 lambda p:p[3]['players'][2]['ability_profile']['counts'].update(strikeout=999),
                 lambda p:p[2]['teams'][0]['observed_pitchers'][1].update(eligible_at_start=True),
                 lambda p:p[2]['teams'][0]['observed_pitchers'].pop(),
                 lambda p:p[3]['provenance'].update(opportunity_report_sha256='wrong')]
        for change in changes:
            p=parts();change(p)
            with self.subTest(change=change),self.assertRaises(ValueError):cast.encode(*p)
    def test_positive_export_pins_all_inputs_and_preserves_bytes(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);r,w,b,data=parts()
            lr=dict(format='2019-lineup-readiness-v1',witnesses=dict(primary=w))
            inputs=[]
            for name,obj in [('op',r),('lineup',lr),('bullpen',b)]:
                path=root/name;path.write_text(json.dumps(obj,sort_keys=True));inputs.append(path)
            data['provenance'].update(opportunity_report_sha256=real.digest(inputs[0]),lineup_report_sha256=real.digest(inputs[1]),bullpen_report_sha256=real.digest(inputs[2]))
            path=root/'workloads';path.write_text(json.dumps(data,sort_keys=True));inputs.append(path)
            pins=tuple(real.digest(p) for p in inputs)
            with patch.object(ratings,'REPORT_SHA',pins[0]),patch.object(real,'REPORT_SHA',pins[1]),patch.object(cast,'BULLPEN_SHA',pins[2]),patch.object(cast,'WORKLOAD_SHA',pins[3]):
                text=cast.export(*inputs,root/'first');repeat=cast.export(*inputs,root/'repeat')
                self.assertEqual(text,repeat);self.assertEqual((root/'first').read_bytes(),text.encode())
            self.assertEqual(tuple(real.digest(p) for p in inputs),pins)
    def test_cli_is_explicit_and_existing_output_never_succeeds_silently(self):
        import subprocess,sys
        r=subprocess.run([sys.executable,'-Werror','-m','preview.pitching_cast','--help'],capture_output=True,text=True)
        self.assertEqual(r.returncode,0,r.stderr);self.assertIn('workloads',r.stdout)
        with tempfile.TemporaryDirectory() as d:
            out=Path(d)/'cast';out.write_text('keep')
            r=subprocess.run([sys.executable,'-Werror','-m','preview.pitching_cast',*(['/missing']*4),str(out)],capture_output=True,text=True)
            self.assertNotEqual(r.returncode,0);self.assertIn('overwrite',r.stderr);self.assertEqual(out.read_text(),'keep')
    def test_early_refusals_and_atomic_final_drift(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);out=root/'out.tsv';out.write_text('keep');missing=root/'missing'
            with self.assertRaisesRegex(ValueError,'overwrite'):cast.export(missing,missing,missing,missing,out)
            self.assertEqual(out.read_text(),'keep');out.unlink();missing.write_text('{}')
            with self.assertRaisesRegex(ValueError,'digest'):cast.export(missing,missing,missing,missing,out)
            self.assertFalse(out.exists())
            inputs=[root/'a',root/'b']
            for p in inputs:p.write_text('before')
            pins=tuple(real.digest(p) for p in inputs)
            with patch('preview.player_ratings.os.fsync',side_effect=lambda _:inputs[0].write_text('after')):
                with self.assertRaisesRegex(ValueError,'changed'):cast.publish('complete\n',inputs,pins,out)
            self.assertFalse(out.exists());self.assertEqual(set(root.iterdir()),{missing,*inputs})
            inputs[0].write_text('before');cast.publish('complete\n',inputs,pins,out)
            with self.assertRaises(FileExistsError):cast.publish('wrong\n',inputs,pins,out)
            self.assertEqual(out.read_text(),'complete\n')
        with self.assertRaisesRegex(ValueError,'outside'):cast.export(*([Path('/missing')]*4),Path(__file__).resolve().parents[1]/'.cast-refusal.tsv')
