"""Inventory cached 2019 outcomes without fitting a model or inferring missing plays."""
import argparse
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path
import re

CATALOG = '2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583'
OPPORTUNITY_UNITS = ('force_exposure', 'steal_opportunities', 'extra_base_chances')
DERIVED_FLAGS = ('derived_known', 'derived_unknown', 'derived_fly', 'derived_ground',
                 'derived_line', 'derived_pop', 'derived_bunts', 'derived_location_known', 'derived_fielder_known')
RUNNER_UNITS = ('runner_runs', 'runner_outs', 'unforced_runner_outs', 'steals', 'cs',
                'pickoffs', 'wp', 'pb', 'balk', 'unspecified', 'indifference', 'error_advances')
FIELDS = set(('event_key game_id event_id season result pa trajectory location depth fielder '
              'bip outs_before base_mask outs runs state_outs state_runs '
              'batting_outs putouts assists errors dp tp gidp frame_end after_outs '
              'after_base_mask batter_id pitcher_id first second third personnel_key '
              'runner_rows').split()) | set(RUNNER_UNITS) | set(DERIVED_FLAGS) | set(OPPORTUNITY_UNITS)


def number(row, name, maximum=None, optional=False):
    value = row[name]
    if value == '' and optional:
        return None
    if not isinstance(value, str) or not re.fullmatch(r'[0-9]+', value):
        raise ValueError(f"event {row.get('event_key')}: missing/invalid {name}")
    value = int(value)
    if maximum is not None and value > maximum:
        raise ValueError(f"event {row['event_key']}: out-of-range {name}")
    return value


def inventory(rows):
    totals = Counter(dict(events=0, pa=0, non_pa=0, outs=0, runs=0, putouts=0, runner_run_units=0))
    coverage, signals, results, raw, derived_flags, multiplicity = (Counter() for _ in range(6))
    signatures, contexts, fielders, runner_units = (Counter() for _ in range(4))
    opportunity_units, opportunity_contexts = Counter(), {}
    locations, depths, fielding_units = Counter(), Counter(), Counter()
    examples, keys, games = {}, set(), set()
    for row in rows:
        if set(row) != FIELDS or any(not isinstance(v, str) for v in row.values()):
            raise ValueError('Unrecognized or incomplete outcome export row')
        key = number(row, 'event_key')
        if key in keys:
            raise ValueError(f'duplicate event key {key}; possible join fanout')
        keys.add(key)
        if row['season'] != '2019' or any(not row[f] for f in ('game_id', 'batter_id', 'pitcher_id', 'personnel_key')):
            raise ValueError(f'event {key}: invalid scope or missing identity/fielding join')
        number(row, 'event_id')
        games.add(row['game_id'])
        pa = number(row, 'pa', 1, optional=True)
        if (bool(row['result']) and pa != 1) or (not row['result'] and pa not in (None, 0)):
            raise ValueError(f'event {key}: result/PA classification mismatch')
        made, runs, before, mask = (number(row, f, limit) for f, limit in
                                  [('outs', 3), ('runs', None), ('outs_before', 2), ('base_mask', 7)])
        if before + made > 3:
            raise ValueError(f'event {key}: more than three outs in frame')
        for i, name in enumerate(('first', 'second', 'third')):
            if bool(row[name]) != bool(mask & (1 << i)):
                raise ValueError(f'event {key}: before base mask/identity mismatch')
        occupied = [row[f] for f in ('first', 'second', 'third') if row[f]]
        if len(occupied) != len(set(occupied)):
            raise ValueError(f'event {key}: duplicate occupied runner identity')
        for name in ('state_outs', 'putouts'):
            if number(row, name) != made:
                raise ValueError(f'event {key}: {name}/event outs mismatch')
        if pa == 1 and number(row, 'batting_outs') != made:
            raise ValueError(f'event {key}: batting/event outs mismatch')
        if number(row, 'state_runs') != runs:
            raise ValueError(f'event {key}: state/event runs mismatch')
        if row['frame_end'] not in ('true', 'false'):
            raise ValueError(f'event {key}: missing frame-end classification')
        boundary = row['frame_end'] == 'true'
        after_outs = number(row, 'after_outs', 3, optional=boundary)
        number(row, 'after_base_mask', 7, optional=boundary)
        if not boundary and after_outs != before + made:
            raise ValueError(f'event {key}: active after-outs mismatch')
        n_runner = number(row, 'runner_rows')
        supplied = {}
        for name in RUNNER_UNITS + OPPORTUNITY_UNITS:
            value = number(row, name, optional=n_runner == 0)
            if n_runner == 0 and value not in (None, 0):
                raise ValueError(f'event {key}: units without runner rows')
            # No rows means zero SUPPLIED credit units, not complete runner history.
            supplied[name] = 0 if value is None else value
        if supplied['runner_runs'] != runs:
            raise ValueError(f'event {key}: supplied runner/event runs mismatch')
        if supplied['unforced_runner_outs'] > supplied['runner_outs'] or supplied['runner_outs'] > made:
            raise ValueError(f'event {key}: runner-out subset mismatch')
        fielding_units.update(putouts=number(row, 'putouts'), assists=number(row, 'assists'), errors=number(row, 'errors'))
        dp, tp, gidp = (number(row, name, 1, optional=pa != 1) for name in ('dp', 'tp', 'gidp'))
        fielder = number(row, 'fielder', 9, optional=True)
        bip_value = number(row, 'bip', 1, optional=True)
        bip = bip_value == 1
        flags = {f: number(row, f, 1, optional=bip_value is None) for f in DERIVED_FLAGS}
        derived_flags.update({f: v for f, v in flags.items() if v is not None})
        result = row['result'] or '<non-PA>'
        trajectory = row['trajectory'] or '<missing>'
        signature = (result, trajectory, str(made), str(dp), str(tp), str(gidp))
        context = (result, trajectory, str(before), str(mask), str(made))
        signatures[signature] += 1
        contexts[context] += 1
        fielders[(result, trajectory, row['fielder'] or '<missing>')] += 1
        if signature not in examples or key < int(examples[signature]['event_key']):
            examples[signature] = dict(row)
        results[result] += 1
        raw[trajectory] += 1
        locations[row['location'] or '<missing>'] += 1
        depths[row['depth'] or '<missing>'] += 1
        multiplicity[str(made)] += 1
        totals.update(events=1, pa=int(pa == 1), non_pa=int(pa != 1), outs=made,
                      runs=runs, putouts=made, runner_run_units=supplied['runner_runs'])
        runner_units.update({f: supplied[f] for f in RUNNER_UNITS})
        opportunity_units.update({f: supplied[f] for f in OPPORTUNITY_UNITS})
        cell = opportunity_contexts.setdefault(context, Counter())
        cell.update(events=1, runner_rows=n_runner, events_without_runner_rows=int(n_runner == 0))
        cell.update({f: supplied[f] for f in OPPORTUNITY_UNITS})
        coverage.update(frame_end_events=int(boundary), events_without_runner_rows=int(n_runner == 0),
                        bip_events=int(bip),
                        bip_raw_trajectory_unknown=int(bip and trajectory in ('<missing>', 'Unknown')),
                        bip_raw_location_unknown=int(bip and row['location'] in ('', 'Unknown')),
                        bip_raw_depth_unknown=int(bip and row['depth'] in ('', 'Unknown')),
                        bip_fielder_unknown=int(bip and fielder in (None, 0)),
                        bip_derived_known_raw_unknown=int(bip and trajectory in ('<missing>', 'Unknown')
                                                         and flags['derived_known'] == 1),
                        bip_derived_trajectory_unknown=int(bip and flags['derived_known'] != 1),
                        bip_derived_location_unknown=int(bip and flags['derived_location_known'] != 1),
                        bip_known_without_narrow_flag=int(bip and flags['derived_known'] == 1
                                                         and not any(flags[f] for f in ('derived_fly', 'derived_ground', 'derived_line', 'derived_pop'))))
        signals.update(double_play_events=int(dp == 1), triple_play_events=int(tp == 1),
                       gidp_events=int(gidp == 1),
                       strikeout_events_without_out=int(result == 'StrikeOut' and made == 0),
                       strikeout_events_with_multiple_outs=int(result == 'StrikeOut' and made > 1))
    if not totals['events']:
        raise ValueError('Empty outcome export')

    def cells(counter, names):
        return [dict(zip(names, values), events=n) for values, n in sorted(counter.items())]

    return dict(format='2019-outcome-inventory-v1', games=len(games), totals=dict(totals),
                results=dict(sorted(results.items())), raw_trajectories=dict(sorted(raw.items())),
                raw_locations=dict(sorted(locations.items())), raw_depths=dict(sorted(depths.items())),
                supplied_fielding_units=dict(fielding_units),
                derived_contact_flags=dict(sorted(derived_flags.items())), out_multiplicity=dict(sorted(multiplicity.items())),
                coverage=dict(coverage), signals=dict(signals), supplied_runner_units=dict(runner_units),
                supplied_opportunity_units=dict(opportunity_units),
                opportunity_contexts=[dict(zip(['result', 'trajectory', 'outs_before', 'base_mask', 'outs'], k), **dict(v))
                                      for k, v in sorted(opportunity_contexts.items())],
                signatures=cells(signatures, ['result', 'trajectory', 'outs', 'dp', 'tp', 'gidp']),
                contexts=cells(contexts, ['result', 'trajectory', 'outs_before', 'base_mask', 'outs']),
                fielders=cells(fielders, ['result', 'trajectory', 'position_tag']),
                examples=[examples[k] for k in sorted(examples)])


def read_manifest(path):
    with path.open(newline='') as stream:
        rows = list(csv.DictReader(stream, strict=True))
    if (len(rows) != 1 or rows[0].get('catalog_sha256') != CATALOG
            or rows[0].get('schema_version') != 'f03-spike-1'
            or rows[0].get('query_version') != '2019-filtered-extract-v1'
            or not rows[0].get('source_url') or not rows[0].get('extracted_at') or not rows[0].get('attribution')):
        raise ValueError('Missing or unsupported snapshot provenance')
    return rows[0]


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('events', type=Path)
    parser.add_argument('manifest', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    provenance = read_manifest(args.manifest)
    with args.events.open(newline='') as stream:
        reader = csv.DictReader(stream, strict=True)
        if reader.fieldnames is None or len(reader.fieldnames) != len(set(reader.fieldnames)) or set(reader.fieldnames) != FIELDS:
            raise ValueError('Invalid outcome export header')
        report = inventory(reader)
    report['provenance'] = dict(snapshot=provenance, events_sha256=digest(args.events),
                                manifest_sha256=digest(args.manifest),
                                query_sha256=digest(Path(__file__).with_name('2019-outcomes.sql')))
    args.output.write_text(json.dumps(report, sort_keys=True, indent=2) + '\n')
    print(json.dumps(dict(totals=report['totals'], coverage=report['coverage'], signals=report['signals']), sort_keys=True))


if __name__ == '__main__':
    main()
