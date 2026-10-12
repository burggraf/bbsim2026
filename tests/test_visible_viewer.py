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


class VisibleViewerTests(unittest.TestCase):
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
