"""Season-pool policy contracts; no native outcomes or daily availability data."""
import copy
from preview import season_manager
import unittest
from test_usage_ledger import targets, game
from preview.usage_ledger import Ledger


def source_and_targets():
    t=targets()
    data=dict(game=[{k:g[k] for k in ['game_id','date','away_team_id','home_team_id']} for g in t['games']],
        person=[dict(player_id=p,first_name=p,last_name='Player') for p in ['a','h']],
        appearance=[dict(game_id=t['games'][0]['game_id'],side=side,player_id=p,fielding_position=pos)
            for side,p in [('Away','a'),('Home','h')] for pos in [1,2]])
    from preview.player_ratings import KEYS
    report=dict(roles={r:dict(profiles=[dict(player_id=p,included_pa=1,
        counts=dict(zip(KEYS,[1,0,0,0,0,0,0,0,0]))) for p in ['a','h']])
        for r in ['batter','pitcher']})
    return dict(data=data,report=report),t


class SeasonManagerTests(unittest.TestCase):
    def setUp(self):
        self.m=season_manager
        self.source,self.targets=source_and_targets()
        self.pool=self.m.build_pool(self.source,self.targets)
        self.snap=Ledger([game()]).snapshot()

    def policy(self):return self.m.Policy(self.pool,self.snap,'2019-04-01')

    def add(self,pid,positions=(1,2),batter=100,pitcher=30):
        actor=copy.deepcopy(self.pool['teams']['SFN']['players']['a'])
        actor.update(player_id=pid,positions=list(positions),targets=dict(batter=batter,pitcher=pitcher,
            start_outs=pitcher,relief_outs=0),starts=5)
        self.pool['teams']['SFN']['players'][pid]=actor

    def used(self,pid,role,n,workload=None,*,starter=False):
        stats=dict(usage=n)
        if role=='pitcher':stats.update(start_outs=n if starter else 0,relief_outs=0 if starter else n)
        self.snap['players'][role+':'+pid]=dict(teams={'SFN':stats},workload=workload or [])

    def test_pool_reconciles_club_roles_and_preserves_sources(self):
        before=copy.deepcopy((self.source,self.targets))
        p=self.m.build_pool(self.source,self.targets)
        self.assertEqual(p['teams']['SFN']['players']['a']['targets'],dict(batter=29,pitcher=27,start_outs=27,relief_outs=0))
        self.assertEqual(p['teams']['SFN']['players']['a']['positions'],[1,2])
        self.assertEqual(p['teams']['SFN']['scheduled_games'],1)
        self.assertEqual((self.source,self.targets),before)
        self.assertIn('season',p['availability_basis'])
        self.assertNotIn('first_date',p['teams']['SFN']['players']['a'])

    def test_source_target_schedule_mismatch_and_unsupported_identity_rejected(self):
        self.source['data']['game'][0]['date']='2019-04-02'
        with self.assertRaises(ValueError):self.m.build_pool(self.source,self.targets)
        self.source,self.targets=source_and_targets()
        self.source['data']['appearance'][0]['player_id']='invented'
        with self.assertRaises(ValueError):self.m.build_pool(self.source,self.targets)

    def test_unknown_targets_are_not_zero_and_absent_profiles_are_labeled_cold(self):
        r=self.targets['records'][0];r['usage']=None;r['unknown_rows']=1
        self.targets['totals']['pa']=None
        self.source['report']['roles']['batter']['profiles']=[]
        p=self.m.build_pool(self.source,self.targets)['teams']['SFN']['players']['a']
        self.assertIsNone(p['targets']['batter']);self.assertEqual(p['batter_counts'],[0]*9)
        self.assertTrue(p['cold_batter'])

    def test_partial_unknown_pitcher_split_preserves_known_other_role(self):
        r=next(r for r in self.targets['records'] if r['team_id']=='SFN' and r['role']=='pitcher')
        r.update(usage=None,unknown_rows=1);self.targets['totals']['outs']=None
        p=self.m.build_pool(self.source,self.targets)['teams']['SFN']['players']['a']['targets']
        self.assertIsNone(p['pitcher']);self.assertIsNone(p['start_outs']);self.assertEqual(p['relief_outs'],0)

    def test_club_targets_remain_separate_for_multi_team_players(self):
        self.source['data']['appearance'].append(dict(game_id=game()['game_id'],side='Home',player_id='a',fielding_position=2))
        next(r for r in self.targets['records'] if r['team_id']=='LAN' and r['role']=='batter')['player_id']='a'
        p=self.m.build_pool(self.source,self.targets)
        self.assertEqual(p['teams']['SFN']['players']['a']['targets']['batter'],29)
        self.assertEqual(p['teams']['LAN']['players']['a']['targets']['batter'],27)
        self.assertIsNone(p['teams']['LAN']['players']['a']['targets']['pitcher'])
        self.assertIsNone(p['teams']['LAN']['players']['h']['targets']['batter'])

    def test_unknown_positions_never_become_legal_defense(self):
        for a in self.source['data']['appearance']:a['fielding_position']=None
        self.assertEqual(self.m.build_pool(self.source,self.targets)['teams']['SFN']['players']['a']['positions'],[])

    def test_reordered_source_records_make_identical_pool(self):
        self.source['data']['appearance'].reverse();self.targets['records'].reverse()
        self.assertEqual(self.m.build_pool(self.source,self.targets),self.pool)

    def test_pace_deadbands_unknown_zero_and_early_season(self):
        self.assertEqual(self.m.pace(12,600,0,162,'batter')['preference'],0)
        self.assertLess(self.m.pace(13,600,0,162,'batter')['preference'],0)
        self.assertGreater(self.m.pace(100,600,81,162,'batter')['preference'],0)
        self.assertEqual(self.m.pace(100,None,81,162,'batter')['preference'],0)
        self.assertTrue(self.m.pace(100,None,81,162,'batter')['unknown'])
        self.assertFalse(self.m.pace(0,0,81,162,'batter')['unknown'])
        self.assertEqual(self.m.pace(12,0,162,162,'batter')['preference'],0)
        for args in [(0,0,1,0,'batter'),(-1,0,0,1,'batter'),(0,0,2,1,'batter'),(0,0,0,1,'bad'),(0,1.5,0,1,'batter')]:
            with self.assertRaises(ValueError):self.m.pace(*args)

    def test_pace_direction_and_bounds_across_roles_and_progress(self):
        for role in ['batter','starter','relief']:
            for total in [1,161,162]:
                for completed in [0,total//2,total]:
                    for target in [0,1,30,600,None]:
                        for used in [0,1,13,40,300,1000]:
                            p=self.m.pace(used,target,completed,total,role)
                            self.assertLessEqual(abs(p['preference']),1)
                            if target is not None:
                                delta=target*completed/total-used
                                self.assertEqual(p['preference']==0,abs(delta)<=p['allowance'])
                                if p['preference']:self.assertGreater(p['preference']*delta,0)

    def test_hitter_inside_band_keeps_incumbent_and_no_mutation(self):
        self.add('b');before=copy.deepcopy((self.pool,self.snap))
        self.assertEqual(self.policy().hitter('SFN','a',2),'a')
        self.assertEqual((self.pool,self.snap),before)

    def test_hitter_overuse_favors_compatible_alternative_not_occupied(self):
        self.add('b');self.add('c',(3,));self.used('a','batter',40)
        self.assertEqual(self.policy().hitter('SFN','a',2),'b')
        self.assertEqual(self.policy().hitter('SFN','a',2,occupied={'b'}),'a')
        with self.assertRaises(ValueError):self.policy().hitter('SFN','a',3)

    def test_hitter_underuse_and_stable_ties(self):
        self.add('b',batter=100);self.add('c',batter=100)
        self.snap['teams']['SFN']['completed_games']=1
        self.used('a','batter',29)
        self.assertEqual(self.policy().hitter('SFN','a',2),'b')

    def test_rest_is_global_across_clubs_and_before_pace(self):
        self.add('b');self.used('a','pitcher',0,[dict(date='2019-03-31',usage=7,started=False,team_id='OTHER')])
        self.assertFalse(self.policy().rested('a',starter=False))
        self.assertFalse(self.policy().rested('a',starter=True))
        self.assertEqual(self.policy().relievers('SFN',{'a':0,'b':0}),['b'])

    def test_starter_requires_four_full_rest_days_and_intervening_relief(self):
        for day,expected in [('2019-03-27',True),('2019-03-28',False)]:
            self.used('a','pitcher',27,[dict(date=day,usage=27,started=True)])
            self.assertEqual(self.policy().rested('a',starter=True),expected)
        self.used('a','pitcher',30,[dict(date='2019-03-27',usage=27,started=True),dict(date='2019-03-31',usage=7,started=False)])
        self.assertFalse(self.policy().rested('a',starter=True))

    def test_same_day_and_fourth_consecutive_day_are_not_available(self):
        for days in [['2019-04-01'],['2019-03-29','2019-03-30','2019-03-31']]:
            self.used('a','pitcher',3,[dict(date=d,usage=1,started=False) for d in days])
            self.assertFalse(self.policy().rested('a',starter=False))
        self.used('a','pitcher',6,[dict(date='2019-03-31',usage=6,started=False)])
        self.assertTrue(self.policy().rested('a',starter=False))

    def test_rotation_normal_skip_return_and_exhaustion(self):
        self.add('b');self.add('c')
        self.assertEqual(self.policy().starter('SFN',['a','b','c'],0),dict(player_id='a',next_turn=1,reason='normal_rotation',budget_scale=1.0))
        self.used('a','pitcher',50,starter=True)
        chosen=self.policy().starter('SFN',['a','b','c'],0)
        self.assertEqual((chosen['player_id'],chosen['next_turn'],chosen['reason']),('b',1,'substantial_overuse'))
        self.assertEqual(self.policy().starter('SFN',['a','b','c'],2)['next_turn'],0)
        self.assertEqual(self.policy().starter('SFN',['a'],0)['budget_scale'],0.8)
        self.used('a','pitcher',27,[dict(date='2019-03-31',usage=27,started=True)])
        with self.assertRaises(ValueError):self.policy().starter('SFN',['a'],0)

    def test_relief_suitability_precedes_bounded_usage_and_stable_ids(self):
        self.add('b');self.add('c');self.used('a','pitcher',100)
        self.assertEqual(self.policy().relievers('SFN',{'a':0,'c':1,'b':1}),['a','b','c'])
        self.assertEqual(self.policy().relievers('SFN',{'a':0,'c':0,'b':0}),['b','c','a'])
        self.assertEqual(self.policy().relievers('SFN',{'a':0},occupied={'a'}),[])
        with self.assertRaises(ValueError):self.policy().relievers('SFN',{'invented':0})

    def test_mixed_pitching_roles_have_separate_pace_clocks(self):
        self.add('b')
        self.pool['teams']['SFN']['players']['a']['targets'].update(pitcher=120,start_outs=90,relief_outs=30)
        self.pool['teams']['SFN']['players']['b']['targets'].update(pitcher=120,start_outs=90,relief_outs=30)
        self.snap['teams']['SFN']['completed_games']=1
        self.snap['players']['pitcher:a']=dict(teams={'SFN':dict(usage=120,start_outs=0,relief_outs=120)},workload=[])
        self.snap['players']['pitcher:b']=dict(teams={'SFN':dict(usage=120,start_outs=90,relief_outs=30)},workload=[])
        self.assertEqual(self.policy().relievers('SFN',{'a':0,'b':0}),['b','a'])
        self.assertEqual(self.policy().starter('SFN',['a','b'],0)['player_id'],'a')

    def test_incomplete_history_clock_mismatch_future_work_and_bad_rotation_rejected(self):
        self.snap['history_complete']=False
        with self.assertRaises(ValueError):self.policy()
        self.snap['history_complete']=True;self.snap['teams']['SFN']['scheduled_games']=162
        with self.assertRaises(ValueError):self.policy()
        self.snap['teams']['SFN']['scheduled_games']=1
        self.used('a','pitcher',1,[dict(date='2019-04-02',usage=1,started=False)])
        with self.assertRaises(ValueError):self.policy()
        self.snap['players']={}
        for rotation,turn in [([],0),(['a','a'],0),(['a'],-1),(['a'],1),(['invented'],0)]:
            with self.assertRaises(ValueError):self.policy().starter('SFN',rotation,turn)


if __name__=='__main__':unittest.main()
