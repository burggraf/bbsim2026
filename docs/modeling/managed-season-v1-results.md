# Managed season v1 — completed evaluation, pitcher-usage NO-GO

## Disposition

**The opt-in implementation is verified; v1 fails its frozen individual-pitcher
usage criterion. Do not adopt it as a validated usage model or change the limits to
make it pass.** No default-policy change, predictive-accuracy claim, historical
eligibility certification, public-data approval or G1–G5 signoff follows.

Implementation: `fb2cde0e34ea9dcae276feb5924cbe68e8e370c6` on native bridge `bf4568b`.
[Contract](managed-season-v1.md) and [frozen usage criteria](season-usage-management-v1.md).
Whole-season club pools, observed positions, global modeled pitching rest and soft
historical targets are explicit hindsight assumptions. Daily roster acquisition
remains stopped and is not a prerequisite for the next policy diagnosis.

## Three production-CLI seasons

Driver `/tmp/bbsim-managed-season-v1/seasons-v1.py`, process `proc_f002`, exit 0 after
3,128 seconds. It ran each seed once through `python3 -B -W error -m
preview.managed_season`, then authenticated/reconstructed retained comparisons.
All three seeds completed **2,429/2,429 games**, with **no input refusals or native
failures**, complete accepted-history status and preserved source/model/native pins.

The frozen headline requires at least 80% of **compared** batter-team entries with
at least 100 actual PA to be within `max(20 PA,25%)`; pitcher-team entries with at
least 60 actual outs must be within `max(18 outs,30%)`. Aggregate relative limits
are 5%. Unknown/uncompared usage is not silently converted to zero.

| Seed | Batter within band | Pitcher within band | Qualifying pitchers uncomparable | PA: simulated vs actual 186,517 | Outs: simulated vs actual 130,270 |
|---|---:|---:|---:|---:|---:|
| 1 | 455/455 (100%) — pass | 337/541 (62.29%) — **fail** | 24 of 565 eligible | 191,400 (+2.618%) — pass | 130,690 (+0.322%) — pass |
| 2 | 455/455 (100%) — pass | 326/541 (60.26%) — **fail** | 24 of 565 eligible | 191,131 (+2.474%) — pass | 130,872 (+0.462%) — pass |
| 3 | 455/455 (100%) — pass | 338/541 (62.48%) — **fail** | 24 of 565 eligible | 191,136 (+2.476%) — pass | 130,708 (+0.336%) — pass |

No qualifying batter is uncomparable. Each seed has 24 qualifying pitcher-team
entries with no simulated pitching appearance. They are missing comparisons, not measured
zero-out observations, and are excluded from the registered 541-entry denominator.
Their identities vary: 20 entries are absent in all three seeds, 28 in the union.
Full player-team errors, opportunity bands, player aggregates, coverage and every
missing entry remain in the retained JSON, not just these headline percentages.
Targets are supplied to the policy: batter matching measures retrospective usage
control, **not independent predictive accuracy**.

## Exactly matched historical comparison

Historical runs complete only 2,384 games and refuse 45. For each seed, comparison
uses exactly those 2,384 IDs on both sides. The managed selections still use their
original uninterrupted 2,429-game history: filtering observations does not rerun or
alter policy context. Historical raw receipts/outputs are authenticated against the
previous sealed archive, not a newly executed historical binary.

| Seed | Matched managed batters | Historical batters | Matched managed pitchers | Historical pitchers |
|---|---:|---:|---:|---:|
| 1 | 453/454 (99.78%) | 442/454 (97.36%) | 332/538 (61.71%) | 515/562 (91.64%) |
| 2 | 454/454 (100%) | 445/454 (98.02%) | 323/538 (60.04%) | 518/562 (92.17%) |
| 3 | 454/454 (100%) | 440/454 (96.92%) | 336/538 (62.45%) | 517/562 (91.99%) |

Managed pitching has 24 uncomparable qualifying entries per matched cohort;
historical pitching has none. This is not a paired comparison of identical player
populations and must not hide the coverage difference. Both policies' matched
league PA/outs pass their aggregate bounds. Historical inputs impose historical
starting personnel, whereas managed inputs make autonomous pool selections; their
better historical pitching agreement does not establish an autonomous baseline.

Full-schedule coverage (all roles):

| Seed | Compared player-team rows | Actual only | Simulated only / unknown actual |
|---|---:|---:|---:|
| 1 | 1,807 | 187 | 280 |
| 2 | 1,809 | 185 | 287 |
| 3 | 1,806 | 188 | 269 |

All 2,429 target games match; 378,542 source rows, zero unknown source usage rows.
Simulated-only rows do not establish historically zero opportunities.

## Descriptive pitching failure evidence — not a refit

| Seed | Compared qualifying pitchers below / above band | Simulated starter outs | Simulated relief outs | Missing pitchers' actual outs |
|---|---:|---:|---:|---:|
| 1 | 125 / 79 | 73,832 | 56,858 | 3,059 |
| 2 | 131 / 84 | 73,748 | 57,124 | 3,005 |
| 3 | 123 / 80 | 73,741 | 56,967 | 2,902 |

Actual starter/relief outs are 75,473/54,797. League-level role totals conceal large
individual errors. Examples from seed 1, in **integer outs**:

- KCA `flynb001`: 88 actual, 470 simulated; 1 actual start, no simulated starts.
- ARI `duplj001`: 110 actual, 463 simulated; 3 actual starts, no simulated starts.
- ATL `gausk001`: 240 actual, 7 simulated; 16 actual starts, no simulated starts.
- TEX `allak001`: 136 actual, 378 simulated; 9 actual starts, 29 simulated starts.
- SDN `lamed001`: 219 actual outs/14 starts; no simulated pitching appearance in seed 1.

The next bounded diagnosis should trace principal-rotation allocation, extra-starter
opportunities, mixed-role selection and situation-class-before-pace bullpen choices
against retained pregame decisions. These observations do **not** isolate causal
contributions or authorize a new fitted policy. Do not invent appearances, relax
rest/legality, acquire daily rosters or adjust outcome probabilities to repair usage.
A changed policy requires a separately versioned, preregistered evaluation; v1 and
its failed metrics remain frozen.

Two exploratory assertions were falsified: missing identities were not identical
across seeds, and not every missing entry was start-only. Their failures are retained
in `diagnostic-first-assertion.stderr` and `diagnostic-followup.stderr`; the complete
`pitcher-descriptive-diagnostics.json` was preserved. Seed 1 missing entries have
zero actual relief outs; seeds 2/3 include 37/107 actual relief outs. These were
analysis assumptions, not failing project tests; neither the policy nor test suite
was changed in response. Final diagnostic arithmetic checks reconcile role totals
and within/outside/uncompared counts in `diagnostic-final.stdout/.stderr`.

## Engineering evidence and limits

Final checks: `/tmp/bbsim-managed-season-v1/verification-v2/results.json`, process
`proc_d78e`, exit 0/818 seconds; raw stdout/stderr retained:

- Offline locked examples build; 769 Python tests with warnings as errors.
- 956 Rust tests, 66 zero-failure summaries, no ignored tests; 4 example tests.
- `cargo fmt --check`, strict offline locked all-target Clippy, `git diff --check`.
- 12 exact historical CLI controls: DH/non-DH × bench/no bench ×
  accounting/radio/detailed against retained `4a3c76c` binary.
- 254 code pins and protected inputs/DBs unchanged; current native SHA
  `f226b27b6562aafa9df5910a4c56ce0b58a8e2080d7b06d752279e63cabe04d8`.

Final-code mechanism v3: `proc_9774`, exit 0/353 seconds; 2,429 legal static inputs,
512 chronological native completions, 10 club doubleheader dates, four exact native
repeats, independent wire accounting/choice/ledger reconstruction and preserved
pins. The prefix includes three normalized shortened games and no suspensions;
whole seasons use the approved ordinary-full-game policy, not replayed suspensions.
No full-seed native repeat was performed; the four mechanism repeats are narrower
proof. UI/desktop end-to-end, ensemble fidelity/S-03 and scientific gates remain open.

Independent review `7f52856e` and narrow follow-ups `2d4ed27d`/`38ea8395` are clear:
strict completed-exit validation, direct dependency pins, scarce shared-starter
allocation and receipt-publication error propagation were fixed with retained RED/
GREEN evidence. Final focused suite: 120 tests. Interrupted full-verification-v1 and
earlier property timeout are retained; only complete final runs certify current code.

## Retained artifacts / reproduction

Root `/tmp/bbsim-managed-season-v1/seasons-v1/` contains all three CLI batches,
`report.json`, `executions.json`, full and exactly matched comparison JSON, native
outputs, inputs, decisions, game receipts, ledgers and diagnostic failures. Do not
rerun its exclusive destinations. Batch receipt SHA-256:

- Seed 1: `9a053124f4e81388de0b14525433baa74a387e6f2ee93a56f29277931a4402d1`
- Seed 2: `e15aa44c5f45e7c6715fc13fc7e1588436a7b2b516cdb6eb544a5c1a87971d73`
- Seed 3: `e17e2f92dfe5e4922788939b825f38a5f524432717a1f9b324f90464f1b26816`

The gated driver records exact CLI commands, authenticates target SHA
`f45cd37997e1027ac17e4e4d752c4aa44e2be96f871f510408f72e432108972a`,
verifies every retained output against its receipt, reconstructs matched usage from
raw native rows and confirms watched source/model/native hashes afterward.
For a new run, use the CLI command in `executions.json` with a **new** output directory;
never overwrite evidence. `preview.managed_season.rebuild` reconstructs a batch from
its retained inputs/outputs without rerunning native games.

Private evidence seal is tracked in `docs/PROGRESS.md`; no archive or source data is
published or committed. The prior historical archive remains separately retained.
