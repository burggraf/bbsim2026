import io
import json
import tempfile
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from urllib.request import Request, urlopen
from urllib.error import HTTPError
from audits.bounded_ranges import Budget, RangeRelay

URL = 'https://data.baseball.computer/baseball/v1/bc_publish_data/example.parquet'

class Response(io.BytesIO):
    def __init__(self, body=b'2345', *, status=206, span='bytes 2-5/10', length='4', etag='"one"', url=URL):
        super().__init__(body)
        self.status, self.url = status, url
        self.headers = {'Content-Range': span, 'Content-Length': length, 'ETag': etag}
    def geturl(self): return self.url

class BoundedRangeTests(unittest.TestCase):
    def test_budget_is_shared_atomic_and_never_refunds_failed_reservations(self):
        budget = Budget(10)
        with ThreadPoolExecutor(4) as pool:
            successes = list(pool.map(budget.reserve, [3, 3, 3, 3]))
        self.assertEqual(sum(successes), 3)
        self.assertEqual(budget.reserved, 9)
        self.assertFalse(budget.reserve(2))
        self.assertEqual(budget.reserved, 9)

    def relay(self, opener, cap=10):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        relay = RangeRelay({'1': {'url': URL, 'size': 10}}, cap, Path(folder.name)/'requests.jsonl', opener=opener)
        self.addCleanup(relay.close)
        return relay

    def request(self, relay, span='bytes=2-5', path='/1', method='GET'):
        request = Request(relay.base_url + path, method=method,
                          headers={} if span is None else {'Range': span})
        try:
            with urlopen(request, timeout=5) as response:
                return response.status, response.read(), response.headers
        except HTTPError as error:
            error.close()
            raise

    def test_exact_range_and_head_use_pinned_size_without_full_download(self):
        seen = []
        def opener(request, **kwargs):
            seen.append(request)
            return Response()
        relay = self.relay(opener)
        status, body, headers = self.request(relay, method='HEAD', span=None)
        self.assertEqual((status, body, headers['Content-Length']), (200, b'', '10'))
        self.assertEqual(seen, [])
        status, body, headers = self.request(relay)
        self.assertEqual((status, body, headers['Content-Range']), (206, b'2345', 'bytes 2-5/10'))
        self.assertEqual(seen[0].full_url, URL)
        self.assertEqual(seen[0].get_header('Range'), 'bytes=2-5')
        self.assertEqual(relay.budget.received, 4)
        relay.close()  # Join handlers before reading their final request evidence.
        records=[json.loads(line) for line in relay.log.read_text().splitlines()]
        self.assertEqual(records[-1]['bytes'], 4)
        self.assertEqual(len(records[-1]['sha256']), 64)

    def test_unknown_routes_full_get_invalid_ranges_and_cap_never_call_origin(self):
        seen=[]
        relay=self.relay(lambda *a, **k: seen.append(a), cap=3)
        for span,path in [(None,'/1'),('bytes=2-5','/other'),('bytes=2-99','/1'),
                          ('bytes=2-','/1'),('bytes=1-2,4-5','/1'),('bytes=2-5','/1')]:
            with self.subTest(span=span,path=path), self.assertRaises(Exception):
                self.request(relay,span,path)
        self.assertEqual(seen,[])
        self.assertEqual(relay.budget.received,0)

    def test_bad_status_range_size_encoding_redirect_and_missing_etag_fail_closed(self):
        cases=[dict(status=200),dict(span='bytes 2-5/11'),dict(length='5'),
               dict(url='https://other.invalid/file'),dict(etag='')]
        for change in cases:
            with self.subTest(change=change):
                relay=self.relay(lambda *a, **k: Response(**change))
                with self.assertRaises(Exception):self.request(relay)
                self.assertEqual(relay.budget.received,0)
        response=Response();response.headers['Content-Encoding']='gzip'
        relay=self.relay(lambda *a, **k: response)
        with self.assertRaises(Exception):self.request(relay)
        self.assertEqual(relay.budget.received,0)

    def test_changed_etag_and_truncated_body_cannot_be_successful(self):
        responses=iter([Response(),Response(etag='"two"')])
        relay=self.relay(lambda *a, **k: next(responses))
        self.assertEqual(self.request(relay)[1],b'2345')
        with self.assertRaises(Exception):self.request(relay)
        self.assertEqual(relay.budget.received,4)
        relay=self.relay(lambda *a, **k: Response(body=b'23'))
        with self.assertRaises(Exception):self.request(relay)
        self.assertEqual(relay.budget.received,2)

    def test_catalog_relative_schema_and_table_paths_are_supported(self):
        url = URL.replace('example.parquet', 'main_models/stg_event_audit/example.parquet')
        with tempfile.TemporaryDirectory() as folder:
            relay = RangeRelay({'1': {'url': url, 'size': 10}}, 10, Path(folder)/'requests.jsonl',
                               opener=lambda *a, **k: Response(url=url))
            try:
                self.assertEqual(self.request(relay)[1], b'2345')
            finally:
                relay.close()

    def test_only_approved_public_objects_are_configurable(self):
        for url in ['http://data.baseball.computer/file','https://other.invalid/file',
                    'https://user:password@data.baseball.computer/baseball/v1/bc_publish_data/f.parquet',
                    'https://data.baseball.computer/baseball/v1/bc_publish_data/../secret']:
            with self.subTest(url=url), self.assertRaises(ValueError):
                RangeRelay({'1':{'url':url,'size':10}},10,Path('/tmp/unused-range-test.jsonl'))

if __name__=='__main__': unittest.main()
