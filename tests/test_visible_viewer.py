"""The viewer must display supplied engine records, not invent outcomes."""
import html
import re
from pathlib import Path
import subprocess
import tempfile
import unittest
from preview.build_viewer import build_viewer

RECORD = """EXPERIMENTAL GENERATED GAME — fictional equal teams, pooled 2019 baseline; not calibrated.
Seed: 1 | Model: pooled-jeffreys-preview-v1
Play 01 | Top 1 | Away-1: ground/contact out (0 run(s))
  Away 0 - Home 0 | Now: Top 1 | Outs: 1
  Bases: 1=- 2=- 3=-
Play 02 | Top 1 | Away-2: single (0 run(s))
  Away 0 - Home 0 | Now: Top 1 | Outs: 1
  Bases: 1=Away-2 2=- 3=-
Play 03 | Bottom 9 | Home-1: home run (1 run(s))
  Away 0 - Home 1 | GAME OVER
  Bases: 1=- 2=- 3=-
FINAL: Away 0 - Home 1
Supplied-credit statistics — not a complete official box.
"""


def traced_record():
    blocks = ''.join(
        f'TRACE PLAY {i} BEGIN\nBefore state: supplied before {i}\nAfter state: supplied after {i}\nBefore:\nGameState {{ outs: {i-1}, over: false }}\n'
        f'Event:\nEvent {{ result: "<inspect & escape>" }}\nAfter:\n'
        f'GameState {{ outs: {i}, over: {str(i == 3).lower()} }}\nTRACE PLAY {i} END\n'
        for i in range(1, 4))
    return 'TRACE FORMAT: engine-debug-v1\n' + blocks + RECORD


def announced_record():
    commentary = 'COMMENTARY FORMAT: radio-v1\n' + ''.join(
        f'CALL PLAY {i}: Home-pitcher to Away-{i}: a single! <inspect & escape>\n'
        f'RECAP PLAY {i}: Before: Top 1 · 0 outs · Away 0–Home 0 · bases empty → After: Top 1 · 0 outs · Away 0–Home 0 · Away-{i} at first\n'
        for i in range(1, 4))
    return commentary + traced_record()


def joint_record():
    prefix = ('EXPERIMENTAL GENERATED GAME — fictional equal teams, pooled 2019 baseline; not calibrated.\n'
              'Seed: 1 | Model: pooled-jeffreys-joint-proxy-preview-v1 | Policy: station-held-joint-proxy-preview-v1 | RNG: splitmix64-v1\n'
              'OPT-IN EXPERIMENT — source-shape mixture; synthetic held retirements; residual partial coverage, not calibration.\n'
              'The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".\n'
              'Supplied note: <inspect & escape>\n')
    source = announced_record()
    native, summary = source.split('EXPERIMENTAL GENERATED GAME', 1)
    plays = summary[summary.index('Play 01 |'):]
    return prefix + native + plays


def real_record():
    from test_real_lineup import witness
    source=joint_record().replace('Policy: station-held-joint-proxy-preview-v1','Policy: station-held-positional-starting-preview-v1')
    w=witness();metadata='LINEUP FORMAT: closed-starting-lineup-v1\nMatchup: OAK201903200 | 2019-03-20 | TOK01\n'
    for team in w['teams']:
        metadata+=f"TEAM {team['side']}: {team['team_name']}\n"
        field={p['player_id']:p for p in team['fielding']}
        pitcher=field[team['pitcher_id']]
        metadata+=f"PLAYER {team['side']} 0 1 {pitcher['player_id']}: {pitcher['name']}\n"
        for p in team['batting_order']:
            metadata+=f"PLAYER {team['side']} {p['slot']} {field[p['player_id']]['position']} {p['player_id']}: {p['name']}\n"
    return source.replace('COMMENTARY FORMAT:',metadata+'COMMENTARY FORMAT:',1)


def native_state(half='Top', inning=9, outs=0, score=(0, 0), cursors=(0, 0), bases=('-', '-', '-'), over=False):
    """Only the engine-debug-v1 fields used by presentation; no source rows."""
    teams = []
    for side, cursor in zip('ah', cursors):
        order = ''.join(f'                "{side}{i}",\n' for i in range(1, 10))
        teams.append('        TeamState {\n            batting_order: [\n' + order
                     + f'            ],\n            next_batter: {cursor},\n            pitcher: "{side}p",\n'
                     + '            dh_slot: Some(\n                3,\n            ),\n'
                     + '            dh_must_bat: false,\n            bench: [],\n            removed: [],\n        },\n')
    charge = 'hp' if half == 'Top' else 'ap'
    runners = ''.join('        None,\n' if p == '-' else
                      '        Some(\n            Runner {\n'
                      + f'                player_id: "{p}",\n                responsible_pitcher: "{charge}",\n'
                      + '            },\n        ),\n' for p in bases)
    debug = (f'GameState {{\n    inning: {inning},\n    half: {half},\n    outs: {outs},\n'
             + f'    score: [\n        {score[0]},\n        {score[1]},\n    ],\n    teams: [\n'
             + ''.join(teams) + '    ],\n    bases: [\n' + runners
             + f'    ],\n    over: {str(over).lower()},\n}}')
    base_summary = ' '.join(f'{i}=' + ('empty' if p == '-' else f'{p} (charged to {charge})')
                            for i, p in enumerate(bases, 1))
    summary = (f'{half} {inning} | Outs: {outs} | Away {score[0]} - Home {score[1]} | '
               + f'Bases: {base_summary} | Game over: {str(over).lower()}')
    return dict(debug=debug, summary=summary, half=half, inning=inning, outs=outs, score=score, bases=bases, over=over)


def native_record(states=None, line_score=True):
    # A short ninth-inning segment: same-half PA, third out, home walkoff.
    if states is None:
        states = [native_state(outs=1, cursors=(8, 0)), native_state(outs=2),
                  native_state(half='Bottom', cursors=(1, 0)),
                  native_state(half='Bottom', score=(0, 1), cursors=(1, 1), over=True)]
    source = real_record().split('COMMENTARY FORMAT:', 1)[0]
    source += 'COMMENTARY FORMAT: radio-v1\nTRACE FORMAT: engine-debug-v1\n'
    for i, (before, after) in enumerate(zip(states, states[1:]), 1):
        side = 'a' if before['half'] == 'Top' else 'h'
        batter = 'a9' if i == 1 else side + '1'
        source += (f'TRACE PLAY {i} BEGIN\nBefore state: {before["summary"]}\nAfter state: {after["summary"]}\n'
                   + f'Before:\n{before["debug"]}\nEvent:\nEvent {{ kind: PlateAppearance }}\nAfter:\n{after["debug"]}\nTRACE PLAY {i} END\n'
                   + f'CALL PLAY {i}: Last play: Player {batter}.\nRECAP PLAY {i}: Supplied recap.\n'
                   + f'Play {i:02} | {before["half"]} {before["inning"]} | Player {batter}: supplied outcome\n'
                   + f'  Away {after["score"][0]} - Home {after["score"][1]} | '
                   + ('GAME OVER' if after['over'] else f'Now: {after["half"]} {after["inning"]} | Outs: {after["outs"]}')
                   + '\n  Bases: ' + ' '.join(f'{n}={p}' for n, p in enumerate(after['bases'], 1)) + '\n')
    source += f'FINAL: Away {states[-1]["score"][0]} - Home {states[-1]["score"][1]}\n'
    if line_score:
        cells = {}
        for before, after in zip(states, states[1:]):
            side = 0 if before['half'] == 'Top' else 1
            key = before['inning'], side
            cells[key] = cells.get(key, 0) + after['score'][side] - before['score'][side]
        innings = sorted({n for n, _ in cells})
        source += 'Line score (observed innings):\nInning  ' + '  '.join(map(str, innings)) + '\n'
        for side, name in enumerate(['Away', 'Home']):
            source += name + '    ' + '  '.join(str(cells.get((n, side), '-')) for n in innings)
            source += f' | {states[-1]["score"][side]}\n'
    return source + 'Supplied-credit statistics — not a complete official box.\n'


def selected_frame(page, index):
    return page.split(f'id="frame{index}"', 1)[1].split('</section>', 1)[0]


class NativePresentationTests(unittest.TestCase):
    def test_normal_lineups_show_names_not_source_id_sublabels(self):
        page = build_viewer(native_record())
        lineups = page.split('<section id="starting-lineups"', 1)[1].split('</section>', 1)[0]
        self.assertIn('Player a1', lineups)
        self.assertIn('Pitcher a', lineups)
        self.assertNotIn('<small>a1</small>', lineups)
        self.assertNotIn('<small>ap</small>', lineups)
        self.assertIn('data-player="a1"', lineups)
        self.assertIn('PLAYER Away 1', page)

    def test_displayed_play_header_matches_its_actors_and_up_next_is_separate(self):
        page = build_viewer(native_record())
        for index, batter, pitcher, next_batter, next_pitcher in [
            (1, 'a9', 'hp', 'a1', 'hp'),
            (2, 'a1', 'hp', 'h1', 'ap'),
            (3, 'h1', 'ap', None, None),
        ]:
            frame = selected_frame(page, index)
            self.assertIn(f'<div class="matchup" data-play-batter="{batter}" data-play-pitcher="{pitcher}">', frame)
            header = frame.split('class="matchup"', 1)[1].split('Last completed play', 1)[0]
            self.assertIn(f'>Player {batter}</strong>', header)
            self.assertIn(f'>Pitcher {pitcher[0]}</strong>', header)
            self.assertIn(f'Last play: Player {batter}.', frame)
            if next_batter:
                upcoming = frame.split('<p class="up-next">', 1)[1].split('</p>', 1)[0]
                self.assertIn('Up next:', upcoming)
                self.assertIn(f'>Player {next_batter}</strong>', upcoming)
                self.assertIn(f'>Pitcher {next_pitcher[0]}</strong>', upcoming)
            else:
                self.assertNotIn('class="up-next"', frame)
        self.assertIn('>At bat</div>', selected_frame(page, 0))
        self.assertIn('class="current-marker">Up next', page)

    def test_native_cursor_wrap_and_half_boundary_choose_current_not_last_matchup(self):
        page = build_viewer(native_record())
        for index, batter, pitcher in [(0, 'a9', 'hp'), (1, 'a1', 'hp'), (2, 'h1', 'ap')]:
            frame = selected_frame(page, index)
            self.assertIn(f'data-batter="{batter}" data-pitcher="{pitcher}"', frame)
            self.assertIn(f'>Player {batter}</strong>', frame)
            self.assertIn(f'>Pitcher {pitcher[0]}</strong>', frame)
            self.assertIn(f'#frame{index}:target~#starting-lineups [data-player="{batter}"]', page)
        self.assertIn('Last play: Player a9.', selected_frame(page, 1))
        self.assertIn('Bottom 9 · 0 outs', selected_frame(page, 2))
        self.assertNotIn('data-batter=', selected_frame(page, 3))
        self.assertIn('Final', selected_frame(page, 3))
        self.assertEqual(page.count('class="frame"'), 4)
        self.assertLess(page.index('id="frame3"'), page.index('id="frame0"'))
        self.assertNotIn('<script', page)

    def test_named_escaped_bases_all_masks_keep_identity_and_no_factual_fielders(self):
        for mask in range(8):
            bases = tuple(f'h{i+2}' if mask & (1 << i) else '-' for i in range(3))
            states = [native_state(half='Bottom', bases=bases),
                      native_state(half='Bottom', bases=bases, score=(0, 1), over=True)]
            source = native_record(states).replace('Player h2', '<Same & name>').replace('Player h3', '<Same & name>')
            frame = selected_frame(build_viewer(source), 1)
            for i, name in enumerate(['First', 'Second', 'Third']):
                label = ('<Same & name>' if i < 2 else 'Player h4') if bases[i] != '-' else 'empty'
                self.assertIn(f'aria-label="{name} base: {html.escape(label)}"', frame)
            self.assertIn('Schematic base occupancy', frame)
            self.assertNotIn('data-batter=', frame)
            self.assertNotIn('<Same & name>', frame)

    def test_prefix_line_score_never_leaks_future_runs_and_keeps_unplayed_dash(self):
        page = build_viewer(native_record())
        for index, away, home in [(0, '—', '—'), (1, '0', '—'), (2, '0', '—'), (3, '0', '1')]:
            frame = selected_frame(page, index)
            self.assertIn(f'data-side="Away" data-inning="9">{away}</td>', frame)
            self.assertIn(f'data-side="Home" data-inning="9">{home}</td>', frame)
        self.assertIn('Line score unavailable', build_viewer(native_record(line_score=False)))

    def test_scoring_third_out_extra_innings_and_unplayed_final_bottom(self):
        states = [native_state(outs=2), native_state(half='Bottom', score=(1, 0)),
                  native_state(inning=10, score=(1, 1)),
                  native_state(inning=10, half='Bottom', score=(1, 1)),
                  native_state(inning=10, half='Bottom', score=(1, 2), over=True)]
        page = build_viewer(native_record(states))
        self.assertIn('data-side="Away" data-inning="9">1</td>', selected_frame(page, 1))
        self.assertNotIn('data-inning="10"', selected_frame(page, 1))
        self.assertIn('data-side="Home" data-inning="10">1</td>', selected_frame(page, 4))
        states = [native_state(outs=2, score=(0, 0)), native_state(score=(0, 0), over=True)]
        frame = selected_frame(build_viewer(native_record(states)), 1)
        self.assertIn('data-side="Home" data-inning="9">—</td>', frame)
        self.assertNotIn('data-batter=', frame)

    def test_partial_invalid_native_teams_summary_and_roster_are_rejected(self):
        source = native_record()
        mutations = [
            ('next_batter: 8', 'next_batter: 9'),
            ('next_batter: 8,', ''),
            ('next_batter: 8', 'next_batter: bad'),
            ('"a9",', ''),
            ('"a9",', '"a8",'),
            ('pitcher: "ap"', 'pitcher: "hp"'),
            ('"a9",', '"unknown",'),
            ('teams: [', 'missing_teams: ['),
            ('TeamState {', 'MissingTeam {'),
            ('half: Top,', 'half: Bottom,'),
            ('After state: Top 9 | Outs: 2', 'After state: Top 9 | Outs: 1'),
            ('  Away 0 - Home 0 | Now: Top 9 | Outs: 2', '  Away 0 - Home 0 | Now: Top 9 | Outs: 1'),
            ('Away    0 | 0', 'Away    1 | 0'),
        ]
        for old, new in mutations:
            self.assertIn(old, source)
            with self.subTest(old=old, new=new), self.assertRaises(ValueError):
                build_viewer(source.replace(old, new, 1))

    def test_legacy_without_any_team_fields_explicitly_discloses_unavailable_matchup(self):
        for source in [RECORD, traced_record(), real_record()]:
            page = build_viewer(source)
            self.assertIn('Current matchup unavailable', page)
            self.assertIn('Line score unavailable', page)
            self.assertNotIn('data-batter=', page)

    def test_gameplay_and_single_visible_lineups_precede_closed_secondary_panels(self):
        from test_player_ratings import rating_record
        # Use its complete probability metadata with the coherent native states.
        metadata = rating_record().split('COMMENTARY FORMAT:', 1)[0]
        source = metadata + 'COMMENTARY FORMAT:' + native_record().split('COMMENTARY FORMAT:', 1)[1]
        page = build_viewer(source)
        self.assertLess(page.index('class="frame"'), page.index('id="starting-lineups"'))
        self.assertLess(page.index('id="starting-lineups"'), page.index('id="player-probabilities"'))
        self.assertIn('<section id="starting-lineups"', page)
        self.assertIn('<details id="player-probabilities">', page)
        self.assertEqual(page.count('class="starting-player"'), 20)
        self.assertEqual(page.count('class="lineup-hitter"'), 18)
        self.assertEqual(page.count('class="matchup-probability"'), 18)
        self.assertEqual(page.count('class="role-posterior"'), 20)
        self.assertIn('coarse advancement', page)
        self.assertIn('not calibrated or predictive', page)
        self.assertIn(html.escape(source[source.index('FINAL:'):]), page)

    def test_installed_native_cli_states_drive_named_matchups(self):
        from test_live_preview import BINARY, joint_inputs
        from preview.real_lineup import encode_witness
        from test_real_lineup import witness
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline, counts = joint_inputs(root)
            lineup = root / 'lineup.tsv'
            lineup.write_text(encode_witness(witness()))
            source = subprocess.check_output([str(BINARY), '--baseline', str(baseline), '--contact-counts',
                                              str(counts), '--lineup', str(lineup), '--seed', '1',
                                              '--autoplay', '--trace'], text=True, timeout=30)
        page = build_viewer(source)
        self.assertIn('data-batter="a1" data-pitcher="hp"', selected_frame(page, 0))
        blocks = re.findall(r'After:\n(GameState .*?)\nTRACE PLAY \d+ END', source, re.DOTALL)
        for i, state in enumerate(blocks, 1):
            if 'over: true' in state:
                self.assertNotIn('data-batter=', selected_frame(page, i))
                continue
            half = re.search(r'half: (Top|Bottom)', state).group(1)
            side = 'a' if half == 'Top' else 'h'
            cursor = int(re.findall(r'next_batter: (\d+)', state)[half == 'Bottom'])
            self.assertIn(f'data-batter="{side}{cursor+1}"', selected_frame(page, i))
            self.assertIn(f'data-pitcher="{"hp" if side == "a" else "ap"}"', selected_frame(page, i))


class VisibleViewerTests(unittest.TestCase):
    def test_real_policy_shows_bound_teams_roster_and_limits_not_fictional_claims(self):
        page=build_viewer(real_record())
        self.assertIn('Seattle Mariners',page)
        self.assertIn('Oakland Athletics',page)
        self.assertIn('id="starting-lineups"',page)
        self.assertIn('Player a4',page)
        self.assertIn('a4',page)
        self.assertIn('DH',page)
        self.assertIn('Closed starting roster',page)
        self.assertIn('no substitutions',page)
        self.assertIn('pooled probabilities',page)
        self.assertNotIn('Fictional equal teams',page)
        self.assertNotIn('fictional player IDs are shown',page)
        self.assertEqual(page.count('class="starting-player"'),20)
        self.assertIn('id="frame3" data-away="0" data-home="1"',page)
        self.assertIn('<pre>'+html.escape(real_record().split('COMMENTARY FORMAT:',1)[0])+'</pre>',page)

    def test_real_roster_c1_control_is_rejected(self):
        with self.assertRaises(ValueError):build_viewer(real_record().replace('Player a1','Player'+chr(133)+'a1'))

    def test_real_roster_is_escaped_and_missing_duplicate_or_unknown_metadata_fails(self):
        source=real_record()
        page=build_viewer(source.replace('Player a1','<Player & a1>'))
        self.assertIn('&lt;Player &amp; a1&gt;',page)
        self.assertNotIn('<Player & a1>',page)
        for bad in [source.replace('PLAYER Away 1 7 a1: Player a1\n',''),
                    source.replace('PLAYER Home 1 7 h1','PLAYER Home 1 7 a1'),
                    source.replace('PLAYER Away 4 10 a4','PLAYER Away 4 9 a4'),
                    source.replace('LINEUP FORMAT: closed-starting-lineup-v1','LINEUP FORMAT: unknown')]:
            with self.subTest(bad=bad),self.assertRaises(ValueError):build_viewer(bad)

    def test_joint_assumptions_and_verbatim_preamble_are_visible_and_escaped(self):
        source = joint_record()
        page = build_viewer(source)
        self.assertIn('Joint-contact experiment', page)
        self.assertIn('synthetic held retirements', page)
        self.assertIn('residual partial coverage, not calibration', page)
        self.assertIn('Retrosheet', page)
        prefix = source[:source.index('COMMENTARY FORMAT:')]
        self.assertIn('<pre>' + html.escape(prefix) + '</pre>', page)
        self.assertNotIn('<inspect & escape>', page)
        self.assertNotIn('complete official box score', page)
        self.assertIn('class="game-log-entry"', page)

    def test_joint_static_page_keeps_recorded_calls_states_and_statistics(self):
        page = build_viewer(joint_record())
        self.assertIn('id="frame3" data-away="0" data-home="1"', page)
        self.assertIn('Home-pitcher to Away-2: a single!', page)
        self.assertIn('FINAL: Away 0 - Home 1', page)
        self.assertIn('id="game-log"', page)
        self.assertNotIn('<script', page)

    def test_radio_calls_and_compact_recaps_are_visible_outside_technical_details(self):
        page = build_viewer(announced_record())
        for i in range(1, 4):
            entry = page.split(f'id="log-play{i}"', 1)[1].split('</article>', 1)[0]
            visible = entry.split('<details>', 1)[0]
            self.assertIn(f'Home-pitcher to Away-{i}: a single!', visible)
            self.assertIn('Before: Top 1 · 0 outs', visible)
            self.assertIn(f'After: Top 1 · 0 outs · Away 0–Home 0 · Away-{i} at first', visible)
            self.assertIn('&lt;inspect &amp; escape&gt;', visible)
            self.assertNotIn('<inspect & escape>', page)
            self.assertNotIn('supplied before', visible)
            self.assertIn('GameState', entry.split('<details>', 1)[1])
            frame = page.split(f'id="frame{i}"', 1)[1].split('</section>', 1)[0]
            self.assertIn(f'Home-pitcher to Away-{i}: a single!', frame)
        self.assertIn('<details class="game-log" id="game-log">', page)

    def test_incomplete_misnumbered_or_unsupported_commentary_is_rejected(self):
        source = announced_record()
        for broken in [source.replace('CALL PLAY 2:', 'CALL PLAY 4:'),
                       source.replace('RECAP PLAY 2:', 'MISSING PLAY 2:'),
                       source.replace('radio-v1', 'unknown'),
                       source.replace('COMMENTARY FORMAT: radio-v1\n', ''),
                       source.replace('RECAP PLAY 1:', 'RECAP PLAY 2:', 1)]:
            with self.subTest(source=broken), self.assertRaises(ValueError):
                build_viewer(broken)

    def test_full_log_is_closed_scrollable_and_preserves_every_native_snapshot(self):
        page = build_viewer(traced_record())
        self.assertIn('<details class="game-log" id="game-log">', page)
        self.assertIn('class="game-log-scroll"', page)
        self.assertIn('overflow-y:auto', page)
        self.assertEqual(page.count('class="game-log-entry"'), 3)
        for i in range(1, 4):
            self.assertIn(f'id="log-play{i}"', page)
            self.assertIn(f'GameState {{ outs: {i-1}, over: false }}', page)
        self.assertIn('Before:', page)
        self.assertIn('After:', page)
        self.assertIn('&lt;inspect &amp; escape&gt;', page)
        self.assertNotIn('<inspect & escape>', page)
        self.assertIn('GameState { outs: 3, over: true }', page)

    def test_partial_or_misnumbered_native_trace_is_not_presented_as_complete(self):
        for source in [traced_record().replace('TRACE PLAY 2 END', 'TRACE PLAY 4 END'),
                       traced_record().replace('TRACE FORMAT: engine-debug-v1', 'TRACE FORMAT: unknown'),
                       traced_record().replace('After:', 'Missing:', 1)]:
            with self.subTest(source=source), self.assertRaises(ValueError):
                build_viewer(source)

    def test_legacy_full_summary_discloses_unavailable_native_snapshots(self):
        page = build_viewer(RECORD)
        self.assertIn('id="game-log"', page)
        self.assertIn('Detailed native before/after snapshots are unavailable', page)
        self.assertIn('Play 01 | Top 1 | Away-1', page)
    def test_supplied_score_runner_and_navigation_are_visible(self):
        page = build_viewer(RECORD)
        self.assertIn('id="frame0"', page)
        self.assertIn('id="frame3"', page)
        self.assertIn('href="#frame1"', page)
        self.assertIn('href="#frame0"', page)
        self.assertIn('data-away="0" data-home="1"', page)
        self.assertIn('aria-label="First base: Away-2"', page)
        self.assertIn("GAME OVER", page)
        self.assertIn("not a complete official box", page)
        self.assertIn("Fictional equal teams", page)
        self.assertNotIn("<script", page)

    def test_unrecognized_or_incomplete_records_are_not_silently_displayed(self):
        for record in ["", RECORD.replace("Play 02", "Play 04"),
                       RECORD.replace("FINAL: Away 0 - Home 1", ""),
                       RECORD.replace("  Bases: 1=Away-2 2=- 3=-", "")]:
            with self.subTest(record=record), self.assertRaises(ValueError):
                build_viewer(record)
