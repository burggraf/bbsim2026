# Visible engine preview

A small local simulation interface, not the finished desktop game. The existing
Rust generator creates each whole game; the browser displays its recorded plays,
score, inning/outs, runners and final supplied-credit statistics. By default,
fictional equal teams use the experimental pooled 2019 baseline. The separate
Seattle–Oakland starting-lineup opt-in below retains pooled probabilities, not
real-player ratings. No managerial choices or accuracy claim yet.

The dark ballpark screen puts gameplay first: a compact team scorebug, current
half/inning/outs, schematic named base occupancy, and the displayed play's batter
and pitcher. Frame zero shows **At bat / Pitching** from the initial native state;
completed plays show their actors from the native before-state, matching the call.
A separate **Up next** line uses the selected after-state's lineup cursor, including
pitcher changes across half innings. Scores, outs and bases remain after-play
snapshots. Final frames retain the last play's actors but advertise no next batter
and do not fabricate three stored outs.

Seattle–Oakland modes show both nine-hitter batting orders once, with separate
starting pitchers and an **Up next** indicator that also works without JavaScript.
The line score shows only the selected prefix: unplayed halves are dashes, not
invented zeroes; future runs/innings are not revealed. It reconciles to the supplied
native final line score. Probability, limitations/provenance and full-log panels
start closed below gameplay; the verbatim raw trace and final statistics remain
available. This is still a generated exhibition with coarse advancement, starters
staying all game and partial supplied-credit statistics—not a realism milestone.
The diamond is schematic, not ball flight, contact geometry or factual fielder
positions. No pitch counts, player statistics or runner outcomes are invented.

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
summary with an explicit notice that detailed before/after snapshots are absent.
Older summaries/toy traces without any native team fields explicitly show current
matchup and prefix line score unavailable. Present-but-partial/malformed native
team metadata is rejected rather than guessed. The bounded extractor accepts the
current generated `engine-debug-v1` state shape (closed starting orders, empty
bench/removed lists), not arbitrary Rust Debug text or the canonical codec:


```sh
cargo run --manifest-path engine/Cargo.toml --example generated_game -- \
  --baseline /tmp/bbsim-preview-2019-counts.csv --seed 1 --autoplay --trace \
  > /tmp/bbsim-game-record.txt
python3 preview/build_viewer.py /tmp/bbsim-game-record.txt /tmp/bbsim-game-preview.html
open /tmp/bbsim-game-preview.html
```

## Optional recorded Seattle–Oakland starting lineup

This bounded experiment uses the [audited2019-03-20 starting witness](../docs/data-audits/2019-lineup-readiness.md),
not that game's recorded outcomes. Source player IDs/order/pitchers/DH/positions
initialize Rust **before** simulation; this is not renaming fictional players
after a game. All20 players still use the same pooled joint probabilities.
No player fitting, effective matchup hands, park effects or availability model.
Closed starting roster: no substitutions or eligible bench/bullpen inference;
starting pitchers stay all game, an explicitly unrealistic limitation.

First reproduce the accepted audit report and joint counts outside Git. The
exporter accepts only the pinned audit report, retains provenance/attribution,
checks input drift and exclusively publishes complete bytes; existing outputs
and repository outputs are refused. No new source query or download:

```sh
lineup_dir=$(mktemp -d /tmp/bbsim-seattle-oakland.XXXXXX)
python3 -Werror -m preview.real_lineup \
  /tmp/bbsim-lineup-readiness/report-first.json "$lineup_dir/starting.tsv"
cargo build --offline --locked --manifest-path engine/Cargo.toml --example generated_game
python3 -Werror -m preview.serve --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup "$lineup_dir/starting.tsv" --port 8765
```

The configured-only **Seattle–Oakland starting-lineup experiment** selector adds
`model=real`; no query still selects the default. Generate/New game preserve the
selected mode and seed semantics. **Recorded starting lineups and positions**
is visible below the field, with nine batting slots/positions/names per
team and a separate starter. Source IDs are not displayed in the normal game view. The native upcoming batter is marked **Up next**;
that indicator clears at Final. Base labels show supplied names with accessible
base/occupant text; identities remain distinct even when names match. Native radio
calls and recaps show supplied names, without bracketed source IDs. A caught
strikeout is called as strike three, without catcher-credit bookkeeping; runners
who do not advance on an inning-ending play are described as stranded, not holding.
Air and ground contact calls include the actual supplied lineup position (for
example, “fly ball to LF, caught by …” or “ground ball to SS, fielded by …”).
Position labels describe the modeled fielding assignment, not measured ball
coordinates, depth or a player's usual position. Without supplied positions,
commentary omits the location rather than guessing.
Raw technical states/metadata/statistic keys retain IDs. Expand the
assumptions/provenance panel for the complete Rust preamble.

Deterministic coarse actors: catcher2 receives K; shortstop6 throws to first
baseman3 on ground outs and receives ROE; center8/right9/third5 take held
fly/line/pop catches. These are **modeled assignments**, not historical locations,
throw histories, range/arm or fielding skills. Residual outs keep Unknown review
and never invent a fielder/contact type. No player-probability/accuracy/complete
box or production manager/gate claim; default and fictional joint modes remain.
Input paths are operator startup settings, never request fields. `--lineup`
requires joint counts; malformed runtime inputs fail visibly, never fall back.

Server-free native inspection uses the same producer:

```sh
engine/target/debug/examples/generated_game \
  --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup "$lineup_dir/starting.tsv" --seed 1 --autoplay --trace \
  > "$lineup_dir/game.txt"
python3 preview/build_viewer.py "$lineup_dir/game.txt" "$lineup_dir/game.html"
```

A supplied report digest in a manually edited TSV does not authenticate its
rows. The pinned exporter plus frozen artifacts/source reconciliation bind this
local experiment to the audit. Do not present arbitrary edited inputs as audited
lineups. Data/exports/pages stay outside Git and must not be redistributed.
[Bounded contract](../docs/modeling/real-lineup-preview.md) fixes the checks;
[live status](../docs/PROGRESS.md) distinguishes pending versus verified work.
Stop the temporary server with Ctrl-C when finished.

## Optional player-specific probability pilot (same matchup)

The separate **Seattle–Oakland player-probability pilot** adds league-shrunk
batter/pitcher participant-proxy effects. This uses full2019 hindsight, not
forecasting or calibrated abilities. The same real starting roster and positional
policy remain; starters stay all game, with no substitutions/fatigue/handedness/
park/fielding-skill model. The original **starting-lineup experiment** remains the
pooled comparison. There is no second fielding roll.

Export the accepted opportunity report and lineup witness to a new external file:

```sh
ratings_dir=$(mktemp -d /tmp/bbsim-player-ratings.XXXXXX)
python3 -Werror -m preview.player_ratings \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-lineup-readiness/report-first.json "$ratings_dir/ratings.tsv"
cargo build --offline --locked --manifest-path engine/Cargo.toml --example generated_game
python3 -Werror -m preview.serve --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup /tmp/bbsim-real-lineup/lineup-first.tsv \
  --ratings "$ratings_dir/ratings.tsv" --port 8765
```

Open `http://127.0.0.1:8765/?model=rated&seed=1`. Choose the player pilot or pooled
starting-lineup experiment with the same seed. Expand the initially closed
**Player-specific probabilities — hindsight pilot** panel below gameplay for
18 batter/starter matchup distributions and20 role
posterior means/component SDs, included participant counts and data weights.
N is **not official PA/BF**. Fixed prior strength200 per role is provisional,
not tuned or proven best; sparse/zero counts shrink toward positive pooled rates.
Generalized-log5 combines independent marginals, retaining opponent/park/defense
confounding. Component SDs are not validated ability intervals or propagated
matchup uncertainty. Expand native provenance for unrounded values and source
pins. Probability displays are supplied by Rust; the browser only renders them.

`--ratings` requires lineup plus joint counts, and malformed inputs fail visibly;
request fields never accept paths. Exported counts/digests do not authenticate
manually edited rows: use the pinned exporter and source reconciliation. No data
or generated pages are bundled/redistributable. Native inspection also accepts
`--ratings "$ratings_dir/ratings.tsv"` alongside the earlier real-lineup command.
[Preregistered formula/checks](../docs/modeling/player-probability-preview.md) and
[live status](../docs/PROGRESS.md) distinguish implementation from verified work.
Stop the local server with Ctrl-C when finished. This is the bounded milestone;
manager/roster expansion and production/accuracy gates remain separate.

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
