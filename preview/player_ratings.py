"""Pinned terminal-event participant counts for a shrunk, hindsight matchup pilot."""
import argparse
import json
import os
from pathlib import Path
import tempfile
from audits.raw_mechanisms import unique_object
from preview import real_lineup as real

MODEL = 'dirichlet-log5-player-proxy-preview-v1'
REPORT_SHA = '382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e'
BASIS = 'literal terminal-event participants; not official credits or PA/BF'
KEYS = ('strikeout', 'walk', 'hbp', 'single', 'double', 'triple', 'home_run', 'roe', 'contact_out')
COUNTS = (42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245)
KAPPA = 200


def encode(report, witness):
    real.encode_witness(witness)  # Preserve the accepted starting-identity contract.
    if (report['format'] != '2019-player-opportunity-support-v1'
            or report['counts'] != dict(zip(KEYS, COUNTS))
            or report['provenance']['snapshot_sha256'] != real.SNAPSHOT_SHA
            or report['provenance']['attribution'] != real.ATTRIBUTION
            or report['provenance']['opportunity_basis'] != BASIS):
        raise ValueError('Unsupported player cohort/provenance')
    fields = dict(format='player-proxy-counts-v1', model=MODEL,
        opportunity_report_sha256=REPORT_SHA, lineup_report_sha256=real.REPORT_SHA,
        snapshot_sha256=real.SNAPSHOT_SHA, supplement_sha256=real.SUPPLEMENT_SHA,
        batter_kappa=str(KAPPA), pitcher_kappa=str(KAPPA),
        league_counts=','.join(map(str, COUNTS)), attribution=real.ATTRIBUTION,
        opportunity_basis=BASIS)
    for role in ('batter', 'pitcher'):
        profiles = {}
        for p in report['roles'][role]['profiles']:
            player = real.literal(p['player_id'])
            if player in profiles:
                raise ValueError('Duplicate player role profile')
            profiles[player] = p
        selected = (p['player_id'] for t in witness['teams'] for p in t['batting_order']) if role == 'batter' else (t['pitcher_id'] for t in witness['teams'])
        for player in sorted(selected):
            if player not in profiles:
                raise ValueError('Missing starting player role profile')
            p = profiles[player]
            counts = p['counts']
            if (set(counts) != set(KEYS) or type(p['included_pa']) is not int
                    or any(type(counts[k]) is not int or not 0 <= counts[k] <= maximum
                           for k, maximum in zip(KEYS, COUNTS))
                    or sum(counts.values()) != p['included_pa']):
                raise ValueError('Invalid player role count conservation')
            fields[f'{role}:{player}'] = ','.join(str(counts[k]) for k in KEYS)
    return 'key\tvalue\n' + ''.join(real.literal(k)+'\t'+real.literal(v)+'\n' for k,v in fields.items())


def publish(text, inputs, pins, output):
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', newline='', dir=output.parent, delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(text)
            stream.flush()
            os.fsync(stream.fileno())
        if tuple(real.digest(p) for p in inputs) != pins:
            raise ValueError('Rating inputs changed before publication')
        os.link(temporary, output)
    finally:
        if temporary is not None:
            temporary.unlink()


def export(report, lineup, output):
    if output.exists():
        raise ValueError('Refusing to overwrite rating output')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Rating output must be outside repository')
    inputs = (report, lineup)
    pins = tuple(real.digest(p) for p in inputs)
    if pins != (REPORT_SHA, real.REPORT_SHA):
        raise ValueError('Unsupported rating input digest')
    data, lineups = (json.loads(p.read_text(), object_pairs_hook=unique_object) for p in inputs)
    if (lineups['format'] != '2019-lineup-readiness-v1'
            or lineups['provenance']['snapshot_sha256'] != real.SNAPSHOT_SHA
            or lineups['provenance']['supplement_sha256'] != real.SUPPLEMENT_SHA):
        raise ValueError('Unsupported rating lineup provenance')
    text = encode(data, lineups['witnesses']['primary'])
    publish(text, inputs, pins, output)
    return text


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('report', 'lineup', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    text = export(args.report, args.lineup, args.output)
    print(f'Exported20 player role vectors: {len(text.encode())} bytes; SHA256 {real.digest(args.output)}')


if __name__ == '__main__':
    main()
