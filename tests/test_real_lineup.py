"""Synthetic lineup export contract; source rows remain outside Git."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from preview import real_lineup as real


def witness():
    teams=[]
    for side,prefix,team_id,name in [('Away','a','SEA','Seattle Mariners'),('Home','h','OAK','Oakland Athletics')]:
        order=[dict(slot=i,player_id=prefix+str(i),name=f'Player {prefix}{i}',bats='L',throws='R') for i in range(1,10)]
        positions=[7,8,3,10,9,2,5,6,4]
        field=[dict(position=1,player_id=prefix+'p',name=f'Pitcher {prefix}',bats='R',throws='L')]+[dict(position=pos,**{k:v for k,v in p.items() if k!='slot'}) for p,pos in zip(order,positions)]
        teams.append(dict(side=side,team_id=team_id,team_name=name,pitcher_id=prefix+'p',dh_slot=4,engine_dh_slot=3,batting_order=order,fielding=field))
    return dict(game_id='OAK201903200',date='2019-03-20',park_id='TOK01',use_dh=True,teams=teams)


class RealLineupTests(unittest.TestCase):
    def test_export_binds_order_and_positions_without_using_ordinals(self):
        w=witness();before=copy.deepcopy(w);text=real.encode_witness(w)
        self.assertTrue(text.startswith('key\tvalue\nformat\tclosed-starting-lineup-v1\n'))
        self.assertIn('Away:0:1:ap\tPitcher a\n',text)
        self.assertIn('Away:1:7:a1\tPlayer a1\n',text)
        self.assertIn('Home:4:10:h4\tPlayer h4\n',text)
        self.assertEqual(len(text.splitlines()),31)
        self.assertEqual(w,before)
        shuffled=copy.deepcopy(w)
        for team in shuffled['teams']:team['fielding'].reverse();team['batting_order'].reverse()
        shuffled['teams'].reverse()
        self.assertEqual(real.encode_witness(shuffled),text)
        self.assertIn(real.ATTRIBUTION,text)
        self.assertIn(real.REPORT_SHA,text)

    def test_invalid_structural_or_control_fields_cannot_export(self):
        for mutate in [lambda w:w.update(use_dh=False),lambda w:w.update(use_dh=1),lambda w:w.update(game_id='OTHER'),
            lambda w:w['teams'][0]['batting_order'][0].update(slot=True),
            lambda w:w['teams'][0]['fielding'][0].update(position=True),
            lambda w:w['teams'][0]['batting_order'][0].update(name='bad\tname'),
            lambda w:w['teams'][0]['batting_order'][0].update(name='bad\x85name'),
            lambda w:w['teams'][0]['fielding'][1].update(position=1),
            lambda w:w['teams'][0]['batting_order'][0].update(player_id='other'),
            lambda w:w['teams'][0].update(pitcher_id='a4'),
            lambda w:w['teams'][0].update(dh_slot=1),
            lambda w:w['teams'][1]['batting_order'][0].update(player_id='a1')]:
            with self.subTest(mutate=mutate),self.assertRaises(ValueError):
                w=witness();mutate(w);real.encode_witness(w)

    def test_c1_control_in_matching_names_is_rejected_not_normalized(self):
        w=witness();name='bad'+chr(133)+'name'
        w['teams'][0]['batting_order'][0]['name']=name
        next(p for p in w['teams'][0]['fielding'] if p['player_id']=='a1')['name']=name
        with self.assertRaisesRegex(ValueError,'literal'):real.encode_witness(w)

    def test_repository_output_is_rejected_before_missing_input(self):
        output=Path(__file__).resolve().parents[1]/'.real-lineup-refusal.tsv'
        self.assertFalse(output.exists())
        with self.assertRaisesRegex(ValueError,'outside repository'):real.export(Path('/missing/report'),output)
        self.assertFalse(output.exists())

    def test_drift_during_staging_cannot_publish_or_leave_temporary_files(self):
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source';source.write_bytes(b'original');pin=real.digest(source)
            with patch('preview.real_lineup.os.fsync',side_effect=lambda _:source.write_bytes(b'changed')):
                with self.assertRaisesRegex(ValueError,'changed'):real.publish('rows\n',source,pin,root/'out')
            self.assertEqual(list(root.iterdir()),[source])

    def test_unaccepted_report_refuses_before_output_creation(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);report=root/'report.json';report.write_text(json.dumps({'witnesses':{'primary':witness()}}))
            with self.assertRaisesRegex(ValueError,'digest'):real.export(report,root/'lineup.tsv')
            self.assertFalse((root/'lineup.tsv').exists())

    def test_existing_output_refuses_before_missing_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);out=root/'lineup.tsv';out.write_text('keep')
            with self.assertRaisesRegex(ValueError,'overwrite'):real.export(root/'missing',out)
            self.assertEqual(out.read_text(),'keep')

    def test_final_reread_and_exclusive_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source';source.write_bytes(b'original')
            import hashlib
            pin=hashlib.sha256(source.read_bytes()).hexdigest()
            source.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError,'changed'):real.publish('rows\n',source,pin,root/'out')
            self.assertFalse((root/'out').exists())
            source.write_bytes(b'original');real.publish('rows\n',source,pin,root/'out')
            self.assertEqual((root/'out').read_text(),'rows\n')
            with self.assertRaises(FileExistsError):real.publish('wrong',source,pin,root/'out')
            self.assertEqual((root/'out').read_text(),'rows\n')


class NativeLineupCliTests(unittest.TestCase):
    def test_native_optin_uses_loaded_personnel_names_and_preserves_plain_joint(self):
        import subprocess
        from test_live_preview import joint_inputs, BINARY
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);baseline,counts=joint_inputs(root)
            lineup=root/'lineup.tsv';lineup.write_text(real.encode_witness(witness()))
            command=[str(BINARY),'--baseline',str(baseline),'--contact-counts',str(counts),'--seed','1','--autoplay','--trace']
            plain=subprocess.check_output(command,text=True,timeout=30)
            result=subprocess.run(command+['--lineup',str(lineup)],capture_output=True,text=True,timeout=30)
            self.assertEqual(result.returncode,0,result.stderr)
            self.assertIn('Policy: station-held-positional-starting-preview-v1',result.stdout)
            self.assertIn('TEAM Away: Seattle Mariners',result.stdout)
            self.assertIn('PLAYER Away 4 10 a4: Player a4',result.stdout)
            self.assertIn('Pitcher h [hp] to Player a1 [a1]',result.stdout)
            self.assertIn('"a1"',result.stdout)
            self.assertNotIn('fictional equal teams',result.stdout)
            self.assertIn('no substitutions',result.stdout)
            self.assertEqual(subprocess.check_output(command+['--lineup',str(lineup)],text=True,timeout=30),result.stdout)
            self.assertEqual(subprocess.check_output(command,text=True,timeout=30),plain)

    def test_lineup_requires_joint_and_malformed_file_never_falls_back(self):
        import subprocess
        from test_live_preview import joint_inputs, BINARY
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);baseline,counts=joint_inputs(root);lineup=root/'bad.tsv';lineup.write_text('bad')
            command=[str(BINARY),'--baseline',str(baseline),'--lineup',str(lineup),'--autoplay','--trace']
            result=subprocess.run(command,capture_output=True,text=True,timeout=30)
            self.assertEqual(result.returncode,2)
            self.assertIn('requires --contact-counts',result.stderr)
            self.assertNotIn('Play 01',result.stdout)
            result=subprocess.run(command+['--contact-counts',str(counts)],capture_output=True,text=True,timeout=30)
            self.assertNotEqual(result.returncode,0)
            self.assertNotIn('Play 01',result.stdout)


if __name__=='__main__':unittest.main()
