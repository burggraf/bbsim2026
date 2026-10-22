"""Synthetic managed personnel contracts; no daily eligibility reconstruction."""
import copy
import importlib
import unittest
from preview import season_manager as policy
from preview import season_runner as season
from preview.usage_ledger import Ledger
import test_season_game_inputs as generic


def fixture(dh=True):
    report,w,b,work=generic.GenericCastTests().parts(dh)
    pool=dict(format=policy.POLICY,availability_basis=policy.AVAILABILITY,teams={})
    g={k:w[k] for k in ['game_id','date','park_id','use_dh']}
    profiles={r:{p['player_id']:p for p in report['roles'][r]['profiles']} for r in ['batter','pitcher']}
    for t,cast in zip(w['teams'],b['teams']):
        tid=t['team_id'];side=t['side'].lower()
        g.update({side+'_team_id':tid,side+'_team_name':t['team_name'],
            'fielding_map_'+side:{str(p['position']):p['player_id'] for p in t['fielding']},
            'lineup_map_'+side:{str(p['slot']):p['player_id'] for p in t['batting_order']}})
        actors={}
        for a in [*t['fielding'],*cast['observed_pitchers']]:
            pid=a['player_id'];pitch=pid in {p['player_id'] for p in cast['observed_pitchers']}
            actor=actors.setdefault(pid,dict(player_id=pid,name=a['name'],positions=[],starts=int(pid==t['pitcher_id']),
                targets=dict(batter=100 if not pitch else 0,pitcher=30 if pitch else None,
                    start_outs=30 if pid==t['pitcher_id'] else 0,relief_outs=0 if pid==t['pitcher_id'] else 30)))
            pos=a.get('position',1)
            if pos not in actor['positions']:actor['positions'].append(pos)
            for role in profiles:
                profile=profiles[role].get(pid)
                actor[role+'_counts']=[profile['counts'][k] for k in season.game.ratings.KEYS] if profile else [0]*9
                actor['cold_'+role]=not any(actor[role+'_counts'])
        pool['teams'][tid]=dict(scheduled_games=1,players=dict(sorted(actors.items())))
    source=dict(data=dict(game=[g]),_workloads={p['player_id']:p for p in work['players']})
    snap=Ledger([g]).snapshot()
    return source,pool,snap,g


def fields(text):return dict(line.split('\t') for line in text.splitlines()[1:])


class ManagedInputsTests(unittest.TestCase):
    def test_shared_starter_is_reserved_for_the_club_without_an_alternative(self):
        s,p,l,g=fixture();away=p['teams']['SFN']['players'];home=p['teams']['LAN']['players']
        away['ap']['starts']=2
        backup=copy.deepcopy(away['ap']);backup.update(player_id='backup',name='Backup',starts=1)
        away['backup']=backup;home['hp']['starts']=0;home['ap']=copy.deepcopy(away['ap'])
        b=self.m.build_game(s,p,l,g['game_id'],{})
        self.assertEqual(b['starter_decisions']['SFN']['player_id'],'backup')
        self.assertEqual(b['starter_decisions']['LAN']['player_id'],'ap')
        self.assertEqual(b['next_turns'],{'SFN':1,'LAN':0})

    def setUp(self):self.m=importlib.import_module('preview.managed_inputs')

    def build(self,dh=True):
        s,p,l,g=fixture(dh)
        return self.m.build_game(s,p,l,g['game_id'],{}),s,p,l,g

    def test_distinct_truthful_formats_dh_non_dh_and_no_input_mutation(self):
        for dh in [False,True]:
            s,p,l,g=fixture(dh);before=copy.deepcopy((s,p,l))
            b=self.m.build_game(s,p,l,g['game_id'],{})
            self.assertEqual((s,p,l),before)
            for key,fmt in [('lineup','season-pool-lineup-v1'),('cast','season-pool-pitching-cast-v1'),('bench','season-pool-bench-v1')]:
                f=fields(b[key]);self.assertEqual(f['format'],fmt)
                self.assertEqual(f['availability_basis'],policy.AVAILABILITY)
                self.assertEqual(f['season_policy'],policy.POLICY)
            starters=[k.split(':') for k in fields(b['lineup']) if k.startswith(('Away:','Home:'))]
            self.assertEqual(len(starters),20 if dh else 18)
            self.assertEqual(sum(k[1]=='0' for k in starters),2 if dh else 0)
            self.assertEqual(b['next_turns'],{'SFN':0,'LAN':0})

    def test_stable_under_pool_and_workload_record_reordering(self):
        b,s,p,l,g=self.build()
        p['teams']=dict(reversed(list(p['teams'].items())))
        for team in p['teams'].values():team['players']=dict(reversed(list(team['players'].items())))
        s['_workloads']=dict(reversed(list(s['_workloads'].items())))
        for work in s['_workloads'].values():work['outing_rows'].reverse()
        self.assertEqual(self.m.build_game(s,p,l,g['game_id'],{}),b)

    def test_matching_does_not_strand_multi_position_actor(self):
        s,p,l,g=fixture(False);actors=p['teams']['SFN']['players']
        f=g['fielding_map_away'];a,b=f['2'],f['3']
        actors[a]['positions']=[2,3];actors[b]['positions']=[2]
        bundle=self.m.build_game(s,p,l,g['game_id'],{})
        selected={int(k.split(':')[2]):k.split(':')[3] for k in fields(bundle['lineup']) if k.startswith('Away:')}
        self.assertEqual(selected[3],a);self.assertEqual(selected[2],b)

    def test_four_shared_slots_match_brute_force_feasibility(self):
        import itertools,random
        rng=random.Random(2019)
        for case in range(64):
            s,p,l,g=fixture();base=copy.deepcopy(p['teams']['SFN']['players']['a2']);choices={}
            for tid,side in [('SFN','Away'),('LAN','Home')]:
                for a in p['teams'][tid]['players'].values():a['positions']=[pos for pos in a['positions'] if pos not in [2,3]]
                for i in range(4):
                    pid='shared'+str(i);a=copy.deepcopy(base);a.update(player_id=pid,name=pid,positions=[pos for pos in [2,3] if rng.getrandbits(1)])
                    p['teams'][tid]['players'][pid]=a
                for pos in [2,3]:choices[side,pos]={pid for pid,a in p['teams'][tid]['players'].items() if pos in a['positions']}
            slots=list(choices)
            feasible=any(all(pid in choices[slot] for slot,pid in zip(slots,ids)) for ids in itertools.permutations(['shared'+str(i) for i in range(4)]))
            with self.subTest(case=case,feasible=feasible):
                if feasible:self.m.build_game(s,p,l,g['game_id'],{})
                else:
                    with self.assertRaisesRegex(ValueError,'positional lineup'):self.m.build_game(s,p,l,g['game_id'],{})

    def test_proposed_rotation_advances_scheduled_turn_not_callers_state(self):
        s,p,l,g=fixture();actors=p['teams']['SFN']['players'];actors['ap']['starts']=11
        for i in range(4):
            pid='spot'+str(i);a=copy.deepcopy(actors['ap']);a.update(player_id=pid,name=pid,starts=10-i);actors[pid]=a
        turns={'SFN':1,'LAN':0};b=self.m.build_game(s,p,l,g['game_id'],turns)
        self.assertEqual(b['starter_decisions']['SFN']['player_id'],'spot0')
        self.assertEqual(b['next_turns'],{'SFN':2,'LAN':0});self.assertEqual(turns,{'SFN':1,'LAN':0})

    def test_missing_position_refuses_without_inventing_one(self):
        s,p,l,g=fixture()
        for actor in p['teams']['SFN']['players'].values():actor['positions']=[n for n in actor['positions'] if n!=2]
        with self.assertRaisesRegex(ValueError,'positional lineup'):
            self.m.build_game(s,p,l,g['game_id'],{})

    def test_shared_club_actor_never_belongs_to_both_sides(self):
        s,p,l,g=fixture();a=copy.deepcopy(p['teams']['SFN']['players']['ar0'])
        p['teams']['LAN']['players']['ar0']=a
        bundle=self.m.build_game(s,p,l,g['game_id'],{})
        actors=bundle['role_actors']
        away={pid for ids in actors['Away'].values() for pid in ids};home={pid for ids in actors['Home'].values() for pid in ids}
        self.assertFalse(away&home)

    def test_cross_side_matching_preserves_scarce_shared_position(self):
        s,p,l,g=fixture();away=p['teams']['SFN']['players'];home=p['teams']['LAN']['players']
        shared=g['fielding_map_away']['2'];old_home=g['fielding_map_home']['2']
        home[shared]=copy.deepcopy(away[shared]);home[old_home]['positions']=[]
        g['fielding_map_home']['2']=shared
        alternate=copy.deepcopy(away[shared]);alternate.update(player_id='alternate',name='Alternate')
        away['alternate']=alternate
        b=self.m.build_game(s,p,l,g['game_id'],{});f=fields(b['lineup'])
        self.assertTrue(any(k.startswith('Home:') and k.endswith(':2:'+shared) for k in f))
        self.assertTrue(any(k.startswith('Away:') and k.endswith(':2:alternate') for k in f))

    def test_rest_exhaustion_refuses_and_does_not_advance_caller_turns(self):
        s,p,l,g=fixture();turns={'SFN':0,'LAN':0}
        for pid,a in p['teams']['SFN']['players'].items():
            if a['starts']:l['players']['pitcher:'+pid]=dict(teams={},workload=[dict(date=g['date'],usage=0,started=True)])
        with self.assertRaisesRegex(ValueError,'rested supported starter'):
            self.m.build_game(s,p,l,g['game_id'],turns)
        self.assertEqual(turns,{'SFN':0,'LAN':0})

    def test_unavailable_selected_role_uses_marked_observed_prior(self):
        s,p,l,g=fixture();pid='ap'
        s['_workloads'][pid]['outing_rows']=[]
        s['_workloads'][pid]['start'].update(pa_sum=0,outings=0,mean_pa=None,limit=None)
        s['_workloads'][pid]['relief'].update(pa_sum=0,outings=0,mean_pa=None,limit=None)
        bundle=self.m.build_game(s,p,l,g['game_id'],{});f=fields(bundle['cast'])
        self.assertEqual(f['budget_prior:start:'+pid],'true')
        samples=list(map(int,f['usage:start:'+pid].split(',')))
        self.assertLessEqual(len(samples),101);self.assertTrue(samples)
        self.assertEqual(list(map(int,f['workload:start:'+pid].split(','))),[sum(samples),len(samples)])
        self.assertIn('pooled',f['workload_basis'])

    def test_unknown_targets_remain_neutral_and_cold_counts_explicit(self):
        s,p,l,g=fixture();a=p['teams']['SFN']['players']['ar0']
        a['targets']['relief_outs']=None;a['pitcher_counts']=[0]*9;a['cold_pitcher']=True
        b=self.m.build_game(s,p,l,g['game_id'],{});f=fields(b['cast'])
        self.assertEqual(f['relief_pace:ar0'],'0');self.assertEqual(f['pitcher:ar0'],'0,0,0,0,0,0,0,0,0')
        self.assertIn(dict(role='pitcher',player_id='ar0'),b['cold_profiles'])

    def test_rotation_members_and_known_start_only_pitchers_are_not_bullpen_roles(self):
        s,p,l,g=fixture();actors=p['teams']['SFN']['players'];actors['ap']['starts']=11
        for i in range(5):
            pid='spot'+str(i);a=copy.deepcopy(actors['ap']);a.update(player_id=pid,name=pid,starts=10-i);actors[pid]=a
            if i==0:a['targets']['relief_outs']=30
        b=self.m.build_game(s,p,l,g['game_id'],{});f=fields(b['cast'])
        ids={k.split(':')[3] for k in f if k.startswith('cast:Away:')}
        self.assertIn('ap',ids)
        self.assertFalse(ids&{'spot'+str(i) for i in range(5)})

    def test_bench_candidates_have_pace_positions_and_batting_counts(self):
        s,p,l,g=fixture();base=copy.deepcopy(p['teams']['SFN']['players']['a2'])
        base.update(player_id='reserve',name='Reserve Batter',positions=[2,3])
        p['teams']['SFN']['players']['reserve']=base
        b=self.m.build_game(s,p,l,g['game_id'],{});f=fields(b['bench'])
        self.assertEqual(f['Away:reserve:positions'],'2,3');self.assertEqual(f['Away:reserve:pace'],'0')
        self.assertIn('reserve',b['role_actors']['Away']['batter'])

    def test_unknown_batting_target_is_not_unavailability_for_known_pool_actor(self):
        s,p,l,g=fixture();base=copy.deepcopy(p['teams']['SFN']['players']['a2'])
        base.update(player_id='unknown',name='Unknown Target',batter_counts=[0]*9,cold_batter=True)
        base['targets']['batter']=None;p['teams']['SFN']['players']['unknown']=base
        b=self.m.build_game(s,p,l,g['game_id'],{});f=fields(b['bench'])
        self.assertEqual(f['Away:unknown:pace'],'0')
        self.assertEqual(f['Away:unknown:counts'],'0,0,0,0,0,0,0,0,0')

    def test_v2_producer_exports_all_starter_debt_and_exact_native_workload_wire(self):
        from preview import pitcher_policy_v2
        s,p,l,g=fixture();tid='SFN';actors=p['teams'][tid]['players']
        for team_id in ['SFN','LAN']:
            p['teams'][team_id]['scheduled_games']=162;l['teams'][team_id]['scheduled_games']=162
        for i in range(6):
            pid=f'spot{i}'
            actor=copy.deepcopy(actors['ap'])
            actor.update(player_id=pid,name=pid,positions=[1],starts=i+1,
                targets=dict(batter=0,pitcher=30,start_outs=30,relief_outs=None))
            actors[pid]=actor
        mixed=copy.deepcopy(actors['ap'])
        mixed.update(player_id='mixed',name='Mixed Role',positions=[1],starts=1,
            targets=dict(batter=0,pitcher=30,start_outs=30,relief_outs=30))
        actors['mixed']=mixed
        start_only=copy.deepcopy(actors['ap'])
        start_only.update(player_id='startonly',name='Start Only',positions=[1],starts=1,
            targets=dict(batter=0,pitcher=30,start_outs=30,relief_outs=0))
        actors['startonly']=start_only

        bundle=self.m.build_game(s,p,l,g['game_id'],{},pitcher_policy=pitcher_policy_v2.POLICY)
        lineup=fields(bundle['lineup']);cast=fields(bundle['cast']);bench=fields(bundle['bench'])
        self.assertEqual(bundle['starter_decisions']['Away']['player_id'],'spot5')
        self.assertEqual(bundle['next_turns'],{})
        self.assertEqual(lineup['format'],'season-pool-lineup-v2')
        self.assertEqual(cast['format'],'season-pool-pitching-cast-v2')
        self.assertEqual(bench['format'],'season-pool-bench-v2')
        self.assertEqual(cast['club_clock:Away'],'0,162')
        pitcher_ids={key.split(':')[3] for key in cast if key.startswith('cast:')}
        relief_usage_ids={key.split(':')[1] for key in cast if key.startswith('relief_usage:')}
        self.assertEqual(pitcher_ids,relief_usage_ids)
        self.assertIn('mixed',pitcher_ids)
        self.assertNotIn('startonly',pitcher_ids)
        self.assertFalse(any(key.startswith('relief_pace:') for key in cast))
        self.assertEqual(bundle['role_actors'],season.actor_bindings(bundle))

    def test_incomplete_history_is_never_usable_policy_context(self):
        s,p,l,g=fixture();l['history_complete']=False
        with self.assertRaisesRegex(ValueError,'Incomplete'):
            self.m.build_game(s,p,l,g['game_id'],{})


if __name__=='__main__':unittest.main()
