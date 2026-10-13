"""The viewer must display supplied engine records, not invent outcomes."""
import html
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
