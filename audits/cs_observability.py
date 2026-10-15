"""Literal caught-stealing observability; no imputed target or attempt rates."""
from collections import Counter
import argparse
import csv
import hashlib
import json
from pathlib import Path
import tempfile
from audits import runner_only_steals as steals

FORMAT = '2019-cs-observability-v1'
FLAGS = ('is_out', 'run_scored_flag', 'explicit_out_flag', 'advanced_on_error_flag', 'is_advance_attempt')
LABELS = ('CaughtStealing', 'PickedOffCaughtStealing')
TOTAL_FIELDS = ('totals', 'split_totals', 'source_totals', 'raw_totals', 'outcome_totals', 'supplied_credit_units')
INPUT_SHA = steals.INPUT_SHA + ('b31083c4b2e9f9e3d623996f939c9d6e1afde9e483b07a432e535993543b15bc',)


def observe(row, context):
    """Retain literal child values; no-play has no invented before-state vector."""
    children = sorted(steals.raw.child_rows(row['runners'], steals.raw.RUNNER_FIELDS),
                      key=lambda child: steals.runner.ROLES.index(child['baserunner']))
    selected = [child for child in children if child['baserunning_play_type'] in LABELS]
    missing = qualification = None
    if context is not None:
        before = [role for role in steals.ROLES if context[role.lower()]]
        observed = {child['baserunner']: child for child in children}
        present = [observed[role] for role in before if role in observed]
        missing = dict(children=len(before)-len(present),
                       identities=sum(child['runner_id'] is None for child in present),
                       endpoints=sum(child['base_end'] is None for child in present),
                       flags={flag: sum(child[flag] is None for child in present) for flag in FLAGS})
        if context['pa'] != '1':
            value = steals.classify(row, context)
            qualification = {key: value[key] for key in ('qualified', 'reasons')}
    return dict(children=selected, labels=sorted({child['baserunning_play_type'] for child in selected}),
                missing=missing, qualification=qualification)


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups=None, expected_prior=None):
    """Unchanged complete prior validation precedes any new observation."""
    raw_rows, outcomes = tuple(raw_rows), tuple(outcome_rows)
    prior = steals.inventory(raw_rows, outcomes, supported_game_ids, native_groups=native_groups)
    if expected_prior is not None and prior != {k: v for k, v in expected_prior.items() if k != 'provenance'}:
        raise ValueError('CS observability prior reconciliation failed')
    contexts = {int(row['event_key']): row for row in outcomes}
    supported = set(supported_game_ids)
    totals = dict(raw_events=len(raw_rows), candidate_events=0, candidate_rows=0,
                  outside_events=0, outside_child_rows=0)
    events, rows, outside_events, outside_rows = (Counter() for _ in range(4))
    def add(counter, value):
        counter[json.dumps(value, sort_keys=True)] += 1
    for row in raw_rows:
        no_play = row['no_play_flag'] == 'true'
        context = None if no_play else contexts[int(row['event_key'])]
        partition = 'no_play' if no_play else 'pa' if context['pa'] == '1' else 'nonpa'
        split = ('excluded' if row['game_id'] not in supported else 'heldout'
                 if hashlib.sha256(row['game_id'].encode('utf-8')).hexdigest()[0] in '012' else 'training')
        common = dict(partition=partition, split=split, result=row['result'])
        value = observe(row, context)
        if value['children']:
            totals['candidate_events'] += 1
            add(events, dict(common, **value))
        else:
            totals['outside_events'] += 1
            add(outside_events, common)
        for child in steals.raw.child_rows(row['runners'], steals.raw.RUNNER_FIELDS):
            selected = child['baserunning_play_type'] in LABELS
            totals['candidate_rows' if selected else 'outside_child_rows'] += 1
            cell = dict(common, role=child['baserunner'], label=child['baserunning_play_type'],
                        identity_present=child['runner_id'] is not None,
                        **{key: child[key] for key in (*FLAGS, 'attempted_advance_to_base', 'base_end')})
            add(rows if selected else outside_rows, cell)
    if (totals['candidate_events'] + totals['outside_events'] != totals['raw_events']
            or totals['candidate_rows'] + totals['outside_child_rows'] != prior['raw_totals']['runner_rows']
            or sum(events.values()) != totals['candidate_events']
            or sum(rows.values()) != totals['candidate_rows']):
        raise ValueError('CS observability partition conservation failed')
    def cells(counter, unit):
        return [dict(json.loads(key), **{unit: count}) for key, count in sorted(counter.items())]
    return dict(format=FORMAT, basis='literal missing-field combinations, not inferred attempts or rates',
                totals=totals, events=cells(events, 'events'), rows=cells(rows, 'rows'),
                outside_events=cells(outside_events, 'events'), outside_rows=cells(outside_rows, 'rows'),
                prior_totals={name: prior[name] for name in TOTAL_FIELDS})


def publish(snapshot, supplement, lineups, manifest, census, prior, output):
    output = Path(output)
    if output.exists() or output.is_symlink():
        raise ValueError('Refusing to overwrite CS observability report')
    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    inputs = list(map(Path, (snapshot, supplement, lineups, manifest, census, prior)))
    if steals._pins(inputs) != INPUT_SHA:
        raise ValueError('Unsupported CS observability input digest')
    exports = [inputs[4].with_suffix('.' + name + '.csv') for name in ('lineup', 'events', 'raw', 'groups')]
    queries = [Path(steals.runner.__file__).with_name(name) for name in (
        '2019-lineup-readiness.sql', '2019-outcomes.sql', '2019-raw-mechanisms.sql', '2019-runner-transitions.sql')]
    recipes = [Path(__file__)] + [Path(module.__file__) for module in (
        steals, steals.runner, steals.lineup, steals.outcome, steals.raw, steals.runner.joint_contact)]
    frozen = inputs + exports + queries + recipes
    if any(path.is_symlink() for path in frozen):
        raise ValueError('CS observability input symlinks are unsupported')
    identities = tuple(steals._identity(path) for path in frozen)
    parent = steals._identity(output.parent)
    pins = steals._pins(frozen)
    if pins[:6] != INPUT_SHA:
        raise ValueError('CS observability inputs changed during read')
    def unchanged():
        if output.resolve() != output or steals._identity(output.parent) != parent:
            raise ValueError('CS observability output path changed before publication')
        if tuple(steals._identity(path) for path in frozen) != identities or steals._pins(frozen) != pins:
            raise ValueError('CS observability input bytes/path changed before publication')
    accepted = json.loads(inputs[5].read_text(encoding='utf-8'), object_pairs_hook=steals.raw.unique_object)
    # This unchanged controller authenticates every input/export/provenance and
    # runs complete lineup/source/raw/native/census validators, never a subset.
    with tempfile.TemporaryDirectory(prefix='bbsim-cs-prior-') as directory:
        report_path = Path(directory) / 'prior.json'
        fresh = steals.publish(*inputs[:5], report_path)
        if fresh != accepted or report_path.read_bytes() != inputs[5].read_bytes():
            raise ValueError('CS observability accepted prior reconciliation failed')
    unchanged()
    lineups_report = json.loads(inputs[2].read_text(encoding='utf-8'), object_pairs_hook=steals.raw.unique_object)
    with exports[1].open(newline='', encoding='utf-8') as stream:
        reader = csv.DictReader(stream, strict=True)
        if (reader.fieldnames is None or len(reader.fieldnames) != len(steals.outcome.FIELDS)
                or set(reader.fieldnames) != steals.outcome.FIELDS):
            raise ValueError('Invalid CS observability outcome export header')
        outcomes = list(reader)
    with exports[2].open(newline='', encoding='utf-8') as stream:
        rows = list(steals.raw.read_export(stream))
    native = steals.runner.read_native_groups(exports[3].read_text(encoding='utf-8'))
    unchanged()
    report = inventory(rows, outcomes, lineups_report['candidate_game_ids'],
                       native_groups=native, expected_prior=accepted)
    report['provenance'] = dict(prior_report_sha256=pins[5], input_sha256=list(pins[:6]),
        native_export_sha256=list(pins[6:10]), query_sha256=list(pins[10:14]),
        recipe_sha256=list(pins[14:]), source_commit=steals.lineup.SOURCE_COMMIT,
        snapshot_id=857, attribution=fresh['provenance']['attribution'])
    unchanged()
    steals.write_report(report, frozen, pins, output, recheck=unchanged)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot', 'supplement', 'lineups', 'manifest', 'census', 'prior', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    report = publish(args.snapshot, args.supplement, args.lineups, args.manifest,
                     args.census, args.prior, args.output)
    print(json.dumps(dict(format=report['format'], totals=report['totals']), sort_keys=True))


if __name__ == '__main__':
    main()
