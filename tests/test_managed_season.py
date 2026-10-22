"""Managed parsing and exactly-once chronological feedback, using synthetic native rows."""
import copy
import importlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from preview import season_manager as manager, season_runner as season, bench_inputs as bench
from preview import managed_inputs
from test_managed_inputs import fixture,fields
import test_season_runner as old
from test_usage_ledger import targets


def accounting(entry,bundle=None):
    from preview import pitcher_policy_v2
    v2=entry.get('accounting_format')=='controlled-accounting-managed-v2'
    version='season-pool-workload-context-2019-v2' if v2 else 'season-pool-workload-context-2019-v1'
    policy=pitcher_policy_v2.POLICY if v2 else manager.POLICY
    format_name='controlled-accounting-managed-v2' if v2 else 'controlled-accounting-managed-v1'
    text=old.accounting().replace('format\tcontrolled-accounting-v1','format\t'+format_name)
    text=text.replace('LAN201904010',entry['game_id']).replace('seed\t7','seed\t'+str(entry['seed'])).replace('manager\ttest','manager\t'+version)
    if bundle:
        lineup=fields(bundle['lineup'])
        for side,pid in [('Away','a'),('Home','h')]:
            batter=next(k.split(':')[3] for k in lineup if k.startswith(side+':1:'))
            pitcher=next(k.split(':')[3] for k in lineup if k.startswith(side+':') and k.split(':')[2]=='1')
            for role,actor in [('batter',batter),('pitcher',pitcher),('fielder',pitcher)]:
                text=text.replace(role+'\t'+side+'\t'+pid+'\t',role+'\t'+side+'\t'+actor+'\t')
    return text+('season_policy\t'+policy+'\navailability_basis\t'+manager.AVAILABILITY+
        '\ndefense_basis\t'+bench.DEFENSE_BASIS+'\npinch_hitting\tlate-close-expected-bases-ph-usage-v1\n')


class ManagedAccountingTests(unittest.TestCase):
    def test_managed_schema_keeps_conservation_unknowns_and_strict_metadata(self):
        e=old.entry();e['accounting_format']='controlled-accounting-managed-v1'
        text=accounting(e);p=season.parse_accounting(text,e)
        self.assertEqual(p['teams']['SFN']['pa'],29)
        self.assertEqual(p['accounting']['season_policy'],manager.POLICY)
        self.assertIsNone(next(p for p in p['players'] if p['role']=='pitcher' and p['team_id']=='LAN')['stats']['er'])
        for bad in [text.replace(manager.AVAILABILITY,'recorded participants'),text.replace(manager.POLICY,'other'),
                text.replace('season-pool-workload-context-2019-v1','test'),text.replace('batter\tAway\ta\t29','batter\tAway\ta\t30'),
                text+'season_policy\t'+manager.POLICY+'\n']:
            with self.subTest(bad=bad),self.assertRaises(ValueError):season.parse_accounting(bad,e)
        with self.assertRaises(ValueError):season.parse_accounting(text,old.entry())

    def test_v2_managed_accounting_requires_its_own_policy_and_manager_metadata(self):
        from preview import pitcher_policy_v2
        e=old.entry();e['accounting_format']='controlled-accounting-managed-v2'
        text=accounting(e);p=season.parse_accounting(text,e)
        self.assertEqual(p['accounting']['season_policy'],pitcher_policy_v2.POLICY)
        self.assertEqual(p['versions']['manager'],pitcher_policy_v2.MANAGER)
        for bad in [text.replace(pitcher_policy_v2.POLICY,manager.POLICY),
                    text.replace(pitcher_policy_v2.MANAGER,'season-pool-workload-context-2019-v1')]:
            with self.assertRaises(ValueError):season.parse_accounting(bad,e)

    def test_managed_actor_binding_is_rederived_from_wires(self):
        s,p,l,g=fixture();b=managed_inputs.build_game(s,p,l,g['game_id'],{})
        self.assertEqual(season.actor_bindings(b),b['role_actors'])
        b['bench']=b['bench'].replace('season-pool-bench-v1','assumed-game-bench-v1')
        with self.assertRaises(ValueError):season.actor_bindings(b)


class ManagedSeasonTests(unittest.TestCase):
    def setUp(self):self.m=importlib.import_module('preview.managed_season')

    def test_cli_selects_registered_v2_alias_and_keeps_v1_default(self):
        import sys
        from preview import pitcher_policy_v2
        paths=['lineups','native','opportunities','events','readiness','targets','baseline','contact','binary','output']
        with patch.object(sys,'argv',['managed_season',*paths,'--pitcher-policy','v2']),\
                patch.object(self.m,'export',return_value={'counts':{},'complete_season':True}) as export:
            self.assertEqual(self.m.main(),0)
        self.assertEqual(export.call_args.args[-1],pitcher_policy_v2.POLICY)
        with patch.object(sys,'argv',['managed_season',*paths]),\
                patch.object(self.m,'export',return_value={'counts':{},'complete_season':True}) as export:
            self.assertEqual(self.m.main(),0)
        self.assertEqual(export.call_args.args[-1],manager.POLICY)

    def inputs(self):
        s,p,l,g=fixture();s['readiness']=dict(rows=[dict(status='ambiguous',candidate_game_ids=[g['game_id']]),dict(status='not_made_up',game_id=None)])
        return s,p,targets(),g

    def fake_native(self,argv,stdout,stderr,**kwargs):
        folder=Path(kwargs['cwd'])
        bundle={key:(folder/(key+'.tsv')).read_text() for key in ['lineup','cast','bench']}
        e=dict(old.entry(),game_id=folder.name,seed=int(argv[argv.index('--seed')+1]))
        stdout.write(accounting(e,bundle).encode())
        return type('Result',(),dict(returncode=0))()

    def fake_native_v2(self,argv,stdout,stderr,**kwargs):
        if '--validate-manager-trace' in argv:
            return type('Result',(),dict(returncode=0,stdout=b'',stderr=b''))()
        folder=Path(kwargs['cwd'])
        bundle={key:(folder/(key+'.tsv')).read_text() for key in ['lineup','cast','bench']}
        (folder/'manager-trace.tsv').write_text('validated by mocked native\n')
        (folder/'event-log.tsv').write_text('validated by mocked native\n')
        e=dict(old.entry(),game_id=folder.name,seed=int(argv[argv.index('--seed')+1]),
            accounting_format='controlled-accounting-managed-v2')
        stdout.write(accounting(e,bundle).encode())
        return type('Result',(),dict(returncode=0,stdout=b'',stderr=b''))()

    def v2_pins(self,root):
        paths=[]
        for name in ['native','baseline','contact']:
            path=Path(root)/name;path.write_text(name);paths.append(path)
        return paths,[season.game.real.digest(path) for path in paths]

    def run_batch(self,s,p,t,out):
        return self.m.run(s,p,t,[],[],Path('/fake/native'),Path('/fake/baseline'),Path('/fake/contact'),1,out)

    def test_accepted_games_feed_ledger_and_preserve_schedule_repeat(self):
        s,p,t,g=self.inputs();before=copy.deepcopy((s,p,t))
        with tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=self.fake_native):
            first=Path(root)/'first';second=Path(root)/'second'
            a=self.run_batch(s,p,t,first);b=self.run_batch(s,p,t,second)
            self.assertTrue(a['complete_season']);self.assertEqual(a['counts'],{'complete':1})
            self.assertEqual(a,b);self.assertEqual((s,p,t),before)
            import json
            ledger=json.loads((first/'ledger.json').read_text());self.assertEqual(ledger['teams']['SFN']['completed_games'],1)
            self.assertEqual(self.m.rebuild(first,s,p,t)[0],ledger)
            self.assertEqual(ledger['teams']['SFN']['pa'],29);self.assertEqual(ledger['teams']['LAN']['outs'],27)
            self.assertEqual(json.loads((first/'schedule.json').read_text()),s['readiness']['rows'])
            for file in first.rglob('*'):
                if file.is_file():self.assertEqual(file.read_bytes(),(second/file.relative_to(first)).read_bytes(),str(file))
            with self.assertRaisesRegex(ValueError,'overwrite'):self.run_batch(s,p,t,first)

    def test_v2_managed_run_persists_trace_and_event_evidence_then_revalidates_it(self):
        from preview import pitcher_policy_v2
        import json
        s,p,t,g=self.inputs()
        for team in p['teams'].values():team['scheduled_games']=162
        with tempfile.TemporaryDirectory() as root:
            inputs,pins=self.v2_pins(root);out=Path(root)/'batch'
            base_ledger=self.m.usage_ledger.Ledger
            class FixtureLedger(base_ledger):
                def snapshot(self):
                    result=super().snapshot()
                    for team in result['teams'].values():
                        team['scheduled_games']=162
                        team['progress']=team['completed_games']/162
                    return result
            with patch.object(self.m.usage_ledger,'Ledger',FixtureLedger),\
                    patch('subprocess.run',side_effect=self.fake_native_v2) as native:
                report=self.m.run(s,p,t,inputs,pins,inputs[0],inputs[1],inputs[2],1,out,pitcher_policy_v2.POLICY)
                rebuilt=self.m.rebuild(out,s,p,t)[0]
            self.assertTrue(report['complete_season'])
            self.assertEqual(report['format'],'managed-season-v2')
            folder=out/'games'/g['game_id']
            receipt=json.loads((folder/'receipt.json').read_text())
            self.assertEqual(receipt['format'],'managed-game-receipt-v2')
            self.assertEqual(set(receipt['output_sha256']),{'lineup.tsv','cast.tsv','bench.tsv','stdout.tsv','stderr.txt','manager-trace.tsv','event-log.tsv'})
            self.assertIn('--manager-trace',receipt['command'])
            self.assertIn('--event-log',receipt['command'])
            self.assertEqual((folder/'manager-trace.tsv').read_text(),'validated by mocked native\n')
            self.assertEqual((folder/'event-log.tsv').read_text(),'validated by mocked native\n')
            self.assertEqual(native.call_count,4)
            self.assertIn('--validate-manager-trace',native.call_args.args[0])
            self.assertEqual(rebuilt,json.loads((out/'ledger.json').read_text()))

    def test_v2_trace_validation_failure_stops_before_ledger_acceptance_and_next_game(self):
        from preview import pitcher_policy_v2
        import json
        s,p,t,g=self.inputs();g2=copy.deepcopy(g);g2.update(game_id='LAN201904020',date='2019-04-02')
        s['data']['game'].append(g2)
        for team in p['teams'].values():team['scheduled_games']=162
        with tempfile.TemporaryDirectory() as root:
            inputs,pins=self.v2_pins(root);out=Path(root)/'batch'
            base_ledger=self.m.usage_ledger.Ledger
            class FixtureLedger(base_ledger):
                def snapshot(self):
                    result=super().snapshot()
                    for team in result['teams'].values():
                        team['scheduled_games']=162
                        team['progress']=team['completed_games']/162
                    return result
            calls=[]
            def native(argv,stdout,stderr,**kwargs):
                calls.append(argv)
                if '--validate-manager-trace' in argv:
                    return type('Result',(),dict(returncode=5,stdout=b'',stderr=b'invalid trace'))()
                return self.fake_native_v2(argv,stdout,stderr,**kwargs)
            with patch.object(self.m.usage_ledger,'Ledger',FixtureLedger),patch('subprocess.run',side_effect=native):
                report=self.m.run(s,p,t,inputs,pins,inputs[0],inputs[1],inputs[2],1,out,pitcher_policy_v2.POLICY)
            self.assertEqual(report['counts'],{'pending':1,'simulation_failed':1})
            self.assertEqual([row['status'] for row in report['games']],['simulation_failed','pending'])
            self.assertEqual(report['games'][0]['reason'],'Native manager trace validation failed (5): invalid trace')
            self.assertEqual(len(calls),2)
            self.assertNotIn('--validate-manager-trace',calls[0])
            self.assertIn('--validate-manager-trace',calls[1])
            failed=out/'games'/g['game_id'];self.assertFalse((failed/'receipt.json').exists())
            self.assertTrue((failed/'manager-trace.tsv').exists());self.assertTrue((failed/'event-log.tsv').exists())
            self.assertFalse((out/'games'/g2['game_id']).exists())
            ledger=json.loads((out/'ledger.json').read_text())
            self.assertFalse(ledger['history_complete']);self.assertEqual(ledger['completed_game_ids'],[])
            self.assertEqual(ledger['teams']['SFN']['completed_games'],0)
            receipt=json.loads((out/'receipt.json').read_text())
            self.assertEqual(receipt['counts'],report['counts']);self.assertFalse(receipt['complete_season'])

    def test_next_game_uses_only_accepted_usage_not_historical_recorded_pa(self):
        s,p,t,g=self.inputs();g2=copy.deepcopy(g);g2.update(game_id='LAN201904060',date='2019-04-06')
        s['data']['game'].append(g2)
        for team in p['teams'].values():team['scheduled_games']=2
        original=g['lineup_map_away']['1'];position=next(k for k,pid in g['fielding_map_away'].items() if pid==original)
        a=p['teams']['SFN']['players'][original]
        reserve=copy.deepcopy(a);reserve.update(player_id='reserve',name='Reserve Batter')
        p['teams']['SFN']['players']['reserve']=reserve;a['targets']['batter']=10
        t=targets([dict(row,starting_pitcher_ids={'SFN':'a','LAN':'h'}) for row in [g,g2]])
        with tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=self.fake_native):
            out=Path(root)/'batch';r=self.run_batch(s,p,t,out)
            self.assertEqual(r['counts'],{'complete':2})
            first=fields((out/'games'/g['game_id']/'lineup.tsv').read_text())
            second=fields((out/'games'/g2['game_id']/'lineup.tsv').read_text())
            self.assertIn(f'Away:1:{position}:{original}',first)
            self.assertIn(f'Away:1:{position}:reserve',second)

    def test_native_failure_stops_without_spending_or_hiding_pending_games(self):
        s,p,t,g=self.inputs();g2=copy.deepcopy(g);g2.update(game_id='LAN201904020',date='2019-04-02')
        s['data']['game'].append(g2)
        for team in p['teams'].values():team['scheduled_games']=2
        def fail(argv,stdout,stderr,**kwargs):
            stderr.write(b'preserved failure\n');return type('Result',(),dict(returncode=3))()
        with tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=fail):
            out=Path(root)/'batch';r=self.run_batch(s,p,t,out)
            self.assertEqual(r['counts'],{'pending':1,'simulation_failed':1});self.assertEqual(r['rotation_turns'],{})
            import json
            ledger=json.loads((out/'ledger.json').read_text())
            self.assertFalse(ledger['history_complete']);self.assertEqual(ledger['teams']['SFN']['completed_games'],0)
            self.assertEqual((out/'games'/g['game_id']/'stderr.txt').read_bytes(),b'preserved failure\n')

    def test_native_launch_error_is_an_explicit_failure_not_an_accepted_game(self):
        s,p,t,g=self.inputs()
        with tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=OSError('launch denied')):
            r=self.run_batch(s,p,t,Path(root)/'batch')
            self.assertEqual(r['counts'],{'simulation_failed':1});self.assertEqual(r['rotation_turns'],{})
            self.assertEqual(r['games'][0]['reason'],'launch denied')

    def test_publication_failure_is_not_reclassified_as_native_failure(self):
        s,p,t,g=self.inputs();publish=self.m.game.ratings.publish
        def fail_receipt(text,inputs,pins,output):
            if Path(output).name=='receipt.json':raise OSError('publication denied')
            return publish(text,inputs,pins,output)
        with tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=self.fake_native),\
                patch.object(self.m.game.ratings,'publish',side_effect=fail_receipt):
            out=Path(root)/'batch'
            with self.assertRaisesRegex(OSError,'publication denied'):self.run_batch(s,p,t,out)
            self.assertFalse((out/'receipt.json').exists())

    def test_input_refusal_stops_before_native_and_changes_no_rotation(self):
        s,p,t,g=self.inputs()
        for a in p['teams']['SFN']['players'].values():a['positions']=[n for n in a['positions'] if n!=2]
        with tempfile.TemporaryDirectory() as root,patch('subprocess.run') as run:
            r=self.run_batch(s,p,t,Path(root)/'batch')
            self.assertEqual(r['counts'],{'input_refused':1});self.assertEqual(r['rotation_turns'],{});run.assert_not_called()

    def test_rebuild_rejects_relabeling_and_missing_receipt_pins(self):
        import json
        for alteration in ['season_policy','versions','missing_pin']:
            with self.subTest(alteration=alteration),tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=self.fake_native):
                s,p,t,g=self.inputs();out=Path(root)/'batch';self.run_batch(s,p,t,out)
                report=json.loads((out/'manifest.json').read_text());receipt=json.loads((out/'receipt.json').read_text())
                if alteration=='season_policy':report['season_policy']='historical'
                if alteration=='versions':report['versions']['manager']='historical'
                (out/'manifest.json').write_text(json.dumps(report))
                receipt['output_sha256']['manifest.json']=season.game.real.digest(out/'manifest.json')
                if alteration=='missing_pin':receipt['output_sha256'].pop('schedule.json')
                (out/'receipt.json').write_text(json.dumps(receipt))
                with self.assertRaises(ValueError):self.m.rebuild(out,s,p,t)

    def test_rebuild_requires_explicit_integer_zero_native_exit(self):
        import json
        for exit_value in [3,None,False,'0']:
            with self.subTest(exit_value=exit_value),tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=self.fake_native):
                s,p,t,g=self.inputs();out=Path(root)/'batch';self.run_batch(s,p,t,out)
                report=json.loads((out/'manifest.json').read_text());receipt=json.loads((out/'receipt.json').read_text())
                if exit_value is None:report['games'][0].pop('exit')
                else:report['games'][0]['exit']=exit_value
                (out/'manifest.json').write_text(json.dumps(report))
                receipt['output_sha256']['manifest.json']=season.game.real.digest(out/'manifest.json')
                (out/'receipt.json').write_text(json.dumps(receipt))
                with self.assertRaisesRegex(ValueError,'exit'):self.m.rebuild(out,s,p,t)

    def test_export_pins_direct_receipt_and_bench_dependencies(self):
        import preview.season_usage as usage
        s,p,t,g=self.inputs();paths=[Path('/fake/'+str(i)).resolve() for i in range(5)]
        target=Path('/fake/targets/targets.json');workload=Path(season.game.workload.__file__)
        hashes=dict(zip(paths,season.game.INPUT_SHA));hashes.update({target:self.m.TARGET_SHA256,workload:season.game.cast.WORKLOAD_RECIPE_SHA})
        with patch.object(season.game.real,'digest',side_effect=lambda path:hashes.get(Path(path),'0'*64)),\
                patch.object(self.m,'_json',return_value=t),patch.object(season.game,'read_sources',return_value=s),\
                patch.object(season.game,'prepare_season',return_value=s),patch.object(manager,'build_pool',return_value=p),\
                patch.object(self.m,'run',return_value={}) as run:
            self.m.export(*paths,Path('/fake/targets'),Path('/fake/baseline'),Path('/fake/contact'),Path('/fake/native'),1,Path('/fake/output'))
        pinned=run.call_args.args[3]
        self.assertTrue({Path(usage.__file__),Path(bench.__file__)}.issubset(pinned))

    def test_changed_pinned_input_never_gets_acceptance_receipt(self):
        s,p,t,g=self.inputs()
        with tempfile.TemporaryDirectory() as root,patch('subprocess.run',side_effect=self.fake_native):
            pin=Path(root)/'input';pin.write_text('before');digest=season.game.real.digest(pin)
            pin.write_text('after');out=Path(root)/'batch'
            with self.assertRaisesRegex(ValueError,'changed'):
                self.m.run(s,p,t,[pin],[digest],Path('/fake/native'),Path('/fake/baseline'),Path('/fake/contact'),1,out)
            self.assertFalse((out/'receipt.json').exists())


if __name__=='__main__':unittest.main()
