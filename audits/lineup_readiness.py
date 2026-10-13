"""Audit literal 2019 starting identities and observed membership, not ratings/rosters."""
import argparse
from collections import Counter, defaultdict
import csv
from datetime import date
import io
import json
from pathlib import Path
import subprocess
from audits.outcome_inventory import CATALOG, digest
from audits.raw_mechanisms import SOURCE_COMMIT, unique_object


SIDES = ('Away', 'Home')
MEMBERSHIP_BASIS = 'observed appearances only; not active-roster intervals or opening-bench eligibility'
INPUT_SHA = ('d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528',
             '554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82')


def nonempty(value):
    return isinstance(value, str) and bool(value.strip())


def known_date(value):
    try:
        return isinstance(value, str) and date.fromisoformat(value).year == 2019
    except ValueError:
        return False


def read_export(text):
    groups = dict(game=[], person=[], appearance=[], manifest=[])
    reader = csv.DictReader(io.StringIO(text), strict=True)
    if reader.fieldnames is None or len(reader.fieldnames) != 2 or set(reader.fieldnames) != {'record_type', 'payload'}:
        raise ValueError('Invalid lineup export header')
    for row in reader:
        if set(row) != {'record_type', 'payload'} or row['record_type'] not in groups or not row['payload']:
            raise ValueError('Invalid lineup export record')
        value = json.loads(row['payload'], object_pairs_hook=unique_object)
        if not isinstance(value, dict):
            raise ValueError('Expected a literal source object')
        groups[row['record_type']].append(value)
    return groups


def inventory(games, people, appearances):
    by_game, bios = {}, defaultdict(list)
    for g in games:
        if g['game_id'] in by_game:
            raise ValueError('Duplicate game grain')
        if (g['season'] != 2019 or g['is_regular_season'] is not True
                or g['home_league'] not in ('AL', 'NL') or g['away_league'] not in ('AL', 'NL')):
            raise ValueError('Invalid 2019 cohort')
        by_game[g['game_id']] = g
    for p in people:
        bios[p['player_id']].append(p)

    def person(player):
        rows = bios.get(player, [])
        return rows[0] if len(rows) == 1 and nonempty(rows[0]['first_name']) and nonempty(rows[0]['last_name']) else None

    coverage = Counter(games=len(games), sides=2*len(games), people=len(people), appearances=len(appearances),
        candidate_games=0, missing_start_lookups=0, unknown_hand_slots=0, start_identity_slots=0,
        start_appearance_slots=0, corroborated_start_slots=0, uncorroborated_start_slots=0,
        unknown_position_appearances=0, unknown_side_appearances=0, unknown_date_appearances=0,
        missing_appearance_lookups=0)
    members, appearance_grains, corroboration, reversed_ranges = {}, set(), set(), []
    position_counts = Counter()
    for row in appearances:
        game = by_game.get(row['game_id'])
        if game is None:
            raise ValueError('Orphan appearance')
        key = (row['game_id'], row['player_id'], row['fielding_position'], row['start_event_id'])
        if key in appearance_grains:
            raise ValueError('Duplicate appearance grain')
        appearance_grains.add(key)
        if not nonempty(row['player_id']):
            raise ValueError('Invalid appearance player identity')
        position = row['fielding_position']
        if position is not None and (type(position) is not int or not 0 <= position <= 255):
            raise ValueError('Invalid supplied appearance position')
        position_counts[str(position)] += 1
        coverage['unknown_position_appearances'] += int(position is None or position == 0 or position > 12)
        coverage['missing_appearance_lookups'] += int(person(row['player_id']) is None)
        start, end = row['start_event_id'], row['end_event_id']
        if any(value is not None and (type(value) is not int or not 0 <= value <= 255) for value in (start, end)):
            raise ValueError('Invalid supplied appearance range')
        if start is not None and end is not None and start > end:
            reversed_ranges.append(dict(row))
        if row['side'] not in SIDES:
            coverage['unknown_side_appearances'] += 1
            continue
        team = game[row['side'].lower()+'_team_id']
        if not nonempty(team):
            coverage['unknown_side_appearances'] += 1
            continue
        corroboration.add((row['game_id'], row['side'], position, row['player_id']))
        profile = members.setdefault((team, row['player_id']), dict(games=set(), dates=set(), positions=set()))
        profile['games'].add(row['game_id'])
        if known_date(game['date']):
            profile['dates'].add(game['date'])
        else:
            coverage['unknown_date_appearances'] += 1
        if position is not None:
            profile['positions'].add(position)

    checks, candidates, side_counts, game_counts = [], [], Counter(), Counter()
    for g in sorted(games, key=lambda g: (g['date'] or '', g['game_id'] or '')):
        issues = []
        if (not nonempty(g['game_id']) or not known_date(g['date']) or type(g['use_dh']) is not bool
                or g['bat_first_side'] != 'Away'
                or not all(nonempty(g[side.lower()+'_team_id']) and nonempty(g[side.lower()+'_team_name']) for side in SIDES)
                or g['home_team_id'] == g['away_team_id']):
            issues.append('game_metadata')
        results = g['results']
        if (not isinstance(results, list) or len(results) != 1
                or any(results[0][side.lower()+'_team_id'] != g[side.lower()+'_team_id'] for side in SIDES)
                or any(results[0][flag] is not False for flag in ('forfeit_flag', 'suspension_flag', 'tie_flag'))):
            issues.append('result_metadata')
        side_checks, starting_sets = [], []
        for side in SIDES:
            prefix = side.lower()
            lineup, field = g['lineup_map_'+prefix], g['fielding_map_'+prefix]
            pitcher = g[prefix+'_starting_pitcher_id']
            has_rule = type(g['use_dh']) is bool
            batting_keys = {str(i) for i in range(1, 10)}
            line_keys = batting_keys | ({'0'} if g['use_dh'] is True else set())
            field_keys = {str(i) for i in range(1, 11 if g['use_dh'] is True else 10)}
            faults = []
            if not isinstance(lineup, dict) or not has_rule or set(lineup) != line_keys:
                faults.append('lineup_keys')
            if not isinstance(field, dict) or not has_rule or set(field) != field_keys:
                faults.append('fielding_keys')
            line_values = list(lineup.values()) if isinstance(lineup, dict) else []
            field_values = list(field.values()) if isinstance(field, dict) else []
            for name, values in [('lineup', line_values), ('fielding', field_values)]:
                if not values or not all(nonempty(value) for value in values) or len(set(values)) != len(values):
                    faults.append(name+'_ids')
            if not nonempty(pitcher) or not isinstance(field, dict) or field.get('1') != pitcher:
                faults.append('pitcher_fielding')
            batting = [lineup.get(str(i)) for i in range(1, 10)] if isinstance(lineup, dict) else []
            if has_rule and isinstance(lineup, dict) and isinstance(field, dict):
                if g['use_dh']:
                    if lineup.get('0') != pitcher:
                        faults.append('pitcher_lineup')
                    expected = [field.get(str(i)) for i in range(2, 11)]
                    if (pitcher in batting or set(batting) != set(expected)
                            or not nonempty(field.get('10')) or batting.count(field.get('10')) != 1):
                        faults.append('role_sets')
                elif batting.count(pitcher) != 1 or set(batting) != set(field_values):
                    faults.append('role_sets')
            starting = set(value for value in line_values+field_values+[pitcher] if nonempty(value))
            starting_sets.append(starting)
            coverage['start_identity_slots'] += len(starting)
            missing = sum(person(player) is None for player in starting)
            coverage['missing_start_lookups'] += missing
            if missing:
                faults.append('person_lookup')
            for player in starting:
                p = person(player)
                for hand in ('bats', 'throws'):
                    coverage['unknown_hand_slots'] += int(p is None or not nonempty(p[hand]) or p[hand] == 'Unknown')
            if isinstance(field, dict):
                for position, player in field.items():
                    coverage['start_appearance_slots'] += 1
                    matched = position.isdecimal() and (g['game_id'], side, int(position), player) in corroboration
                    coverage['corroborated_start_slots'] += int(matched)
                    coverage['uncorroborated_start_slots'] += int(not matched)
            faults = sorted(set(faults))
            side_counts.update(faults)
            side_checks.append(dict(side=side, team_id=g[prefix+'_team_id'], issues=faults))
        if starting_sets[0] & starting_sets[1]:
            issues.append('cross_team_ids')
        issues.sort()
        game_counts.update(issues)
        checks.append(dict(game_id=g['game_id'], date=g['date'], use_dh=g['use_dh'], issues=issues, sides=side_checks))
        if not issues and all(not side['issues'] for side in side_checks):
            candidates.append(g)
    coverage['candidate_games'] = len(candidates)
    coverage['side_issue_counts'] = dict(sorted(side_counts.items()))
    coverage['game_issue_counts'] = dict(sorted(game_counts.items()))
    coverage['position_appearance_counts'] = dict(sorted(position_counts.items()))

    def witness(g):
        if g is None:
            return None
        teams = []
        for side in SIDES:
            prefix = side.lower()
            def named(player):
                p = person(player)
                return dict(player_id=player, name=p['first_name']+' '+p['last_name'], bats=p['bats'], throws=p['throws'])
            order = [dict(slot=i, **named(g['lineup_map_'+prefix][str(i)])) for i in range(1, 10)]
            fielding = [dict(position=int(i), **named(player)) for i, player in
                        sorted(g['fielding_map_'+prefix].items(), key=lambda pair: int(pair[0]))]
            dh = next(p['slot'] for p in order if p['player_id'] == g['fielding_map_'+prefix]['10']) if g['use_dh'] else None
            teams.append(dict(side=side, team_id=g[prefix+'_team_id'], team_name=g[prefix+'_team_name'],
                pitcher_id=g[prefix+'_starting_pitcher_id'], dh_slot=dh, engine_dh_slot=dh-1 if dh is not None else None,
                batting_order=order, fielding=fielding))
        return dict(game_id=g['game_id'], date=g['date'], park_id=g['park_id'], use_dh=g['use_dh'], teams=teams)

    profiles = [dict(team_id=team, player_id=player, games=len(p['games']),
        first_date=min(p['dates']) if p['dates'] else None, last_date=max(p['dates']) if p['dates'] else None,
        positions=sorted(p['positions'])) for (team, player), p in sorted(members.items())]
    team_counts = Counter(p['player_id'] for p in profiles)
    return dict(format='2019-lineup-readiness-v1', coverage=dict(coverage), checks=checks,
        candidate_game_ids=[g['game_id'] for g in candidates],
        witnesses=dict(primary=witness(next(iter(candidates), None)),
            dh=witness(next((g for g in candidates if g['use_dh']), None)),
            no_dh=witness(next((g for g in candidates if not g['use_dh']), None))),
        membership=dict(basis=MEMBERSHIP_BASIS, profiles=profiles, pairs=len(profiles),
            players=len(team_counts), multi_team_players=sum(n>1 for n in team_counts.values()),
            reversed_ranges=sorted(reversed_ranges, key=lambda r: (r['game_id'], r['player_id'], r['start_event_id']))))


def _write_report(report, inputs, before, output):
    text = json.dumps(report, sort_keys=True, indent=2)+'\n'
    if tuple(digest(path) for path in inputs) != before:
        raise ValueError('Lineup readiness inputs changed during read')
    with output.open('x') as stream:
        stream.write(text)


def publish(snapshot, supplement, output):
    raw = output.with_suffix('.native.csv')
    errors = output.with_suffix('.native.stderr')
    if any(path.exists() for path in (output, raw, errors)):
        raise ValueError('Refusing to overwrite an existing report or native evidence')
    if output.resolve().is_relative_to(Path(__file__).resolve().parent.parent):
        raise ValueError('Readiness artifacts must be outside the repository')
    query = Path(__file__).with_name('2019-lineup-readiness.sql')
    inputs = (snapshot, supplement, query, Path(__file__))
    before = tuple(digest(path) for path in inputs)
    if before[:2] != INPUT_SHA:
        raise ValueError('Unsupported accepted snapshot/supplement digest')
    # Paths are local CLI inputs, never HTTP/browser parameters; quote literal SQL paths.
    detail = str(supplement.resolve()).replace("'", "''")
    setup = f"ATTACH '{detail}' AS detail (READ_ONLY); SET enable_external_access=false;"
    with raw.open('x') as out, errors.open('x') as err:
        result = subprocess.run(['duckdb', '-init', '/dev/null', '-readonly', '-bail', '-batch', '-csv',
            '-cmd', setup, str(snapshot.resolve()), '-f', str(query.resolve())],
            stdout=out, stderr=err, check=False, timeout=300)
    if result.returncode:
        raise RuntimeError(f'Native lineup query failed ({result.returncode}); see {errors}')
    exported = read_export(raw.read_text())
    manifests = exported['manifest']
    if (len(manifests) != 1 or manifests[0].get('catalog_sha256') != CATALOG
            or manifests[0].get('schema_version') != 'f03-spike-1'
            or manifests[0].get('query_version') != '2019-filtered-extract-v1'
            or not all(nonempty(manifests[0].get(key)) for key in ('source_url', 'extracted_at', 'attribution'))):
        raise ValueError('Missing or unsupported snapshot provenance')
    report = inventory(exported['game'], exported['person'], exported['appearance'])
    report['provenance'] = dict(snapshot=manifests[0], source_commit=SOURCE_COMMIT,
        snapshot_sha256=before[0], supplement_sha256=before[1], query_sha256=before[2],
        recipe_sha256=before[3], native_export_sha256=digest(raw), attribution=manifests[0]['attribution'])
    _write_report(report, inputs, before, output)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot', 'supplement', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    report = publish(args.snapshot, args.supplement, args.output)
    print(json.dumps(dict(coverage=report['coverage'], membership_pairs=report['membership']['pairs'],
        witnesses={key: value['game_id'] if value else None for key, value in report['witnesses'].items()}), sort_keys=True))


if __name__ == '__main__':
    main()
