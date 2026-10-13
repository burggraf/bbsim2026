"""Audit literal 2019 source participant exposures, not official box credits or ratings."""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
from audits import outcome_inventory as outcome
from audits import raw_mechanisms as raw
from audits.joint_contact import CATEGORIES, GROUPS

INPUT_SHA = ('ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a',
             '443e6a07341e381cd2b171599ac09658348e1462351ed41809fb70b4aaf60f9f',
             '1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca',
             '2422c4acb74702892607852b58b587c0efa59174676e8e25b5a20eaa8419e677')

BANDS = ((0, 0, '0'), (1, 9, '1-9'), (10, 49, '10-49'),
         (50, 199, '50-199'), (200, 499, '200-499'), (500, None, '500+'))


def inventory(rows):
    counts = Counter(dict.fromkeys(CATEGORIES, 0))
    exclusions = Counter(dict(excluded_ibb=0, excluded_interference=0))
    roles = {'batter': {}, 'pitcher': {}}

    def validated_rows():
        for row in rows:
            # Aggregate only after the complete existing validator consumes this row.
            yield row
            result = row['result']
            category = GROUPS.get(result) if result else None
            if result and category is None:
                raise ValueError('Unknown PA classification')
            if category in exclusions:
                exclusions[category] += 1
            elif category is not None:
                counts[category] += 1
            for role, profiles in roles.items():
                player = row[role + '_id']
                line = profiles.setdefault(player, dict(player_id=player, events=0, games=set(),
                    selected_pa=0, non_pa=0, counts=dict.fromkeys(CATEGORIES, 0),
                    included_pa=0, exclusions=dict(excluded_ibb=0, excluded_interference=0)))
                line['events'] += 1
                line['games'].add(row['game_id'])
                line['selected_pa'] += int(bool(result))
                line['non_pa'] += int(not result)
                if category in exclusions:
                    line['exclusions'][category] += 1
                elif category is not None:
                    line['counts'][category] += 1
                    line['included_pa'] += 1

    source = outcome.inventory(validated_rows())
    for role, profiles in roles.items():
        lines = []
        for player in sorted(profiles):
            line = profiles[player]
            line['games'] = len(line['games'])
            line['zero_categories'] = sum(n == 0 for n in line['counts'].values())
            if (line['events'] != line['selected_pa'] + line['non_pa']
                    or line['selected_pa'] != line['included_pa'] + sum(line['exclusions'].values())
                    or line['included_pa'] != sum(line['counts'].values())):
                raise ValueError('Player opportunity conservation failed')
            lines.append(line)
        for key, expected in [('events', source['totals']['events']),
                              ('selected_pa', source['totals']['pa']),
                              ('non_pa', source['totals']['non_pa']),
                              ('included_pa', sum(counts.values()))]:
            if sum(line[key] for line in lines) != expected:
                raise ValueError('Role opportunity conservation failed')
        for group, expected in [('counts', counts), ('exclusions', exclusions)]:
            if any(sum(line[group][key] for line in lines) != n for key, n in expected.items()):
                raise ValueError('Role category conservation failed')
        bands = [dict(band=name, players=0, included_pa=0) for _, _, name in BANDS]
        for line in lines:
            exposure = line['included_pa']
            index = next(i for i, (low, high, _) in enumerate(BANDS)
                         if exposure >= low and (high is None or exposure <= high))
            bands[index]['players'] += 1
            bands[index]['included_pa'] += exposure
        if (sum(band['players'] for band in bands) != len(lines)
                or sum(band['included_pa'] for band in bands) != sum(counts.values())):
            raise ValueError('Exposure band conservation failed')
        roles[role] = dict(profiles=lines, summary=dict(players=len(lines),
            zero_exposure_players=sum(line['included_pa'] == 0 for line in lines),
            zero_categories={key: sum(line['counts'][key] == 0 for line in lines) for key in CATEGORIES},
            exposure_bands=bands))
    return dict(format='2019-player-opportunity-support-v1', counts=dict(counts),
                exclusions=dict(exclusions), roles=roles, games=source['games'],
                outcome_totals=source['totals'])


def _write_report(report, inputs, before, output):
    """Post-validation writer: reread all hashes before exclusive publication."""
    text = json.dumps(report, sort_keys=True, indent=2)+'\n'
    if tuple(outcome.digest(path) for path in inputs) != before:
        raise ValueError('Player opportunity inputs changed during read')
    with output.open('x') as stream:
        stream.write(text)


def publish(events, manifest, certificate, output):
    if output.exists():
        raise ValueError('Refusing to overwrite an existing file')
    query = Path(outcome.__file__).with_name('2019-outcomes.sql')
    inputs = (events, manifest, certificate, query, Path(__file__))
    before = tuple(outcome.digest(path) for path in inputs)
    if before[:4] != INPUT_SHA:
        raise ValueError('Unsupported input digest for accepted 2019 player cohort')
    snapshot = outcome.read_manifest(manifest)
    accepted = json.loads(certificate.read_text(), object_pairs_hook=raw.unique_object)
    with events.open(newline='') as stream:
        reader = csv.DictReader(stream, strict=True)
        if (reader.fieldnames is None or len(reader.fieldnames) != len(set(reader.fieldnames))
                or set(reader.fieldnames) != outcome.FIELDS):
            raise ValueError('Invalid outcome export header')
        report = inventory(reader)
    provenance = accepted.get('provenance', {})
    if (accepted.get('format') != '2019-joint-contact-proxies-v1'
            or any(report[key] != accepted.get(key) for key in
                   ('counts', 'exclusions', 'outcome_totals', 'games'))
            or snapshot != provenance.get('snapshot')
            or provenance.get('catalog_sha256') != outcome.CATALOG
            or provenance.get('source_commit') != raw.SOURCE_COMMIT
            or provenance.get('snapshot_sha256') != raw.SNAPSHOT_SHA
            or provenance.get('outcome_events_sha256') != before[0]
            or provenance.get('outcome_manifest_sha256') != before[1]
            or provenance.get('outcome_query_sha256') != before[3]):
        raise ValueError('Original 2019 player cohort/provenance conservation failed')
    report['provenance'] = dict(snapshot=snapshot, catalog_sha256=outcome.CATALOG,
        source_commit=raw.SOURCE_COMMIT, snapshot_sha256=raw.SNAPSHOT_SHA,
        outcome_events_sha256=before[0], outcome_manifest_sha256=before[1],
        certificate_sha256=before[2], outcome_query_sha256=before[3], recipe_sha256=before[4],
        attribution=snapshot['attribution'],
        opportunity_basis='literal terminal-event participants; not official credits or PA/BF')
    _write_report(report, inputs, before, output)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('events', 'manifest', 'certificate', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    report = publish(args.events, args.manifest, args.certificate, args.output)
    print(json.dumps(dict(counts=report['counts'], exclusions=report['exclusions'],
        roles={role: data['summary'] for role, data in report['roles'].items()}), sort_keys=True))


if __name__ == '__main__':
    main()
