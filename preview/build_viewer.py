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
    return ('<details id="player-probabilities" open><summary>Player-specific probabilities — hindsight pilot</summary>'
            '<p>Fixed Dirichlet prior strength200 per role; generalized-log5 means, not calibrated or predictive. '
            'N is included terminal-event participant observations, not official PA/BF. Data weight shows shrinkage. '
            'No effective-handedness, park or opponent-adjusted fit; fielding/contact conditionals remain pooled.</p>'
            '<div style="overflow-x:auto"><table><thead>'+header(['Side','Slot','Batter','Opposing starter','Batter N','Pitcher N','Batter data weight','Pitcher data weight',*labels])+'</thead><tbody>'+''.join(cells)+'</tbody></table></div>'
            '<details><summary>Component counts / posterior means / SDs (assumed Dirichlet uncertainty)</summary>'
            '<p>SDs are not validated ability intervals or propagated matchup uncertainty. Full-precision native values are in the provenance panel.</p>'
            '<div style="overflow-x:auto"><table><thead>'+header(['Role','Player','N','Data weight',*labels])+'</thead><tbody>'+''.join(posterior)+'</tbody></table></div></details></details>')


CSS = """
*{box-sizing:border-box}body{margin:0;background:#101c25;color:#e9f0f2;font:16px system-ui,sans-serif}
main{max-width:1060px;margin:auto;padding:30px 24px}header{display:flex;justify-content:space-between;align-items:center;gap:20px}
h1{font-size:30px;margin:8px 0}.eyebrow{color:#9ab7b1;letter-spacing:3px;font-size:12px}.badge{padding:10px 16px;border:1px solid #425850;border-radius:30px;color:#a6e4bd}
.intro,footer{color:#aebfc8;line-height:1.6}.frame{display:none}#frame0{display:block}.frame:target{display:block}.frame:target~#frame0{display:none}
.score{display:grid;grid-template-columns:1fr 1fr 1fr;align-items:center;background:#1a2b38;border:1px solid #304756;border-radius:16px;padding:22px;margin:24px 0;text-align:center}
.team{color:#aac1d0;font-size:13px;letter-spacing:3px}.runs{font-size:60px;font-weight:750;line-height:1.2}.status{color:#a6e4bd;font-weight:600}
.layout{display:grid;grid-template-columns:1.3fr 1fr;gap:24px}.field,.play{background:#182c2b;border:1px solid #34504a;border-radius:16px;padding:24px}.play{background:#1a2b38;border-color:#304756}
svg{display:block;width:100%;height:auto;max-height:390px}.play h2{font-size:23px;line-height:1.4}.progress{color:#a6e4bd;font-size:13px;text-transform:uppercase;letter-spacing:2px}
nav{display:flex;flex-wrap:wrap;gap:10px;margin-top:24px}a{display:inline-block;text-decoration:none;color:#e9f0f2;background:#2a414f;border:1px solid #466373;border-radius:8px;padding:12px 16px}a.primary{background:#b6eb91;color:#142719;border-color:#b6eb91;font-weight:700}a:focus-visible{outline:3px solid #ffda77;outline-offset:4px}a:hover{filter:brightness(1.15)}
footer{font-size:13px;margin-top:26px}pre{white-space:pre-wrap;overflow-wrap:anywhere;font:13px ui-monospace,monospace;line-height:1.6;background:#15242f;border-radius:12px;padding:20px}summary{cursor:pointer;color:#b6eb91;padding:16px 0}
.game-log{margin-top:24px;border:1px solid #304756;border-radius:12px;padding:0 18px}.game-log-scroll{max-height:520px;overflow-y:auto;overscroll-behavior:contain;padding:0 12px 12px}.game-log-entry{border-top:1px solid #304756;padding:12px 0}.game-log-entry h3{font-size:17px}.game-log-entry pre{margin:8px 0}.game-log-entry summary{font-size:13px}.radio-call{line-height:1.55;margin:8px 0}.state-recap{font-size:13px;color:#aebfc8;line-height:1.6;margin:8px 0}.game-log-scroll:focus-visible{outline:3px solid #ffda77}
@media(max-width:720px){.layout{grid-template-columns:1fr}header{align-items:flex-start;flex-direction:column}.runs{font-size:44px}.score{padding:16px}.status{font-size:13px}main{padding:20px 14px}}
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
        positions = ['','P','C','1B','2B','3B','SS','LF','CF','RF','DH']
        tables = []
        for side in ['Away','Home']:
            rows = sorted((p for p in real[1] if p[0] == side),key=lambda p:int(p[1]))
            cells = ''.join('<tr class="starting-player">'
                + ''.join(f'<td>{escape(v)}</td>' for v in [p[1] if p[1]!='0' else 'Pitcher',
                           positions[int(p[2])],p[4],p[3]])+'</tr>' for p in rows)
            tables.append(f'<h3>{escape(real[0][side])} — {side}</h3><table><thead><tr>'
                          '<th>Batting slot</th><th>Position</th><th>Name</th><th>Source ID</th>'
                          f'</tr></thead><tbody>{cells}</tbody></table>')
        experiment += ('<details id="starting-lineups"><summary>Recorded starting lineups and positions</summary>'
                       + ''.join(tables) + '</details>')
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
                'supplied player names and source IDs are shown; raw native states keep source IDs.')
    else:
        log_content = ('<p>Detailed native before/after snapshots are unavailable in this older record. '
                       'This is the complete supplied summary; no missing states are inferred.</p>'
                       f'<pre>{escape(source)}</pre>')
    game_log = (f'<details class="game-log" id="game-log"><summary>Full game log — {count} plays</summary>'
                '<div class="game-log-scroll" tabindex="0" role="region" aria-label="Full game history">'
                + log_content + '</div></details>')

    def frame(index):
        description, away, home, status, bases = frames[index]
        markers = []
        for name, player, (x, y) in zip(["First", "Second", "Third"], bases,
                                       [(315, 240), (200, 125), (85, 240)]):
            occupied = player != "-"
            label = player if occupied else "empty"
            color = "#ffda77" if occupied else "#dae7dc"
            markers.append(f'<g role="img" aria-label="{name} base: {escape(label)}">'
                           f'<rect x="{x-9}" y="{y-9}" width="18" height="18" '
                           f'transform="rotate(45 {x} {y})" fill="{color}"/>'
                           f'<text x="{x}" y="{y-23}" text-anchor="middle" '
                           f'fill="{color}" font-size="13">{escape(label)}</text></g>')
        next_link = (f'<a class="primary" href="#frame{index+1}">Next play →</a>' if index < count
                     else '<a class="primary" href="#frame0">Replay game →</a>')
        previous = f'<a href="#frame{max(index-1, 0)}">← Previous</a>'
        stats = ('<details open><summary>Final line score and supplied-credit statistics</summary>'
                 f'<pre>{escape(source[final.start():])}</pre></details>') if index == count else ""
        return f'''<section class="frame" id="frame{index}" data-away="{away}" data-home="{home}">
<div class="score"><div><div class="team">{escape(labels[0])}</div><div class="runs">{away}</div></div>
<div class="status">{escape(status)}<br><small>Play {index} / {count}</small></div>
<div><div class="team">{escape(labels[1])}</div><div class="runs">{home}</div></div></div>
<div class="layout"><div class="field"><svg viewBox="0 0 400 400" aria-label="Baseball field">
<path d="M200 355L40 195Q200 -15 360 195Z" fill="#2c674c"/>
<path d="M200 355L85 240L200 125L315 240Z" fill="#b18a5f"/>
<path d="M200 355L85 240L200 125L315 240Z" fill="none" stroke="#f3ecda" stroke-width="2"/>
<circle cx="200" cy="240" r="18" fill="#c8a179"/>
<path d="M192 350H208V359L200 365L192 359Z" fill="#f4eee2"/>
{''.join(markers)}</svg></div>
<div class="play"><div class="progress">{'First pitch' if index == 0 else 'Recorded play '+str(index)}</div>
<h2>{escape(description)}</h2><p>Advance through the actual Rust-engine record. The field and score show the state after each play.</p>
<nav>{next_link}{previous}<a href="#frame0">Restart</a><a href="#frame{count}">Final result</a></nav></div></div>{stats}</section>'''

    # Put the initial frame last so native CSS :target hides it when another
    # frame is selected. No browser JavaScript, outcomes, server or dependencies.
    body = "\n".join(frame(i) for i in [*range(1, count + 1), 0])
    return f'''<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Baseball Simulator — visible engine preview</title><style>{CSS}</style></head>
<body><main><header><div><div class="eyebrow">2019 BASEBALL · ENGINE PREVIEW</div>
<h1>Baseball Simulator</h1></div><div class="badge">Offline · Rust-engine replay</div></header>
<p class="intro">{escape(intro)}</p>
<p class="intro">{escape(metadata)}</p>{experiment}{body}{game_log}
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
