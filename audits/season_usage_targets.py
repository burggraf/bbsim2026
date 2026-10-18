"""Offline credited historical PA/current-pitcher outs; not rosters or eligibility."""
import argparse
from collections import Counter, defaultdict
from datetime import date
import hashlib
import json
from pathlib import Path
import subprocess

FORMAT = '2019-season-usage-targets-v1'
SNAPSHOT_SHA256 = 'd036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528'
SQL_SHA256 = 'd975ff6764047019965de4e568d8b99442f7f81054eed6b773489f0e63603623'
CATALOG_SHA256 = '2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583'
SOURCE_COMMIT = 'b449f7bec49f852c27803e096303a4c78ac661b9'
WARNINGS = [
    'Source PA exceeds AB+BB+HBP+SH+SF+CI by 62 missing derived sacrifice credits; no repair or official-stat certification.',
    '16 mid-PA batter participant/responsibility switches: targets use batting.batter_id credit; simulated current-batter accounting differs.',
    'Outs use the current event pitcher, never the responsibility pitcher; 936 outs occur on non-PA events.',
    'Suspended-game events retain the original game date, not the resumption date (including Diekman KCA/OAK on 2019-05-19); not literal recent-rest history.',
    'Observed usage is not verified dated rosters, eligibility, injury availability or membership stints; no stints are inferred.',
    'Only batting/live-event observations are usage records; absence is not zero. Six supplement-only position-1 appearances without live events are not represented.',
    'Duration/team putout reconciliation shares source lineage and is not independent official-IP certification. Integer outs, not decimal IP.',
]


def _text(value):
    return isinstance(value, str) and bool(value.strip())


def _count(value, maximum=None):
    if type(value) is not int or value < 0 or (maximum is not None and value > maximum):
        raise ValueError('Invalid nonnegative integer count')
    return value


def _unique(rows, key):
    result = {}
    for row in rows:
        identity = key(row)
        if identity in result:
            raise ValueError('Duplicate source grain')
        result[identity] = row
    return result


def inventory(games, people, batting, events, team_results):
    """Validate ordinary source rows; preserve unknown, observed zero and absent usage."""
    people = _unique(people, lambda p: p['player_id'])
    if any(not _text(p) for p in people):
        raise ValueError('Invalid player identity')
    games = _unique(games, lambda g: g['game_id'])
    clean_games = []
    team_games = Counter()
    fields = ('game_id', 'date', 'away_team_id', 'home_team_id', 'starting_pitcher_ids',
              'suspended', 'shortened', 'duration_outs')
    for gid, g in games.items():
        if not _text(gid) or not isinstance(g['date'], str):
            raise ValueError('Invalid game identity/date')
        try:
            parsed = date.fromisoformat(g['date'])
        except ValueError as exc:
            raise ValueError('Invalid game date') from exc
        if parsed.year != 2019 or parsed.isoformat() != g['date']:
            raise ValueError('Expected literal 2019 date')
        teams = (g['away_team_id'], g['home_team_id'])
        if any(not _text(t) for t in teams) or teams[0] == teams[1]:
            raise ValueError('Invalid game teams')
        starters = g['starting_pitcher_ids']
        if (not isinstance(starters, dict) or set(starters) != set(teams)
                or any(p not in people for p in starters.values())
                or len(set(starters.values())) != 2):
            raise ValueError('Invalid starting pitchers')
        if any(type(g[k]) is not bool for k in ('suspended', 'shortened')):
            raise ValueError('Unknown game flags')
        _count(g['duration_outs'])
        clean_games.append({k: dict(g[k]) if k == 'starting_pitcher_ids' else g[k] for k in fields})
        team_games.update(teams)

    def team(gid, tid):
        if gid not in games or tid not in (games[gid]['away_team_id'], games[gid]['home_team_id']):
            raise ValueError('Orphan game or invalid team')

    cells = defaultdict(list)
    team_outs = defaultdict(list)
    events = _unique(events, lambda e: _count(e['event_key']))
    live = {}
    for key, e in events.items():
        gid = e['game_id']; tid = e['fielding_team_id']
        team(gid, tid); team(gid, e['batting_team_id'])
        if tid == e['batting_team_id'] or type(e['no_play_flag']) is not bool:
            raise ValueError('Invalid event team/flag')
        if e['no_play_flag']:
            continue
        if e['pitcher_id'] not in people or e['batter_id'] not in people:
            raise ValueError('Unknown live-event participant')
        if e['result'] is not None and not _text(e['result']):
            raise ValueError('Invalid terminal event result')
        outs = e['outs_on_play']
        if outs is not None:
            _count(outs, 3)
        live[key] = e
        cells[gid, tid, e['pitcher_id'], 'pitcher'].append(outs)
        team_outs[gid, tid].append(outs)
    batting = _unique(batting, lambda b: _count(b['event_key']))
    for key, b in batting.items():
        gid = b['game_id']; tid = b['batting_team_id']
        team(gid, tid)
        e = live.get(key)
        if (e is None or e['result'] is None or e['game_id'] != gid
                or e['batting_team_id'] != tid or b['batter_id'] not in people):
            raise ValueError('Invalid batting/live-event binding')
        pa = b['plate_appearances']
        if pa is not None:
            _count(pa, 1)
        cells[gid, tid, b['batter_id'], 'batter'].append(pa)
    if set(batting) != {k for k, e in live.items() if e['result'] is not None}:
        raise ValueError('Missing batting credit on terminal event')
    teams = _unique(team_results, lambda t: (t['game_id'], t['team_id']))
    expected = {(gid, t) for gid, g in games.items() for t in (g['away_team_id'], g['home_team_id'])}
    if set(teams) != expected or set(team_outs) != expected:
        raise ValueError('Missing/orphan team result or live-event side')
    for key, t in teams.items():
        _count(t['putouts'])
        if None not in team_outs[key] and sum(team_outs[key]) != t['putouts']:
            raise ValueError('Team putout reconciliation failed')
    for gid, g in games.items():
        outs = team_outs[gid, g['away_team_id']] + team_outs[gid, g['home_team_id']]
        if None not in outs and sum(outs) != g['duration_outs']:
            raise ValueError('Game duration reconciliation failed')
    records = []
    for (gid, tid, pid, role), values in sorted(cells.items()):
        unknown = values.count(None)
        records.append(dict(game_id=gid, team_id=tid, player_id=pid, role=role,
                            usage=None if unknown else sum(values), unknown_rows=unknown,
                            source_rows=len(values)))
    totals = {}
    for role, metric in (('batter', 'pa'), ('pitcher', 'outs')):
        values = [r['usage'] for r in records if r['role'] == role]
        totals[metric] = None if None in values else sum(values)
    return dict(format=FORMAT, games=sorted(clean_games, key=lambda g: (g['date'], g['game_id'])),
                records=records, team_games=dict(sorted(team_games.items())), totals=totals)


def _digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def _write(path, value):
    with Path(path).open('x', encoding='utf-8') as f:
        json.dump(value, f, indent=2, sort_keys=True, allow_nan=False)
        f.write('\n')


def publish(snapshot, output):
    """Create an exclusive evidence directory; receipt.json is the completion marker."""
    snapshot = Path(snapshot).resolve(); output = Path(output).resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]) or output.exists():
        raise ValueError('Output must be new and outside repository')
    query = Path(__file__).with_name('2019-season-usage-targets.sql').resolve()
    inputs = [snapshot, query, Path(__file__).resolve()]
    before = [_digest(p) for p in inputs]
    if before[:2] != [SNAPSHOT_SHA256, SQL_SHA256]:
        raise ValueError('Unsupported accepted snapshot/SQL digest')
    output.mkdir()
    raw = output/'native.json'; errors = output/'native.stderr'
    command = ['duckdb', '-init', '/dev/null', '-readonly', '-bail', '-batch', '-json',
               '-cmd', 'SET enable_external_access=false', str(snapshot), '-f', str(query)]
    with raw.open('x', encoding='utf-8') as out, errors.open('x', encoding='utf-8') as err:
        subprocess.run(command, stdout=out, stderr=err, check=True, timeout=300)
    groups = defaultdict(list)
    for row in json.loads(raw.read_text()):
        if set(row) != {'kind', 'payload'}:
            raise ValueError('Unexpected native export row')
        groups[row['kind']].append(json.loads(row['payload']))
    if set(groups) != {'manifest', 'person', 'game', 'batting', 'event', 'team_result',
                       'target', 'check', 'source_count'}:
        raise ValueError('Unexpected native groups')
    if len(groups['manifest']) != 1 or len(groups['check']) != 1:
        raise ValueError('Invalid manifest/check grain')
    manifest = groups['manifest'][0]
    if (manifest.get('schema_version') != 'f03-spike-1'
            or manifest.get('catalog_sha256') != CATALOG_SHA256
            or manifest.get('query_version') != '2019-filtered-extract-v1'
            or manifest.get('source_url') != 'https://data.baseball.computer/baseball/v1/baseball.ducklake'
            or not all(_text(manifest.get(k)) for k in ('extracted_at', 'attribution'))):
        raise ValueError('Unsupported snapshot provenance')
    checks = groups['check'][0]
    nonzero = dict(games=2429, live_events=192025, pa=186517, outs=130270, non_pa_outs=936,
                   ibb=753, ci=61, responsibility_switches=16,
                   responsibility_pitcher_differences=13, sacrifice_credit_gap=62)
    zero = ('invalid_cohort', 'invalid_results', 'starter_map_mismatches', 'event_date_mismatches',
            'state_out_mismatches', 'fielding_out_mismatches', 'duplicate_state_keys', 'duplicate_fielding_keys')
    if checks != dict(nonzero, **{k: 0 for k in zero}):
        raise ValueError('Independent source checks disagree')
    counts = _unique(groups['source_count'], lambda c: c['name'])
    if (set(counts) != {'games', 'events', 'batting', 'results', 'team_results', 'people', 'states', 'fielding'}
            or any(c['source_rows'] != c['local_rows'] for c in counts.values())):
        raise ValueError('Source/local count reconciliation failed')
    report = inventory(groups['game'], groups['person'], groups['batting'], groups['event'], groups['team_result'])
    expected = sorted(groups['target'], key=lambda r: (r['game_id'], r['team_id'], r['player_id'], r['role']))
    if (report['records'] != expected or report['totals'] != dict(pa=186517, outs=130270)
            or any(r['unknown_rows'] for r in report['records'])
            or len(report['team_games']) != 30
            or any(n != (161 if t in ('DET', 'CHA') else 162) for t, n in report['team_games'].items())
            or sum(g['suspended'] for g in report['games']) != 4
            or sum(g['shortened'] for g in report['games']) != 10):
        raise ValueError('Independent target/cohort reconciliation failed')
    report['source_checks'] = checks
    report['warnings'] = WARNINGS
    report['provenance'] = dict(snapshot=manifest, source_commit=SOURCE_COMMIT,
        snapshot_sha256=before[0], query_sha256=before[1], recipe_sha256=before[2],
        native_export_sha256=_digest(raw), attribution=manifest['attribution'],
        usage_basis=dict(batter='batting.batter_id credited plate_appearances including IBB/CI',
                         pitcher='events.pitcher_id outs_on_play on every live event'),
        date_basis='original game date; no resumption-event dates or inferred membership stints')
    if [_digest(p) for p in inputs] != before:
        raise ValueError('Source/recipes changed during export')
    _write(output/'targets.json', report)
    if [_digest(p) for p in inputs] != before:
        raise ValueError('Source/recipes changed before receipt')
    _write(output/'receipt.json', dict(format=FORMAT+'-receipt',
        payload_sha256=_digest(output/'targets.json'), native_export_sha256=_digest(raw),
        inputs_sha256={p.name: h for p, h in zip(inputs, before)}, command=command))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('snapshot', type=Path)
    parser.add_argument('output', type=Path, help='New external directory; receipt.json is written last')
    args = parser.parse_args()
    report = publish(args.snapshot, args.output)
    print(json.dumps(dict(format=FORMAT, games=len(report['games']), records=len(report['records']),
                          totals=report['totals']), sort_keys=True))


if __name__ == '__main__':main()
