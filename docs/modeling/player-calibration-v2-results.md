# Player calibration — verified development checkpoint (2026-10-17)

## Disposition

The [v1 comparison](player-calibration-v1.md) **failed** its frozen guardrail;
it remains rejected. The separately preregistered [v2 tree](player-calibration-v2.md)
passes the same numerical bounds and is available only through an explicitly
supplied development artifact. **The v2 test benchmark was already used by v1.**
These results are retrospective development evidence, not fresh independent
predictive validation, true-ability estimates, or P-01/P-02/G1–G5 signoff.
The ordinary four-input text-game command and existing preview remain unchanged.

Authority: 15:48–17:48UTC; the approximately60-minute checkpoint was delivered
before16:48UTC. No new seasons, transfers, dependencies, UI or native mechanics.
Evidence root throughout: `/tmp/bbsim-player-calibration-v1/`. Real source,
effects, ensemble rows and full game transcripts remain external to Git.

## Frozen comparisons and measured results

All192025 source events are validated before classification;185703 included
terminal-participant proxies across2429 games reconcile to990 batter/831 pitcher
profiles. Exclude753 IBB and61 interference events from the nine rated parents;
retain interference separately in game sampling. These are not complete official
PA/BF credits. Training parameters use only training outcomes; validation chooses
among training-qualified fits before test evaluation. Full refits use the audited
full cohort only for hindsight reconstruction. A terminal-opportunity denominator
means all nine categories in the evaluated partition, not a conditional risk set.

V1:99 specified candidates, no convergence failures. Validation selected
`joint-20-20`. Its mean test NLL difference versus fixed200 is
**+.010389321421514129**, greater than the frozen+.01 allowance. The paired-game
95% normal interval is[+.007376590389302129,+.013402052453726129]. Versus league,
the difference is−.01313021144506074, interval
[−.01891060253307745,−.0073498203570440315]. Full reconstruction and variation pass,
but **accepted:false**; neither rounding nor a better-looking Brier score rescues it.

V2: all16 strength pairs at each of8 nodes converge. Selected batter/pitcher
strengths are20/20 for K, BB, HBP, any-hit and HR;200/200 for triple, double and ROE.
The selected pairs are refitted without reselection. One complete nine-vector
is constructed before a single parent CDF draw; there are no node-level rolls.

Reused test benchmark:19002 opportunities/249 whole games:

| Model | Mean NLL (nats/proxy) | Multiclass Brier |
|---|---:|---:|
| League | 1.5540884065 | .7210869325 |
| Fixed200 marginal/log5 | 1.5305688737 | .7112969002 |
| Rejected v1 joint20/20 | 1.5409581951 | .7108561703 |
| V2 outcome tree | **1.5276428749** | **.7100148394** |

V2 minus fixed200:−.002925998787061284, descriptive paired-game95% interval
[−.004978448372011682,−.000873549202110887]. V2 minus league:
−.026445531653636153, interval[−.0309906709874876,−.021900392319784707].
The inherited JSON field`heldout_guardrails:true` records arithmetic satisfaction
on this **reused** benchmark; it is not a new independent-validation certificate.

The improvement over v1 is concentrated in ROE(−.00447767), triples(−.00437001)
and doubles(−.00201341) category NLL contributions. Improvement is not uniform
versus fixed200: HBP(+.00117626), singles(+.00124158), HR(+.00022317) and contact-out
(+.00083826) contributions are worse. All nine contributions and their sum checks
are retained in`supplemental-diagnostics.json`; no candidate or threshold was
changed after these measurements.

## Individual reconstruction and remaining mismatch

Full-cohort actual-opponent expectations, players with>=200 included proxies.
Numbers below are **percentage points of terminal opportunities**, not relative
percentage errors. Every listed MAE and maximum is no worse than fixed200;
all frozen absolute common/rare-category bounds also pass.

| Role/category | Fixed200 MAE | V2 MAE | Fixed200 maximum | V2 maximum |
|---|---:|---:|---:|---:|
| Batter K | 1.6628 | .2588 | 10.6608 | 1.7929 |
| Batter BB | .8373 | .1803 | 4.3246 | 1.1711 |
| Batter HR | .4313 | .2448 | 2.4527 | 1.4663 |
| Batter combined hits | .8645 | .2016 | 4.0434 | 1.1087 |
| Pitcher K | 1.9011 | .3451 | 12.2637 | 2.1022 |
| Pitcher BB | .7719 | .1772 | 4.4391 | .8730 |
| Pitcher HR | .3794 | .2244 | 2.0905 | 1.3385 |
| Pitcher combined hits | 1.0823 | .2522 | 5.2589 | 1.1088 |

**This is not complete player-stat fidelity.** Singles/doubles had no separate
frozen primary bounds. For qualifying batters, singles MAE is.7622pp and maximum
4.3776pp; doubles MAE.6238pp and maximum4.1172pp. Both worst deviations concern
`buxtb001`: too many singles, too few doubles. This allocation mismatch remains
visible even when combined hits match well. Do not hide it behind the passing
combined-hit constraint or change the current study's criteria after scoring.
A separately preregistered follow-up should address it.

The supplemental report retains all120 role/exposure-band/category reconstruction
rows, including empty/zero bands, and32 partition/node risk-set receipts. Undefined
zero-exposure rate biases are null, not invented zeros. Full conditional risk
populations are185703/142880/127738/125754/42039/35263/34478/83715. At the full-fit
HR-given-hit node,221 batter/17 pitcher profiles have zero hit opportunities;
provided cold profiles are neutral priors, not missing-role fallbacks. In training,
24 batters/12 pitchers have zero total included exposure.

Sparse players remain uncertain. For example, the pinned six-opportunity,
zero-observed-HR hitter still has positive model HR probability and nonzero
simulation variation. Neutral-opponent rates are regularized point estimates,
not Bayesian ability posteriors or exact historical frequencies. There is no
propagated parameter uncertainty, handedness/park adjustment, verified availability
or historically validated full-schedule/playing-time variance.

## Independent arithmetic, runtime and compatibility

- Final native repeats reproduce **all110 v1 files and all19 v2 files byte-for-byte**.
  Const-generic2/9 fitting did not alter the rejected v1 result. A disposable
  synthetic input changes one test-only count; all8 node candidate tables,
  training effects and pre-test selection remain identical. Original inputs
  are never modified. See`final-comparison-lifecycle.json`.
- Fresh Python audits independently re-read/reconcile the source and game split,
  recompute probabilities, NLL/Brier/O-E, clustered differences, full player bias,
  and every one of364200 ensemble rows/16389 variation cells. Both expected
  dispositions agree;200 seed1..200 fixed-opportunity ensembles pass unchanged
  centering/nonzero-variance/SD bounds. They do not establish real season variance.
- Separately recomputed full augmented-margin ratio residuals are at most
  3.254e-9(v1) and4.481e-9(v2), below1e-8. This also supplies full-fit numerical
  evidence beyond the retained candidate iteration receipts. A separate same-
  specification v1 full-refit probe recovers22 cycles/correction6.851e-9 and
  reproduces every full-effect byte; it does not change any model or score.
  Minimum probability over audited observed full-cohort matchups is.0000311938,
  with no dropped mass/zeros/retries.
- The versioned artifact requires all18 hitters/all12 pinned pitchers, exact
  source counts/metadata, finite conditional rates strictly between0and1 and
  complete positive normalized vectors. Missing current actors refuse. Evidence
  hashes are provenance declarations, **not signatures** or independent-validation
  certificates. The export binds unchanged audited inputs and selected effects.
- Actual explicit-artifact CLI seeds1..1000:85005 native events,76709 ordinary
  parent draws,1501 runner-only events,6777 pitcher changes and18 tactical IBB.
  All30 roles/all12 pitchers are consumed; before actors, native reliever entry,
  words, full vectors and parent CDF selections reconcile independently. Maximum
  Rust/Python probability discrepancy is0; all300 role/parent sampling flags pass,
  largest absolute standardized residual2.64251. This is a sampling bug screen,
  not an accuracy p-value. Every game executes native continuity/accounting checks.
- Seeds1/2/3 repeat exactly: Seattle–Oakland5–13(101 events/93 draws),
  3–7(85/76),4–3(90/81). Those are exhibition realizations, not forecasts.
- The unchanged four-input command reruns all1000 earlier seeds: **all2000
  stdout/stderr files byte-identical** to the corrected prior baseline cohort.
  See`baseline-compatibility.json`; calibrated and baseline game audits are separate.
- 25 real artifact/export refusal controls pass, including missing reliever,
  wrong version/count/cohort/strength/league, duplicate/extra/control fields,
  NaN/zero/one/wrong width/underflow vectors, existing output, changed audited
  input, rejected evidence and wrong binding. Failed CLI loads emit zero stdout.

## Verification and review

Final source-pinned run`proc_8740`,16:49:05–17:00:04UTC, exits0/660s:

```sh
cargo build --offline --locked --manifest-path engine/Cargo.toml --examples
cargo test --offline --locked --manifest-path engine/Cargo.toml --test full_player_game --test player_calibration --test player_calibration_tree
cargo test --offline --locked --manifest-path engine/Cargo.toml
cargo test --offline --locked --manifest-path engine/Cargo.toml --examples
cargo fmt --manifest-path engine/Cargo.toml --check
cargo clippy --offline --locked --manifest-path engine/Cargo.toml --all-targets -- -D warnings
python3 -B -W error::ResourceWarning -m unittest discover -s tests -v
git diff --check
```

Results:14 focused tests;866 Rust tests/63 summaries;2 example tests/5 summaries;
620 Python tests in60.863s; build/fmt/strict Clippy/diff pass. All197 Rust/Cargo/
Python source pins unchanged through the run. Raw commands, times, exits and logs:
`verification-final/`; no source edits during verification.

Fresh read-only Sol-high reviewer`42e76d01-a86a-4490-93f1-68a687602b0f` found no
issues in the fitter/tree/selection/scoring/source binding/current-actor path.
The reviewer inspected code and retained focused logs, but did **not** independently
execute tests. Parent fresh suites/audits above close its verification conditions.
Full review is retained under the session's`reviews/player-calibration-v1-v2.md`.

Retained failures: original missing APIs, plain-IPF convergence, exact cold
neutrality, legitimate hyphenated participant, calibrated loader/export/CLI-flag
reds; offense-method compiler typo and initial Clippy loop/drop errors. V1's
scientific rejection is separate from these implementation/harness failures.
No source exclusion, tolerance relaxation, candidate switching or threshold retuning.

## Reproduction and use

Audited offline source fixtures and a new output path are required. Never overwrite
a prior evidence directory. Native comparison remains separate from the rules engine:

```sh
cargo build --release --offline --locked --manifest-path engine/Cargo.toml --example calibrate_players
python3 -B -W error::ResourceWarning -m audits.player_calibration \
  /tmp/bbsim-joint-contact/context-first.csv \
  /tmp/bbsim-player-opportunities/report-first.json /tmp/NEW_CALIBRATION_INPUT
engine/target/release/examples/calibrate_players \
  /tmp/NEW_CALIBRATION_INPUT/opportunities.tsv /tmp/NEW_TREE_COMPARISON --tree-v2
python3 -B -W error::ResourceWarning -m audits.player_calibration_review \
  /tmp/bbsim-joint-contact/context-first.csv \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/NEW_CALIBRATION_INPUT /tmp/NEW_TREE_COMPARISON /tmp/NEW_TREE_AUDIT.json
```

Export with the source-bound helper (all arguments are `pathlib.Path`):

```sh
python3 -B - <<'PY'
from pathlib import Path
from audits.player_calibration_review import export_tree
export_tree(Path('/tmp/NEW_TREE_COMPARISON'), Path('/tmp/NEW_TREE_AUDIT.json'),
            Path('/tmp/bbsim-pitcher-workload/cast-first.tsv'),
            Path('/tmp/NEW_TREE_RATINGS.tsv'))
PY
```

Refuse stale/changed evidence rather than copying a pass flag. Current verified artifact:
`/tmp/bbsim-player-calibration-v1/calibrated-cast-v2.tsv`, SHA256
`905ee5c6153f67e474409b0f70e55cec1ffa913e511afe901cc1bbc4d820fb43`.

```sh
cargo build --offline --locked --manifest-path engine/Cargo.toml --examples
engine/target/debug/examples/text_game \
  --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup /tmp/bbsim-real-lineup/lineup-first.tsv \
  --cast /tmp/bbsim-pitcher-workload/cast-first.tsv \
  --calibrated-ratings /tmp/bbsim-player-calibration-v1/calibrated-cast-v2.tsv \
  --seed 1
```

Omit`--calibrated-ratings` to retain the prior model/outputs. Full retained drivers:
`verify-final.py`, `run-final-comparisons.py`, `diagnostics-tree.py`,
`check-baseline-compatibility.py`, `supplemental-diagnostics.py`,
`refusal-controls.py`, plus the deterministic`full-refit-probe.rs`. Their raw
outputs remain alongside them. The last two
are supplemental read-only reporting/control checks, not new fitted criteria.

Work is uncommitted; local evidence is not remotely durable and`/tmp` may be
cleaned. No push, release, data publication or fresh independent validation.
Next research: preregister singles/doubles reconstruction constraints and a
validation design appropriate to any proposed predictive claim, before new fits.
