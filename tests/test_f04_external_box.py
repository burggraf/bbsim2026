import csv
import tempfile
import unittest
import zipfile
from pathlib import Path

from audits.verify_retrosheet_box import compare_box


GAME = 'ANA201904040'
METRICS = ('runs', 'hits', 'pa', 'steals', 'caught_stealing', 'errors', 'putouts', 'assists', 'outs')
EXPECTED = {
    0: dict(runs=3, hits=2, pa=3, steals=1, caught_stealing=0, errors=0, putouts=3, assists=1, outs=3),
    1: dict(runs=4, hits=1, pa=3, steals=0, caught_stealing=1, errors=1, putouts=2, assists=2, outs=2),
}


def batting(player, side, *, ab, hits, walks, sb, cs):
    # boxfile.txt: id,side,pos,seq,ab,r,h,2b,3b,hr,rbi,sh,sf,hbp,bb,ibb,k,sb,cs,gidp,int
    return ['stat', 'bline', player, side, 1, 1, ab, 0, hits, 0, 0, 0, 0,
            0, 0, 0, walks, 0, 0, sb, cs, 0, 0]


class ExternalBoxTests(unittest.TestCase):
    def fixture(self, root):
        archive = root / '2019box.zip'
        rows = [
            ['id', GAME], ['info', 'visteam', 'TEX'], ['info', 'hometeam', 'ANA'],
            ['line', 0, 1, 2], ['line', 1, 0, 4],
            batting('away', 0, ab=2, hits=2, walks=1, sb=1, cs=0),
            batting('home', 1, ab=3, hits=1, walks=0, sb=0, cs=1),
            # id,side,seq,pos,if*3,po,a,e,dp,tp,pb
            ['stat', 'dline', 'away', 0, 1, 6, 9, 3, 1, 0, 0, 0, 0],
            ['stat', 'dline', 'home', 1, 1, 6, 9, 2, 2, 1, 0, 0, 0],
        ]
        import io
        out = io.StringIO()
        csv.writer(out).writerows(rows)
        with zipfile.ZipFile(archive, 'w') as z:
            z.writestr('2019.EBA', out.getvalue())
            z.writestr('2019.EBN', '')
        reference = root / 'reconcile.csv'
        with reference.open('w', newline='') as f:
            writer = csv.writer(f)
            for side, label, team in ((0, 'Away', 'TEX'), (1, 'Home', 'ANA')):
                for metric in METRICS:
                    value = EXPECTED[side][metric]
                    writer.writerow((GAME, team, label, metric, value, value, 'NULL'))
        return archive, reference

    def test_compares_all_nine_metrics_per_team_from_separate_box_records(self):
        with tempfile.TemporaryDirectory() as name:
            archive, reference = self.fixture(Path(name))
            self.assertEqual(compare_box(archive, reference), 18)

    def test_rejects_disagreeing_box_credit(self):
        with tempfile.TemporaryDirectory() as name:
            archive, reference = self.fixture(Path(name))
            lines = reference.read_text().replace('Away,assists,1,1,NULL', 'Away,assists,2,2,NULL')
            reference.write_text(lines)
            with self.assertRaisesRegex(ValueError, 'assists'):
                compare_box(archive, reference)

    def test_rejects_missing_box_line_even_if_zero_would_match(self):
        with tempfile.TemporaryDirectory() as name:
            archive, reference = self.fixture(Path(name))
            with zipfile.ZipFile(archive) as z:
                box = z.read('2019.EBA').decode()
            box = ''.join(line for line in box.splitlines(keepends=True)
                          if not line.startswith('line,0,'))
            with zipfile.ZipFile(archive, 'w') as z:
                z.writestr('2019.EBA', box)
                z.writestr('2019.EBN', '')
            reference.write_text(reference.read_text().replace('Away,runs,3,3,NULL', 'Away,runs,0,0,NULL'))
            with self.assertRaisesRegex(ValueError, 'missing box line'):
                compare_box(archive, reference)

    def test_rejects_absent_game(self):
        with tempfile.TemporaryDirectory() as name:
            archive, reference = self.fixture(Path(name))
            reference.write_text(reference.read_text().replace(GAME, 'ANA201904050'))
            with self.assertRaisesRegex(ValueError, 'missing box game'):
                compare_box(archive, reference)


if __name__ == '__main__':
    unittest.main()
