"""Real loopback HTTP and Rust CLI integration; no mocked game outcomes."""
from contextlib import contextmanager
from pathlib import Path
import html
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
def running(binary, baseline, contact_counts=None, lineup=None):
    if lineup is not None:
        server=make_server(binary,baseline,contact_counts=contact_counts,lineup=lineup)
    else:
        server = (make_server(binary, baseline) if contact_counts is None else
                  make_server(binary, baseline, contact_counts=contact_counts))
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


def joint_inputs(root):
    """Self-contained accepted logical fixture; no external dataset required."""
    rust = (ROOT / 'engine/tests/joint_contact_preview.rs').read_text()
    joint = re.search(r'const JOINT_CSV: &str = r#"(.*?)"#;', rust, re.DOTALL).group(1)
    fields = dict(line.split(',', 1) for line in joint.splitlines()[1:])
    categories = ['strikeout', 'walk', 'hbp', 'single', 'double', 'triple', 'home_run', 'roe', 'contact_out']
    baseline = ('key,value\nformat,preview-counts-v1\ncatalog_sha256,' + fields['catalog_sha256']
                + '\nquery_version,preview-baseline-v1\nselected_pa,186517\n'
                + 'excluded_ibb,753\nexcluded_interference,61\n'
                + ''.join(key + ',' + fields[key] + '\n' for key in categories))
    paths = root/'baseline.csv', root/'joint.csv'
    paths[0].write_text(baseline)
    paths[1].write_text(joint)
    return paths


class LivePreviewTests(unittest.TestCase):
    def test_rated_selector_and_prerequisites(self):
        from test_player_ratings import rating_record
        source=rating_record()
        page=build_live_viewer(source,1,joint_available=True,real_available=True,rated_available=True)
        self.assertIn('<option value="rated" selected>',page)
        self.assertEqual(page.count(' selected>'),1)
        server=None
        try:
            with self.assertRaises(ValueError):server=make_server('/missing','/missing',ratings='/missing')
        finally:
            if server is not None:server.server_close()
        with self.assertRaises(ValueError):build_live_viewer(source,1,joint_available=True,rated_available=True)

    def test_rated_http_matches_native_and_keeps_pooled_comparison(self):
        from preview import player_ratings,real_lineup
        from test_player_ratings import report
        from test_real_lineup import witness
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);base,counts=joint_inputs(root);lineup=root/'lineup';lineup.write_text(real_lineup.encode_witness(witness()))
            ratings=root/'ratings';ratings.write_text(player_ratings.encode(report(),witness()))
            server=make_server(BINARY,base,contact_counts=counts,lineup=lineup,ratings=ratings)
            thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
            try:
                url=f'http://127.0.0.1:{server.server_port}'
                for mode in ['default','joint','real','rated']:
                    for seed in [1,2,0,18446744073709551615]:
                        cmd=[str(BINARY),'--baseline',str(base),'--seed',str(seed),'--autoplay','--trace']
                        if mode!='default':cmd+=['--contact-counts',str(counts)]
                        if mode in ['real','rated']:cmd+=['--lineup',str(lineup)]
                        if mode=='rated':cmd+=['--ratings',str(ratings)]
                        native=subprocess.check_output(cmd,text=True,timeout=30)
                        try:response=urlopen(url+f'/?seed={seed}&model={mode}',timeout=30)
                        except HTTPError as error:
                            with error:self.fail(f'Expected200: {error.code}: {error.read().decode()}')
                        with response:
                            page = response.read().decode()
                        self.assertEqual(page, build_live_viewer(native, seed, joint_available=True, real_available=True, rated_available=True))
                        count = len(re.findall(r'^Play ', native, re.MULTILINE))
                        self.assertEqual(page.count('class="frame"'), count + 1)
                        self.assertIn(f'const total = {count};', page)
                        if mode in ['real', 'rated']:
                            from test_visible_viewer import selected_frame
                            self.assertIn('data-batter="a1" data-pitcher="hp"', selected_frame(page, 0))
                            self.assertIn('<strong>Player a1</strong>', selected_frame(page, 0))
                            self.assertNotIn('data-batter=', selected_frame(page, count))
                            self.assertEqual(page.count('class="lineup-hitter"'), 18)
                            self.assertIn('<section id="starting-lineups"', page)
                            self.assertIn('aria-label="Schematic base occupancy"', page)
                            if mode == 'rated':
                                self.assertIn('<details id="player-probabilities">', page)
                ratings.write_text('bad')
                with self.assertRaises(HTTPError) as error:urlopen(url+'/?model=rated',timeout=30)
                with error.exception as response:self.assertEqual(response.code,500);response.read()
                with urlopen(url+'/?model=real',timeout=30) as response:self.assertEqual(response.code,200);response.read()
            finally:server.shutdown();thread.join(timeout=5);server.server_close()

    def test_rated_queries_reject_paths_and_unconfigured_mode(self):
        server=make_server('/missing','/missing',contact_counts='/missing',lineup='/missing',ratings='/missing')
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        try:
            url=f'http://127.0.0.1:{server.server_port}'
            for route in ['/?model=rated&model=rated','/?model=rated&ratings=/tmp/file','/?model=/tmp/ratings','/?model=rated&seed=-1']:
                with self.assertRaises(HTTPError) as error:urlopen(url+route,timeout=5)
                with error.exception as response:self.assertEqual(response.code,400);response.read()
        finally:server.shutdown();thread.join(timeout=5);server.server_close()
        with running('/missing','/missing','/missing','/missing') as url:
            with self.assertRaises(HTTPError) as error:urlopen(url+'/?model=rated',timeout=5)
            with error.exception as response:self.assertEqual(response.code,400);response.read()

    def test_ratings_startup_guard_before_binding(self):
        result=subprocess.run(['python3','-Werror','-m','preview.serve','--help'],capture_output=True,timeout=5)
        self.assertEqual(result.returncode,0);self.assertIn(b'--ratings',result.stdout)
        with tempfile.TemporaryDirectory() as d:
            base,counts=joint_inputs(Path(d))
            result=subprocess.run(['python3','-Werror','-m','preview.serve','--baseline',str(base),'--ratings','/missing','--port','0'],capture_output=True,timeout=5)
            self.assertEqual(result.returncode,2);self.assertIn(b'requires --lineup',result.stderr);self.assertNotIn(b'Ready:',result.stdout)

    def test_real_selector_requires_configuration_and_uses_native_policy(self):
        from test_visible_viewer import RECORD, joint_record, real_record
        for source,selected in [(RECORD,'default'),(joint_record(),'joint'),(real_record(),'real')]:
            page=build_live_viewer(source,1,joint_available=True,real_available=True)
            self.assertIn(f'<option value="{selected}" selected>',page)
            self.assertEqual(page.count(' selected>'),1)
            self.assertIn('<option value="real"',page)
        self.assertNotIn('<option value="real"',build_live_viewer(RECORD,1,joint_available=True))
        with self.assertRaises(ValueError):build_live_viewer(RECORD,1,real_available=True)
        with self.assertRaises(ValueError):make_server('/missing','/missing',lineup='/missing')

    def test_real_lineup_http_full_page_matches_independent_native_all_models(self):
        from preview.real_lineup import encode_witness
        from test_real_lineup import witness
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);baseline,counts=joint_inputs(root);lineup=root/'lineup.tsv';lineup.write_text(encode_witness(witness()))
            with running(BINARY,baseline,counts,lineup) as url:
                for mode in ['default','joint','real']:
                    for seed in [1,2,18446744073709551615,0]:
                        args=[str(BINARY),'--baseline',str(baseline),'--seed',str(seed),'--autoplay','--trace']
                        if mode!='default':args+=['--contact-counts',str(counts)]
                        if mode=='real':args+=['--lineup',str(lineup)]
                        native=subprocess.check_output(args,encoding='utf-8',timeout=30)
                        try:
                            response=urlopen(url+f'/?seed={seed}&model={mode}',timeout=30)
                        except HTTPError as error:
                            with error:self.fail(f'Expected200 for {mode}/{seed}, got {error.code}: {error.read().decode()}')
                        with response:
                            self.assertEqual(response.headers['Cache-Control'],'no-store')
                            self.assertEqual(response.read().decode(),build_live_viewer(native,seed,joint_available=True,real_available=True))
                default=build_live_viewer(subprocess.check_output([str(BINARY),'--baseline',str(baseline),'--seed','1','--autoplay','--trace'],encoding='utf-8',timeout=30),1,joint_available=True,real_available=True)
                with urlopen(url+'/?seed=1',timeout=30) as response:self.assertEqual(response.read().decode(),default)

    def test_real_queries_reject_paths_duplicates_and_extra_fields_before_engine(self):
        with running('/missing','/missing','/missing','/missing') as url:
            for route in ['/?model=real&model=real','/?model=/tmp/lineup.tsv','/?model=real&lineup=/tmp/file','/?model=real&seed=-1','/?model=unknown']:
                with self.subTest(route=route),self.assertRaises(HTTPError) as error:urlopen(url+route,timeout=5)
                with error.exception as response:self.assertEqual(response.code,400)
        with running('/missing','/missing','/missing') as url:
            with self.assertRaises(HTTPError) as error:urlopen(url+'/?model=real',timeout=5)
            with error.exception as response:self.assertEqual(response.code,400)

    def test_bad_real_lineup_returns_500_without_affecting_joint_or_default(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);baseline,counts=joint_inputs(root);lineup=root/'bad.tsv';lineup.write_text('bad')
            with running(BINARY,baseline,counts,lineup) as url:
                with self.assertRaises(HTTPError) as error:urlopen(url+'/?model=real',timeout=30)
                with error.exception as response:
                    self.assertEqual(response.code,500);self.assertIn('Could not generate game',response.read().decode())
                for mode in ['joint','default']:
                    with urlopen(url+'/?model='+mode,timeout=30) as response:
                        self.assertEqual(response.code,200)
                        self.assertIn(b'id="frame0"',response.read())

    def test_real_startup_help_requires_joint_and_existing_lineup_before_bind(self):
        help_result=subprocess.run(['python3','-Werror','-m','preview.serve','--help'],capture_output=True,timeout=5)
        self.assertEqual(help_result.returncode,0);self.assertIn(b'--lineup',help_result.stdout)
        with tempfile.TemporaryDirectory() as directory:
            baseline,counts=joint_inputs(Path(directory))
            command=['python3','-Werror','-m','preview.serve','--binary',str(BINARY),'--baseline',str(baseline),'--lineup',str(Path(directory)/'missing.tsv'),'--port','0']
            for suffix,error in [([],b'requires --contact-counts'),(['--contact-counts',str(counts)],b'existing local lineup TSV')]:
                result=subprocess.run(command+suffix,capture_output=True,timeout=5)
                self.assertEqual(result.returncode,2);self.assertIn(error,result.stderr);self.assertNotIn(b'Ready:',result.stdout)

    def test_configured_selector_uses_loaded_native_model_and_preserves_default_api(self):
        from test_visible_viewer import RECORD, joint_record
        plain = build_live_viewer(RECORD, 1)
        self.assertNotIn('id="model"', plain)
        page = build_live_viewer(RECORD, 1, joint_available=True)
        self.assertIn('<select id="model" name="model">', page)
        self.assertIn('<label for="model">Model</label>', page)
        self.assertIn('<option value="default" selected>', page)
        self.assertIn('<option value="joint">', page)
        joint = build_live_viewer(joint_record(), 1, joint_available=True)
        self.assertIn('<option value="joint" selected>', joint)
        self.assertNotIn('<option value="default" selected>', joint)
        self.assertIn('form.requestSubmit()', joint)
        self.assertIn('18446744073709551615n', joint)
        self.assertEqual(build_live_viewer(RECORD, 1), plain)

    def test_real_joint_http_matches_independent_cli_and_keeps_default_opt_in(self):
        with tempfile.TemporaryDirectory() as directory:
            baseline, counts = joint_inputs(Path(directory))
            def native(seed, joint=False):
                args = [str(BINARY), '--baseline', str(baseline), '--seed', str(seed), '--autoplay', '--trace']
                if joint:
                    args += ['--contact-counts', str(counts)]
                return subprocess.check_output(args, encoding='utf-8', timeout=30)
            expected_default = native(1)
            expected_joint = {seed: native(seed, True) for seed in [1, 2, 18446744073709551615, 0]}
            self.assertNotEqual(expected_joint[1], expected_default)
            self.assertNotEqual(expected_joint[1], expected_joint[2])
            with running(BINARY, baseline, counts) as url:
                def get(query):
                    try:
                        response = urlopen(url + '/' + query, timeout=30)
                    except HTTPError as error:
                        with error:
                            self.fail(f'Expected200 for {query}, got {error.code}: {error.read().decode()}')
                    with response:
                        self.assertEqual(response.headers['Cache-Control'], 'no-store')
                        return response.read().decode()
                default_page = build_live_viewer(expected_default, 1, joint_available=True)
                self.assertEqual(get('?seed=1'), default_page)
                self.assertEqual(get('?seed=1&model=default'), default_page)
                for seed, source in expected_joint.items():
                    self.assertEqual(get(f'?seed={seed}&model=joint'),
                                     build_live_viewer(source, seed, joint_available=True))
                self.assertEqual(get('?seed=1&model=joint'), get('?seed=1&model=joint'))
                self.assertEqual(get('?seed=1'), default_page)

    def test_configured_bad_model_queries_fail_before_running_missing_engine(self):
        with running(Path('/missing/binary'), Path('/missing/baseline'), Path('/missing/contact')) as url:
            routes = ['/?model=', '/?model=unknown', '/?model=joint&model=default',
                      '/?model=joint&model=joint', '/?model=/tmp/counts.csv',
                      '/?model=--contact-counts', '/?contact-counts=/tmp/counts.csv',
                      '/?model=joint&seed=-1', '/?model=joint&seed=1&seed=2',
                      '/?model=joint&other=1']
            for route in routes:
                with self.subTest(route=route), self.assertRaises(HTTPError) as error:
                    urlopen(url + route, timeout=5)
                with error.exception as response:
                    self.assertEqual(response.code, 400)
            with self.assertRaises(HTTPError) as error:
                urlopen(Request(url + '/?model=joint', headers={'Host': 'foreign.example'}), timeout=5)
            with error.exception as response:
                self.assertEqual(response.code, 421)
            with self.assertRaises(HTTPError) as error:
                urlopen(url + '/secret?model=joint', timeout=5)
            with error.exception as response:
                self.assertEqual(response.code, 404)

    def test_unconfigured_server_still_rejects_model_query(self):
        with running(Path('/missing/binary'), Path('/missing/baseline')) as url:
            for mode in ['default', 'joint']:
                with self.assertRaises(HTTPError) as error:
                    urlopen(url + '/?model=' + mode, timeout=5)
                with error.exception as response:
                    self.assertEqual(response.code, 400)

    def test_bad_joint_csv_is_visible_failure_not_silent_default_fallback(self):
        with tempfile.TemporaryDirectory() as directory:
            baseline, counts = joint_inputs(Path(directory))
            counts.write_text(counts.read_text().replace('residual,39614', 'residual,39613'))
            with running(BINARY, baseline, counts) as url:
                with self.assertRaises(HTTPError) as error:
                    urlopen(url + '/?seed=1&model=joint', timeout=30)
                with error.exception as response:
                    self.assertEqual(response.code, 500)
                    body = response.read().decode()
                    self.assertIn('Could not generate game', body)
                    self.assertNotIn('id="frame', body)
                with urlopen(url + '/?seed=1', timeout=30) as response:
                    self.assertEqual(response.code, 200)
                    self.assertIn('pooled-jeffreys-preview-v1', response.read().decode())

    def test_startup_help_exposes_opt_in_and_missing_contact_file_prevents_bind(self):
        help_result = subprocess.run(['python3', '-Werror', '-m', 'preview.serve', '--help'], capture_output=True)
        self.assertEqual(help_result.returncode, 0)
        self.assertIn(b'--contact-counts', help_result.stdout)
        with tempfile.TemporaryDirectory() as directory:
            baseline, _ = joint_inputs(Path(directory))
            result = subprocess.run(['python3', '-Werror', '-m', 'preview.serve',
                '--binary', str(BINARY), '--baseline', str(baseline), '--contact-counts',
                str(Path(directory)/'missing.csv'), '--port', '0'], capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 2)
            self.assertIn(b'existing local contact-counts CSV', result.stderr)
            self.assertNotIn(b'Ready:', result.stdout)

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
            self.assertIn('COMMENTARY FORMAT: radio-v1', traced)
            for seed, native in expected.items():
                page = build_live_viewer(native, seed)
                calls = re.findall(r'^CALL PLAY \d+: (.+)$', native, re.MULTILINE)
                self.assertGreater(len(calls), 0, f'seed {seed}')
                for call in calls:
                    self.assertIn(html.escape(call), page)
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
