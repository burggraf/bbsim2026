"""Check bounded F-02 event/summary/box aggregate CSV (stdlib only)."""

import csv
import sys

METRICS = ('pa', 'runs', 'outs', 'hits', 'steals', 'caught_stealing',
           'putouts', 'assists', 'errors')


def validate(stream, game_ids):
    rows = {}
    teams = {}
    for fields in csv.reader(stream):
        if len(fields) != 7:
            raise ValueError(f'expected seven fields: {fields!r}')
        game, team, side, metric, event, summary, box = fields
        if game not in game_ids or side not in ('Home', 'Away') or metric not in METRICS:
            raise ValueError(f'unexpected fixture row: {fields!r}')
        key = (game, side, metric)
        if key in rows:
            raise ValueError(f'duplicate fixture row: {key}')
        prior = teams.setdefault((game, side), team)
        if not team or prior != team:
            raise ValueError(f'inconsistent team ID: {key}')
        def integer(value, label, nullable=False):
            if value in ('', 'NULL'):
                if nullable:
                    return None
                raise ValueError(f'missing {label}: {key}')
            try:
                result = int(value)
            except ValueError as exc:
                raise ValueError(f'invalid {label}: {key}') from exc
            if result < 0:
                raise ValueError(f'negative {label}: {key}')
            return result
        e = integer(event, 'event total')
        s = integer(summary, 'summary')
        b = integer(box, 'box', nullable=True)
        if e != s:
            raise ValueError(f'summary mismatch: {key} event={e} summary={s}')
        if b is not None and e != b:
            raise ValueError(f'box mismatch: {key} event={e} box={b}')
        rows[key] = (team, e, s, b)
    expected = {(g, side, m) for g in game_ids for side in ('Home', 'Away') for m in METRICS}
    if rows.keys() != expected:
        raise ValueError(f'missing rows: {sorted(expected - rows.keys())}; unexpected: {sorted(rows.keys() - expected)}')
    return rows


if __name__ == '__main__':
    with open(sys.argv[1], newline='') as source:
        result = validate(source, {'ANA201904040', 'ANA201904050', 'ANA201904070'})
    print(f'validated {len(result)} fixture metric comparisons; '
          f'{sum(v[3] is not None for v in result.values())} independent box cells present')
