# F-01-season-replay-readiness — offline 2019 prerequisite

## Scope and observable checks (before new measurements)

Maintainer approved the sequence: all-team coverage audit → minimal headless
historical-lineup season runner → pinch hitting → defensive replacement policy.
This task measures readiness using existing local snapshots; it does not claim
S-01–S-03/G1–G5 signoff, historical roster eligibility or predictive validation.
No new downloads, dependencies, outcome/ratings changes, UI work or push.

Inputs: accepted F-03 snapshot SHA256
`d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`
and detail supplement SHA256
`554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82`.
Read via native DuckDB READ_ONLY, external access disabled, no startup script.
Raw exports, reports, before/after hashes and failures stay outside Git.

Acceptance fixed before measurements:

- Reconcile **every** schedule row against starts/results, preserving original
  dates, postponement/makeup text, doubleheaders and unmatched/ambiguous keys.
  No assumption that 30×162/2 equals actual games; no silent four-game exclusion.
  Match explicit source dimensions; unknown date/remap meanings remain blockers.
- Reuse existing lineup-readiness validation over all games. Report DH/non-DH,
  source exception flags and reasoned per-game candidates, not just a count.
  Actual appearances are controlled historical casts, not pregame active rosters.
- Inventory participant profiles and starting-player coverage; distinguish raw
  season counts from training-only rating support and runtime export support.
  Inspect existing loader/caller restrictions rather than removing safeguards.
- Identify cached batting/pitching/fielding reference coverage and missing credits;
  do not substitute participant PA proxies for official season totals. Check
  player/team grains, missing values and traded-player aggregation requirements.
  Same-lineage derived totals are not independent external season validation.
- Synthetic regression tests cover schedule cardinality, doubleheaders, unmatched
  rows, unknowns, postponements and duplicate identities before audit behavior.
  Native repeated exports/reports must be byte-equal; databases unchanged.
- Run all applicable Python checks and diff validation. With no Rust/runtime edits,
  record existing Rust verification as historical, not a new test pass.
- Deliver explicit readiness/blocker dispositions and one concrete implementation
  slice; never label a partial game set a complete MLB season.

Initial owned paths: this report, `docs/PROGRESS.md`, a small audit module/query
and its tests. Reuse existing read-only lineup/personnel/probability audits.
Any runnable season implementation is a subsequent bounded slice with its own
contract/tests; discovering missing data is not permission to invent it.

## Measured results — 2026-10-17

Bounded audit complete. **GO for generalizing historical game inputs; NO-GO for
claiming a complete season runner, official player-stat comparison or season
fidelity.** No game has been simulated by this task. The first input-generalization
slice need not wait for an autonomous roster/manager or official scoring completion.

Implementation: [read-only query](../../audits/2019-season-readiness.sql),
[audit/CLI](../../audits/season_readiness.py),
[seven regression tests](../../tests/test_season_readiness.py). Existing lineup
validation is reused unchanged. Reports retain every original schedule row,
candidate game IDs, all games/results, personnel checks, role exposures and
player/team-grain reference values with explicit unknown-row counts. Ambiguous
edges are retained, not greedily assigned; no postponed game inherits its original
DH number as evidence of makeup order. Reports are exclusive, outside Git and
source-pinned. They are inventories, not production rating/roster files.

### Schedule and game coverage

- **2,430 original schedule slots; 2,429 observed games; all 30 teams.**
  2,363 slots match uniquely; 66 have ambiguous source-slot-to-DH-game mapping;
  one is explicitly not made up. Zero unmatched/unknown-date slots.
- The omitted game is the **2019-07-02 DET at CHA** slot, whose source makeup text
  is literally `20190927 - Not made up`. It is not an inferred unplayed game.
- All 2,429 actual IDs occur in the candidate graph. The 66 unassigned IDs are
  present games across 33 doubleheaders, **not 66 missing games**. Original slot
  assignment remains ambiguous; the source game IDs/dates/DH labels already
  identify the actual games for a controlled played-game manifest. Preserve both
  identities; do not guess makeup order or treat slots as extra games.
- **1,214 DH / 1,215 non-DH games.** All 46,150 starting identity/appearance slots
  corroborate; no missing start lookups or unknown hands. Existing conservative
  candidate classification is 2,425, explicitly excluding four suspensions.
  That classification is not a runtime or complete-season claim.
- Four suspended games stay visible: `CHA201905270` finished May 28,
  `DET201905190` finished September 6, `NYN201906130` finished June 14 and
  `BOS201908070` finished August 22. Ten additional games are marked shortened;
  all 14 exceptional game records are in `summary.json`. Neither suspended nor
  shortened games may silently disappear from a full-season result.

### Personnel, ratings and runtime boundary

- 69,613 observed fielding appearances; 1,559 player/team membership pairs across
  1,405 appearance players; 145 multi-team players. These are observations, not
  active-roster intervals, bench eligibility, rest or injury evidence. Keep traded
  players' stable IDs and team splits rather than adding season totals twice.
- One reversed appearance range remains: `culbc001`, `ATL201906110`, Home LF,
  start92/end91. Do not repair it by inventing an interval or silently dropping it.
- Native event profiles cover 990 batters/831 pitchers; there are 871 distinct
  starting batters/368 starting pitchers. **12 starting batters have no full-season
  live batting-profile events**; all starting pitchers have live pitching events.
  A starting identity must not be discarded because it never actually batted.
- Separately examined the previously authenticated 118,464-pair calibration input
  (SHA256 `fbe123fa2e90ec624a9f64367c653ba24bc1e581877ed7014f6069688765c986`).
  Its role/player ordinary-opportunity totals reconcile with the new native
  profiles. Training folds2–9 support **966 batters/819 pitchers**, not the full
  990/831 pools. **21 starting batters and one starter pitcher (`gilms001`) have
  zero training opportunities**; the 12 full-season-zero batters are included in
  those 21. Lists and input pins are in `training-coverage.json`. Count support is
  not rating validity or forecasting proof. Zero counts need explicit priors;
  never substitute a missing actor, full-season data or held-out observations.
- Existing `real_lineup.rs` is pinned to `OAK201903200`; its exporter loads that DH
  fixture. `full_manager.rs`/cast exports use its closed Seattle–Oakland personnel.
  Existing mean/empirical artifacts are not all-team casts. Generalized input
  exports/loaders, explicit zero-count profiles and non-DH pitching-slot handling
  are prerequisites; do not remove their current missing-profile safeguards.
- Fixed200 stays default, calibrated v2 opt-in and scientific hit-mix v3 rejected.
  No probabilities, outcome RNG, manager policy or rating artifacts changed.

### Reference-stat inventory and an actual failed reconciliation

The source has **13,278 player/team/metric rows** for PA/AB/H/2B/3B/HR/BB/K/HBP/RBI
(990 batters) and R/SB/CS (816 observed runners). All 30 teams are represented;
no exported metric source rows are NULL. Aggregate by stable player ID for league
player totals, retaining player/team rows for trades. Absence from a source role
is not automatically an official zero. Native source totals include H42,039,
HR6,776, R23,467 and source PA186,517. These are same-lineage derived credits,
**not independently certified official player season totals**.

An additional native team-results cross-check of 390 team/metric cells found:

- **360/360 non-PA cells agree.** Six PA cells also agree; **24 PA cells fail**.
- Source terminal PA186,517 exceeds
  `AB + BB + HBP + SH + SF + reached-on-interference` by **62**. Both source
  aggregates give SH721/SF1,143, so a shared-derived team comparison alone would
  hide the credit gap. The failing command/assertion and 24 differences are
  retained in `independent-results.json`; no assertion/tolerance was weakened.
- Exact raw detail identifies 40 `ReachedOnError` and 22 `FieldersChoice` events
  with source PA1/AB0/SH0/SF0; **55 raw plays carry `/SH`, seven `/SF`**. Examples:
  `ANA201904240` event28 `E2/SH.1-2`; `CHN201904130` event105
  `E7/SF/F7LM.3-H;2-3;B-2`. Full 62 rows are in `pa-62-events.json`.
  This localizes the missing derived sacrifice credits; it does not authorize
  patching all official stats from a result label or altering the cached source.
  **Reference PA/SH/SF certification remains blocked pending a separately tested
  scorer-aware reconciliation.** Do not call source PA an official BF/IP proxy.
- Pitcher participant profiles/terminal PA and recorded outs are available, but a
  certified player pitching-stat projection with responsibility/BF/ER/IP is not
  exported. Existing engine AB/RBI/ER uncertainty must remain visible.
- Cached fielding has 192,025 event/team rows, 22,633 personnel-state keys and no
  missing keys/source unknown-putout/incomplete flags. It is **not a player-grain
  fielding-stat table**. The supplement has appearances/raw fielding actors; a
  validated complete-season bridge/projection is still required. Zero source
  unknown flags do not certify official scoring or range/arm opportunity rates.

### Verification and reproduction

Local-only evidence root: `/tmp/bbsim-season-readiness-v1/`. Raw datasets/exports
are deliberately not committed or cleared for redistribution. `/tmp` is not a
remotely durable archive.

- `proc_2ac9`, exit0/4s: `python3 -B /tmp/bbsim-season-readiness-v1/run.py`.
  Two 6,531,663-byte reports and native exports are byte-identical; both accepted
  databases and four query/implementation inputs unchanged (`before.json`,
  `after.json`, `source.stdout`).
- `proc_27db`, exit0/56s: `python3 -B /tmp/bbsim-season-readiness-v1/verify.py`.
  **33 focused tests / all636 Python tests** pass with ResourceWarning-as-error;
  `git diff --check` passes; all101 audited Python/SQL/test pins unchanged. Seven
  new tests include an observed date-boundary red/green; initial import red and
  malformed-date assertion/error evidence are retained separately.
- `independent.sql` independently reconciles all2,430 schedule dispositions/unique
  IDs using SQL joins/window cardinalities; all played IDs remain covered. Its
  separate team-reference assertion **fails as documented above**. Native output,
  stderr and failure receipt retained. `training-coverage.py` exit0 independently
  checks all pair folds/counts against the existing input and new native profiles.
  `pa-diagnostic.json`/`pa-62-events.json` and `fielding-diagnostic.json` retain
  the additional read-only findings. No source data or scientific artifact changed.
- No Rust changes: **Rust/build/Clippy were not rerun** and prior results are not
  fresh certification. No delegated independent review or external official-season
  comparison was performed. This is an author-inspected, source-pinned audit.

Core audit can be reproduced to a new destination without any new transfer:

```sh
out=$(mktemp -d /tmp/bbsim-season-readiness-repro.XXXXXX)
python3 -B -m audits.season_readiness \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  /tmp/bbsim-outcome-details/2019-outcomes.duckdb "$out/report.json"
python3 -B -W error::ResourceWarning -m unittest tests.test_season_readiness tests.test_lineup_readiness -v
python3 -B -W error::ResourceWarning -m unittest discover -s tests -v
git diff --check
```

### Concrete next slice

**P-02-season-game-inputs-v1:** generalize the existing strict historical lineup and
closed-cast export/load path, first proving one DH and one non-DH game can run with
stable-ID player profiles and canonical accounting, without the Seattle–Oakland
identity restriction. Keep the old command/fixture behavior unchanged; explicit
cold profiles stay distinguishable from missing actors. Do not build a second
outcome engine or use season-wide rosters as a fictitious daily bullpen.

Then build the headless played-game manifest/runner and cumulative statistics.
Before calling that run a whole-season historical replay, record suspension and
shortened-game treatment and the incomplete reference/scoring support. Keep
controlled historical casts separate from autonomous management, which still needs
pinch hitting, legal bench/replacement handling, availability and persistent usage.
Completing games/accounting is not player-stat fidelity or any S/G gate signoff.
