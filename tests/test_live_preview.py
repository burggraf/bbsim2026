"""Real loopback HTTP and Rust CLI integration; no mocked game outcomes."""
from contextlib import contextmanager
from pathlib import Path
import json
import re
import subprocess
import tempfile
import threading
import unittest
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from preview.serve import build_live_viewer, make_server, seed_value

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "engine/target/debug/examples/generated_game"


@contextmanager
def running(binary, baseline):
    server = make_server(binary, baseline)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        thread.join(timeout=5)
        server.server_close()
        if thread.is_alive():
            raise RuntimeError("Preview test server did not stop")


class LivePreviewTests(unittest.TestCase):
    def test_exact_seed_domain_is_not_float_or_modulo_coercion(self):
        for text in ["0", "1", "42", "18446744073709551615"]:
            self.assertEqual(seed_value(text), int(text))
        for text in ["", "-1", "1.2", "1e3", " 1", "+1", "18446744073709551616", "１"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                seed_value(text)

    def test_live_controls_reuse_record_and_preserve_exact_large_seed(self):
        from test_visible_viewer import RECORD
        page = build_live_viewer(RECORD, 18446744073709551615)
        self.assertIn('id="seed"', page)
        self.assertIn('value="18446744073709551615"', page)
        self.assertIn('id="new-game"', page)
        self.assertIn('id="autoplay"', page)
        self.assertIn('id="frame3" data-away="0" data-home="1"', page)
        self.assertIn('BigInt', page)

    def test_real_rust_generation_is_repeatable_and_different_seed_changes_plays(self):
        self.assertTrue(BINARY.is_file(), "Build generated_game before running integration tests")
        rust = (ROOT / "engine/tests/generated_preview.rs").read_text()
        csv = json.loads(re.search(r'const CSV: &str = (".*");', rust).group(1))
        with tempfile.TemporaryDirectory() as directory:
            baseline = Path(directory) / "counts.csv"
            baseline.write_text(csv)
            expected = {}
            for seed in [1, 2]:
                expected[seed] = subprocess.check_output(
                    [str(BINARY), "--baseline", str(baseline), "--seed", str(seed), "--autoplay", "--trace"],
                    encoding="utf-8", timeout=30)
            plain = subprocess.check_output(
                [str(BINARY), "--baseline", str(baseline), "--seed", "1", "--autoplay"],
                encoding="utf-8", timeout=30)
            traced = expected[1]
            without_trace = traced[:traced.index('TRACE FORMAT:')] + traced[traced.index('Play 01 |'):]
            self.assertEqual(without_trace, plain)
            self.assertEqual(traced.count('TRACE PLAY 1 BEGIN'), 1)
            self.assertIn('Before state:', traced)
            self.assertIn('After state:', traced)
            plays = lambda text: text[text.index("Play 01"):]
            self.assertNotEqual(plays(expected[1]), plays(expected[2]))
            with running(BINARY, baseline) as url:
                def get(seed):
                    with urlopen(url + f"/?seed={seed}", timeout=30) as response:
                        self.assertEqual(response.headers["Cache-Control"], "no-store")
                        return response.read().decode()
                self.assertEqual(get(1), build_live_viewer(expected[1], 1))
                self.assertEqual(get(1), get(1))
                self.assertEqual(get(2), build_live_viewer(expected[2], 2))

    def test_bad_routes_seeds_and_foreign_hosts_are_rejected_before_engine(self):
        with running(Path("/nonexistent/binary"), Path("/nonexistent/baseline")) as url:
            for route, code in [("/secret", 404), ("/?seed=-1", 400),
                                ("/?seed=1&seed=2", 400), ("/?seed=", 400),
                                ("/?seed=18446744073709551616", 400), ("/?other=1", 400)]:
                with self.subTest(route=route):
                    with self.assertRaises(HTTPError) as error:
                        urlopen(url + route, timeout=5)
                    with error.exception as response:
                        self.assertEqual(response.code, code)
            request = Request(url + "/?seed=1", headers={"Host": "foreign.example"})
            with self.assertRaises(HTTPError) as error:
                urlopen(request, timeout=5)
            with error.exception as response:
                self.assertEqual(response.code, 421)

    def test_engine_failure_is_visible_not_a_fabricated_game(self):
        with running(Path("/nonexistent/binary"), Path("/nonexistent/baseline")) as url:
            with self.assertRaises(HTTPError) as error:
                urlopen(url + "/?seed=1", timeout=5)
            with error.exception as response:
                self.assertEqual(response.code, 500)
                self.assertIn(b"Could not generate game", response.read())
