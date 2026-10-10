import csv
import io
from pathlib import Path
import tempfile
import unittest
import zipfile

from audits.verify_pitcher_er import compare_pitcher_er, pitcher_er


GAME = 'SYN201904010'


def pline(player, side, earned='0', runs='2'):
    # Retrosheet version 3: r is index 12, er index 13; synthetic only.
    return ['stat', 'pline', player, str(side), '1', '3', '0', '4', '1',
            '0', '0', '0', runs, earned, '0', '0', '1', '0', '0', '0', '0', '0']


def game_rows():
    return [['version', '3'], pline('away_pitcher', 0, '0'), pline('home_pitcher', 1, '2')]


class PitcherErTests(unittest.TestCase):
    def compare(self, source_rows=None, boxes=None, source_text=None):
        if source_rows is None:
            source_rows = [(GAME, 'away_pitcher', '0'), (GAME, 'home_pitcher', '2')]
        if boxes is None:
            boxes = game_rows()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            text = io.StringIO()
            writer = csv.writer(text)
            writer.writerow(['id', GAME])
            writer.writerows(boxes)
            with zipfile.ZipFile(root / 'box.zip', 'w') as archive:
                archive.writestr('2019.EBA', text.getvalue())
                archive.writestr('2019.EBN', '')
            if source_text is None:
                with (root / 'source.csv').open('w', newline='') as f:
                    writer = csv.writer(f)
                    writer.writerow(['game_id', 'player_id', 'earned_runs'])
                    writer.writerows(source_rows)
            else:
                (root / 'source.csv').write_text(source_text)
            return compare_pitcher_er(root / 'box.zip', root / 'source.csv', wanted={GAME})

    def test_compares_exact_player_totals_including_explicit_zero(self):
        self.assertEqual(self.compare(), 2)

    def test_valid_quoted_source_numbers_are_accepted(self):
        source = f'game_id,player_id,earned_runs\n{GAME},away_pitcher,"0"\n{GAME},home_pitcher,"2"\n'
        self.assertEqual(self.compare(source_text=source), 2)

    def test_rejects_malformed_quoted_source_fields_and_header(self):
        valid = f'game_id,player_id,earned_runs\n{GAME},away_pitcher,0\n{GAME},home_pitcher,2\n'
        for source in [valid.replace('away_pitcher,0', 'away_pitcher,"0"0'),
                       valid.replace('away_pitcher,0', '"away_"pitcher,0'),
                       valid.replace('game_id', '"game_"id')]:
            with self.subTest(source=source), self.assertRaisesRegex(ValueError, 'CSV syntax'):
                self.compare(source_text=source)

    def test_reads_er_not_runs_or_outs(self):
        rows = [['version', '3'], pline('away_pitcher', 0, '1', '4'), pline('home_pitcher', 1, '0', '3')]
        self.assertEqual(pitcher_er(rows), {'away_pitcher': 1, 'home_pitcher': 0})

    def test_rejects_mismatching_er(self):
        with self.assertRaisesRegex(ValueError, 'mismatch'):
            self.compare([(GAME, 'away_pitcher', '1'), (GAME, 'home_pitcher', '2')])

    def test_missing_source_zero_is_not_filled(self):
        with self.assertRaisesRegex(ValueError, 'coverage'):
            self.compare([(GAME, 'home_pitcher', '2')])

    def test_missing_box_pitcher_is_not_filled(self):
        with self.assertRaisesRegex(ValueError, 'coverage'):
            self.compare(boxes=[['version', '3'], pline('away_pitcher', 0)])

    def test_rejects_unknown_source_er(self):
        for unknown in ['', 'NULL', '-1', '1.0']:
            with self.subTest(unknown=unknown), self.assertRaises(ValueError):
                self.compare([(GAME, 'away_pitcher', unknown), (GAME, 'home_pitcher', '2')])

    def test_rejects_unknown_or_impossible_box_er(self):
        for er, runs in [('', '2'), ('NULL', '2'), ('-1', '2'), ('3', '2')]:
            with self.subTest(er=er), self.assertRaises(ValueError):
                pitcher_er([['version', '3'], pline('away_pitcher', 0, er, runs), pline('home_pitcher', 1)])

    def test_rejects_malformed_source_rows(self):
        for malformed in [(GAME, 'away_pitcher'), (GAME, 'away_pitcher', '0', 'extra')]:
            with self.subTest(row=malformed), self.assertRaisesRegex(ValueError, 'CSV row'):
                self.compare([malformed, (GAME, 'home_pitcher', '2')])

    def test_rejects_empty_source_player_identity(self):
        with self.assertRaisesRegex(ValueError, 'identity'):
            self.compare([(GAME, '', '0'), (GAME, 'home_pitcher', '2')])

    def test_rejects_duplicate_source_key(self):
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.compare([(GAME, 'away_pitcher', '0'), (GAME, 'away_pitcher', '0'), (GAME, 'home_pitcher', '2')])

    def test_rejects_duplicate_box_pitcher(self):
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            pitcher_er(game_rows() + [pline('away_pitcher', 0)])

    def test_rejects_missing_or_unsupported_box_version(self):
        for rows in [game_rows()[1:], [['version', '2']] + game_rows()[1:]]:
            with self.subTest(rows=rows), self.assertRaisesRegex(ValueError, 'version'):
                pitcher_er(rows)

    def test_rejects_unexpected_source_game(self):
        with self.assertRaisesRegex(ValueError, 'game'):
            self.compare([('OTHER201904010', 'away_pitcher', '0'), (GAME, 'home_pitcher', '2')])

    def test_matching_partial_sources_cannot_omit_a_pitching_side(self):
        with self.assertRaisesRegex(ValueError, 'coverage'):
            self.compare([(GAME, 'away_pitcher', '0')], [['version', '3'], pline('away_pitcher', 0)])

    def test_rejects_empty_source(self):
        with self.assertRaises(ValueError):
            self.compare([])

    def test_rejects_empty_player_identity_and_invalid_box_side(self):
        with self.assertRaises(ValueError):
            pitcher_er([['version', '3'], pline('', 0), pline('home_pitcher', 1)])
        with self.assertRaises(ValueError):
            pitcher_er([['version', '3'], pline('away_pitcher', 2), pline('home_pitcher', 1)])


if __name__ == '__main__':
    unittest.main()
