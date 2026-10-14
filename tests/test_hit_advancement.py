"""A complete endpoint proxy is not a causal hit or NoMisplay certificate."""
import copy
import csv
import hashlib
import io
import itertools
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from audits import hit_advancement as hit
from audits import runner_transitions as runner
from audits import lineup_readiness as lineup
from tests.test_lineup_readiness import fixture as lineup_fixture
from tests.test_joint_contact import fixture
from tests.test_runner_transitions import attached, child


def hit_pair(value=1, mask=0, outs=0, key=1, endpoints=None):
    row, context = fixture(key=key, mask=mask, outs=outs)
    result = 'Single' if value == 1 else 'Double'
    row.update(result=result, outs='0', runs='0', credits='[]', raw_play=('S' if value == 1 else 'D')+'7/G')
    context.update(result=result, outs='0', state_outs='0', batting_outs='0', putouts='0',
        after_outs=str(outs), frame_end='false', runs='0', state_runs='0')
    endpoints = endpoints or {}
    destinations = ['First', 'Second', 'Third', 'Home']
    children = [child('Batter', 'BATTER', base_end=destinations[value-1])]
    for i, role in enumerate(['First', 'Second', 'Third']):
        if mask & (1 << i):
            target = endpoints.get(role, destinations[min(i+value, 3)])
            children.append(child(role, ['F1','S1','T1'][i], base_end=target,
                run_scored_flag=target=='Home'))
    after_mask = sum(1 << destinations.index(c['base_end']) for c in children if c['base_end']!='Home')
    runs = sum(c['run_scored_flag'] for c in children)
    row['runs'] = context['runs'] = context['state_runs'] = str(runs)
    context['after_base_mask'] = str(after_mask)
    return attached((row,context),children)


class QualificationTests(unittest.TestCase):
    def test_single_and_double_complete_vectors_have_no_failed_predicates(self):
        for value in [1,2]:
            pair = hit_pair(value=value)
            with self.subTest(value=value):
                result = hit.classify(*pair)
                self.assertTrue(result['qualified'])
                self.assertEqual(result['reasons'], [])
                self.assertEqual(result['runners'][0]['base_end'], 'First' if value==1 else 'Second')

    def test_held_and_nonstation_joint_scorers_are_not_forced_to_station_policy(self):
        for mask,endpoints,expected in [(2,{'Second':'Second'},['First','Second']),
                (3,{'First':'Third','Second':'Home'},['First','Third','Home'])]:
            pair=hit_pair(mask=mask,endpoints=endpoints)
            with self.subTest(mask=mask):
                result=hit.classify(*pair)
                self.assertTrue(result['qualified'],result['reasons'])
                self.assertEqual([r['base_end'] for r in result['runners']],expected)

    def test_missing_child_is_unresolved_not_a_held_runner(self):
        row,context=hit_pair(mask=1)
        children=json.loads(row['runners']);children=children[:1]
        row,context=attached((row,context),children)
        result=hit.classify(row,context)
        self.assertFalse(result['qualified'])
        self.assertIn('incomplete_vector',result['reasons'])

    def test_null_cause_flags_are_not_false_or_no_misplay(self):
        pair=hit_pair(mask=1)
        for field in ['explicit_out_flag','advanced_on_error_flag']:
            row,context=copy.deepcopy(pair)
            children=json.loads(row['runners']);children[1][field]=None
            row,context=attached((row,context),children)
            with self.subTest(field=field):
                result=hit.classify(row,context)
                self.assertFalse(result['qualified'])
                self.assertIn('runner_effect_or_unknown',result['reasons'])

    def test_all_failing_predicates_are_retained_in_fixed_order(self):
        row,context=hit_pair(mask=1)
        row['raw_play']='S7+E4/TH.1-3'
        children=json.loads(row['runners']);children[1]['base_end']=None
        children[1]['advanced_on_error_flag']=True
        row,context=attached((row,context),children)
        result=hit.classify(row,context)
        self.assertFalse(result['qualified'])
        self.assertEqual(result['reasons'],['notation_uncertain','incomplete_vector','runner_effect_or_unknown'])

    def test_geometry_cannot_overtake_a_leading_survivor(self):
        row,context=hit_pair(mask=3,endpoints={'First':'Home','Second':'Third'})
        result=hit.classify(row,context)
        self.assertFalse(result['qualified'])
        self.assertIn('unsupported_geometry',result['reasons'])

    def test_unavailable_boundary_after_state_is_preserved_not_cast_or_zero_filled(self):
        for blank in [('after_base_mask',),('after_outs',),('after_base_mask','after_outs')]:
            row,context=hit_pair()
            context['frame_end']='true'
            for key in blank:context[key]=''
            original=copy.deepcopy((row,context))
            with self.subTest(blank=blank):
                source=runner.inventory([row],[context],['GAME1'])
                result=hit.inventory([row],[context],['GAME1'],expected_census=source)
                self.assertEqual(result['totals'],dict(candidate_events=1,qualified_events=0,unresolved_events=1))
                self.assertEqual(result['first_reasons'],{'terminal_or_out':1})
                self.assertEqual(result['all_reasons'],{'terminal_or_out':1,'unsupported_geometry':1})
                self.assertEqual((row,context),original)
        row,context=hit_pair();context['after_base_mask']=''
        with patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
            with self.assertRaisesRegex(ValueError,'after_base_mask'):hit.inventory([row],[context],['GAME1'])
            measured.assert_not_called()

    def test_known_out_endpoint_is_not_missing_and_keeps_all_independent_reasons(self):
        for role in ['Batter','First']:
            row,context=hit_pair(mask=1)
            children=json.loads(row['runners'])
            for runner_child in children:
                if runner_child['baserunner']==role:
                    runner_child.update(base_end='Out',is_out=True,explicit_out_flag=True)
            row,context=attached((row,context),children)
            row['outs']=context['outs']=context['state_outs']=context['after_outs']='1'
            context['after_base_mask']='2' if role=='Batter' else '1'
            value=hit.classify(row,context)
            with self.subTest(role=role):
                self.assertNotIn('incomplete_vector',value['reasons'])
                expected=['terminal_or_out','runner_effect_or_unknown']
                if role=='Batter':expected.append('hit_destination')
                expected.append('unsupported_geometry')
                self.assertEqual(value['reasons'],expected)
                self.assertEqual(next(c for c in value['runners'] if c['baserunner']==role)['base_end'],'Out')

    def test_missing_other_endpoint_does_not_erase_known_geometry_contradictions(self):
        cases=[(1,{'First':'Out'}),(2,{'Second':'First'}),(3,{'First':'Third','Second':'Third'}),
            (3,{'First':'Home','Second':'Third'})]
        for mask,endpoints in cases:
            row,context=hit_pair(mask=mask)
            children=json.loads(row['runners']);children[0]['base_end']=None
            for runner_child in children[1:]:
                runner_child['base_end']=endpoints[runner_child['baserunner']]
                runner_child['is_out']=runner_child['explicit_out_flag']=runner_child['base_end']=='Out'
                runner_child['run_scored_flag']=runner_child['base_end']=='Home'
            row,context=attached((row,context),children)
            with self.subTest(endpoints=endpoints):
                value=hit.classify(row,context)
                self.assertIn('incomplete_vector',value['reasons'])
                self.assertIn('unsupported_geometry',value['reasons'])
        row,context=hit_pair(mask=1)
        children=json.loads(row['runners']);children[0]['base_end']=None
        row,context=attached((row,context),children)
        self.assertNotIn('unsupported_geometry',hit.classify(row,context)['reasons'])

    def test_extra_observed_child_stays_in_unresolved_joint_vector(self):
        row,context=hit_pair(mask=1)
        children=json.loads(row['runners'])+[child('Third','EXTRA',base_end='Home',run_scored_flag=True,
            attempted_advance_to_base='Unmapped',is_advance_attempt=None)]
        row,context=attached((row,context),children)
        value=hit.classify(row,context)
        self.assertEqual(value['reasons'],['incomplete_vector'])
        self.assertEqual(value['runners'],children)
        self.assertEqual(value['runners'][-1]['attempted_advance_to_base'],'Unmapped')
        self.assertIsNone(value['runners'][-1]['is_advance_attempt'])

    def test_supported_id_collection_contract_is_preserved_before_measurement(self):
        pair=hit_pair(mask=1)
        expected=hit.inventory([pair[0]],[pair[1]],['GAME1'])
        self.assertEqual(hit.inventory(iter([pair[0]]),iter([pair[1]]),('GAME1',)),expected)
        # Complete census deliberately requires a list/tuple, not an iterator or set.
        # This is an already-green refusal characterization, not a partition repair.
        for ids in [iter(['GAME1']),{'GAME1'},['GAME1','GAME1'],['FOREIGN'],[None]]:
            with self.subTest(kind=type(ids).__name__),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
                with self.assertRaisesRegex(ValueError,'Invalid supported runner-census game identities'):
                    hit.inventory([pair[0]],[pair[1]],ids)
                measured.assert_not_called()

    def test_all_masks_outs_values_conserve_candidate_vectors_without_mutation(self):
        pairs=[hit_pair(value=value,mask=mask,outs=outs,key=i+1)
            for i,(value,mask,outs) in enumerate(itertools.product([1,2],range(8),range(3)))]
        original=copy.deepcopy(pairs)
        result=hit.inventory([p[0] for p in pairs],[p[1] for p in pairs],['GAME1'])
        self.assertEqual(result['format'],'2019-hit-advancement-v1')
        self.assertEqual(result['totals']['candidate_events'],48)
        self.assertEqual(result['totals']['qualified_events'],48)
        self.assertEqual(result['totals']['unresolved_events'],0)
        self.assertEqual(sum(c['events'] for c in result['joint_vectors']),48)
        self.assertEqual(result,hit.inventory([p[0] for p in reversed(pairs)],[p[1] for p in reversed(pairs)],['GAME1']))
        self.assertEqual(pairs,original)


class CoverageTests(unittest.TestCase):
    def test_nonordinary_and_no_play_rows_remain_counted_outside_candidate_universe(self):
        single=hit_pair(key=1)
        special=hit_pair(value=2,key=2)
        special[0].update(result='GroundRuleDouble',raw_play='DGR')
        special[1]['result']='GroundRuleDouble'
        no_play=dict(single[0],event_key='3',event_id='3',no_play_flag='true',result='',
            raw_play='NP',runners='[]',trajectory='',angle='')
        result=hit.inventory([single[0],special[0],no_play],[single[1],special[1]],['GAME1'])
        self.assertEqual(result['totals']['candidate_events'],1)
        self.assertEqual(result['outside_results'],{'GroundRuleDouble':1})
        self.assertEqual(result['source_totals']['real_events'],2)
        self.assertEqual(result['source_totals']['no_play_events'],1)

    def test_all_three_reserved_partitions_conserve_candidates_and_unresolved_events(self):
        pairs=[hit_pair(key=1),hit_pair(key=2)]
        for pair,game in zip(pairs,['SYNTHETIC_SUPPORTED','SYNTHETIC_EXCLUDED']):
            pair[0]['game_id']=pair[1]['game_id']=game
        row,context=pairs[1];row,context=attached((row,context),[]);pairs[1]=(row,context)
        result=hit.inventory([p[0] for p in pairs],[p[1] for p in pairs],['SYNTHETIC_SUPPORTED'])
        self.assertEqual(set(result['split_totals']),{'training','heldout','excluded'})
        self.assertEqual(sum(c['candidate_events'] for c in result['split_totals'].values()),2)
        self.assertEqual(result['split_totals']['excluded'],{'candidate_events':1,'qualified_events':0,'unresolved_events':1})
        self.assertEqual(result['first_reasons'],{'incomplete_vector':1})

    def test_valid_attempt_unknowns_are_retained_not_attempt_policy_certificates(self):
        row,context=hit_pair(mask=1)
        children=json.loads(row['runners']);children[1].update(is_advance_attempt=None,
            attempted_advance_to_base='Unmapped',baserunning_play_type='Unmapped')
        row,context=attached((row,context),children)
        result=hit.classify(row,context)
        self.assertTrue(result['qualified'])
        self.assertEqual(result['runners'][1]['attempted_advance_to_base'],'Unmapped')
        self.assertIsNone(result['runners'][1]['is_advance_attempt'])
        self.assertEqual(result['runners'][1]['baserunning_play_type'],'Unmapped')

    def test_literal_annotations_duplicate_origins_and_destination_disagreement_stay_unqualified(self):
        row,context=hit_pair(mask=1)
        for play in ['S7/TH.1-2','S7/G.1-2(E4)','S7/G.1X2','S7/G.1-2;1-2',
                'S7/G.1-3','S7/G.2-3','S7/G/IF','D7/G','S7..1-2']:
            with self.subTest(play=play):
                result=hit.classify(dict(row,raw_play=play),context)
                self.assertFalse(result['qualified'])
                self.assertIn('notation_uncertain',result['reasons'])
        self.assertTrue(hit.classify(dict(row,raw_play='S7/G.1-2;B-1'),context)['qualified'])

    def test_terminal_fielding_and_batter_extra_advance_keep_independent_reasons(self):
        row,context=hit_pair()
        children=json.loads(row['runners']);children[0]['base_end']='Second'
        row,context=attached((row,context),children);context.update(frame_end='true',after_base_mask='2')
        row['credits']=json.dumps([dict(sequence_id=1,fielding_position=7,fielding_play='Error')])
        result=hit.classify(row,context)
        self.assertEqual(result['reasons'],['terminal_or_out','fielding_effect','hit_destination'])

    def test_duplicate_role_identity_and_cross_event_validator_guards_are_unchanged(self):
        pair=hit_pair(mask=1)
        children=json.loads(pair[0]['runners'])
        broken=[dict(pair[0],runners=json.dumps(children+children[:1])),
            dict(pair[0],runners=json.dumps([dict(c,runner_id='FOREIGN') if c['baserunner']=='First' else c for c in children])),
            dict(pair[0],game_id='FOREIGN')]
        for row in broken:
            with self.subTest(row=row),self.assertRaises(ValueError):
                hit.inventory([row],[pair[1]],['GAME1'])


class ReconciliationPublicationTests(unittest.TestCase):
    def test_native_mismatch_blocks_new_qualification(self):
        pair=hit_pair(mask=1)
        with patch.object(hit,'classify',side_effect=AssertionError('new qualification started')) as measured:
            with self.assertRaisesRegex(ValueError,'Native runner group reconciliation failed'):
                hit.inventory([pair[0]],[pair[1]],['GAME1'],native_groups={})
            measured.assert_not_called()

    def test_census_mismatch_blocks_new_qualification(self):
        pair=hit_pair(mask=1)
        with patch.object(hit,'classify',side_effect=AssertionError('new qualification started')) as measured:
            with self.assertRaisesRegex(ValueError,'census reconciliation'):
                hit.inventory([pair[0]],[pair[1]],['GAME1'],expected_census={})
            measured.assert_not_called()

    def test_matching_native_and_complete_census_do_not_change_measurement(self):
        pair=hit_pair(mask=1)
        census=runner.inventory([pair[0]],[pair[1]],['GAME1'])
        groups={json.dumps({k:v for k,v in cell.items() if k!='rows'},sort_keys=True):cell['rows']
            for cell in census['raw_child_cells']}
        ordinary=hit.inventory([pair[0]],[pair[1]],['GAME1'])
        checked=hit.inventory([pair[0]],[pair[1]],['GAME1'],native_groups=groups,expected_census=census)
        self.assertEqual(checked,ordinary)

    def test_repository_output_refuses_before_reading_missing_inputs(self):
        output=Path(__file__).resolve().parents[1]/'hit-advancement-forbidden.json'
        with self.assertRaisesRegex(ValueError,'outside repository'):
            hit.publish(*(['missing']*5),output)
        self.assertFalse(output.exists())

    def test_existing_output_and_symlink_refuse_before_input_reads(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);existing=root/'existing.json';existing.write_text('keep')
            linked=root/'linked.json';linked.symlink_to(existing)
            for output in [existing,linked]:
                with self.subTest(output=output),self.assertRaisesRegex(ValueError,'Refusing to overwrite'):
                    hit.publish(*(['missing']*5),output)
            self.assertEqual(existing.read_text(),'keep')
            self.assertTrue(linked.is_symlink())

    def test_unaccepted_input_digest_refuses_before_missing_exports_or_report(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);inputs=[root/str(i) for i in range(5)]
            for path in inputs:path.write_text('synthetic unaccepted')
            with self.assertRaisesRegex(ValueError,'Unsupported.*digest'):
                hit.publish(*inputs,root/'out.json')
            self.assertEqual(set(root.iterdir()),set(inputs))

    def test_final_reread_and_exclusive_finite_sorted_writer(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);source=root/'source';source.write_bytes(b'stable')
            pins=[hashlib.sha256(source.read_bytes()).hexdigest()]
            output=root/'out.json';report={'z':None,'a':0}
            hit.write_report(report,[source],pins,output)
            expected=(json.dumps(report,sort_keys=True,indent=2,allow_nan=False)+'\n').encode()
            self.assertEqual(output.read_bytes(),expected)
            with self.assertRaises(FileExistsError):hit.write_report({},[source],pins,output)
            with self.assertRaises(ValueError):hit.write_report({'bad':float('nan')},[source],pins,root/'nan.json')
            self.assertFalse((root/'nan.json').exists())
            source.write_bytes(b'drift')
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                hit.write_report(report,[source],pins,root/'drift.json')
            self.assertFalse((root/'drift.json').exists())
            self.assertEqual(output.read_bytes(),expected)


class PinnedControllerTests(unittest.TestCase):
    def setUp(self):
        temp=tempfile.TemporaryDirectory();self.addCleanup(temp.cleanup)
        self.root=Path(temp.name)
        self.inputs=[self.root/name for name in ['snapshot','supplement','lineups.json','manifest.json','census.json']]
        self.inputs[0].write_bytes(b'synthetic snapshot');self.inputs[1].write_bytes(b'synthetic supplement')
        digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
        source_pins=[digest(p) for p in self.inputs[:2]]
        parts=lineup_fixture(game_id='GAME1');lr=lineup.inventory(*parts)
        lr['provenance']=dict(snapshot_sha256=source_pins[0],supplement_sha256=source_pins[1],attribution='notice')
        self.inputs[2].write_text(json.dumps(lr,sort_keys=True))
        detail=dict(format='outcome-detail-2019-v1',source_commit=lineup.SOURCE_COMMIT,catalog_sha256=lineup.CATALOG,
            snapshot_sha256=source_pins[0],database_sha256=source_pins[1],snapshot_id=857,attribution='notice')
        self.inputs[3].write_text(json.dumps(detail,sort_keys=True))
        snapshot_manifest=dict(schema_version='f03-spike-1',query_version='2019-filtered-extract-v1',
            catalog_sha256=lineup.CATALOG,source_url='synthetic local',extracted_at='synthetic',attribution='notice')
        records=[dict(record_type=kind,payload=json.dumps(item,sort_keys=True)) for kind,items in
            [('game',parts[0]),('person',parts[1]),('appearance',parts[2]),('manifest',[snapshot_manifest])] for item in items]
        self.pair=hit_pair(mask=1)
        census=runner.inventory([self.pair[0]],[self.pair[1]],['GAME1'])
        groups=[dict(facts=json.dumps({k:v for k,v in cell.items() if k!='rows'},sort_keys=True),rows=cell['rows'])
            for cell in census['raw_child_cells']]
        exports=[self.csv_text(['record_type','payload'],records),self.csv_text(list(self.pair[1]),[self.pair[1]]),
            self.csv_text(list(self.pair[0]),[self.pair[0]]),self.csv_text(['facts','rows'],groups)]
        self.exports=[self.inputs[4].with_suffix('.'+kind+'.csv') for kind in ['lineup','events','raw','groups']]
        for path,text in zip(self.exports,exports):path.write_text(text)
        self.export_pins=tuple(digest(p) for p in self.exports)
        queries=[Path(runner.__file__).with_name(name) for name in
            ['2019-lineup-readiness.sql','2019-outcomes.sql','2019-raw-mechanisms.sql','2019-runner-transitions.sql']]
        recipes=[Path(module.__file__) for module in [runner,lineup,runner.outcome,runner.raw,runner.joint_contact]]
        census['provenance']=dict(query_sha256=[digest(p) for p in queries],recipe_sha256=[digest(p) for p in recipes],
            source_commit=lineup.SOURCE_COMMIT,snapshot_id=857,snapshot=snapshot_manifest,
            snapshot_sha256=source_pins[0],supplement_sha256=source_pins[1],lineup_report_sha256=digest(self.inputs[2]),
            detail_manifest_sha256=digest(self.inputs[3]),native_export_sha256=list(self.export_pins),attribution='notice')
        self.inputs[4].write_text(json.dumps(census,sort_keys=True))
        self.input_pins=tuple(digest(p) for p in self.inputs)
        self.cohort=dict(games=1,supported_games=1,excluded_game_ids=[],raw_totals=census['raw_totals'],
            supplied_credit_units=census['supplied_credit_units'])

    @staticmethod
    def csv_text(fields,rows):
        stream=io.StringIO();writer=csv.DictWriter(stream,fieldnames=fields)
        writer.writeheader();writer.writerows(rows);return stream.getvalue()

    def configured(self):
        from contextlib import ExitStack
        stack=ExitStack();stack.enter_context(patch.object(hit,'INPUT_SHA',self.input_pins))
        stack.enter_context(patch.object(hit,'EXPORT_SHA',self.export_pins,create=True))
        stack.enter_context(patch.object(hit,'COHORT',self.cohort,create=True))
        return stack

    def test_positive_controller_repeats_and_preserves_all_nine_inputs(self):
        with self.configured():
            first=hit.publish(*self.inputs,self.root/'first.json')
            repeat=hit.publish(*self.inputs,self.root/'repeat.json')
        self.assertEqual(first,repeat)
        self.assertEqual(first['totals']['qualified_events'],1)
        self.assertEqual((self.root/'first.json').read_bytes(),(self.root/'repeat.json').read_bytes())
        self.assertEqual(tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs),self.input_pins)
        self.assertEqual(tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.exports),self.export_pins)
        self.assertEqual(first['provenance']['census_sha256'],self.input_pins[4])

    def test_complete_native_disagreement_blocks_before_new_classification(self):
        self.exports[3].write_text('facts,rows\n')
        changed=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.exports)
        census=json.loads(self.inputs[4].read_text());census['provenance']['native_export_sha256']=list(changed)
        self.inputs[4].write_text(json.dumps(census,sort_keys=True))
        pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)
        with self.configured(),patch.object(hit,'INPUT_SHA',pins),patch.object(hit,'EXPORT_SHA',changed),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
            with self.assertRaisesRegex(ValueError,'Native runner group reconciliation failed'):
                hit.publish(*self.inputs,self.root/'mismatch.json')
            measured.assert_not_called()
        self.assertFalse((self.root/'mismatch.json').exists())

    def test_final_input_reread_blocks_report_after_classification(self):
        original=hit.classify
        def drift(*args,**kwargs):
            result=original(*args,**kwargs);self.inputs[3].write_text('drift');return result
        with self.configured(),patch.object(hit,'classify',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                hit.publish(*self.inputs,self.root/'drift.json')
        self.assertFalse((self.root/'drift.json').exists())

    def test_export_headers_block_before_any_new_classification(self):
        original_census=self.inputs[4].read_bytes()
        for path in self.exports:
            original=path.read_bytes();path.write_text('wrong,header\n')
            export_pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.exports)
            census=json.loads(original_census);census['provenance']['native_export_sha256']=list(export_pins)
            self.inputs[4].write_text(json.dumps(census,sort_keys=True))
            input_pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)
            with self.subTest(path=path),self.configured(),patch.object(hit,'INPUT_SHA',input_pins),patch.object(hit,'EXPORT_SHA',export_pins),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
                with self.assertRaises(ValueError):hit.publish(*self.inputs,self.root/('bad-'+path.stem+'.json'))
                measured.assert_not_called()
            path.write_bytes(original);self.inputs[4].write_bytes(original_census)

    def test_manifest_lineup_and_census_provenance_block_before_measurement(self):
        for index,key in [(3,'source_commit'),(2,'provenance'),(4,'provenance')]:
            path=self.inputs[index];original=path.read_bytes();data=json.loads(original);data[key]='unsupported'
            path.write_text(json.dumps(data,sort_keys=True))
            input_pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)
            with self.subTest(index=index),self.configured(),patch.object(hit,'INPUT_SHA',input_pins),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
                with self.assertRaisesRegex(ValueError,'provenance'):hit.publish(*self.inputs,self.root/f'bad-provenance-{index}.json')
                measured.assert_not_called()
            path.write_bytes(original)

    def test_census_query_and_validator_recipe_identity_block_before_measurement(self):
        original=self.inputs[4].read_bytes()
        for key in ['query_sha256','recipe_sha256']:
            census=json.loads(original);census['provenance'][key]=[]
            self.inputs[4].write_text(json.dumps(census,sort_keys=True))
            input_pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)
            with self.subTest(key=key),self.configured(),patch.object(hit,'INPUT_SHA',input_pins),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
                with self.assertRaisesRegex(ValueError,'provenance'):hit.publish(*self.inputs,self.root/f'bad-{key}.json')
                measured.assert_not_called()
            self.inputs[4].write_bytes(original)

    def test_exact_cohort_units_cannot_be_replaced_with_a_pinned_tiny_subset(self):
        for key,value in [('games',2),('supported_games',2),('supplied_credit_units',{})]:
            with self.subTest(key=key),self.configured(),patch.object(hit,'COHORT',dict(self.cohort,**{key:value})),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
                with self.assertRaisesRegex(ValueError,'cohort'):hit.publish(*self.inputs,self.root/f'bad-cohort-{key}.json')
                measured.assert_not_called()

    def test_final_export_reread_blocks_native_artifact_drift(self):
        original=hit.classify
        def drift(*args,**kwargs):
            result=original(*args,**kwargs);self.exports[2].write_text('drift');return result
        with self.configured(),patch.object(hit,'classify',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                hit.publish(*self.inputs,self.root/'export-drift.json')
        self.assertFalse((self.root/'export-drift.json').exists())

    def test_final_recipe_reread_does_not_mutate_real_repository_sources(self):
        repository=self.root/'synthetic-repository';audits=repository/'audits';audits.mkdir(parents=True)
        recipe=audits/'hit_advancement.py';recipe.write_bytes(Path(hit.__file__).read_bytes())
        original=hit.classify
        def drift(*args,**kwargs):
            result=original(*args,**kwargs);recipe.write_bytes(recipe.read_bytes()+b'\n');return result
        with self.configured(),patch.object(hit,'__file__',str(recipe)),patch.object(hit,'classify',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                hit.publish(*self.inputs,self.root/'recipe-drift.json')
        self.assertFalse((self.root/'recipe-drift.json').exists())

    def test_changed_census_body_blocks_before_qualification(self):
        census=json.loads(self.inputs[4].read_text());census['basis']='unsupported replacement'
        self.inputs[4].write_text(json.dumps(census,sort_keys=True))
        pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)
        with self.configured(),patch.object(hit,'INPUT_SHA',pins),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
            with self.assertRaisesRegex(ValueError,'census reconciliation'):
                hit.publish(*self.inputs,self.root/'census-body.json')
            measured.assert_not_called()

    def test_input_drift_during_export_read_blocks_before_measurement(self):
        original=hit.raw.read_export
        def drift(stream):
            rows=list(original(stream));self.inputs[3].write_text('drift');return iter(rows)
        with self.configured(),patch.object(hit.raw,'read_export',side_effect=drift),patch.object(hit,'classify',side_effect=AssertionError('qualification started')) as measured:
            with self.assertRaisesRegex(ValueError,'inputs changed during read'):
                hit.publish(*self.inputs,self.root/'read-drift.json')
            measured.assert_not_called()

    def test_final_query_reread_uses_private_copies_not_real_sql_mutation(self):
        audits=self.root/'private-queries';audits.mkdir()
        recipe=audits/'runner_transitions.py';recipe.write_bytes(Path(runner.__file__).read_bytes())
        names=['2019-lineup-readiness.sql','2019-outcomes.sql','2019-raw-mechanisms.sql','2019-runner-transitions.sql']
        for name in names:(audits/name).write_bytes(Path(runner.__file__).with_name(name).read_bytes())
        query=audits/names[-1];original=hit.classify
        def drift(*args,**kwargs):
            result=original(*args,**kwargs);query.write_bytes(query.read_bytes()+b'\n');return result
        with self.configured(),patch.object(runner,'__file__',str(recipe)),patch.object(hit,'classify',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                hit.publish(*self.inputs,self.root/'query-drift.json')
        self.assertFalse((self.root/'query-drift.json').exists())

    def test_created_output_between_measurement_and_publication_is_never_overwritten(self):
        output=self.root/'late-collision.json';original=hit.classify
        def collide(*args,**kwargs):
            result=original(*args,**kwargs);output.write_bytes(b'other writer');return result
        with self.configured(),patch.object(hit,'classify',side_effect=collide):
            with self.assertRaises(FileExistsError):hit.publish(*self.inputs,output)
        self.assertEqual(output.read_bytes(),b'other writer')

    def test_parent_redirect_before_publication_cannot_create_report_in_repository(self):
        repository=self.root/'synthetic-repository';audits=repository/'audits';audits.mkdir(parents=True)
        recipe=audits/'hit_advancement.py';recipe.write_bytes(Path(hit.__file__).read_bytes())
        parent=self.root/'output-directory';parent.mkdir();output=parent/'report.json';original=hit.classify
        def redirect(*args,**kwargs):
            result=original(*args,**kwargs);parent.rmdir();parent.symlink_to(repository,target_is_directory=True);return result
        with self.configured(),patch.object(hit,'__file__',str(recipe)),patch.object(hit,'classify',side_effect=redirect):
            with self.assertRaisesRegex(ValueError,'output path changed'):
                hit.publish(*self.inputs,output)
        self.assertFalse((repository/'report.json').exists())

    def test_cli_help_names_only_local_accepted_inputs_and_new_output(self):
        result=subprocess.run([sys.executable,'-Werror','-m','audits.hit_advancement','--help'],text=True,capture_output=True,check=True)
        for name in ['snapshot','supplement','lineups','manifest','census','output']:self.assertIn(name,result.stdout)
        self.assertEqual(result.stderr,'')


if __name__=='__main__':
    unittest.main()
