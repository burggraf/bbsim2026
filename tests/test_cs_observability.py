"""Literal missing-field observations cannot fill attempted destinations."""
import copy
import itertools
import json
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
from tests import test_runner_only_steals as steal_tests
from audits import cs_observability as cs
from tests.test_runner_only_steals import fixture
from tests.test_joint_contact import fixture as contact_fixture
from tests.test_runner_transitions import attached


class ObservationTests(unittest.TestCase):
    def test_null_target_and_three_attempt_flags_remain_distinct(self):
        for target, attempt in itertools.product((None, 'Second'), (None, False, True)):
            row, context = fixture('CaughtStealing')
            children = json.loads(row['runners'])
            children[0].update(attempted_advance_to_base=target, is_advance_attempt=attempt)
            row['runners'] = json.dumps(children)
            before = copy.deepcopy((row, context))
            value = cs.observe(row, context)
            with self.subTest(target=target, attempt=attempt):
                self.assertEqual(value['children'], children)
                self.assertEqual(value['labels'], ['CaughtStealing'])
                self.assertEqual(value['missing']['children'], 0)
                self.assertEqual(value['missing']['flags']['is_advance_attempt'], int(attempt is None))
                self.assertEqual((row, context), before)

    def test_missing_held_row_is_not_unknown_flags_on_an_observed_row(self):
        row, context = fixture('CaughtStealing', mask=5)
        row['runners'] = json.dumps(json.loads(row['runners'])[:1])
        value = cs.observe(row, context)
        self.assertEqual(value['missing'], dict(children=1, identities=0, endpoints=0,
            flags={name: 0 for name in cs.FLAGS}))
        self.assertEqual(value['qualification']['reasons'], ['incomplete_vector'])

    def test_both_labels_multiple_children_and_order_do_not_merge_events(self):
        row, context = fixture('CaughtStealing', mask=5)
        children = json.loads(row['runners'])
        children[1]['baserunning_play_type'] = 'PickedOffCaughtStealing'
        row['runners'] = json.dumps(children)
        value = cs.observe(row, context)
        self.assertEqual(value['labels'], ['CaughtStealing', 'PickedOffCaughtStealing'])
        self.assertEqual(len(value['children']), 2)
        row['runners'] = json.dumps(list(reversed(children)))
        self.assertEqual(cs.observe(row, context), value)

    def test_no_play_has_no_invented_before_vector_or_qualification(self):
        row, _ = fixture('CaughtStealing')
        row['no_play_flag'] = 'true'
        value = cs.observe(row, None)
        self.assertIsNone(value['missing'])
        self.assertIsNone(value['qualification'])
        self.assertEqual(value['labels'], ['CaughtStealing'])

    def test_present_unknowns_and_absent_rows_are_separate_literal_counts(self):
        row, context = fixture('CaughtStealing', mask=5)
        children=json.loads(row['runners'])
        children[0].update(runner_id=None,base_end=None,explicit_out_flag=None,advanced_on_error_flag=None)
        row['runners']=json.dumps(children[:1])
        value=cs.observe(row,context)
        self.assertEqual(value['missing'],dict(children=1,identities=1,endpoints=1,
            flags=dict(is_out=0,run_scored_flag=0,explicit_out_flag=1,advanced_on_error_flag=1,is_advance_attempt=0)))
        self.assertIsNone(value['children'][0]['runner_id'])
        self.assertIsNone(value['children'][0]['base_end'])


class InventoryTests(unittest.TestCase):
    def pairs(self):
        first = fixture('CaughtStealing', key=1)
        second = fixture('CaughtStealing', key=2)
        values = json.loads(second[0]['runners'])
        values[0].update(baserunning_play_type='PickedOffCaughtStealing',
                         attempted_advance_to_base=None, is_advance_attempt=False)
        second[0]['runners'] = json.dumps(values)
        pa = contact_fixture(key=3, mask=1)
        child = json.loads(first[0]['runners'])[0]
        pa = attached(pa, [dict(child, runner_id=pa[1]['first'])])
        # No-play history keeps its literal label but cannot carry real out/credit units.
        historical = dict(json.loads(first[0]['runners'])[0], is_out=False, base_end=None)
        no_play = dict(first[0], no_play_flag='true', event_key='4', event_id='4',
                       outs='0', credits='[]', runners=json.dumps([historical]))
        outside = contact_fixture(key=5)
        return [first, second, pa, outside], no_play

    def test_event_child_outside_and_prior_partitions_are_exact(self):
        pairs, no_play = self.pairs()
        rows = [p[0] for p in pairs] + [no_play]
        contexts = [p[1] for p in pairs]
        before = copy.deepcopy((rows, contexts))
        report = cs.inventory(rows, contexts, ['GAME1'])
        self.assertEqual(report['totals'], dict(raw_events=5, candidate_events=4,
            candidate_rows=4, outside_events=1, outside_child_rows=0))
        self.assertEqual(sum(v['events'] for v in report['events']), 4)
        self.assertEqual(sum(v['rows'] for v in report['rows']), 4)
        self.assertEqual({v['partition'] for v in report['events']}, {'pa', 'nonpa', 'no_play'})
        self.assertEqual(sum(v['rows'] for v in report['rows'] if v['label']=='PickedOffCaughtStealing'), 1)
        self.assertEqual(sum(v['rows'] for v in report['rows'] if v['attempted_advance_to_base'] is None
                             and v['is_advance_attempt'] is False), 1)
        self.assertEqual(report, cs.inventory(list(reversed(rows)), list(reversed(contexts)), ['GAME1']))
        self.assertEqual((rows, contexts), before)
        prior = cs.steals.inventory(rows, contexts, ['GAME1'])
        for name in ('totals','split_totals','source_totals','raw_totals','outcome_totals','supplied_credit_units'):
            self.assertEqual(report['prior_totals'][name], prior[name])

    def test_multilabel_events_split_and_joint_counts_do_not_double_count(self):
        pairs=[]
        for key,game in enumerate(('GAME1','GAME7','EXCLUDED'),1):
            row,context=fixture('CaughtStealing',mask=5,key=key)
            row['game_id']=context['game_id']=game
            values=json.loads(row['runners'])
            values[1]['baserunning_play_type']='PickedOffCaughtStealing'
            row['runners']=json.dumps(values)
            pairs.append((row,context))
        report=cs.inventory([p[0] for p in pairs],[p[1] for p in pairs],['GAME1','GAME7'])
        self.assertEqual(report['totals']['candidate_events'],3)
        self.assertEqual(report['totals']['candidate_rows'],6)
        self.assertEqual({v['split'] for v in report['events']},{'training','heldout','excluded'})
        self.assertTrue(all(v['labels']==list(cs.LABELS) for v in report['events']))
        self.assertEqual(sum(v['rows'] for v in report['rows'] if v['label']=='CaughtStealing' and v['is_out'] is True),3)
        self.assertEqual(sum(v['rows'] for v in report['rows'] if v['label']=='PickedOffCaughtStealing' and v['is_out'] is False),3)
        self.assertTrue(all('not_simple_steal' in v['qualification']['reasons'] for v in report['events']))

    def test_invalid_source_native_or_prior_blocks_new_observation(self):
        pairs, _ = self.pairs(); rows=[p[0] for p in pairs]; contexts=[p[1] for p in pairs]
        for kwargs in (dict(native_groups={}), dict(expected_prior={})):
            with self.subTest(kwargs=kwargs), patch.object(cs, 'observe') as observe:
                with self.assertRaises(ValueError): cs.inventory(rows, contexts, ['GAME1'], **kwargs)
                observe.assert_not_called()
        rows[0]['runners'] = rows[0]['runners'].replace('r1', 'WRONG')
        # Real conflicting identity, not a fabricated aggregate-only validator.
        children=json.loads(rows[0]['runners']);children[0]['runner_id']='WRONG'
        rows[0]['runners']=json.dumps(children)
        with patch.object(cs, 'observe') as observe:
            with self.assertRaises(ValueError): cs.inventory(rows, contexts, ['GAME1'])
            observe.assert_not_called()


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.source = steal_tests.PublicationTests()
        self.source.setUp()
        self.addCleanup(self.source.doCleanups)
        self.root = self.source.root
        self.prior = self.root / 'accepted-steals.json'
        with self.source.configured():
            cs.steals.publish(*self.source.inputs, self.prior)
        self.inputs = [*self.source.inputs, self.prior]
        self.pins = tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)

    def configured(self):
        return patch.object(cs, 'INPUT_SHA', self.pins, create=True)

    def test_authenticated_prior_is_exact_before_repeated_observations(self):
        with self.source.configured(), self.configured():
            first = cs.publish(*self.inputs, self.root / 'cs-first.json')
            repeat = cs.publish(*self.inputs, self.root / 'cs-repeat.json')
        self.assertIsInstance(first, dict)
        self.assertEqual(first, repeat)
        self.assertEqual(first['totals']['candidate_events'], 0)
        self.assertEqual((self.root/'cs-first.json').read_bytes(), (self.root/'cs-repeat.json').read_bytes())
        self.assertEqual(first['provenance']['prior_report_sha256'], self.pins[-1])
        self.assertEqual(tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs), self.pins)

    def test_wrong_pin_and_output_paths_refuse_before_predecessor_or_measurement(self):
        existing=self.root/'existing';existing.write_text('keep')
        symlink=self.root/'symlink';symlink.symlink_to(self.root/'missing')
        for output in (existing, symlink, Path(cs.__file__).resolve().parents[1]/'forbidden-cs.json'):
            with self.subTest(output=output), patch.object(cs.steals,'publish') as prior, patch.object(cs,'inventory') as measured:
                with self.assertRaises(ValueError): cs.publish(*self.inputs, output)
                prior.assert_not_called();measured.assert_not_called()
        with patch.object(cs.steals,'publish') as prior:
            with self.assertRaisesRegex(ValueError,'digest'): cs.publish(*self.inputs,self.root/'wrong-pin.json')
            prior.assert_not_called()
        self.assertEqual(existing.read_text(),'keep')

    def test_changed_prior_or_validator_failure_blocks_new_measurement(self):
        value=json.loads(self.prior.read_text());value['totals']['qualified_events']+=1
        self.prior.write_text(json.dumps(value))
        pins=tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs)
        with self.source.configured(), patch.object(cs,'INPUT_SHA',pins,create=True), patch.object(cs,'inventory') as measured:
            with self.assertRaisesRegex(ValueError,'prior'): cs.publish(*self.inputs,self.root/'bad-prior.json')
            measured.assert_not_called()
        with self.source.configured(), patch.object(cs,'INPUT_SHA',pins,create=True), \
                patch.object(cs.steals,'publish',side_effect=ValueError('full validator refused')),patch.object(cs,'inventory') as measured:
            with self.assertRaisesRegex(ValueError,'full validator refused'): cs.publish(*self.inputs,self.root/'validator.json')
            measured.assert_not_called()

    def test_late_drift_identity_collision_and_nonfinite_never_publish(self):
        originals={p:p.read_bytes() for p in self.inputs}
        original=cs.inventory
        for defect in ('prior','export','identity','collision','nonfinite'):
            for p,data in originals.items():p.write_bytes(data)
            export=self.source.exports[2];export_bytes=export.read_bytes()
            output=self.root/('late-'+defect+'.json')
            def mutate(*args,**kwargs):
                report=original(*args,**kwargs)
                if defect=='prior':self.prior.write_text('drift')
                if defect=='export':export.write_text('drift')
                if defect=='identity':
                    replacement=self.root/'replacement';replacement.write_bytes(self.prior.read_bytes());os.replace(replacement,self.prior)
                if defect=='collision':output.write_bytes(b'keep')
                if defect=='nonfinite':report['bad']=float('nan')
                return report
            with self.subTest(defect=defect),self.source.configured(),self.configured(),patch.object(cs,'inventory',side_effect=mutate):
                with self.assertRaises((ValueError,FileExistsError)):cs.publish(*self.inputs,output)
            if defect=='collision':self.assertEqual(output.read_bytes(),b'keep')
            else:self.assertFalse(output.exists())
            export.write_bytes(export_bytes)
        self.assertFalse(any(p.name.startswith('.runner-steals-') for p in self.root.iterdir()))

    def test_parent_redirect_and_private_recipe_drift_refuse_without_final_report(self):
        original=cs.inventory
        for defect in ('parent','recipe'):
            directory=self.root/('out-'+defect);directory.mkdir()
            output=directory/'report.json'
            private=self.root/'private'/'audits'/'cs_observability.py'
            private.parent.mkdir(parents=True,exist_ok=True)
            private.write_bytes(Path(cs.__file__).read_bytes())
            def mutate(*args,**kwargs):
                value=original(*args,**kwargs)
                if defect=='parent':
                    directory.rename(self.root/'parked')
                    redirect=self.root/'redirect';redirect.mkdir();directory.symlink_to(redirect)
                else:private.write_text('drift')
                return value
            with self.subTest(defect=defect),self.source.configured(),self.configured(), \
                    patch.object(cs,'__file__',str(private)),patch.object(cs,'inventory',side_effect=mutate):
                with self.assertRaises(ValueError):cs.publish(*self.inputs,output)
            self.assertFalse(output.exists())

    def test_cli_exposes_prior_report_and_uses_authenticated_publisher(self):
        result=subprocess.run([sys.executable,'-Werror','-m','audits.cs_observability','--help'],text=True,capture_output=True,check=True)
        self.assertIn('prior',result.stdout)
        output=self.root/'cli.json'
        with self.source.configured(),self.configured(),patch.object(sys,'argv',['cs-observability',*map(str,self.inputs),str(output)]):cs.main()
        self.assertTrue(output.exists())


if __name__ == '__main__':
    unittest.main()
