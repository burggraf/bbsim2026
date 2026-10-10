import csv
import io
import unittest

from audits.verify_2019 import validate


class AuditValidationTests(unittest.TestCase):
    def sample(self, changes=None):
        rows = {
            'games_source_rows': (2, 2, 0),
            'games_distinct_ids': (2, 2, 0),
            'games_with_results': (2, 2, 0),
            'events_rows': (5, 5, 0),
            'events_distinct_keys': (5, 5, 0),
            'events_pa': (4, 3, 0),
            'events_non_pa': (4, 1, 0),
            'events_no_play': (5, 1, 0),
            'events_state_before': (5, 5, 0),
            'events_state_after': (5, 4, 1),
        }
        rows.update(changes or {})
        buffer = io.StringIO()
        writer = csv.writer(buffer)
        for name, (denominator, count, unknown) in rows.items():
            writer.writerow((name, denominator, count, unknown, 'fixture'))
        buffer.seek(0)
        return buffer

    def test_accepts_consistent_count_with_unknown_after_state(self):
        self.assertEqual(validate(self.sample())['events_rows'][1], 5)

    def test_rejects_duplicate_game_ids(self):
        with self.assertRaisesRegex(ValueError, 'games_distinct_ids'):
            validate(self.sample({'games_distinct_ids': (2, 1, 1)}))

    def test_rejects_lost_event_partition(self):
        with self.assertRaisesRegex(ValueError, 'PA partition'):
            validate(self.sample({'events_non_pa': (4, 0, 0)}))

    def test_rejects_unknown_count_above_denominator(self):
        with self.assertRaisesRegex(ValueError, 'events_state_after'):
            validate(self.sample({'events_state_after': (5, 4, 6)}))

    def test_accepts_ten_slots_per_side_when_dh_used(self):
        self.assertEqual(validate(self.sample({'lineup_slots': (20, 20, 0)}))['lineup_slots'], (20, 20, 0))

    def test_rejects_lineup_slots_exceeding_dh_aware_denominator(self):
        with self.assertRaisesRegex(ValueError, 'lineup_slots'):
            validate(self.sample({'lineup_slots': (18, 20, -2)}))

    def test_rejects_duplicate_metric(self):
        text = self.sample().read() + 'events_rows,5,5,0,duplicate\n'
        with self.assertRaisesRegex(ValueError, 'duplicate metric'):
            validate(io.StringIO(text))


if __name__ == '__main__':
    unittest.main()
