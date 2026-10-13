"""Loopback-only, capped public Parquet range relay; not a production importer."""
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import re
import threading
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener


class Budget:
    def __init__(self, cap):
        if not isinstance(cap, int) or cap <= 0:
            raise ValueError('Positive byte cap required')
        self.cap, self.reserved, self.received = cap, 0, 0
        self.lock = threading.Lock()

    def reserve(self, n):
        if not isinstance(n, int) or n <= 0:
            raise ValueError('Positive reservation required')
        with self.lock:
            if self.reserved + n > self.cap:
                return False
            self.reserved += n
            return True

    def receive(self, n):
        with self.lock:
            self.received += n
            assert self.received <= self.reserved <= self.cap


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ValueError('Origin redirects prohibited')


class RangeRelay:
    def __init__(self, objects, cap, log, *, opener=None):
        self.objects = {str(key): dict(value) for key, value in objects.items()}
        for key, item in self.objects.items():
            url = urlsplit(item['url'])
            if (not re.fullmatch(r'[0-9]+', key) or url.scheme != 'https'
                    or url.netloc != 'data.baseball.computer' or url.query or url.fragment
                    or not re.fullmatch(r'/baseball/v1/bc_publish_data/(?:[a-zA-Z0-9_-]+/)*[a-zA-Z0-9_-]+\.parquet', url.path)
                    or not isinstance(item['size'], int) or item['size'] <= 0):
                raise ValueError('Only pinned approved public objects allowed')
        self.budget = Budget(cap)
        self.log = log
        self.etags = {}
        self.lock = threading.Lock()
        self.opener = opener or build_opener(NoRedirect()).open
        relay = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_HEAD(self):
                self.serve(head=True)

            def do_GET(self):
                self.serve(head=False)

            def serve(self, *, head):
                if self.headers.get('Host') != relay.base_url.removeprefix('http://'):
                    self.send_error(421)
                    return
                item = relay.objects.get(self.path.removeprefix('/'))
                if item is None:
                    self.send_error(404)
                    return
                if head:
                    self.send_response(200)
                    self.send_header('Content-Length', str(item['size']))
                    self.send_header('Accept-Ranges', 'bytes')
                    self.end_headers()
                    return
                match = re.fullmatch(r'bytes=(\d+)-(\d+)', self.headers.get('Range', ''))
                if not match:
                    self.send_error(416)
                    return
                start, end = map(int, match.groups())
                if not 0 <= start <= end < item['size']:
                    self.send_error(416)
                    return
                length = end - start + 1
                if not relay.budget.reserve(length):
                    self.send_error(429)
                    return
                record = dict(url=item['url'], start=start, end=end, bytes=0, status='failed')
                digest, headers_sent = hashlib.sha256(), False
                try:
                    request = Request(item['url'], headers={'Range': f'bytes={start}-{end}',
                                      'Accept-Encoding': 'identity', 'User-Agent': 'bbsim2026-range-audit/1.0'})
                    with relay.opener(request, timeout=30) as response:
                        expected_span = f"bytes {start}-{end}/{item['size']}"
                        etag = response.headers.get('ETag', '')
                        if (response.status != 206 or response.geturl() != item['url']
                                or response.headers.get('Content-Range') != expected_span
                                or response.headers.get('Content-Length') != str(length)
                                or response.headers.get('Content-Encoding', 'identity').lower() != 'identity'
                                or not (etag.startswith('"') and etag.endswith('"'))):
                            raise ValueError('Origin status/range/size/encoding/ETag mismatch')
                        with relay.lock:
                            if relay.etags.setdefault(item['url'], etag) != etag:
                                raise ValueError('Origin object ETag changed')
                        record['etag'] = etag
                        self.send_response(206)
                        self.send_header('Content-Length', str(length))
                        self.send_header('Content-Range', expected_span)
                        self.send_header('ETag', etag)
                        self.end_headers()
                        headers_sent = True
                        while record['bytes'] < length:
                            chunk = response.read(min(65536, length - record['bytes']))
                            if not chunk:
                                raise ValueError('Truncated origin range')
                            relay.budget.receive(len(chunk))
                            record['bytes'] += len(chunk)
                            digest.update(chunk)
                            self.wfile.write(chunk)
                        record['status'] = 'complete'
                except Exception as error:
                    if isinstance(error, HTTPError):
                        error.close()
                    record['error_type'] = type(error).__name__
                    if not headers_sent:
                        self.send_error(502)
                finally:
                    record['sha256'] = digest.hexdigest()
                    with relay.lock:
                        with relay.log.open('a') as stream:
                            stream.write(json.dumps(record, sort_keys=True) + '\n')
                    self.close_connection = True

        self.server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        self.server.daemon_threads = False
        self.base_url = f'http://127.0.0.1:{self.server.server_port}'
        self.thread = threading.Thread(target=self.server.serve_forever)
        self.thread.start()
        self.closed = False

    def close(self):
        if not self.closed:
            self.server.shutdown()
            self.server.server_close()
            self.thread.join()
            self.closed = True
