"""Offline, read-only visual playback of a Rust generated_game terminal record."""
from pathlib import Path
import argparse
import html
import re

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
.game-log{margin-top:24px;border:1px solid #304756;border-radius:12px;padding:0 18px}.game-log-scroll{max-height:520px;overflow-y:auto;overscroll-behavior:contain;padding:0 12px 12px}.game-log-entry{border-top:1px solid #304756;padding:12px 0}.game-log-entry h3{font-size:17px}.game-log-entry pre{margin:8px 0}.game-log-entry summary{font-size:13px}.game-log-scroll:focus-visible{outline:3px solid #ffda77}
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
    count = len(records)
    trace = list(re.finditer(r'^TRACE PLAY (\d+) BEGIN\n(.*?)^TRACE PLAY \1 END\n',
                             source, re.MULTILINE | re.DOTALL))
    has_trace = bool(re.search(r'^TRACE (FORMAT:|PLAY )', source, re.MULTILINE))
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
            heading = f'Play {i:02} · {record.group(2)} · {record.group(3)}'
            entries.append(f'<article class="game-log-entry" id="log-play{i}"><h3>{escape(heading)}</h3>'
                           f'<pre>Before: {escape(parts.group(1))}\nAfter:  {escape(parts.group(2))}</pre>'
                           '<details><summary>Full native state and event facts</summary>'
                           f'<pre>{escape(body)}</pre></details></article>')
        log_content = ('<p>Engine-provided before/event/after snapshots. After states normalize inning '
                       'boundaries; the event retains outs, runs, runner moves and credits. '
                       'Expand an entry for full lineups, pitchers, runner charges and all supplied facts.</p>'
                       + ''.join(entries))
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
<div class="score"><div><div class="team">AWAY</div><div class="runs">{away}</div></div>
<div class="status">{escape(status)}<br><small>Play {index} / {count}</small></div>
<div><div class="team">HOME</div><div class="runs">{home}</div></div></div>
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
<p class="intro">Fictional equal teams, using the pooled 2019 league baseline. This is a visual simulation replay, not yet realistic team matchups or managerial gameplay.</p>
<p class="intro">{escape(metadata)}</p>{body}{game_log}
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
