"""Compare source-published pitcher-game ER totals with separate Retrosheet boxes.

External artifacts only; equality is shared-upstream reconciliation, not proof of
engine errorless-inning reconstruction. Version 3 pline format:
https://www.retrosheet.org/boxfile.txt
"""

import argparse
import csv
from pathlib import Path

try:
    from .verify_retrosheet_box import read_games
except ImportError:  # Direct CLI execution, matching the other audit scripts.
    from verify_retrosheet_box import read_games


FIXTURE_GAMES = frozenset({'ANA201904040', 'ANA201904050', 'ANA201904070'})


def nonnegative(value):
    if not value.isascii() or not value.isdecimal():
        raise ValueError(f'unknown or invalid earned-run number: {value!r}')
    return int(value)


def pitcher_er(rows):
    versions = [row for row in rows if row and row[0] == 'version']
    if versions != [['version', '3']]:
        raise ValueError('missing or unsupported box version')
    result = {}
    sides = set()
    for row in rows:
        if row[:2] != ['stat', 'pline']:
            continue
        if len(row) != 22 or not row[2] or row[3] not in ('0', '1'):
            raise ValueError('invalid box pitcher line')
        player = row[2]
        if player in result:
            raise ValueError(f'duplicate box pitcher: {player}')
        runs, earned = nonnegative(row[12]), nonnegative(row[13])
        if earned > runs:
            raise ValueError(f'box earned runs exceed charged runs: {player}')
        result[player] = earned
        sides.add(row[3])
    if sides != {'0', '1'}:
        raise ValueError('incomplete box pitching-side coverage')
    return result


def compare_pitcher_er(archive, source_csv, wanted=FIXTURE_GAMES):
    if not wanted:
        raise ValueError('empty requested game coverage')
    boxes = read_games(archive, wanted, parser=pitcher_er)
    expected = {(game, player): er for game, lines in boxes.items() for player, er in lines.items()}
    source = {}
    with Path(source_csv).open(newline='') as f:
        reader = csv.DictReader(f, strict=True)
        try:
            if reader.fieldnames != ['game_id', 'player_id', 'earned_runs']:
                raise ValueError('invalid source ER CSV header')
            for row in reader:
                if None in row or any(value is None for value in row.values()):
                    raise ValueError('invalid source ER CSV row')
                game, player = row['game_id'], row['player_id']
                if game not in wanted or not player:
                    raise ValueError('unexpected source game or empty player identity')
                key = game, player
                if key in source:
                    raise ValueError(f'duplicate source pitcher/game key: {key}')
                source[key] = nonnegative(row['earned_runs'])
        except csv.Error as error:
            raise ValueError(f'invalid source ER CSV syntax: {error}') from error
    if not source or source.keys() != expected.keys() or {game for game, _ in source} != set(wanted):
        raise ValueError(f'pitcher ER coverage mismatch: missing={sorted(expected.keys() - source.keys())}, extra={sorted(source.keys() - expected.keys())}')
    for key, earned in source.items():
        if earned != expected[key]:
            raise ValueError(f'pitcher ER mismatch in {key}: source={earned} box={expected[key]}')
    return len(source)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('box_zip', type=Path)
    parser.add_argument('source_er_csv', type=Path)
    args = parser.parse_args()
    print(f'validated {compare_pitcher_er(args.box_zip, args.source_er_csv)} pitcher-game ER cells (shared upstream; not engine reconstruction)')
