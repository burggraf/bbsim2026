"""Synthetic exact joint support and proper-score controls, not model validation."""
from contextlib import ExitStack, redirect_stdout
import copy
import hashlib
import io
from fractions import Fraction
import itertools
import json
import math
from pathlib import Path
import unittest
from unittest.mock import patch
from audits import hit_advancement_comparison as compare
from audits import hit_advancement as hit
from tests.test_hit_advancement import hit_pair
from tests import test_hit_advancement as hit_tests

MODELS=('uniform-legal-v1','pooled-mask-jeffreys-v1','base-out-jeffreys-v1')


def synthetic_cell(split,outs,targets,count=1,value=1,mask=1):
    roles=[role for i,role in enumerate(('First','Second','Third')) if mask & (1<<i)]
    labels=('First','Second','Third','Home')
    endpoints={role:labels[target-1] for role,target in zip(roles,targets[1:])}
    row,context=hit_pair(value=value,mask=mask,outs=outs,endpoints=endpoints)
    result=hit.inventory([row],[context],['GAME1'])
    cell=copy.deepcopy(result['joint_vectors'][0])
    assert cell['qualified']
    cell.update(split=split,events=count)
    return cell


def synthetic_report(*cells):
    # Artificial aggregate opportunities; actual publication must first regenerate
    # and byte-match the complete accepted source audit, never this tiny fixture.
    totals=dict(candidate_events=0,qualified_events=0,unresolved_events=0)
    splits={split:dict(totals) for split in ('training','heldout','excluded')}
    for cell in cells:
        key='qualified_events' if cell['qualified'] else 'unresolved_events'
        for counter in (totals,splits[cell['split']]):
            counter['candidate_events']+=cell['events'];counter[key]+=cell['events']
    return dict(format=hit.FORMAT,totals=totals,split_totals=splits,joint_vectors=copy.deepcopy(list(cells)))


def independent_support(value, mask):
    origins=[i+1 for i in range(3) if mask & (1<<i)]
    vectors=[]
    for destinations in itertools.product(*(range(origin,5) for origin in origins)):
        vector=(value,*destinations)
        if all(left<right or left==right==4 for left,right in itertools.combinations(vector,2)):
            vectors.append(vector)
    return tuple(sorted(vectors))


class SupportTests(unittest.TestCase):
    def test_full_legal_support_matches_independent_pairwise_enumeration(self):
        supports=[]
        for value,mask in itertools.product((1,2),range(8)):
            with self.subTest(value=value,mask=mask):
                actual=compare.legal_vectors(value,mask)
                expected=independent_support(value,mask)
                self.assertEqual(actual,expected)
                self.assertTrue(actual)
                self.assertEqual(actual,tuple(sorted(set(actual))))
                supports.append(actual)
        self.assertEqual(sum(map(len,supports)),40)
        self.assertEqual(compare.legal_vectors(1,2),((1,2),(1,3),(1,4)))
        self.assertEqual(compare.legal_vectors(2,1),((2,3),(2,4)))
        self.assertEqual(compare.legal_vectors(1,0),((1,),))

    def test_value_mask_types_and_structural_bounds_are_not_coerced(self):
        for value,mask in [(0,0),(3,0),(True,0),(1.0,0),(1,True),(1,-1),(1,8),(1,'1')]:
            with self.subTest(value=value,mask=mask),self.assertRaises(ValueError):
                compare.legal_vectors(value,mask)


class ProbabilityScoreTests(unittest.TestCase):
    def test_exact_half_per_full_legal_vector_prior(self):
        self.assertEqual(compare.jeffreys((4,0,1)),(Fraction(9,13),Fraction(1,13),Fraction(3,13)))
        for value,mask,outs in itertools.product((1,2),range(8),range(3)):
            k=len(independent_support(value,mask));counts=tuple(range(k))
            with self.subTest(value=value,mask=mask,outs=outs):
                q=compare.jeffreys(counts)
                self.assertEqual(q,tuple(Fraction(2*n+1,2*sum(counts)+k) for n in counts))
                self.assertTrue(all(type(p) is Fraction and p>0 for p in q))
                self.assertEqual(sum(q),1)
                self.assertEqual(compare.jeffreys((0,)*k),(Fraction(1,k),)*k)

    def test_counts_are_checked_u64_not_bool_coercion_or_unbounded_aggregation(self):
        for counts in [(),None,(True,),(-1,),(1.0,),('1',),(float('nan'),),(2**64,),(2**64-1,1)]:
            with self.subTest(counts=counts),self.assertRaises(ValueError):compare.jeffreys(counts)
        self.assertEqual(compare.jeffreys((2**64-1,)),(Fraction(1),))

    def test_literal_proper_scores_are_count_weighted_not_brier_divided_by_k(self):
        q=(Fraction(2,3),Fraction(1,3));counts=(2,1)
        score=compare.scores(q,counts)
        self.assertEqual(score['events'],3)
        self.assertAlmostEqual(score['negative_log_score'],-(2*math.log(2/3)+math.log(1/3))/3,places=15)
        self.assertAlmostEqual(score['brier_score'],4/9,places=15)
        self.assertEqual(compare.scores(tuple(reversed(q)),tuple(reversed(counts))),score)

    def test_zero_scoring_opportunity_is_none_not_a_measured_zero(self):
        self.assertEqual(compare.scores((Fraction(1,2),)*2,(0,0)),
            dict(events=0,negative_log_score=None,brier_score=None))
        self.assertEqual(compare.scores((Fraction(1),),(10,)),
            dict(events=10,negative_log_score=0.0,brier_score=0.0))

    def test_invalid_probability_support_and_count_shapes_cannot_be_scored(self):
        cases=[((),()),((Fraction(1,2),),(1,)),((Fraction(0),Fraction(1)),(1,1)),
            ((Fraction(-1),Fraction(2)),(1,1)),((Fraction(1),),(1,1)),
            ((float('nan'),),(1,)),((Fraction(1),),(True,)),((Fraction(1),),(-1,)),
            ((Fraction(1,2),)*2,(2**64-1,1))]
        for q,counts in cases:
            with self.subTest(q=q,counts=counts),self.assertRaises(ValueError):compare.scores(q,counts)


class TrainingReportTests(unittest.TestCase):
    def test_all_48_contexts_and_literal_pooled_versus_out_specific_counts(self):
        report=synthetic_report(synthetic_cell('training',0,(1,2),3),
            synthetic_cell('training',1,(1,3),2),synthetic_cell('heldout',0,(1,4),7),
            synthetic_cell('excluded',0,(1,2),11))
        fitted=compare.fit(report)
        self.assertEqual(set(fitted),set(itertools.product((1,2),range(8),range(3))))
        for outs,counts in [(0,(3,0,0)),(1,(0,2,0)),(2,(0,0,0))]:
            context=fitted[(1,1,outs)]
            self.assertEqual(context['support'],((1,2),(1,3),(1,4)))
            self.assertEqual(context['training_counts'],counts)
            self.assertEqual(context['models'][MODELS[0]],(Fraction(1,3),)*3)
            self.assertEqual(context['models'][MODELS[1]],(Fraction(7,13),Fraction(5,13),Fraction(1,13)))
            self.assertEqual(context['models'][MODELS[2]],tuple(Fraction(2*n+1,2*sum(counts)+3) for n in counts))

    def test_reserved_and_excluded_counts_or_endpoints_never_change_fitting(self):
        training=synthetic_cell('training',0,(1,2),4)
        first=synthetic_report(training,synthetic_cell('heldout',0,(1,3),2),synthetic_cell('excluded',1,(1,4),3))
        second=synthetic_report(training,synthetic_cell('heldout',2,(1,4),100),synthetic_cell('excluded',0,(1,2),500))
        self.assertEqual(compare.fit(first),compare.fit(second))

    def test_unqualified_opportunities_are_coverage_not_training_or_scoring(self):
        good=synthetic_cell('heldout',0,(1,3),2)
        unresolved=synthetic_cell('training',1,(1,4),5)
        unresolved.update(qualified=False,reasons=['incomplete_vector'],runners=[])
        report=synthetic_report(good,unresolved)
        self.assertEqual(compare.fit(report),compare.fit(synthetic_report(good)))
        measured=compare.inventory(report)
        self.assertEqual(measured['coverage'],dict(totals=report['totals'],split_totals=report['split_totals']))
        self.assertEqual(measured['subsets']['all']['models'][MODELS[0]]['events'],2)

    def test_shuffling_is_exact_and_inputs_are_not_mutated(self):
        report=synthetic_report(synthetic_cell('training',0,(1,3),5),synthetic_cell('heldout',2,(1,4),7))
        original=copy.deepcopy(report)
        reversed_report=dict(report,joint_vectors=list(reversed(report['joint_vectors'])))
        self.assertEqual(compare.fit(report),compare.fit(reversed_report))
        self.assertEqual(compare.inventory(report),compare.inventory(reversed_report))
        self.assertEqual(report,original)

    def test_illegal_qualified_endpoints_and_partition_counts_fail_before_fitting(self):
        report=synthetic_report(synthetic_cell('training',0,(1,2),3))
        broken=[]
        collision=copy.deepcopy(report);collision['joint_vectors'][0]['runners'][1]['base_end']='First';broken.append(collision)
        count=copy.deepcopy(report);count['totals']['candidate_events']+=1;broken.append(count)
        split=copy.deepcopy(report);split['split_totals']['heldout']['qualified_events']=1;broken.append(split)
        boolean=copy.deepcopy(report);boolean['joint_vectors'][0]['events']=True;broken.append(boolean)
        foreign=copy.deepcopy(report);foreign['joint_vectors'][0]['result']='Triple';broken.append(foreign)
        for data in broken:
            with self.subTest(data=data),patch.object(compare,'jeffreys',side_effect=AssertionError('fitting started')) as started:
                with self.assertRaises(ValueError):compare.fit(data)
                started.assert_not_called()

    def test_all_and_occupied_subsets_do_not_hide_empty_base_determinism(self):
        report=synthetic_report(synthetic_cell('training',0,(1,2),3),
            synthetic_cell('heldout',0,(1,3),2),synthetic_cell('heldout',1,(1,),10,mask=0))
        measured=compare.inventory(report)
        self.assertEqual(len(measured['contexts']),48)
        overall=measured['subsets']['all']['models'][MODELS[0]]
        occupied=measured['subsets']['occupied']['models'][MODELS[0]]
        self.assertEqual(overall['events'],12);self.assertEqual(occupied['events'],2)
        self.assertAlmostEqual(overall['negative_log_score'],2*math.log(3)/12,places=15)
        self.assertAlmostEqual(occupied['negative_log_score'],math.log(3),places=15)
        self.assertAlmostEqual(occupied['brier_score'],2/3,places=15)
        self.assertIsNone(measured['subsets']['double']['models'][MODELS[0]]['negative_log_score'])

    def test_full_probability_presentation_retains_exact_fractions_and_unscored_cells(self):
        measured=compare.inventory(synthetic_report(synthetic_cell('training',0,(1,2),3)))
        context=next(c for c in measured['contexts'] if c['value']==1 and c['base_mask']==1 and c['outs_before']==0)
        self.assertEqual(context['probabilities'][MODELS[2]][0],dict(vector=[1,2],numerator=7,denominator=9,probability=7/9))
        self.assertEqual(context['scoring_events'],0)
        self.assertIsNone(context['scores'][MODELS[2]]['brier_score'])
        delta=measured['subsets']['all']['comparisons']['base-out-minus-pooled']
        self.assertIsNone(delta['negative_log_score']);self.assertIsNone(delta['nonworsening_log'])


class ReportDetailTests(unittest.TestCase):
    def test_all_original_outside_reason_and_source_coverage_is_retained_not_scored(self):
        report=synthetic_report(synthetic_cell('heldout',0,(1,3),2))
        extra=dict(outside_results={'Triple':9},first_reasons={'incomplete_vector':4},
            all_reasons={'incomplete_vector':4,'notation_uncertain':2},source_totals={'raw_events':99})
        report.update(copy.deepcopy(extra))
        result=compare.inventory(report)
        self.assertEqual(result['source_coverage'],extra)
        self.assertEqual(result['subsets']['all']['models'][MODELS[0]]['events'],2)
        result['source_coverage']['outside_results']['Triple']=0
        self.assertEqual(report['outside_results']['Triple'],9)

    def test_context_deltas_and_per_model_training_n_report_worsening_without_selection(self):
        result=compare.inventory(synthetic_report(synthetic_cell('training',0,(1,2),3),synthetic_cell('heldout',0,(1,3),2)))
        context=next(c for c in result['contexts'] if c['value']==1 and c['base_mask']==1 and c['outs_before']==0)
        self.assertEqual(context['model_training_events'],dict(zip(MODELS,(0,3,3))))
        delta=context['comparisons']['base-out-minus-uniform']
        self.assertAlmostEqual(delta['negative_log_score'],math.log(3),places=15)
        self.assertAlmostEqual(delta['brier_score'],20/27,places=15)
        self.assertFalse(delta['nonworsening_log']);self.assertFalse(delta['nonworsening_brier'])
        self.assertEqual(context['comparisons']['base-out-minus-pooled'],dict(
            negative_log_score=0.0,brier_score=0.0,nonworsening_log=True,nonworsening_brier=True))

    def test_every_context_probability_and_subset_denominator_conserves_exactly(self):
        result=compare.inventory(synthetic_report(synthetic_cell('training',0,(1,2),3),
            synthetic_cell('heldout',0,(1,3),2),synthetic_cell('heldout',2,(2,4),7,value=2)))
        for context in result['contexts']:
            for model in MODELS:
                probabilities=context['probabilities'][model]
                self.assertEqual(len(probabilities),context['support_size'])
                self.assertEqual(sum(Fraction(p['numerator'],p['denominator']) for p in probabilities),1)
                self.assertTrue(all(math.isfinite(p['probability']) and p['probability']>0 for p in probabilities))
                self.assertLessEqual(abs(math.fsum(p['probability'] for p in probabilities)-1),32*math.ulp(1.0))
        for model in MODELS:
            subsets=result['subsets']
            self.assertEqual(subsets['all']['models'][model]['events'],9)
            self.assertEqual(sum(subsets[key]['models'][model]['events'] for key in ('single','double')),9)
            self.assertEqual(sum(subsets[f'outs{o}']['models'][model]['events'] for o in range(3)),9)
        json.dumps(result,allow_nan=False)

    def test_coarsening_preserves_joint_endpoint_counts_not_trajectory_or_attempt_features(self):
        one=synthetic_cell('training',0,(1,3),3)
        two=synthetic_cell('training',0,(1,3),2);two['trajectory']='Ground'
        self.assertEqual(compare.fit(synthetic_report(one,two)),compare.fit(synthetic_report(synthetic_cell('training',0,(1,3),5))))

    def test_typed_contexts_and_aggregate_overflow_are_already_green_prefit_refusals(self):
        cell=synthetic_cell('training',0,(1,2),3)
        changes=[dict(base_mask=True),dict(outs_before=True),dict(qualified=1),dict(events=0),dict(events=-1),dict(split='foreign')]
        invalid=[synthetic_report(dict(cell,**change)) for change in changes[:-1]]
        foreign=synthetic_report(cell);foreign['joint_vectors'][0]['split']='foreign';invalid.append(foreign)
        overflow=synthetic_report(dict(cell,events=2**64-1),synthetic_cell('training',1,(1,4),1));invalid.append(overflow)
        for report in invalid:
            with self.subTest(report=report),patch.object(compare,'jeffreys',side_effect=AssertionError('fitting started')) as started:
                with self.assertRaises(ValueError):compare.fit(report)
                started.assert_not_called()


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.fixture=hit_tests.PinnedControllerTests('test_positive_controller_repeats_and_preserves_all_nine_inputs')
        self.addCleanup(self.fixture.doCleanups);self.fixture.setUp()
        self.root=self.fixture.root
        self.accepted=self.root/'accepted-hit.json'
        with self.fixture.configured():hit.publish(*self.fixture.inputs,self.accepted)
        self.accepted_bytes=self.accepted.read_bytes()
        self.inputs=[*self.fixture.inputs,self.accepted]
        self.repin()

    def repin(self):
        self.input_pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)
        self.fixture.input_pins=self.input_pins[:5]
        self.fixture.export_pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.fixture.exports)

    def configured(self):
        stack=ExitStack();stack.enter_context(self.fixture.configured())
        stack.enter_context(patch.object(compare,'INPUT_SHA',self.input_pins,create=True))
        stack.enter_context(patch.object(compare,'EXPORT_SHA',self.fixture.export_pins,create=True))
        return stack

    def test_repository_output_and_both_collision_paths_refuse_before_input_reads(self):
        repo=Path(hit.__file__).resolve().parents[1]
        with self.assertRaisesRegex(ValueError,'outside repository'):
            compare.publish(*(['missing']*6),repo/'comparison-forbidden.json')
        for name in ('existing','linked','qualification'):
            output=self.root/(name+'.json')
            if name=='existing':output.write_text('keep')
            elif name=='linked':output.symlink_to(self.accepted)
            else:output.with_suffix('.qualification.json').write_text('keep qualification')
            with self.subTest(name=name),self.assertRaisesRegex(ValueError,'Refusing to overwrite'):
                compare.publish(*(['missing']*6),output)
        self.assertEqual((self.root/'existing.json').read_text(),'keep')
        self.assertEqual((self.root/'qualification.qualification.json').read_text(),'keep qualification')
        self.assertFalse((repo/'comparison-forbidden.json').exists())

    def test_digest_refuses_before_regeneration_or_fitting(self):
        self.accepted.write_bytes(b'unaccepted')
        with self.configured(),patch.object(hit,'publish',side_effect=AssertionError('regeneration started')):
            with self.assertRaisesRegex(ValueError,'Unsupported.*digest'):
                compare.publish(*self.inputs,self.root/'bad.json')
        self.assertFalse((self.root/'bad.json').exists())
        self.assertFalse((self.root/'bad.qualification.json').exists())

    def test_complete_synthetic_controller_repeats_and_qualification_precedes_scoring(self):
        original=compare.inventory
        expected=[self.root/'first.qualification.json']
        def after_qualification(report):
            self.assertEqual(expected[0].read_bytes(),self.accepted_bytes)
            return original(report)
        with self.configured(),patch.object(compare,'inventory',side_effect=after_qualification):
            first=compare.publish(*self.inputs,self.root/'first.json')
            expected[0]=self.root/'repeat.qualification.json'
            repeat=compare.publish(*self.inputs,self.root/'repeat.json')
        self.assertEqual(first,repeat)
        self.assertEqual((self.root/'first.json').read_bytes(),(self.root/'repeat.json').read_bytes())
        self.assertEqual((self.root/'first.qualification.json').read_bytes(),self.accepted_bytes)
        self.assertEqual((self.root/'repeat.qualification.json').read_bytes(),self.accepted_bytes)
        self.assertEqual(tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs),self.input_pins)
        self.assertEqual(len(first['contexts']),48)
        self.assertEqual(first['provenance']['accepted_qualification_sha256'],self.input_pins[5])

    def test_semantically_equal_but_byte_different_accepted_audit_stops_before_fitting(self):
        self.accepted.write_text(json.dumps(json.loads(self.accepted_bytes),sort_keys=True))
        self.repin()
        with self.configured(),patch.object(compare,'inventory',side_effect=AssertionError('fitting started')):
            with self.assertRaisesRegex(ValueError,'qualification.*byte'):
                compare.publish(*self.inputs,self.root/'mismatch.json')
        self.assertFalse((self.root/'mismatch.json').exists())
        self.assertEqual((self.root/'mismatch.qualification.json').read_bytes(),self.accepted_bytes)

    def test_full_native_group_disagreement_stops_before_fitting(self):
        self.fixture.exports[3].write_text('facts,rows\n')
        new_exports=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.fixture.exports)
        census=json.loads(self.inputs[4].read_text());census['provenance']['native_export_sha256']=list(new_exports)
        self.inputs[4].write_text(json.dumps(census,sort_keys=True));self.repin()
        with self.configured(),patch.object(compare,'inventory',side_effect=AssertionError('fitting started')):
            with self.assertRaisesRegex(ValueError,'Native runner group reconciliation failed'):
                compare.publish(*self.inputs,self.root/'native-mismatch.json')
        self.assertFalse((self.root/'native-mismatch.json').exists())
        self.assertFalse((self.root/'native-mismatch.qualification.json').exists())

    def test_scoring_failure_preserves_complete_qualification_without_report(self):
        with self.configured(),patch.object(compare,'inventory',side_effect=ValueError('synthetic scoring failure')):
            with self.assertRaisesRegex(ValueError,'synthetic scoring failure'):
                compare.publish(*self.inputs,self.root/'failed-score.json')
        self.assertFalse((self.root/'failed-score.json').exists())
        self.assertEqual((self.root/'failed-score.qualification.json').read_bytes(),self.accepted_bytes)

    def test_final_input_and_model_recipe_drift_refuse_but_preserve_qualification(self):
        original=compare.inventory
        recipe=self.root/'comparison-recipe.py';recipe.write_bytes(Path(compare.__file__).read_bytes())
        for name,path in [('input',self.inputs[3]),('recipe',recipe)]:
            before=path.read_bytes()
            def drift(report):
                result=original(report);path.write_bytes(before+b'\ndrift');return result
            with self.subTest(name=name):
                with self.configured(),patch.object(compare,'__file__',str(recipe)),patch.object(compare,'inventory',side_effect=drift):
                    with self.assertRaisesRegex(ValueError,'inputs changed'):
                        compare.publish(*self.inputs,self.root/(name+'-drift.json'))
                self.assertFalse((self.root/(name+'-drift.json')).exists())
                self.assertEqual((self.root/(name+'-drift.qualification.json')).read_bytes(),self.accepted_bytes)
            path.write_bytes(before)

    def test_final_query_drift_refuses_without_mutating_repository_queries(self):
        original=compare.inventory
        folder=self.root/'query-copy';folder.mkdir()
        runner_copy=folder/'runner_transitions.py';runner_copy.write_bytes(Path(hit.runner.__file__).read_bytes())
        queries=['2019-lineup-readiness.sql','2019-outcomes.sql','2019-raw-mechanisms.sql','2019-runner-transitions.sql']
        for name in queries:(folder/name).write_bytes(Path(hit.runner.__file__).with_name(name).read_bytes())
        def drift(report):
            result=original(report);(folder/queries[0]).write_text('drift');return result
        with self.configured(),patch.object(hit.runner,'__file__',str(runner_copy)),patch.object(compare,'inventory',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                compare.publish(*self.inputs,self.root/'query-drift.json')
        self.assertFalse((self.root/'query-drift.json').exists())
        self.assertEqual((self.root/'query-drift.qualification.json').read_bytes(),self.accepted_bytes)

    def test_redirected_output_parent_refuses_before_any_repository_publication(self):
        original=compare.inventory
        parent=self.root/'external-output';parent.mkdir();saved=self.root/'saved-output'
        repo=Path(hit.__file__).resolve().parents[1]
        output=parent/'comparison-parent-redirect-test.json'
        self.assertFalse((repo/output.name).exists())
        def redirect(report):
            result=original(report);parent.rename(saved);parent.symlink_to(repo,target_is_directory=True);return result
        with self.configured(),patch.object(compare,'inventory',side_effect=redirect):
            with self.assertRaisesRegex(ValueError,'output path changed'):
                compare.publish(*self.inputs,output)
        self.assertFalse((repo/output.name).exists())
        self.assertEqual((saved/output.with_suffix('.qualification.json').name).read_bytes(),self.accepted_bytes)

    def test_nonfinite_score_cannot_be_published(self):
        original=compare.inventory
        def nonfinite(report):return dict(original(report),nonfinite=float('nan'))
        with self.configured(),patch.object(compare,'inventory',side_effect=nonfinite):
            with self.assertRaises(ValueError):compare.publish(*self.inputs,self.root/'nonfinite.json')
        self.assertFalse((self.root/'nonfinite.json').exists())
        self.assertEqual((self.root/'nonfinite.qualification.json').read_bytes(),self.accepted_bytes)

    def test_model_recipe_drift_during_qualification_is_an_already_green_prefit_refusal(self):
        original=hit.publish
        recipe=self.root/'prefit-recipe.py';recipe.write_bytes(Path(compare.__file__).read_bytes())
        def drift(*args):
            report=original(*args);recipe.write_bytes(recipe.read_bytes()+b'\ndrift');return report
        with self.configured(),patch.object(compare,'__file__',str(recipe)),patch.object(hit,'publish',side_effect=drift),patch.object(compare,'inventory',side_effect=AssertionError('fitting started')):
            with self.assertRaisesRegex(ValueError,'inputs changed before fitting'):
                compare.publish(*self.inputs,self.root/'prefit-drift.json')
        self.assertFalse((self.root/'prefit-drift.json').exists())
        self.assertEqual((self.root/'prefit-drift.qualification.json').read_bytes(),self.accepted_bytes)

    def test_qualification_artifact_drift_is_an_already_green_final_reread_refusal(self):
        original=compare.inventory
        output=self.root/'qualification-drift.json'
        def drift(report):
            result=original(report);output.with_suffix('.qualification.json').write_text('drift');return result
        with self.configured(),patch.object(compare,'inventory',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                compare.publish(*self.inputs,output)
        self.assertFalse(output.exists())
        self.assertEqual(output.with_suffix('.qualification.json').read_text(),'drift')
        self.assertEqual(self.accepted.read_bytes(),self.accepted_bytes)

    def test_late_output_collision_is_an_already_green_exclusive_writer_refusal(self):
        original=compare.inventory
        output=self.root/'late-collision.json'
        def collide(report):
            result=original(report);output.write_bytes(b'keep late collision');return result
        with self.configured(),patch.object(compare,'inventory',side_effect=collide):
            with self.assertRaises(FileExistsError):compare.publish(*self.inputs,output)
        self.assertEqual(output.read_bytes(),b'keep late collision')
        self.assertEqual(output.with_suffix('.qualification.json').read_bytes(),self.accepted_bytes)

    def test_cli_uses_same_exclusive_validated_publisher(self):
        output=self.root/'cli.json';stream=io.StringIO()
        with self.configured(),patch('sys.argv',['comparison',*map(str,self.inputs),str(output)]),redirect_stdout(stream):
            compare.main()
        saved=json.loads(output.read_text())
        self.assertEqual(json.loads(stream.getvalue())['format'],saved['format'])
        self.assertEqual(output.with_suffix('.qualification.json').read_bytes(),self.accepted_bytes)


if __name__=='__main__':
    unittest.main()
