"""Source-shape counts are not historical catch/throw certificates."""
import copy
import csv
import io
import itertools
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from audits.joint_contact import inventory, read_export
from tests.test_outcome_inventory import event
from tests.test_raw_mechanisms import row, SUPPLEMENT_SHA, SOURCE_COMMIT, CATALOG_SHA, SNAPSHOT_SHA

BINS = ['ground_candidate', 'fly_candidate', 'line_candidate', 'pop_candidate', 'residual']

def fixture(tag='Fly', key=1, mask=0, outs=0):
    prefixes={'GroundBall':'G','Fly':'F','LineDrive':'L','PopUp':'P','Unknown':'F','':'F'}
    ground=tag=='GroundBall'
    credits=([dict(sequence_id=1,fielding_position=6,fielding_play='Assist')] if ground else [])
    credits.append(dict(sequence_id=len(credits)+1,fielding_position=3 if ground else 7,fielding_play='Putout'))
    raw=row(event_key=str(key),event_id=str(key),result='InPlayOut',outs='1',trajectory=tag,
            angle='Default',raw_play=('63' if ground else '7')+'/'+prefixes[tag]+'7D',
            credits=json.dumps(credits))
    context=event(event_key=key,event_id=key,trajectory=tag,assists=int(ground),
                  base_mask=mask,outs_before=outs,after_outs=outs+1,frame_end='true' if outs==2 else 'false',
                  first='F1' if mask&1 else '',second='S1' if mask&2 else '',third='T1' if mask&4 else '')
    return raw,context

def classify(raw, context):
    return inventory([raw],[context])

class JointContactTests(unittest.TestCase):
    def test_four_candidates_and_unknown_residual_conserve_original_contact_count(self):
        pairs=[fixture(tag,i+1) for i,tag in enumerate(['GroundBall','Fly','LineDrive','PopUp','Unknown'])]
        report=inventory([p[0] for p in pairs],[p[1] for p in pairs])
        self.assertEqual(report['contact_bins'],dict.fromkeys(BINS,1))
        self.assertEqual(report['counts']['contact_out'],5)
        self.assertEqual(sum(report['counts'].values()),5)
        self.assertEqual(report['residual_reasons'],{'unknown_trajectory':1})
        unknown=next(x for x in report['examples'] if x['bin']=='residual')
        self.assertEqual(unknown['event']['trajectory'],'Unknown')
        self.assertEqual(unknown['event']['raw_play'],'7/F7D')

    def test_all_before_cells_and_independent_row_order_are_exact(self):
        pairs=[]
        for tag,mask,outs in itertools.product(['GroundBall','Fly','LineDrive','PopUp'],range(8),range(3)):
            pairs.append(fixture(tag,len(pairs)+1,mask,outs))
        raw,context=[p[0] for p in pairs],[p[1] for p in pairs]
        report=inventory(raw,context)
        self.assertEqual(report,inventory(list(reversed(raw)),list(reversed(context))))
        self.assertEqual(len(report['contexts']),96)
        self.assertTrue(all(c['events']==1 for c in report['contexts']))
        self.assertEqual(report['contact_bins'],dict(ground_candidate=24,fly_candidate=24,line_candidate=24,pop_candidate=24,residual=0))

    def test_flags_compounds_advances_and_unknown_modifiers_stay_residual(self):
        for changes,reason in [
            ({'flags':json.dumps([dict(sequence_id=1,flag='InfieldFlyRule')])},'flags_present'),
            ({'raw_play':'7/F7D/IF'},'literal_not_supported'),
            ({'raw_play':'7/F7D/MREV'},'literal_not_supported'),
            ({'raw_play':'7/F7D.1-1'},'literal_not_supported'),
            ({'raw_play':'7+WP/F7D'},'literal_not_supported'),
            ({'raw_play':'7/F7D.'},'literal_not_supported'),
            ({'raw_play':'7/BF7D'},'literal_not_supported'),
            ({'raw_play':'7/L7D'},'literal_not_supported'),
            ({'raw_play':'8/F7D'},'root_credit_mismatch'),
        ]:
            raw,context=fixture()
            raw.update(changes)
            with self.subTest(changes=changes):
                report=classify(raw,context)
                self.assertEqual(report['contact_bins']['residual'],1)
                self.assertEqual(report['residual_reasons'],{reason:1})
                self.assertEqual(report['examples'][0]['event'],raw)

    def test_extra_assist_or_unknown_credit_position_is_residual_not_a_history_cap(self):
        for credits in [
            [dict(sequence_id=1,fielding_position=0,fielding_play='Putout')],
            [dict(sequence_id=1,fielding_position=7,fielding_play='Putout')]+[
                dict(sequence_id=i+2,fielding_position=6,fielding_play='Assist') for i in range(7)],
        ]:
            raw,context=fixture();raw['credits']=json.dumps(credits)
            report=classify(raw,context)
            self.assertEqual(report['residual_reasons'],{'credit_shape':1})

    def test_explicit_held_identity_qualifies_but_missing_endpoint_or_attempt_does_not(self):
        held=dict(baserunner='First',runner_id='F1',baserunning_play_type='Advance',is_out=False,
                  run_scored_flag=False,explicit_out_flag=False,advanced_on_error_flag=False,
                  attempted_advance_to_base=None,base_end='First',is_advance_attempt=False)
        raw,context=fixture(mask=1);raw['runners']=json.dumps([held])
        self.assertEqual(classify(raw,context)['contact_bins']['fly_candidate'],1)
        for changes in [dict(base_end=None),dict(base_end='Second'),dict(runner_id=None),
                        dict(runner_id='OTHER'),dict(is_advance_attempt=True),dict(advanced_on_error_flag=None)]:
            bad=dict(raw,runners=json.dumps([dict(held,**changes)]))
            with self.subTest(changes=changes):
                self.assertEqual(classify(bad,context)['residual_reasons'],{'runner_effect_or_unknown':1})

    def test_residual_precedence_does_not_erase_unknowns(self):
        raw,context=fixture('Unknown')
        raw['flags']=json.dumps([dict(sequence_id=1,flag='UnspecifiedDoublePlay')])
        self.assertEqual(classify(raw,context)['residual_reasons'],{'unknown_trajectory':1})
        raw['result']=context['result']='SacrificeFly'
        self.assertEqual(classify(raw,context)['residual_reasons'],{'non_inplay_result':1})

    def test_joins_reject_missing_extra_duplicate_or_disagreeing_real_events(self):
        raw,context=fixture()
        for raws,contexts in [([raw],[]),([],[context]),([raw,raw],[context]),([raw],[context,context]),
                              ([dict(raw,game_id='OTHER')],[context]),
                              ([dict(raw,trajectory='PopUp')],[context]),
                              ([dict(raw,angle='Foul')],[context])]:
            with self.subTest(raws=raws,contexts=contexts),self.assertRaises(ValueError):inventory(raws,contexts)

    def test_existing_required_units_json_grain_and_before_state_guards_remain_strict(self):
        raw,context=fixture()
        for changes in [dict(outs=''),dict(flags='null'),dict(credits='[]'),
                        dict(flags='[{"sequence_id":1,"flag":"A","flag":"B"}]')]:
            with self.subTest(changes=changes),self.assertRaises(ValueError):classify(dict(raw,**changes),context)
        for changes in [dict(outs_before=''),dict(base_mask='1'),dict(season='2020')]:
            with self.subTest(changes=changes),self.assertRaises(ValueError):classify(raw,dict(context,**changes))

    def test_no_play_history_is_validated_but_does_not_create_counts(self):
        raw,context=fixture()
        np=row(event_key='2',event_id='2',no_play_flag='true',result='',outs='0',runs='0',raw_play='NP')
        report=inventory([raw,np],[context])
        self.assertEqual(report['counts']['contact_out'],1)
        self.assertEqual(report['raw_totals']['no_play_events'],1)
        self.assertEqual(report['raw_totals']['events'],2)

    def test_all_original_coarsenings_and_two_exclusions_are_separate(self):
        groups={'strikeout':['StrikeOut'],'walk':['Walk'],'hbp':['HitByPitch'],'single':['Single'],
                'double':['Double','GroundRuleDouble'],'triple':['Triple'],
                'home_run':['HomeRun','InsideTheParkHomeRun'],'roe':['ReachedOnError'],
                'contact_out':['InPlayOut','FieldersChoice','SacrificeFly','SacrificeHit'],
                'excluded_ibb':['IntentionalWalk'],'excluded_interference':['Interference']}
        raws=[];contexts=[]
        for names in groups.values():
            for name in names:
                raw,context=fixture(key=len(raws)+1)
                raw['result']=context['result']=name
                raws.append(raw);contexts.append(context)
        report=inventory(raws,contexts)
        self.assertEqual(report['counts'],{key:len(names) for key,names in groups.items() if not key.startswith('excluded_')})
        self.assertEqual(report['exclusions'],dict(excluded_ibb=1,excluded_interference=1))
        self.assertEqual(sum(report['contact_bins'].values()),4)
        self.assertEqual(report['residual_reasons'],{'non_inplay_result':3})
        raw,context=fixture();raw['result']=context['result']='UnknownResult'
        with self.assertRaises(ValueError):classify(raw,context)

    def test_raw_header_validation_is_reused_not_relaxed(self):
        for text in ['event_key,event_key\n1,1\n','event_key,unknown\n1,2\n']:
            with self.subTest(text=text),self.assertRaises(ValueError):list(read_export(io.StringIO(text)))

class JointCliTests(unittest.TestCase):
    def test_valid_tiny_data_cannot_publish_as_complete_pinned_2019_cohort(self):
        with tempfile.TemporaryDirectory() as directory:
            folder=Path(directory);raw,context=fixture()
            paths=[folder/name for name in ['raw.csv','context.csv','raw-manifest.json','context-manifest.csv','out.json']]
            for path,value in zip(paths[:2],[raw,context]):
                with path.open('w',newline='') as stream:
                    writer=csv.DictWriter(stream,fieldnames=value.keys());writer.writeheader();writer.writerow(value)
            paths[2].write_text(json.dumps(dict(format='outcome-detail-2019-v1',source_commit=SOURCE_COMMIT,
                catalog_sha256=CATALOG_SHA,snapshot_sha256=SNAPSHOT_SHA,database_sha256=SUPPLEMENT_SHA,
                snapshot_id=857,attribution='Retrosheet')))
            value=dict(catalog_sha256=CATALOG_SHA,schema_version='f03-spike-1',query_version='2019-filtered-extract-v1',
                       source_url='https://example.invalid/pinned',extracted_at='fixed fixture',attribution='Retrosheet')
            with paths[3].open('w',newline='') as stream:
                writer=csv.DictWriter(stream,fieldnames=value.keys());writer.writeheader();writer.writerow(value)
            result=subprocess.run([sys.executable,'-m','audits.joint_contact',*map(str,paths)],text=True,capture_output=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('Incomplete or mismatched 2019',result.stderr)
            self.assertFalse(paths[-1].exists())

    def test_existing_output_refused_before_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            out=Path(directory)/'existing.json';out.write_text('preserve')
            result=subprocess.run([sys.executable,'-m','audits.joint_contact','missing-raw','missing-context',
                'missing-raw-manifest','missing-context-manifest',str(out)],text=True,capture_output=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('Refusing to overwrite',result.stderr)
            self.assertEqual(out.read_text(),'preserve')

if __name__=='__main__':unittest.main()
