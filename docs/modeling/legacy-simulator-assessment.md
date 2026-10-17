# Recovered legacy simulator assessment — 2026-10-17

## What these repositories do

`../burggraf-baseball` prepares historical Retrosheet/Chadwick event data and player,
league, lineup and location summaries. `../burggraf-bb` uses those summaries to
select individual batter–pitcher outcomes and display complete simulated games.
Assumed use: one person running an offline historical game, not a shared production
service. Technology age is **not** a negative in this assessment.

**Recommendation:** retain the current Rust transition/accounting core as the rules
base; use the legacy simulator as a working product reference and comparison
baseline, and reuse its statistical extraction ideas after reconciliation. Do not
adopt its game controller or probability calculation unchanged. Do not continue
the current pooled-game implementation unchanged either: the next authorized
implementation must address individual batter/current-pitcher probabilities and
measure player-level results before adding more mechanics or UI.

This is an assessment, not migration approval or a claim that the hybrid is quick.
There is no agreed implementation budget, time cap, statistical acceptance threshold
or permission to start that next implementation.

## What already works and is worth keeping

- The legacy **complete-game path actually uses individual profiles**, unlike the
  current complete `text_game` command. It selects batter outcome rates against the
  pitcher's throwing hand, pitcher rates against the batter's effective batting
  hand, and a corresponding league split; switch hitters bat opposite the pitcher.
  The same pattern supplies out-type and single-location descriptions.
- The app database is a compact offline artifact: `bb.db` is 17,287,168 bytes. It
  contains 7,969 player-year rows, 8,711 roster rows, 29,156 lineups, 18,232 outcome
  profiles and 18,262 out profiles. Core player/profile/lineup tables cover 2012–2017.
  Raw `baseball.db.data` has a broader 1921–2017 year range; that is not proof of
  complete coverage or playable profiles for every intervening season. No 2019
  legacy profile set was found in these inspected tables.
- The **2017 persisted counts are useful, not rubbish**. Independent grouping of
  the raw database reproduced all eight counts in each of the 3,120 non-league
  persisted outcome profiles, with zero mismatches. Every corresponding stored
  percentage matched integer-truncated counts per thousand. This does not establish
  complete PA/BF denominators, but it does establish the inspected summaries' origin.
- Schedule → historical game → named lineup/pitcher selection → Play Ball → inning
  line score, R/H/E totals and batter-by-batter log is an already working product
  flow. I exercised it in the restored browser preview, not just by reading HTML.
  Pregame lineup/pitcher editors and player split/detail screens are useful references.
- The existing browser-restoration helpers avoid the old cloud configuration and
  expose SQLite read-only. Their tests passed; they are separate from the original
  simulator and do not validate its baseball algorithms.

## Main flow and source map

| Stage | Relevant source | What it establishes |
|---|---|---|
| Raw conversion/import | `burggraf-baseball/import_scripts/script{,2,3,4}.sh` | Chadwick `cwevent`, season CSVs, roster/SQLite imports; scripts were inspected, not run. |
| Profile preparation | `create_players.js`, `create_league_averages.js`, `fix*.js`, `sql.txt`, `fill_outs{,2}.sql` | Multiple historical SQLite and CouchDB/PouchDB preparation paths, not one demonstrated fresh rebuild. |
| Opening lineups | `get_opening_day_lineups.js` and app `db.service.js::getGameInfo` | Historical lineup identities/order and selected starting pitcher. |
| Data loading | `www/scripts/common/services/db.service.js`, `home/home.controller.js` | Joins rosters/profiles, maps profile keys, loads chosen teams and lineups. |
| Outcome sampling | `game/game.controller.js:426–601` | Actual participants/handedness, eight-category matchup weights and random selection. |
| Consequences/endings | `game/game.controller.js:108–413` | Mutable inning/out/base/score updates; fixed runner advancement. |
| Presentation | `home/home.html`, `chooselineup`, `choosepitcher`, `showplayer`, `game/game.html` | Setup/detail tools, final score and HTML play log. |

The eight outcome categories are HR, triple, double, single, walk, HBP, strikeout
and out. In preparation, `out` includes raw event codes **2 + 18 + 19**: ordinary
outs, reached-on-error and fielder's choice. Walk code 14 excludes intentional walk
code 15. Thus these rates are not eight clean official terminal-PA categories.

`create_players.js` has a hard-coded 2017 location query and starts with 2017;
its fielder/runner routines query events but do not estimate usable fielding/running
ratings. `create_league_averages.js` operates on year documents in CouchDB. The
exploratory runner-destination queries in `sql.txt` are not the app's runner policy.
The recovered files do not yet establish an end-to-end reproducible rebuild of the
entire delivered SQLite database. No importer, repair, installer or cloud script
was executed. `install.sh` is not a harmless inspection command.

## Must fix before using the legacy simulator as the accuracy base

### 1. The matchup formula uses the wrong probability scale

Location: `game/game.controller.js:507–601`; preparation scale in `sql.txt:544–553`.

- **What this is:** Each category goes through binary log5 arithmetic, then the
  resulting weights are rounded and normalized by cumulative sampling.
- **Problem:** Inputs such as 21, 23 and 11 mean rates per thousand. `log5` multiplies
  nonzero inputs by another ten, then uses `1 - B`, `1 - P`, `1 - L`. These
  complements require fractions from zero to one. Taking the absolute value hides
  negative results and near-pole explosions.
- **Fix:** Use consistent fractional units and test finite nonnegative weights,
  normalization and a league-neutral opponent identity. Then evaluate the
  multi-category matchup model; a unit correction alone is not calibration.
- **If skipped:** The output can be dominated by a numerical artifact instead of
  player ability. This is not a merely theoretical low-opportunity edge case.

Verified real-profile example: **Howie Kendrick versus Christian Bergman**, 2017
right/right splits. Kendrick's profile has 234 selected events, Bergman's 129. Their
HBP rates are 21/1000 and 23/1000; league is 11/1000. The actual controller assigns
HBP weight **526,470**, versus 4,819 for out, and samples HBP with **98.1766%**
probability. The same binary-log5 arithmetic in fractional units, followed by the
same category normalization, gives about **4.2228%**. That comparison diagnoses the
unit defect; **4.2228% is not a validated real-world matchup probability**.

A fixed scan of 624,023 roster-compatible 2017 handedness profile pairs found no
nonfinite actual weights in that finite set, but that does not make the scale
correct. Mean total-variation distance from the fractional-unit diagnostic was
0.073742; 57,771 pairs put at least 90% on one category. Sparse profiles contribute
to these extremes, so the last count must not be described as 57,771 independently
proved bad matchups. The Kendrick/Bergman case establishes serious distortion
without relying on one-event profiles.

### 2. Missing and sparse profiles get arbitrary behavior

Location: `game.controller.js:426–453,507–601`; loading in `db.service.js:513–553`.

- **What this is:** Missing records and zero rates feed the same sampler as observed
  rates; absent/zero category values are replaced with `1` rather than estimated
  from exposure.
- **Problem:** With all three profiles absent, the actual function assigns **12.5%
  to each outcome**, including HR, triple and HBP. Missing-record diagnostics do not
  stop the game. There is no opportunity-aware shrinkage.
- **Fix:** Check profile coverage before play; use an explicit, documented sparse-data
  policy with counts and uncertainty. Refuse unhandled missing roles instead of
  silently making a uniform distribution.
- **If skipped:** Rarely used players and incomplete imports can produce nonsense
  while appearing to be individual statistical simulations.

The raw 2017 grouping contains 3,264 role/hand profile keys; 144 are absent from
persisted outcomes (122 batting, 22 pitching), totaling 242 selected events. Small
exposure explains why this is easy to miss, but does not justify the runtime
fallback. The active historical setup loads player metadata and outcome/out records
separately; possessing a player row therefore does not establish role/hand profile
coverage. An older combined `getPlayers` query is not this active setup path.

### 3. Errors and fielder's choice have been converted into outs

Location: preparation `sql.txt:136–143` and analogous updates;
`game.controller.js:108–173,276–413`; `game/game.html:43–79`.

- **What this is:** The UI displays an Errors column, and the source contains real
  error/FC events.
- **Problem:** Those terminal categories are pooled into `out` before sampling; the
  controller retires the batter for generic outs. Team errors are initialized to
  zero and never incremented in this controller. Out-type labels do not supply
  separate runner/fielding consequences.
- **Fix:** Preserve error and FC categories through preparation and consequences,
  with explicit actors, runner destinations and independent stat tests.
- **If skipped:** Errors remain cosmetic zeroes and historical reaches are modeled
  as retirements. This changes opportunities, innings, runs and individual results.

The raw 2017 source has 1,521 error terminal events and 491 FC events. Their presence
in raw data does not mean the running simulator supports them. All 1,000 audited
legacy games had zero recorded errors; this is consistent with the source code,
not evidence of excellent fielding.

### 4. Runner movement is fixed station-to-station, not a running model

Location: `game.controller.js:276–413`.

- **What this is:** Hits, walks and HBP move named runners with direct base updates.
- **Problem:** Singles advance everybody one base, doubles two, triples three, and
  HR clear the bases. Outs/K hold existing runners. Sampled out-type labels are not
  used by the runner function: no operational DP, FC, sacrifice/tag-up, throw-out
  or speed/arm policy.
  There are no sampled steals, pickoffs, WP, PB or balks in this game path.
- **Fix:** Reuse recorded runner-destination information to estimate conditional
  decisions, and let one rules authority apply lawful joint consequences. No second
  independent hit-to-out draw should be added.
- **If skipped:** A runner on second never scores on a single; a runner on first
  never scores on a double. Runner skill and defensive differences cannot matter.

Constructed probes confirmed the actual closures' loaded-base behavior for all eight
sampled categories. The raw database contains the missing variation: with a runner
on second on a 2017 single, destination code 3 occurs 1,844 times and code 4 occurs
2,657 times, with other destinations also present. These are descriptive raw counts,
not a fitted running policy, and destination/third-out/scoring context needs decoding
before estimating a probability.

### 5. Non-HR walkoffs can count runs after the winner

Location: `game.controller.js:164–183,259–274,353–397`.

- **What this is:** The controller checks a bottom-half ending after applying the
  full PA's advances.
- **Problem:** Constructed bottom-nine tie, runners on second/third, double: both
  runners score and the home score increases by two, though the non-HR game should
  stop at the winning run. The controller has no official hit-value/stop treatment.
- **Fix:** Resolve the winning non-HR endpoint before applying additional runs and
  final scoring; preserve the all-runs exception for an out-of-park HR.
- **If skipped:** Some final scores and player credits are wrong.

A separate direct probe shows `isGameOver` returns true for home ahead during top
nine with zero outs. **This is not reported as an ordinary-game premature-ending
bug:** its actual top-half caller waits until the half has finished. The source
condition alone would have overstated the finding.

## Should fix if these components are reused

### 6. Starting pitchers remain in for the whole game

Location: `game.controller.js:137–142,426–453`; setup `home.controller.js`/`choosepitcher`.

- **What this is:** Pregame pitcher selection supplies the matchup's opposing pitcher.
- **Problem:** The game loop never changes that pitcher. Stored pitch-count tables
  do not drive substitutions or fatigue in this path. Pregame selection is not
  relief management.
- **Fix:** Use an explicit finite pitcher cast and current-pitcher lookup on every
  PA; validate relief profile coverage too.
- **If skipped:** Starter-only opponent distributions distort both batter and pitcher
  expectations, even after fixing the formula.

### 7. Duplicate league out profiles make lookup ambiguous

Location: `bb.db.outs`; `db.service.js:647–667`; `fill_outs{,2}.sql`.

- **What this is:** League out-type rows supply a handedness-specific reference.
- **Problem:** Every inspected 2012–2017 season has five `leagueLL` rows and two
  `league` rows. In 2017 the LL rows contain four different sets of values. Loading
  assigns the same key repeatedly without an explicit query order. The usual
  delivered order ends with the matching LL row, but uniqueness is not enforced.
- **Fix:** Reconcile/rebuild unique role/season/split keys and reject duplicates.
- **If skipped:** A changed query order or rebuilt database can silently select the
  wrong out-type reference. This is separate from the outcome-scale defect.

### 8. The working interface lacks a tested simulation boundary

Location: `game.controller.js`, empty `game.service.js`, `game/game.html`.

- **What this is:** One controller loads league data, samples, mutates game state,
  scores and constructs HTML for a synchronous complete-game loop.
- **Problem:** The seven Python and seven Node restoration tests cover transport,
  security/startup and assets, not matchup identities, baseball rules or individual
  fidelity. `Math.random` has no recorded replay seed. HTML log text is not an
  immutable before/after/stat record.
- **Fix:** Put tests around the actual outcome and transition boundary before reuse;
  do not introduce a second state reducer merely to adapt the screen.
- **If skipped:** A believable-looking final game can conceal statistical or scoring
  regressions with no reproducible case.

## Display/layout assessment

The setup screen is compact and useful: Away/Home panels, real player names,
positions, pitchers and a clear Play Ball action. The game screen has an inning
line score, R/H/E totals and a readable ordered log with named batters and shorthand
such as `F8`, `G6`, `S G9`, `K`. The browser run finished CHN 3–SLN 0, hits 13–3,
errors 0–0; screenshots are retained. That is functionality evidence, not accuracy.

The game screen is a **finished-game result**, not a live per-PA control surface.
The whole game executes synchronously on entry. `Next` returns to choosing another
game. It does not display a persistent named base-state panel, full batting/pitching
box score, or the actual current matchup probabilities. Runner log messages occur
for implemented moves, but are not complete state snapshots. The separate animate
folder is a prototype, not evidence of an integrated ball/runner animation engine.

Reuse the product flow and information hierarchy if desired; do not mistake the
Errors column, location labels or animation assets for implemented mechanics.
Escaping/import trust and old cloud/native integrations were not comprehensively
security-audited; use the safe local restoration preview, not raw cloud configuration.

## Current versus legacy, without a stack-age penalty

| Requirement | Recovered legacy | Current project |
|---|---|---|
| Individual batter/pitcher sampling in a complete game | **Present**, including handedness; serious scale/sparse-data defects | **Absent from complete `text_game`**; pooled initial PA rates. Separate nine-category rating prototype is not integrated/validated. |
| Player-level historical fidelity and believable variance | Not established | Not established |
| Source counts | Valuable reconciled 2017 role/hand profiles; incomplete selected-event denominator; rebuild gaps | Audited 2019 source/opportunity work; participant proxies still not complete official PA/BF |
| Rules/state/accounting authority | Mutable UI controller; major omissions above | Canonical immutable Rust transitions, identities and explicit deltas; much stronger rule/accounting evidence, still incomplete |
| Runners/errors/compound events | Mostly absent/fixed/collapsed | Much broader lawful mechanisms; many generation policies are pooled/provisional, not calibrated |
| Pitcher changes | Pregame only, starter all game | Deterministic finite workload cast; not verified historical eligibility/fatigue |
| Reproduction | Unrecorded `Math.random`; seeded only by this audit harness | Explicit seed/model/rules data boundaries and native replay/checks |
| Usable historical selection UI | Existing schedule, lineup and player-detail workflow | Preview/text paths exist, but the requested integrated rated game remains missing |

The current prototype's fixed shrinkage (`kappa=200`) and normalized batter ×
pitcher ÷ league weights are not a proven replacement just because they use valid
fractional units. It also lacks handedness and full relief coverage. Neither repo
has earned a player-calibration or predictive-accuracy claim.

## Base decision and smallest useful next milestone

1. **Do not wholesale replace the current engine with the legacy controller.** It
   would discard useful rules/accounting guarantees and reintroduce known failures;
   repairing the formula alone leaves errors, running, relief and endings incomplete.
2. **Do not use current test totals as a reason to continue the same work plan.**
   The legacy app is ahead on the central product connection: player statistics
   actually influence its finished games. The current project has not delivered that.
3. **Use a narrow hybrid:** current canonical rules authority; legacy statistical
   extraction/handedness/product-flow ideas and preserved baseline; one explicit,
   validated matchup producer connected to the complete-game path. No UI migration
   or additional rare-event expansion is needed to test that central proposition.
4. **Before coding, agree a spending/time cap and stop criteria.** Then preregister
   quantitative player-event, opportunity-conditioned and ensemble-variation checks
   before fitting. Cover the real current batter and pitcher, including all allowed
   relievers, and account explicitly for the nine-versus-ten-category boundary.
5. Deliver unfiltered named rated games with per-PA profile/probability/selection
   evidence, plus repeated-game individual means and variation against explicitly
   declared hindsight targets/opponent mix. Distinguish sampling variation from
   uncertainty in estimated ability; report small exposures and failed metrics.
   Do not call selected-season reconstruction predictive accuracy.

No threshold was fitted, no probability was tuned, and no implementation was started
for this assessment. If that bounded deliverable cannot meet its agreed checks,
stop rather than diverting back into presentation or rare rules.

## Checks, reproduction and preservation

Evidence root: `/tmp/bbsim-legacy-assessment-v1/` (local temporary storage, **not a
remote/durable publication**). In addition to this report and `docs/PROGRESS.md`,
no current-project implementation files were intentionally changed by the assessment.
Both legacy folders/databases were treated read-only; no install/dependency changes.

- `proc_ff2e`: read-only database inspection, exit 0; `inspect_data.py`,
  `data-inspection.json`, `database-schemas.json`, `fixture.json`.
- `proc_e710`: both existing suites, exit 0: **7 Python + 7 Node tests**, zero failures:
  ```sh
  cd /Users/markb/dev/burggraf-bb
  PYTHONDONTWRITEBYTECODE=1 python3 -B -W error::ResourceWarning -m unittest -v test_preview
  node --test test_preview_db.cjs
  ```
  Raw `existing-python.{stdout,stderr}`, `existing-node.{stdout,stderr}` retained.
- `proc_9137`: actual-controller characterization, exit 0; **1,000/1,000** games on
  first 2017 fixture `ARI201704020`, audit seeds 1–1000, **75,424** selected events,
  no caught failures/missing-profile diagnostics in this fixture. Means were 3.4165
  runs and 8.926 hits per team-game. A repeated single fixture is **not** a season,
  fairness, player-fidelity or calibration evaluation.
  ```sh
  node /tmp/bbsim-legacy-assessment-v1/probe_engine.cjs
  ```
  The VM exposes existing controller closures, records selections/weights and
  substitutes an audit-only seeded random stream. It does not repair the algorithm.
  Its external per-game timeout is an audit guard, not a production ending rule.
  Full probes, per-seed results, player outcomes and first-seed events/log retained
  in `engine-probes.json`, `fixture-seed1-events.json`, `fixture-seed1-log.html`.
- `proc_f1ef`: the first probe attempt failed because the temporary database stub did
  not implement `.then(...).catch(...)`. This was a **harness interface failure**,
  not a legacy simulation failure; original stdout/stderr were retained.
- `proc_6bb5`: independent 2017 count/percentage reconciliation, exit 0:
  ```sh
  python3 -B /tmp/bbsim-legacy-assessment-v1/reconcile_2017.py
  ```
  `2017-reconciliation.json` retains all comparisons, missing keys and runner counts.
- Browser: schedule/setup/full original game exercised against the loopback read-only
  preview; `legacy-setup.png`, `legacy-game.png`, `browser-game.json` and
  `browser-final-state.json` retained. Assessment server `proc_66ae` was stopped
  deliberately after use; unrelated previews/browser sessions were not closed.
- `proc_c647`: repeat characterization and preservation checks, exit 0. Printed
  characterization summary reproduced byte-for-byte. **107 selected authored files**
  matched initial SHA-256 pins; both database sizes/mtime-nanoseconds were unchanged.
  `preservation-and-repeat.json` records the result. This is not a before/after full
  hash proof for the large raw database.

Verdict: the older project contains a real, useful statistical product prototype,
not a better ready-to-trust simulation core. Correct the project's priority by
proving integrated individual outcomes next; retain the stronger current rules core.

Not checked: fresh full database rebuild, every old season/script/vendor file,
legacy native/mobile/cloud execution, historical roster eligibility, full official
scoring, production security, season-level individual calibration or predictive
validity. Current Rust/Python suites were not rerun for this read-only assessment;
their previously recorded recovery passes do not validate either matchup model.
