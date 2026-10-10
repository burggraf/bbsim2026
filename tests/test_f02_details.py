import csv
import io
import unittest

from audits.verify_fixture_details import validate


class FixtureDetailTests(unittest.TestCase):
    def row(self):
        # Synthetic non-PA runner-only advance: first -> second, no out/run/PA.
        return ['GAME','1','10','false','NULL','BAT','PIT','H','A',
                '3','Bottom','0','1','R','NULL','NULL','0','0','3','Bottom','0','2',
                'NULL','R','NULL','0','0','0','0','false','NULL','NULL','NULL',
                '0','0','0','0','First:R->Second[WildPitch]']

    def check(self, row):
        buf = io.StringIO()
        csv.writer(buf).writerow(row)
        buf.seek(0)
        return validate(buf, {('GAME', 1)})

    def test_accepts_runner_only_transition(self):
        self.assertEqual(len(self.check(self.row())), 1)

    def test_rejects_non_pa_with_pa_credit(self):
        r = self.row(); r[30] = '1'
        with self.assertRaisesRegex(ValueError, 'non-PA'):
            self.check(r)

    def test_rejects_out_accounting_mismatch(self):
        r = self.row(); r[20] = '1'
        with self.assertRaisesRegex(ValueError, 'outs'):
            self.check(r)

    def test_rejects_score_delta_mismatch(self):
        r = self.row(); r[27] = '1'
        with self.assertRaisesRegex(ValueError, 'score'):
            self.check(r)

    def test_rejects_runner_identity_without_base(self):
        r = self.row(); r[22] = 'R'
        with self.assertRaisesRegex(ValueError, 'base identities'):
            self.check(r)


if __name__ == '__main__':
    unittest.main()
