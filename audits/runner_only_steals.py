"""Runner-only endpoint proxies, never a pitch clock or sole-cause certificate."""
from collections import Counter
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile
from audits import raw_mechanisms as raw
from audits import runner_transitions as runner
from audits import outcome_inventory as outcome
from audits import lineup_readiness as lineup

FORMAT = '2019-runner-only-steals-v1'
INPUT_SHA = runner.INPUT_SHA + ('bea6d15b199aa2835bfab4334bf61b1b3de9fc3f382285e6b542f564a0ef098a',)
EXPORT_SHA = (
    'bef8108455784113702dd4003172ac99162d24dfb8522da4e2a0c53fa16dbf02',
    'ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a',
    '40c08edc432ed8480e07a660b6896236316c9cb3235ffae6acf0ce9c9d21c199',
    'fe4d84e1c51ea945c3538b4bcda43bc8b94ba8eeb08376cf7aa7d66d4d431041',
)
COHORT = runner.COHORT
REASONS = ('not_simple_steal', 'notation_uncertain', 'incomplete_vector',
           'runner_effect_or_unknown', 'unsupported_geometry')
ROLES = runner.ROLES[1:]
KINDS = ('StolenBase', 'CaughtStealing')
ATTEMPT_LABELS = KINDS + ('PickedOff', 'PickedOffCaughtStealing')


def classify(row, context):
    """Endpoint/notation subset only; full source validation belongs to inventory."""
    failed = set()
    children = sorted(raw.child_rows(row['runners'], raw.RUNNER_FIELDS),
                      key=lambda value: runner.ROLES.index(value['baserunner']))
    observed = {value['baserunner']: value for value in children}
    roles = {role: context[role.lower()] for role in ROLES if context[role.lower()]}
    targets = [value for value in children
               if value['baserunner'] in roles and value['baserunning_play_type'] in KINDS]
    target = targets[0] if len(targets) == 1 else None
    kind = target['baserunning_play_type'] if target else None
    role = target['baserunner'] if target else None
    index = ROLES.index(role) if role else None
    next_base = ('Second', 'Third', 'Home')[index] if index is not None else None
    caught = kind == 'CaughtStealing'
    credits = raw.child_rows(row['credits'], raw.CREDIT_FIELDS)
    credit_shape = (sum(c['fielding_play'] == 'Putout' for c in credits) == 1
                    and all(c['fielding_play'] in ('Putout', 'Assist')
                            and type(c['fielding_position']) is int
                            and 1 <= c['fielding_position'] <= 9 for c in credits)
                    if caught else not credits)
    if (target is None or context['pa'] == '1' or row['result']
            or int(row['runs']) != 0 or int(row['outs']) != int(caught)
            or 'Batter' in observed or not credit_shape
            or any(value is not target and (value['is_advance_attempt'] is True
                       or value['baserunning_play_type'] in ATTEMPT_LABELS) for value in children)):
        failed.add('not_simple_steal')

    lexical = raw.lex(row['raw_play'])
    atom = ('CS' if caught else 'SB') + ({'Second': '2', 'Third': '3', 'Home': 'H'}.get(next_base, '?'))
    pattern = re.escape(atom) + (r'\([1-9]+\)' if caught else '')
    if (target is None or lexical['status'] != 'split' or lexical['base_separators']
            or lexical['modifiers'] or lexical['advances'] or len(lexical['base_atoms']) != 1
            or re.fullmatch(pattern, lexical['base_atoms'][0]) is None
            or raw.child_rows(row['flags'], raw.FLAG_FIELDS)):
        failed.add('notation_uncertain')

    flags = ('is_out', 'run_scored_flag', 'explicit_out_flag',
             'advanced_on_error_flag', 'is_advance_attempt')
    complete = (set(observed) == set(roles)
                and all(observed[r]['runner_id'] == identity and identity
                        and observed[r]['base_end'] in runner.ENDPOINTS
                        and all(type(observed[r][f]) is bool for f in flags)
                        for r, identity in roles.items() if r in observed)
                and (target is None or target['attempted_advance_to_base'] in ('Second', 'Third', 'Home')))
    if not complete:
        failed.add('incomplete_vector')
    for value in children:
        if value is target:
            valid = (value['is_advance_attempt'] is True
                     and value['attempted_advance_to_base'] == next_base
                     and value['base_end'] == ('Out' if caught else next_base)
                     and value['is_out'] is caught and value['run_scored_flag'] is False)
        else:
            valid = (value['baserunner'] in roles and value['base_end'] == value['baserunner']
                     and value['is_out'] is False and value['run_scored_flag'] is False
                     and value['is_advance_attempt'] is False)
        if not valid or value['advanced_on_error_flag'] is not False:
            failed.add('runner_effect_or_unknown')

    mask = int(context['base_mask'])
    before_outs = int(context['outs_before'])
    boundary = context['frame_end'] == 'true'
    after_outs = outcome.number(context, 'after_outs', 3, optional=boundary)
    after_mask = outcome.number(context, 'after_base_mask', 7, optional=boundary)
    if index is None or next_base not in ('Second', 'Third'):
        failed.add('unsupported_geometry')
    else:
        expected_mask = mask & ~(1 << index)
        if not caught:
            expected_mask |= 1 << (index + 1)
        expected_outs = before_outs + int(caught)
        geometry = (bool(mask & (1 << index)) and not mask & (1 << (index + 1))
                    and int(row['runs']) == 0 and int(row['outs']) == int(caught)
                    and boundary == (expected_outs == 3)
                    and all(value['base_end'] == value['baserunner']
                            for value in children if value is not target))
        if boundary:
            geometry = geometry and after_outs in (None, 0, 3) and after_mask in (None, 0)
        else:
            geometry = geometry and after_outs == expected_outs and after_mask == expected_mask
        if not geometry:
            failed.add('unsupported_geometry')
    return dict(qualified=not failed, reasons=[reason for reason in REASONS if reason in failed],
                kind=kind, target_role=role, target_base=next_base, runners=children)


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups=None,
              expected_census=None, cohort=None):
    """Complete source/native/census validation always precedes new measurement."""
    rows, outcomes = tuple(raw_rows), tuple(outcome_rows)
    source = runner.inventory(rows, outcomes, supported_game_ids, native_groups=native_groups)
    if expected_census is not None and (not isinstance(expected_census, dict)
            or source != {k: v for k, v in expected_census.items() if k != 'provenance'}):
        raise ValueError('Runner-only census reconciliation failed')
    if cohort is not None:
        for key, expected in cohort.items():
            actual = source[key]
            if key == 'raw_totals':
                actual = {name: actual.get(name) for name in expected}
            if actual != expected:
                raise ValueError('Unsupported runner-only source cohort/units')
    contexts = {int(value['event_key']): value for value in outcomes}
    supported = set(supported_game_ids)
    totals = dict(candidate_events=0, qualified_events=0, unresolved_events=0,
                  before_runner_opportunities=0)
    split_totals = {name: dict(totals) for name in ('training', 'heldout', 'excluded')}
    exposures, vectors, qualified, labels, pa_labels = (Counter() for _ in range(5))
    first_reasons, all_reasons, outside = (Counter() for _ in range(3))
    encode = lambda value: json.dumps(value, sort_keys=True)
    for row in rows:
        if row['no_play_flag'] == 'true':
            continue
        context = contexts[int(row['event_key'])]
        split = ('excluded' if row['game_id'] not in supported else 'heldout'
                 if hashlib.sha256(row['game_id'].encode('utf-8')).hexdigest()[0] in '012' else 'training')
        pa = context['pa'] == '1'
        mask, outs = int(context['base_mask']), int(context['outs_before'])
        for index, role in enumerate(ROLES):
            if not context[role.lower()]:
                continue
            exposures[encode(dict(split=split, role=role, base_mask=mask, outs_before=outs,
                pa=pa, clear_next_base=index < 2 and not bool(mask & (1 << (index + 1)))))] += 1
            totals['before_runner_opportunities'] += 1
            split_totals[split]['before_runner_opportunities'] += 1
        if pa:
            outside[row['result']] += 1
            for value in raw.child_rows(row['runners'], raw.RUNNER_FIELDS):
                if value['baserunning_play_type'] in ATTEMPT_LABELS:
                    pa_labels[encode(dict(split=split, result=row['result'],
                                          label=value['baserunning_play_type']))] += 1
            continue
        value = classify(row, context)
        category = 'qualified_events' if value['qualified'] else 'unresolved_events'
        for counter in (totals, split_totals[split]):
            counter['candidate_events'] += 1
            counter[category] += 1
        if value['reasons']:
            first_reasons[value['reasons'][0]] += 1
            all_reasons.update(value['reasons'])
        common = dict(split=split, base_mask=mask, outs_before=outs)
        vectors[encode(dict(common, **value))] += 1
        for child in value['runners']:
            labels[encode(dict(split=split, label=child['baserunning_play_type']))] += 1
        if value['qualified']:
            qualified[encode(dict(common, kind=value['kind'], target_role=value['target_role'],
                                  target_base=value['target_base']))] += 1
    if (totals['candidate_events'] != source['totals']['non_pa']
            or totals['qualified_events'] + totals['unresolved_events'] != totals['candidate_events']
            or totals['before_runner_opportunities'] != source['totals']['before_runner_opportunities']
            or sum(outside.values()) != source['totals']['terminal_pa']
            or sum(first_reasons.values()) != totals['unresolved_events']):
        raise ValueError('Runner-only partition conservation failed')
    def cells(counter, unit):
        return [dict(json.loads(key), **{unit: count}) for key, count in sorted(counter.items())]
    return dict(format=FORMAT, basis='event-frame endpoint proxies, not pitches or attempt rates',
        totals=totals, split_totals=split_totals, source_totals=source['totals'],
        raw_totals=source['raw_totals'], outcome_totals=source['outcome_totals'],
        supplied_credit_units=source['supplied_credit_units'], outside_results=dict(sorted(outside.items())), no_play_events=source['totals']['no_play_events'],
        exposures=cells(exposures, 'opportunities'), joint_vectors=cells(vectors, 'events'),
        qualified_cells=cells(qualified, 'events'), candidate_child_labels=cells(labels, 'rows'),
        pa_attempt_labels=cells(pa_labels, 'rows'), first_reasons=dict(sorted(first_reasons.items())),
        all_reasons=dict(sorted(all_reasons.items())))


def _pins(paths):
    return tuple(hashlib.sha256(Path(path).read_bytes()).hexdigest() for path in paths)


def _identity(path):
    resolved = Path(path).resolve(strict=True)
    stat = resolved.stat()
    return resolved, stat.st_dev, stat.st_ino


def write_report(report, inputs, pins, output, *, recheck=None):
    """Complete finite JSON, flushed before exclusive publication; no partial final."""
    output = Path(output)
    if output.exists() or output.is_symlink():
        raise FileExistsError(output)
    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    identities = tuple(_identity(path) for path in inputs)
    parent = _identity(output.parent)
    def unchanged():
        if output.resolve() != output or _identity(output.parent) != parent:
            raise ValueError('Runner-only output path changed before publication')
        if tuple(_identity(path) for path in inputs) != identities:
            raise ValueError('Runner-only input path changed before publication')
        if _pins(inputs) != tuple(pins):
            raise ValueError('Runner-only inputs changed before publication')
        if recheck is not None:
            recheck()
    unchanged()
    payload = (json.dumps(report, sort_keys=True, indent=2, allow_nan=False) + '\n').encode('utf-8')
    stage = None
    try:
        with tempfile.NamedTemporaryFile(dir=output.parent, prefix='.runner-steals-', delete=False) as stream:
            stage = Path(stream.name)
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        unchanged()
        os.link(stage, output)
    finally:
        if stage is not None:
            stage.unlink()


def publish(snapshot, supplement, lineups, manifest, census, output):
    """Authenticate accepted cache and unchanged complete validators before measuring."""
    output = Path(output)
    if output.exists() or output.is_symlink():
        raise ValueError('Refusing to overwrite runner-only report')
    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    parent = _identity(output.parent)
    inputs = list(map(Path, (snapshot, supplement, lineups, manifest, census)))
    pins = _pins(inputs)
    if pins != INPUT_SHA:
        raise ValueError('Unsupported runner-only input digest')
    exports = [inputs[4].with_suffix('.' + name + '.csv') for name in ('lineup', 'events', 'raw', 'groups')]
    queries = [Path(runner.__file__).with_name(name) for name in (
        '2019-lineup-readiness.sql', '2019-outcomes.sql', '2019-raw-mechanisms.sql', '2019-runner-transitions.sql')]
    recipes = [Path(__file__)] + [Path(module.__file__) for module in
        (runner, lineup, outcome, raw, runner.joint_contact)]
    frozen = inputs + exports + queries + recipes
    if any(path.is_symlink() for path in frozen):
        raise ValueError('Runner-only input symlinks are unsupported')
    identities = tuple(_identity(path) for path in frozen)
    all_pins = _pins(frozen)
    if all_pins[:5] != pins:
        raise ValueError('Runner-only inputs changed during read')
    if all_pins[5:9] != EXPORT_SHA:
        raise ValueError('Unsupported runner-only native export digest')
    def unchanged():
        if output.resolve() != output or _identity(output.parent) != parent:
            raise ValueError('Runner-only output path changed before publication')
        if tuple(_identity(path) for path in frozen) != identities:
            raise ValueError('Runner-only input path changed before publication')
        if _pins(frozen) != all_pins:
            raise ValueError('Runner-only inputs changed before publication')
    accepted_lineups, detail, accepted_census = [json.loads(path.read_text(encoding='utf-8'),
        object_pairs_hook=raw.unique_object) for path in inputs[2:5]]
    if not all(isinstance(value, dict) for value in (accepted_lineups, detail, accepted_census)):
        raise ValueError('Unsupported runner-only provenance')
    required = dict(format='outcome-detail-2019-v1', source_commit=lineup.SOURCE_COMMIT,
        catalog_sha256=lineup.CATALOG, snapshot_sha256=pins[0], database_sha256=pins[1], snapshot_id=857)
    if (any(detail.get(key) != value for key, value in required.items())
            or not lineup.nonempty(detail.get('attribution'))):
        raise ValueError('Unsupported runner-only detail provenance')
    lp, cp = accepted_lineups.get('provenance'), accepted_census.get('provenance')
    if (not isinstance(lp, dict) or lp.get('snapshot_sha256') != pins[0]
            or lp.get('supplement_sha256') != pins[1] or lp.get('attribution') != detail['attribution']):
        raise ValueError('Unsupported runner-only lineup provenance')
    expected = dict(source_commit=lineup.SOURCE_COMMIT, snapshot_id=857, snapshot_sha256=pins[0],
        supplement_sha256=pins[1], lineup_report_sha256=pins[2], detail_manifest_sha256=pins[3],
        native_export_sha256=list(all_pins[5:9]), query_sha256=list(all_pins[9:13]),
        recipe_sha256=list(all_pins[14:]), attribution=detail['attribution'])
    if not isinstance(cp, dict) or any(cp.get(key) != value for key, value in expected.items()):
        raise ValueError('Unsupported runner-only census provenance')
    records = lineup.read_export(exports[0].read_text(encoding='utf-8'))
    if len(records['manifest']) != 1:
        raise ValueError('Unsupported runner-only snapshot provenance')
    snapshot_provenance = records['manifest'][0]
    if (snapshot_provenance != cp.get('snapshot')
            or snapshot_provenance.get('schema_version') != 'f03-spike-1'
            or snapshot_provenance.get('query_version') != '2019-filtered-extract-v1'
            or snapshot_provenance.get('catalog_sha256') != lineup.CATALOG
            or snapshot_provenance.get('attribution') != detail['attribution']
            or any(not lineup.nonempty(snapshot_provenance.get(key)) for key in ('source_url', 'extracted_at'))):
        raise ValueError('Unsupported runner-only snapshot provenance')
    fresh = lineup.inventory(records['game'], records['person'], records['appearance'])
    if fresh != {key: value for key, value in accepted_lineups.items() if key != 'provenance'}:
        raise ValueError('Runner-only lineup reconciliation failed')
    with exports[1].open(newline='', encoding='utf-8') as stream:
        reader = csv.DictReader(stream, strict=True)
        if (reader.fieldnames is None or len(reader.fieldnames) != len(outcome.FIELDS)
                or set(reader.fieldnames) != outcome.FIELDS):
            raise ValueError('Invalid runner-only outcome export header')
        outcomes = list(reader)
        if any(set(row) != outcome.FIELDS or None in row.values() for row in outcomes):
            raise ValueError('Invalid runner-only outcome export row')
    with exports[2].open(newline='', encoding='utf-8') as stream:
        rows = list(raw.read_export(stream))
    native = runner.read_native_groups(exports[3].read_text(encoding='utf-8'))
    unchanged()
    report = inventory(rows, outcomes, fresh['candidate_game_ids'], native_groups=native,
                       expected_census=accepted_census, cohort=COHORT)
    report['provenance'] = dict(expected, snapshot=snapshot_provenance, census_sha256=pins[4],
                                qualification_recipe_sha256=all_pins[13])
    unchanged()
    write_report(report, frozen, all_pins, output, recheck=unchanged)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot', 'supplement', 'lineups', 'manifest', 'census', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    report = publish(args.snapshot, args.supplement, args.lineups, args.manifest, args.census, args.output)
    print(json.dumps(dict(format=report['format'], totals=report['totals']), sort_keys=True))


if __name__ == '__main__':
    main()
