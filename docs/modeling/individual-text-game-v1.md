# Individual-stat text-game integration v1

Preregistered 2026-10-17 before implementation or new diagnostics. Maintainer
approved continuing the current Rust engine and selected a **60-minute initial
work cap** in the interview. Work starts at approximately 14:51 UTC; stop/report
by 15:51 UTC, sooner if blocked. No UI work, new rare events, dependencies, source
transfer, model tuning, release or gate signoff.

## Smallest deliverable

Connect the existing provisional nine-category Dirichlet/generalized-log5 model
to the complete `text_game`, using actual current batter and opposing current
pitcher, including the explicitly supplied 12-pitcher cast. Reuse the cast TSV's
existing 18 batter and 12 pitcher count vectors; no new exporter/database needed.
Retain the prior standalone starter-only preview and pooled mechanism-test API.

The existing model remains fixed: kappa200 per role, league-centered zero exposure,
normalized posterior_batter * posterior_pitcher / league. No claim it is optimal,
calibrated, handedness-adjusted or predictive. Inputs are selected terminal-event
participant proxies, not complete official PA/BF. No silent missing-profile fallback.

## Nine-to-ten categories and sampling

Ratings cover K, non-IBB BB, HBP, 1B, 2B, 3B, HR, ROE and non-hit contact. Keep the
existing tenth catcher-interference parent **pooled** at (61+.5)/(185764+5).
Scale all nine matchup probabilities together by 1 minus that mass; do not drop
or redistribute ROE/contact mass. Tactical IBB remains a manager override and
runner-only events remain non-PA. Existing contact/runner conditionals unchanged.
Sample the rated parent with one SplitMix64 word using its high 53 bits and a
validated cumulative distribution; no retry, second hit-to-out roll or uniform
fallback. Record the actual probability vector supplied at each ordinary PA,
with current named actors and corresponding event; distinguish tactical IBB.

## Observable acceptance (before implementation)

- Exact required role set: starting batting orders plus every permitted cast
  pitcher; reject missing/extra/invalid/overflow/out-of-cohort vectors before play.
- Pure controls: zero exposure is league-centered; batter HR and pitcher K changes
  move the corresponding probability in the expected direction; all ten categories
  are finite, strictly positive, normalized; invalid arrays refuse.
- Current-state lookup changes at a pitcher substitution; a reliever-only profile
  change cannot alter the seeded game's pre-entry prefix. Missing current roles
  refuse rather than use the starter/league/uniform distribution silently.
- Actual full-game selection depends on ratings, not only printed diagnostics;
  fixed words exercise CDF boundaries and consume one word per parent. Preserve
  pooled-path determinism and existing rules/accounting checks.
- Ordinary real-personnel seeds **1,2,3**, unfiltered, complete and reproduce.
- Fixed real-personnel cohort **seeds1..=1000**, all games retained, no expanding
  range/retuning after results: complete games, canonical continuity, native
  accounting, no missing ratings. Compare each ordinary PA's recorded participants
  and vector with independent Python arithmetic from original source counts.
- For each hitter/pitcher and category, report O (selected outcomes), E (sum of
  conditional probabilities), V (sum p*(1-p)) and |O-E|/sqrt(V). A value over5
  flags a failed sampling diagnostic; it is not a player-fidelity acceptance test.
  Report per-game hitter PA/outcome means, SDs and empirical quantiles including
  zero-opportunity/zero-result games. Nonzero variation is only a mechanism check,
  not evidence that the amount of variation is realistic.
- Separately report hitter raw rates, posterior rates, and matchup expectations
  averaged over the 2019 pitcher profile pool weighted by selected exposure.
  Disclose opportunity sizes and discrepancies without fitting. That pool is a
  diagnostic opponent mix, not each hitter's historical schedule or a predictive
  evaluation. No individual historical-fidelity gate is awarded from this pilot.
- Run all applicable Rust tests (including examples), fmt, strict all-target Clippy,
  example build, full Python discovery with ResourceWarnings as errors, and diff
  checks. Preserve semantic reds, harness failures, raw outputs and failing seeds.

## Scope and stop conditions

Expected source reach: shared player-rating loader, cast count exposure, complete
sampler/CLI, regression tests, this report and progress ledger. No canonical event
schema/reducer, UI or old-repository change. If full verification or diagnostics
cannot finish within the cap, leave an explicit incomplete checkpoint, not a
completion claim. Numerical diagnostic failures stay visible; do not tune kappa
or conditional probabilities to hide them. Further calibration/model comparison
needs a new preregistered evaluation and subsequent scope decision.

## Implementation and observed results (2026-10-17)

The complete CLI now consumes the existing cast vectors, not only names and
pitcher workload. `Ratings::from_counts` shares the prior loader's cohort bounds
and arithmetic, while the CLI requires exactly18 hitter and12 supplied pitcher
roles. Missing/extra profiles refuse before simulation. An explicit zero-count
profile is league-centered, not treated as a missing identity. The standalone
starter-only preview and pooled `simulate` mechanism-test path are retained.

The shared full-game loop calls the matchup provider only at an ordinary PA's
parent draw, after runner-only and tactical-IBB decisions. A retained `PaDraw`
records the exact vector, raw word and selected category against its event number;
the renderer does not reconstruct that choice from the credited hit value. No
canonical schema/reducer, manager policy or contact/runner conditional changed.
Cast metadata/shape/bounds checks are not cryptographic authentication of arbitrary
edited TSVs. The actual-source audit separately reconciles all30 vectors with the
pinned original opportunity report.

### Fixed real-personnel cohort

`proc_9573` exited0 in63s. **All seeds1..1000**, no selection/expansion/tuning:

| Check | Observed result |
| --- | --- |
| Completed games | 1000/1000 |
| Retained native events | 86,042 |
| Individual ordinary PA draws | 77,551 |
| Tactical IBB / runner-only events | 38 / 1577, neither consumes a PA-model draw |
| Pitching changes | 6876 |
| Current pitchers actually observed | all12 supplied pitchers |
| Exact role coverage | all18 batters +12 pitchers; source vectors reconcile |
| Independent current-actor/vector/word/parent checks | every ordinary PA |
| Largest Rust versus independent Python probability difference | 1.1102230246251565e-16 |
| Native continuity/accounting | executed and passed for every complete game |
| Preregistered standardized-residual flags (absolute value>5) | none across300 role/category cells |
| Largest absolute standardized residual | 2.6281515, Ryan Buchter singles allowed |
| Variation mechanism | all18 hitters have nonzero per-game PA and HR variation |

The normalized parent test is not a test of official box-score calibration.
Walkoff hit credits can shorten; intentional walks are excluded from ordinary
parent outcomes. Standardized residuals, especially for sparse cells, are not
Gaussian p-values or proof of realistic variance.

Unfiltered ordinary examples and repeat runs:

| Seed | Seattle–Oakland | Events | Ordinary draws |
| --- | --- | --- | --- |
| 1 | 6–5 | 98 | 89 |
| 2 | 2–6 | 83 | 76 |
| 3 | 5–7 | 85 | 75 |

Each stdout/stderr pair reproduces byte-for-byte. These are modeled exhibitions,
not reconstructions of the historical opening game's result.

### Means and variation: useful diagnostics, not fidelity acceptance

The report retains **every** role/category O/E/V/residual, all18 hitters' per-game
means/SDs/minima/5th/50th/95th percentiles/maxima (zero-result games included), and
all nine raw/posterior/opponent-pool probabilities. Illustrative HR results:

| Hitter | Source selected opportunities | Raw HR% | Posterior HR% | 2019 opponent-pool HR% | Mean PA/game (SD) | Mean selected HR/game (SD) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Dee Strange-Gordon | 420 | 0.714 | 1.661 | 1.646 | 4.798 (0.710) | 0.083 (0.280) |
| Mitch Haniger | 282 | 5.319 | 4.626 | 4.589 | 4.696 (0.691) | 0.237 (0.466) |
| Jay Bruce | 332 | 7.831 | 6.259 | 6.197 | 4.587 (0.686) | 0.289 (0.525) |
| Matt Chapman | 670 | 5.373 | 4.977 | 4.936 | 4.569 (0.660) | 0.198 (0.451) |
| Ichiro Suzuki | 6 | 0.000 | 3.543 | 3.513 | 3.924 (0.696) | 0.136 (0.368) |

The opponent pool comprises831 pitching profiles, weighted by185,703 selected
exposures; it is not each hitter's actual historical schedule. Its largest
absolute departure from the corresponding hitter posterior is0.00269951 across
all18×9 cells. That near-neutrality does **not** validate the posterior itself.
The Dee/Ichiro examples expose important unresolved shrinkage/sparse-prior behavior,
not successes to hide in league totals. Kappa200 is unchanged, not fit to this
cohort. No age/career/split prior, opponent/park/defense adjustment, predictive
validation, propagated ability uncertainty or realistic-variance acceptance is
established. **Individual outcome connection is implemented; the broader historical
player-fidelity requirement is not complete.**

### Reproduction and retained failures

No new input export is needed. The same command now uses the cast abilities:

```sh
cargo build --offline --locked --manifest-path engine/Cargo.toml --examples
engine/target/debug/examples/text_game \
  --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup /tmp/bbsim-real-lineup/lineup-first.tsv \
  --cast /tmp/bbsim-pitcher-workload/cast-first.tsv \
  --seed 1
```

#### Readable radio transcript

Add `--format radio` for play calls instead of diagnostic output:

```sh
engine/target/debug/examples/text_game \
  --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup /tmp/bbsim-real-lineup/lineup-first.tsv \
  --cast /tmp/bbsim-pitcher-workload/cast-first.tsv \
  --seed 1 --format radio | less
```

This shows inning headings, named batters, pitching changes, runner advances and
holds, scoring and the final result, followed by an inning-by-inning line score
with team **R** (runs), **H** (hits), and **E** (errors). All extra innings are
shown; `0` means a played scoreless half and `X` means the home team's final half
was not played. Totals reuse canonical game accounting (errors belong to the
fielding team), not a second scorer or a complete-official-box certification.

A standard **box score** follows: each team's batting lineup in batting order
(PA, AB, R, H, RBI, BB, SO), pitchers in entry order (IP, H, R, ER, BB, SO, HR,
BF), and team totals. IP is baseball notation: `5.2` means five innings and two
outs, not 5.2 decimal innings. Inherited runs stay charged to the responsible
pitcher. Notes list recorded 2B/3B/HR, IBB, HBP, SH/SF, GDP, SB/CS, CI, errors,
passed balls, wild pitches and balks when present. BB includes intentional walks;
IBB is a subset, not an additional walk. Unknown scoring cells remain `?`,
including team totals affected by them. Fully observed, completed halves with
explicit whole-chance no-misplay review now have numeric ER even with sacrifices,
runner advances or pitching changes; inherited runs stay with their responsible
pitcher. Zero errors alone is insufficient: passed balls and unsupported
error/interference histories still require reconstruction. Some automatic
uncaught-third-strike/held-runner events retain unknown whole-chance review;
this correction does not invent that missing review even with no E/PB credits.
See the shared
[accounting contract](../engine/accounting.md). This does not invent official AB/RBI/ER,
pitcher wins/losses/saves, pitch counts, or season averages. Names and all values
come from the retained game and canonical accounting, not the source game's box.

Hits use field locations (for example,
“Chad Pinder: Single to left field.”), without repeating the batter's implied
arrival on base. Play calls name fielders only for recorded putouts or assists;
redundant runner-out lines after double-play calls are omitted. Extra-base
throw-outs remain explicit. Probability vectors, raw random numbers, source
hashes and before/after state dumps are omitted. A short plain-language
development caveat remains; the data attribution is kept here rather than in the
radio transcript:

> The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".

It is a presentation of the **same generated game**, not a different model or
another random draw. `--format detailed` (or no format option) retains the
verbose diagnostic format. Invalid/duplicate format options are refused.
The local `/tmp` input files above are still required; they are not in a clone.

Raw evidence and independent harness: `/tmp/bbsim-individual-text-game-v1/`.
`diagnostics.json` has source/input/binary pins and every metric; `cohort/` retains
all1000 stdout/stderr pairs. To repeat without overwriting that evidence:

```sh
python3 -B /tmp/bbsim-individual-text-game-v1/diagnostics.py \
  /tmp/NEW_EMPTY_INDIVIDUAL_AUDIT_DIRECTORY
```

The harness and evidence are local external artifacts, not remotely durable. It
uses the fixed audited local inputs and fixed1000 seeds, refuses an existing
cohort directory, and changes no rates or engine results.

Retained reds/failures: `ignored-profiles-red.stderr` proves changing a real
hitter's HR vector originally had no effect on the complete game (exit1);
`ignored-profiles-green.*` is its passing rerun. `api-red.stderr` records missing
interfaces, `focused-first.stderr` an introduced local name shadowing `draw`, and
`clippy-first.stderr` tuple complexity/format-string lints. The interrupted first
full check additionally found a test-only nonexistent `HomeRun` credit variant;
the correct existing oracle is `Hit` plus `OutOfParkHomeRun`. These are explicit
implementation/harness failures, not hidden baseball or statistical failures.

### Pre-review verification (boundary correction follows)

`proc_52d6` exited0 in661s (15:12:32–15:23:31UTC), using
`PYTHONDONTWRITEBYTECODE=1 python3 -B /tmp/bbsim-individual-text-game-v1/verify-v2.py`.
Every individual command/exit/timestamp is retained in
`verification-v2/results.json`, with unfiltered separate stdout/stderr:

| Exact check | Result |
| --- | --- |
| `cargo build --offline --locked --manifest-path engine/Cargo.toml --examples` | pass |
| `cargo test --offline --locked --manifest-path engine/Cargo.toml --test full_player_game` | all4 new tests pass |
| `cargo test --offline --locked --manifest-path engine/Cargo.toml` | 856 passed,61 zero-failure summaries |
| `cargo test --offline --locked --manifest-path engine/Cargo.toml --examples` | 1 passed,four summaries |
| `cargo fmt --manifest-path engine/Cargo.toml --check` | pass |
| `cargo clippy --offline --locked --manifest-path engine/Cargo.toml --all-targets -- -D warnings` | pass |
| `python3 -B -W error::ResourceWarning -m unittest discover -s tests -v` | 611 passed in60.852s |
| `git diff --check` | pass |

All112 Rust-source/Cargo hashes matched before/after and at that inspection. The
compiled example/input pins also stayed fixed through both pre-review audits.
Those passes do not certify the subsequent boundary correction below.

A separate same-cohort reproducibility run `proc_04d8` exited0 in87s with the
output-directory-capable harness. Every one of1000 stdout/stderr pairs and every
scientific report cell matches the first run. The only report differences are
completion timestamp and added harness digest; `repeat-equality.json` retains
all2000 raw hashes and both script/report hashes. The cohort was repeated, not
expanded; no failed seed/rate was discarded or changed.

### Independent review and complete-cast correction

Fresh read-only Sol-high reviewer `d6e03ff3` confirmed current-state lookup,
interference scaling, draw placement/receipts and explicit pooled compatibility,
but found **P1: removing a terminal cast member plus all its profile/workload rows
allowed a shortened cast**. Existing shared manager subsets are intentional; the
complete CLI failed to require the full pinned membership. Verified against actual
source and a real CLI semantic red: `truncated-red.stderr` exit1, with the wrongly
accepted game's original stdout/stderr retained. New API red is
`complete-cast-api-red.*`.

The minimal correction adds `Cast::read_complete_tsv`, checking both pinned team
lengths after the existing contiguous identity/name/profile validation, and uses
it at the complete CLI boundary. Shared `read_tsv` subset behavior remains.
The synthetic rating fixture now has all12 pitchers/30 vectors. A fifth regression
removes either team's last pitcher **and all associated rows**, confirms the old
subset loader remains compatible, and requires the complete loader to refuse.
All5 focused tests pass (`complete-cast-focused.*`); the real CLI red is green
with no game output (`truncated-green.*` / `truncated-cast-post-review.stderr`).
Parent disposition: valid P1, corrected; not a statistical-means finding.

Same fixed1000-seed post-review audit `proc_520c` exited0/64s on the newly built
binary. All1000 stdout/stderr pairs and every scientific metric are unchanged;
`post-review-equality.json` retains the2000 hashes. Only completion time, harness
digest and rebuilt binary hash differ from the first report. Binary SHA256 is
`79043e49ff4772764fb2313cdbdd2d49961b78b7ca71c878f57958c8aa8103f1`.
**Fresh post-review full verification `proc_99cf` exited0/573s**,
15:33:51–15:43:22UTC, using the same eight exact check commands above via
`PYTHONDONTWRITEBYTECODE=1 python3 -B /tmp/bbsim-individual-text-game-v1/verify-v3.py`.
Current results: **857 Rust tests/61 zero-failure summaries;5 focused regressions;
1 example/four summaries;611 Python tests in57.267s;examples build,fmt,strict
all-target Clippy and diff clean.** All112 Rust/Cargo hashes match before/after
and at final inspection; the audited compiled example hash also remains exact.
`verification-v3/` retains complete command/exit/timestamp receipts, source pins
and unfiltered per-command stdout/stderr. These are the final corrected-source
checks, not the superseded pre-review receipts.

The initial work checkpoint stopped/reported within its60-minute cap. The
review's P1 is parent-verified corrected with retained red/green evidence and the
fresh complete suite; no second independent-review verdict is implied. All
checkpoint processes finished; unrelated previews remain untouched.
No P-01/P-02 completion, G1–G5 signoff, commit, push or publication is claimed.
The next scientific task is preregistered player-level mean/variance adequacy
evaluation and sparse/shrinkage treatment, not more UI or rare-event machinery.
