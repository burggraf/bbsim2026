import csv
import io
import unittest

from audits.verify_fixture_reconcile import METRICS, validate


class FixtureReconcileTests(unittest.TestCase):
    def sample(self, change=None):
        rows = []
        for side in ('Home', 'Away'):
            for metric in METRICS:
                row = ['GAME', side.upper(), side, metric, '2', '2', 'NULL']
                if change and (side, metric) == change[0]:
                    row = change[1](row)
                rows.append(row)
        buffer = io.StringIO()
        csv.writer(buffer).writerows(rows)
        buffer.seek(0)
        return buffer

    def test_accepts_matching_event_and_summary_with_unavailable_box(self):
        self.assertEqual(len(validate(self.sample(), {'GAME'})), 2 * len(METRICS))

    def test_rejects_event_summary_mismatch(self):
        with self.assertRaisesRegex(ValueError, 'summary mismatch'):
            validate(self.sample((('Home', 'runs'), lambda r: r[:5] + ['3', r[6]])), {'GAME'})

    def test_rejects_independent_box_mismatch(self):
        with self.assertRaisesRegex(ValueError, 'box mismatch'):
            validate(self.sample((('Away', 'hits'), lambda r: r[:6] + ['3'])), {'GAME'})

    def test_rejects_missing_summary_not_zero(self):
        with self.assertRaisesRegex(ValueError, 'missing summary'):
            validate(self.sample((('Home', 'pa'), lambda r: r[:5] + ['NULL', r[6]])), {'GAME'})

    def test_rejects_missing_metric_row(self):
        data = self.sample().read().splitlines()
        with self.assertRaisesRegex(ValueError, 'missing rows'):
            validate(io.StringIO('\n'.join(data[:-1]) + '\n'), {'GAME'})

    def test_rejects_duplicate_metric_row(self):
        text = self.sample().read()
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            validate(io.StringIO(text + text.splitlines()[0] + '\n'), {'GAME'})


if __name__ == '__main__':
    unittest.main()
