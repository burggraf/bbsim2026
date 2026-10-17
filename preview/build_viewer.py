"""Offline, read-only visual playback of a Rust generated_game terminal record."""
from pathlib import Path
import argparse
import html
import re
import json
import math

JOINT_MODEL = 'pooled-jeffreys-joint-proxy-preview-v1'
RATED_MODEL = 'dirichlet-log5-player-proxy-preview-v1'


def is_rated_record(source):
    metadata = next((line for line in source.splitlines() if line.startswith('Seed:')), '')
    return f'Model: {RATED_MODEL}' in metadata.split(' | ')


def is_joint_record(source):
    metadata = next((line for line in source.splitlines() if line.startswith('Seed:')), '')
    return f'Model: {JOINT_MODEL}' in metadata.split(' | ') or is_rated_record(source)


REAL_POLICY = 'station-held-positional-starting-preview-v1'


def real_metadata(source):
    metadata = next((line for line in source.splitlines() if line.startswith('Seed:')), '')
    if f'Policy: {REAL_POLICY}' not in metadata.split(' | '):
        return None
    if not is_joint_record(source):
        raise ValueError('Real lineup requires the joint model')
    start = re.search(r'^(?:TRACE FORMAT:|COMMENTARY FORMAT:|Play [0-9]+ \|)', source, re.MULTILINE)
    preamble = source[:start.start()] if start else source
    if (re.findall(r'^LINEUP FORMAT: (.+)$', preamble, re.MULTILINE) != ['closed-starting-lineup-v1']
            or re.findall(r'^Matchup: (.+)$', preamble, re.MULTILINE) != ['OAK201903200 | 2019-03-20 | TOK01']):
        raise ValueError('Unsupported real-lineup metadata')
    teams = re.findall(r'^TEAM (Away|Home): ([^\n]+)$', preamble, re.MULTILINE)
    players = re.findall(r'^PLAYER (Away|Home) ([0-9]) (10|[1-9]) ([a-z0-9]+): ([^\n]+)$', preamble, re.MULTILINE)
    if (len(teams) != 2 or dict(teams) != {'Away':'Seattle Mariners','Home':'Oakland Athletics'}
            or len(re.findall(r'^TEAM ',preamble,re.MULTILINE)) != 2
            or len(players) != 20 or len(re.findall(r'^PLAYER ',preamble,re.MULTILINE)) != 20
            or len({p[3] for p in players}) != 20):
        raise ValueError('Missing/invalid real starting records')
    for side in ['Away','Home']:
        rows = [p for p in players if p[0] == side]
        if (len(rows) != 10 or {int(p[1]) for p in rows} != set(range(10))
                or {int(p[2]) for p in rows} != set(range(1,11))
                or any((p[1]=='0') != (p[2]=='1') or not p[4].strip()
                       or any(ord(c)<32 or 127<=ord(c)<=159 for c in p[4]) for p in rows)):
            raise ValueError('Invalid starting slot/position/name grain')
    return dict(teams), players


def rating_panel(source, real):
    if not is_rated_record(source):
        return ''
    if real is None:
        raise ValueError('Rated model requires real starting metadata')
    start = re.search(r'^(?:TRACE FORMAT:|COMMENTARY FORMAT:|Play [0-9]+ \|)', source, re.MULTILINE)
    preamble = source[:start.start()] if start else source
    if (re.findall(r'^RATINGS FORMAT: (.+)$', preamble, re.MULTILINE) != ['player-proxy-counts-v1']
            or re.findall(r'^Rating prior strength: (.+)$', preamble, re.MULTILINE) != ['batter200 pitcher200']):
        raise ValueError('Unsupported player rating metadata')
    def vector(text, normalized=True):
        values = json.loads(text)
        if (not isinstance(values,list) or len(values)!=9
                or any(type(x) not in (int,float) or not math.isfinite(x)
                       or (not 0<x<1 if normalized else not 0<=x<=.5) for x in values)
                or normalized and abs(sum(values)-1)>32*math.ulp(1.0)):
            raise ValueError('Invalid native player probability/SD vector')
        return values
    rows=re.findall(r'^RATING (batter|pitcher) ([a-z0-9]+) \| N: ([0-9]+) \| Weight: ([^|\n]+) \| Mean: (\[[^\n]+\]) \| SD: (\[[^\n|]+\]) \| Counts: (\[[^\n|]+\])$',preamble,re.MULTILINE)
    expected={('pitcher' if p[1]=='0' else 'batter',p[3]) for p in real[1]}
    if len(rows)!=20 or len(re.findall(r'^RATING ',preamble,re.MULTILINE))!=20 or {(r[0],r[1]) for r in rows}!=expected:
        raise ValueError('Incomplete/duplicate player posterior rows')
    ratings={}
    for role,player,n,weight,mean,sd,counts in rows:
        weight=float(weight)
        counts=json.loads(counts)
        if (not math.isfinite(weight) or not 0<=weight<1 or int(n)>185703
                or not isinstance(counts,list) or len(counts)!=9
                or any(type(x) is not int or x<0 for x in counts) or sum(counts)!=int(n)):
            raise ValueError('Invalid native shrinkage weight/count conservation')
        ratings[role,player]=(n,weight,vector(mean),vector(sd,False),counts)
    matches=re.findall(r'^MATCHUP (Away|Home) ([1-9]) ([a-z0-9]+) ([a-z0-9]+): (\[[^\n]+\])$',preamble,re.MULTILINE)
    pitchers={p[0]:p[3] for p in real[1] if p[1]=='0'}
    expected={(p[0],p[1],p[3],pitchers['Home' if p[0]=='Away' else 'Away']) for p in real[1] if p[1]!='0'}
    if len(matches)!=18 or len(re.findall(r'^MATCHUP ',preamble,re.MULTILINE))!=18 or {r[:4] for r in matches}!=expected:
        raise ValueError('Incomplete/duplicate matchup probability rows')
    names={p[3]:p[4] for p in real[1]}
    cells=[]
    for side,slot,batter,pitcher,p in sorted(matches,key=lambda r:(r[0],int(r[1]))):
        values=[side,slot,names[batter]+' ['+batter+']',names[pitcher]+' ['+pitcher+']',
                ratings['batter',batter][0],ratings['pitcher',pitcher][0],
                f'{ratings["batter",batter][1]:.1%}',f'{ratings["pitcher",pitcher][1]:.1%}',
                *[f'{x:.3%}' for x in vector(p)]]
        cells.append('<tr class="matchup-probability">'+''.join('<td>'+html.escape(v)+'</td>' for v in values)+'</tr>')
    posterior=[]
    for (role,player),(n,weight,mean,sd,counts) in sorted(ratings.items()):
        values=[role,names[player]+' ['+player+']',n,f'{weight:.1%}',*[f'{c} / {m:.3%} / {s:.3%}' for c,m,s in zip(counts,mean,sd)]]
        posterior.append('<tr class="role-posterior">'+''.join('<td>'+html.escape(v)+'</td>' for v in values)+'</tr>')
    labels=['K','BB','HBP','1B','2B','3B','HR','ROE','Contact out']
    header=lambda values:'<tr>'+''.join('<th>'+html.escape(v)+'</th>' for v in values)+'</tr>'
    return ('<details id="player-probabilities"><summary>Player-specific probabilities — hindsight pilot</summary>'
            '<p>Fixed Dirichlet prior strength200 per role; generalized-log5 means, not calibrated or predictive. '
            'N is included terminal-event participant observations, not official PA/BF. Data weight shows shrinkage. '
            'No effective-handedness, park or opponent-adjusted fit; fielding/contact conditionals remain pooled.</p>'
            '<div style="overflow-x:auto"><table><thead>'+header(['Side','Slot','Batter','Opposing starter','Batter N','Pitcher N','Batter data weight','Pitcher data weight',*labels])+'</thead><tbody>'+''.join(cells)+'</tbody></table></div>'
            '<details><summary>Component counts / posterior means / SDs (assumed Dirichlet uncertainty)</summary>'
            '<p>SDs are not validated ability intervals or propagated matchup uncertainty. Full-precision native values are in the provenance panel.</p>'
            '<div style="overflow-x:auto"><table><thead>'+header(['Role','Player','N','Data weight',*labels])+'</thead><tbody>'+''.join(posterior)+'</tbody></table></div></details></details>')


def native_state_fields(debug, summary):
    """Read only the generated engine-debug-v1 state shape, never event/reducer logic."""
    if not re.search(r'\b(?:teams|TeamState|batting_order|next_batter|pitcher)\b', debug):
        return None  # Legacy inspection stubs have no team metadata at all.
    identifier = r'[A-Za-z0-9-]{1,64}'
    state = re.fullmatch(
        r'GameState \{\s*inning: ([0-9]+),\s*half: (Top|Bottom),\s*outs: ([0-2]),'
        r'\s*score: \[\s*([0-9]+),\s*([0-9]+),\s*\],\s*teams: \[(.*?)\],'
        r'\s*bases: \[(.*?)\],\s*over: (true|false),\s*\}', debug, re.DOTALL)
    if not state:
        raise ValueError('Unsupported or partial native GameState')
    inning, half, outs, away, home, team_text, base_text, over = state.groups()
    if not 1 <= int(inning) <= 65535 or max(int(away), int(home)) > 4294967295:
        raise ValueError('Native clock/score out of range')
    team_pattern = (r'\s*TeamState \{\s*batting_order: \[((?:\s*"' + identifier + r'",){9})\s*\],'
                    r'\s*next_batter: ([0-8]),\s*pitcher: "(' + identifier + r')",'
                    r'\s*dh_slot: (?:None|Some\(\s*[0-8],\s*\)),\s*dh_must_bat: (?:true|false),'
                    r'\s*bench: \[\s*\],\s*removed: \[\s*\],\s*\},')
    teams = list(re.finditer(team_pattern, team_text))
    if len(teams) != 2 or ''.join(t.group() for t in teams).strip() != team_text.strip():
        raise ValueError('Expected two complete generated native teams')
    teams = [(tuple(re.findall(r'"(' + identifier + r')"', t[1])), int(t[2]), t[3]) for t in teams]
    identities = [p for order, _, pitcher in teams for p in (*order, pitcher)]
    if len(set(identities)) != 20:
        raise ValueError('Duplicate native starting identities')
    runner_pattern = (r'\s*(?:None|Some\(\s*Runner \{\s*player_id: "(' + identifier
                      + r')",\s*responsible_pitcher: "(' + identifier + r')",\s*\},\s*\)),')
    runners = list(re.finditer(runner_pattern, base_text))
    if len(runners) != 3 or ''.join(r.group() for r in runners).strip() != base_text.strip():
        raise ValueError('Expected three complete native base slots')
    bases = tuple((r[1], r[2]) if r[1] else None for r in runners)
    offense = half == 'Bottom'
    occupied = [r[0] for r in bases if r]
    if (len(set(occupied)) != len(occupied)
            or any(p not in teams[offense][0] for p in occupied)
            or any(r and r[1] != teams[1-offense][2] for r in bases)):
        raise ValueError('Native bases disagree with closed starting teams')
    base_summary = ' '.join(f'{i}=' + (f'{r[0]} (charged to {r[1]})' if r else 'empty')
                            for i, r in enumerate(bases, 1))
    expected = (f'{half} {inning} | Outs: {outs} | Away {away} - Home {home} | '
                f'Bases: {base_summary} | Game over: {over}')
    if summary != expected:
        raise ValueError('Native state and summary disagree')
    return dict(inning=int(inning), half=half, outs=int(outs), score=(int(away), int(home)),
                teams=teams, bases=tuple(r[0] if r else '-' for r in bases), over=over == 'true')


def prefix_line_scores(source, states):
    section = re.search(r'^Line score \(observed innings\):\nInning([^\n]+)\n'
                        r'Away\s+([^|\n]+)\| ([0-9]+)\nHome\s+([^|\n]+)\| ([0-9]+)$', source, re.MULTILINE)
    if 'Line score (observed innings):' not in source:
        return None
    if not section:
        raise ValueError('Malformed native line score')
    if not states:
        return None
    prefixes = [{}]
    for before, after in zip(states, states[1:]):
        cells = prefixes[-1].copy()
        side = int(before['half'] == 'Bottom')
        deltas = [a-b for a, b in zip(after['score'], before['score'])]
        if min(deltas) < 0 or deltas[1-side] != 0:
            raise ValueError('Native score changes disagree with batting half')
        key = before['inning'], side
        cells[key] = cells.get(key, 0) + deltas[side]
        if tuple(sum(v for (_, s), v in cells.items() if s == side) for side in range(2)) != after['score']:
            raise ValueError('Prefix line score does not reconcile to native score')
        prefixes.append(cells)
    innings = sorted({n for n, _ in prefixes[-1]})
    if section[1].split() != list(map(str, innings)):
        raise ValueError('Native line-score innings disagree with trace')
    for side, (text, total) in enumerate([(section[2], section[3]), (section[4], section[5])]):
        if (text.split() != [str(prefixes[-1].get((n, side), '-')) for n in innings]
                or int(total) != states[-1]['score'][side]):
            raise ValueError('Native final line score disagrees with trace')
    return prefixes


CSS = """
*{box-sizing:border-box}body{margin:0;background:#101c25;color:#edf4f6;font:16px system-ui,sans-serif}
main{max-width:1180px;margin:auto;padding:26px 28px}header{display:flex;justify-content:space-between;align-items:center;gap:20px;border-bottom:1px solid #344651;padding-bottom:18px}
h1{font-size:28px;letter-spacing:-1px;margin:6px 0}.eyebrow,.progress{color:#90c8c4;letter-spacing:2px;font-size:11px;text-transform:uppercase}.badge{padding:8px 12px;border:1px solid #425850;border-radius:6px;color:#b6e9d2;font-size:12px}
.intro,footer{color:#afc0c8;line-height:1.6}.notice{font-size:13px;margin:14px 0}.frame{display:none}#frame0{display:block}.frame:target{display:block}.frame:target~#frame0{display:none}
.score{display:grid;grid-template-columns:1fr auto 1fr;align-items:stretch;background:#172934;border:1px solid #354b58;border-radius:12px;overflow:hidden;margin:18px 0}
.team-score{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:16px 20px;border-top:3px solid var(--accent)}.away{--accent:#6ed3d0}.home{--accent:#f1c96c}.team{font-size:16px;font-weight:650}.side-label{display:block;font-size:10px;letter-spacing:2px;color:#afc0c8;margin-bottom:4px}.runs{font:700 40px ui-monospace,monospace}.status{align-self:center;text-align:center;padding:12px 20px;color:#b6e9d2;font-size:14px;font-weight:650}.status small{display:block;font-weight:400;color:#afc0c8;margin-top:5px}
.layout{display:grid;grid-template-columns:1fr 1.15fr;gap:20px}.field,.play{min-width:0;background:#142b29;border:1px solid #34504a;border-radius:12px;padding:22px}.field{background:radial-gradient(ellipse at 50% 65%,#254f3c,#122c29 75%)}.play{background:#172934;border-color:#354b58;display:flex;flex-direction:column}.field svg{display:block;width:100%;max-height:240px;margin:0 auto}.field-caption{text-align:center;color:#a7c6b7;font-size:11px;letter-spacing:1px;margin:8px 0}.base-list{display:grid;grid-template-columns:repeat(3,1fr);gap:8px;margin-top:12px}.base-list div{min-width:0;background:#102522;padding:9px;border-radius:6px;font-size:12px;overflow-wrap:anywhere}.base-list span{display:block;color:#a4bdb3;font-size:10px;text-transform:uppercase;letter-spacing:1px;margin-bottom:4px}.matchup{display:grid;grid-template-columns:1fr 1fr;gap:16px;border-bottom:1px solid #354b58;padding-bottom:18px;margin-bottom:20px}.matchup strong{display:block;font-size:21px;margin:6px 0;overflow-wrap:anywhere}.matchup small{color:#afc0c8}.play h2{font-size:22px;line-height:1.5;margin:8px 0 20px;overflow-wrap:anywhere}.ready{color:#afc0c8;line-height:1.5}.final-label{font-size:24px;color:#f1c96c;margin-bottom:14px}
nav{display:flex;flex-wrap:wrap;gap:8px;margin-top:auto;padding-top:18px}a{display:inline-block;text-decoration:none;color:#edf4f6;background:#263f4d;border:1px solid #466373;border-radius:6px;padding:10px 13px;font-size:13px}a.primary{background:#b6eb91;color:#142719;border-color:#b6eb91;font-weight:700}a:focus-visible,summary:focus-visible{outline:3px solid #ffda77;outline-offset:4px}a:hover{filter:brightness(1.15)}
.line-score-wrap{overflow-x:auto;margin:16px 0}.line-score{width:100%;border-collapse:collapse;font:13px ui-monospace,monospace;background:#14242e}.line-score caption{text-align:left;font:11px system-ui,sans-serif;color:#afc0c8;letter-spacing:1px;text-transform:uppercase;padding:8px 0}.line-score th,.line-score td{padding:10px;text-align:center;border-bottom:1px solid #30434f;white-space:nowrap}.line-score th:first-child{text-align:left}.line-score .total{font-weight:bold;color:#f1c96c;border-left:1px solid #30434f}
.lineups{display:grid;grid-template-columns:1fr 1fr;gap:20px;margin:28px 0}.lineup{min-width:0;border:1px solid #354b58;border-top:3px solid var(--accent);border-radius:8px;background:#152631;padding:18px}.lineup h3{font-size:18px;margin:0 0 12px}.lineup table{border-collapse:collapse;width:100%;font-size:14px}.lineup th,.lineup td{text-align:left;padding:8px 6px;border-bottom:1px solid #2a404d}.lineup th{font-size:10px;text-transform:uppercase;letter-spacing:1px;color:#afc0c8}.lineup td:first-child{color:#afc0c8;width:26px}.lineup td:nth-child(2){font:12px ui-monospace,monospace;width:42px}.lineup small{display:block;color:#afc0c8;font-size:10px;margin-top:3px}.current-marker{display:none;font-size:10px;color:#f1c96c;margin-left:8px}.starter{font-size:13px;margin:16px 0 0;line-height:1.8;color:#afc0c8}.starter strong{color:#edf4f6}.lineups-title{grid-column:1/-1;margin:0;font-size:13px;letter-spacing:2px;text-transform:uppercase;color:#afc0c8}
footer{font-size:12px;margin-top:26px}pre{white-space:pre-wrap;overflow-wrap:anywhere;font:12px ui-monospace,monospace;line-height:1.6;background:#15242f;border-radius:8px;padding:18px}summary{cursor:pointer;color:#b6eb91;padding:16px 0}#model-details,#player-probabilities,#provenance{border-top:1px solid #304756;font-size:13px}#player-probabilities table{font-size:12px;border-spacing:10px}
.game-log{margin-top:24px;border:1px solid #304756;border-radius:8px;padding:0 18px}.game-log-scroll{max-height:520px;overflow-y:auto;overscroll-behavior:contain;padding:0 12px 12px}.game-log-entry{border-top:1px solid #304756;padding:12px 0}.game-log-entry h3{font-size:17px}.game-log-entry pre{margin:8px 0}.game-log-entry summary{font-size:13px}.radio-call{line-height:1.55;margin:8px 0}.state-recap{font-size:13px;color:#afc0c8;line-height:1.6;margin:8px 0}.game-log-scroll:focus-visible{outline:3px solid #ffda77}
@media(max-width:720px){main{padding:18px 12px}.layout,.lineups{grid-template-columns:1fr}header{align-items:flex-start}.badge{display:none}h1{font-size:24px}.team-score{padding:12px 9px;gap:8px}.team{font-size:12px}.runs{font-size:30px}.status{padding:10px;font-size:12px}.score{grid-template-columns:1fr 86px 1fr}.side-label{font-size:9px;letter-spacing:1px}.field,.play{padding:16px}.matchup strong{font-size:18px}.play h2{font-size:20px}.line-score th,.line-score td{padding:8px}.lineups{gap:16px}.lineups-title{grid-column:auto}}
"""


def build_viewer(source: str) -> str:
    pattern = re.compile(
        r"^Play (\d+) \| ([^|\n]+) \| ([^\n]+)\n"
        r"  Away (\d+) - Home (\d+) \| ([^\n]+)\n"
        r"  Bases: 1=(\S+) 2=(\S+) 3=(\S+)$", re.MULTILINE)
    records = list(pattern.finditer(source))
    final = re.search(r"^FINAL: Away (\d+) - Home (\d+)$", source, re.MULTILINE)
    if not records or not final or len(records) != len(re.findall(r"^Play ", source, re.MULTILINE)):
        raise ValueError("Expected a complete, recognized generated_game record")
    frames = [("Ready for the first pitch", 0, 0, "Top 1 · 0 outs", ["-"] * 3)]
    for expected, record in enumerate(records, 1):
        number, inning, description, away, home, status, first, second, third = record.groups()
        if int(number) != expected:
            raise ValueError("Play numbering must be continuous")
        if status == "GAME OVER":
            if expected != len(records):
                raise ValueError("Game over must be the last play")
            shown_status = "GAME OVER"
        else:
            state = re.fullmatch(r"Now: (Top|Bottom) (\d+) \| Outs: ([0-2])", status)
            if not state:
                raise ValueError("Unrecognized after-state")
            half, current_inning, outs = state.groups()
            shown_status = f"{half} {current_inning} · {outs} outs"
        frames.append((f"{inning} · {description}", int(away), int(home), shown_status,
                       [first, second, third]))
    if records[-1].group(6) != "GAME OVER" or frames[-1][1:3] != tuple(map(int, final.groups())):
        raise ValueError("Final score must match the terminal play")
    metadata = next((line for line in source.splitlines() if line.startswith("Seed:")), "Seed not supplied")
    escape = html.escape
    real = real_metadata(source)
    labels = ('AWAY', 'HOME') if real is None else (real[0]['Away'], real[0]['Home'])
    intro = ('Fictional equal teams, using the pooled 2019 league baseline. This is a visual simulation replay, '
             'not yet realistic team matchups or managerial gameplay.')
    experiment = ''
    lineups = ''
    positions = ['', 'P', 'C', '1B', '2B', '3B', 'SS', 'LF', 'CF', 'RF', 'DH']
    names = {} if real is None else {p[3]: p[4] for p in real[1]}
    player_positions = {} if real is None else {p[3]: positions[int(p[2])] for p in real[1]}
    if is_joint_record(source):
        start = re.search(r'^(?:TRACE FORMAT:|COMMENTARY FORMAT:|Play [0-9]+ \|)', source, re.MULTILINE)
        preamble = source[:start.start()]
        experiment = ('<p class="intro">Joint-contact experiment: source-shape mixture with synthetic '
                      'named catches and held runners; residual coverage remains partial. '
                      'Not calibrated player matchups or a fielding-skill model.</p>'
                      '<details id="model-details"><summary>Joint-contact assumptions and supplied provenance</summary>'
                      f'<pre>{escape(preamble)}</pre></details>')
    if real is not None:
        intro = ('Recorded Seattle–Oakland starting personnel, 2019-03-20 (TOK01). Generated exhibition, '
                 'not the historical result. All players use identical pooled probabilities, '
                 'not fitted player abilities or effective matchup hands. Closed starting roster: '
                 'no substitutions or eligible bench/bullpen claim; starting pitchers stay all game. '
                 'Coarse positional assignments are not historical ball locations or fielding skill.')
        tables = []
        for side in ['Away', 'Home']:
            rows = sorted((p for p in real[1] if p[0] == side), key=lambda p: int(p[1]))
            cells = ''.join(f'<tr class="starting-player" data-player="{p[3]}">'
                           f'<td>{p[1]}</td><td>{positions[int(p[2])]}</td>'
                           f'<td class="lineup-hitter">{escape(p[4])}<span class="current-marker">Up next</span>'
                           '</td></tr>' for p in rows[1:])
            pitcher = rows[0]
            tables.append(f'<div class="lineup {side.lower()}"><h3>{escape(real[0][side])}</h3>'
                          '<table><thead><tr><th scope="col">Order</th><th scope="col">Pos</th>'
                          f'<th scope="col">Hitter</th></tr></thead><tbody>{cells}</tbody></table>'
                          f'<div class="starting-player"><p class="starter">Starting pitcher · '
                          f'<strong>{escape(pitcher[4])}</strong></p></div></div>')
        lineups = ('<section id="starting-lineups" class="lineups" aria-label="Recorded starting lineups and positions">'
                   '<h2 class="lineups-title">Recorded starting lineups and positions</h2>' + ''.join(tables) + '</section>')
    if is_rated_record(source):
        intro = ('Recorded Seattle–Oakland starting personnel. Player-specific, league-shrunk participant-proxy '
                 'probabilities: selected-season hindsight reconstruction, not calibrated or predictive. '
                 'Closed starting roster; no substitutions or eligible bench/bullpen claim; starters stay all game. '
                 'No handedness/park/fielding-skill model or historical-result claim.')
        experiment += rating_panel(source, real)
    count = len(records)
    trace = list(re.finditer(r'^TRACE PLAY (\d+) BEGIN\n(.*?)^TRACE PLAY \1 END\n',
                             source, re.MULTILINE | re.DOTALL))
    has_trace = bool(re.search(r'^TRACE (FORMAT:|PLAY )', source, re.MULTILINE))
    commentary = list(re.finditer(r'^CALL PLAY (\d+): ([^\n]+)\nRECAP PLAY \1: ([^\n]+)\n',
                                  source, re.MULTILINE))
    has_commentary = bool(re.search(r'^(COMMENTARY FORMAT:|CALL PLAY |RECAP PLAY )', source, re.MULTILINE))
    if has_commentary:
        if (not has_trace
                or re.findall(r'^COMMENTARY FORMAT: (.+)$', source, re.MULTILINE) != ['radio-v1']
                or len(commentary) != count
                or len(re.findall(r'^(?:CALL|RECAP) PLAY ', source, re.MULTILINE)) != 2 * count
                or [int(c.group(1)) for c in commentary] != list(range(1, count + 1))):
            raise ValueError('Expected complete ordered supported commentary per native play')
    entries = []
    native_pairs = []
    if has_trace:
        if (re.findall(r'^TRACE FORMAT: (.+)$', source, re.MULTILINE) != ['engine-debug-v1']
                or len(trace) != count
                or len(re.findall(r'^TRACE PLAY ', source, re.MULTILINE)) != 2 * count):
            raise ValueError('Expected one complete supported native trace per play')
        for i, (item, record) in enumerate(zip(trace, records), 1):
            if int(item.group(1)) != i:
                raise ValueError('Native trace numbering must match play numbering')
            body = item.group(2)
            parts = re.fullmatch(r'Before state: ([^\n]+)\nAfter state: ([^\n]+)\n'
                                 r'Before:\n(.+?)\nEvent:\n(.+?)\nAfter:\n(.+)\n', body, re.DOTALL)
            if not parts:
                raise ValueError('Native trace must include before, event and after snapshots')
            before = native_state_fields(parts[3], parts[1])
            after = native_state_fields(parts[5], parts[2])
            native_pairs.append((before, after))
            heading = f'Play {i:02} · {record.group(2)}'
            if has_commentary:
                call, recap = commentary[i - 1].group(2, 3)
                compact = f'<p class="radio-call">{escape(call)}</p><p class="state-recap">{escape(recap)}</p>'
                frames[i] = (call, *frames[i][1:])
            else:
                heading += f' · {record.group(3)}'
                compact = f'<pre>Before: {escape(parts.group(1))}\nAfter:  {escape(parts.group(2))}</pre>'
            entries.append(f'<article class="game-log-entry" id="log-play{i}"><h3>{escape(heading)}</h3>'
                           + compact + '<details><summary>Full native state and event facts</summary>'
                           f'<pre>{escape(body)}</pre></details></article>')
        log_content = ('<p>Calls and state recaps use recorded facts only; fictional player IDs are shown. '
                       'Hit locations and unrecorded fielders are not invented. '
                       'Engine-provided before/event/after snapshots. After states normalize inning '
                       'boundaries; the event retains outs, runs, runner moves and credits. '
                       'Expand an entry for full lineups, pitchers, runner charges and all supplied facts.</p>'
                       + ''.join(entries))
        if real is not None:
            log_content = log_content.replace('fictional player IDs are shown.',
                'supplied player names are shown; raw native states keep source IDs.')
    else:
        log_content = ('<p>Detailed native before/after snapshots are unavailable in this older record. '
                       'This is the complete supplied summary; no missing states are inferred.</p>'
                       f'<pre>{escape(source)}</pre>')
    game_log = (f'<details class="game-log" id="game-log"><summary>Full game log — {count} plays</summary>'
                '<div class="game-log-scroll" tabindex="0" role="region" aria-label="Full game history">'
                + log_content + '</div></details>')

    states = []
    if any(before is not None or after is not None for before, after in native_pairs):
        if any(before is None or after is None for before, after in native_pairs):
            raise ValueError('Partial native team metadata across snapshots')
        states = [native_pairs[0][0], *[after for _, after in native_pairs]]
        for i, ((before, after), record) in enumerate(zip(native_pairs, records), 1):
            if before != states[i-1] or before['over'] or after['over'] != (i == count):
                raise ValueError('Native state continuity or terminal status disagrees')
            if record[2] != f'{before["half"]} {before["inning"]}':
                raise ValueError('Play heading disagrees with native before state')
            status = 'GAME OVER' if after['over'] else f'Now: {after["half"]} {after["inning"]} | Outs: {after["outs"]}'
            if (tuple(map(int, record.group(4, 5))) != after['score']
                    or record[6] != status or record.group(7, 8, 9) != after['bases']):
                raise ValueError('Play summary disagrees with native after state')
            if real is not None:
                for state in [before, after]:
                    for side, (order, _, pitcher) in zip(['Away', 'Home'], state['teams']):
                        roster = sorted((p for p in real[1] if p[0] == side), key=lambda p: int(p[1]))
                        if order != tuple(p[3] for p in roster[1:]) or pitcher != roster[0][3]:
                            raise ValueError('Native team disagrees with closed starting roster')
        initial = states[0]
        frames[0] = ('Game ready', *initial['score'],
                     f'{initial["half"]} {initial["inning"]} · {initial["outs"]} outs', initial['bases'])
    prefixes = prefix_line_scores(source, states)

    def current_matchup(state):
        if state is None or state['over']:
            return None
        side = int(state['half'] == 'Bottom')
        order, cursor, _ = state['teams'][side]
        return order[cursor], state['teams'][1-side][2]

    lineup_css = ''
    if states and lineups:
        initial_matchup = current_matchup(states[0])
        if initial_matchup:
            selector = f'#starting-lineups [data-player="{initial_matchup[0]}"]'
            lineup_css += selector + '{background:#294537}' + selector + ' .current-marker{display:inline}'
        lineup_css += ('.frame:target~#starting-lineups [data-player]{background:transparent}'
                       '.frame:target~#starting-lineups .current-marker{display:none}')
        for i, state in enumerate(states):
            matchup = current_matchup(state)
            if matchup:
                selector = f'#frame{i}:target~#starting-lineups [data-player="{matchup[0]}"]'
                lineup_css += selector + '{background:#294537}' + selector + ' .current-marker{display:inline}'

    def line_score(index):
        if prefixes is None:
            return '<p class="state-recap">Line score unavailable in this record.</p>'
        cells = prefixes[index]
        innings = sorted({n for n, _ in cells} | {states[0]['inning']})
        headings = ''.join(f'<th scope="col">{n}</th>' for n in innings)
        rows = []
        for side, name in enumerate(['Away', 'Home']):
            values = ''.join(f'<td data-side="{name}" data-inning="{n}">{cells.get((n, side), "—")}</td>' for n in innings)
            rows.append(f'<tr><th scope="row">{escape(labels[side])}</th>{values}'
                        f'<td class="total">{states[index]["score"][side]}</td></tr>')
        return ('<div class="line-score-wrap"><table class="line-score"><caption>Line score · through this play</caption>'
                '<thead><tr><th scope="col">Team</th>' + headings + '<th scope="col">R</th></tr></thead>'
                '<tbody>' + ''.join(rows) + '</tbody></table></div>')

    def frame(index):
        description, away, home, status, bases = frames[index]
        state = states[index] if states else None
        matchup = current_matchup(state)
        identity = ''
        upcoming = ''
        if matchup:
            batter, pitcher = matchup
            identity = f' data-batter="{batter}" data-pitcher="{pitcher}"'
            if index:
                upcoming = ('<p class="up-next">Up next: '
                            f'<strong>{escape(names.get(batter, batter))}</strong> vs '
                            f'<strong>{escape(names.get(pitcher, pitcher))}</strong></p>')
        # Completed-play actors come from its before-state, not the next lineup cursor.
        played = current_matchup(states[index-1]) if states and index else matchup
        if played:
            batter, pitcher = played
            current = (f'<div class="matchup" data-play-batter="{batter}" data-play-pitcher="{pitcher}">'
                       f'<div><div class="progress">{"Batter · play " + str(index) if index else "At bat"}</div>'
                       f'<strong>{escape(names.get(batter, batter))}</strong><small>{escape(player_positions.get(batter, ""))}</small></div>'
                       '<div><div class="progress">Pitching</div>'
                       f'<strong>{escape(names.get(pitcher, pitcher))}</strong><small>Starting pitcher</small></div></div>')
        else:
            current = '<p class="state-recap">Current matchup unavailable — native team cursors not supplied.</p>'
        if state and state['over']:
            status = f'Final · GAME OVER · {state["outs"]} outs'
            current = '<div class="final-label">Final</div>' + current
        base_labels = []
        markers = []
        for name, player, (x, y) in zip(["First", "Second", "Third"], bases,
                                       [(315, 240), (200, 125), (85, 240)]):
            occupied = player != "-"
            label = names.get(player, player) if occupied else "empty"
            base_labels.append(f'<div><span>{name} base</span>{escape(label) if occupied else "—"}</div>')
            color = "#ffda77" if occupied else "#dae7dc"
            markers.append(f'<g role="img" aria-label="{name} base: {escape(label)}">'
                           f'<rect x="{x-9}" y="{y-9}" width="18" height="18" '
                           f'transform="rotate(45 {x} {y})" fill="{color}"/>'
                           f'<text x="{x}" y="{y-23}" text-anchor="middle" '
                           f'fill="{color}" font-size="13">{name}</text></g>')
        next_link = (f'<a class="primary" href="#frame{index+1}">Next play →</a>' if index < count
                     else '<a class="primary" href="#frame0">Replay game →</a>')
        previous = f'<a href="#frame{max(index-1, 0)}">← Previous</a>'
        stats = ('<details><summary>Final line score and supplied-credit statistics</summary>'
                 f'<pre>{escape(source[final.start():])}</pre></details>') if index == count else ""
        return f'''<section class="frame" id="frame{index}" data-away="{away}" data-home="{home}"{identity}>
<div class="score"><div class="team-score away"><div class="team"><span class="side-label">Away{' · Batting' if state and not state['over'] and state['half']=='Top' else ''}</span>{escape(labels[0])}</div><div class="runs">{away}</div></div>
<div class="status">{escape(status)}<small>Play {index} / {count}</small></div>
<div class="team-score home"><div class="team"><span class="side-label">Home{' · Batting' if state and not state['over'] and state['half']=='Bottom' else ''}</span>{escape(labels[1])}</div><div class="runs">{home}</div></div></div>
<div class="layout"><div class="field"><svg viewBox="0 0 400 400" role="img" aria-label="Schematic base occupancy">
<path d="M200 355L40 195Q200 -15 360 195Z" fill="#28513c"/>
<path d="M200 355L85 240L200 125L315 240Z" fill="#866d4e"/>
<path d="M200 355L85 240L200 125L315 240Z" fill="none" stroke="#e6e0c9" stroke-width="2"/>
<path d="M192 350H208V359L200 365L192 359Z" fill="#f4eee2"/>
{''.join(markers)}</svg><p class="field-caption">SCHEMATIC · BASE OCCUPANCY ONLY</p>
<div class="base-list">{''.join(base_labels)}</div></div>
<div class="play">{current}<div class="progress">{'Game ready' if index == 0 else 'Last completed play · '+str(index)}</div>
<h2>{escape(description)}</h2>{upcoming}
<nav aria-label="Recorded game navigation">{next_link}{previous}<a href="#frame0">Restart</a><a href="#frame{count}">Final result</a></nav></div></div>{line_score(index)}{stats}</section>'''


    # Put the initial frame last so native CSS :target hides it when another
    # frame is selected. No browser JavaScript, outcomes, server or dependencies.
    body = "\n".join(frame(i) for i in [*range(1, count + 1), 0])
    return f'''<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{escape(labels[0])} vs {escape(labels[1])} — Baseball Simulator</title><style>{CSS}{lineup_css}</style></head>
<body><main><header><div><div class="eyebrow">2019 BASEBALL · ENGINE PREVIEW</div>
<h1>{'Seattle at Oakland' if real else 'Baseball Simulator'}</h1></div><div class="badge">Generated exhibition · Offline replay</div></header>
<p class="intro notice">Experimental generated exhibition · coarse advancement · starters stay all game · partial supplied-credit statistics. Not calibrated or predictive.</p>
{body}{lineups}
<details id="provenance"><summary>Experiment limitations and native metadata</summary><p class="intro">{escape(intro)}</p><p class="intro">{escape(metadata)}</p></details>
{experiment}{game_log}
<footer>Read-only playback: the browser does not generate outcomes. Statistics are supplied-credit totals, not a complete official box. No accuracy or finished-product claim.</footer>
</main></body></html>'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.write_text(build_viewer(args.input.read_text()))


if __name__ == "__main__":
    main()
