"""Source-named contracts for canonical usage, chronology and matched comparisons."""
import copy
import unittest
from preview import season_runner as season
from preview.usage_ledger import Ledger
from test_season_runner import accounting, entry

SHA='a'*64


def game(gid='LAN201904010',day='2019-04-01'):
    e=entry(gid);e['date']=day
    e['starting_batter_ids']={'SFN':['a'],'LAN':['h']}
    e['starting_pitcher_ids']={'SFN':'a','LAN':'h'}
    return e


def parsed(e=None):
    e=e or game()
    return season.parse_accounting(accounting().replace('LAN201904010',e['game_id']),e)


def targets(games=None):
    games=games or [game()]
    rows=[]
    for g in games:
        for team,pid,pa in [('SFN','a',29),('LAN','h',27)]:
            for role,usage in [('batter',pa),('pitcher',27)]:
                rows.append(dict(game_id=g['game_id'],team_id=team,player_id=pid,role=role,
                    usage=usage,unknown_rows=0,source_rows=1))
    return dict(format='2019-season-usage-targets-v1',games=[{**{k:g[k] for k in
        ['game_id','date','away_team_id','home_team_id','starting_pitcher_ids']},
        'suspended':False,'shortened':False,'duration_outs':54} for g in games],
        records=rows,team_games={'SFN':len(games),'LAN':len(games)},
        totals={'pa':56*len(games),'outs':54*len(games)},provenance={'source':'synthetic'},warnings=[])


class UsageLedgerTests(unittest.TestCase):
    def test_canonical_usage_starts_zero_out_appearances_and_recent_workload(self):
        e=game();r=parsed(e)
        # An entered pitcher can face a batter and record zero outs.
        e['role_actors']['Away']['pitcher'].append('relief')
        p=copy.deepcopy(next(p for p in r['players'] if p['role']=='pitcher' and p['team_id']=='SFN'))
        p['player_id']='relief';p['stats']={k:0 for k in p['stats']};p['stats']['bf']=1;p['stats']['ab']=1;p['stats']['so']=1
        old=next(p for p in r['players'] if p['role']=='pitcher' and p['team_id']=='SFN')
        for key in ['bf','ab','so']:old['stats'][key]-=1
        r['players'].append(p)
        ledger=Ledger([e]);self.assertTrue(ledger.accept(e,r,SHA))
        snap=ledger.snapshot()
        self.assertEqual(snap['teams']['SFN']['pa'],29)
        self.assertEqual(snap['teams']['SFN']['outs'],27)
        self.assertEqual(snap['teams']['SFN']['scheduled_games'],1)
        self.assertEqual(snap['players']['batter:a']['usage'],29)
        self.assertEqual(snap['players']['pitcher:a']['starts'],1)
        relief=snap['players']['pitcher:relief']
        self.assertEqual((relief['usage'],relief['appearances'],relief['starts'],relief['bf']),(0,1,0,1))
        self.assertEqual((relief['start_outs'],relief['relief_outs']),(0,0))
        self.assertEqual(snap['players']['pitcher:a']['start_outs'],27)
        self.assertEqual(relief['workload'][0]['date'],'2019-04-01')
        self.assertIn('not verified',snap['limits'])

    def test_exactly_once_chronological_atomic_and_replay(self):
        games=[game(),game('LAN201904012'),game('LAN201904030','2019-04-03')]
        ledger=Ledger(games);empty=ledger.snapshot()
        with self.assertRaises(ValueError):ledger.accept(games[1],parsed(games[1]),SHA)
        self.assertEqual(ledger.snapshot(),empty)
        self.assertTrue(ledger.accept(games[0],parsed(games[0]),SHA))
        once=ledger.snapshot();self.assertFalse(ledger.accept(games[0],parsed(games[0]),SHA))
        self.assertEqual(ledger.snapshot(),once)
        with self.assertRaises(ValueError):ledger.accept(games[0],parsed(games[0]),'b'*64)
        changed=copy.deepcopy(games[0]);changed['seed']=8
        with self.assertRaises(ValueError):ledger.accept(changed,parsed(games[0]),SHA)
        self.assertEqual(ledger.snapshot(),once)
        for e in games[1:]:ledger.accept(e,parsed(e),SHA)
        rebuilt=Ledger(games)
        for e in games:rebuilt.accept(e,parsed(e),SHA)
        self.assertEqual(rebuilt.snapshot(),ledger.snapshot())
        self.assertEqual([w['date'] for w in ledger.snapshot()['players']['pitcher:a']['workload']],
            ['2019-04-01','2019-04-01','2019-04-03'])

    def test_observational_schedule_reports_history_completeness_explicitly(self):
        snapshot=Ledger([game()]).snapshot()
        self.assertIs(snapshot.get('history_complete'),True)
        self.assertEqual(snapshot.get('observed_game_ids'),[])
        self.assertIs(snapshot.get('observed_schedule_complete'),False)

    def test_explicit_refusal_advances_observation_not_completed_clock_and_rebuilds(self):
        games=[game(),game('LAN201904020','2019-04-02'),game('LAN201904030','2019-04-03')]
        ledger=Ledger(games);ledger.accept(games[0],parsed(games[0]),SHA)
        before=ledger.snapshot()
        with self.assertRaises(ValueError):ledger.accept(games[2],parsed(games[2]),SHA)
        self.assertEqual(ledger.snapshot(),before)
        self.assertTrue(ledger.record_refusal(games[1],'Unsupported source personnel'))
        refused=ledger.snapshot()
        self.assertEqual(refused['completed_game_ids'],[games[0]['game_id']])
        self.assertEqual(refused['observed_game_ids'],[g['game_id'] for g in games[:2]])
        self.assertEqual(refused['next_game_id'],games[2]['game_id'])
        self.assertEqual(refused['teams']['SFN']['pa'],before['teams']['SFN']['pa'])
        self.assertEqual(refused['teams']['SFN']['completed_games'],1)
        self.assertEqual(refused['teams']['SFN']['progress'],1/3)
        self.assertEqual(refused['teams']['SFN']['observed_games'],2)
        self.assertEqual(refused['teams']['SFN']['refused_games'],1)
        self.assertFalse(refused['teams']['SFN']['history_complete'])
        self.assertEqual(refused['players'],before['players'])
        self.assertFalse(refused['history_complete'])
        self.assertFalse(refused['complete_schedule'])
        self.assertFalse(refused['observed_schedule_complete'])
        self.assertFalse(ledger.record_refusal(games[1],'Unsupported source personnel'))
        self.assertEqual(ledger.snapshot(),refused)
        self.assertTrue(ledger.accept(games[2],parsed(games[2]),SHA))
        final=ledger.snapshot()
        self.assertTrue(final['observed_schedule_complete'])
        self.assertFalse(final['complete_schedule']);self.assertFalse(final['history_complete'])
        self.assertEqual(final['teams']['SFN']['completed_games'],2)
        self.assertEqual(final['refused_games'][0]['reason'],'Unsupported source personnel')
        rebuilt=Ledger(games)
        rebuilt.accept(games[0],parsed(games[0]),SHA)
        rebuilt.record_refusal(games[1],'Unsupported source personnel')
        rebuilt.accept(games[2],parsed(games[2]),SHA)
        self.assertEqual(rebuilt.snapshot(),final)
        report=ledger.compare(targets(games))
        self.assertEqual(report['matched_game_ids'],[games[0]['game_id'],games[2]['game_id']])
        self.assertEqual(report['aggregate']['pa']['actual'],112)
        self.assertEqual(report['coverage']['refused_games'],1)
        self.assertFalse(report['history_complete'])
        self.assertFalse(report['full_target_schedule'])

    def test_refusal_identity_order_reason_conflicts_are_atomic(self):
        games=[game(),game('LAN201904020','2019-04-02')];ledger=Ledger(games)
        before=ledger.snapshot()
        for e,reason in [(games[1],'later'),(dict(games[0],home_team_id='NYN'),'wrong'),
                (games[0],''),(games[0],None),(games[0],'  ')]:
            with self.subTest(e=e,reason=reason),self.assertRaises(ValueError):ledger.record_refusal(e,reason)
            self.assertEqual(ledger.snapshot(),before)
        ledger.record_refusal(games[0],'source refusal');once=ledger.snapshot()
        for e,reason in [(games[0],'altered'),(dict(games[0],seed=8),'source refusal')]:
            with self.assertRaises(ValueError):ledger.record_refusal(e,reason)
            self.assertEqual(ledger.snapshot(),once)
        with self.assertRaises(ValueError):ledger.accept(games[0],parsed(games[0]),SHA)
        self.assertEqual(ledger.snapshot(),once)
        ledger.accept(games[1],parsed(games[1]),SHA);final=ledger.snapshot()
        with self.assertRaises(ValueError):ledger.record_refusal(games[1],'already accepted')
        self.assertEqual(ledger.snapshot(),final)

    def test_trust_boundaries_versions_partial_and_total_drift_do_not_advance(self):
        e=game();r=parsed(e)
        bad=[]
        for key,value in [('game','LAN201904020'),('seed','8'),('format','fake'),('rules','2020')]:
            p=copy.deepcopy(r);p['accounting'][key]=value;bad.append(p)
        for key in ['pa','outs','bf']:
            p=copy.deepcopy(r);p['teams']['SFN'][key]+=1;bad.append(p)
        p=copy.deepcopy(r);p['players'][0]['team_id']='NYN';bad.append(p)
        p=copy.deepcopy(r);p['teams']['LAN']['runs']=1;bad.append(p)
        for p in bad:
            ledger=Ledger([e]);before=ledger.snapshot()
            with self.subTest(p=p),self.assertRaises(ValueError):ledger.accept(e,p,SHA)
            self.assertEqual(ledger.snapshot(),before)
        for receipt in ['',None,'g'*64]:
            with self.assertRaises(ValueError):Ledger([e]).accept(e,r,receipt)
        g2=game('LAN201904020','2019-04-02');ledger=Ledger([e,g2]);ledger.accept(e,r,SHA)
        p=parsed(g2);p['versions']['manager']='changed';p['accounting']['manager']='changed'
        with self.assertRaises(ValueError):ledger.accept(g2,p,SHA)
        self.assertEqual(ledger.snapshot()['completed_game_ids'],[e['game_id']])

    def test_schedule_refuses_duplicates_wrong_dates_and_nonchronological_order(self):
        for games in [[game(),game()],[game('LAN201904020','2019-04-02'),game()],
                      [game(day='2019-04-02')]]:
            with self.assertRaises(ValueError):Ledger(games)

    def test_team_denominator_and_player_team_splits_not_eligibility(self):
        e=game();g2=game('NYN201904020','2019-04-02');g2['home_team_id']='NYN'
        g2['starting_batter_ids']={'SFN':['a'],'NYN':['h']};g2['starting_pitcher_ids']={'SFN':'a','NYN':'h'}
        r=season.parse_accounting(accounting().replace('LAN201904010',g2['game_id']).replace('Home\tLAN','Home\tNYN'),g2)
        ledger=Ledger([e,g2]);ledger.accept(e,parsed(e),SHA)
        self.assertEqual(ledger.snapshot()['teams']['SFN']['progress'],0.5)
        self.assertEqual(ledger.snapshot()['teams']['NYN']['completed_games'],0)
        ledger.accept(g2,r,SHA)
        self.assertEqual(set(ledger.snapshot()['players']['batter:h']['teams']),{'LAN','NYN'})
        self.assertEqual(ledger.snapshot()['players']['batter:h']['usage'],54)

    def test_comparison_matches_only_accepted_games_and_reports_frozen_criteria(self):
        e=game();g2=game('LAN201904020','2019-04-02');ledger=Ledger([e,g2]);ledger.accept(e,parsed(e),SHA)
        report=ledger.compare(targets([e,g2]))
        self.assertEqual(report['matched_game_ids'],[e['game_id']])
        self.assertEqual(report['aggregate']['pa']['actual'],56)
        self.assertEqual(report['aggregate']['outs']['simulated'],54)
        self.assertTrue(report['aggregate']['pa']['passed'])
        pitcher=next(r for r in report['player_team'] if r['role']=='pitcher' and r['player_id']=='a')
        self.assertEqual((pitcher['actual_starts'],pitcher['actual_start_outs'],pitcher['actual_relief_outs']),(1,27,0))
        self.assertEqual(pitcher['simulated_start_outs'],27)
        self.assertIsNone(report['headline']['batter']['passed'])
        self.assertEqual(report['headline']['batter']['minimum_actual'],100)
        self.assertEqual(report['headline']['pitcher']['minimum_actual'],60)
        self.assertEqual(sum(b['count'] for b in report['bands']['batter'].values()),2)
        self.assertFalse(report['full_target_schedule'])
        self.assertEqual(report['coverage']['matched_games'],1)

    def test_unknown_actual_only_sim_only_not_zero_and_failed_thresholds_visible(self):
        e=game();ledger=Ledger([e]);ledger.accept(e,parsed(e),SHA);t=targets()
        t['records'][0].update(usage=None,unknown_rows=1)
        t['records'][1]['player_id']='actual-only'
        t['games'][0]['starting_pitcher_ids']['SFN']='actual-only'
        t['records'][2]['usage']=200
        t['totals']={'pa':None,'outs':54}
        report=ledger.compare(t)
        unknown=next(p for p in report['player_team'] if p['role']=='batter' and p['player_id']=='a')
        self.assertIsNone(unknown['actual']);self.assertIsNone(unknown['deviation'])
        actual_only=next(p for p in report['player_team'] if p['player_id']=='actual-only')
        self.assertIsNone(actual_only['simulated'])
        sim_only=next(p for p in report['player_team'] if p['role']=='pitcher' and p['player_id']=='a')
        self.assertIsNone(sim_only['actual'])
        self.assertFalse(report['headline']['batter']['passed'])
        self.assertIsNone(report['aggregate']['pa']['actual'])
        self.assertIsNone(report['aggregate']['pa']['passed'])
        self.assertEqual(report['coverage']['actual_only'],1)
        self.assertEqual(report['coverage']['sim_only'],1)

    def test_frozen_eighty_percent_boundary_and_aggregate_failures(self):
        games=[game(f'LAN2019040{i}0',f'2019-04-0{i}') for i in range(1,5)]
        ledger=Ledger(games);t=targets(games)
        for i,e in enumerate(games):
            pid=f'a{i}'
            for role in season.METRICS:e['role_actors']['Away'][role]=[pid]
            e['starting_batter_ids']['SFN']=[pid];e['starting_pitcher_ids']['SFN']=pid
            text=accounting().replace('LAN201904010',e['game_id']).replace('\tAway\ta\t',f'\tAway\t{pid}\t')
            text=text.replace(f'batter\tAway\t{pid}\t29\t28\t1\t1\t?\t1\t',f'batter\tAway\t{pid}\t100\t28\t1\t1\t?\t72\t')
            text=text.replace('pitcher\tHome\th\t29\t27\t28\t1\t?\t1\t1\t','pitcher\tHome\th\t100\t27\t28\t1\t?\t1\t72\t')
            ledger.accept(e,season.parse_accounting(text,e),SHA)
            t['games'][i]['starting_pitcher_ids']['SFN']=pid
            for r in t['records']:
                if r['game_id']==e['game_id'] and r['team_id']=='SFN':
                    r['player_id']=pid
                    if r['role']=='batter':r['usage']=500 if i==3 else 100
        t['totals']['pa']=908
        report=ledger.compare(t)
        self.assertEqual((report['headline']['batter']['count'],report['headline']['batter']['within']),(5,4))
        self.assertEqual(report['headline']['batter']['fraction'],0.8)
        self.assertTrue(report['headline']['batter']['passed'])
        self.assertFalse(report['aggregate']['pa']['passed'])
        self.assertTrue(report['headline']['pitcher']['passed'])
        self.assertEqual(report['headline']['pitcher']['absolute_allowance'],18)
        self.assertEqual(report['headline']['pitcher']['relative_allowance'],0.30)
        self.assertEqual(sum(p['usage'] for p in ledger.snapshot()['players'].values() if p['role']=='batter'),508)
        self.assertEqual(sum(p['usage'] for p in ledger.snapshot()['players'].values() if p['role']=='pitcher'),216)

    def test_missing_target_coverage_and_refused_partial_games_are_atomic(self):
        e=game();ledger=Ledger([e]);before=ledger.snapshot()
        for status in ['input_refused','simulation_failed','partial']:
            bad=dict(e,status=status)
            with self.assertRaises(ValueError):ledger.accept(bad,parsed(e),SHA)
            self.assertEqual(ledger.snapshot(),before)
        text=accounting().replace('pitcher\tHome\th\t29\t27','pitcher\tHome\th\t29\t24').replace('fielder\tHome\th\t29\t27','fielder\tHome\th\t29\t24')
        with self.assertRaises(ValueError):ledger.accept(e,season.parse_accounting(text,e),SHA)
        self.assertEqual(ledger.snapshot(),before)
        ledger.accept(e,parsed(e),SHA)
        t=targets();t['records']=[r for r in t['records'] if not (r['role']=='pitcher' and r['team_id']=='LAN')]
        t['totals']['outs']=27
        with self.assertRaises(ValueError):ledger.compare(t)
        t=targets();t['games']=[];t['records']=[];t['team_games']={};t['totals']={'pa':0,'outs':0}
        with self.assertRaises(ValueError):ledger.compare(t)
        empty=Ledger([e]).compare(targets())
        self.assertIsNone(empty['aggregate']['pa']['actual'])
        self.assertIsNone(empty['headline']['batter']['passed'])

    def test_all_opportunity_bands_including_zero_and_unknown_are_retained(self):
        ledger=Ledger([game()]);ledger.accept(game(),parsed(),SHA);t=targets()
        for role,usages in [('batter',[0,1,100,300,None]),('pitcher',[0,1,60,180,None])]:
            for i,n in enumerate(usages):
                t['records'].append(dict(game_id=game()['game_id'],team_id='SFN',player_id=f'extra-{role}-{i}',
                    role=role,usage=n,unknown_rows=int(n is None),source_rows=1))
        t['totals']={'pa':None,'outs':None}
        report=ledger.compare(t)
        for role in ['batter','pitcher']:
            self.assertEqual(len(report['bands'][role]),5)
            self.assertTrue(all(b['count']>0 for b in report['bands'][role].values()))
        self.assertEqual(report['coverage']['unknown_rows'],2)
        self.assertFalse(report['headline']['batter']['passed'])
        self.assertFalse(report['headline']['pitcher']['passed'])

    def test_input_and_snapshot_mutation_cannot_change_accepted_state(self):
        e=game();r=parsed(e);ledger=Ledger([e]);ledger.accept(e,r,SHA);before=ledger.snapshot()
        e['starting_batter_ids']['SFN'].append('stranger');r['players'][0]['stats']['pa']=999
        snapshot=ledger.snapshot();snapshot['teams']['SFN']['pa']=0
        self.assertEqual(ledger.snapshot(),before)

    def test_targets_duplicate_misbound_unknown_and_total_drift_refuse(self):
        e=game();ledger=Ledger([e]);ledger.accept(e,parsed(e),SHA)
        cases=[]
        t=targets();t['records'].append(copy.deepcopy(t['records'][0]));cases.append(t)
        for key,value in [('team_id','NYN'),('game_id','LAN201904020'),('role','fielder'),('usage',True),('source_rows',0),('unknown_rows',1)]:
            t=targets();t['records'][0][key]=value;cases.append(t)
        t=targets();t['totals']['pa']+=1;cases.append(t)
        t=targets();t['team_games']['LAN']=162;cases.append(t)
        t=targets();t['games'][0]['home_team_id']='NYN';cases.append(t)
        for t in cases:
            with self.subTest(t=t),self.assertRaises(ValueError):ledger.compare(t)


if __name__=='__main__':unittest.main()
