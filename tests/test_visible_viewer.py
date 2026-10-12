"""The viewer must display supplied engine records, not invent outcomes."""
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


class VisibleViewerTests(unittest.TestCase):
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
