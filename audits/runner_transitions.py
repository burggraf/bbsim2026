"""Descriptive before-runner opportunities; no causal scoring or fitted probabilities."""
from collections import Counter
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
from audits import lineup_readiness as lineup
from audits import outcome_inventory as outcome
from audits import joint_contact
from audits import raw_mechanisms as raw

FORMAT = '2019-runner-transitions-v1'
BASIS = 'literal before-event runner/terminal-PA batter opportunities; endpoints are not causal scoring or ability facts'
ROLES = ('Batter', 'First', 'Second', 'Third')
ENDPOINTS = ('First', 'Second', 'Third', 'Home', 'Out')
INPUT_SHA = ('d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528',
    '554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82',
    '5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b',
    '659a55d60cf5ff4a80965501ccca5ce3c5420d0e0b4c18dc7d4c42daa8c1eb44')
COHORT = dict(games=2429, supported_games=2425,
    excluded_game_ids=['BOS201908070', 'CHA201905270', 'DET201905190', 'NYN201906130'],
    raw_totals=raw.SOURCE_TOTALS, supplied_credit_units=dict(putouts=130270, assists=43968, errors=2836))
NULLABLE_FLAGS = ('explicit_out_flag', 'advanced_on_error_flag', 'is_advance_attempt')
DETAILS = ('base_end', 'baserunning_play_type', 'attempted_advance_to_base',
           'is_out', 'run_scored_flag') + NULLABLE_FLAGS


def _notation_report(rows):
    report = raw.inventory(rows)
    # Normalize report-only child JSON serialization, retaining roles, sequence
    # identities and every nullable value. Original CSV/raw play stays untouched.
    for record in report['examples'] + report['structural_unknown_events']:
        for name, fields, key in [('runners', raw.RUNNER_FIELDS, 'baserunner'),
                ('flags', raw.FLAG_FIELDS, 'sequence_id'), ('credits', raw.CREDIT_FIELDS, 'sequence_id')]:
            children = raw.child_rows(record['event'][name], fields)
            record['event'][name] = json.dumps(sorted(children, key=lambda v: v[key]), sort_keys=True)
    return report


def read_native_groups(text):
    """Read native literal cells strictly; missing/duplicate facts never become zero."""
    reader = csv.DictReader(io.StringIO(text), strict=True)
    if reader.fieldnames != ['facts', 'rows']:
        raise ValueError('Invalid native runner group header')
    fields = set(DETAILS) | set('no_play split result trajectory base_mask outs_before event_outs event_runs baserunner identity_present'.split())
    groups = Counter()
    for row in reader:
        if set(row) != {'facts', 'rows'} or any(not isinstance(v, str) for v in row.values()):
            raise ValueError('Invalid native runner group row')
        if not row['rows'].isascii() or not row['rows'].isdigit() or int(row['rows']) < 1:
            raise ValueError('Invalid native runner group count')
        facts = json.loads(row['facts'], object_pairs_hook=raw.unique_object)
        if not isinstance(facts, dict) or set(facts) != fields:
            raise ValueError('Invalid native runner group facts')
        if (any(type(facts[k]) is not bool for k in ['no_play', 'identity_present', 'is_out', 'run_scored_flag'])
                or any(facts[k] is not None and type(facts[k]) is not bool for k in NULLABLE_FLAGS)
                or any(facts[k] is not None and not isinstance(facts[k], str)
                       for k in ['base_end', 'baserunning_play_type', 'attempted_advance_to_base'])
                or any(not isinstance(facts[k], str) for k in ['result', 'trajectory'])
                or facts['baserunner'] not in ROLES or facts['split'] not in ['training', 'heldout', 'excluded']
                or any(type(facts[k]) is not int or facts[k] < 0 for k in ['event_outs', 'event_runs'])
                or facts['event_outs'] > 3):
            raise ValueError('Invalid native runner group types/labels')
        if facts['no_play']:
            valid = (facts['base_mask'] is None and facts['outs_before'] is None
                     and not facts['result'] and facts['event_outs'] == facts['event_runs'] == 0)
        else:
            valid = (type(facts['base_mask']) is int and 0 <= facts['base_mask'] <= 7
                     and type(facts['outs_before']) is int and 0 <= facts['outs_before'] <= 2)
        if not valid:
            raise ValueError('Invalid native runner group context')
        key = json.dumps(facts, sort_keys=True)
        if key in groups:
            raise ValueError('Duplicate native runner group')
        groups[key] = int(row['rows'])
    return groups


def _raw_groups(rows, contexts, split_of, children):
    groups = Counter()
    for row in rows:
        key = int(row['event_key'])
        context = contexts.get(key)
        for value in children[key].values():
            facts = dict(no_play=row['no_play_flag'] == 'true', split=split_of[row['game_id']],
                result=row['result'], trajectory=row['trajectory'],
                base_mask=None if context is None else int(context['base_mask']),
                outs_before=None if context is None else int(context['outs_before']),
                event_outs=int(row['outs']), event_runs=int(row['runs']),
                baserunner=value['baserunner'], identity_present=value['runner_id'] is not None,
                **{name: value[name] for name in DETAILS})
            groups[json.dumps(facts, sort_keys=True)] += 1
    return groups


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups=None):
    """Complete source validation precedes any new opportunity measurement."""
    raw_rows, outcome_rows = tuple(raw_rows), tuple(outcome_rows)
    source = joint_contact.inventory(raw_rows, outcome_rows)
    contexts = {int(row['event_key']): row for row in outcome_rows}
    games = {row['game_id'] for row in raw_rows}
    if (not isinstance(supported_game_ids, (list, tuple))
            or any(not isinstance(game, str) or not game for game in supported_game_ids)
            or len(set(supported_game_ids)) != len(supported_game_ids)
            or not set(supported_game_ids) <= games):
        raise ValueError('Invalid supported runner-census game identities')
    supported = set(supported_game_ids)
    split_of = {game: ('excluded' if game not in supported else
        'heldout' if hashlib.sha256(game.encode('utf-8')).hexdigest()[0] in '012' else 'training')
        for game in games}
    children = {}
    for row in raw_rows:
        values = raw.child_rows(row['runners'], raw.RUNNER_FIELDS)
        for value in values:
            if any(value[name] is not None and type(value[name]) is not bool for name in NULLABLE_FLAGS):
                raise ValueError('Invalid nullable runner flag type')
            context = contexts.get(int(row['event_key']))
            if context is not None:
                expected = context['batter_id' if value['baserunner'] == 'Batter' else value['baserunner'].lower()]
                if expected and value['runner_id'] is not None and value['runner_id'] != expected:
                    raise ValueError('Conflicting before runner/child identity')
        children[int(row['event_key'])] = {value['baserunner']: value for value in values}

    raw_cells = _raw_groups(raw_rows, contexts, split_of, children)
    if native_groups is not None and native_groups != raw_cells:
        raise ValueError('Native runner group reconciliation failed')
    return _measure(raw_rows, contexts, split_of, children, source, supported, games, raw_cells)


def _measure(raw_rows, contexts, split_of, children, source, supported, games, raw_cells):
    """Called only after complete validation and any supplied native reconciliation."""
    totals = dict(real_events=0, terminal_pa=0, non_pa=0,
        no_play_events=source['raw_totals']['no_play_events'],
        before_runner_opportunities=0, batter_opportunities=0, opportunities=0,
        matching_child_rows=0, extra_child_rows=0,
        no_play_child_rows=source['raw_totals']['no_play_runner_rows'])
    cells, vectors = Counter(), Counter()
    findings = Counter(dict(home_without_counted_run=0, unknown_attempt_label=0, out_without_retirement=0))
    splits = {split: dict(game_ids=sorted(game for game in games if split_of[game] == split),
                         real_events=0, opportunities=0)
              for split in ('training', 'heldout', 'excluded')}
    for row in raw_rows:
        if row['no_play_flag'] == 'true':
            continue
        key = int(row['event_key'])
        context = contexts[key]
        split = split_of[row['game_id']]
        pa = context['pa'] == '1'
        lexical = raw.lex(row['raw_play'])
        common = dict(split=split, result=context['result'], trajectory=context['trajectory'],
            base_mask=int(context['base_mask']), outs_before=int(context['outs_before']),
            event_outs=int(row['outs']), event_runs=int(row['runs']),
            frame_end=context['frame_end'] == 'true', notation_status=lexical['status'],
            compound=bool(lexical['base_separators']), pa=pa, modifiers=lexical['modifiers'],
            source_flags=[f['flag'] for f in raw.sequence_rows(raw.child_rows(row['flags'], raw.FLAG_FIELDS))])
        roles = ([('Batter', context['batter_id'])] if pa else []) + [
            (role, context[role.lower()]) for role in ROLES[1:] if context[role.lower()]]
        observed = children[key]
        endpoints = []
        for role, identity in roles:
            value = observed.get(role)
            details = {name: None if value is None else value[name] for name in DETAILS}
            if value is None:
                status = 'missing_row'
            elif value['runner_id'] is None:
                status = 'missing_identity'
            elif value['base_end'] is None:
                status = 'missing_endpoint'
            elif value['base_end'] not in ENDPOINTS:
                status = 'unknown_endpoint'
            else:
                status = 'observed'
            endpoint = dict(role=role, endpoint_status=status, **details)
            endpoints.append(endpoint)
            cells[json.dumps(dict(common, **endpoint), sort_keys=True)] += 1
            if status != 'observed':
                findings[status] += 1
            if value is not None:
                if value['base_end'] == 'Home' and not value['run_scored_flag']:
                    findings['home_without_counted_run'] += 1
                if value['base_end'] == 'Out' and not value['is_out']:
                    findings['out_without_retirement'] += 1
                attempt = value['attempted_advance_to_base']
                if attempt is not None and attempt not in ENDPOINTS[:-1]:
                    findings['unknown_attempt_label'] += 1
            totals['opportunities'] += 1
            totals['batter_opportunities' if role == 'Batter' else 'before_runner_opportunities'] += 1
            totals['matching_child_rows'] += int(value is not None)
            splits[split]['opportunities'] += 1
        totals['extra_child_rows'] += len(set(observed) - {role for role, _ in roles})
        totals['real_events'] += 1
        totals['terminal_pa' if pa else 'non_pa'] += 1
        splits[split]['real_events'] += 1
        vectors[json.dumps(dict(common, runners=endpoints), sort_keys=True)] += 1
    if (totals['real_events'] != source['outcome_totals']['events']
            or totals['terminal_pa'] != source['outcome_totals']['pa']
            or totals['non_pa'] != source['outcome_totals']['non_pa']
            or sum(cells.values()) != totals['opportunities']
            or sum(vectors.values()) != totals['real_events']
            or sum(raw_cells.values()) != source['raw_totals']['runner_rows']
            or totals['matching_child_rows'] + totals['extra_child_rows'] != source['raw_totals']['real_runner_rows']):
        raise ValueError('Runner opportunity partition conservation failed')
    return dict(format=FORMAT, basis=BASIS, games=len(games), supported_games=len(supported),
        excluded_game_ids=sorted(games - supported), splits=splits, totals=totals,
        cells=[dict(json.loads(key), opportunities=n) for key, n in sorted(cells.items())],
        joint_vectors=[dict(json.loads(key), events=n) for key, n in sorted(vectors.items())],
        findings=dict(sorted(findings.items())), raw_notation=_notation_report(raw_rows),
        raw_child_cells=[dict(json.loads(key), rows=n) for key, n in sorted(raw_cells.items())],
        raw_totals=source['raw_totals'],
        outcome_totals=source['outcome_totals'], supplied_credit_units=source['supplied_credit_units'])


def write_report(report, inputs, pins, output):
    current = tuple(hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in inputs)
    if current != tuple(pins):
        raise ValueError('Runner audit inputs changed before publication')
    text = json.dumps(report, sort_keys=True, indent=2, allow_nan=False) + '\n'
    with Path(output).open('x', encoding='utf-8') as stream:
        stream.write(text)


def publish(snapshot, supplement, lineups, manifest, output):
    output = Path(output)
    if output.is_symlink():
        raise ValueError('Refusing to overwrite runner report/native evidence')
    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    artifacts = [output] + [output.with_suffix('.' + label + suffix)
        for label in ['lineup', 'events', 'raw', 'groups']
        for suffix in ['.csv', '.stderr', '.query.sql']]
    if any(p.exists() or p.is_symlink() for p in artifacts):
        raise ValueError('Refusing to overwrite runner report/native evidence')
    inputs = list(map(Path, [snapshot, supplement, lineups, manifest]))
    pins = tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs)
    if pins != INPUT_SHA:
        raise ValueError('Unsupported accepted runner audit input digest')
    accepted_lineups, detail_manifest = [json.loads(p.read_text(), object_pairs_hook=raw.unique_object) for p in inputs[2:]]
    required = dict(format='outcome-detail-2019-v1', source_commit=lineup.SOURCE_COMMIT,
        catalog_sha256=lineup.CATALOG, snapshot_sha256=pins[0], database_sha256=pins[1], snapshot_id=857)
    if (not isinstance(detail_manifest, dict)
            or any(detail_manifest.get(k) != v for k, v in required.items())
            or not lineup.nonempty(detail_manifest.get('attribution'))
            or not isinstance(accepted_lineups, dict)):
        raise ValueError('Unsupported runner audit provenance')
    provenance = accepted_lineups.get('provenance', {})
    if (not isinstance(provenance, dict) or provenance.get('snapshot_sha256') != pins[0]
            or provenance.get('supplement_sha256') != pins[1]
            or provenance.get('attribution') != detail_manifest['attribution']):
        raise ValueError('Unsupported runner lineup provenance')
    labels = ['lineup', 'events', 'raw', 'groups']
    queries = [Path(__file__).with_name(name) for name in
        ['2019-lineup-readiness.sql', '2019-outcomes.sql', '2019-raw-mechanisms.sql', '2019-runner-transitions.sql']]
    recipes = [Path(__file__), Path(lineup.__file__), Path(outcome.__file__), Path(raw.__file__), Path(joint_contact.__file__)]
    inputs += queries + recipes
    all_pins = tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs)
    if all_pins[:4] != pins:
        raise ValueError('Runner audit inputs changed during read')
    exports = [output.with_suffix('.' + name + '.csv') for name in labels]
    errors = [output.with_suffix('.' + name + '.stderr') for name in labels]
    effective_queries = [output.with_suffix('.' + name + '.query.sql') for name in labels]
    detail = str(inputs[1].resolve()).replace("'", "''")
    for i, (query, export, error, effective) in enumerate(zip(queries, exports, errors, effective_queries)):
        text = query.read_text()
        if labels[i] == 'raw':
            text += '\nSELECT * FROM mechanism_export ORDER BY event_key;\n'
            database = inputs[1]
            setup = 'SET enable_external_access=false;'
        else:
            database = inputs[0]
            setup = f"ATTACH '{detail}' AS detail (READ_ONLY); SET enable_external_access=false;"
        with effective.open('x', encoding='utf-8') as stream:
            stream.write(text)
        with export.open('x', encoding='utf-8') as stdout, error.open('x', encoding='utf-8') as stderr:
            result = subprocess.run(['duckdb', '-init', '/dev/null', '-readonly', '-bail', '-batch', '-csv',
                '-nullvalue', '', '-cmd', setup, str(database.resolve())], input=text,
                stdout=stdout, stderr=stderr, text=True, check=False, timeout=300)
        if result.returncode:
            raise RuntimeError(f'Native runner query failed ({result.returncode}); see {error}')
    exported = lineup.read_export(exports[0].read_text())
    manifests = exported['manifest']
    if (len(manifests) != 1 or manifests[0].get('catalog_sha256') != lineup.CATALOG
            or manifests[0].get('schema_version') != 'f03-spike-1'
            or manifests[0].get('query_version') != '2019-filtered-extract-v1'
            or any(not lineup.nonempty(manifests[0].get(k)) for k in ['source_url', 'extracted_at', 'attribution'])
            or manifests[0]['attribution'] != detail_manifest['attribution']):
        raise ValueError('Unsupported runner snapshot provenance')
    fresh = lineup.inventory(exported['game'], exported['person'], exported['appearance'])
    if fresh != {k:v for k, v in accepted_lineups.items() if k != 'provenance'}:
        raise ValueError('Runner lineup reconciliation failed')
    with exports[1].open(newline='', encoding='utf-8') as stream:
        reader = csv.DictReader(stream, strict=True)
        if reader.fieldnames is None or len(reader.fieldnames) != len(outcome.FIELDS) or set(reader.fieldnames) != outcome.FIELDS:
            raise ValueError('Invalid runner outcome export header')
        outcomes = list(reader)
    with exports[2].open(newline='', encoding='utf-8') as stream:
        raws = list(raw.read_export(stream))
    # All complete validators remain unchanged; scope/units/native groups must
    # agree before inventory is permitted to measure any before opportunities.
    source = joint_contact.inventory(raws, outcomes)
    if (source['games'] != COHORT['games']
            or len(fresh['candidate_game_ids']) != COHORT['supported_games']
            or sorted({r['game_id'] for r in raws} - set(fresh['candidate_game_ids'])) != COHORT['excluded_game_ids']
            or any(source['raw_totals'].get(k) != v for k, v in COHORT['raw_totals'].items())
            or source['supplied_credit_units'] != COHORT['supplied_credit_units']):
        raise ValueError('Unsupported runner source cohort/units')
    native = read_native_groups(exports[3].read_text())
    report = inventory(raws, outcomes, fresh['candidate_game_ids'], native_groups=native)
    digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    report['provenance'] = dict(snapshot=manifests[0], source_commit=lineup.SOURCE_COMMIT,
        snapshot_id=857, snapshot_sha256=pins[0], supplement_sha256=pins[1],
        lineup_report_sha256=pins[2], detail_manifest_sha256=pins[3],
        query_sha256=list(all_pins[4:8]), recipe_sha256=list(all_pins[8:]),
        effective_query_sha256=[digest(p) for p in effective_queries],
        native_export_sha256=[digest(p) for p in exports], attribution=detail_manifest['attribution'],
        split_basis='SHA256 literal UTF-8 game ID first hex digit 0/1/2 reserved; hindsight ratings are not heldout evaluation')
    write_report(report, inputs, all_pins, output)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['snapshot', 'supplement', 'lineups', 'manifest', 'output']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    report = publish(args.snapshot, args.supplement, args.lineups, args.manifest, args.output)
    print(json.dumps(dict(format=report['format'], games=report['games'], basis=report['basis']), sort_keys=True))


if __name__ == '__main__':
    main()
