"""Offline observed continuation/change census; not removal causes or fatigue."""
import argparse
from collections import Counter, defaultdict
import csv
import hashlib
import json
from pathlib import Path

from audits import pitcher_workload, player_opportunities
from audits.raw_mechanisms import unique_object

HITS = {'Single', 'Double', 'GroundRuleDouble', 'Triple', 'HomeRun', 'InsideTheParkHomeRun'}
CONTEXT_FIELDS = ['event_key', 'game_id', 'event_id', 'inning', 'batting_side']


def inventory(contexts, events, workload):
    contexts, events = list(contexts), list(events)
    by_key = {r['event_key']: r for r in contexts}
    if len(by_key) != len(contexts) or {r['event_key'] for r in events} != set(by_key):
        raise ValueError('Context/event keys do not reconcile')
    games = defaultdict(list)
    seen = set()
    for event in events:
        c = by_key[event['event_key']]
        if (event['event_key'] in seen or c['game_id'] != event['game_id']
                or c['event_id'] != event['event_id']
                or c['batting_side'] not in ('Away', 'Home')
                or not c['inning'].isascii() or not c['inning'].isdigit() or int(c['inning']) < 1):
            raise ValueError('Invalid context identity/dimensions')
        seen.add(event['event_key'])
        games[event['game_id']].append((event, c))
    decisions = defaultdict(list)
    for gid, rows in sorted(games.items()):
        rows.sort(key=lambda pair: int(pair[0]['event_id']))
        if len({r['event_id'] for r, _ in rows}) != len(rows):
            raise ValueError('Duplicate game event ID')
        score = [0, 0]
        counters = defaultdict(Counter)
        defensive = [[], []]
        for event, c in rows:
            offense = int(c['batting_side'] == 'Home'); defense = 1 - offense
            score[offense] += int(event['runs'])
            pid = event['pitcher_id']; counter = counters[pid]
            counter.update(pa=int(bool(event['result'])), outs=int(event['outs']),
                           hits=int(event['result'] in HITS), runs=int(event['runs']))
            d = dict(event_id=int(event['event_id']), inning=int(c['inning']),
                     terminal_pa=bool(event['result']), lead=score[defense] - score[offense],
                     **counter)
            defensive[defense].append((pid, d))
        for side, rows in enumerate(defensive):
            for i, (pid, d) in enumerate(rows):
                d['status'] = ('game_end_censored' if i + 1 == len(rows) else
                               'continue' if rows[i + 1][0] == pid else 'change')
                decisions[gid, pid, ('Away', 'Home')[side]].append(d)
    outings = []; excluded = 0; totals = Counter(); cells = defaultdict(Counter)
    for player in sorted(workload['players'], key=lambda p: p['player_id']):
        for outing in sorted(player['outing_rows'], key=lambda o: o['game_id']):
            if not outing['training']:
                excluded += 1
                continue
            key = outing['game_id'], player['player_id'], outing['side']
            ds = decisions.get(key, [])
            pa = sum(d['terminal_pa'] for d in ds)
            if (pa != outing['terminal_pa'] or len(ds) - pa != outing['non_pa']
                    or outing['role'] not in ('start', 'relief')):
                raise ValueError('Selected outing/event reconciliation failed')
            last = ds[-1] if ds else dict(pa=0, outs=0, hits=0, runs=0)
            outings.append(dict(game_id=key[0], player_id=key[1], role=outing['role'],
                                multiple_appearance_rows=outing['position1_rows'] != 1,
                                **{k: last[k] for k in ('pa', 'outs', 'hits', 'runs')}, decisions=ds))
            totals.update(outings=1, events=len(ds), pa=pa, zero_event_outings=int(not ds))
            for d in ds:
                totals['continue_' if d['status'] == 'continue' else d['status']] += 1
                bucket = '1-8' if d['pa'] < 9 else '9-17' if d['pa'] < 18 else '18-26' if d['pa'] < 27 else '27+'
                run_bucket = '0' if not d['runs'] else '1-3' if d['runs'] < 4 else '4+'
                cell = (outing['role'], 'terminal_pa' if d['terminal_pa'] else 'runner_only',
                        '0' if not d['pa'] else bucket, run_bucket,
                        'leading' if d['lead'] > 0 else 'tied_or_trailing')
                cells[cell][d['status']] += 1
    totals = {k: totals[k] for k in ['outings', 'events', 'pa', 'zero_event_outings',
                                    'continue_', 'change', 'game_end_censored']}
    assert totals['events'] == totals['continue_'] + totals['change'] + totals['game_end_censored']
    return dict(format='2019-pitcher-usage-v1', outings=outings, totals=totals, excluded_outings=excluded,
                conditional_cells=[dict(zip(['role', 'decision', 'pa', 'runs', 'lead'], key), **value)
                                   for key, value in sorted(cells.items())])


def read_csv(path, fields):
    with path.open(newline='', encoding='utf-8') as f:
        reader = csv.DictReader(f, strict=True)
        if reader.fieldnames != fields:
            raise ValueError('Unexpected usage export header')
        rows = list(reader)
    if any(set(r) != set(fields) or any(not isinstance(v, str) for v in r.values()) for r in rows):
        raise ValueError('Malformed usage export row')
    return rows


def publish(contexts, events, workload, output):
    paths = list(map(Path, [contexts, events, workload]))
    output = Path(output).resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]) or output.exists():
        raise ValueError('Usage report requires a new path outside the repository')
    recipes = [Path(__file__), Path(__file__).with_name('2019-pitcher-usage.sql')]
    digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    before = {str(p): digest(p) for p in paths + recipes}
    accepted = json.loads(paths[2].read_text(), object_pairs_hook=unique_object)
    if (accepted.get('format') != '2019-pitcher-workload-v1'
            or accepted.get('basis') != pitcher_workload.BASIS
            or digest(paths[2]) != '125768bd82ce54ead277d92301fbb7434e823a8fc4432777087c15b8e46ad2f9'
            or digest(paths[1]) != accepted['provenance']['native_export_sha256'][1]):
        raise ValueError('Unsupported accepted workload/events provenance')
    from audits.outcome_inventory import FIELDS
    with paths[1].open(newline='', encoding='utf-8') as f:
        header = next(csv.reader(f))
    if len(header) != len(FIELDS) or set(header) != FIELDS:
        raise ValueError('Unexpected outcome export header')
    rows = read_csv(paths[1], header)
    validated = player_opportunities.inventory(rows)
    if (hashlib.sha256(json.dumps(validated['roles'], sort_keys=True).encode()).hexdigest()
            != accepted['opportunity_roles_sha256']):
        raise ValueError('Usage opportunity profile reconciliation failed')
    report = inventory(read_csv(paths[0], CONTEXT_FIELDS), rows, accepted)
    report['provenance'] = dict(input_sha256=before, workload=accepted['provenance'],
                                basis='terminal-participant PA; recorded outs and runs while pitching; not official IP/BF or removal reasons')
    if before != {str(p): digest(p) for p in paths + recipes}:
        raise ValueError('Usage inputs changed before publication')
    with output.open('x', encoding='utf-8') as f:
        json.dump(report, f, sort_keys=True, indent=2, allow_nan=False); f.write('\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['contexts', 'events', 'workload', 'output']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    report = publish(args.contexts, args.events, args.workload, args.output)
    print(json.dumps(report['totals'], sort_keys=True))


if __name__ == '__main__': main()
