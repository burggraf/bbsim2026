"""Preregistered pure v2 arithmetic/choices; no native games or usage fitting."""
import copy
from fractions import Fraction
import importlib
import importlib.util
import itertools
import unittest
from preview import season_manager


class PitcherPolicyV2Tests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(importlib.util.find_spec('preview.pitcher_policy_v2'),
            'Missing preregistered pure v2 policy component')
        self.m=importlib.import_module('preview.pitcher_policy_v2')
        # Synthetic partial actor populations, using the existing pool/ledger shapes.
        self.pool=dict(format=season_manager.POLICY,teams={tid:dict(scheduled_games=162,players={})
            for tid in ['AAA','BBB']})
        self.snapshot=dict(history_complete=True,players={},teams={tid:dict(scheduled_games=162,
            completed_games=80,history_complete=True) for tid in self.pool['teams']})

    def add(self,pid,*,team='AAA',starts=0,start_outs=120,relief_outs=120,positions=(1,)):
        self.pool['teams'][team]['players'][pid]=dict(player_id=pid,positions=list(positions),starts=starts,
            targets=dict(batter=100,pitcher=None,start_outs=start_outs,relief_outs=relief_outs))

    def used(self,pid,*,team='AAA',starts=0,start_outs=0,relief_outs=0,workload=()):
        p=self.snapshot['players'].setdefault('pitcher:'+pid,dict(teams={},workload=[]))
        p['teams'][team]=dict(starts=starts,start_outs=start_outs,relief_outs=relief_outs,
            usage=start_outs+relief_outs)
        p['workload'].extend(copy.deepcopy(workload))

    def policy(self):return self.m.Policy(self.pool,self.snapshot,'2019-06-01')

    def rank(self,used=30,target=120,c=80,n=162,*,pa_sum=60,outings=10,inning=5,gap=0):
        return self.m.relief_rank(used,target,c,n,pa_sum=pa_sum,outings=outings,inning=inning,score_gap=gap)

    def test_v2_is_separate_from_v1_context_and_versions(self):
        self.assertEqual(self.m.POLICY,'season-pool-pitcher-usage-v2')
        self.assertEqual(season_manager.POLICY,'season-pool-soft-usage-v1')
        self.assertIsNot(self.m.Policy,season_manager.Policy)
        self.assertIs(self.m.Policy.rested,season_manager.Policy.rested)
        self.assertIs(self.m.Policy.hitter,season_manager.Policy.hitter)

    def test_rank_six_competes_with_rested_principals(self):
        for i,s in enumerate([30,30,30,29,29]):
            pid='principal'+str(i);self.add(pid,starts=s);self.used(pid,starts=16)
        self.add('extra',starts=14)
        choice=self.policy().starter('AAA')
        self.assertEqual(choice['player_id'],'extra')
        self.assertEqual(choice['budget_scale'],100)
        self.assertNotIn('next_turn',choice)
        self.assertEqual(len(choice['candidates']),6)
        extra=next(p for p in choice['candidates'] if p['player_id']=='extra')
        self.assertEqual((extra['starts_target'],extra['starts_before'],extra['debt'],extra['excluded']),
            (14,0,Fraction(7),None))
        # The default v1 API remains rotation-based, not silently replaced.
        old=season_manager.Policy(self.pool,self.snapshot,'2019-06-01')
        self.assertEqual(old.starter('AAA',['principal0'],0)['player_id'],'principal0')

    def test_low_start_principal_does_not_swallow_supported_extra_opportunities(self):
        self.add('principal',starts=9);self.add('extra',starts=14)
        self.used('principal',starts=20);self.used('extra',starts=6)
        self.assertEqual(self.policy().starter('AAA')['player_id'],'extra')

    def test_start_debt_ties_use_global_last_appearance_then_stable_id(self):
        for pid in ['z','b','a']:self.add(pid,starts=10)
        self.used('a',workload=[dict(date='2019-05-20',usage=1,started=False,team_id='BBB')])
        self.used('b',workload=[dict(date='2019-05-19',usage=1,started=False,team_id='BBB')])
        self.assertEqual(self.policy().starter('AAA')['player_id'],'z')
        self.used('z',workload=[dict(date='2019-05-20',usage=1,started=False,team_id='BBB')])
        self.assertEqual(self.policy().starter('AAA')['player_id'],'b')
        self.snapshot['players']={}
        self.assertEqual(self.policy().starter('AAA')['player_id'],'a')

    def test_rest_and_occupied_filter_before_debt_with_explicit_reasons(self):
        self.add('due',starts=32);self.add('backup',starts=1)
        self.used('due',workload=[dict(date='2019-05-31',usage=7,started=False,team_id='BBB')])
        choice=self.policy().starter('AAA')
        self.assertEqual(choice['player_id'],'backup')
        self.assertEqual(next(r for r in choice['candidates'] if r['player_id']=='due')['excluded'],'rest')
        self.snapshot['players']={}
        choice=self.policy().starter('AAA',occupied={'due'})
        self.assertEqual(choice['player_id'],'backup')
        self.assertEqual(next(r for r in choice['candidates'] if r['player_id']=='due')['excluded'],'occupied')
        with self.assertRaisesRegex(ValueError,'No rested supported starter'):
            self.policy().starter('AAA',occupied={'due','backup'})

    def test_global_rest_off_days_doubleheaders_and_consecutive_days(self):
        self.add('a',starts=32)
        cases=[([dict(date='2019-05-27',usage=27,started=True)],True),
               ([dict(date='2019-05-28',usage=27,started=True)],False),
               ([dict(date='2019-05-31',usage=6,started=False)],True),
               ([dict(date='2019-05-31',usage=7,started=False)],False),
               ([dict(date='2019-06-01',usage=0,started=False)],False),
               ([dict(date=d,usage=1,started=False) for d in ['2019-05-29','2019-05-30','2019-05-31']],False)]
        for history,available in cases:
            with self.subTest(history=history):
                self.snapshot['players']={};self.used('a',team='BBB',workload=history)
                if available:self.assertEqual(self.policy().starter('AAA')['player_id'],'a')
                else:
                    with self.assertRaises(ValueError):self.policy().starter('AAA')

    def test_start_support_and_counts_are_not_inferred_from_targets(self):
        self.add('no_starts',starts=0,start_outs=1000)
        self.add('no_position',starts=32,positions=(2,))
        self.add('supported',starts=1)
        choice=self.policy().starter('AAA')
        self.assertEqual(choice['player_id'],'supported')
        self.assertEqual([r['player_id'] for r in choice['candidates']],['supported'])

    def test_joint_scarce_club_first_includes_existing_occupied_filter(self):
        for tid,pid,s in [('AAA','shared',32),('AAA','backup',1),('BBB','shared',32),('BBB','busy',1)]:
            self.add(pid,team=tid,starts=s)
        original={'busy'}
        choices=self.policy().starters('AAA','BBB',occupied=original)
        self.assertEqual({s:p['player_id'] for s,p in choices.items()},{'Away':'backup','Home':'shared'})
        self.assertEqual(original,{'busy'})
        self.pool['teams']['BBB']['players'].pop('busy')
        self.pool['teams']['AAA']['players'].pop('backup')
        before=copy.deepcopy((self.pool,self.snapshot))
        with self.assertRaises(ValueError):self.policy().starters('AAA','BBB')
        self.assertEqual((self.pool,self.snapshot),before)
        with self.assertRaises(ValueError):self.policy().starters('AAA','AAA')

    def test_occupied_identity_iterables_are_snapshotted_once(self):
        for pid,starts in [('a',1),('b',32),('c',31)]:self.add(pid,starts=starts)
        self.assertEqual(self.policy().starter('AAA',occupied=iter(['b','c']))['player_id'],'a')
        self.assertEqual(self.policy().relievers('AAA',{pid:(60,10) for pid in ['a','b','c']},
            inning=5,score_gap=0,occupied=iter(['b','c'])),['a'])

    def test_joint_ties_keep_away_first(self):
        for tid in ['AAA','BBB']:
            for pid in ['a','b']:self.add(pid,team=tid,starts=10)
        picks=self.policy().starters('AAA','BBB')
        self.assertEqual((picks['Away']['player_id'],picks['Home']['player_id']),('a','b'))

    def test_start_budget_uses_lookahead_separate_role_and_strict_both_thresholds(self):
        self.add('a',starts=32,start_outs=100)
        for used,scale in [(86,100),(87,80)]:
            self.used('a',starts=16,start_outs=used,relief_outs=1000)
            self.assertEqual(self.policy().starter('AAA')['budget_scale'],scale)
        self.pool['teams']['AAA']['players']['a']['targets']['start_outs']=200
        for used,scale in [(140,100),(141,80)]:
            self.used('a',starts=16,start_outs=used)
            self.assertEqual(self.policy().starter('AAA')['budget_scale'],scale)
        self.pool['teams']['AAA']['players']['a']['targets']['start_outs']=None
        self.assertEqual(self.policy().starter('AAA')['budget_scale'],100)
        self.pool['teams']['AAA']['players']['a']['targets']['start_outs']=0
        self.used('a',starts=16,start_outs=36)
        self.assertEqual(self.policy().starter('AAA')['budget_scale'],100)
        self.used('a',starts=16,start_outs=37)
        self.assertEqual(self.policy().starter('AAA')['budget_scale'],80)

    def test_club_start_counts_and_role_outs_remain_separate(self):
        self.add('a',starts=32);self.add('b',starts=32)
        self.used('a',team='BBB',starts=32,start_outs=500,relief_outs=500)
        self.used('a',starts=0,relief_outs=500)
        self.used('b',starts=16,start_outs=0)
        self.assertEqual(self.policy().starter('AAA')['player_id'],'a')
        self.assertEqual(self.policy().starter('AAA')['budget_scale'],100)

    def test_exact_relief_rank_and_explicit_unknown_zero(self):
        self.assertEqual(self.rank(),dict(usage=30,expected=Fraction(60),allowance=Fraction(12),
            debt=Fraction(5,2),preference=Fraction(1),unknown=False,long=True,wants_long=True,score=Fraction(5,4)))
        unknown=self.rank(100,None)
        self.assertIsNone(unknown['expected']);self.assertIsNone(unknown['allowance'])
        self.assertEqual((unknown['debt'],unknown['preference'],unknown['score'],unknown['unknown']),
            (Fraction(0),Fraction(0),Fraction(1,4),True))
        zero=self.rank(6,0)
        self.assertEqual((zero['expected'],zero['allowance'],zero['debt'],zero['unknown']),
            (Fraction(0),Fraction(6),Fraction(-1),False))

    def test_small_positive_debt_is_not_rounded_to_neutral_or_id(self):
        for tid in self.snapshot['teams']:self.snapshot['teams'][tid]['completed_games']=0
        self.add('z',relief_outs=60);self.add('a',relief_outs=59)
        self.assertEqual(self.rank(0,60,0)['debt'],Fraction(5,81))
        self.assertEqual(self.policy().relievers('AAA',{'a':(60,10),'z':(60,10)},inning=5,score_gap=0),['z','a'])

    def test_relief_debt_can_beat_workload_class_without_a_hard_cap(self):
        self.add('long');self.add('short')
        self.used('long',relief_outs=120);self.used('short',relief_outs=60)
        profiles={'long':(60,10),'short':(59,10)}
        self.assertEqual(self.policy().relievers('AAA',profiles,inning=5,score_gap=0),['short','long'])
        self.assertEqual(self.policy().relievers('AAA',{'long':(60,10)},inning=5,score_gap=0),['long'])
        old=season_manager.Policy(self.pool,self.snapshot,'2019-06-01')
        self.assertEqual(old.relievers('AAA',{'long':0,'short':1}),['long','short'])

    def test_relief_class_and_situation_boundaries(self):
        for pa,n in [(0,1),(59,10),(60,10),(61,10)]:
            for inning,gap in [(5,0),(6,3),(6,4),(6,-4),(9,0)]:
                r=self.rank(0,0,pa_sum=pa,outings=n,inning=inning,gap=gap)
                is_long=pa>=6*n;wants_long=inning<=5 or abs(gap)>=4
                self.assertEqual((r['long'],r['wants_long']),(is_long,wants_long))
                self.assertEqual(r['score'],Fraction(1,4) if is_long==wants_long else Fraction(0))

    def test_saturated_relief_scores_use_unclipped_debt_then_stable_id(self):
        self.add('a');self.add('z');profiles={'a':(60,10),'z':(60,10)}
        for a,z in [(30,0),(120,84)]:
            self.used('a',relief_outs=a);self.used('z',relief_outs=z)
            self.assertEqual(self.policy().relievers('AAA',profiles,inning=5,score_gap=0),['z','a'])
        self.used('a',relief_outs=0);self.used('z',relief_outs=0)
        self.assertEqual(self.policy().relievers('AAA',profiles,inning=5,score_gap=0),['a','z'])

    def test_mixed_role_relief_is_not_reserved_out_and_rest_still_wins(self):
        self.add('principal',starts=32);self.add('other')
        profiles={'principal':(60,10),'other':(60,10)}
        self.used('other',relief_outs=60)
        self.assertEqual(self.policy().relievers('AAA',profiles,inning=5,score_gap=0),['principal','other'])
        self.used('principal',workload=[dict(date='2019-05-31',usage=1,started=True)])
        self.assertEqual(self.policy().relievers('AAA',profiles,inning=5,score_gap=0),['other'])
        self.assertEqual(self.policy().relievers('AAA',profiles,inning=5,score_gap=0,occupied={'other'}),[])

    def test_relief_support_and_position_are_caller_declared_not_invented(self):
        self.add('a',starts=32,relief_outs=0);self.add('b',positions=(2,))
        for profiles in [{'a':(0,0)},{'b':(60,10)},{'missing':(60,10)},[('a',(60,10)),('a',(60,10))],{'a':None}]:
            with self.subTest(profiles=profiles),self.assertRaises(ValueError):
                self.policy().relievers('AAA',profiles,inning=5,score_gap=0)
        self.assertEqual(self.policy().relievers('AAA',{},inning=5,score_gap=0),[])

    def test_first_final_clock_and_finished_other_club(self):
        self.add('a',starts=32)
        self.snapshot['teams']['AAA']['completed_games']=0
        self.assertEqual(self.policy().starter('AAA')['candidates'][0]['debt'],Fraction(16,81))
        self.snapshot['teams']['AAA']['completed_games']=161
        self.snapshot['teams']['BBB']['completed_games']=162
        self.used('a',starts=31)
        self.assertEqual(self.policy().starter('AAA')['candidates'][0]['debt'],1)
        self.snapshot['teams']['AAA']['completed_games']=162
        with self.assertRaises(ValueError):self.policy().starter('AAA')
        with self.assertRaises(ValueError):self.policy().relievers('AAA',{},inning=5,score_gap=0)

    def test_strict_counts_clocks_and_situation_values(self):
        for used,target,c,n in [(True,0,0,162),(0,False,0,162),(-1,0,0,162),(0,1.0,0,162),
                (0,0,False,162),(0,0,0,True),(0,0,0,0),(0,0,162,162),(0,0,0,163),
                (2**32,0,0,162),(0,2**32,0,162)]:
            with self.subTest(args=(used,target,c,n)),self.assertRaises(ValueError):self.rank(used,target,c,n)
        for kw in [dict(pa_sum=True),dict(pa_sum=-1),dict(outings=0),dict(outings=1.0),
                dict(outings=2**32),dict(inning=0),dict(inning=True),dict(gap=True),dict(gap=1.0),dict(gap=2**32)]:
            with self.subTest(kw=kw),self.assertRaises(ValueError):self.rank(**kw)
        for starts in [True,-1,1.0,163]:
            self.add('a',starts=starts)
            with self.assertRaises(ValueError):self.policy().starter('AAA')
        self.add('a',starts=32)
        for count in [True,-1,1.0,81]:
            self.used('a',starts=count)
            with self.assertRaises(ValueError):self.policy().starter('AAA')

    def test_empty_profiles_do_not_bypass_situation_or_club_validation(self):
        for inning,gap in [(0,0),(True,0),(1,True),(1,1.0),(1,2**32)]:
            with self.subTest(inning=inning,gap=gap),self.assertRaises(ValueError):
                self.policy().relievers('AAA',{},inning=inning,score_gap=gap)
        self.snapshot['teams']['CCC']=copy.deepcopy(self.snapshot['teams']['AAA'])
        with self.assertRaises(ValueError):self.policy().relievers('CCC',{},inning=5,score_gap=0)
        with self.assertRaises(ValueError):self.policy().starter('CCC')

    def test_starter_exact_reference_matrix_and_maximum_role_counts(self):
        for n in [161,162]:
            self.pool['teams']['AAA']['scheduled_games']=n
            self.snapshot['teams']['AAA']['scheduled_games']=n
            for c in [0,80,n-1]:
                self.snapshot['teams']['AAA']['completed_games']=c
                for s,a in itertools.product([1,9,14,32],sorted({0,min(1,c),c})):
                    self.add('a',starts=s,start_outs=2**32-1)
                    self.add('z',starts=14,start_outs=None)
                    self.used('a',starts=a,start_outs=2**32-1)
                    rows=self.policy().starter('AAA')['candidates']
                    self.assertEqual(rows[0]['debt'],Fraction(s*(c+1),n)-a)
                    debts={'a':Fraction(s*(c+1),n)-a,'z':Fraction(14*(c+1),n)}
                    expected=min(debts,key=lambda pid:(-debts[pid],pid))
                    self.assertEqual(self.policy().starter('AAA')['player_id'],expected)
                    e=Fraction((2**32-1)*(c+1),n);excess=2**32-1-e
                    self.assertEqual(rows[0]['budget_scale'],80 if excess>36 and excess>e*Fraction(2,5) else 100)
        for key in ['start_outs','relief_outs']:
            for bad in [True,-1,1.0,2**32]:
                self.add('a',starts=32)
                self.pool['teams']['AAA']['players']['a']['targets'][key]=bad
                with self.assertRaises(ValueError):
                    if key=='start_outs':self.policy().starter('AAA')
                    else:self.policy().relievers('AAA',{'a':(60,10)},inning=5,score_gap=0)

    def test_unknown_relief_target_is_not_treated_as_known_zero(self):
        self.add('unknown',relief_outs=None);self.add('zero',relief_outs=0)
        for pid in ['unknown','zero']:self.used(pid,relief_outs=6)
        self.assertEqual(self.policy().relievers('AAA',{'unknown':(60,10),'zero':(60,10)},
            inning=5,score_gap=0),['unknown','zero'])

    def test_refused_mismatched_and_future_context_still_rejected(self):
        self.snapshot['history_complete']=False
        with self.assertRaises(ValueError):self.policy()
        self.snapshot['history_complete']=True;self.snapshot['teams']['AAA']['scheduled_games']=161
        with self.assertRaises(ValueError):self.policy()
        self.snapshot['teams']['AAA']['scheduled_games']=162
        self.used('a',workload=[dict(date='2019-06-02',usage=0,started=False)])
        with self.assertRaises(ValueError):self.policy()

    def test_permutations_and_calls_leave_inputs_and_context_unchanged(self):
        for pid in ['c','b','a']:self.add(pid,starts=10)
        original=copy.deepcopy((self.pool,self.snapshot));p=self.policy()
        context=copy.deepcopy((p.pool,p.snapshot))
        expected=p.starter('AAA');profiles={pid:(60,10) for pid in ['a','b','c']}
        for order in itertools.permutations(['a','b','c']):
            self.pool['teams']['AAA']['players']={pid:original[0]['teams']['AAA']['players'][pid] for pid in order}
            self.assertEqual(self.policy().starter('AAA'),expected)
            self.assertEqual(self.policy().relievers('AAA',{pid:profiles[pid] for pid in order},inning=5,score_gap=0),['a','b','c'])
        self.assertEqual((self.pool,self.snapshot),original)
        self.assertEqual((p.pool,p.snapshot),context)
        expected['candidates'][0]['debt']=999
        self.assertNotEqual(p.starter('AAA'),expected)

    def test_relief_exact_reference_property_matrix(self):
        for n in [1,161,162]:
            for c in sorted({0,n//2,n-1}):
                for target in [0,1,60,120,2**32-1,None]:
                    for used in [0,1,6,30,2**32-1]:
                        for pa,outings in [(0,1),(6,1),(59,10),(60,10)]:
                            for inning,gap in [(5,0),(6,3),(6,4),(6,-4)]:
                                r=self.rank(used,target,c,n,pa_sum=pa,outings=outings,inning=inning,gap=gap)
                                expected=None if target is None else Fraction(target*(c+1),n)
                                allowance=None if expected is None else max(Fraction(6),expected/5)
                                debt=Fraction(0) if expected is None else (expected-used)/allowance
                                q=max(Fraction(-1),min(Fraction(1),debt))
                                bonus=Fraction(1,4) if (pa>=6*outings)==(inning<=5 or abs(gap)>=4) else Fraction(0)
                                self.assertEqual((r['expected'],r['allowance'],r['debt'],r['preference'],r['score']),
                                    (expected,allowance,debt,q,q+bonus),
                                    dict(n=n,c=c,target=target,used=used,pa_sum=pa,outings=outings,inning=inning,gap=gap))
                                self.assertTrue(all(isinstance(r[k],Fraction) for k in ['debt','preference','score']))


if __name__=='__main__':unittest.main()
