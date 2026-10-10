"""Compare F-02 fixture metrics to separately published Retrosheet 2019 box files.

The zip and reconcile CSV are external artifacts; no source rows are committed.
Fields are defined by https://www.retrosheet.org/boxfile.txt (version 3).
"""

import argparse
import csv
import io
from pathlib import Path
import zipfile


METRICS = {'runs', 'hits', 'pa', 'steals', 'caught_stealing', 'errors', 'putouts', 'assists', 'outs'}


def number(row, index):
    if index >= len(row) or row[index] == '':
        raise ValueError(f'missing box number in {row[:3]} at index {index}')
    return int(row[index])


def box_metrics(rows):
    teams = {}
    totals = {side: dict.fromkeys(METRICS, 0) for side in (0, 1)}
    seen = {kind: set() for kind in ('line', 'batting', 'fielding')}
    for row in rows:
        if not row:
            continue
        if row[:2] == ['info', 'visteam']:
            teams[0] = row[2]
        elif row[:2] == ['info', 'hometeam']:
            teams[1] = row[2]
        elif row[0] == 'line':
            side = number(row, 1)
            if side not in totals:
                raise ValueError(f'bad box side: {side}')
            seen['line'].add(side)
            totals[side]['runs'] += sum(int(v) for v in row[2:] if v.lower() != 'x')
        elif row[:2] == ['stat', 'bline']:
            side = number(row, 3)
            if side not in totals:
                raise ValueError(f'bad box side: {side}')
            seen['batting'].add(side)
            t = totals[side]
            t['hits'] += number(row, 8)
            t['pa'] += sum(number(row, i) for i in (6, 13, 14, 15, 16, 22))
            t['steals'] += number(row, 19)
            t['caught_stealing'] += number(row, 20)
        elif row[:2] == ['stat', 'dline']:
            side = number(row, 3)
            if side not in totals:
                raise ValueError(f'bad box side: {side}')
            seen['fielding'].add(side)
            t = totals[side]
            t['putouts'] += number(row, 7)
            t['assists'] += number(row, 8)
            t['errors'] += number(row, 9)
    if set(teams) != {0, 1}:
        raise ValueError('missing box team identities')
    for kind, sides in seen.items():
        if sides != {0, 1}:
            raise ValueError(f'missing box {kind} for sides {sorted({0, 1} - sides)}')
    # F-02's 'outs' is fielding-team outs_on_play, not offensive outs.
    for side in (0, 1):
        totals[side]['outs'] = totals[side]['putouts']
    return teams, totals


def read_games(archive, wanted, parser=box_metrics):
    found = {}
    with zipfile.ZipFile(archive) as z:
        for member in ('2019.EBA', '2019.EBN'):
            info = z.getinfo(member)
            if info.file_size > 50_000_000:
                raise ValueError(f'oversized box member: {member}')
            current, rows = None, []
            for row in csv.reader(io.StringIO(z.read(member).decode('latin-1'))):
                if row and row[0] == 'id':
                    if current is not None:
                        if current in found:
                            raise ValueError(f'duplicate box game: {current}')
                        found[current] = parser(rows)
                    current = row[1] if row[1] in wanted else None
                    rows = []
                elif current is not None:
                    rows.append(row)
            if current is not None:
                if current in found:
                    raise ValueError(f'duplicate box game: {current}')
                found[current] = parser(rows)
    missing = wanted - found.keys()
    if missing:
        raise ValueError(f'missing box game: {sorted(missing)}')
    return found


def compare_box(archive, reconcile):
    with Path(reconcile).open(newline='') as f:
        reference = list(csv.reader(f))
    if not reference or any(len(row) != 7 for row in reference):
        raise ValueError('invalid F-02 reconcile CSV')
    games = read_games(archive, {row[0] for row in reference})
    seen = set()
    for game, team, label, metric, event_value, derived_value, catalog_box in reference:
        if metric not in METRICS or label not in ('Away', 'Home'):
            raise ValueError(f'unexpected fixture metric/side: {metric} / {label}')
        key = game, label, metric
        if key in seen:
            raise ValueError(f'duplicate fixture metric: {key}')
        seen.add(key)
        side = 0 if label == 'Away' else 1
        teams, totals = games[game]
        if teams[side] != team:
            raise ValueError(f'team identity mismatch in {game}: {team} vs {teams[side]}')
        if catalog_box != 'NULL' or event_value != derived_value:
            raise ValueError(f'unexpected F-02 lineage in {key}')
        if int(event_value) != totals[side][metric]:
            raise ValueError(f'independent box mismatch in {key}: event={event_value} box={totals[side][metric]}')
    for game in games:
        for label in ('Away', 'Home'):
            if {metric for g, side, metric in seen if g == game and side == label} != METRICS:
                raise ValueError(f'incomplete fixture metrics: {game}/{label}')
    return len(seen)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('box_zip', type=Path)
    parser.add_argument('f02_reconcile_csv', type=Path)
    args = parser.parse_args()
    print(f'validated {compare_box(args.box_zip, args.f02_reconcile_csv)} external box comparisons')
