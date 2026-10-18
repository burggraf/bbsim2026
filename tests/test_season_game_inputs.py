"""Synthetic generic identities only; real game personnel stay outside Git."""
import copy
from pathlib import Path
import subprocess
import tempfile
import unittest
from preview import real_lineup as real
from test_real_lineup import witness


def game_witness(dh=True):
    w=witness();w.update(game_id='LAN201904010',date='2019-04-01',park_id='LOS03',use_dh=dh)
    for t,tid,name in zip(w['teams'],['SFN','LAN'],['San Francisco Giants','Los Angeles Dodgers']):
        t.update(team_id=tid,team_name=name)
        if not dh:
            pitcher=next(p for p in t['fielding'] if p['position']==1)
            t['batting_order'][3]=dict(slot=4,**{k:v for k,v in pitcher.items() if k!='position'})
            t['fielding']=[p for p in t['fielding'] if p['position']!=10]
            t.update(dh_slot=None,engine_dh_slot=None)
    return w


class GenericLineupTests(unittest.TestCase):
    def test_explicit_generic_format_preserves_dh_and_non_dh_roles(self):
        for dh,n in [(True,34),(False,32)]:
            with self.subTest(dh=dh):
                w=game_witness(dh);before=copy.deepcopy(w)
                text=real.encode_witness(w,generic=True)
                self.assertIn('format\tclosed-starting-lineup-v2\n',text)
                self.assertIn('use_dh\t'+str(dh).lower()+'\n',text)
                self.assertIn('game_id\tLAN201904010\n',text)
                self.assertIn('away_team_id\tSFN\n',text)
                self.assertIn('home_team_id\tLAN\n',text)
                self.assertEqual(len(text.splitlines()),n)
                self.assertEqual(w,before)
                if dh:self.assertIn('Away:0:1:ap\tPitcher a\n',text)
                else:
                    self.assertIn('Away:4:1:ap\tPitcher a\n',text)
                    self.assertNotIn('Away:0:',text)
                    self.assertNotIn('Away:4:10:',text)
                for team in w['teams']:team['fielding'].reverse();team['batting_order'].reverse()
                w['teams'].reverse()
                self.assertEqual(real.encode_witness(w,generic=True),text)
                with self.assertRaises(ValueError):real.encode_witness(w)

    def test_generic_metadata_and_role_boundaries(self):
        for mutate in [lambda w:w.update(use_dh=1),lambda w:w.update(date='2019-02-29'),
            lambda w:w.update(date='2019-4-1'),lambda w:w.update(date='2020-04-01'),
            lambda w:w.update(game_id='LAN201904020'),lambda w:w.update(game_id='XXX201904010'),
            lambda w:w.update(park_id='bad\tpark'),lambda w:w['teams'][0].update(team_id='LAN'),
            lambda w:w['teams'][0].update(dh_slot=4),lambda w:w['teams'][0].update(engine_dh_slot=3),
            lambda w:w['teams'][0]['batting_order'][3].update(player_id='a4'),
            lambda w:w['teams'][0]['fielding'][0].update(position=True),
            lambda w:w['teams'][0]['batting_order'][0].update(slot=True),
            lambda w:w['teams'][0]['fielding'].append(dict(position=10,player_id='a4',name='Extra')),
            lambda w:w['teams'][0]['batting_order'][0].update(name='different'),
            lambda w:w['teams'][0]['batting_order'][0].update(player_id='hp')]:
            with self.subTest(mutate=mutate),self.assertRaises(ValueError):
                w=game_witness(False);mutate(w);real.encode_witness(w,generic=True)

    def test_literal_source_hyphen_is_not_a_missing_identity(self):
        w=game_witness(False)
        for collection in ['batting_order','fielding']:
            next(p for p in w['teams'][0][collection] if p['player_id']=='a1')['player_id']='jay-j001'
        self.assertIn('Away:1:7:jay-j001\tPlayer a1\n',real.encode_witness(w,generic=True))


class GenericCastTests(unittest.TestCase):
    def parts(self, dh=False):
        from test_pitching_cast import parts
        r,w,b,d=parts();w=game_witness(dh)
        b['game_id']=d['game_id']=w['game_id']
        from preview import pitching_cast as cast
        d['provenance'].update(workload_recipe_sha256=cast.WORKLOAD_RECIPE_SHA,outcome_events_sha256=cast.EVENTS_SHA)
        for team,initial in zip(b['teams'],w['teams']):team['team_id']=initial['team_id']
        for pid in [p['player_id'] for t in b['teams'] for p in t['observed_pitchers']]:
            if not any(p['player_id']==pid for p in r['roles']['batter']['profiles']):
                from preview import player_ratings as ratings
                r['roles']['batter']['profiles'].append(dict(player_id=pid,included_pa=0,counts=dict.fromkeys(ratings.KEYS,0)))
        return r,w,b,d

    def test_generic_cast_has_explicit_complete_size_and_incoming_batting_profiles(self):
        from preview import pitching_cast as cast
        for dh in [False,True]:
            p=self.parts(dh);before=copy.deepcopy(p)
            text=cast.encode(*p,empirical=True,generic=True)
            self.assertIn('format\tclosed-pitching-cast-v3\n',text)
            self.assertIn('game_id\tLAN201904010\n',text)
            self.assertIn('cast_sizes\t5,7\n',text)
            self.assertNotIn('workload_report_sha256',text)
            if not dh:self.assertIn('batter:ar0\t0,0,0,0,0,0,0,0,0\n',text)
            self.assertEqual(p,before)

    def test_missing_profile_is_not_an_explicit_cold_profile(self):
        from preview import pitching_cast as cast
        p=self.parts();p[0]['roles']['batter']['profiles']=[v for v in p[0]['roles']['batter']['profiles'] if v['player_id']!='ar0']
        with self.assertRaisesRegex(ValueError,'Missing'):cast.encode(*p,empirical=True,generic=True)
        p=self.parts();p[3]['players'][0]['issues']=['unknown_appearance_role']
        with self.assertRaisesRegex(ValueError,'workload'):cast.encode(*p,empirical=True,generic=True)
        with self.assertRaises(ValueError):cast.encode(*self.parts(),generic=True)


class PublicationTests(unittest.TestCase):
    def test_output_guards_run_before_input_reads(self):
        from preview import game_inputs as game
        with self.assertRaisesRegex(ValueError,'outside repository'):
            game.export(*([Path('/missing')]*5),'LAN201904010',Path(__file__).resolve().parents[1]/'.invalid-game-inputs')
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError,'overwrite'):
                game.export(*([Path('/missing')]*5),'LAN201904010',Path(directory))

    def test_receipt_only_after_complete_source_stable_exclusive_bundle(self):
        from preview import game_inputs as game
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source';source.write_text('original');pins=(real.digest(source),)
            bundle=dict(game_id='LAN201904010',lineup='one\n',cast='two\n')
            with patch('preview.real_lineup.os.fsync',side_effect=lambda _:source.write_text('changed')):
                with self.assertRaisesRegex(ValueError,'changed'):game._publish(bundle,[source],pins,root/'bad')
            self.assertFalse((root/'bad/receipt.json').exists())
            source.write_text('original');receipt=game._publish(bundle,[source],pins,root/'good')
            self.assertTrue((root/'good/receipt.json').exists())
            self.assertEqual((root/'good/lineup.tsv').read_text(),'one\n')
            self.assertEqual(receipt['game_id'],'LAN201904010')
            with self.assertRaisesRegex(ValueError,'overwrite'):game._publish(bundle,[source],pins,root/'good')

    def test_drift_after_first_file_never_publishes_an_acceptance_receipt(self):
        from preview import game_inputs as game
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source';source.write_text('original');pins=(real.digest(source),)
            calls=[]
            def drift(_):
                calls.append(1)
                if len(calls)==2:source.write_text('changed')
            with patch('preview.real_lineup.os.fsync',side_effect=drift):
                with self.assertRaisesRegex(ValueError,'changed'):
                    game._publish(dict(lineup='one\n',cast='two\n'),[source],pins,root/'bad')
            self.assertTrue((root/'bad/lineup.tsv').exists())
            self.assertFalse((root/'bad/cast.tsv').exists())
            self.assertFalse((root/'bad/receipt.json').exists())

    def test_exception_flags_are_refused_not_dropped_or_normalized(self):
        from preview import game_inputs as game
        for flag in ['shortened','suspended','forfeit','tied']:
            result=dict(shortened=False,suspended=False,forfeit=False,tied=False,duration_outs=54)
            result[flag]=True
            source=dict(data=dict(game=[dict(game_id='LAN201904010')],person=[],appearance=[]),
                readiness=dict(games=[dict(game_id='LAN201904010',results=[result])]))
            with self.subTest(flag=flag),self.assertRaisesRegex(ValueError,'Exceptional'):
                game.build_game(source,'LAN201904010')
            with self.assertRaisesRegex(ValueError,'Unknown'):game.build_game(source,'OTHER')

    def test_unaccepted_source_refuses_without_output(self):
        from preview import game_inputs as game
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source';source.write_text('{}')
            with self.assertRaisesRegex(ValueError,'digest'):
                game.export(*([source]*5),'LAN201904010',root/'out')
            self.assertFalse((root/'out').exists())


class GenericCliTests(unittest.TestCase):
    def run_game(self, root, dh=False, mutate=None):
        from preview import pitching_cast as cast
        from test_live_preview import joint_inputs
        baseline,contact=joint_inputs(root)
        p=GenericCastTests().parts(dh)
        lineup=real.encode_witness(p[1],generic=True);pitchers=cast.encode(*p,generic=True,empirical=True)
        if mutate:lineup,pitchers=mutate(lineup,pitchers)
        (root/'lineup.tsv').write_text(lineup);(root/'cast.tsv').write_text(pitchers)
        command=[str(Path(__file__).resolve().parents[1]/'engine/target/debug/examples/text_game'),
            '--baseline',str(baseline),'--contact-counts',str(contact),'--lineup',str(root/'lineup.tsv'),
            '--cast',str(root/'cast.tsv'),'--seed','1']
        return subprocess.run(command,capture_output=True,text=True,timeout=30),command

    def test_generic_cli_completes_dh_and_non_dh_without_false_team_labels(self):
        for dh in [False,True]:
            with self.subTest(dh=dh),tempfile.TemporaryDirectory() as directory:
                result,command=self.run_game(Path(directory),dh)
                self.assertEqual(result.returncode,0,result.stderr)
                self.assertIn('San Francisco Giants at Los Angeles Dodgers',result.stdout)
                self.assertIn('Source witness LAN201904010',result.stdout)
                self.assertNotIn('Seattle',result.stdout);self.assertNotIn('Oakland',result.stdout)
                self.assertEqual(subprocess.check_output(command,text=True),result.stdout)
                radio=subprocess.check_output(command+['--format','radio'],text=True)
                self.assertIn('San Francisco Giants at Los Angeles Dodgers — radio transcript',radio)

    def test_game_mismatch_and_absent_incoming_batting_profile_refuse_before_output(self):
        mutations=[lambda a,b:(a,b.replace('LAN201904010','LAN201904020')),
            lambda a,b:(a,'\n'.join(s for s in b.splitlines() if not s.startswith('batter:ar0\t'))+'\n')]
        for mutate in mutations:
            with tempfile.TemporaryDirectory() as directory:
                result,_=self.run_game(Path(directory),mutate=mutate)
                self.assertNotEqual(result.returncode,0)
                self.assertEqual(result.stdout,'')


if __name__=='__main__':unittest.main()
