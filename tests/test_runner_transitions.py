"""Literal runner opportunities are not causal scoring or physical ability estimates."""
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
from audits import runner_transitions as runner
from audits import outcome_inventory as outcome
from audits import lineup_readiness as lineup
from audits import raw_mechanisms as raw_mechanisms
from tests.test_lineup_readiness import fixture as lineup_fixture
from types import SimpleNamespace
from tests.test_joint_contact import fixture


def child(role='First', identity='F1', **changes):
    value=dict(baserunner=role,runner_id=identity,baserunning_play_type='Advance',
        is_out=False,run_scored_flag=False,explicit_out_flag=False,
        advanced_on_error_flag=False,attempted_advance_to_base=None,
        base_end=role,is_advance_attempt=False)
    value.update(changes)
    return value


def attached(pair,children):
    raw,context=copy.deepcopy(pair)
    raw['runners']=json.dumps(children)
    context['runner_rows']=str(len(children))
    for name in outcome.RUNNER_UNITS+outcome.OPPORTUNITY_UNITS:context[name]='0'
    context['runner_runs']=str(sum(c['run_scored_flag'] for c in children))
    context['runner_outs']=str(sum(c['is_out'] for c in children))
    return raw,context


def report(pairs,supported=('GAME1',)):
    return runner.inventory([p[0] for p in pairs],[p[1] for p in pairs],list(supported))


class OpportunityTests(unittest.TestCase):
    def test_absent_child_does_not_turn_into_a_held_endpoint(self):
        value=report([fixture(mask=1)])
        self.assertEqual(value['format'],'2019-runner-transitions-v1')
        self.assertEqual(value['totals']['opportunities'],2)
        self.assertEqual(value['totals']['before_runner_opportunities'],1)
        self.assertEqual(value['totals']['batter_opportunities'],1)
        self.assertEqual({c['role'] for c in value['cells']},{'Batter','First'})
        self.assertTrue(all(c['endpoint_status']=='missing_row' and c['base_end'] is None for c in value['cells']))
        self.assertEqual(sum(c['opportunities'] for c in value['cells']),2)

    def test_explicit_hold_null_flags_and_unknown_endpoints_are_distinct(self):
        for endpoint,attempt,error in [('First',False,False),(None,None,None),('Unmapped',True,None)]:
            pair=attached(fixture(mask=1),[child(base_end=endpoint,is_advance_attempt=attempt,advanced_on_error_flag=error)])
            with self.subTest(endpoint=endpoint):
                value=report([pair]);cell=next(c for c in value['cells'] if c['role']=='First')
                self.assertEqual(cell['base_end'],endpoint)
                self.assertEqual(cell['is_advance_attempt'],attempt)
                self.assertEqual(cell['advanced_on_error_flag'],error)
                self.assertEqual(cell['endpoint_status'],{'First':'observed',None:'missing_endpoint','Unmapped':'unknown_endpoint'}[endpoint])
                self.assertEqual(value['totals']['matching_child_rows'],1)
                self.assertEqual(value['totals']['opportunities'],2)

    def test_all_before_masks_outs_and_joint_vectors_conserve_opportunities(self):
        pairs=[fixture(key=i+1,mask=m,outs=o) for i,(m,o) in enumerate(itertools.product(range(8),range(3)))]
        value=report(pairs)
        self.assertEqual(value['totals']['real_events'],24)
        self.assertEqual(value['totals']['before_runner_opportunities'],36)
        self.assertEqual(value['totals']['batter_opportunities'],24)
        self.assertEqual(value['totals']['opportunities'],60)
        self.assertEqual(sum(c['opportunities'] for c in value['cells']),60)
        self.assertEqual(sum(c['events'] for c in value['joint_vectors']),24)
        self.assertEqual(value,report(list(reversed(pairs))))
        for cell in value['joint_vectors']:
            self.assertEqual(len(cell['runners']),cell['base_mask'].bit_count()+1)

    def test_known_identity_conflict_and_invalid_nullable_flag_types_fail(self):
        for changes in [dict(runner_id='OTHER'),dict(is_advance_attempt=0),dict(advanced_on_error_flag='false'),dict(explicit_out_flag=[])] :
            pair=attached(fixture(mask=1),[child(**changes)])
            with self.subTest(changes=changes),self.assertRaises(ValueError):report([pair])
        pair=attached(fixture(mask=1),[child(runner_id=None)])
        cell=next(c for c in report([pair])['cells'] if c['role']=='First')
        self.assertEqual(cell['endpoint_status'],'missing_identity')

    def test_source_partition_and_reserved_hash_split_are_not_result_dependent(self):
        pairs=[fixture(key=i+1) for i in range(3)]
        for game,pair in zip(['SYNTHETIC_A','SYNTHETIC_B','SYNTHETIC_EXCLUDED'],pairs):
            pair[0]['game_id']=pair[1]['game_id']=game
        value=report(pairs,['SYNTHETIC_A','SYNTHETIC_B'])
        self.assertEqual(value['games'],3)
        self.assertEqual(value['supported_games'],2)
        self.assertEqual(value['excluded_game_ids'],['SYNTHETIC_EXCLUDED'])
        for game in ['SYNTHETIC_A','SYNTHETIC_B']:
            split='heldout' if hashlib.sha256(game.encode()).hexdigest()[0] in '012' else 'training'
            self.assertIn(game,value['splits'][split]['game_ids'])
        self.assertEqual(value['splits']['excluded']['real_events'],1)
        for bad in [['SYNTHETIC_A','SYNTHETIC_A'],['UNSEEN']]:
            with self.subTest(bad=bad),self.assertRaises(ValueError):report(pairs,bad)

    def test_no_play_child_facts_remain_raw_without_creating_opportunities(self):
        real=fixture(mask=1)
        np=dict(real[0],event_key='2',event_id='2',no_play_flag='true',result='',outs='0',runs='0',credits='[]',raw_play='NP',runners=json.dumps([child(base_end='Second',is_advance_attempt=None)]))
        value=runner.inventory([real[0],np],[real[1]],['GAME1'])
        self.assertEqual(value['totals']['opportunities'],2)
        self.assertEqual(value['totals']['no_play_events'],1)
        self.assertEqual(value['totals']['no_play_child_rows'],1)
        self.assertEqual(sum(c['rows'] for c in value['raw_child_cells']),1)
        cell=value['raw_child_cells'][0]
        self.assertTrue(cell['no_play'])
        self.assertEqual(cell['base_end'],'Second')
        self.assertIsNone(cell['is_advance_attempt'])
        self.assertIsNone(cell['base_mask'])
        self.assertIsNone(cell['outs_before'])

    def test_runner_only_batter_child_is_observed_not_a_pa_opportunity(self):
        raw,context=fixture(mask=1)
        raw.update(result='',outs='0',trajectory='',angle='',credits='[]',raw_play='SB2.1-2')
        context.update(result='',pa='',batting_outs='',outs='0',state_outs='0',putouts='0',after_outs='0',after_base_mask='3',trajectory='',angle='')
        pair=attached((raw,context),[child(base_end='Second',is_advance_attempt=True,attempted_advance_to_base='Second'),child('Batter','BATTER',base_end='First')])
        value=report([pair])
        self.assertEqual(value['totals']['non_pa'],1)
        self.assertEqual(value['totals']['terminal_pa'],0)
        self.assertEqual(value['totals']['batter_opportunities'],0)
        self.assertEqual(value['totals']['opportunities'],1)
        self.assertEqual(value['totals']['matching_child_rows'],1)
        self.assertEqual(value['totals']['extra_child_rows'],1)
        self.assertEqual({c['role'] for c in value['cells']},{'First'})
        self.assertEqual({c['baserunner'] for c in value['raw_child_cells']},{'First','Batter'})

    def test_home_without_counted_run_and_normalized_third_out_are_not_retirements(self):
        pair=attached(fixture(mask=4,outs=2),[child('Third','T1',base_end='Home')])
        value=report([pair]);cell=next(c for c in value['cells'] if c['role']=='Third')
        self.assertEqual(cell['base_end'],'Home')
        self.assertFalse(cell['is_out'])
        self.assertFalse(cell['run_scored_flag'])
        self.assertTrue(cell['frame_end'])
        self.assertEqual(value['findings']['home_without_counted_run'],1)
        self.assertEqual(value['totals']['before_runner_opportunities'],1)

    def test_correlated_scoring_vector_is_not_independent_station_advances(self):
        raw,context=fixture(mask=6)
        raw.update(result='Single',outs='0',runs='2',raw_play='S7.2-H;3-H;B-1',credits='[]')
        context.update(result='Single',outs='0',runs='2',state_outs='0',state_runs='2',batting_outs='0',putouts='0',after_outs='0',after_base_mask='1')
        children=[child('Batter','BATTER',base_end='First'),child('Second','S1',base_end='Home',run_scored_flag=True),child('Third','T1',base_end='Home',run_scored_flag=True)]
        pair=attached((raw,context),children);value=report([pair])
        self.assertEqual(value['totals']['opportunities'],3)
        self.assertEqual(value['joint_vectors'][0]['events'],1)
        self.assertEqual([r['base_end'] for r in value['joint_vectors'][0]['runners']],['First','Home','Home'])
        shuffled=attached((raw,context),list(reversed(children)))
        self.assertEqual(value,report([shuffled]))
        self.assertEqual(sum(c['rows'] for c in value['raw_child_cells']),3)

    def test_unknown_attempt_and_out_endpoint_contradiction_remain_findings(self):
        pair=attached(fixture(mask=1),[child(base_end='Out',attempted_advance_to_base='Unmapped',is_advance_attempt=None)])
        value=report([pair]);cell=next(c for c in value['cells'] if c['role']=='First')
        self.assertFalse(cell['is_out'])
        self.assertEqual(cell['base_end'],'Out')
        self.assertEqual(cell['attempted_advance_to_base'],'Unmapped')
        self.assertEqual(value['findings']['unknown_attempt_label'],1)
        self.assertEqual(value['findings']['out_without_retirement'],1)

    def test_compound_flags_annotations_unknown_lexing_and_literal_pa_are_retained(self):
        pairs=[fixture(key=1,mask=1),fixture(key=2)]
        pairs[0][0].update(raw_play='S7+E4/TH.1-3',flags=json.dumps([dict(sequence_id=1,flag='UnmappedFlag')]))
        pairs[1][0]['raw_play']='S7..1-3'
        value=report(pairs)
        first=next(c for c in value['joint_vectors'] if c['base_mask']==1)
        self.assertTrue(first['pa'])
        self.assertTrue(first['compound'])
        self.assertEqual(first['modifiers'],['TH'])
        self.assertEqual(first['source_flags'],['UnmappedFlag'])
        self.assertEqual(value['raw_notation']['source_flags'],{'UnmappedFlag':1})
        self.assertEqual(value['raw_notation']['root_families']['error_notation'],1)
        self.assertEqual(value['raw_notation']['structural_unknown_reasons'],{'multiple_advance_delimiters':1})
        self.assertEqual(value['raw_notation']['structural_unknown_events'][0]['event']['raw_play'],'S7..1-3')
        self.assertEqual(value,report(list(reversed(pairs))))

    def test_complete_existing_join_and_scope_validators_are_not_bypassed(self):
        raw,context=fixture(mask=1)
        for bad in [dict(context,base_mask='0'),dict(context,outs_before='3'),dict(context,season='2018')]:
            with self.subTest(bad=bad),self.assertRaises(ValueError):runner.inventory([raw],[bad],['GAME1'])
        for bad in [dict(raw,credits='[]'),dict(raw,flags='null'),dict(raw,game_id='OTHER')]:
            with self.subTest(bad=bad),self.assertRaises(ValueError):runner.inventory([bad],[context],['GAME1'])
        for raws,contexts in [([raw,raw],[context]),([raw],[context,context]),([],[context]),([raw],[])]:
            with self.subTest(raws=raws,contexts=contexts),self.assertRaises(ValueError):runner.inventory(raws,contexts,['GAME1'])


class PublicationTests(unittest.TestCase):
    def test_cli_names_only_local_inputs_and_new_output(self):
        result=subprocess.run([sys.executable,'-Werror','-m','audits.runner_transitions','--help'],text=True,capture_output=True,check=True)
        for name in ['snapshot','supplement','lineups','manifest','output']:self.assertIn(name,result.stdout)
        self.assertEqual(result.stderr,'')

    def test_repository_output_refuses_before_missing_input_read(self):
        output=Path(__file__).resolve().parents[1]/'runner-census-forbidden.json'
        with self.assertRaisesRegex(ValueError,'outside repository'):
            runner.publish(*(['missing']*4),output)
        self.assertFalse(output.exists())

    def test_report_native_and_query_collisions_refuse_before_input_reads(self):
        for suffix in ['.json','.events.csv','.groups.csv','.raw.stderr','.lineup.query.sql']:
            with self.subTest(suffix=suffix),tempfile.TemporaryDirectory() as d:
                output=Path(d)/'report.json';collision=output.with_suffix(suffix);collision.write_text('keep')
                with self.assertRaisesRegex(ValueError,'Refusing to overwrite'):
                    runner.publish(*(['missing']*4),output)
                self.assertEqual(collision.read_text(),'keep')
                self.assertEqual(list(Path(d).iterdir()),[collision])

    def test_unaccepted_input_pins_refuse_before_native_or_outputs(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);inputs=[root/str(n) for n in range(4)]
            for p in inputs:p.write_bytes(b'synthetic unaccepted')
            with patch('subprocess.run',side_effect=AssertionError('native query started')) as query:
                with self.assertRaisesRegex(ValueError,'Unsupported.*digest'):runner.publish(*inputs,root/'out.json')
                query.assert_not_called()
            self.assertEqual(set(root.iterdir()),set(inputs))

    def test_final_reread_blocks_drift_without_a_report(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);source=root/'source';source.write_bytes(b'stable')
            pins=[hashlib.sha256(source.read_bytes()).hexdigest()];source.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                runner.write_report({'observed':True},[source],pins,root/'out.json')
            self.assertFalse((root/'out.json').exists())

    def test_sorted_report_is_exclusive_and_does_not_mutate_input(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);source=root/'source';source.write_bytes(b'stable')
            pins=[hashlib.sha256(source.read_bytes()).hexdigest()];output=root/'out.json'
            report={'z':None,'a':0};runner.write_report(report,[source],pins,output)
            original=output.read_bytes()
            self.assertEqual(original,(json.dumps(report,sort_keys=True,indent=2,allow_nan=False)+'\n').encode())
            self.assertEqual(report,{'z':None,'a':0})
            with self.assertRaises(FileExistsError):runner.write_report({'bad':1},[source],pins,output)
            self.assertEqual(output.read_bytes(),original)


class PublisherControllerTests(unittest.TestCase):
    def setUp(self):
        temp=tempfile.TemporaryDirectory();self.addCleanup(temp.cleanup)
        self.root=Path(temp.name)
        self.paths=[self.root/name for name in ['snapshot','detail','lineups.json','manifest.json']]
        self.paths[0].write_bytes(b'synthetic snapshot');self.paths[1].write_bytes(b'synthetic detail')
        self.pair=attached(fixture(mask=1),[child(base_end='Second')])
        self.parts=lineup_fixture(game_id='GAME1')
        self.snapshot_manifest=dict(schema_version='f03-spike-1',query_version='2019-filtered-extract-v1',catalog_sha256=lineup.CATALOG,source_url='synthetic local',extracted_at='synthetic',attribution='notice')
        pins=[hashlib.sha256(p.read_bytes()).hexdigest() for p in self.paths[:2]]
        lr=lineup.inventory(*self.parts)
        lr['provenance']=dict(snapshot_sha256=pins[0],supplement_sha256=pins[1],attribution='notice')
        detail=dict(format='outcome-detail-2019-v1',source_commit=lineup.SOURCE_COMMIT,catalog_sha256=lineup.CATALOG,snapshot_sha256=pins[0],database_sha256=pins[1],snapshot_id=857,attribution='notice')
        for p,value in zip(self.paths[2:],[lr,detail]):p.write_text(json.dumps(value,sort_keys=True))
        self.pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.paths)
        cohort_raw=raw_mechanisms.inventory([self.pair[0]])
        self.cohort=dict(games=1,supported_games=1,excluded_game_ids=[],raw_totals=cohort_raw['totals'],supplied_credit_units=cohort_raw['supplied_credit_units'])
        records=[dict(record_type=kind,payload=json.dumps(value,sort_keys=True)) for kind,values in [('game',self.parts[0]),('person',self.parts[1]),('appearance',self.parts[2]),('manifest',[self.snapshot_manifest])] for value in values]
        cell=report([self.pair])['raw_child_cells'][0].copy();n=cell.pop('rows')
        self.outputs=dict(lineup=self.csv_text(['record_type','payload'],records),
            events=self.csv_text(list(self.pair[1]),[self.pair[1]]),
            raw=self.csv_text(list(self.pair[0]),[self.pair[0]]),
            groups=ReconciliationTests.encoded([(json.dumps(cell),n)]))

    @staticmethod
    def csv_text(fields,rows):
        stream=io.StringIO();writer=csv.DictWriter(stream,fieldnames=fields)
        writer.writeheader();writer.writerows(rows);return stream.getvalue()

    def query(self,args,**kw):
        self.assertTrue(all(v in args for v in ['-readonly','-init','/dev/null','-bail','-batch','-csv','-nullvalue']))
        self.assertEqual(args[args.index('-nullvalue')+1],'')
        self.assertIn('enable_external_access=false',' '.join(args).lower())
        # Fixed audit recipes mention the prohibition in -- comments; check
        # executable recipe lines, not that explanatory prose.
        active_sql='\n'.join(line.split('--',1)[0] for line in kw['input'].splitlines())
        self.assertNotIn('COPY',active_sql.upper())
        label=Path(kw['stdout'].name).suffixes[-2][1:]
        kw['stdout'].write(self.outputs[label]);return SimpleNamespace(returncode=0)

    def configured(self):
        from contextlib import ExitStack
        stack=ExitStack();stack.enter_context(patch.object(runner,'INPUT_SHA',self.pins))
        stack.enter_context(patch.object(runner,'COHORT',self.cohort,create=True))
        stack.enter_context(patch('subprocess.run',side_effect=self.query))
        return stack

    def test_complete_controller_repeats_and_preserves_inputs(self):
        with self.configured():
            first=runner.publish(*self.paths,self.root/'first.json')
            repeat=runner.publish(*self.paths,self.root/'repeat.json')
        self.assertEqual(first,repeat)
        self.assertEqual((self.root/'first.json').read_bytes(),(self.root/'repeat.json').read_bytes())
        self.assertEqual(tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.paths),self.pins)
        self.assertEqual(first['totals']['opportunities'],2)
        self.assertEqual(first['provenance']['snapshot_sha256'],self.pins[0])
        self.assertEqual(first['provenance']['supplement_sha256'],self.pins[1])
        self.assertEqual(len(first['provenance']['native_export_sha256']),4)
        self.assertTrue(all((self.root/f'first.{kind}.query.sql').exists() for kind in self.outputs))

    def test_native_disagreement_prevents_measurement_and_report(self):
        self.outputs['groups']=ReconciliationTests.encoded([])
        with self.configured(),patch.object(runner,'_measure',side_effect=AssertionError('measurement started')) as measured:
            with self.assertRaisesRegex(ValueError,'Native runner group reconciliation failed'):
                runner.publish(*self.paths,self.root/'bad.json')
            measured.assert_not_called()
        self.assertFalse((self.root/'bad.json').exists())
        self.assertTrue((self.root/'bad.groups.csv').exists())

    def test_native_failure_preserves_raw_error_without_report(self):
        def failure(args,**kw):
            self.query(args,**kw);kw['stderr'].write('synthetic native failure\n');return SimpleNamespace(returncode=9)
        with self.configured(),patch('subprocess.run',side_effect=failure):
            with self.assertRaisesRegex(RuntimeError,'Native runner query failed'):
                runner.publish(*self.paths,self.root/'failed.json')
        self.assertFalse((self.root/'failed.json').exists())
        self.assertEqual((self.root/'failed.lineup.stderr').read_text(),'synthetic native failure\n')

    def test_bad_detail_and_lineup_provenance_refuse_before_queries(self):
        for index,key in [(3,'source_commit'),(2,'provenance')]:
            with self.subTest(index=index):
                original=self.paths[index].read_bytes();value=json.loads(original);value[key]='unsupported'
                self.paths[index].write_text(json.dumps(value))
                pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.paths)
                with self.configured(),patch.object(runner,'INPUT_SHA',pins),patch('subprocess.run',side_effect=AssertionError('query started')) as query:
                    with self.assertRaisesRegex(ValueError,'provenance'):runner.publish(*self.paths,self.root/f'bad-provenance{index}.json')
                    query.assert_not_called()
                self.paths[index].write_bytes(original)
                self.assertFalse((self.root/f'bad-provenance{index}.json').exists())

    def test_invalid_export_headers_block_measurement(self):
        for kind in self.outputs:
            with self.subTest(kind=kind):
                original=self.outputs[kind];self.outputs[kind]='wrong,header\n'
                with self.configured(),patch.object(runner,'_measure',side_effect=AssertionError('measurement started')) as measured:
                    with self.assertRaises(ValueError):runner.publish(*self.paths,self.root/f'bad-header-{kind}.json')
                    measured.assert_not_called()
                self.outputs[kind]=original
                self.assertFalse((self.root/f'bad-header-{kind}.json').exists())

    def test_source_cohort_units_block_before_measurement(self):
        for key,value in [('games',2),('supported_games',2),('supplied_credit_units',{})]:
            with self.subTest(key=key),self.configured(),patch.object(runner,'COHORT',dict(self.cohort,**{key:value})),patch.object(runner,'_measure',side_effect=AssertionError('measurement started')) as measured:
                with self.assertRaisesRegex(ValueError,'source cohort/units'):runner.publish(*self.paths,self.root/f'bad-cohort-{key}.json')
                measured.assert_not_called()
                self.assertFalse((self.root/f'bad-cohort-{key}.json').exists())

    def test_unchanged_complete_validator_failure_blocks_publication(self):
        broken=dict(self.pair[0],runs='1')
        self.outputs['raw']=self.csv_text(list(broken),[broken])
        with self.configured(),patch.object(runner,'_measure',side_effect=AssertionError('measurement started')) as measured:
            with self.assertRaises(ValueError):runner.publish(*self.paths,self.root/'broken-join.json')
            measured.assert_not_called()
        self.assertFalse((self.root/'broken-join.json').exists())

    def test_final_query_and_recipe_reread_blocks_drift(self):
        original_file=Path(runner.__file__)
        # Preserve the module's repository/audits layout so the real output-
        # origin guard still sees the external reports as outside the toy repo.
        local=self.root/'synthetic-repository'/'audits';local.mkdir(parents=True)
        recipe=local/original_file.name;recipe.write_bytes(original_file.read_bytes())
        names=['2019-lineup-readiness.sql','2019-outcomes.sql','2019-raw-mechanisms.sql','2019-runner-transitions.sql']
        for name in names:(local/name).write_bytes(original_file.with_name(name).read_bytes())
        original=runner._measure
        for label,target in [('query',local/names[-1]),('recipe',recipe)]:
            before=target.read_bytes()
            def drift(*args,**kw):
                value=original(*args,**kw);target.write_bytes(before+b'\n');return value
            with self.subTest(label=label),self.configured(),patch.object(runner,'__file__',str(recipe)),patch.object(runner,'_measure',side_effect=drift):
                with self.assertRaisesRegex(ValueError,'inputs changed'):runner.publish(*self.paths,self.root/f'{label}-drift.json')
            self.assertFalse((self.root/f'{label}-drift.json').exists())
            target.write_bytes(before)

    def test_final_recipe_input_drift_prevents_report(self):
        original=runner._measure
        def drift(*args,**kw):
            value=original(*args,**kw);self.paths[3].write_text('changed');return value
        with self.configured(),patch.object(runner,'_measure',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):
                runner.publish(*self.paths,self.root/'drift.json')
        self.assertFalse((self.root/'drift.json').exists())


class ReconciliationTests(unittest.TestCase):
    def setUp(self):
        self.pair=attached(fixture(mask=1),[child(base_end='Second')])
        cell=report([self.pair])['raw_child_cells'][0].copy()
        self.count=cell.pop('rows')
        self.facts=cell

    @staticmethod
    def encoded(rows):
        stream=io.StringIO();writer=csv.writer(stream)
        writer.writerow(['facts','rows']);writer.writerows(rows)
        return stream.getvalue()

    def test_strict_native_group_reader_preserves_nullable_values_and_grain(self):
        text=self.encoded([(json.dumps(self.facts),self.count)])
        groups=runner.read_native_groups(text)
        self.assertEqual(groups,{json.dumps(self.facts,sort_keys=True):self.count})
        self.assertEqual(groups,runner.read_native_groups(self.encoded([(json.dumps(dict(reversed(list(self.facts.items())))),self.count)])))

    def test_native_reader_rejects_headers_duplicates_missing_and_invalid_types(self):
        good=json.dumps(self.facts)
        cases=['facts,rows,rows\n',self.encoded([(good,0)]),self.encoded([(good,-1)]),
               self.encoded([(good,'1.0')]),self.encoded([(good,1),(good,1)]),
               self.encoded([(json.dumps(dict(self.facts,is_out=0)),1)]),
               self.encoded([(json.dumps(dict(self.facts,base_mask=1.5)),1)]),
               self.encoded([(json.dumps(dict(self.facts,identity_present=None)),1)]),
               self.encoded([(json.dumps(dict(self.facts,base_end=0)),1)]),
               self.encoded([(good.replace('"no_play": false','"no_play": false, "no_play": false'),1)]),
               self.encoded([(json.dumps({k:v for k,v in self.facts.items() if k!='base_end'}),1)])]
        for text in cases:
            with self.subTest(text=text),self.assertRaises(ValueError):runner.read_native_groups(text)

    def test_native_mismatch_prevents_opportunity_measurement(self):
        with patch.object(runner,'_measure',create=True,side_effect=AssertionError('opportunity measurement started')) as measured:
            with self.assertRaisesRegex(ValueError,'Native runner group reconciliation failed'):
                runner.inventory([self.pair[0]],[self.pair[1]],['GAME1'],native_groups={})
            measured.assert_not_called()

    def test_matching_native_groups_leave_complete_report_exact(self):
        groups={json.dumps(self.facts,sort_keys=True):self.count}
        self.assertEqual(report([self.pair]),runner.inventory([self.pair[0]],[self.pair[1]],['GAME1'],native_groups=groups))


class NativeGroupTests(unittest.TestCase):
    def setUp(self):
        temp=tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.directory=Path(temp.name)
        self.snapshot=self.directory/'snapshot.duckdb'
        self.detail=self.directory/'detail.duckdb'
        self.query=Path(__file__).resolve().parents[1]/'audits/2019-runner-transitions.sql'
        self.sql(self.snapshot,"""CREATE TABLE events(event_key BIGINT,game_id VARCHAR,no_play_flag BOOLEAN,plate_appearance_result VARCHAR,outs INTEGER,base_state INTEGER,outs_on_play INTEGER,runs_on_play INTEGER,batted_trajectory VARCHAR);
        INSERT INTO events VALUES(1,'SYNTHETIC_A',false,'InPlayOut',0,1,1,0,'Fly'),(2,'SYNTHETIC_A',true,NULL,NULL,NULL,0,0,NULL);""")
        self.sql(self.detail,"""CREATE TABLE selected_events(event_key BIGINT,game_id VARCHAR,event_id INTEGER,no_play_flag BOOLEAN,plate_appearance_result VARCHAR,outs_on_play INTEGER,runs_on_play INTEGER,batted_trajectory VARCHAR);
        INSERT INTO selected_events VALUES(1,'SYNTHETIC_A',1,false,'InPlayOut',1,0,'Fly'),(2,'SYNTHETIC_A',2,true,NULL,0,0,NULL);
        CREATE TABLE stg_event_baserunners(event_key BIGINT,game_id VARCHAR,event_id INTEGER,baserunner VARCHAR,runner_id VARCHAR,baserunning_play_type VARCHAR,is_out BOOLEAN,run_scored_flag BOOLEAN,explicit_out_flag BOOLEAN,advanced_on_error_flag BOOLEAN,attempted_advance_to_base VARCHAR,base_end VARCHAR,is_advance_attempt BOOLEAN);
        INSERT INTO stg_event_baserunners VALUES(1,'SYNTHETIC_A',1,'First','F1','Advance',false,false,false,NULL,'Second','Second',true),(2,'SYNTHETIC_A',2,'First',NULL,'Advance',false,false,NULL,NULL,NULL,'Unmapped',NULL);""")

    @staticmethod
    def sql(db,sql):
        return subprocess.run(['duckdb','-init','/dev/null','-bail','-batch',str(db)],input=sql,text=True,capture_output=True,check=True)

    def execute(self):
        setup=f"ATTACH '{self.detail}' AS detail (READ_ONLY); SET enable_external_access=false;"
        return subprocess.run(['duckdb','-init','/dev/null','-readonly','-bail','-batch','-csv','-nullvalue','','-cmd',setup,str(self.snapshot),'-f',str(self.query)],text=True,capture_output=True,check=False)

    def test_native_groups_preserve_source_nulls_contexts_and_readonly_bytes(self):
        before=[hashlib.sha256(p.read_bytes()).hexdigest() for p in (self.snapshot,self.detail)]
        result=self.execute()
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(result.stderr,'')
        rows=list(csv.DictReader(io.StringIO(result.stdout)))
        self.assertEqual(set(rows[0]),{'facts','rows'})
        cells=[dict(json.loads(r['facts']),rows=int(r['rows'])) for r in rows]
        self.assertEqual(sum(c['rows'] for c in cells),2)
        real=next(c for c in cells if not c['no_play'])
        self.assertEqual((real['base_mask'],real['outs_before']),(1,0))
        self.assertEqual(real['base_end'],'Second')
        self.assertTrue(real['identity_present'])
        self.assertIsNone(real['advanced_on_error_flag'])
        self.assertTrue(real['is_advance_attempt'])
        np=next(c for c in cells if c['no_play'])
        self.assertFalse(np['identity_present'])
        self.assertIsNone(np['base_mask'])
        self.assertIsNone(np['outs_before'])
        self.assertIsNone(np['is_advance_attempt'])
        self.assertEqual(np['base_end'],'Unmapped')
        self.assertEqual(real['split'],'heldout' if hashlib.sha256(b'SYNTHETIC_A').hexdigest()[0] in '012' else 'training')
        self.assertEqual(result.stdout,self.execute().stdout)
        self.assertEqual(before,[hashlib.sha256(p.read_bytes()).hexdigest() for p in (self.snapshot,self.detail)])

    def test_literal_null_identity_is_not_sql_null(self):
        self.sql(self.detail,"UPDATE stg_event_baserunners SET runner_id='NULL' WHERE event_key=2;")
        result=self.execute();self.assertEqual(result.returncode,0,result.stderr)
        cells=[json.loads(r['facts']) for r in csv.DictReader(io.StringIO(result.stdout))]
        self.assertTrue(next(c for c in cells if c['no_play'])['identity_present'])

    def test_native_guard_rejects_snapshot_fanout(self):
        self.sql(self.snapshot,'INSERT INTO events SELECT * FROM events WHERE event_key=1;')
        result=self.execute();self.assertNotEqual(result.returncode,0)
        self.assertIn('unsupported runner transition group facts',result.stderr)

    def test_native_guard_rejects_orphan_runner_row(self):
        self.sql(self.detail,'UPDATE stg_event_baserunners SET event_key=999 WHERE event_key=1;')
        result=self.execute();self.assertNotEqual(result.returncode,0)
        self.assertIn('unsupported runner transition group facts',result.stderr)

    def test_guard_still_executes_when_empty_children_would_hide_parent_mismatch(self):
        self.sql(self.detail,'DELETE FROM stg_event_baserunners;')
        self.sql(self.snapshot,"UPDATE events SET game_id='OTHER' WHERE event_key=1;")
        result=self.execute();self.assertNotEqual(result.returncode,0)
        self.assertIn('unsupported runner transition group facts',result.stderr)

    def test_native_guard_rejects_duplicate_role_and_missing_required_units(self):
        self.sql(self.detail,'INSERT INTO stg_event_baserunners SELECT * FROM stg_event_baserunners WHERE event_key=1;')
        result=self.execute();self.assertNotEqual(result.returncode,0)
        self.assertIn('unsupported runner transition group facts',result.stderr)
        self.sql(self.detail,'DELETE FROM stg_event_baserunners WHERE event_key=1; UPDATE stg_event_baserunners SET is_out=NULL;')
        result=self.execute();self.assertNotEqual(result.returncode,0)
        self.assertIn('unsupported runner transition group facts',result.stderr)


if __name__=='__main__':unittest.main()
