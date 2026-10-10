"""Validate F-01 CSV invariants without storing source rows (stdlib only)."""

import csv
import sys


REQUIRED = {
    'games_source_rows', 'games_distinct_ids', 'games_with_results',
    'events_rows', 'events_distinct_keys', 'events_pa', 'events_non_pa',
    'events_no_play', 'events_state_before', 'events_state_after',
}


def validate(stream):
    rows = {}
    for fields in csv.reader(stream):
        if len(fields) != 5:
            raise ValueError(f'expected five CSV fields: {fields!r}')
        name, denominator, count, unknown, _note = fields
        if name in rows:
            raise ValueError(f'duplicate metric: {name}')
        try:
            values = tuple(int(v) for v in (denominator, count, unknown))
        except ValueError as exc:
            raise ValueError(f'non-numeric count in {name}') from exc
        d, c, u = values
        if d < 0 or c < 0 or u < 0 or u > d:
            raise ValueError(f'invalid counts in {name}: {values}')
        rows[name] = values
    if missing := REQUIRED - rows.keys():
        raise ValueError(f'missing metrics: {sorted(missing)}')
    for a, b in [('games_source_rows', 'games_distinct_ids'),
                 ('events_rows', 'events_distinct_keys')]:
        if rows[a][1] != rows[b][1] or rows[b][2]:
            raise ValueError(f'{b} differs from {a}')
    if rows['events_pa'][0] != rows['events_non_pa'][0] or (
        rows['events_pa'][1] + rows['events_non_pa'][1]
        != rows['events_rows'][1] - rows['events_no_play'][1]
    ):
        raise ValueError('PA partition inconsistent with no-play events')
    return rows


if __name__ == '__main__':
    with open(sys.argv[1], newline='') as file:
        result = validate(file)
    print(f'validated {len(result)} metrics; {result["games_source_rows"][1]} games; '
          f'{result["events_rows"][1]} events')
