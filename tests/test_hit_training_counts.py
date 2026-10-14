"""Synthetic training-count wire contract; no real source inputs or fitted games."""
from contextlib import ExitStack, redirect_stdout
import copy
import hashlib
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from audits import hit_training_counts as export
from audits import hit_advancement as hit
from tests import test_hit_advancement_comparison as comparison_tests
from tests.test_hit_advancement_comparison import independent_support, synthetic_cell, synthetic_report

DIGEST = '0123456789abcdef' * 4


def reference_wire(report_counts):
    lines = ['joint-hit-training-counts-v1', 'proxy_endpoints_not_certified_causes\ttrue',
             'qualification_sha256\t' + DIGEST,
             'training_events\t' + str(sum(sum(counts) for counts in report_counts.values())),
             'value\tbase_mask\touts_before\tcounts']
    for value in (1, 2):
        for mask in range(8):
            for outs in range(3):
                counts = report_counts.get((value, mask, outs), [0] * len(independent_support(value, mask)))
                lines.append(f'{value}\t{mask}\t{outs}\t' + ','.join(map(str, counts)))
    return ('\n'.join(lines) + '\n').encode('ascii')


class EncodingTests(unittest.TestCase):
    def test_empty_qualified_training_is_complete_canonical_zero_table(self):
        actual = export.encode(synthetic_report(), DIGEST)
        self.assertIs(type(actual), bytes)
        self.assertEqual(actual, reference_wire({}))
        self.assertEqual(len(actual.splitlines()), 53)
        self.assertEqual(sum(len(row.split(b'\t')[3].split(b',')) for row in actual.splitlines()[5:]), 120)

    def test_literal_sparse_training_retains_full_ordered_joint_categories(self):
        report = synthetic_report(synthetic_cell('training', 0, [1, 2], 3),
                                  synthetic_cell('training', 0, [1, 4], 1),
                                  synthetic_cell('training', 2, [2, 3], 2, value=2))
        self.assertEqual(export.encode(report, DIGEST), reference_wire({(1, 1, 0): [3, 0, 1], (2, 1, 2): [2, 0]}))

    def test_valid_reserved_and_excluded_changes_do_not_change_training_bytes(self):
        training = synthetic_cell('training', 0, [1, 3], 4)
        first = synthetic_report(training, synthetic_cell('heldout', 0, [1, 2], 1))
        changed = synthetic_report(training, synthetic_cell('heldout', 2, [1, 4], 37),
                                   synthetic_cell('excluded', 1, [2, 4], 11, value=2))
        self.assertEqual(export.encode(first, DIGEST), export.encode(changed, DIGEST))

    def test_shuffle_is_exact_and_report_is_not_mutated(self):
        report = synthetic_report(synthetic_cell('training', 0, [1, 2], 3),
                                  synthetic_cell('training', 0, [1, 4], 1))
        original = copy.deepcopy(report)
        changed = copy.deepcopy(report)
        changed['joint_vectors'].reverse()
        self.assertEqual(export.encode(report, DIGEST), export.encode(changed, DIGEST))
        self.assertEqual(report, original)

    def test_digest_is_metadata_but_requires_exact_lowercase_sha256_syntax(self):
        report = synthetic_report()
        for digest in (None, True, b'0' * 64, '', '0' * 63, '0' * 65, 'A' * 64, 'g' * 64, ' ' * 64):
            with self.subTest(digest=digest), self.assertRaises(ValueError):
                export.encode(report, digest)
        actual = export.encode(report, '0' * 64)
        self.assertIn(b'qualification_sha256\t' + b'0' * 64 + b'\n', actual)

    def test_digest_change_is_visible_without_changing_training_rows(self):
        first = export.encode(synthetic_report(), DIGEST)
        changed = export.encode(synthetic_report(), 'f' * 64)
        self.assertNotEqual(first.splitlines()[2], changed.splitlines()[2])
        self.assertEqual(first.splitlines()[3:], changed.splitlines()[3:])

    def test_complete_existing_aggregate_validator_failures_block_encoding(self):
        report = synthetic_report(synthetic_cell('training', 0, [1, 2]))
        cases = [None, {}, dict(report, format='unsupported')]
        for field, value in [('events', True), ('events', -1), ('events', 1 << 64), ('outs_before', 3)]:
            changed = copy.deepcopy(report)
            changed['joint_vectors'][0][field] = value
            cases.append(changed)
        changed = copy.deepcopy(report)
        changed['totals']['qualified_events'] += 1
        cases.append(changed)
        changed = copy.deepcopy(report)
        changed['joint_vectors'][0]['runners'][0]['base_end'] = 'Second'
        cases.append(changed)
        for invalid in cases:
            with self.subTest(report=invalid), self.assertRaises(ValueError):
                export.encode(invalid, DIGEST)

    def test_u64_maximum_is_exact_not_float_clipping(self):
        report = synthetic_report(synthetic_cell('training', 0, [1], (1 << 64) - 1, mask=0))
        actual = export.encode(report, DIGEST)
        self.assertEqual(actual, reference_wire({(1, 0, 0): [(1 << 64) - 1]}))

    def test_global_u64_overflow_refusal_is_inherited_aggregate_characterization(self):
        report = synthetic_report(synthetic_cell('training', 0, [1], (1 << 64) - 1, mask=0),
                                  synthetic_cell('training', 0, [2], 1, value=2, mask=0))
        with self.assertRaises(ValueError):
            export.encode(report, DIGEST)


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.fixture = comparison_tests.PublicationTests('test_complete_synthetic_controller_repeats_and_qualification_precedes_scoring')
        self.addCleanup(self.fixture.doCleanups)
        self.fixture.setUp()
        self.root, self.inputs = self.fixture.root, self.fixture.inputs
        self.accepted_bytes = self.fixture.accepted_bytes

    def configured(self):
        stack = ExitStack()
        stack.enter_context(self.fixture.configured())
        stack.enter_context(patch.object(export, 'INPUT_SHA', self.fixture.input_pins, create=True))
        stack.enter_context(patch.object(export, 'EXPORT_SHA', self.fixture.fixture.export_pins, create=True))
        return stack

    def test_complete_controller_repeats_and_full_qualification_precedes_encoding(self):
        original = export.encode
        outputs = [self.root / 'first.qualification.json', self.root / 'repeat.qualification.json']
        def encode_after_qualification(report, digest):
            self.assertEqual(outputs.pop(0).read_bytes(), self.accepted_bytes)
            self.assertEqual(digest, self.fixture.input_pins[5])
            return original(report, digest)
        with self.configured(), patch.object(export, 'encode', side_effect=encode_after_qualification):
            first = export.publish(*self.inputs, self.root / 'first.tsv')
            repeat = export.publish(*self.inputs, self.root / 'repeat.tsv')
        self.assertEqual(first, repeat)
        self.assertEqual(first, (self.root / 'first.tsv').read_bytes())
        self.assertEqual(first, (self.root / 'repeat.tsv').read_bytes())
        self.assertEqual(len(first.splitlines()), 53)
        self.assertEqual(tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in self.inputs), self.fixture.input_pins)

    def test_initial_repository_and_count_qualification_collisions_refuse_before_reads(self):
        repo = Path(hit.__file__).resolve().parents[1]
        with self.assertRaisesRegex(ValueError, 'outside repository'):
            export.publish(*(['missing'] * 6), repo / 'forbidden-counts.tsv')
        for name in ('count', 'qualification', 'link'):
            output = self.root / (name + '.tsv')
            if name == 'count': output.write_bytes(b'keep')
            elif name == 'qualification': output.with_suffix('.qualification.json').write_bytes(b'keep qualification')
            else: output.symlink_to(self.inputs[5])
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'Refusing to overwrite'):
                export.publish(*(['missing'] * 6), output)
        self.assertEqual((self.root / 'count.tsv').read_bytes(), b'keep')
        self.assertEqual((self.root / 'qualification.qualification.json').read_bytes(), b'keep qualification')
        self.assertFalse((repo / 'forbidden-counts.tsv').exists())

    def test_unaccepted_digest_refuses_before_any_qualification_or_encoding(self):
        self.inputs[5].write_bytes(b'unaccepted')
        with self.configured(), patch.object(hit, 'publish', side_effect=AssertionError('qualification started')):
            with self.assertRaisesRegex(ValueError, 'Unsupported.*digest'):
                export.publish(*self.inputs, self.root / 'bad.tsv')
        self.assertFalse((self.root / 'bad.tsv').exists())
        self.assertFalse((self.root / 'bad.qualification.json').exists())

    def test_native_export_digest_refuses_before_qualification(self):
        self.fixture.fixture.exports[3].write_bytes(b'drift')
        with self.configured(), patch.object(hit, 'publish', side_effect=AssertionError('qualification started')):
            with self.assertRaisesRegex(ValueError, 'native export digest'):
                export.publish(*self.inputs, self.root / 'bad-native.tsv')
        self.assertFalse((self.root / 'bad-native.tsv').exists())

    def test_byte_different_semantically_equal_qualification_refuses_before_encoding(self):
        self.inputs[5].write_text(json.dumps(json.loads(self.accepted_bytes), separators=(',', ':')))
        self.fixture.repin()
        with self.configured(), patch.object(export, 'encode', side_effect=AssertionError('encoding started')):
            with self.assertRaisesRegex(ValueError, 'qualification.*byte mismatch'):
                export.publish(*self.inputs, self.root / 'different.tsv')
        self.assertFalse((self.root / 'different.tsv').exists())
        self.assertEqual((self.root / 'different.qualification.json').read_bytes(), self.accepted_bytes)

    def test_native_group_validator_failure_is_preserved_not_replaced_by_count_parser(self):
        self.fixture.fixture.exports[3].write_text('facts,rows\n')
        pins = [hashlib.sha256(p.read_bytes()).hexdigest() for p in self.fixture.fixture.exports]
        census = json.loads(self.inputs[4].read_text())
        census['provenance']['native_export_sha256'] = pins
        self.inputs[4].write_text(json.dumps(census, sort_keys=True))
        self.fixture.repin()
        with self.configured(), patch.object(export, 'encode', side_effect=AssertionError('encoding started')):
            with self.assertRaisesRegex(ValueError, 'Native runner group reconciliation failed'):
                export.publish(*self.inputs, self.root / 'disagreement.tsv')
        self.assertFalse((self.root / 'disagreement.tsv').exists())
        self.assertFalse((self.root / 'disagreement.qualification.json').exists())

    def test_encode_failure_preserves_completed_qualification_without_counts(self):
        with self.configured(), patch.object(export, 'encode', side_effect=ValueError('synthetic encode failure')):
            with self.assertRaisesRegex(ValueError, 'synthetic encode failure'):
                export.publish(*self.inputs, self.root / 'failed.tsv')
        self.assertFalse((self.root / 'failed.tsv').exists())
        self.assertEqual((self.root / 'failed.qualification.json').read_bytes(), self.accepted_bytes)

    def test_final_input_export_comparison_and_exporter_recipe_drift_refuse(self):
        original = export.encode
        for label, path, module in [('input', self.inputs[3], None), ('export', self.fixture.fixture.exports[3], None),
                                    ('comparison', self.root / 'comparison-copy.py', comparison_tests.compare),
                                    ('exporter', self.root / 'exporter-copy.py', export)]:
            if module is not None: path.write_bytes(Path(module.__file__).read_bytes())
            before = path.read_bytes()
            def drift(report, digest):
                result = original(report, digest)
                path.write_bytes(before + b'\ndrift')
                return result
            with self.subTest(label=label), self.configured(), ExitStack() as stack:
                if module is not None: stack.enter_context(patch.object(module, '__file__', str(path)))
                stack.enter_context(patch.object(export, 'encode', side_effect=drift))
                with self.assertRaisesRegex(ValueError, 'inputs changed'):
                    export.publish(*self.inputs, self.root / (label + '-drift.tsv'))
            self.assertFalse((self.root / (label + '-drift.tsv')).exists())
            self.assertEqual((self.root / (label + '-drift.qualification.json')).read_bytes(), self.accepted_bytes)
            path.write_bytes(before)

    def test_query_drift_refuses_without_altering_any_repository_query(self):
        folder = self.root / 'queries'; folder.mkdir()
        runner_copy = folder / 'runner_transitions.py'
        runner_copy.write_bytes(Path(hit.runner.__file__).read_bytes())
        names = ['2019-lineup-readiness.sql', '2019-outcomes.sql', '2019-raw-mechanisms.sql', '2019-runner-transitions.sql']
        for name in names: (folder / name).write_bytes(Path(hit.runner.__file__).with_name(name).read_bytes())
        original = export.encode
        def drift(report, digest):
            result = original(report, digest); (folder / names[0]).write_text('drift'); return result
        with self.configured(), patch.object(hit.runner, '__file__', str(runner_copy)), patch.object(export, 'encode', side_effect=drift):
            with self.assertRaisesRegex(ValueError, 'inputs changed'):
                export.publish(*self.inputs, self.root / 'query.tsv')
        self.assertFalse((self.root / 'query.tsv').exists())
        self.assertEqual((self.root / 'query.qualification.json').read_bytes(), self.accepted_bytes)

    def test_final_qualification_reread_refuses_drift_and_keeps_complete_evidence(self):
        original = export.encode
        qualification = self.root / 'qualification-drift.qualification.json'
        def drift(report, digest):
            result = original(report, digest); qualification.write_bytes(self.accepted_bytes + b'\n'); return result
        with self.configured(), patch.object(export, 'encode', side_effect=drift):
            with self.assertRaisesRegex(ValueError, 'inputs changed'):
                export.publish(*self.inputs, self.root / 'qualification-drift.tsv')
        self.assertFalse((self.root / 'qualification-drift.tsv').exists())
        self.assertEqual(qualification.read_bytes(), self.accepted_bytes + b'\n')

    def test_final_input_path_identity_refuses_even_identical_replacement_bytes(self):
        original = export.encode
        path = self.inputs[3]
        copy_path = self.root / 'manifest-copy.json'; copy_path.write_bytes(path.read_bytes())
        def redirect(report, digest):
            result = original(report, digest); path.unlink(); path.symlink_to(copy_path); return result
        with self.configured(), patch.object(export, 'encode', side_effect=redirect):
            with self.assertRaisesRegex(ValueError, 'input.*path changed'):
                export.publish(*self.inputs, self.root / 'redirect.tsv')
        self.assertFalse((self.root / 'redirect.tsv').exists())

    def test_final_output_parent_identity_refuses_before_redirected_publication(self):
        folder = self.root / 'output-dir'; folder.mkdir()
        elsewhere = self.root / 'elsewhere'; elsewhere.mkdir()
        original = export.encode
        def redirect(report, digest):
            result = original(report, digest); folder.rename(self.root / 'original-output-dir'); folder.symlink_to(elsewhere); return result
        with self.configured(), patch.object(export, 'encode', side_effect=redirect):
            with self.assertRaisesRegex(ValueError, 'output path changed'):
                export.publish(*self.inputs, folder / 'redirect.tsv')
        self.assertFalse((elsewhere / 'redirect.tsv').exists())
        self.assertEqual((self.root / 'original-output-dir/redirect.qualification.json').read_bytes(), self.accepted_bytes)

    def test_late_count_collision_is_exclusive_and_does_not_replace_evidence(self):
        original = export.encode
        output = self.root / 'collision.tsv'
        def collision(report, digest):
            result = original(report, digest); output.write_bytes(b'keep late evidence'); return result
        with self.configured(), patch.object(export, 'encode', side_effect=collision):
            with self.assertRaises(FileExistsError): export.publish(*self.inputs, output)
        self.assertEqual(output.read_bytes(), b'keep late evidence')
        self.assertEqual(output.with_suffix('.qualification.json').read_bytes(), self.accepted_bytes)

    def test_stage_flush_failure_is_already_green_no_partial_count_characterization(self):
        output = self.root / 'flush-failure.tsv'
        with self.configured(), patch.object(export.os, 'fsync', side_effect=OSError('synthetic flush failure')):
            with self.assertRaisesRegex(OSError, 'synthetic flush failure'):
                export.publish(*self.inputs, output)
        self.assertFalse(output.exists())
        self.assertEqual(output.with_suffix('.qualification.json').read_bytes(), self.accepted_bytes)
        self.assertEqual(list(self.root.glob('.hit-counts-*')), [])

    def test_exclusive_link_failure_is_already_green_complete_stage_cleanup_characterization(self):
        output = self.root / 'link-failure.tsv'
        expected = export.encode(json.loads(self.accepted_bytes), self.fixture.input_pins[5])
        def fail_link(stage, destination):
            self.assertEqual(Path(stage).read_bytes(), expected)
            self.assertEqual(destination, output.resolve())
            raise OSError('synthetic link failure')
        with self.configured(), patch.object(export.os, 'link', side_effect=fail_link):
            with self.assertRaisesRegex(OSError, 'synthetic link failure'):
                export.publish(*self.inputs, output)
        self.assertFalse(output.exists())
        self.assertEqual(output.with_suffix('.qualification.json').read_bytes(), self.accepted_bytes)
        self.assertEqual(list(self.root.glob('.hit-counts-*')), [])

    def test_cli_uses_same_authenticated_exclusive_publication(self):
        output = self.root / 'cli.tsv'
        with self.configured(), patch('sys.argv', ['hit-training-counts', *map(str, self.inputs), str(output)]), redirect_stdout(io.StringIO()) as stream:
            export.main()
        summary = json.loads(stream.getvalue())
        self.assertEqual(summary['format'], 'joint-hit-training-counts-v1')
        self.assertEqual(summary['training_events'], int(output.read_bytes().splitlines()[3].split(b'\t')[1]))
        self.assertEqual(output.with_suffix('.qualification.json').read_bytes(), self.accepted_bytes)


if __name__ == '__main__':
    unittest.main()
