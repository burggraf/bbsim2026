"""Literal source proxies, not canonical episodes or fitted probabilities."""
import copy
import csv
import io
import hashlib
import subprocess
import itertools
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from audits import contact_consequence_support as audit
from audits import runner_transitions as runner
from tests.test_joint_contact import fixture as source_fixture
from tests.test_runner_transitions import attached, child
from tests.test_runner_only_steals import fixture as runner_only_fixture


def fixture(family, mask=None, outs=0, key=1):
    """Use the real source schema, not Rust event-kind/trajectory names."""
    mask = (4 if family == 'SF' else 5) if mask is None else mask
    row, context = source_fixture('Fly' if family == 'SF' else 'GroundBall',
        key=key, mask=mask, outs=outs)
    row['game_id'] = context['game_id'] = 'ANA201904040'
    event_outs = 2 if family == 'GIDP' else 1
    runs = int(family == 'SF')
    row.update(outs=str(event_outs), runs=str(runs),
        raw_play={'SF': '7/SF', 'SH': '13/SH', 'GIDP': '64(1)3/GDP'}[family])
    context.update(outs=str(event_outs), state_outs=str(event_outs),
        batting_outs=str(event_outs), putouts=str(event_outs), runs=str(runs),
        state_runs=str(runs), gidp=str(int(family == 'GIDP')),
        dp=str(int(family == 'GIDP')), derived_bunts=str(int(family == 'SH')))
    if family != 'SF':
        credits = [(1, 'Assist'), (3, 'Putout')] if family == 'SH' else [
            (6, 'Assist'), (4, 'Putout'), (4, 'Assist'), (3, 'Putout')]
        row['credits'] = json.dumps([dict(sequence_id=i+1, fielding_position=p,
            fielding_play=kind) for i, (p, kind) in enumerate(credits)])
        context['assists'] = str(sum(kind == 'Assist' for _, kind in credits))
    roles = [('Batter', context['batter_id'])] + [(role, context[role.lower()])
        for i, role in enumerate(('First', 'Second', 'Third')) if mask & (1 << i)]
    children = []
    for role, identity in roles:
        target = ('Out' if role == 'Batter' or family == 'GIDP' and role == 'First'
            else 'Home' if family == 'SF' and role == 'Third'
            else 'Second' if family == 'SH' and role == 'First' else role)
        children.append(child(role, identity, base_end=target, is_out=target == 'Out',
            run_scored_flag=target == 'Home'))
    boundary = outs + event_outs >= 3
    after_mask = mask & ~4 if family == 'SF' else (mask & ~1) | 2 if family == 'SH' else mask & ~1
    context.update(after_outs=str(outs + event_outs),
        after_base_mask='' if boundary else str(after_mask),
        frame_end='true' if boundary else 'false')
    return attached((row, context), children)


class LiteralShapes(unittest.TestCase):
    def test_positive_fixtures_pass_unchanged_complete_source_validation(self):
        cases = [('SF', mask, outs) for mask, outs in itertools.product((4, 5, 6, 7), (0, 1))]
        cases += [(family, mask, outs) for family, mask, outs in
            itertools.product(('SH', 'GIDP'), (1, 5), (0, 1))]
        for family, mask, outs in cases:
            row, context = fixture(family, mask, outs)
            with self.subTest(family=family, mask=mask, outs=outs):
                source = runner.inventory([row], [context], ['ANA201904040'])
                self.assertEqual(source['totals']['real_events'], 1)

    def test_sf_literal_shape_is_not_a_clean_causal_certificate(self):
        row, context = fixture('SF')
        original = copy.deepcopy((row, context))
        value = audit.classify(row, context)
        self.assertEqual(value['labels'], ['SF'])
        self.assertEqual(value['by_family']['SF']['shape'], 'complete_literal')
        self.assertEqual(value['causal_certification'], 'unavailable')
        self.assertEqual((row, context), original)

    def test_sh_literal_shape_retains_optional_held_third(self):
        row, context = fixture('SH')
        value = audit.classify(row, context)
        self.assertEqual(value['labels'], ['SH'])
        self.assertEqual(value['by_family']['SH']['shape'], 'complete_literal')
        self.assertEqual([(v['baserunner'], v['base_end']) for v in value['runners']],
            [('Batter', 'Out'), ('First', 'Second'), ('Third', 'Third')])

    def test_gidp_unit_and_two_retired_roles_do_not_certify_order(self):
        row, context = fixture('GIDP', outs=1)
        value = audit.classify(row, context)
        self.assertEqual(value['labels'], ['GIDP'])
        self.assertEqual(value['by_family']['GIDP']['shape'], 'complete_literal')
        self.assertEqual(value['causal_certification'], 'unavailable')

    def test_missing_retired_endpoint_is_not_rewritten_as_out(self):
        row, context = fixture('SF')
        children = json.loads(row['runners'])
        children[0]['base_end'] = None
        row['runners'] = json.dumps(children)
        value = audit.classify(row, context)
        self.assertEqual(value['by_family']['SF']['shape'],
            'compatible_with_missing_retired_endpoint')
        self.assertIsNone(value['runners'][0]['base_end'])
        self.assertTrue(value['runners'][0]['is_out'])
        self.assertEqual(value['causal_certification'], 'unavailable')

    def test_missing_safe_endpoint_is_never_held(self):
        row, context = fixture('SH')
        children = json.loads(row['runners'])
        children[-1]['base_end'] = None
        row['runners'] = json.dumps(children)
        value = audit.classify(row, context)
        self.assertEqual(value['by_family']['SH']['shape'], 'contradicted_or_incomplete')
        self.assertIsNone(value['runners'][-1]['base_end'])


class ClassificationRegressions(unittest.TestCase):
    def test_cartesian_before_masks_and_outs_have_exact_supported_geometry(self):
        for family, mask, outs in itertools.product(('SF', 'SH', 'GIDP'), range(8), range(3)):
            allowed = mask in ((4, 5, 6, 7) if family == 'SF' else (1, 5)) and outs in (0, 1)
            with self.subTest(family=family, mask=mask, outs=outs):
                value = audit.classify(*fixture(family, mask, outs))
                self.assertEqual(value['by_family'][family]['shape'] == 'complete_literal', allowed)

    def test_child_permutations_preserve_literal_role_identity_vectors(self):
        for family in ('SF', 'SH', 'GIDP'):
            row, context = fixture(family)
            expected = audit.classify(row, context)
            for children in itertools.permutations(json.loads(row['runners'])):
                permuted = dict(row, runners=json.dumps(children))
                original = copy.deepcopy((permuted, context))
                with self.subTest(family=family, roles=[c['baserunner'] for c in children]):
                    self.assertEqual(audit.classify(permuted, context), expected)
                    self.assertEqual((permuted, context), original)

    def test_overlapping_labels_are_all_retained_not_silently_prioritized(self):
        row, context = fixture('SF')
        row['raw_play'] = '7/SF/SH'
        context['gidp'] = '1'
        value = audit.classify(row, context)
        self.assertEqual(value['labels'], ['SF', 'SH', 'GIDP'])
        for family in value['labels']:
            self.assertIn('conflicting_labels', value['by_family'][family]['reasons'])

    def test_substrings_lowercase_and_bunt_dp_notation_do_not_fabricate_channels(self):
        row, context = fixture('SH')
        for marker in ('sf', 'SFX', 'SHX', 'BGDP', 'GDP'):
            row['raw_play'] = '13/' + marker
            with self.subTest(marker=marker):
                self.assertEqual(audit.classify(row, context)['labels'], [])
        row['raw_play'] = '13/SH'
        self.assertEqual(audit.classify(row, context)['labels'], ['SH'])

    def test_compound_atoms_keep_label_and_explicit_uncertainty(self):
        row, context = fixture('SH')
        row['raw_play'] = '13+WP/SH'
        value = audit.classify(row, context)
        self.assertEqual(value['labels'], ['SH'])
        self.assertIn('notation_uncertain', value['by_family']['SH']['reasons'])
        self.assertEqual(row['raw_play'], '13+WP/SH')

    def test_structural_unknown_does_not_erase_independent_gidp_unit(self):
        row, context = fixture('GIDP')
        row['raw_play'] = '64(1)3/GDP.'
        value = audit.classify(row, context)
        self.assertEqual(value['labels'], ['GIDP'])
        self.assertIn('notation_uncertain', value['by_family']['GIDP']['reasons'])

    def test_nullable_effect_flags_remain_unknown_not_clean_or_false(self):
        for family, field in itertools.product(('SF', 'SH', 'GIDP'),
                ('explicit_out_flag', 'advanced_on_error_flag', 'is_advance_attempt')):
            row, context = fixture(family)
            children = json.loads(row['runners'])
            children[-1][field] = None
            row['runners'] = json.dumps(children)
            with self.subTest(family=family, field=field):
                value = audit.classify(row, context)
                self.assertIsNone(value['runners'][-1][field])
                self.assertIn('runner_effect_or_unknown', value['by_family'][family]['reasons'])
                self.assertEqual(value['causal_certification'], 'unavailable')

    def test_missing_extra_and_unknown_identity_children_cannot_disappear(self):
        row, context = fixture('SH')
        original = json.loads(row['runners'])
        variants = [original[:-1], original + [child('Second', 'EXTRA')],
            [dict(c, runner_id=None) if c['baserunner'] == 'Third' else c for c in original]]
        for children in variants:
            with self.subTest(children=children):
                changed = dict(row, runners=json.dumps(children))
                value = audit.classify(changed, context)
                self.assertIn('incomplete_vector', value['by_family']['SH']['reasons'])
                self.assertEqual(value['by_family']['SH']['shape'], 'contradicted_or_incomplete')
                self.assertEqual(len(value['runners']), len(children))

    def test_known_out_bit_contradiction_is_not_missing_out_destination(self):
        row, context = fixture('SF')
        children = json.loads(row['runners'])
        children[0].update(base_end=None, is_out=False)
        row['runners'] = json.dumps(children)
        value = audit.classify(row, context)
        self.assertEqual(value['by_family']['SF']['shape'], 'contradicted_or_incomplete')
        self.assertIn('out_run_contradiction', value['by_family']['SF']['reasons'])
        self.assertFalse(value['runners'][0]['is_out'])
        self.assertIsNone(value['runners'][0]['base_end'])

    def test_after_state_evidence_cannot_silently_repair_source_snapshot(self):
        row, context = fixture('SH')
        context['after_base_mask'] = '4'
        before = copy.deepcopy((row, context))
        value = audit.classify(row, context)
        self.assertIn('after_state_evidence', value['by_family']['SH']['reasons'])
        self.assertEqual((row, context), before)
        self.assertEqual(value['causal_certification'], 'unavailable')

    def test_all_descriptive_reasons_are_retained_in_preregistered_order(self):
        row, context = fixture('SH', mask=7)
        row['raw_play'] = '13+WP/SH'
        children = json.loads(row['runners'])
        children[1].update(base_end=None, advanced_on_error_flag=True)
        row['runners'] = json.dumps(children)
        value = audit.classify(row, context)
        self.assertEqual(value['by_family']['SH']['reasons'],
            ['notation_uncertain', 'unsupported_before', 'incomplete_vector', 'runner_effect_or_unknown'])


def native_sf():
    """Independent literal native-row oracle for the default SF fixture."""
    return dict(event_key='1', game_id='ANA201904040', event_id='1',
        result='InPlayOut', pa=1, gidp=0, base_mask=4, outs_before=0,
        event_outs=1, event_runs=1, frame_end=False, after_outs=1,
        after_base_mask=0, batter_id='BATTER', first=None, second=None,
        third='T1', trajectory='Fly', raw_play='7/SF', flags=[], errors=0,
        runners=[child('Batter', 'BATTER', base_end='Out', is_out=True),
                 child('Third', 'T1', base_end='Home', run_scored_flag=True)])


def native_csv(rows):
    stream = io.StringIO()
    writer = csv.writer(stream)
    writer.writerow(['event_key', 'facts'])
    for key, facts in rows:
        writer.writerow([key, json.dumps(facts, allow_nan=False)])
    return stream.getvalue()


class NativeJointRegressions(unittest.TestCase):
    def test_strict_native_literal_record_roundtrip_retains_nulls(self):
        facts = native_sf()
        self.assertEqual(audit.read_native_joint(native_csv([('1', facts)])), {'1': facts})
        self.assertIsNone(facts['first'])
        self.assertFalse(facts['runners'][0]['explicit_out_flag'])

    def test_native_duplicate_grain_and_mismatched_event_key_refuse(self):
        facts = native_sf()
        for rows in [[('1', facts), ('1', facts)], [('2', facts)]]:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                audit.read_native_joint(native_csv(rows))

    def test_native_missing_fields_and_integer_boolean_aliases_refuse(self):
        variants = [dict(native_sf(), frame_end=0), dict(native_sf(), pa=True)]
        missing = native_sf(); del missing['gidp']; variants.append(missing)
        nested = native_sf(); nested['runners'][0]['is_out'] = 1; variants.append(nested)
        for facts in variants:
            with self.subTest(facts=facts), self.assertRaises(ValueError):
                audit.read_native_joint(native_csv([('1', facts)]))

    def test_native_nonstring_roles_are_structured_refusals(self):
        for role in (None, [], {}, 0, True):
            facts = native_sf()
            facts['runners'][0]['baserunner'] = role
            with self.subTest(role=role), self.assertRaises(ValueError):
                audit.read_native_joint(native_csv([('1', facts)]))

    def test_native_header_duplicate_json_keys_and_nan_refuse(self):
        for text in ['facts,event_key,event_key\n', 'event_key,facts\n1,"{}"\n',
                'event_key,facts\n1,"{\"\"x\"\":1,\"\"x\"\":2}"\n',
                'event_key,facts\n1,"{\"\"x\"\":NaN}"\n']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                audit.read_native_joint(text)


class InventoryRegressions(unittest.TestCase):
    def test_overlapping_family_memberships_do_not_double_count_union(self):
        row, context = fixture('SH')
        row['raw_play'] = '13/SH/SF'
        value = audit.inventory([row], [context], ['ANA201904040'])
        self.assertEqual(value['totals']['candidate_events'], 1)
        self.assertEqual(value['totals']['overlap_events'], 1)
        self.assertEqual(value['label_counts'], dict(SF=1, SH=1, GIDP=0))
        self.assertEqual(sum(v['events'] for v in value['families'].values()), 2)
        for family in ('SF', 'SH'):
            self.assertEqual(value['families'][family]['first_reasons'], {'conflicting_labels': 1})

    def test_no_play_signals_remain_separate_from_real_candidates(self):
        row, context = fixture('SF')
        np = dict(row, event_key='2', event_id='2', no_play_flag='true', result='',
            outs='0', runs='0', credits='[]', raw_play='NP/SF',
            runners=json.dumps([child('Third', 'T1', base_end=None, is_advance_attempt=None)]))
        census = runner.inventory([row, np], [context], ['ANA201904040'])
        value = audit.inventory([row, np], [context], ['ANA201904040'], expected_census=census)
        self.assertEqual(value['totals'], dict(source_events=2, real_events=1,
            no_play_events=1, candidate_events=1, outside_events=0, overlap_events=0))
        self.assertEqual(value['no_play_label_counts'], dict(SF=1, SH=0, GIDP=0))
        self.assertIsNone(json.loads(np['runners'])[0]['base_end'])

    def test_runner_only_modifier_signal_never_becomes_a_terminal_pa(self):
        row, context = runner_only_fixture(key=2)
        row['game_id'] = context['game_id'] = 'ANA201904040'
        row['raw_play'] = 'SB2/SF'
        value = audit.inventory([row], [context], ['ANA201904040'])
        self.assertEqual(value['totals']['candidate_events'], 1)
        self.assertEqual(value['families']['SF']['first_reasons'], {'not_terminal_pa': 1})
        self.assertEqual(value['families']['SF']['shapes']['contradicted_or_incomplete'], 1)
        self.assertEqual(context['pa'], '')

    def test_distinct_union_family_shapes_and_outside_partition_conserve(self):
        pairs = [fixture('SF', key=1), fixture('SH', key=2), fixture('GIDP', key=3),
                 fixture('SF', key=4), fixture('GIDP', key=5)]
        children = json.loads(pairs[3][0]['runners']); children[0]['base_end'] = None
        pairs[3][0]['runners'] = json.dumps(children)
        pairs[4][1]['gidp'] = '0'  # GDP notation is not a supplied GIDP unit.
        rows, contexts = [p[0] for p in pairs], [p[1] for p in pairs]
        census = runner.inventory(rows, contexts, ['ANA201904040'])
        value = audit.inventory(rows, contexts, ['ANA201904040'], expected_census=census)
        self.assertEqual(value['totals'], dict(source_events=5, real_events=5,
            no_play_events=0, candidate_events=4, outside_events=1, overlap_events=0))
        self.assertEqual(value['label_counts'], dict(SF=2, SH=1, GIDP=1))
        self.assertEqual(value['families']['SF']['shapes'], dict(complete_literal=1,
            compatible_with_missing_retired_endpoint=1, contradicted_or_incomplete=0))
        for family in ('SF', 'SH', 'GIDP'):
            group = value['families'][family]
            self.assertEqual(sum(group['shapes'].values()), group['events'])
            self.assertEqual(sum(group['first_reasons'].values()), group['events'])
        self.assertEqual(value['causal_certification'], 'unavailable')
        self.assertEqual(value, audit.inventory(list(reversed(rows)), list(reversed(contexts)),
            ['ANA201904040'], expected_census=census))

    def test_scope_pins_all_known_units_without_discarding_extra_census_totals(self):
        row, context = fixture('SF')
        census = runner.inventory([row], [context], ['ANA201904040'])
        scope = {name: copy.deepcopy(census[name]) for name in runner.COHORT}
        scope['raw_totals'] = {name: census['raw_totals'][name] for name in runner.COHORT['raw_totals']}
        self.assertTrue(set(census['raw_totals']) - set(scope['raw_totals']))
        value = audit.inventory([row], [context], ['ANA201904040'], expected_census=census, cohort=scope)
        self.assertEqual(value['raw_totals'], census['raw_totals'])
        for name in scope['raw_totals']:
            wrong = copy.deepcopy(scope); wrong['raw_totals'][name] += 1
            with self.subTest(name=name), patch.object(audit, 'classify', side_effect=AssertionError('measurement before scope validation')):
                with self.assertRaisesRegex(ValueError, 'cohort|units'):
                    audit.inventory([row], [context], ['ANA201904040'], expected_census=census, cohort=wrong)

    def test_whole_game_split_is_preserved_without_fitting(self):
        pairs = [fixture(family, key=i+1) for i, family in enumerate(('SF', 'SH', 'GIDP'))]
        games = ['SYNTHETIC_A', 'SYNTHETIC_HELDOUT_7', 'SYNTHETIC_EXCLUDED']
        for game, (row, context) in zip(games, pairs):
            row['game_id'] = context['game_id'] = game
        value = audit.inventory([p[0] for p in pairs], [p[1] for p in pairs], games[:2])
        self.assertEqual({k: v['candidate_events'] for k, v in value['splits'].items()},
            dict(training=1, heldout=1, excluded=1))
        self.assertEqual(value['excluded_game_ids'], ['SYNTHETIC_EXCLUDED'])
        self.assertEqual(value['causal_certification'], 'unavailable')

    def test_complete_source_validation_precedes_candidate_classification(self):
        row, context = fixture('SF')
        del context['season']
        with patch.object(audit, 'classify', side_effect=AssertionError('classification reached')):
            with self.assertRaisesRegex(ValueError, 'incomplete outcome export row'):
                audit.inventory([row], [context], ['ANA201904040'])

    def test_census_mismatch_precedes_candidate_classification(self):
        row, context = fixture('SF')
        census = runner.inventory([row], [context], ['ANA201904040'])
        census['totals']['real_events'] += 1
        with patch.object(audit, 'classify', side_effect=AssertionError('classification reached')):
            with self.assertRaisesRegex(ValueError, '[Cc]ensus'):
                audit.inventory([row], [context], ['ANA201904040'], expected_census=census)

    def test_independent_joint_equality_is_required_before_classification(self):
        row, context = fixture('SF')
        facts = native_sf(); facts['runners'][1]['base_end'] = None
        with patch.object(audit, 'classify', side_effect=AssertionError('classification reached')):
            with self.assertRaisesRegex(ValueError, '[Jj]oint'):
                audit.inventory([row], [context], ['ANA201904040'], native_joint={'1': facts})

    def test_matching_independent_joint_facts_and_census_are_accepted(self):
        row, context = fixture('SF')
        census = runner.inventory([row], [context], ['ANA201904040'])
        value = audit.inventory([row], [context], ['ANA201904040'],
            expected_census=census, native_joint={'1': native_sf()})
        self.assertEqual(value['totals']['candidate_events'], 1)
        self.assertEqual(value['label_counts'], dict(SF=1, SH=0, GIDP=0))

    def test_old_native_child_mismatch_is_not_waived(self):
        row, context = fixture('SF')
        with patch.object(audit, 'classify', side_effect=AssertionError('classification reached')):
            with self.assertRaises(ValueError):
                audit.inventory([row], [context], ['ANA201904040'], native_groups=[])


class PublicationRegressions(unittest.TestCase):
    def test_input_output_collision_never_changes_input_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); inputs = [root/f'input-{i}' for i in range(5)]
            for path in inputs: path.write_bytes(b'input bytes')
            with patch('subprocess.run', side_effect=AssertionError('native work before collision check')):
                with self.assertRaisesRegex(ValueError, 'overwrite|Refus|collision'):
                    audit.publish(*inputs, inputs[0])
            self.assertTrue(all(p.read_bytes() == b'input bytes' for p in inputs))

    def test_existing_file_directory_and_symlink_outputs_are_never_touched(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); marker = root/'marker'; marker.write_bytes(b'unchanged')
            targets = [root/'report.json', root/'occupied', root/'link.json']
            targets[0].write_bytes(b'prior report'); targets[1].mkdir(); targets[2].symlink_to(marker)
            for target in targets:
                with self.subTest(target=target), patch('subprocess.run',
                        side_effect=AssertionError('native work before output preflight')):
                    with self.assertRaisesRegex(ValueError, 'overwrite|Refus|occupied'):
                        audit.publish(*(root/f'missing-{i}' for i in range(5)), target)
            self.assertEqual(targets[0].read_bytes(), b'prior report')
            self.assertTrue(targets[1].is_dir()); self.assertTrue(targets[2].is_symlink())
            self.assertEqual(marker.read_bytes(), b'unchanged')

    def test_unaccepted_input_bytes_refuse_before_any_native_work_or_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); inputs = [root/f'input-{i}' for i in range(5)]
            for path in inputs: path.write_bytes(b'not an accepted input')
            target = root/'report.json'
            with patch('subprocess.run', side_effect=AssertionError('native work before pins')):
                with self.assertRaisesRegex(ValueError, 'digest|[Uu]nsupported|accepted'):
                    audit.publish(*inputs, target)
            self.assertFalse(target.exists())
            self.assertTrue(all(p.read_bytes() == b'not an accepted input' for p in inputs))

    def test_repository_output_refuses_before_missing_inputs_or_native_work(self):
        target = Path(__file__).resolve().parents[1]/'contact-audit-must-not-be-written.json'
        self.assertFalse(target.exists())
        with patch('subprocess.run', side_effect=AssertionError('native work inside repo')):
            with self.assertRaisesRegex(ValueError, 'outside repository'):
                audit.publish(*[Path('/not-a-real-source')]*5, target)
        self.assertFalse(target.exists())

    def test_cli_missing_arguments_refuses_without_output(self):
        result = subprocess.run([sys.executable, '-Werror', '-m', 'audits.contact_consequence_support'],
            capture_output=True, text=True, check=False, timeout=15)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('usage:', result.stderr)


class NativeSQLRegressions(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix='contact-support-native-')
        self.addCleanup(temp.cleanup)
        self.directory = Path(temp.name)
        self.snapshot, self.detail = (self.directory / n for n in ('snapshot.duckdb', 'detail.duckdb'))
        self.query = Path(__file__).resolve().parents[1] / 'audits/2019-contact-consequence-support.sql'
        self.sql(self.snapshot, """CREATE TABLE events(event_key BIGINT,game_id VARCHAR,event_id INTEGER,season INTEGER,no_play_flag BOOLEAN,plate_appearance_result VARCHAR,outs INTEGER,base_state INTEGER,outs_on_play INTEGER,runs_on_play INTEGER,batted_trajectory VARCHAR,batter_id VARCHAR);
INSERT INTO events VALUES(1,'ANA201904040',1,2019,false,'InPlayOut',0,4,1,1,'Fly','BATTER'),(2,'ANA201904040',2,2019,true,NULL,NULL,NULL,0,0,NULL,NULL);
CREATE TABLE states(event_key BIGINT,frame_end_flag BOOLEAN,outs_end INTEGER,base_state_end INTEGER,runner_first_id_start VARCHAR,runner_second_id_start VARCHAR,runner_third_id_start VARCHAR);
INSERT INTO states VALUES(1,false,1,0,NULL,NULL,'T1');
CREATE TABLE batting(event_key BIGINT,plate_appearances INTEGER,grounded_into_double_plays INTEGER);
INSERT INTO batting VALUES(1,1,0);
CREATE TABLE fielding(event_key BIGINT,errors INTEGER);INSERT INTO fielding VALUES(1,0);""")
        self.sql(self.detail, """CREATE TABLE selected_events(event_key BIGINT,game_id VARCHAR,event_id INTEGER,no_play_flag BOOLEAN,plate_appearance_result VARCHAR,outs_on_play INTEGER,runs_on_play INTEGER,batted_trajectory VARCHAR);
INSERT INTO selected_events VALUES(1,'ANA201904040',1,false,'InPlayOut',1,1,'Fly'),(2,'ANA201904040',2,true,NULL,0,0,NULL);
CREATE TABLE stg_event_audit(event_key BIGINT,game_id VARCHAR,event_id INTEGER,raw_play VARCHAR);
INSERT INTO stg_event_audit VALUES(1,'ANA201904040',1,'7/SF'),(2,'ANA201904040',2,'NP');
CREATE TABLE stg_event_flags(event_key BIGINT,sequence_id INTEGER,flag VARCHAR);
INSERT INTO stg_event_flags VALUES(1,1,'OpaqueFlag'),(1,2,'SF');
CREATE TABLE stg_event_baserunners(event_key BIGINT,game_id VARCHAR,event_id INTEGER,baserunner VARCHAR,runner_id VARCHAR,baserunning_play_type VARCHAR,is_out BOOLEAN,run_scored_flag BOOLEAN,explicit_out_flag BOOLEAN,advanced_on_error_flag BOOLEAN,attempted_advance_to_base VARCHAR,base_end VARCHAR,is_advance_attempt BOOLEAN);
INSERT INTO stg_event_baserunners VALUES(1,'ANA201904040',1,'Batter','BATTER','Advance',true,false,false,false,NULL,'Out',false),(1,'ANA201904040',1,'Third','T1','Advance',false,true,false,NULL,NULL,'Home',false),(2,'ANA201904040',2,'First',NULL,'Advance',false,false,NULL,NULL,NULL,'Second',NULL);""")

    @staticmethod
    def sql(db, sql):
        return subprocess.run(['duckdb','-init','/dev/null','-bail','-batch',str(db)],
            input=sql, text=True, capture_output=True, check=True)

    def execute(self):
        setup = f"ATTACH '{self.detail}' AS detail (READ_ONLY); SET enable_external_access=false;"
        return subprocess.run(['duckdb','-init','/dev/null','-readonly','-bail','-batch','-csv',
            '-nullvalue','','-cmd',setup,str(self.snapshot),'-f',str(self.query)],
            text=True, capture_output=True, check=False)

    def facts(self):
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, '')
        values = audit.read_native_joint(result.stdout)
        self.assertEqual(set(values), {'1'})
        return values['1']

    def refused(self):
        result = self.execute()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('unsupported contact consequence joint facts', result.stderr)

    def test_native_literal_joint_aggregates_children_before_join_and_preserves_bytes(self):
        before = [hashlib.sha256(p.read_bytes()).hexdigest() for p in (self.snapshot, self.detail)]
        expected = native_sf()
        expected['flags'] = [dict(sequence_id=1, flag='OpaqueFlag'), dict(sequence_id=2, flag='SF')]
        expected['runners'][1]['advanced_on_error_flag'] = None
        self.assertEqual(self.facts(), expected)
        self.assertEqual(self.execute().stdout, self.execute().stdout)
        self.assertEqual(before, [hashlib.sha256(p.read_bytes()).hexdigest() for p in (self.snapshot, self.detail)])

    def test_native_nullable_half_boundary_and_literal_null_id_are_not_normalized(self):
        self.sql(self.snapshot, "UPDATE events SET outs=2 WHERE event_key=1;UPDATE states SET frame_end_flag=true,outs_end=NULL,base_state_end=NULL,runner_third_id_start='NULL';")
        self.sql(self.detail, "UPDATE stg_event_baserunners SET runner_id='NULL' WHERE baserunner='Third';")
        facts = self.facts()
        self.assertIsNone(facts['after_outs']); self.assertIsNone(facts['after_base_mask'])
        self.assertTrue(facts['frame_end']); self.assertEqual(facts['outs_before'], 2)
        self.assertEqual(facts['third'], 'NULL'); self.assertEqual(facts['runners'][1]['runner_id'], 'NULL')

    def test_native_guards_all_parent_fanout(self):
        for table in ('events', 'states', 'batting', 'fielding'):
            with self.subTest(table=table):
                self.sql(self.snapshot, f'INSERT INTO {table} SELECT * FROM {table} WHERE event_key=1;')
                try:
                    self.refused()
                finally:
                    self.sql(self.snapshot, f'DELETE FROM {table} WHERE rowid=(SELECT max(rowid) FROM {table} WHERE event_key=1);')

    def test_native_orphan_child_refuses(self):
        self.sql(self.detail, "UPDATE stg_event_baserunners SET event_key=999 WHERE baserunner='Third';")
        self.refused()

    def test_native_duplicate_role_and_missing_required_bit_refuse(self):
        self.sql(self.detail, "INSERT INTO stg_event_baserunners SELECT * FROM stg_event_baserunners WHERE baserunner='Third';")
        self.refused()
        self.sql(self.detail, "DELETE FROM stg_event_baserunners WHERE rowid=(SELECT max(rowid) FROM stg_event_baserunners);UPDATE stg_event_baserunners SET is_out=NULL WHERE baserunner='Third';")
        self.refused()

    def test_native_duplicate_sequence_and_missing_flag_refuse(self):
        self.sql(self.detail, 'INSERT INTO stg_event_flags SELECT * FROM stg_event_flags WHERE sequence_id=1;')
        self.refused()
        self.sql(self.detail, 'DELETE FROM stg_event_flags WHERE rowid=(SELECT max(rowid) FROM stg_event_flags);UPDATE stg_event_flags SET flag=NULL WHERE sequence_id=1;')
        self.refused()

    def test_native_guard_executes_without_children_to_hide_parent_mismatch(self):
        self.sql(self.detail, 'DELETE FROM stg_event_baserunners;')
        self.sql(self.snapshot, "UPDATE events SET game_id='OTHER' WHERE event_key=1;")
        self.refused()

    def test_native_missing_real_state_or_boundary_bit_refuses(self):
        self.sql(self.snapshot, 'UPDATE states SET frame_end_flag=NULL;')
        self.refused()
        self.sql(self.snapshot, 'DELETE FROM states;')
        self.refused()


if __name__ == '__main__':
    unittest.main()
