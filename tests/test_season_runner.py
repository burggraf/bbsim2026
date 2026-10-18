"""Synthetic manifest/accounting contracts; real season evidence stays outside Git."""
import copy
import hashlib
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from preview import season_runner as season
import test_season_game_inputs as fixtures


def entry(gid='LAN201904010'):
    return dict(game_id=gid,date='2019-04-01',away_team_id='SFN',home_team_id='LAN',
        seed=7,actors={'Away':['a'],'Home':['h']},
        role_actors={side:{role:[pid] for role in season.METRICS} for side,pid in [('Away','a'),('Home','h')]})


def accounting():
    lines=['format\tcontrolled-accounting-v1','game\tLAN201904010','seed\t7',
        'engine\t0.1.0','mechanics\ttest','ratings\ttest','manager\ttest','rng\ttest','rules\t2019',
        'team\tAway\tSFN\t1','team\tHome\tLAN\t0']
    for role,side,pid,values in [
        ('batter','Away','a',[29,28,1,1,None,1,27,0,0,0,1,0,0,0,0,0,0,0,0]),
        ('batter','Home','h',[27,27,0,0,0,0,27,0,0,0,0,0,0,0,0,0,0,0,0]),
        ('pitcher','Away','a',[27,27,27,0,0,0,0,27,0,0,0,0,0,0,0,0,0,0]),
        ('pitcher','Home','h',[29,27,28,1,None,1,1,27,0,0,0,1,0,0,0,0,0,0]),
        ('fielder','Away','a',[27,27,0,0,0]),('fielder','Home','h',[29,27,0,0,0])]:
        lines.append('\t'.join([role,side,pid,*('?' if n is None else str(n) for n in values)]))
    return '\n'.join(lines)+'\n'


class ManifestTests(unittest.TestCase):
    def test_identity_seed_is_order_independent_and_has_an_exact_algorithm(self):
        expected=int.from_bytes(hashlib.sha256(b'bbsim-season-v1:1:LAN201904010').digest()[:8],'big')
        self.assertEqual(season.game_seed(1,'LAN201904010'),expected)
        for seed in [-1,2**64,True,'1']:
            with self.assertRaises(ValueError):season.game_seed(seed,'LAN201904010')
        for gid in ['bad','LAN201902290','LAN202004010','LAN201904013']:
            with self.assertRaises(ValueError):season.game_seed(1,gid)

    def test_all_ids_sorted_once_and_schedule_dispositions_untouched(self):
        games=[dict(entry(gid),date=day) for gid,day in [('LAN201904020','2019-04-02'),('LAN201904010','2019-04-01')]]
        source=dict(data=dict(game=games),readiness=dict(rows=[dict(status='ambiguous',candidate_game_ids=[g['game_id'] for g in games]),dict(status='not_made_up',game_id=None)]))
        before=copy.deepcopy(source);rows=season.manifest(source,1)
        self.assertEqual([r['game_id'] for r in rows],['LAN201904010','LAN201904020'])
        source['data']['game'].reverse()
        self.assertEqual(season.manifest(source,1),rows)
        source['data']['game'].reverse();self.assertEqual(source,before)
        source['data']['game'].append(games[0])
        with self.assertRaisesRegex(ValueError,'duplicate'):season.manifest(source,1)


class AccountingTests(unittest.TestCase):
    def test_strict_native_rows_preserve_unknowns_and_stable_identity(self):
        result=season.parse_accounting(accounting(),entry())
        self.assertIsNone(next(p for p in result['players'] if p['role']=='batter' and p['player_id']=='a')['stats']['rbi'])
        self.assertEqual(result['teams']['SFN']['runs'],1)
        self.assertEqual(result['teams']['LAN']['losses'],1)
        self.assertEqual(len(result['players']),6)

    def test_duplicate_missing_mismatched_and_nonconserving_rows_refuse(self):
        text=accounting()
        for bad in [text+text.splitlines()[-1]+'\n',text.replace('seed\t7','seed\t8'),
            text.replace('LAN201904010','LAN201904020'),text.replace('team\tHome\tLAN','team\tHome\tNYN'),
            text.replace('batter\tAway\ta\t29','batter\tAway\ta\t30'),
            text.replace('pitcher\tHome\th\t29','pitcher\tHome\th\t27'),
            text.replace('fielder\tAway\ta\t27\t27','fielder\tAway\ta\t27\t26'),
            text.replace('batter\tAway\ta\t29','batter\tAway\ta\t029'),
            text.replace('batter\tAway\ta\t29','batter\tAway\ta\t?'),
            text.replace('pitcher\tHome\th\t29','pitcher\tHome\th\t-1'),
            text.replace('batter\tAway\ta','batter\tAway\tstranger'),
            '\n'.join(s for s in text.splitlines() if not s.startswith('pitcher\tHome'))+'\n',
            text+'unknown\tvalue\n']:
            with self.subTest(bad=bad),self.assertRaises(ValueError):season.parse_accounting(bad,entry())

    def test_opposing_credits_and_known_cell_relationships_are_checked(self):
        for role,side,key,value in [('pitcher','Home','so',26),('pitcher','Home','er',2),
                ('batter','Away','ab',29),('batter','Away','ibb',2),('batter','Away','so',28)]:
            lines=accounting().splitlines()
            i=next(i for i,line in enumerate(lines) if line.startswith(role+'\t'+side+'\t'))
            cells=lines[i].split('\t');cells[3+season.METRICS[role].index(key)]=str(value)
            lines[i]='\t'.join(cells)
            with self.subTest(role=role,key=key),self.assertRaises(ValueError):
                season.parse_accounting('\n'.join(lines)+'\n',entry())

    def test_known_game_actor_in_an_unpermitted_role_refuses(self):
        for role in ['batter','pitcher','fielder']:
            e=entry();e['role_actors']['Home'][role]=[]
            with self.subTest(role=role),self.assertRaises(ValueError):
                season.parse_accounting(accounting(),e)

    def test_malformed_and_cross_side_actor_declarations_refuse(self):
        for actors in [None,{},[],{'Away':{},'Home':{}},
                {'Away':{r:[{}] for r in season.METRICS},'Home':{r:['h'] for r in season.METRICS}},
                {s:{r:['both'] for r in season.METRICS} for s in ['Away','Home']}]:
            e=entry();e['role_actors']=actors
            with self.subTest(actors=actors),self.assertRaises(ValueError):season.parse_accounting(accounting(),e)

    def test_team_splits_and_unknown_propagation_are_not_partial_totals(self):
        first=season.parse_accounting(accounting(),entry());second=copy.deepcopy(first)
        for p in second['players']:
            if p['team_id']=='SFN':p['team_id']='NYN'
        second['teams']['NYN']=second['teams'].pop('SFN')
        for p in second['players']:
            if p['role']=='batter' and p['player_id']=='a':p['stats']['rbi']=2
        totals=season.accumulate([first,second])
        actor=totals['players']['batter:a']
        self.assertEqual(actor['stats']['pa'],58)
        self.assertIsNone(actor['stats']['rbi'])
        self.assertEqual(set(actor['teams']),{'SFN','NYN'})
        self.assertEqual(actor['teams']['NYN']['rbi'],2)
        self.assertEqual(totals['teams']['LAN']['losses'],2)


class PhAccountingTests(unittest.TestCase):
    def ph_text(self):
        from preview import bench_inputs as bench
        return accounting().replace('controlled-accounting-v1','controlled-accounting-ph-v1').replace(
            'seed\t7\n',f'seed\t7\npinch_hitting\t{bench.POLICY}\navailability_basis\t{bench.AVAILABILITY}\ndefense_basis\t{bench.DEFENSE_BASIS}\n')

    def test_explicit_ph_metadata_and_declared_roles_are_required(self):
        e=entry();e['accounting_format']='controlled-accounting-ph-v1'
        result=season.parse_accounting(self.ph_text(),e)
        self.assertEqual(result['accounting']['format'],'controlled-accounting-ph-v1')
        self.assertEqual(result['accounting']['seed'],'7')
        for text in [self.ph_text().replace('pinch_hitting\t','wrong_policy\t'),
                self.ph_text().replace('not verified eligibility','verified eligibility'),
                self.ph_text().replace('not verified defensive eligibility','verified defense'),
                accounting().replace('controlled-accounting-v1','controlled-accounting-ph-v1'),
                self.ph_text().replace('batter\tAway\ta','batter\tAway\tbench')]:
            with self.subTest(text=text),self.assertRaises(ValueError):season.parse_accounting(text,e)
        with self.assertRaises(ValueError):season.parse_accounting(self.ph_text(),entry())
        with self.assertRaises(ValueError):season.parse_accounting(accounting(),e)
        e['role_actors']['Away']['batter'].append('bench')
        result=season.parse_accounting(self.ph_text().replace('batter\tAway\ta','batter\tAway\tbench'),e)
        self.assertIn('bench',[p['player_id'] for p in result['players']])
        with self.assertRaises(ValueError):
            season.parse_accounting(self.ph_text().replace('pitcher\tAway\ta','pitcher\tAway\tbench'),e)

    def bundle(self):
        from preview import bench_inputs as bench
        p=fixtures.GenericCastTests().parts(False)
        return dict(lineup=season.game.real.encode_witness(p[1],generic=True),
            cast=season.game.cast.encode(*p,empirical=True,generic=True),
            bench='key\tvalue\nformat\tassumed-game-bench-v1\ngame_id\tLAN201904010\n'+
                f'policy\t{bench.POLICY}\navailability_basis\t{bench.AVAILABILITY}\ndefense_basis\t{bench.DEFENSE_BASIS}\n'+
                'Away:bench:name\tBench Actor\nAway:bench:positions\t7\nAway:bench:counts\t0,0,0,0,0,0,0,0,0\n')

    def test_bench_bindings_expand_only_batting_and_fielding(self):
        bundle=self.bundle();bindings=season.actor_bindings(bundle)
        self.assertIn('bench',bindings['Away']['batter'])
        self.assertIn('bench',bindings['Away']['fielder'])
        self.assertNotIn('bench',bindings['Away']['pitcher'])
        self.assertNotIn('bench',bindings['Home']['batter'])
        for text in [bundle['bench'].replace(':positions',':pitcher'),
                bundle['bench']+'Home:bench:name\tBench Actor\nHome:bench:positions\t7\nHome:bench:counts\t0,0,0,0,0,0,0,0,0\n',
                bundle['bench'].replace('Away:bench:','Away:h1:'),
                bundle['bench'].replace('Away:bench:positions\t7\n',''),
                bundle['bench'].replace('Away:bench:name','Unknown:bench:name'),
                bundle['bench'].replace('LAN201904010','LAN201904020')]:
            with self.subTest(text=text),self.assertRaises(ValueError):
                season.actor_bindings(dict(bundle,bench=text))


class NativeTests(unittest.TestCase):
    def test_existing_native_ph_output_reconciles_with_expanded_bindings(self):
        from preview import bench_inputs as bench
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);_,command=fixtures.GenericCliTests().run_game(root,False)
            bundle=PhAccountingTests().bundle()
            text=bundle['bench']
            text+=f'snapshot_sha256\t{season.game.real.SNAPSHOT_SHA}\nsupplement_sha256\t{season.game.real.SUPPLEMENT_SHA}\n'
            text+=f'opportunity_report_sha256\t{season.game.ratings.REPORT_SHA}\nattribution\t{season.game.real.ATTRIBUTION}\n'
            text=text.replace('0,0,0,0,0,0,0,0,0','0,0,0,0,0,0,100,0,0')
            path=root/'bench.tsv';path.write_text(text);bundle['bench']=text
            e=entry();e.update(seed=1,accounting_format='controlled-accounting-ph-v1',role_actors=season.actor_bindings(bundle))
            native=subprocess.run(command+['--bench',str(path),'--format','accounting'],capture_output=True,text=True,timeout=30)
            self.assertEqual(native.returncode,0,native.stderr)
            parsed=season.parse_accounting(native.stdout,e)
            self.assertEqual(parsed['accounting']['pinch_hitting'],bench.POLICY)
            self.assertEqual(sum(t['pa'] for t in parsed['teams'].values()),sum(p['stats']['pa'] for p in parsed['players'] if p['role']=='batter'))
            self.assertEqual(subprocess.check_output(command+['--bench',str(path),'--format','accounting'],text=True),native.stdout)
            with self.assertRaises(ValueError):season.parse_accounting(native.stdout,dict(e,accounting_format='controlled-accounting-v1'))

    def test_actual_native_dh_and_non_dh_accounting_repeats_and_matches_radio_scores(self):
        for dh in [False,True]:
            with self.subTest(dh=dh),tempfile.TemporaryDirectory() as directory:
                _,command=fixtures.GenericCliTests().run_game(Path(directory),dh)
                radio=subprocess.run(command+['--format','radio'],capture_output=True,text=True,timeout=30)
                self.assertEqual(radio.returncode,0,radio.stderr)
                native=subprocess.run(command+['--format','accounting'],capture_output=True,text=True,timeout=30)
                self.assertEqual(native.returncode,0,native.stderr)
                self.assertEqual(subprocess.check_output(command+['--format','accounting'],text=True),native.stdout)
                e=entry();e['seed']=1
                parts=fixtures.GenericCastTests().parts(dh)
                e['actors']={t['side']:sorted({p['player_id'] for p in t['fielding']}|{p['player_id'] for p in next(b for b in parts[2]['teams'] if b['side']==t['side'])['observed_pitchers']}) for t in parts[1]['teams']}
                e['role_actors']=season.actor_bindings(dict(
                    lineup=Path(command[command.index('--lineup')+1]).read_text(),
                    cast=Path(command[command.index('--cast')+1]).read_text()))
                parsed=season.parse_accounting(native.stdout,e)
                from preview.usage_ledger import Ledger
                lineup_keys=[line.split('\t')[0].split(':') for line in Path(command[command.index('--lineup')+1]).read_text().splitlines() if line.startswith(('Away:','Home:'))]
                ids={'Away':'SFN','Home':'LAN'}
                e['starting_batter_ids']={tid:[k[3] for k in lineup_keys if k[0]==side and k[1]!='0'] for side,tid in ids.items()}
                e['starting_pitcher_ids']={tid:next(k[3] for k in lineup_keys if k[0]==side and k[2]=='1') for side,tid in ids.items()}
                ledger=Ledger([e]);self.assertTrue(ledger.accept(e,parsed,'a'*64))
                snapshot=ledger.snapshot()
                self.assertEqual(sum(p['usage'] for p in snapshot['players'].values() if p['role']=='batter'),sum(t['pa'] for t in parsed['teams'].values()))
                self.assertEqual(sum(p['usage'] for p in snapshot['players'].values() if p['role']=='pitcher'),sum(t['outs'] for t in parsed['teams'].values()))
                self.assertEqual(sum(p['starts'] for p in snapshot['players'].values() if p['role']=='batter'),18)
                self.assertEqual(sum(p['starts'] for p in snapshot['players'].values() if p['role']=='pitcher'),2)
                self.assertEqual(len(parsed['teams']),2)
                for team,name in [('SFN','San Francisco Giants'),('LAN','Los Angeles Dodgers')]:
                    self.assertIn(name,radio.stdout)
                    self.assertGreater(parsed['teams'][team]['pa'],0)
                self.assertIn(f"Final: SFN {parsed['teams']['SFN']['runs']}, LAN {parsed['teams']['LAN']['runs']}.",radio.stdout)


class PreparationTests(unittest.TestCase):
    def test_full_cohort_workload_is_computed_once_without_mutating_the_source(self):
        source=dict(data=dict(game=[],person=[],appearance=[dict(fielding_position=1,player_id='p')]),rows=[])
        before=copy.deepcopy(source)
        with patch('preview.game_inputs.workload.inventory',return_value=dict(players=[dict(player_id='p')])) as inventory:
            prepared=season.game.prepare_season(source)
        self.assertEqual(source,before)
        self.assertEqual(prepared['_workloads'],{'p':dict(player_id='p')})
        self.assertEqual(inventory.call_args.args[-1],['p'])

    def test_prepared_and_ordinary_exports_are_identical(self):
        p=fixtures.GenericCastTests().parts(False);report,witness,bullpen,workloads=p
        for role in report['roles'].values():
            for profile in role['profiles']:profile.setdefault('events',profile['included_pa'])
        profiles={p['player_id']:p for p in report['roles']['pitcher']['profiles']}
        for p in workloads['players']:p['ability_profile']=copy.deepcopy(profiles[p['player_id']])
        ids=[v['player_id'] for v in workloads['players']]
        source=dict(data=dict(game=[dict(game_id=witness['game_id'])],person=[],
            appearance=[dict(game_id=witness['game_id'],fielding_position=1,player_id=pid) for pid in ids]),
            report=report,rows=[],readiness=dict(games=[dict(game_id=witness['game_id'],results=[dict(
                forfeit=False,shortened=False,suspended=False,tied=False,duration_outs=54)])]))
        with patch('preview.game_inputs.lineup.inventory',return_value=dict(witnesses=dict(primary=witness))),\
             patch('preview.game_inputs.bullpen.inventory',return_value=bullpen),\
             patch('preview.game_inputs.workload.inventory',return_value=workloads) as inventory:
            ordinary=season.game.build_game(source,witness['game_id'])
            cached=dict(source,_workloads={v['player_id']:v for v in workloads['players']})
            prepared=season.game.build_game(cached,witness['game_id'])
        self.assertEqual(prepared,ordinary)
        self.assertEqual(inventory.call_count,1)

    def test_historical_results_do_not_change_full_game_inputs(self):
        for dh in [False,True]:
            report,witness,bullpen,workloads=fixtures.GenericCastTests().parts(dh)
            for role in report['roles'].values():
                for profile in role['profiles']:profile.setdefault('events',profile['included_pa'])
            profiles={p['player_id']:p for p in report['roles']['pitcher']['profiles']}
            for p in workloads['players']:p['ability_profile']=copy.deepcopy(profiles[p['player_id']])
            source=dict(data=dict(game=[dict(game_id=witness['game_id'])],person=[],appearance=[
                dict(game_id=witness['game_id'],fielding_position=1,player_id=p['player_id'])
                for p in workloads['players']]),report=report,rows=[])
            normal=dict(forfeit=False,shortened=False,suspended=False,tied=False,duration_outs=54)
            cases=[[normal]]+[[dict(normal,**{flag:True})] for flag in ['shortened','suspended','forfeit','tied']]
            cases += [[dict(normal,duration_outs=n)] for n in [22,45,66,None]]+[[],None,[normal,normal]]
            with patch('preview.game_inputs.lineup.inventory',return_value=dict(witnesses=dict(primary=witness))),\
                 patch('preview.game_inputs.bullpen.inventory',return_value=bullpen),\
                 patch('preview.game_inputs.workload.inventory',return_value=workloads):
                for results in cases:
                    source['readiness']=dict(games=[dict(game_id=witness['game_id'],results=results)])
                    before=copy.deepcopy(source)
                    with self.subTest(dh=dh,results=results):
                        actual=season.game.build_game(source,witness['game_id'])
                        self.assertEqual(actual['lineup'],season.game.real.encode_witness(witness,generic=True))
                        self.assertEqual(actual['cast'],season.game.cast.encode(report,witness,bullpen,workloads,empirical=True,generic=True))
                        self.assertEqual(actual.get('simulation_policy'),'regular-full-game-v1')
                        self.assertEqual(source,before)


class BatchTests(unittest.TestCase):
    def test_destination_guards_precede_source_reads(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError,'overwrite'):
                season.export(*([Path('/missing')]*8),1,Path(directory))
        with self.assertRaisesRegex(ValueError,'outside repository'):
            season.export(*([Path('/missing')]*8),1,Path(__file__).resolve().parents[1]/'.invalid-season')

    def test_expected_refusal_retains_game_and_native_failure_is_not_a_refusal(self):
        games=[dict(entry('LAN201904010')),dict(entry('LAN201904020'),date='2019-04-02')]
        source=dict(data=dict(game=games),readiness=dict(rows=[dict(status='not_made_up')]))
        with tempfile.TemporaryDirectory() as directory,patch('preview.season_runner.game.build_game',side_effect=ValueError('Exceptional game')):
            result=season.run(source,[],[],Path('/missing'),Path('/missing'),Path('/missing'),1,Path(directory)/'batch')
            self.assertEqual([r['status'] for r in result['games']],['input_refused']*2)
            self.assertFalse(result['complete_season'])
            self.assertEqual(result.get('simulation_policy'),'regular-full-game-v1')
            self.assertEqual(result['schedule'],source['readiness']['rows'])
            self.assertTrue((Path(directory)/'batch/receipt.json').exists())

    def test_native_exit_timeout_and_malformed_output_are_simulation_failures(self):
        pieces=fixtures.GenericCastTests().parts(True)
        bundle=dict(lineup=season.game.real.encode_witness(pieces[1],generic=True),
            cast=season.game.cast.encode(*pieces,empirical=True,generic=True),cold_profiles=[],cast_sizes={})
        source=dict(data=dict(game=[entry()]),readiness=dict(rows=[]))
        for outcome in [subprocess.CompletedProcess([],1),subprocess.TimeoutExpired([],30),subprocess.CompletedProcess([],0)]:
            with self.subTest(outcome=outcome),tempfile.TemporaryDirectory() as directory,\
                 patch('preview.season_runner.game.build_game',return_value=bundle),\
                 patch('preview.season_runner.subprocess.run',side_effect=outcome if isinstance(outcome,Exception) else None,return_value=outcome):
                root=Path(directory)/'batch'
                result=season.run(source,[],[],Path('/missing'),Path('/missing'),Path('/missing'),1,root)
                self.assertEqual(result['games'][0]['status'],'simulation_failed')
                self.assertFalse(result['complete_season'])
                self.assertTrue((root/'games/LAN201904010/stdout.tsv').exists())
                self.assertTrue((root/'receipt.json').exists())

    def test_unexpected_error_or_drift_has_no_acceptance_receipt(self):
        source=dict(data=dict(game=[entry()]),readiness=dict(rows=[]))
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);pin=root/'input';pin.write_text('old')
            def refuse(*_):pin.write_text('changed');raise ValueError('expected refusal')
            with patch('preview.season_runner.game.build_game',side_effect=refuse):
                with self.assertRaisesRegex(ValueError,'changed'):
                    season.run(source,[pin],[hashlib.sha256(b'old').hexdigest()],root/'native',root/'base',root/'contact',1,root/'drift')
            self.assertFalse((root/'drift/receipt.json').exists())
            with patch('preview.season_runner.game.build_game',side_effect=KeyError('bug')):
                with self.assertRaises(KeyError):season.run(source,[],[],root/'native',root/'base',root/'contact',1,root/'bug')
            self.assertFalse((root/'bug/receipt.json').exists())


if __name__=='__main__':unittest.main()
