"""Validate bounded F-02 transition rows and their independent state invariants."""

import csv
import sys


def validate(stream, expected):
    seen = set()
    for row in csv.reader(stream):
        if len(row) != 38:
            raise ValueError(f'expected 38 columns, got {len(row)}')
        game, event_id = row[0], int(row[1])
        key = (game, event_id)
        if key not in expected or key in seen:
            raise ValueError(f'unexpected or duplicate fixture event: {key}')
        seen.add(key)
        if row[3] != 'false':
            raise ValueError(f'no-play fixture event: {key}')
        def number(i):
            return None if row[i] in ('NULL', '') else int(row[i])
        if row[4] == 'NULL':
            if any(number(i) is not None for i in (30,31,32)):
                raise ValueError(f'non-PA credited batting statistics: {key}')
        elif number(30) != 1:
            raise ValueError(f'PA without exactly one PA credit: {key}')
        if number(11) + number(16) != number(20):
            raise ValueError(f'outs mismatch: {key}')
        if number(27) - number(25) + number(28) - number(26) != number(17):
            raise ValueError(f'score delta mismatch: {key}')
        for state, slots in ((12,(13,14,15)),(21,(22,23,24))):
            mask = number(state)
            if mask is None:
                if row[29] != 'true':
                    raise ValueError(f'null non-boundary base state: {key}')
                continue
            for bit, slot in zip((1,2,4),slots):
                if bool(mask & bit) != (row[slot] not in ('NULL','')):
                    raise ValueError(f'base identities mismatch: {key}')
    if seen != expected:
        raise ValueError(f'missing fixture events: {sorted(expected - seen)}')
    return seen


if __name__ == '__main__':
    expected = {('ANA201904040',36),('ANA201904040',96),('ANA201904040',99),
                ('ANA201904050',4),('ANA201904070',3)}
    with open(sys.argv[1], newline='') as source:
        found = validate(source, expected)
    print(f'validated {len(found)} fixture transitions and state invariants')
