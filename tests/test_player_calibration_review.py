"""Independent probability and game-cluster arithmetic, not a second Rust fit."""
import math
from pathlib import Path
import tempfile
import unittest
from audits import player_calibration_review as review

ROLES={'batter:b','pitcher:p'}

def text(tree=False):
    rates=[.2,.1,.05,.3,.2,.01,.2,.02] if tree else [1/9]*9
    vector=','.join(map(str,rates))
    head='format\tplayer-calibration-tree-candidate-v2\nleague_nodes\t' if tree else 'format\tplayer-calibration-candidate-v1\ncandidate\tjoint-20-20\nleague\t'
    return 'key\tvalue\n'+head+vector+'\nbatter:b\t'+vector+'\npitcher:p\t'+vector+'\n'

class CalibrationReviewTests(unittest.TestCase):
    def test_literal_tree_products_and_independent_nine_category_combination(self):
        model=review.read_effects(text(True),ROLES)
        got=review.predict(model,'b','p')
        for a,b in zip(got,[.2,.08,.036,.13001472,.03250368,.0016416,.04104,.009576,.469224]):
            self.assertAlmostEqual(a,b,places=14)
        self.assertAlmostEqual(sum(got),1,places=14)
        plain=review.read_effects(text(),ROLES)
        for p in review.predict(plain,'b','p'):
            self.assertAlmostEqual(p,1/9,places=14)
        with self.assertRaises(ValueError):review.predict(model,'missing','p')

    def test_wrong_roles_versions_duplicate_controls_and_nonpositive_rates_refuse(self):
        for bad in [text().replace('candidate-v1','candidate-v9'),text()+'batter:b\t'+','.join(['.1']*9)+'\n',text().replace('batter:b','batter:extra'),text().replace('0.1111111111111111','nan',1),text().replace('0.1111111111111111','0',1),text(True).replace('0.2','1.0',1),text()+'x\t1\n',text().replace('key\tvalue','key\tvalue\r')]:
            with self.subTest(bad=bad[:80]),self.assertRaises(ValueError):review.read_effects(bad,ROLES)
        with self.assertRaises(ValueError):review.read_effects(text(),ROLES|{'pitcher:extra'})

    def test_cluster_ratio_standard_error_is_based_on_games_not_independent_pas(self):
        # Two equal-sized games: per-game loss differences 1 and3 over10 PA each.
        mean,low,high=review.cluster_difference([(1.0,10),(3.0,10)])
        self.assertAlmostEqual(mean,.2)
        self.assertAlmostEqual((high-low)/(2*1.96),.1)
        with self.assertRaises(ValueError):review.cluster_difference([(1,10)])
        with self.assertRaises(ValueError):review.cluster_difference([(1,0),(2,10)])

    def test_closed_tree_export_keeps_literal_counts_and_refuses_missing_profiles(self):
        model=review.read_effects(text(True),ROLES)
        counts={key:[0]*9 for key in ROLES}
        result=review.encode_tree(model,counts,'a'*64,'b'*64)
        self.assertIn('counts:batter:b\t0,0,0,0,0,0,0,0,0\n',result)
        self.assertIn('reused_test_benchmark\ttrue\n',result)
        self.assertIn('format\tclosed-opponent-tree-ratings-v2\n',result)
        with self.assertRaises(ValueError):review.encode_tree(model,{**counts,'batter:missing':[0]*9},'a'*64,'b'*64)
        with self.assertRaises(ValueError):review.encode_tree(model,counts,'a'*63,'b'*64)
        with self.assertRaises(ValueError):review.encode_tree(review.read_effects(text(),ROLES),counts,'a'*64,'b'*64)

    def test_v3_candidate_selection_refuses_incomplete_matrix_before_scoring(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            (root/'tree-node-6.csv').write_text('node,batter_strength,pitcher_strength,converged,iterations,max_correction,train_qualified,validation_conditional_nll\n6,1,1,true,1,0.0,true,0.5\n')
            with self.assertRaisesRegex(ValueError,'Invalid hit-mix candidate matrix'):
                review.audit_hit_mix_selection([],{},root,[[20,20]]*8)
            header='node,batter_strength,pitcher_strength,converged,iterations,max_correction,train_qualified,validation_conditional_nll\n'
            matrix=header+''.join(f'6,{b},{p},true,1,0.0,true,0.5\n' for b in [1,5,20,200] for p in [1,5,20,200])
            (root/'tree-node-6.csv').write_text(matrix.replace('6,1,1,','6,1.9,1,',1))
            with self.assertRaisesRegex(ValueError,'Invalid hit-mix candidate matrix'):
                review.audit_hit_mix_selection([],{},root,[[20,20]]*8)

    def test_v3_candidate_has_distinct_version_and_cannot_export_as_v2(self):
        source=text(True).replace('tree-candidate-v2','tree-candidate-v3')
        model=review.read_effects(source,ROLES)
        self.assertEqual(model['version'],'player-calibration-tree-candidate-v3')
        self.assertEqual(review.predict(model,'b','p'),review.predict(review.read_effects(text(True),ROLES),'b','p'))
        with self.assertRaises(ValueError):review.encode_tree(model,{key:[0]*9 for key in ROLES},'a'*64,'b'*64)

    def test_v3_bounds_are_separate_from_v2_and_do_not_round_failures(self):
        baseline={(role,k):dict(mae=.02,maximum=.05) for role in ['batter','pitcher'] for k in ['single','double']}
        good={key:dict(mae=.01,maximum=.03) for key in baseline}
        self.assertTrue(review.hit_mix_qualifies(good,baseline))
        for key in good:
            bad={k:dict(v) for k,v in good.items()};bad[key]['maximum']=.0300000001
            self.assertFalse(review.hit_mix_qualifies(bad,baseline))
            old={k:dict(v) for k,v in baseline.items()};old[key]['mae']=.009
            self.assertFalse(review.hit_mix_qualifies(good,old))
        self.assertTrue(review.hit_mix_guardrails([.001,0,.002]))
        self.assertFalse(review.hit_mix_guardrails([.0010000001,0,.002]))
        self.assertFalse(review.hit_mix_guardrails([math.nan,0,.002]))
        self.assertFalse(review.hit_mix_guardrails([.001,.002,0]))

    def test_statistical_rejection_does_not_get_rounded_into_a_pass(self):
        self.assertFalse(review.guardrails([-.013,-.019,-.007],[.010389,.007,.013]))
        self.assertTrue(review.guardrails([-.013,-.019,-.007],[.009999,.007,.013]))
        self.assertFalse(review.guardrails([-.013,-.019,0],[.001,-.002,.003]))
        self.assertFalse(review.guardrails([math.nan,0,0],[.001,0,.002]))
