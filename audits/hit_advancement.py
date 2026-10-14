"""Descriptive ordinary-hit endpoint proxies, never causal hit certificates."""
from collections import Counter
import argparse
import csv
import hashlib
import json
from pathlib import Path
import re
from audits import runner_transitions as runner
from audits import raw_mechanisms as raw
from audits import lineup_readiness as lineup
from audits import outcome_inventory as outcome

FORMAT = '2019-hit-advancement-v1'
INPUT_SHA = runner.INPUT_SHA + ('bea6d15b199aa2835bfab4334bf61b1b3de9fc3f382285e6b542f564a0ef098a',)
EXPORT_SHA = (
    'bef8108455784113702dd4003172ac99162d24dfb8522da4e2a0c53fa16dbf02',
    'ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a',
    '40c08edc432ed8480e07a660b6896236316c9cb3235ffae6acf0ce9c9d21c199',
    'fe4d84e1c51ea945c3538b4bcda43bc8b94ba8eeb08376cf7aa7d66d4d431041',
)
COHORT = runner.COHORT
REASONS = ('terminal_or_out', 'notation_uncertain', 'fielding_effect',
    'incomplete_vector', 'runner_effect_or_unknown', 'hit_destination', 'unsupported_geometry')
TARGETS = ('First', 'Second', 'Third', 'Home')


def classify(row, context):
    """Qualify endpoint shape only; callers first run complete source validation."""
    failed = set()
    if context['pa'] != '1' or row['outs'] != '0' or context['frame_end'] != 'false':
        failed.add('terminal_or_out')
    children = raw.child_rows(row['runners'], raw.RUNNER_FIELDS)
    observed = {child['baserunner']: child for child in children}
    roles = [('Batter', context['batter_id'])] + [(role, context[role.lower()])
        for role in runner.ROLES[1:] if context[role.lower()]]
    complete = (set(observed) == {role for role, _ in roles}
        and all(observed[role]['runner_id'] == identity and identity
            and observed[role]['base_end'] in (*TARGETS,'Out') for role, identity in roles if role in observed))
    if not complete:
        failed.add('incomplete_vector')
    lexical = raw.lex(row['raw_play'])
    prefix = 'S' if row['result'] == 'Single' else 'D'
    notation = (lexical['status'] == 'split' and not lexical['base_separators']
        and len(lexical['base_atoms']) == 1
        and re.fullmatch(prefix+r'[0-9U]*', lexical['base_atoms'][0]) is not None
        and not raw.child_rows(row['flags'], raw.FLAG_FIELDS)
        and (not lexical['modifiers'] or len(lexical['modifiers']) == 1
            and re.fullmatch(r'[GFLP][1-9]?[LRSMDX]*[-+]?', lexical['modifiers'][0]) is not None))
    origins = set()
    for advance in lexical['advances']:
        match = re.fullmatch(r'([B123])-([123H])', advance)
        if match is None:
            notation = False
            continue
        origin, target = match.groups()
        role = {'B':'Batter', '1':'First', '2':'Second', '3':'Third'}[origin]
        endpoint = {'1':'First', '2':'Second', '3':'Third', 'H':'Home'}[target]
        if (origin in origins or role not in {r for r, _ in roles}
                or role not in observed or observed[role]['base_end'] != endpoint):
            notation = False
        origins.add(origin)
    if not notation:
        failed.add('notation_uncertain')
    if raw.child_rows(row['credits'], raw.CREDIT_FIELDS):
        failed.add('fielding_effect')
    for child in children:
        if (child['is_out'] is not False or child['explicit_out_flag'] is not False
                or child['advanced_on_error_flag'] is not False
                or child['base_end'] in TARGETS
                and child['run_scored_flag'] != (child['base_end'] == 'Home')):
            failed.add('runner_effect_or_unknown')
    batter = observed.get('Batter')
    if batter is not None and batter['base_end'] in (*TARGETS,'Out'):
        expected = 'First' if row['result'] == 'Single' else 'Second'
        if batter['base_end'] != expected:
            failed.add('hit_destination')
    # Missing endpoints cannot certify the full geometry, but must not erase
    # contradictions already established by independently observed named runners.
    boundary = context['frame_end']=='true'
    after_outs = outcome.number(context,'after_outs',3,optional=boundary)
    after_base_mask = outcome.number(context,'after_base_mask',7,optional=boundary)
    known = [(runner.ROLES.index(role),TARGETS.index(observed[role]['base_end'])+1)
        for role,_ in roles if role in observed and observed[role]['base_end'] in TARGETS]
    safe = not any(observed[role]['base_end']=='Out' for role,_ in roles if role in observed)
    if (not safe or any(target<origin for origin,target in known)
            or any(a>=b and not a==b==4 for (_,a),(_,b) in zip(known,known[1:]))
            or after_outs!=int(context['outs_before'])):
        failed.add('unsupported_geometry')
    if complete and safe:
        targets = [TARGETS.index(observed[role]['base_end'])+1 for role, _ in roles]
        origins = [runner.ROLES.index(role) for role, _ in roles]
        after_mask = 0
        for target in targets:
            if target < 4:
                after_mask |= 1 << (target-1)
        geometry = (all(target >= origin for target, origin in zip(targets, origins))
            and all(a < b or a == b == 4 for a, b in zip(targets, targets[1:]))
            and after_base_mask == after_mask
            and int(row['runs']) == targets.count(4)
            and after_outs == int(context['outs_before']))
        if not geometry:
            failed.add('unsupported_geometry')
    return dict(qualified=not failed, reasons=[reason for reason in REASONS if reason in failed],
        runners=[dict(observed[role]) for role in runner.ROLES if role in observed])


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups=None, expected_census=None, cohort=None):
    """Unchanged complete census validation precedes endpoint qualification."""
    rows, outcomes = tuple(raw_rows), tuple(outcome_rows)
    source = runner.inventory(rows, outcomes, supported_game_ids, native_groups=native_groups)
    if expected_census is not None and (not isinstance(expected_census,dict)
            or source != {k:v for k,v in expected_census.items() if k!='provenance'}):
        raise ValueError('Hit advancement census reconciliation failed')
    if cohort is not None:
        for key,value in cohort.items():
            actual = source[key]
            if key=='raw_totals':
                actual = {name:actual.get(name) for name in value}
            if actual != value:
                raise ValueError('Unsupported hit advancement source cohort/units')
    contexts = {int(context['event_key']):context for context in outcomes}
    supported = set(supported_game_ids)
    totals = dict(candidate_events=0, qualified_events=0, unresolved_events=0)
    first_reasons, all_reasons, vectors, outside = Counter(), Counter(), Counter(), Counter()
    split_totals = {split:dict(totals) for split in ('training','heldout','excluded')}
    for row in rows:
        if row['no_play_flag'] == 'true':
            continue
        if row['result'] not in ('Single', 'Double'):
            outside[row['result']] += 1
            continue
        context = contexts[int(row['event_key'])]
        value = classify(row, context)
        totals['candidate_events'] += 1
        totals['qualified_events' if value['qualified'] else 'unresolved_events'] += 1
        if value['reasons']:
            first_reasons[value['reasons'][0]] += 1
            all_reasons.update(value['reasons'])
        split = ('excluded' if row['game_id'] not in supported else
            'heldout' if hashlib.sha256(row['game_id'].encode('utf-8')).hexdigest()[0] in '012' else 'training')
        split_totals[split]['candidate_events'] += 1
        split_totals[split]['qualified_events' if value['qualified'] else 'unresolved_events'] += 1
        facts = dict(split=split, result=row['result'], base_mask=int(context['base_mask']),
            outs_before=int(context['outs_before']), trajectory=row['trajectory'],
            qualified=value['qualified'], reasons=value['reasons'],
            runners=[{k:v for k,v in child.items() if k!='runner_id'} for child in value['runners']])
        vectors[json.dumps(facts, sort_keys=True)] += 1
    assert totals['qualified_events'] + totals['unresolved_events'] == totals['candidate_events']
    assert sum(first_reasons.values()) == totals['unresolved_events']
    assert sum(vectors.values()) == totals['candidate_events']
    assert sum(outside.values()) + totals['candidate_events'] == source['totals']['real_events']
    assert sum(s['candidate_events'] for s in split_totals.values()) == totals['candidate_events']
    return dict(format=FORMAT, totals=totals, outside_results=dict(sorted(outside.items())),
        split_totals=split_totals,
        first_reasons=dict(sorted(first_reasons.items())),
        all_reasons=dict(sorted(all_reasons.items())),
        joint_vectors=[dict(json.loads(key),events=count) for key,count in sorted(vectors.items())],
        source_totals=source['totals'])


write_report = runner.write_report


def publish(snapshot, supplement, lineups, manifest, census, output):
    """Revalidate nine pinned local inputs and complete census before qualification."""
    output = Path(output)
    if output.exists() or output.is_symlink():
        raise ValueError('Refusing to overwrite hit advancement report')
    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    inputs = list(map(Path,[snapshot,supplement,lineups,manifest,census]))
    pins = tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs)
    if pins != INPUT_SHA:
        raise ValueError('Unsupported hit advancement input digest')
    exports = [inputs[4].with_suffix('.'+name+'.csv') for name in ('lineup','events','raw','groups')]
    queries = [Path(runner.__file__).with_name(name) for name in (
        '2019-lineup-readiness.sql','2019-outcomes.sql','2019-raw-mechanisms.sql','2019-runner-transitions.sql')]
    recipes = [Path(__file__)] + [Path(module.__file__) for module in
        (runner,lineup,outcome,raw,runner.joint_contact)]
    frozen = inputs + exports + queries + recipes
    all_pins = tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in frozen)
    if all_pins[:5] != pins:
        raise ValueError('Hit advancement inputs changed during read')
    if all_pins[5:9] != EXPORT_SHA:
        raise ValueError('Unsupported hit advancement native export digest')
    accepted_lineups, detail, accepted_census = [json.loads(path.read_text(encoding='utf-8'),
        object_pairs_hook=raw.unique_object) for path in inputs[2:5]]
    if not all(isinstance(item,dict) for item in (accepted_lineups,detail,accepted_census)):
        raise ValueError('Unsupported hit advancement provenance')
    if (detail.get('source_commit')!=lineup.SOURCE_COMMIT or detail.get('catalog_sha256')!=lineup.CATALOG
            or detail.get('snapshot_sha256')!=pins[0] or detail.get('database_sha256')!=pins[1]
            or detail.get('snapshot_id')!=857 or not isinstance(detail.get('attribution'),str)
            or not detail['attribution'].strip()):
        raise ValueError('Unsupported hit advancement detail provenance')
    lp = accepted_lineups.get('provenance')
    cp = accepted_census.get('provenance')
    if (not isinstance(lp,dict) or lp.get('snapshot_sha256')!=pins[0]
            or lp.get('supplement_sha256')!=pins[1] or lp.get('attribution')!=detail['attribution']):
        raise ValueError('Unsupported hit advancement lineup provenance')
    expected = dict(source_commit=lineup.SOURCE_COMMIT,snapshot_id=857,snapshot_sha256=pins[0],
        supplement_sha256=pins[1],lineup_report_sha256=pins[2],detail_manifest_sha256=pins[3],
        native_export_sha256=list(all_pins[5:9]),query_sha256=list(all_pins[9:13]),
        recipe_sha256=list(all_pins[14:]),attribution=detail['attribution'])
    if not isinstance(cp,dict) or any(cp.get(key)!=value for key,value in expected.items()):
        raise ValueError('Unsupported hit advancement census provenance')
    records = dict(game=[],person=[],appearance=[],manifest=[])
    with exports[0].open(newline='',encoding='utf-8') as stream:
        reader = csv.DictReader(stream,strict=True)
        if reader.fieldnames!=['record_type','payload']:
            raise ValueError('Invalid hit advancement lineup export header')
        for row in reader:
            if set(row)!=set(reader.fieldnames) or None in row.values() or row['record_type'] not in records:
                raise ValueError('Invalid hit advancement lineup export row')
            records[row['record_type']].append(json.loads(row['payload'],object_pairs_hook=raw.unique_object))
    if len(records['manifest'])!=1:
        raise ValueError('Unsupported hit advancement snapshot provenance')
    snapshot_provenance = records['manifest'][0]
    if (not isinstance(snapshot_provenance,dict) or snapshot_provenance!=cp.get('snapshot')
            or snapshot_provenance.get('schema_version')!='f03-spike-1'
            or snapshot_provenance.get('query_version')!='2019-filtered-extract-v1'
            or snapshot_provenance.get('catalog_sha256')!=lineup.CATALOG
            or snapshot_provenance.get('attribution')!=detail['attribution']
            or any(not isinstance(snapshot_provenance.get(key),str) or not snapshot_provenance[key].strip()
                for key in ('source_url','extracted_at'))):
        raise ValueError('Unsupported hit advancement snapshot provenance')
    fresh_lineups = lineup.inventory(records['game'],records['person'],records['appearance'])
    if fresh_lineups!={key:value for key,value in accepted_lineups.items() if key!='provenance'}:
        raise ValueError('Hit advancement lineup reconciliation failed')
    with exports[1].open(newline='',encoding='utf-8') as stream:
        reader = csv.DictReader(stream,strict=True)
        if (reader.fieldnames is None or len(reader.fieldnames)!=len(outcome.FIELDS)
                or set(reader.fieldnames)!=outcome.FIELDS):
            raise ValueError('Invalid hit advancement outcome export header')
        outcomes = list(reader)
        if any(set(row)!=outcome.FIELDS or None in row.values() for row in outcomes):
            raise ValueError('Invalid hit advancement outcome export row')
    with exports[2].open(newline='',encoding='utf-8') as stream:
        rows = list(raw.read_export(stream))
    with exports[3].open(newline='',encoding='utf-8') as stream:
        native = runner.read_native_groups(stream.read())
    if tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in frozen)!=all_pins:
        raise ValueError('Hit advancement inputs changed during read')
    report = inventory(rows,outcomes,fresh_lineups['candidate_game_ids'],native_groups=native,
        expected_census=accepted_census,cohort=COHORT)
    report['provenance'] = dict(expected,snapshot=snapshot_provenance,census_sha256=pins[4],
        qualification_recipe_sha256=all_pins[13])
    if output.resolve()!=output:
        raise ValueError('Hit advancement output path changed before publication')
    write_report(report,frozen,all_pins,output)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot','supplement','lineups','manifest','census','output'):
        parser.add_argument(name,type=Path)
    args = parser.parse_args()
    report = publish(args.snapshot,args.supplement,args.lineups,args.manifest,args.census,args.output)
    print(json.dumps(dict(format=report['format'],totals=report['totals']),sort_keys=True))


if __name__=='__main__':
    main()
