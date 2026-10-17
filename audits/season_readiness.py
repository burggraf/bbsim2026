"""Offline all-team 2019 readiness; not an eligible roster or season simulator."""
import argparse
from collections import Counter, defaultdict
from datetime import date
import json
from pathlib import Path
import re
import subprocess

from audits import lineup_readiness as lineup
from audits.outcome_inventory import digest
from audits.raw_mechanisms import unique_object


def day(value):
    try:
        parsed = date.fromisoformat(value)
        if parsed.year == 2019:
            return parsed.isoformat()
    except (TypeError, ValueError):
        pass
    raise ValueError('Expected a literal 2019 date')


def reconcile_schedule(schedule, games):
    """Conservative identity candidates; never choose an arbitrary doubleheader."""
    by_key = defaultdict(list)
    ids = set()
    numbers = {'SingleGame': 0, 'DoubleHeaderGame1': 1, 'DoubleHeaderGame2': 2}
    for g in games:
        gid = g['game_id']
        if not isinstance(gid, str) or not gid or gid in ids or g['doubleheader_status'] not in numbers:
            raise ValueError('Invalid/duplicate game identity or doubleheader status')
        ids.add(gid)
        key = day(g['date']), g['away_team_id'], g['home_team_id']
        if any(not isinstance(t, str) or not t.strip() for t in key[1:]) or key[1] == key[2]:
            raise ValueError('Invalid game teams')
        by_key[key].append(g)
    rows = []
    grains = set()
    for row in sorted(schedule, key=lambda r: json.dumps(r, sort_keys=True)):
        if not isinstance(row['date'], str):
            raise ValueError('Invalid schedule date')
        original = day(row['date'][:10])
        if row['date'] not in (original, original + ' 00:00:00'):
            raise ValueError('Invalid schedule timestamp')
        number = row['double_header']
        key = original, row['visiting_team'], row['home_team']
        if (type(number) is not int or number not in (0, 1, 2)
                or any(not isinstance(t, str) or not t.strip() for t in key[1:]) or key[1] == key[2]
                or (*key, number) in grains):
            raise ValueError('Invalid/duplicate schedule grain')
        grains.add((*key, number))
        d = dict(source=dict(row), effective_date=original, candidate_game_ids=[], status='unmatched', game_id=None)
        reason, makeup = row['postponement_indicator'], row['makeup_dates']
        if ((reason is not None and not isinstance(reason, str))
                or (makeup is not None and not isinstance(makeup, str))):
            raise ValueError('Invalid schedule annotations')
        if reason:
            if makeup and re.fullmatch(r'\d{8} - Not made up', makeup):
                d.update(status='not_made_up', effective_date=None)
            elif makeup and re.fullmatch(r'\d{8}', makeup):
                d['effective_date'] = day(f'{makeup[:4]}-{makeup[4:6]}-{makeup[6:]}')
            else:
                d.update(status='unknown', effective_date=None)
        elif makeup:
            d.update(status='unknown', effective_date=None)
        if d['effective_date'] is not None:
            candidates = by_key[d['effective_date'], *key[1:]]
            # Original DH numbering is not evidence of a postponed game's makeup order.
            if number and not reason:
                candidates = [g for g in candidates if numbers[g['doubleheader_status']] == number]
            d['candidate_game_ids'] = sorted(g['game_id'] for g in candidates)
        rows.append(d)
    claims = Counter(gid for row in rows for gid in row['candidate_game_ids'])
    assigned = set()
    for row in rows:
        candidates = row['candidate_game_ids']
        if candidates:
            if len(candidates) == 1 and claims[candidates[0]] == 1:
                row.update(status='matched', game_id=candidates[0]); assigned.add(candidates[0])
            else:
                row['status'] = 'ambiguous'
    return dict(counts={s: sum(r['status'] == s for r in rows)
                        for s in ('matched', 'unmatched', 'ambiguous', 'not_made_up', 'unknown')},
                rows=rows, unassigned_game_ids=sorted(ids - assigned))


def publish(snapshot, supplement, output):
    output = Path(output).resolve()
    raw = output.with_suffix('.native.json'); errors = output.with_suffix('.native.stderr')
    line = output.with_suffix('.lineup.json')
    if output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Readiness evidence must be outside the repository')
    if any(p.exists() for p in (output, raw, errors, line, line.with_suffix('.native.csv'), line.with_suffix('.native.stderr'))):
        raise ValueError('Refusing to overwrite readiness evidence')
    query = Path(__file__).with_name('2019-season-readiness.sql')
    inputs = [Path(snapshot), Path(supplement), query, Path(__file__),
              Path(lineup.__file__), Path(lineup.__file__).with_name('2019-lineup-readiness.sql')]
    before = [digest(p) for p in inputs]
    if tuple(before[:2]) != lineup.INPUT_SHA:
        raise ValueError('Unsupported accepted snapshots')
    personnel = lineup.publish(Path(snapshot), Path(supplement), line)
    with raw.open('x') as out, errors.open('x') as err:
        subprocess.run(['duckdb', '-init', '/dev/null', '-readonly', '-bail', '-batch', '-json',
                        '-cmd', 'SET enable_external_access=false', str(Path(snapshot).resolve()),
                        '-f', str(query.resolve())], stdout=out, stderr=err, check=True, timeout=300)
    records = json.loads(raw.read_text(), object_pairs_hook=unique_object)
    groups = defaultdict(list)
    for row in records:
        groups[row['record_type']].append(json.loads(row['payload'], object_pairs_hook=unique_object))
    if set(groups) != {'schedule', 'game', 'reference', 'profile'}:
        raise ValueError('Unexpected readiness export groups')
    result = reconcile_schedule(groups['schedule'], groups['game'])
    checks = {g['game_id']: g for g in personnel['checks']}
    if set(checks) != {g['game_id'] for g in groups['game']}:
        raise ValueError('Schedule/personnel game sets disagree')
    result.update(format='2019-season-readiness-v1', games=groups['game'],
                  personnel=personnel, references=groups['reference'], profiles=groups['profile'])
    if [digest(p) for p in inputs] != before:
        raise ValueError('Readiness inputs changed during audit')
    result['provenance'] = dict(input_sha256={str(p.name): h for p, h in zip(inputs, before)},
                                native_export_sha256=digest(raw), lineup_export_sha256=digest(line.with_suffix('.native.csv')))
    with output.open('x') as f:
        json.dump(result, f, indent=2, sort_keys=True, allow_nan=False); f.write('\n')
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot', 'supplement', 'output'):p.add_argument(name, type=Path)
    args = p.parse_args(); r = publish(args.snapshot, args.supplement, args.output)
    print(json.dumps(dict(schedule=r['counts'], games=len(r['games']), unassigned=len(r['unassigned_game_ids']),
                         personnel=r['personnel']['coverage']), sort_keys=True))


if __name__ == '__main__':main()
