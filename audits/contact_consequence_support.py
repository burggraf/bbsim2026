"""Literal source support, never a clean causal episode or fitted occurrence rate.

Read-only offline implementation; literal support never certifies a clean causal episode.
"""
from collections import Counter
import argparse
import copy
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
from audits import raw_mechanisms as raw
from audits import runner_transitions as runner

FAMILIES = ('SF', 'SH', 'GIDP')
ROLES = ('Batter', 'First', 'Second', 'Third')
REASONS = ('not_terminal_pa', 'conflicting_labels', 'notation_uncertain',
    'unsupported_before', 'event_units', 'incomplete_vector', 'out_run_contradiction',
    'endpoint_shape', 'runner_effect_or_unknown', 'after_state_evidence')


def _optional_number(context, name):
    value = None if context is None else context.get(name)
    return None if value in (None, '') else int(value)


def _labels(lexical, context):
    return [family for family in FAMILIES if
        (family in ('SF', 'SH') and family in lexical['modifiers']) or
        (family == 'GIDP' and _optional_number(context, 'gidp') == 1)]


def classify(row, context):
    """Compare literal vectors only; full source validation belongs to inventory."""
    lexical = raw.lex(row['raw_play'])
    labels = _labels(lexical, context)
    children = sorted(raw.child_rows(row['runners'], raw.RUNNER_FIELDS),
        key=lambda value: ROLES.index(value['baserunner']))
    c = context or {}
    mask, outs = (_optional_number(context, name) for name in ('base_mask', 'outs_before'))
    observed = {value['baserunner']: value for value in children}
    expected = {'Batter': c.get('batter_id')}
    expected.update({role: c.get(role.lower()) for role in ROLES[1:] if c.get(role.lower())})
    identity_gap = (len(children) != len(expected) or set(observed) != set(expected) or
        any(not value['runner_id'] or value['runner_id'] != expected.get(value['baserunner'])
            for value in children))
    by_family = {}
    for family in labels:
        allowed_masks = (4, 5, 6, 7) if family == 'SF' else (1, 5)
        event_outs, event_runs = (2 if family == 'GIDP' else 1), int(family == 'SF')
        targets = {role: ('Out' if role == 'Batter' or family == 'GIDP' and role == 'First'
            else 'Home' if family == 'SF' and role == 'Third'
            else 'Second' if family == 'SH' and role == 'First' else role)
            for role in expected}
        missing_retired = missing_safe = mismatch = bit_conflict = False
        for value in children:
            target = targets.get(value['baserunner'])
            endpoint = value['base_end']
            if target is None:
                continue
            bit_conflict |= (value['is_out'] != (target == 'Out') or
                value['run_scored_flag'] != (target == 'Home') or
                endpoint == 'Out' and not value['is_out'] or
                endpoint == 'Home' and not value['run_scored_flag'])
            if endpoint is None:
                if target == 'Out' and value['is_out']:
                    missing_retired = True
                else:
                    missing_safe = True
            elif endpoint != target:
                mismatch = True
        before_bad = mask not in allowed_masks or outs not in (0, 1)
        units_bad = int(row['outs']) != event_outs or int(row['runs']) != event_runs
        not_pa = row['no_play_flag'] == 'true' or c.get('pa') != '1'
        geometry_bad = not_pa or before_bad or units_bad or identity_gap or missing_safe or mismatch or bit_conflict
        shape = ('contradicted_or_incomplete' if geometry_bad else
            'compatible_with_missing_retired_endpoint' if missing_retired else 'complete_literal')
        after_evidence = False
        if not geometry_bad:
            total_outs = outs + event_outs
            boundary = c.get('frame_end') == 'true'
            after_evidence = (boundary != (total_outs == 3) or
                _optional_number(context, 'after_outs') != total_outs)
            if not boundary:
                target_mask = mask & ~4 if family == 'SF' else (mask & ~1) | 2 if family == 'SH' else mask & ~1
                after_evidence |= _optional_number(context, 'after_base_mask') != target_mask
        failed = dict(not_terminal_pa=not_pa, conflicting_labels=len(labels) > 1,
            notation_uncertain=(lexical['status'] != 'split' or bool(lexical['base_separators']) or
                len(lexical['base_atoms']) != 1 or
                any(raw.root_family(atom) != 'numeric_fielding_notation' for atom in lexical['base_atoms']) or
                any(modifier not in ('SF', 'SH', 'GDP') for modifier in lexical['modifiers'])),
            unsupported_before=before_bad, event_units=units_bad,
            incomplete_vector=identity_gap or missing_safe or missing_retired,
            out_run_contradiction=bit_conflict, endpoint_shape=mismatch,
            runner_effect_or_unknown=(_optional_number(context, 'errors') != 0 or
                any(value['baserunning_play_type'] != 'Advance' or
                    value['attempted_advance_to_base'] is not None or
                    any(value[field] is not False for field in
                        ('explicit_out_flag', 'advanced_on_error_flag', 'is_advance_attempt'))
                    for value in children)),
            after_state_evidence=after_evidence)
        by_family[family] = dict(shape=shape, reasons=[reason for reason in REASONS if failed[reason]])
    return dict(labels=labels, by_family=by_family, runners=children,
        causal_certification='unavailable')


JOINT_FIELDS = frozenset(('event_key game_id event_id result pa gidp base_mask '
    'outs_before event_outs event_runs frame_end after_outs after_base_mask '
    'batter_id first second third trajectory raw_play flags runners errors').split())


def _reject_constant(value):
    raise ValueError('Nonfinite native joint JSON constant: ' + value)


def _validate_joint(facts):
    if not isinstance(facts, dict) or set(facts) != JOINT_FIELDS:
        raise ValueError('Invalid native joint fields')
    for name in ('event_key', 'game_id', 'event_id', 'batter_id', 'raw_play'):
        if not isinstance(facts[name], str) or not facts[name]:
            raise ValueError('Invalid native joint literal identity/text')
    if not facts['event_key'].isdigit() or int(facts['event_key']) < 1:
        raise ValueError('Invalid native joint event key')
    for name in ('result', 'first', 'second', 'third', 'trajectory'):
        if facts[name] is not None and not isinstance(facts[name], str):
            raise ValueError('Invalid native joint optional literal text')
    for name, maximum, nullable in [('pa', 1, True), ('gidp', 1, True),
            ('base_mask', 7, False), ('outs_before', 2, False),
            ('event_outs', 3, False), ('event_runs', None, False),
            ('after_outs', 3, True), ('after_base_mask', 7, True), ('errors', None, True)]:
        value = facts[name]
        if value is None and nullable:
            continue
        if type(value) is not int or value < 0 or maximum is not None and value > maximum:
            raise ValueError('Invalid native joint integer/NULL field')
    if type(facts['frame_end']) is not bool:
        raise ValueError('Invalid native joint boundary bit')
    flags = raw.child_rows(json.dumps(facts['flags'], allow_nan=False), raw.FLAG_FIELDS)
    for value in flags:
        if not isinstance(value['flag'], str):
            raise ValueError('Invalid native joint opaque flag')
    facts['flags'] = raw.sequence_rows(flags)
    children = raw.child_rows(json.dumps(facts['runners'], allow_nan=False), raw.RUNNER_FIELDS)
    roles = [v['baserunner'] for v in children]
    if any(not isinstance(role, str) or role not in ROLES for role in roles) or len(roles) != len(set(roles)):
        raise ValueError('Invalid native joint runner role grain')
    for value in children:
        if any(type(value[name]) is not bool for name in ('is_out', 'run_scored_flag')):
            raise ValueError('Invalid native joint runner out/run bits')
        if any(value[name] is not None and type(value[name]) is not bool for name in
                ('explicit_out_flag', 'advanced_on_error_flag', 'is_advance_attempt')):
            raise ValueError('Invalid native joint nullable effect bit')
        if any(value[name] is not None and not isinstance(value[name], str) for name in
                ('runner_id', 'baserunning_play_type', 'base_end', 'attempted_advance_to_base')):
            raise ValueError('Invalid native joint optional runner literal')
    facts['runners'] = sorted(children, key=lambda v: ROLES.index(v['baserunner']))
    return facts


def read_native_joint(text):
    """Strict full-real-event map; no causal inference or integer/boolean aliases."""
    reader = csv.DictReader(io.StringIO(text), strict=True)
    if reader.fieldnames is None or len(reader.fieldnames) != 2 or set(reader.fieldnames) != {'event_key', 'facts'}:
        raise ValueError('Invalid native joint CSV header')
    result = {}
    try:
        for row in reader:
            if set(row) != {'event_key', 'facts'} or any(v is None for v in row.values()):
                raise ValueError('Invalid native joint CSV row')
            facts = _validate_joint(json.loads(row['facts'], object_pairs_hook=raw.unique_object,
                parse_constant=_reject_constant))
            key = row['event_key']
            if key != facts['event_key'] or key in result:
                raise ValueError('Duplicate/mismatched native joint event key')
            result[key] = facts
    except csv.Error as error:
        raise ValueError('Invalid native joint CSV') from error
    return result


def _joint_facts(row, context):
    """Encode original literal inputs independently of family classification."""
    facts = {name: row[name] for name in ('event_key', 'game_id', 'event_id', 'raw_play')}
    facts.update({name: context[name] if context[name] != '' else None for name in
        ('result', 'batter_id', 'first', 'second', 'third', 'trajectory')})
    facts.update({name: _optional_number(context, name) for name in
        ('pa', 'gidp', 'base_mask', 'outs_before', 'after_outs', 'after_base_mask', 'errors')})
    facts.update(event_outs=int(row['outs']), event_runs=int(row['runs']),
        frame_end=context['frame_end'] == 'true',
        flags=raw.child_rows(row['flags'], raw.FLAG_FIELDS),
        runners=raw.child_rows(row['runners'], raw.RUNNER_FIELDS))
    return _validate_joint(facts)


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups=None,
              expected_census=None, native_joint=None, cohort=None):
    """Complete census/scope/native reconciliation precedes any new label counts."""
    rows, outcomes = tuple(raw_rows), tuple(outcome_rows)
    source = runner.inventory(rows, outcomes, supported_game_ids, native_groups=native_groups)
    if expected_census is not None and (not isinstance(expected_census, dict) or
            source != {k: v for k, v in expected_census.items() if k != 'provenance'}):
        raise ValueError('Contact support census reconciliation failed')
    if cohort is not None:
        if (not isinstance(cohort, dict) or set(cohort) != set(runner.COHORT) or
                not isinstance(cohort['raw_totals'], dict) or
                set(cohort['raw_totals']) != set(runner.COHORT['raw_totals'])):
            raise ValueError('Unsupported contact source cohort/units')
        observed_scope = {name: source[name] for name in runner.COHORT}
        # COHORT pins the original source-unit subset, not every auxiliary census
        # finding. Complete census equality above still binds all additional totals.
        observed_scope['raw_totals'] = {name: source['raw_totals'].get(name) for name in cohort['raw_totals']}
        if observed_scope != cohort:
            raise ValueError('Unsupported contact source cohort/units')
    contexts = {r['event_key']: r for r in outcomes}
    real = [r for r in rows if r['no_play_flag'] == 'false']
    if native_joint is not None:
        if not isinstance(native_joint, dict) or any(type(k) is not str for k in native_joint):
            raise ValueError('Invalid native joint map')
        if set(native_joint) != {r['event_key'] for r in real}:
            raise ValueError('Independent native joint reconciliation failed')
        for row in real:
            key = row['event_key']
            if _validate_joint(copy.deepcopy(native_joint[key])) != _joint_facts(row, contexts[key]):
                raise ValueError('Independent native joint reconciliation failed')
    shapes = ('complete_literal', 'compatible_with_missing_retired_endpoint', 'contradicted_or_incomplete')
    families = {f: dict(events=0, shapes=dict.fromkeys(shapes, 0), first_reasons=Counter(),
        all_reasons=Counter()) for f in FAMILIES}
    totals = dict(source_events=len(rows), real_events=len(real),
        no_play_events=len(rows)-len(real), candidate_events=0, outside_events=0, overlap_events=0)
    labels, no_play_labels = dict.fromkeys(FAMILIES, 0), dict.fromkeys(FAMILIES, 0)
    split_of = {game: ('excluded' if game in source['excluded_game_ids'] else
        'heldout' if hashlib.sha256(game.encode('utf-8')).hexdigest()[0] in '012' else 'training')
        for game in {r['game_id'] for r in rows}}
    splits = {s: dict(real_events=0, candidate_events=0, outside_events=0)
        for s in ('training', 'heldout', 'excluded')}
    cells, frames = Counter(), Counter()
    eligible = {s: dict.fromkeys(FAMILIES, 0) for s in splits}
    for row in rows:
        if row['no_play_flag'] == 'true':
            for family in _labels(raw.lex(row['raw_play']), None):
                no_play_labels[family] += 1
            continue
        context = contexts[row['event_key']]
        split = split_of[row['game_id']]
        splits[split]['real_events'] += 1
        mask, outs = int(context['base_mask']), int(context['outs_before'])
        if context['pa'] == '1':
            frame = dict(split=split, base_mask=mask, outs_before=outs)
            frames[json.dumps(frame, sort_keys=True)] += 1
            for family in FAMILIES:
                if mask in ((4, 5, 6, 7) if family == 'SF' else (1, 5)) and outs in (0, 1):
                    eligible[split][family] += 1
        value = classify(row, context)
        if not value['labels']:
            totals['outside_events'] += 1
            splits[split]['outside_events'] += 1
            continue
        totals['candidate_events'] += 1
        splits[split]['candidate_events'] += 1
        totals['overlap_events'] += int(len(value['labels']) > 1)
        for family in value['labels']:
            labels[family] += 1
            group = families[family]
            group['events'] += 1
            result = value['by_family'][family]
            group['shapes'][result['shape']] += 1
            group['first_reasons'][result['reasons'][0] if result['reasons'] else 'none'] += 1
            group['all_reasons'].update(result['reasons'])
            facts = _joint_facts(row, context)
            cell = {name: facts[name] for name in ('result', 'pa', 'gidp', 'trajectory', 'base_mask',
                'outs_before', 'event_outs', 'event_runs', 'frame_end', 'after_outs', 'after_base_mask', 'errors', 'flags')}
            cell.update(family=family, split=split, **result,
                runners=[dict({k: v for k, v in child.items() if k != 'runner_id'},
                    identity_present=child['runner_id'] is not None) for child in facts['runners']])
            cells[json.dumps(cell, sort_keys=True)] += 1
    if (totals['candidate_events'] + totals['outside_events'] != totals['real_events'] or
            sum(v['real_events'] for v in splits.values()) != totals['real_events'] or
            sum(v['candidate_events'] for v in splits.values()) != totals['candidate_events'] or
            sum(cells.values()) != sum(labels.values()) or
            sum(frames.values()) != source['totals']['terminal_pa']):
        raise ValueError('Contact source distinct/split/cell/PA partition conservation failed')
    for family, group in families.items():
        if group['events'] != labels[family] or sum(group['shapes'].values()) != group['events'] or sum(group['first_reasons'].values()) != group['events']:
            raise ValueError('Contact source family/reason partition conservation failed')
        for name in ('first_reasons', 'all_reasons'):
            group[name] = dict(sorted(group[name].items()))
    return dict(format='2019-contact-consequence-support-v1',
        basis='observed SF/SH outer modifiers and supplied GIDP unit;literal shapes,not clean legal episodes or fitted rates',
        games=source['games'], supported_games=source['supported_games'], excluded_game_ids=source['excluded_game_ids'],
        totals=totals, label_counts=labels, no_play_label_counts=no_play_labels, families=families, splits=splits,
        causal_certification='unavailable', validation=dict(census=expected_census is not None,
            native_child=native_groups is not None, native_joint=native_joint is not None, cohort=cohort is not None),
        source_totals=source['totals'], raw_totals=source['raw_totals'], outcome_totals=source['outcome_totals'],
        supplied_credit_units=source['supplied_credit_units'], raw_notation=source['raw_notation'], source_findings=source['findings'],
        opportunity_basis='source PA before-state frames;not tactical attempts,legal mass or player ability',
        eligible_before_state_pa_frames=eligible,
        before_state_pa_frames=[dict(json.loads(key), frames=n) for key, n in sorted(frames.items())],
        context_basis='labeled family memberships;overlaps may contribute more than one cell per distinct event',
        contexts=[dict(json.loads(key), events=n) for key, n in sorted(cells.items())])


CENSUS_SHA = 'bea6d15b199aa2835bfab4334bf61b1b3de9fc3f382285e6b542f564a0ef098a'
EXPORT_SHA = ('bef8108455784113702dd4003172ac99162d24dfb8522da4e2a0c53fa16dbf02',
    'ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a',
    '40c08edc432ed8480e07a660b6896236316c9cb3235ffae6acf0ce9c9d21c199',
    'fe4d84e1c51ea945c3538b4bcda43bc8b94ba8eeb08376cf7aa7d66d4d431041')


def _sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def publish(snapshot, supplement, lineups, manifest, census, output):
    """Exclusive offline audit; no final report before all literal reconciliations."""
    output = Path(output)
    if output.is_symlink():
        raise ValueError('Refusing to overwrite contact audit evidence')
    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    source_output = output.with_suffix('.source.json')
    labels = ('lineup', 'events', 'raw', 'groups')
    source_artifacts = [source_output] + [source_output.with_suffix('.' + label + suffix)
        for label in labels for suffix in ('.csv', '.stderr', '.query.sql')]
    native, error, effective = [output.with_suffix(suffix) for suffix in
        ('.joint.csv', '.joint.stderr', '.joint.query.sql')]
    artifacts = [output, native, error, effective] + source_artifacts
    if len(set(artifacts)) != len(artifacts) or any(p.exists() or p.is_symlink() for p in artifacts):
        raise ValueError('Refusing to overwrite contact audit report/native evidence')
    inputs = list(map(Path, (snapshot, supplement, lineups, manifest, census)))
    if any(not p.is_file() for p in inputs) or tuple(map(_sha, inputs)) != runner.INPUT_SHA + (CENSUS_SHA,):
        raise ValueError('Unsupported accepted contact audit input digest')
    inputs += [inputs[4].with_suffix('.' + label + '.csv') for label in labels]
    pins = runner.INPUT_SHA + (CENSUS_SHA,) + EXPORT_SHA
    if any(not p.is_file() for p in inputs) or tuple(map(_sha, inputs)) != pins:
        raise ValueError('Unsupported accepted contact audit cached export digest')
    accepted = json.loads(inputs[4].read_text(), object_pairs_hook=raw.unique_object, parse_constant=_reject_constant)
    recipes = [Path(__file__), Path(__file__).with_name('2019-contact-consequence-support.sql')]
    bound_inputs = inputs + recipes
    bound_pins = tuple(map(_sha, bound_inputs))
    fresh = runner.publish(*inputs[:4], source_output)
    if fresh != accepted:
        raise ValueError('Regenerated accepted census/provenance reconciliation failed')
    exports = [source_output.with_suffix('.' + label + '.csv') for label in labels]
    if tuple(map(_sha, exports)) != EXPORT_SHA:
        raise ValueError('Regenerated cached source export reconciliation failed')
    setup = "ATTACH '" + str(inputs[1].resolve()).replace("'", "''") + "' AS detail (READ_ONLY); SET enable_external_access=false;"
    text = recipes[1].read_text()
    with effective.open('x', encoding='utf-8') as stream:
        stream.write(text)
    with native.open('x', encoding='utf-8') as stdout, error.open('x', encoding='utf-8') as stderr:
        result = subprocess.run(['duckdb', '-init', '/dev/null', '-readonly', '-bail', '-batch', '-csv',
            '-nullvalue', '', '-cmd', setup, str(inputs[0].resolve())], input=text,
            stdout=stdout, stderr=stderr, text=True, check=False, timeout=300)
    if result.returncode:
        raise RuntimeError(f'Native contact joint query failed ({result.returncode}); see {error}')
    # Bind every newly consumed export before reading. No new family count yet.
    consumed = [source_output, *exports, native, effective]
    bound_inputs += consumed
    bound_pins += tuple(map(_sha, consumed))
    with exports[1].open(newline='', encoding='utf-8') as stream:
        reader = csv.DictReader(stream, strict=True)
        if reader.fieldnames is None or len(reader.fieldnames) != len(runner.outcome.FIELDS) or set(reader.fieldnames) != runner.outcome.FIELDS:
            raise ValueError('Invalid contact outcome export header')
        outcomes = list(reader)
    with exports[2].open(newline='', encoding='utf-8') as stream:
        rows = list(raw.read_export(stream))
    accepted_lineups = json.loads(inputs[2].read_text(), object_pairs_hook=raw.unique_object, parse_constant=_reject_constant)
    report = inventory(rows, outcomes, accepted_lineups['candidate_game_ids'],
        native_groups=runner.read_native_groups(exports[3].read_text()), expected_census=accepted,
        native_joint=read_native_joint(native.read_text()), cohort=runner.COHORT)
    report['provenance'] = dict(source_commit=runner.lineup.SOURCE_COMMIT, snapshot_id=857,
        input_sha256=list(pins), recipe_sha256=list(bound_pins[9:11]),
        regenerated_census_sha256=_sha(source_output), regenerated_source=fresh['provenance'],
        effective_query_sha256=_sha(effective), native_joint_sha256=_sha(native),
        attribution=fresh['provenance']['attribution'],
        split_basis='unchanged whole-game SHA256 literal UTF-8 game ID first hex digit 0/1/2 heldout;descriptive only')
    runner.write_report(report, bound_inputs, bound_pins, output)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot', 'supplement', 'lineups', 'manifest', 'census', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    report = publish(args.snapshot, args.supplement, args.lineups, args.manifest, args.census, args.output)
    print(json.dumps(dict(format=report['format'], games=report['games'], basis=report['basis']), sort_keys=True))


if __name__ == '__main__':
    main()
