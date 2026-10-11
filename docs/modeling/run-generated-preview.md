# Run a generated game (experimental preview)

This is no longer a scripted sequence: the seed selects varied PA outcomes from
a smoothed, pooled 2019 baseline. Teams are still fictional/equal, advancement
and contact outs are coarse, and statistics remain supplied-credit views.
It is **not calibrated player simulation, complete official boxes or gate signoff**.
[Preregistered contract](generated-preview.md); [live status/results](../PROGRESS.md).

## On the current development machine

The derived baseline is outside Git at `/tmp/bbsim-preview-2019-counts.csv`.
From the repository root:

```sh
cargo run --manifest-path engine/Cargo.toml --example generated_game -- \
  --baseline /tmp/bbsim-preview-2019-counts.csv --seed 1
```

Enter advances one play; q/EOF stops display. To run the whole game:

```sh
cargo run --manifest-path engine/Cargo.toml --example generated_game -- \
  --baseline /tmp/bbsim-preview-2019-counts.csv --seed 1 --autoplay
```

Change `--seed 1` to `--seed 2` for a different game. Identical counts and
seed/model/policy/RNG versions reproduce the same log **with this engine revision**.
The checkpoint and verification retain the exact engine/query source hashes. The header prints all nine
counts and probabilities. Unknown ER displays `?`, not a fabricated zero.
The prior `scripted_game` example remains available for deterministic debugging.

## Reproduce baseline and native checks

Requires the existing verified offline F-03 DuckDB snapshot and installed
DuckDB CLI; there is no network access or automatic full-season download.
Keep source and derived files outside Git. Use the appropriate snapshot path:

```sh
duckdb -readonly -csv /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  < audits/2019-preview-baseline.sql > /tmp/preview-counts.staging.csv && \
  mv /tmp/preview-counts.staging.csv /tmp/bbsim-preview-2019-counts.csv
cargo run --manifest-path engine/Cargo.toml --example preview_diagnostics -- \
  /tmp/bbsim-preview-2019-counts.csv
```

The SQL rejects unsupported seasons, duplicate grains, missing PA events and
unknown PA classifications. It reports intentional-walk/interference exclusions
rather than quietly reclassifying them. Counts are a small derivative, not a
production importer/ratings snapshot; licensing/release holds remain unchanged.
Retrosheet attribution from the source: “The information used here was obtained
free of charge from and is copyrighted by Retrosheet. Interested parties may
contact Retrosheet at www.retrosheet.org.”

## Limitations you should expect

- K is caught; other-contact outs use one coarse ground-out/SS→1B chain.
  Fly/line/pop types, DP, sacrifices, FC tactics and realistic advancement await
  separate modeling. No fielding roll converts a decided hit to an out.
- No team/player differences, park/handedness, substitutions, fatigue, steals,
  WP/PB/balks or deliberate IBB yet. Unimplemented legal histories are not declared
  impossible or waived from production requirements.
- Ordinary walk-off scoring uses a conservative supported hit-value/runner policy
  because current records lack winning-run order. Out-of-park HR keeps its exception.
- ER, scorer completeness and some official box fields are still unproven.
  Batch scoring summaries are exploratory, not model accuracy or thresholds to tune.
- A 10,000-event resource ceiling fails explicitly; it never forces an out/winner,
  retries a failed seed, or silently excludes a bad game.
