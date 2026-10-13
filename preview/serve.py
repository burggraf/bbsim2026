"""Loopback-only preview adapter; the existing Rust CLI generates every game."""
import argparse
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
import re
import subprocess
from urllib.parse import parse_qs, urlsplit

from preview.build_viewer import build_viewer, is_joint_record

MAX_SEED = (1 << 64) - 1
ROOT = Path(__file__).resolve().parents[1]


def seed_value(text):
    if not re.fullmatch(r"[0-9]{1,20}", text) or int(text) > MAX_SEED:
        raise ValueError("Seed must be a decimal integer from 0 to 18446744073709551615")
    return int(text)


def build_live_viewer(source, seed, joint_available=False):
    seed = seed_value(str(seed))
    page = build_viewer(source)
    count = page.count('class="frame"') - 1
    controls = f'''<form class="controls" id="game-form" action="/" method="get">
<label for="seed">Seed</label><input id="seed" name="seed" value="{seed}" inputmode="numeric" pattern="[0-9]{{1,20}}" required>
<button type="submit">Generate game</button><button type="button" id="new-game">New game</button>
<button type="button" id="autoplay" aria-pressed="false">Autoplay</button>
<span id="playback-status" aria-live="polite">Paused</span></form>'''
    style = '''<style>.controls{position:sticky;top:0;z-index:10;display:flex;flex-wrap:wrap;align-items:center;gap:10px;padding:14px 0;background:#101c25}
input,button{font:inherit;border:1px solid #466373;border-radius:8px;padding:10px;color:#e9f0f2;background:#2a414f}input{width:230px}button{cursor:pointer}button:hover{filter:brightness(1.2)}button:focus-visible,input:focus-visible{outline:3px solid #ffda77;outline-offset:3px}button:disabled{opacity:.5;cursor:default}#new-game{background:#b6eb91;color:#142719}#playback-status{color:#a6e4bd;font-size:13px}.frame{scroll-margin-top:130px}</style>'''
    if joint_available:
        joint = is_joint_record(source)
        selector = ('<label for="model">Model</label><select id="model" name="model">'
                    f'<option value="default"{"" if joint else " selected"}>Default pooled ground policy</option>'
                    f'<option value="joint"{" selected" if joint else ""}>Joint source-shape experiment</option>'
                    '</select>\n')
        controls = controls.replace('<button type="submit">', selector + '<button type="submit">', 1)
        style = style.replace('input,button{', 'input,button,select{').replace(
            'input:focus-visible{', 'input:focus-visible,select:focus-visible{').replace(
            '</style>', '#model{max-width:100%}</style>')
    script = '''<script>
(() => {
  const total = TOTAL;
  const loadedSeed = BigInt("SEED");
  const form = document.getElementById('game-form');
  const seedInput = document.getElementById('seed');
  const play = document.getElementById('autoplay');
  const status = document.getElementById('playback-status');
  let timer = null;
  const current = () => {
    const match = /^#frame([0-9]+)$/.exec(location.hash);
    return match ? Math.min(Number(match[1]), total) : 0;
  };
  function stop() {
    if (timer !== null) clearInterval(timer);
    timer = null;
    play.textContent = 'Autoplay';
    play.setAttribute('aria-pressed', 'false');
    status.textContent = current() === total ? 'Game over' : 'Paused';
  }
  function sync() {
    play.disabled = current() === total;
    if (play.disabled) stop();
  }
  document.getElementById('new-game').addEventListener('click', () => {
    const next = loadedSeed === 18446744073709551615n ? 0n : loadedSeed + 1n;
    seedInput.value = next.toString();
    form.requestSubmit();
  });
  play.addEventListener('click', () => {
    if (timer !== null) { stop(); return; }
    if (current() === total) return;
    play.textContent = 'Pause';
    play.setAttribute('aria-pressed', 'true');
    status.textContent = 'Playing · one play per second';
    timer = setInterval(() => {
      location.hash = 'frame' + Math.min(current() + 1, total);
      sync();
    }, 1000);
  });
  document.addEventListener('click', event => {
    if (event.target.closest('a[href^="#frame"]')) stop();
  });
  window.addEventListener('hashchange', sync);
  document.addEventListener('visibilitychange', () => { if (document.hidden) stop(); });
  form.addEventListener('submit', () => {
    stop();
    status.textContent = 'Generating in Rust…';
    for (const button of form.querySelectorAll('button')) button.disabled = true;
  });
  sync();
})();
</script>'''.replace('TOTAL', str(count)).replace('SEED', str(seed))
    page = page.replace('</head>', style + '</head>')
    page = page.replace('</header>', '</header>' + controls)
    return page.replace('</body>', script + '</body>')


def make_server(binary, baseline, port=0, contact_counts=None):
    binary, baseline = Path(binary).resolve(), Path(baseline).resolve()
    contact_counts = Path(contact_counts).resolve() if contact_counts is not None else None

    class Handler(BaseHTTPRequestHandler):
        def setup(self):
            super().setup()
            self.connection.settimeout(10)

        def respond(self, code, body):
            data = body.encode('utf-8')
            self.send_response(code)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(data)))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.send_header('Cross-Origin-Resource-Policy', 'same-origin')
            self.send_header('Content-Security-Policy', "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'")
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            hosts = {f'127.0.0.1:{self.server.server_port}', f'localhost:{self.server.server_port}'}
            if self.headers.get('Host') not in hosts:
                self.respond(421, 'Local preview host required')
                return
            url = urlsplit(self.path)
            if url.path != '/' or url.scheme or url.netloc:
                self.respond(404, 'Not found')
                return
            params = parse_qs(url.query, keep_blank_values=True)
            try:
                allowed = {'seed', 'model'} if contact_counts is not None else {'seed'}
                if set(params) - allowed or len(params.get('seed', ['1'])) != 1:
                    raise ValueError('Supply exactly one seed')
                seed = seed_value(params.get('seed', ['1'])[0])
                model = params.get('model', ['default'])
                if len(model) != 1 or model[0] not in ('default', 'joint'):
                    raise ValueError('Supply exactly one model: default or joint')
            except ValueError as error:
                self.respond(400, str(error))
                return
            try:
                command = [str(binary), '--baseline', str(baseline), '--seed', str(seed), '--autoplay', '--trace']
                if model[0] == 'joint':
                    command += ['--contact-counts', str(contact_counts)]
                result = subprocess.run(command, check=True, capture_output=True, encoding='utf-8', timeout=30)
                page = build_live_viewer(result.stdout, seed, joint_available=contact_counts is not None)
            except (OSError, subprocess.SubprocessError, ValueError) as error:
                print(f'Generation failed: {type(error).__name__}', flush=True)
                self.respond(500, 'Could not generate game. Check the local server terminal. <a href="/">Try again</a>')
                return
            self.respond(200, page)

    return HTTPServer(('127.0.0.1', port), Handler)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=ROOT / 'engine/target/debug/examples/generated_game')
    parser.add_argument('--port', type=int, default=8765)
    parser.add_argument('--contact-counts', type=Path, help='Enable optional joint source-shape model selection')
    args = parser.parse_args()
    if not args.binary.is_file() or not args.baseline.is_file():
        parser.error('Build generated_game and supply an existing local baseline CSV')
    if args.contact_counts is not None and not args.contact_counts.is_file():
        parser.error('Supply an existing local contact-counts CSV')
    with make_server(args.binary, args.baseline, args.port, args.contact_counts) as server:
        print(f'Ready: http://127.0.0.1:{server.server_port}/ — local Rust preview', flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == '__main__':
    main()
