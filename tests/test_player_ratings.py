"""Synthetic counts; real source profiles remain outside Git."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from preview import player_ratings as ratings
from preview import real_lineup
from test_real_lineup import witness

KEYS=('strikeout','walk','hbp','single','double','triple','home_run','roe','contact_out')
COUNTS=(42823,15142,1984,25947,8531,785,6776,1470,82245)

def report():
    roles={role:dict(profiles=[]) for role in ['batter','pitcher']}
    for team in witness()['teams']:
        for p in team['batting_order']:
            values=[3,2,0,4,1,0,1,0,9]
            roles['batter']['profiles'].append(dict(player_id=p['player_id'],included_pa=sum(values),counts=dict(zip(KEYS,values))))
        values=[5,1,0,3,1,0,1,0,9]
        roles['pitcher']['profiles'].append(dict(player_id=team['pitcher_id'],included_pa=sum(values),counts=dict(zip(KEYS,values))))
    return dict(format='2019-player-opportunity-support-v1',counts=dict(zip(KEYS,COUNTS)),roles=roles,
                provenance=dict(snapshot_sha256=real_lineup.SNAPSHOT_SHA,attribution=real_lineup.ATTRIBUTION,opportunity_basis=ratings.BASIS))

def rating_record():
    from test_visible_viewer import real_record
    source=real_record().replace('Model: pooled-jeffreys-joint-proxy-preview-v1','Model: '+ratings.MODEL)
    meta='RATINGS FORMAT: player-proxy-counts-v1\nRating prior strength: batter200 pitcher200\n'
    r=report()
    for role in ['batter','pitcher']:
        for p in r['roles'][role]['profiles']:
            mu=[(p['counts'][k]+200*((n+.5)/(sum(COUNTS)+4.5)))/(p['included_pa']+200) for k,n in zip(KEYS,COUNTS)]
            import math
            sd=[math.sqrt(x*(1-x)/(p['included_pa']+201)) for x in mu]
            meta+=f"RATING {role} {p['player_id']} | N: {p['included_pa']} | Weight: {p['included_pa']/(p['included_pa']+200)} | Mean: {mu} | SD: {sd} | Counts: {[p['counts'][k] for k in KEYS]}\n"
    for team in witness()['teams']:
        opponent=next(t for t in witness()['teams'] if t['side']!=team['side'])
        for p in team['batting_order']:
            meta+=f"MATCHUP {team['side']} {p['slot']} {p['player_id']} {opponent['pitcher_id']}: {[ (n+.5)/(sum(COUNTS)+4.5) for n in COUNTS ]}\n"
    return source.replace('COMMENTARY FORMAT:',meta+'COMMENTARY FORMAT:',1)

class ExportTests(unittest.TestCase):
    def test_literal_role_counts_and_fixed_prior_without_python_probability_export(self):
        r=report();before=copy.deepcopy(r);text=ratings.encode(r,witness())
        self.assertIn('format\tplayer-proxy-counts-v1\n',text)
        self.assertIn('batter_kappa\t200\n',text)
        self.assertIn('batter:a1\t3,2,0,4,1,0,1,0,9\n',text)
        self.assertIn('pitcher:hp\t5,1,0,3,1,0,1,0,9\n',text)
        self.assertEqual(len([x for x in text.splitlines() if x.startswith(('batter:','pitcher:'))]),20)
        self.assertNotIn('probabilities\t',text);self.assertEqual(r,before)
        for role in r['roles']:r['roles'][role]['profiles'].reverse()
        self.assertEqual(ratings.encode(r,witness()),text)
    def test_zero_exposure_is_preserved_and_invalid_profile_never_borrowed(self):
        r=report();p=r['roles']['batter']['profiles'][0];p['counts']=dict.fromkeys(KEYS,0);p['included_pa']=0
        self.assertIn('batter:a1\t0,0,0,0,0,0,0,0,0\n',ratings.encode(r,witness()))
        for change in [lambda r:r['roles']['batter']['profiles'].pop(0),
            lambda r:r['roles']['batter']['profiles'].append(r['roles']['batter']['profiles'][0]),
            lambda r:r['roles']['batter']['profiles'][0]['counts'].update(strikeout=True),
            lambda r:r['roles']['batter']['profiles'][0]['counts'].update(strikeout=-1),
            lambda r:r['roles']['batter']['profiles'][0].update(included_pa=999),
            lambda r:r['counts'].update(strikeout=0),
            lambda r:r['provenance'].update(opportunity_basis='official BF')]:
            bad=report();change(bad)
            with self.subTest(change=change),self.assertRaises(ValueError):ratings.encode(bad,witness())
    def test_existing_repository_and_unpinned_inputs_refuse_early(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);out=root/'out';out.write_text('keep')
            with self.assertRaisesRegex(ValueError,'overwrite'):ratings.export(root/'missing',root/'missing',out)
            self.assertEqual(out.read_text(),'keep')
            source=root/'report';source.write_text(json.dumps(report()))
            with self.assertRaisesRegex(ValueError,'digest'):ratings.export(source,source,root/'new')
            self.assertFalse((root/'new').exists())
        with self.assertRaisesRegex(ValueError,'outside'):ratings.export(Path('/missing'),Path('/missing'),Path(__file__).resolve().parents[1]/'.rating-refusal.tsv')
    def test_final_drift_exclusive_atomic_publication_and_cleanup(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);inputs=[root/'a',root/'b']
            for p in inputs:p.write_bytes(b'original')
            pins=tuple(real_lineup.digest(p) for p in inputs)
            with patch('preview.player_ratings.os.fsync',side_effect=lambda _:inputs[1].write_bytes(b'changed')):
                with self.assertRaisesRegex(ValueError,'changed'):ratings.publish('complete\n',inputs,pins,root/'out')
            self.assertFalse((root/'out').exists());self.assertEqual(set(root.iterdir()),set(inputs))
            inputs[1].write_bytes(b'original');ratings.publish('complete\n',inputs,pins,root/'out')
            self.assertEqual((root/'out').read_text(),'complete\n')
            with self.assertRaises(FileExistsError):ratings.publish('wrong\n',inputs,pins,root/'out')
            self.assertEqual((root/'out').read_text(),'complete\n')

class PresentationTests(unittest.TestCase):
    def test_rated_native_panel_and_limits_not_identical_abilities_claim(self):
        from preview.build_viewer import build_viewer
        page=build_viewer(rating_record())
        self.assertIn('id="player-probabilities"',page)
        self.assertEqual(page.count('class="matchup-probability"'),18)
        self.assertIn('hindsight',page);self.assertIn('participant',page)
        self.assertIn('200',page);self.assertIn('not calibrated',page)
        self.assertNotIn('All players use identical pooled probabilities',page)
    def test_missing_duplicate_invalid_ratings_or_matchup_metadata_fails(self):
        from preview.build_viewer import build_viewer
        source=rating_record()
        line=next(x for x in source.splitlines() if x.startswith('RATING batter a1 '))
        match=next(x for x in source.splitlines() if x.startswith('MATCHUP Away 1 '))
        for bad in [source.replace(line+'\n',''),source.replace(line,line+'\n'+line),
                    source.replace(match+'\n',''),source.replace('RATINGS FORMAT: player-proxy-counts-v1','RATINGS FORMAT: bad'),
                    source.replace(match,match.split(':')[0]+': [NaN, 0, 0, 0, 0, 0, 0, 0, 1]')]:
            with self.subTest(bad=bad),self.assertRaises(ValueError):build_viewer(bad)

class CliTests(unittest.TestCase):
    def test_native_rated_optin_and_missing_prerequisites_fail_without_fallback(self):
        import subprocess
        from test_live_preview import joint_inputs,BINARY
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);base,contact=joint_inputs(root)
            lineup=root/'lineup';lineup.write_text(real_lineup.encode_witness(witness()))
            counts=root/'ratings';counts.write_text(ratings.encode(report(),witness()))
            cmd=[str(BINARY),'--baseline',str(base),'--contact-counts',str(contact),'--lineup',str(lineup),'--ratings',str(counts),'--seed','1','--autoplay','--trace']
            result=subprocess.run(cmd,capture_output=True,text=True,timeout=30)
            self.assertEqual(result.returncode,0,result.stderr);self.assertIn('Model: '+ratings.MODEL,result.stdout)
            self.assertIn('RATING batter a1 ',result.stdout);self.assertIn('MATCHUP Away 1 a1 hp:',result.stdout)
            self.assertNotIn('no player fitting',result.stdout);self.assertNotIn('No player ratings,',result.stdout)
            self.assertIn('Reference pooled joint CDF edges:',result.stdout)
            self.assertEqual(subprocess.check_output(cmd,text=True,timeout=30),result.stdout)
            counts.write_text('bad')
            bad=subprocess.run(cmd,capture_output=True,text=True,timeout=30)
            self.assertNotEqual(bad.returncode,0);self.assertNotIn('Play 01',bad.stdout)
            missing=subprocess.run([str(BINARY),'--baseline',str(base),'--ratings',str(counts)],capture_output=True,text=True,timeout=30)
            self.assertEqual(missing.returncode,2);self.assertIn('requires --lineup',missing.stderr)

if __name__=='__main__':unittest.main()
