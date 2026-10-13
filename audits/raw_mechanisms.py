"""Inventory literal Retrosheet notation without inferring legal events or clocks."""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
import re
from audits.outcome_inventory import number, digest
from audits.outcome_details import SOURCE_COMMIT, CATALOG_SHA, SNAPSHOT_SHA

SUPPLEMENT_SHA = '554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82'
SOURCE_TOTALS = dict(events=222047, real_events=192025, no_play_events=30022,
                     pa=186517, non_pa=5508, outs=130270, runs=23467,
                     runner_rows=196161, real_runner_rows=182718,
                     no_play_runner_rows=13443, flag_rows=10548, credit_rows=177670)

FIELDS = set('event_key game_id event_id no_play_flag result outs runs trajectory angle raw_play flags credits runners'.split())
FLAG_FIELDS = {'sequence_id', 'flag'}
CREDIT_FIELDS = {'sequence_id', 'fielding_position', 'fielding_play'}
RUNNER_FIELDS = set('baserunner runner_id baserunning_play_type is_out run_scored_flag explicit_out_flag advanced_on_error_flag attempted_advance_to_base base_end is_advance_attempt'.split())


def outer_parts(text, delimiters):
    """Return literal segments and delimiters only at parenthesis depth zero."""
    parts, separators, depth, start = [], [], 0, 0
    for i, char in enumerate(text):
        if char == '(':
            depth += 1
        elif char == ')':
            depth -= 1
            if depth < 0:
                raise ValueError('unbalanced_parentheses')
        elif depth == 0 and char in delimiters:
            parts.append(text[start:i])
            separators.append(char)
            start = i + 1
    if depth:
        raise ValueError('unbalanced_parentheses')
    parts.append(text[start:])
    return parts, separators


def lex(raw):
    """Split outer notation losslessly, without interpreting retirement or timing."""
    value = dict(raw=raw, status='structural_unknown', unknown_reason=None,
                 base_atoms=[], base_separators=[], modifiers=[], advances=[])
    try:
        sections, dots = outer_parts(raw, '.')
        if len(dots) > 1:
            raise ValueError('multiple_advance_delimiters')
        pieces, _ = outer_parts(sections[0], '/')
        atoms, separators = outer_parts(pieces[0], '+;')
        advances = outer_parts(sections[1], ';')[0] if dots else []
        if any(not part for part in atoms + pieces[1:] + advances):
            raise ValueError('empty_outer_segment')
        value.update(status='split', base_atoms=atoms, base_separators=separators,
                     modifiers=pieces[1:], advances=advances)
    except ValueError as error:
        value['unknown_reason'] = str(error)
    return value


def root_family(atom):
    """Recognize documented symbol families, never their legal effect or timing."""
    for pattern, family in (
        (r'NP', 'no_play_notation'), (r'BK', 'balk_notation'),
        (r'WP', 'wild_pitch_notation'), (r'PB', 'passed_ball_notation'),
        (r'DI', 'indifference_notation'), (r'OA', 'other_advance_notation'),
        (r'HP', 'hit_by_pitch_notation'), (r'C', 'interference_notation'),
        (r'DGR', 'ground_rule_double_notation'),
        (r'IW|I', 'intentional_walk_notation'), (r'W', 'walk_notation'),
        (r'K[0-9U]*', 'strikeout_notation'),
        (r'H(?:R)?[0-9U]*', 'home_run_notation'),
        (r'S[0-9U]*', 'single_notation'), (r'D[0-9U]*', 'double_notation'),
        (r'T[0-9U]*', 'triple_notation'), (r'FC[0-9U]*', 'fielders_choice_notation'),
        (r'FLE[0-9U]', 'foul_error_notation'), (r'[0-9U]*E[0-9U]', 'error_notation'),
        (r'POCS[23H]\(.+\)', 'pickoff_caught_stealing_notation'),
        (r'PO[123]\(.+\)', 'pickoff_notation'),
        (r'CS[23H]\(.+\)', 'caught_stealing_notation'),
        (r'SB[23H](?:\(.+\))?', 'stolen_base_notation'),
        (r'[0-9U]+(?:\([B123]\)[0-9U]*)*', 'numeric_fielding_notation'),
    ):
        if re.fullmatch(pattern, atom):
            return family
    return 'unknown_symbol'


def check_scope(report, manifest):
    required = dict(format='outcome-detail-2019-v1', source_commit=SOURCE_COMMIT,
                    catalog_sha256=CATALOG_SHA, snapshot_sha256=SNAPSHOT_SHA,
                    database_sha256=SUPPLEMENT_SHA, snapshot_id=857)
    if any(manifest.get(k) != v for k, v in required.items()) or not manifest.get('attribution'):
        raise ValueError('Missing or mismatched supplement provenance')
    if report['games'] != 2429 or any(report['totals'].get(k) != v for k, v in SOURCE_TOTALS.items()):
        raise ValueError('Incomplete or mismatched 2019 event/child cohort')
    if report['supplied_credit_units'] != dict(putouts=130270, assists=43968, errors=2836):
        raise ValueError('Supplied fielding units do not match accepted supplement')


def read_export(stream):
    reader = csv.DictReader(stream, strict=True)
    if reader.fieldnames is None or len(reader.fieldnames) != len(set(reader.fieldnames)) or set(reader.fieldnames) != FIELDS:
        raise ValueError('Invalid raw-mechanism export header')
    return reader


def unique_object(pairs):
    value = {}
    for name, item in pairs:
        if name in value:
            raise ValueError('Duplicate child JSON field')
        value[name] = item
    return value


def child_rows(raw, fields):
    values = json.loads(raw, object_pairs_hook=unique_object)
    if not isinstance(values, list) or any(not isinstance(v, dict) or set(v) != fields for v in values):
        raise ValueError('Invalid child-row JSON shape')
    return values


def sequence_rows(values):
    ids = [v['sequence_id'] for v in values]
    if any(type(n) is not int or n < 1 for n in ids) or len(ids) != len(set(ids)):
        raise ValueError('Invalid child sequence grain')
    return sorted(values, key=lambda v: v['sequence_id'])


def inventory(rows):
    totals = Counter(dict(events=0, real_events=0, no_play_events=0, pa=0, non_pa=0,
                          outs=0, runs=0, compound_real_events=0, runner_outs=0,
                          runner_runs=0, runner_rows=0, real_runner_rows=0,
                          no_play_runner_rows=0, flag_rows=0, credit_rows=0))
    units = Counter(dict(putouts=0, assists=0, errors=0))
    roots, unknown, flags, labels, modifiers, trajectories, angles = (Counter() for _ in range(7))
    results, multiplicity, statuses, shapes, fielding_plays = (Counter() for _ in range(5))
    examples, keys, identities, games = {}, set(), set(), set()
    unknown_events, unknown_reasons = [], Counter()
    for row in rows:
        if set(row) != FIELDS or any(not isinstance(v, str) for v in row.values()):
            raise ValueError('Invalid raw-mechanism export row')
        key, event_id = number(row, 'event_key'), number(row, 'event_id')
        identity = (row['game_id'], event_id)
        if not row['game_id'] or key in keys or identity in identities or not row['raw_play'].strip():
            raise ValueError('Missing/duplicate event identity or raw play')
        keys.add(key); identities.add(identity); games.add(row['game_id'])
        if row['no_play_flag'] not in ('true', 'false'):
            raise ValueError('Missing no-play classification')
        np = row['no_play_flag'] == 'true'
        outs, runs = number(row, 'outs', 3), number(row, 'runs')
        if np and (outs or runs or row['result']):
            raise ValueError('No-play statistics must not be fabricated')
        flag_rows = sequence_rows(child_rows(row['flags'], FLAG_FIELDS))
        credit_rows = sequence_rows(child_rows(row['credits'], CREDIT_FIELDS))
        runners = child_rows(row['runners'], RUNNER_FIELDS)
        if any(not isinstance(f['flag'], str) or not f['flag'] for f in flag_rows):
            raise ValueError('Missing supplied flag label')
        roles = [r['baserunner'] for r in runners]
        if len(roles) != len(set(roles)) or any(r not in ('Batter', 'First', 'Second', 'Third') for r in roles):
            raise ValueError('Invalid runner grain/role')
        for runner in runners:
            if any(type(runner[f]) is not bool for f in ('is_out', 'run_scored_flag')):
                raise ValueError('Missing required runner units')
            if any(runner[f] is not None and not isinstance(runner[f], str)
                   for f in ('runner_id', 'baserunning_play_type', 'attempted_advance_to_base', 'base_end')):
                raise ValueError('Invalid supplied runner identity/label')
        event_units = Counter(dict(putouts=0, assists=0, errors=0))
        assists = set()
        for credit in credit_rows:
            position, kind = credit['fielding_position'], credit['fielding_play']
            if type(position) is not int or not 0 <= position <= 9 or kind not in ('Assist', 'Putout', 'Error', 'FieldersChoice'):
                raise ValueError('Invalid supplied fielding play')
            fielding_plays[kind] += 1
            if kind == 'Assist':
                assists.add(position)
            elif kind in ('Putout', 'Error'):
                event_units['assists'] += len(assists)
                assists.clear()
                event_units['putouts' if kind == 'Putout' else 'errors'] += 1
        event_units['assists'] += len(assists)
        runner_runs = sum(r['run_scored_flag'] for r in runners)
        runner_outs = sum(r['is_out'] for r in runners)
        if event_units['putouts'] != outs or runner_runs != runs or runner_outs > outs:
            raise ValueError('Supplied event/child unit mismatch')
        value = lex(row['raw_play'])
        if value['status'] == 'structural_unknown':
            unknown_events.append(dict(event=dict(row), lexical=value))
            unknown_reasons[value['unknown_reason']] += 1
        families = [root_family(atom) for atom in value['base_atoms']]
        roots.update(families)
        unknown.update(atom for atom, family in zip(value['base_atoms'], families) if family == 'unknown_symbol')
        flags.update(f['flag'] for f in flag_rows)
        labels.update(r['baserunning_play_type'] if r['baserunning_play_type'] is not None else '<missing>' for r in runners)
        modifiers.update(value['modifiers'])
        statuses[value['status']] += 1
        units.update(event_units)
        totals.update(events=1, real_events=int(not np), no_play_events=int(np),
                      pa=int(not np and bool(row['result'])), non_pa=int(not np and not row['result']),
                      outs=outs, runs=runs, compound_real_events=int(not np and bool(value['base_separators'])),
                      runner_outs=runner_outs, runner_runs=runner_runs, runner_rows=len(runners),
                      real_runner_rows=len(runners) if not np else 0,
                      no_play_runner_rows=len(runners) if np else 0,
                      flag_rows=len(flag_rows), credit_rows=len(credit_rows))
        if not np:
            results[row['result'] or '<non-PA>'] += 1
            multiplicity[str(outs)] += 1
            trajectories[row['trajectory'] or '<missing>'] += 1
            angles[row['angle'] or '<missing>'] += 1
        shape = (np, row['result'], outs, row['trajectory'], row['angle'],
                 tuple(families), tuple(value['base_separators']), value['status'])
        shapes[shape] += 1
        if shape not in examples or key < int(examples[shape]['event']['event_key']):
            examples[shape] = dict(event=dict(row), lexical=value)
    if not totals['events']:
        raise ValueError('Empty raw-mechanism export')
    def ordered(counter):
        return dict(sorted(counter.items()))
    return dict(format='2019-raw-mechanisms-v1', games=len(games), totals=dict(totals),
                supplied_credit_units=dict(units), root_families=ordered(roots),
                unknown_roots=ordered(unknown), source_flags=ordered(flags),
                source_fielding_plays=ordered(fielding_plays),
                runner_labels=ordered(labels), modifier_literals=ordered(modifiers),
                raw_trajectories=ordered(trajectories), raw_angles=ordered(angles),
                real_results=ordered(results), real_out_multiplicity=ordered(multiplicity),
                structural_status=ordered(statuses),
                structural_unknown_reasons=ordered(unknown_reasons),
                structural_unknown_events=sorted(unknown_events, key=lambda v: int(v['event']['event_key'])),
                shapes=[dict(no_play=k[0], result=k[1], outs=k[2], trajectory=k[3], angle=k[4],
                             root_families=list(k[5]), base_separators=list(k[6]), status=k[7], events=n)
                        for k, n in sorted(shapes.items())],
                examples=[examples[k] for k in sorted(examples)])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('events', type=Path)
    parser.add_argument('manifest', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError('Refusing to overwrite an existing file')
    before = digest(args.events), digest(args.manifest)
    manifest = json.loads(args.manifest.read_text())
    with args.events.open(newline='') as stream:
        report = inventory(read_export(stream))
    check_scope(report, manifest)
    if before != (digest(args.events), digest(args.manifest)):
        raise ValueError('Inventory inputs changed during read')
    report['provenance'] = dict(events_sha256=before[0], manifest_sha256=before[1],
        supplement_sha256=manifest['database_sha256'], catalog_sha256=CATALOG_SHA,
        snapshot_sha256=SNAPSHOT_SHA, source_commit=SOURCE_COMMIT, snapshot_id=857,
        query_sha256=digest(Path(__file__).with_name('2019-raw-mechanisms.sql')),
        attribution=manifest['attribution'])
    with args.output.open('x') as output:
        output.write(json.dumps(report, sort_keys=True, indent=2) + '\n')
    print(json.dumps(dict(totals=report['totals'], structural_status=report['structural_status'],
                         unknown_roots=report['unknown_roots']), sort_keys=True))


if __name__ == '__main__':
    main()
