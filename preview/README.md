# Visible engine preview

A small local simulation interface, not the finished desktop game. The existing
Rust generator creates each whole game; the browser displays its recorded plays,
score, inning/outs, runners and final supplied-credit statistics. Fictional equal
teams use the experimental pooled 2019 baseline. No real-player ratings,
managerial choices or accuracy claim yet.

## Run the interactive preview

From the repository root, with the previously audited internal baseline present:

```sh
cargo build --manifest-path engine/Cargo.toml --example generated_game
python3 -m preview.serve --baseline /tmp/bbsim-preview-2019-counts.csv --port 8765
```

Open **http://127.0.0.1:8765/**. Keep that terminal running; Ctrl-C stops it.
Use `--port 8766` if the port is occupied. The baseline is not bundled or silently
downloaded; [baseline reproduction](../docs/modeling/run-generated-preview.md)
describes the existing internal data path.

### Optional joint-contact experiment

First reproduce the accepted external26-field CSV using the
[joint sampler instructions](../docs/modeling/joint-contact-preview.md). Then:

```sh
cargo build --offline --locked --manifest-path engine/Cargo.toml --example generated_game
python3 -m preview.serve --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv --port 8765
```

The **Model** selector appears only with that startup setting. Default pooled
ground policy remains the default; choose **Joint source-shape experiment** and
Generate game to opt in. Generate/New game use the selected mode; Restart replays
it without a new draw. Executable/baseline/contact paths are startup settings,
never browser inputs. Requests accept only the seed and, when configured, one
`model=default` or `model=joint`; bad/missing joint data shows a generation error,
never a silent default fallback. No dataset download or additional dependencies.

Joint games prominently disclose source-shape mixture/synthetic held retirements
and partial residual coverage; expand **Joint-contact assumptions and supplied
provenance** for the verbatim Rust preamble, versions, counts and Retrosheet notice.
Radio calls identify fly balls, line drives and pop-ups **only** from explicit
named native catch facts. Unknown residual outs retain generic calls; no catch,
throw, position, range/arm or historical certainty is inferred from endpoints or
putout counts. All calls/recaps/full snapshots still originate in Rust. The
[bounded integration contract](../docs/modeling/joint-contact-viewer.md) fixes
acceptance; [PROGRESS](../docs/PROGRESS.md) records current verification status.
This does not authorize real-player fitting or complete-box/accuracy/gate claims.

- **Generate game:** uses the entered decimal seed. Same seed + same baseline/
  engine/model versions reproduces the same game. Supports the exact full
  unsigned 64-bit range, including seeds above JavaScript's safe Number range.
- **New game:** generates with the loaded game's seed + 1 (maximum wraps to 0).
- **Next play / Previous:** step through the recorded game.
- **Autoplay / Pause:** display one recorded play per second; no browser outcomes.
- **Restart:** replay this same game from the beginning, not reroll it.
- **Final result:** jump to the final score, line score and supplied statistics.
- **Full game log:** below the field, closed by default. Open it for a scrolling
  list of every play, with a concise radio-style call naming the batter, pitcher,
  moving/scoring/held runners, and involved fielders when recorded. A compact
  before/after recap gives inning, outs, score and base occupants. The same call
  appears beside the field during playback. Fictional player IDs are identities,
  not actual MLB names; unrecorded hit locations/fielders and strike types are not
  invented. Expand any entry's **Full native state and event facts** for complete
  immutable snapshots, pitcher charges, lineups/cursors, pitcher/DH/bench
  state, runner moves, outs/runs/credits and all supplied event facts. The entire
  game is available independently of playback position; no missing states are
  guessed. After states normalize half-inning boundaries (outs reset to zero);
  the event still records the out(s) and scoring on the play.

Manual navigation stops autoplay. Playback stops at game end and pauses when
its tab is hidden. New generation resets playback. Errors are displayed instead
of fabricated games. The standard-library server binds only to loopback, serves
no arbitrary files, rejects foreign Host headers and accepts only a seed plus the
fixed optional model enum when explicitly configured. Input paths are startup
settings, not browser inputs. This is a development prototype, not a production web server or release.
No new dependencies or simulation-core/sampler changes.

## Optional server-free playback

The original static viewer remains available, without JavaScript or a server.
`--trace` adds the detailed native log; without it, older records show their full
summary with an explicit notice that detailed before/after snapshots are absent:

```sh
cargo run --manifest-path engine/Cargo.toml --example generated_game -- \
  --baseline /tmp/bbsim-preview-2019-counts.csv --seed 1 --autoplay --trace \
  > /tmp/bbsim-game-record.txt
python3 preview/build_viewer.py /tmp/bbsim-game-record.txt /tmp/bbsim-game-preview.html
open /tmp/bbsim-game-preview.html
```

## Verification

```sh
cargo fmt --manifest-path engine/Cargo.toml --check
cargo build --manifest-path engine/Cargo.toml --example generated_game
cargo test --manifest-path engine/Cargo.toml
cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings
python3 -Werror -m unittest discover -s tests -v
```

HTTP integration tests run the real compiled Rust generator using the repository's
synthetic test baseline, not mocked outcomes or an external dataset. They check
complete-page reproduction, distinct actual play streams for different seeds,
seed bounds, request validation, loopback host restrictions and visible engine
failure. Browser checks additionally cover real New Game/Generate, exact maximum
seed and wrap-to-zero, playback/pause/restart, and automatic terminal stopping.

With the audited baseline, seed 1 has 75 plays/Away 3–Home 12; seed 2 has 89 plays/
Away 9–Home 3. These are exploratory simulations, not historical matchup evidence.
The optional `engine-debug-v1` trace is a human-inspection format, not the
canonical codec or independent validation certificate. Native trace tests compare
all full snapshots/summary fields to the actual Rust transitions, including
inning boundaries and the terminal play. Separate `radio-v1` commentary comes
from those same native transitions; original debug bodies are unchanged. Tests
cover all nine preview results, names/fielding facts, all base masks/outs/halves,
held runners, uncredited home crossings, inning endings and terminal recaps.
Older debug-only records retain their original summaries; malformed or partial
commentary is rejected. Traced/untraced game/stat output agrees.
Statistics remain supplied-credit totals, not complete official boxes; unknowns
remain `?`. The viewer recognizes only the current complete terminal format and
rejects malformed/incomplete records; it does not replace canonical validation.

Baseline, generated pages and logs stay outside Git. Public redistribution rights
remain unresolved. Accurate engine work is **retained and deferred**, not abandoned;
production Tauri/Svelte plans and E-03/G1/G2 requirements remain unchanged.
See [the live ledger](../docs/PROGRESS.md) for evidence and the next checkpoint.
