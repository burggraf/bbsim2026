import unittest

from audits.profile_http import parse_http_stats


class HttpProfileTests(unittest.TestCase):
    def test_sums_reported_http_payloads_and_get_requests(self):
        text = '''HTTPFS HTTP Stats
││ in: 142.7 KiB ││
││ out: 0 bytes ││
││ #HEAD: 0 ││
││ #GET: 4 ││
HTTPFS HTTP Stats
││ in: 1.0 MiB ││
││ out: 0 bytes ││
││ #HEAD: 1 ││
││ #GET: 3 ││'''
        result = parse_http_stats(text)
        self.assertEqual(result['statements'], 2)
        self.assertEqual(result['get_requests'], 7)
        self.assertAlmostEqual(result['reported_payload_bytes'], (142.7 * 1024) + 1048576)

    def test_rejects_missing_http_measurement(self):
        with self.assertRaisesRegex(ValueError, 'HTTPFS'):
            parse_http_stats('no remote query here')


if __name__ == '__main__':
    unittest.main()
