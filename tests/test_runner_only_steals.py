"""Simple endpoint proxies are not a pitch clock or sole-cause certificate."""
import copy
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from audits import runner_only_steals as steals
from tests import test_hit_advancement as hit_fixtures
from tests.test_joint_contact import fixture as contact_fixture
from tests.test_runner_transitions import attached, child

ROLES = ('First', 'Second', 'Third')


def fixture(kind='StolenBase', role='First', mask=1, outs=0, key=1):
    row, context = contact_fixture(key=key, mask=mask, outs=outs)
    index = ROLES.index(role)
    target = ('Second', 'Third', 'Home')[index]
    caught = kind == 'CaughtStealing'
    credits = ([dict(sequence_id=1, fielding_position=2, fielding_play='Assist'),
                dict(sequence_id=2, fielding_position=6, fielding_play='Putout')]
               if caught else [])
    row.update(result='', outs=str(int(caught)), runs='0', trajectory='', angle='',
               credits=json.dumps(credits), raw_play=('CS' if caught else 'SB')
               + ('2', '3', 'H')[index] + ('(26)' if caught else ''))
    boundary = caught and outs == 2
    after_mask = mask & ~(1 << index)
    if not caught and target != 'Home':
        after_mask |= 1 << (index + 1)
    context.update(result='', pa='', batting_outs='', outs=str(int(caught)),
                   state_outs=str(int(caught)), putouts=str(int(caught)),
                   assists=str(int(caught)), errors='0', trajectory='', angle='',
                   bip='', dp='', tp='', gidp='', frame_end=str(boundary).lower(),
                   after_outs='' if boundary else str(outs + int(caught)),
                   after_base_mask='' if boundary else str(after_mask))
    children = []
    for i, before_role in enumerate(ROLES):
        if mask & (1 << i):
            value = child(before_role, context[before_role.lower()])
            if before_role == role:
                value.update(baserunning_play_type=kind, is_out=caught,
                             base_end='Out' if caught else target,
                             is_advance_attempt=True, attempted_advance_to_base=target)
            children.append(value)
    return attached((row, context), children)


class QualificationTests(unittest.TestCase):
    def test_literal_next_base_sb_and_cs_with_boundary_are_supported_proxies(self):
        for kind, role, outs in itertools.product(
                ('StolenBase', 'CaughtStealing'), ('First', 'Second'), range(3)):
            mask = 1 << ROLES.index(role)
            row, context = fixture(kind, role, mask, outs)
            before = copy.deepcopy((row, context))
            with self.subTest(kind=kind, role=role, outs=outs):
                value = steals.classify(row, context)
                self.assertTrue(value['qualified'], value)
                self.assertEqual(value['reasons'], [])
                self.assertEqual(value['kind'], kind)
                self.assertEqual(value['target_role'], role)
                self.assertEqual(value['target_base'], 'Second' if role == 'First' else 'Third')
                self.assertEqual((row, context), before)

    def test_all_masks_outs_target_roles_use_independent_clear_target_oracle(self):
        attempts = accepted = 0
        for kind, mask, outs, role in itertools.product(
                ('StolenBase', 'CaughtStealing'), range(8), range(3), ('First', 'Second')):
            origin = ROLES.index(role)
            if not mask & (1 << origin):
                continue
            row, context = fixture(kind, role, mask, outs)
            expected = not bool(mask & (1 << (origin + 1)))
            normal = steals.classify(row, context)
            reversed_row = dict(row, runners=json.dumps(list(reversed(json.loads(row['runners'])))),
                                credits=json.dumps(list(reversed(json.loads(row['credits'])))))
            with self.subTest(kind=kind, mask=mask, outs=outs, role=role):
                self.assertEqual(normal['qualified'], expected, normal)
                self.assertEqual(normal, steals.classify(reversed_row, context))
                if not expected:
                    self.assertIn('unsupported_geometry', normal['reasons'])
            attempts += 1
            accepted += int(normal['qualified'])
        self.assertEqual((attempts, accepted), (48, 24))

    def test_missing_row_and_flags_are_not_held_zeroes(self):
        row, context = fixture(mask=5)
        values = json.loads(row['runners'])
        row['runners'] = json.dumps(values[:1])
        value = steals.classify(row, context)
        self.assertFalse(value['qualified'])
        self.assertIn('incomplete_vector', value['reasons'])
        row, context = fixture()
        values = json.loads(row['runners'])
        values[0]['advanced_on_error_flag'] = None
        row['runners'] = json.dumps(values)
        value = steals.classify(row, context)
        self.assertFalse(value['qualified'])
        self.assertIn('incomplete_vector', value['reasons'])
        self.assertIn('runner_effect_or_unknown', value['reasons'])

    def test_compounds_pickoff_cs_home_and_errors_remain_unresolved(self):
        for change, expected in [
                (dict(raw_play='SB2;SB3'), 'notation_uncertain'),
                (dict(raw_play='SB2.1-2'), 'notation_uncertain'),
                (dict(raw_play='POCS2(136)'), 'notation_uncertain'),
                (dict(raw_play='SB2/E2'), 'notation_uncertain')]:
            row, context = fixture()
            row.update(change)
            with self.subTest(change=change):
                value = steals.classify(row, context)
                self.assertFalse(value['qualified'])
                self.assertIn(expected, value['reasons'])
        row, context = fixture(role='Third', mask=4)
        self.assertIn('unsupported_geometry', steals.classify(row, context)['reasons'])
        row, context = fixture(kind='PickedOffCaughtStealing')
        self.assertIn('not_simple_steal', steals.classify(row, context)['reasons'])


class BoundaryAndUnknownTests(unittest.TestCase):
    def test_boundary_normalization_is_not_physical_runner_retirement(self):
        attempts = accepted = 0
        for after_outs, after_mask in itertools.product(('', '0', '1', '2', '3'), ('', '0', '1', '4')):
            row, context = fixture('CaughtStealing', mask=5, outs=2)
            context.update(after_outs=after_outs, after_base_mask=after_mask)
            value = steals.classify(row, context)
            expected = after_outs in ('', '0', '3') and after_mask in ('', '0')
            with self.subTest(after_outs=after_outs, after_mask=after_mask):
                self.assertEqual(value['qualified'], expected)
                self.assertEqual(value['reasons'], [] if expected else ['unsupported_geometry'])
                self.assertEqual(value['runners'][1]['base_end'], 'Third')
                self.assertFalse(value['runners'][1]['is_out'])
            attempts += 1; accepted += int(value['qualified'])
        self.assertEqual((attempts, accepted), (20, 6))

    def test_null_flags_and_known_contradictions_keep_exact_independent_reasons(self):
        for field in ('is_out', 'run_scored_flag', 'explicit_out_flag',
                      'advanced_on_error_flag', 'is_advance_attempt'):
            row, context = fixture()
            values = json.loads(row['runners']); values[0][field] = None
            row['runners'] = json.dumps(values)
            expected = ['incomplete_vector']
            if field != 'explicit_out_flag': expected.append('runner_effect_or_unknown')
            with self.subTest(field=field):
                self.assertEqual(steals.classify(row, context)['reasons'], expected)
        row, context = fixture(mask=5)
        row['runners'] = json.dumps(json.loads(row['runners'])[:1])
        context['after_base_mask'] = '1'
        self.assertEqual(steals.classify(row, context)['reasons'],
                         ['incomplete_vector', 'unsupported_geometry'])
        row, context = fixture()
        values = json.loads(row['runners']); values[0]['explicit_out_flag'] = True
        row['runners'] = json.dumps(values)
        self.assertTrue(steals.classify(row, context)['qualified'])

    def test_credit_effect_attempt_and_literal_modifiers_are_not_coerced(self):
        row, context = fixture('CaughtStealing')
        row['credits'] = '[]'
        self.assertEqual(steals.classify(row, context)['reasons'], ['not_simple_steal'])
        row, context = fixture()
        values = json.loads(row['runners']); values[0]['attempted_advance_to_base'] = 'Third'
        row['runners'] = json.dumps(values)
        self.assertEqual(steals.classify(row, context)['reasons'], ['runner_effect_or_unknown'])
        row['raw_play'] = 'SB2/TH'
        self.assertEqual(steals.classify(row, context)['reasons'],
                         ['notation_uncertain', 'runner_effect_or_unknown'])
        row, context = fixture(mask=5)
        values = json.loads(row['runners'])
        values[1].update(baserunning_play_type='StolenBase', is_advance_attempt=True,
                         attempted_advance_to_base='Home', base_end='Home')
        row['runners'] = json.dumps(values)
        reasons = steals.classify(row, context)['reasons']
        self.assertEqual(reasons, ['not_simple_steal', 'notation_uncertain',
                                  'runner_effect_or_unknown', 'unsupported_geometry'])


class InventoryTests(unittest.TestCase):
    def measure(self, pairs, **kwargs):
        return steals.inventory([p[0] for p in pairs], [p[1] for p in pairs], ['GAME1'], **kwargs)

    def test_full_candidate_exposure_and_outside_partitions_are_order_invariant(self):
        pairs = [fixture(key=1, mask=5), fixture('CaughtStealing', 'Second', 3, 2, 2),
                 contact_fixture(key=3, mask=1)]
        # Missing held endpoint stays unresolved, not an invented hold.
        unresolved = fixture(key=4, mask=5)
        values = json.loads(unresolved[0]['runners'])
        unresolved[0]['runners'] = json.dumps(values[:1])
        unresolved[1]['runner_rows'] = '1'
        pairs.append(unresolved)
        frozen = copy.deepcopy(pairs)
        report = self.measure(pairs)
        self.assertEqual(report['format'], '2019-runner-only-steals-v1')
        self.assertEqual(report['totals'], dict(candidate_events=3, qualified_events=2,
                                             unresolved_events=1, before_runner_opportunities=7))
        self.assertEqual(report['source_totals']['real_events'], 4)
        self.assertEqual(report['outside_results'], {'InPlayOut': 1})
        self.assertEqual(sum(x['opportunities'] for x in report['exposures']), 7)
        self.assertEqual(sum(x['events'] for x in report['joint_vectors']), 3)
        self.assertEqual(sum(x['events'] for x in report['qualified_cells']), 2)
        self.assertEqual(sum(report['first_reasons'].values()), 1)
        self.assertEqual(report, self.measure(list(reversed(pairs))))
        self.assertEqual(pairs, frozen)

    def test_clear_next_base_proxy_counts_all_before_frames_without_attempt_inference(self):
        pairs = [contact_fixture(key=i + 1, mask=mask, outs=outs)
                 for i, (mask, outs) in enumerate(itertools.product(range(8), range(3)))]
        report = self.measure(pairs)
        self.assertEqual(report['totals']['candidate_events'], 0)
        self.assertEqual(report['totals']['before_runner_opportunities'], 36)
        self.assertEqual(sum(x['opportunities'] for x in report['exposures'] if x['clear_next_base']), 12)
        self.assertEqual(report['qualified_cells'], [])
        self.assertTrue(all(x['pa'] for x in report['exposures']))
        self.assertNotIn('attempt_rate', report)

    def test_full_source_totals_are_preserved_without_reinterpreting_credit_units(self):
        pair = fixture('CaughtStealing')
        source = steals.runner.inventory([pair[0]], [pair[1]], ['GAME1'])
        report = self.measure([pair])
        for name in ('raw_totals', 'outcome_totals', 'supplied_credit_units'):
            with self.subTest(name=name): self.assertEqual(report[name], source[name])
        self.assertEqual(report['source_totals'], source['totals'])

    def test_complete_native_and_source_guards_precede_classification(self):
        pair = fixture()
        for options in [dict(native_groups={}), dict(expected_census={})]:
            with self.subTest(options=options), patch.object(steals, 'classify') as classify:
                with self.assertRaises(ValueError):
                    self.measure([pair], **options)
                classify.assert_not_called()
        values = json.loads(pair[0]['runners'])
        values[0]['runner_id'] = 'SYNTHETIC_CONFLICT'
        pair[0]['runners'] = json.dumps(values)
        with patch.object(steals, 'classify') as classify:
            with self.assertRaises(ValueError):
                self.measure([pair])
            classify.assert_not_called()


class OutsidePartitionTests(unittest.TestCase):
    def test_no_play_pa_compounds_and_unrelated_nonpa_labels_are_retained(self):
        pairs = [fixture(key=1), fixture(key=2), fixture(key=3), contact_fixture(key=4, mask=1)]
        # No-play raw history creates neither a runner frame nor a candidate.
        no_play = dict(pairs[0][0], no_play_flag='true', outs='0', runs='0', result='')
        no_play.update(event_key='5', event_id='5')
        for pair, game in zip(pairs[:3], ('GAME1', 'GAME7', 'EXCLUDED')):
            pair[0]['game_id'] = pair[1]['game_id'] = game
        values = json.loads(pairs[2][0]['runners'])
        values[0]['baserunning_play_type'] = 'OtherAdvance'
        pairs[2][0]['runners'] = json.dumps(values)
        pa = pairs[3]
        values = [child('First', pa[1]['first'], base_end='Second', baserunning_play_type='StolenBase',
                        is_advance_attempt=True, attempted_advance_to_base='Second')]
        pairs[3] = attached(pa, values)
        pairs[3][0]['raw_play'] = '7/F7D+SB2'
        pairs[3][1]['after_base_mask'] = '2'
        report = steals.inventory([p[0] for p in pairs] + [no_play], [p[1] for p in pairs], ['GAME1', 'GAME7'])
        self.assertEqual(report['no_play_events'], 1)
        self.assertEqual(report['totals']['candidate_events'], 3)
        self.assertEqual(report['totals']['qualified_events'], 2)
        self.assertEqual(report['totals']['unresolved_events'], 1)
        self.assertEqual(report['totals']['before_runner_opportunities'], 4)
        self.assertEqual({k: v['candidate_events'] for k, v in report['split_totals'].items()},
                         dict(training=1, heldout=1, excluded=1))
        self.assertEqual(report['outside_results'], {'InPlayOut': 1})
        self.assertEqual(sum(x['rows'] for x in report['pa_attempt_labels']), 1)
        self.assertEqual({x['label'] for x in report['candidate_child_labels']}, {'StolenBase', 'OtherAdvance'})


class PublicationTests(unittest.TestCase):
    def setUp(self):
        # Reuse accepted synthetic controller fixtures, never real source data.
        self.source = hit_fixtures.PinnedControllerTests()
        self.source.setUp()
        self.addCleanup(self.source.doCleanups)
        self.root = self.source.root
        self.inputs, self.exports = self.source.inputs, self.source.exports
        self.pair = fixture(mask=5)
        self.census = steals.runner.inventory([self.pair[0]], [self.pair[1]], ['GAME1'])
        provenance = json.loads(self.inputs[4].read_text())['provenance']
        groups = [dict(facts=json.dumps({k: v for k, v in cell.items() if k != 'rows'}, sort_keys=True),
                       rows=cell['rows']) for cell in self.census['raw_child_cells']]
        for path, fields, rows in [(self.exports[1], list(self.pair[1]), [self.pair[1]]),
                                  (self.exports[2], list(self.pair[0]), [self.pair[0]]),
                                  (self.exports[3], ['facts', 'rows'], groups)]:
            path.write_text(self.source.csv_text(fields, rows))
        self.census['provenance'] = provenance
        self.repin()
        self.cohort = dict(games=1, supported_games=1, excluded_game_ids=[],
            raw_totals=self.census['raw_totals'], supplied_credit_units=self.census['supplied_credit_units'])

    @staticmethod
    def pins(paths):
        return tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in paths)

    def repin(self):
        self.export_pins = self.pins(self.exports)
        self.census['provenance']['native_export_sha256'] = list(self.export_pins)
        self.inputs[4].write_text(json.dumps(self.census, sort_keys=True))
        self.input_pins = self.pins(self.inputs)

    def configured(self):
        return patch.multiple(steals, INPUT_SHA=self.input_pins, EXPORT_SHA=self.export_pins,
                              COHORT=self.cohort, create=True)

    def test_complete_authenticated_controller_repeat_and_exact_provenance(self):
        before = self.pins(self.inputs + self.exports)
        with self.configured():
            first = steals.publish(*self.inputs, self.root / 'first.json')
            repeat = steals.publish(*self.inputs, self.root / 'repeat.json')
        self.assertEqual(first['totals']['qualified_events'], 1)
        self.assertEqual(first, repeat)
        self.assertEqual((self.root / 'first.json').read_bytes(), (self.root / 'repeat.json').read_bytes())
        self.assertEqual(before, self.pins(self.inputs + self.exports))
        self.assertEqual(first['provenance']['census_sha256'], self.input_pins[4])
        self.assertEqual(first['provenance']['native_export_sha256'], list(self.export_pins))
        self.assertEqual(first['format'], steals.FORMAT)

    def test_early_paths_and_digests_refuse_without_measurement(self):
        repo = Path(steals.__file__).resolve().parents[1]
        existing = self.root / 'existing'; existing.write_bytes(b'keep')
        linked = self.root / 'linked'; linked.symlink_to(self.root / 'missing')
        for output, error in [(repo / 'forbidden-steals.json', 'outside repository'),
                              (existing, 'Refusing to overwrite'), (linked, 'Refusing to overwrite')]:
            with self.subTest(output=output), patch.object(steals, 'inventory') as measured:
                with self.assertRaisesRegex(ValueError, error):
                    steals.publish(*(['missing'] * 5), output)
                measured.assert_not_called()
        with patch.object(steals, 'inventory') as measured:
            with self.assertRaisesRegex(ValueError, 'digest'):
                steals.publish(*self.inputs, self.root / 'unaccepted.json')
            measured.assert_not_called()
        self.assertEqual(existing.read_bytes(), b'keep')
        self.assertTrue(linked.is_symlink())

    def test_headers_native_census_provenance_and_cohort_block_classification(self):
        originals = {p: p.read_bytes() for p in self.inputs + self.exports}
        for defect in ('header', 'native', 'census', 'provenance', 'cohort'):
            for p, data in originals.items(): p.write_bytes(data)
            self.census = json.loads(self.inputs[4].read_text())
            if defect == 'header': self.exports[1].write_text('bad,header\n')
            if defect == 'native': self.exports[3].write_text('facts,rows\n')
            if defect == 'census': self.census['totals']['non_pa'] += 1
            if defect == 'provenance': self.census['provenance']['snapshot_id'] = 999
            self.repin()
            cohort = dict(self.cohort, games=2) if defect == 'cohort' else self.cohort
            output = self.root / ('bad-' + defect + '.json')
            with self.subTest(defect=defect), self.configured(), patch.object(steals, 'COHORT', cohort), \
                    patch.object(steals, 'classify', side_effect=AssertionError('measurement started')) as measured:
                with self.assertRaises(ValueError): steals.publish(*self.inputs, output)
                measured.assert_not_called()
            self.assertFalse(output.exists())

    def test_late_drift_identity_collision_and_nonfinite_report_never_publish(self):
        originals = {p: p.read_bytes() for p in self.inputs + self.exports}
        original = steals.inventory
        for defect in ('input', 'export', 'identity', 'collision', 'nonfinite', 'parent'):
            for p, data in originals.items(): p.write_bytes(data)
            directory = self.root / ('output-' + defect); directory.mkdir()
            output = directory / 'report.json'
            def mutate(*args, **kwargs):
                value = original(*args, **kwargs)
                if defect == 'input': self.inputs[3].write_text('drift')
                if defect == 'export': self.exports[2].write_text('drift')
                if defect == 'identity':
                    replacement = self.root / 'replacement'; replacement.write_bytes(self.inputs[3].read_bytes())
                    os.replace(replacement, self.inputs[3])
                if defect == 'collision': output.write_bytes(b'keep')
                if defect == 'nonfinite': value['invalid'] = float('nan')
                if defect == 'parent':
                    directory.rename(self.root / 'parked')
                    directory.symlink_to(self.root / 'redirect')
                    (self.root / 'redirect').mkdir()
                return value
            with self.subTest(defect=defect), self.configured(), patch.object(steals, 'inventory', side_effect=mutate):
                with self.assertRaises((ValueError, FileExistsError)):
                    steals.publish(*self.inputs, output)
            if defect == 'collision': self.assertEqual(output.read_bytes(), b'keep')
            else: self.assertFalse(output.exists())
            self.assertFalse(any(p.name.startswith('.runner-steals-') for p in directory.iterdir()))

    def test_private_query_and_recipe_drift_block_publication(self):
        directory = self.root / 'private' / 'audits'; directory.mkdir(parents=True)
        runner_copy = directory / 'runner_transitions.py'
        runner_copy.write_bytes(Path(steals.runner.__file__).read_bytes())
        names = ('2019-lineup-readiness.sql', '2019-outcomes.sql', '2019-raw-mechanisms.sql',
                 '2019-runner-transitions.sql')
        for name in names:
            (directory / name).write_bytes(Path(steals.runner.__file__).with_name(name).read_bytes())
        original = steals.inventory
        for target in (directory / names[0], runner_copy):
            before = target.read_bytes()
            output = self.root / ('drift-' + target.name + '.json')
            def mutate(*args, **kwargs):
                value = original(*args, **kwargs); target.write_text('drift'); return value
            with self.subTest(target=target), self.configured(), \
                    patch.object(steals.runner, '__file__', str(runner_copy)), \
                    patch.object(steals, 'inventory', side_effect=mutate):
                with self.assertRaisesRegex(ValueError, 'inputs changed'):
                    steals.publish(*self.inputs, output)
            self.assertFalse(output.exists())
            target.write_bytes(before)

    def test_writer_stages_complete_finite_json_fsyncs_then_links_exclusively(self):
        output = self.root / 'writer.json'; source = self.inputs[0]
        report = dict(z=None, a=0)
        expected = (json.dumps(report, sort_keys=True, indent=2, allow_nan=False) + '\n').encode()
        actual_link = os.link
        def link(stage, final):
            self.assertEqual(Path(stage).read_bytes(), expected)
            self.assertFalse(Path(final).exists())
            actual_link(stage, final)
        with patch.object(os, 'link', side_effect=link), patch.object(os, 'fsync', wraps=os.fsync) as fsync:
            steals.write_report(report, [source], self.pins([source]), output)
            fsync.assert_called_once()
        self.assertEqual(output.read_bytes(), expected)
        for defect in ('collision', 'nan', 'drift', 'flush'):
            final = output if defect == 'collision' else self.root / (defect + '.json')
            pins = self.pins([source])
            if defect == 'drift': source.write_text('drift')
            value = dict(bad=float('nan')) if defect == 'nan' else report
            with self.subTest(defect=defect):
                if defect == 'flush':
                    with patch.object(os, 'fsync', side_effect=OSError('fsync failed')):
                        with self.assertRaises(OSError): steals.write_report(value, [source], pins, final)
                else:
                    with self.assertRaises((ValueError, FileExistsError)):
                        steals.write_report(value, [source], pins, final)
            if defect != 'collision': self.assertFalse(final.exists())
        self.assertFalse(any(p.name.startswith('.runner-steals-') for p in self.root.iterdir()))

    def test_cli_names_only_local_cache_inputs_and_calls_publisher(self):
        help_run = subprocess.run([sys.executable, '-Werror', '-m', 'audits.runner_only_steals', '--help'],
                                  text=True, capture_output=True, check=True)
        for name in ('snapshot', 'supplement', 'lineups', 'manifest', 'census', 'output'):
            self.assertIn(name, help_run.stdout)
        with self.configured(), patch.object(sys, 'argv', ['runner-only-steals', *map(str, self.inputs),
                                                         str(self.root / 'cli.json')]):
            steals.main()
        self.assertEqual(json.loads((self.root / 'cli.json').read_text())['totals']['qualified_events'], 1)


if __name__ == '__main__':
    unittest.main()
