"""Prepare the preregistered offline2019 game-split calibration cohort; never fit here."""
import argparse
from collections import defaultdict
import csv
import hashlib
import json
from pathlib import Path
from audits import player_opportunities as opportunity
from audits import outcome_inventory as outcome
from audits.joint_contact import CATEGORIES, GROUPS
from audits.raw_mechanisms import unique_object

EVENT_SHA = opportunity.INPUT_SHA[0]
REPORT_SHA = '382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e'
HEADER = ('fold', 'game_id', 'batter_id', 'pitcher_id') + CATEGORIES


def fold(game):
    return int.from_bytes(hashlib.sha256(b'player-calibration-v1\0' + game.encode('utf-8')).digest()[:8], 'big') % 10


def collect(rows):
    pairs = defaultdict(lambda: [0] * 9)
    def validated_rows():
        for row in rows:
            yield row  # The original full event validator consumes this first.
            category = GROUPS.get(row['result'])
            if category in CATEGORIES:
                key = (fold(row['game_id']), row['game_id'], row['batter_id'], row['pitcher_id'])
                pairs[key][CATEGORIES.index(category)] += 1
    report = opportunity.inventory(validated_rows())
    return [(*key, counts) for key, counts in sorted(pairs.items())], report


def encode(pairs):
    return '\t'.join(HEADER) + '\n' + ''.join(
        '\t'.join(map(str, (*key, *counts))) + '\n' for *key, counts in pairs)


def prepare(events, source_report, output):
    if output.exists():
        raise ValueError('Refusing existing calibration output')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Calibration output must be outside repository')
    inputs = (events, source_report, Path(__file__))
    pins = tuple(outcome.digest(p) for p in inputs)
    if pins[:2] != (EVENT_SHA, REPORT_SHA):
        raise ValueError('Unsupported calibration source digest')
    original = json.loads(source_report.read_text(), object_pairs_hook=unique_object)
    with events.open(newline='') as stream:
        reader = csv.DictReader(stream, strict=True)
        if reader.fieldnames is None or len(reader.fieldnames) != len(set(reader.fieldnames)) or set(reader.fieldnames) != outcome.FIELDS:
            raise ValueError('Invalid calibration source header')
        pairs, report = collect(reader)
    if report != {k: v for k, v in original.items() if k != 'provenance'}:
        raise ValueError('Calibration source/profile reconciliation failed')
    text = encode(pairs)
    metadata = dict(format='player-calibration-input-v1', input_sha256=list(pins),
        provenance=original['provenance'], dataset_sha256=hashlib.sha256(text.encode()).hexdigest(),
        pairs=len(pairs), folds=[dict(fold=i, games=len({p[1] for p in pairs if p[0] == i}),
            opportunities=sum(sum(p[4]) for p in pairs if p[0] == i)) for i in range(10)],
        counts=report['counts'], roles={role: data['summary'] for role, data in report['roles'].items()})
    if tuple(outcome.digest(p) for p in inputs) != pins:
        raise ValueError('Calibration source changed before publication')
    output.mkdir()
    with (output / 'opportunities.tsv').open('x') as stream:
        stream.write(text)
    with (output / 'source.json').open('x') as stream:
        json.dump(metadata, stream, sort_keys=True, indent=2)
        stream.write('\n')
    return metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('events', 'source_report', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    print(json.dumps(prepare(args.events, args.source_report, args.output), sort_keys=True))


if __name__ == '__main__':
    main()
