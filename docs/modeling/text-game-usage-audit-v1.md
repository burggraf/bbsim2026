# P-02-text-game-usage-v1 — conditional usage and closed-bullpen audit

## Preregistration (2026-10-17, base `48eb3e0`)

Maintainer approved a bounded offline removal/bullpen audit followed by one
root-cause correction. No downloads, new dependencies, eligible personnel,
PA probabilities, ratings, outcome RNG, canonical engine/accounting or UI changes.
The fixed Seattle–Oakland exhibition is not a representative season replay.
Existing source reports, cast-v1 default bytes and empirical cast-v2 stay intact.
No push or gate signoff. Existing preview servers stay running.

### Source questions and observable audit acceptance

Use the accepted read-only 2019 snapshot, existing validated outcome export and
workload report. Pin inputs and recipes before reading and again before exclusive
publication outside Git. Reconcile context keys one-to-one against the entire
validated event export and all selected training outing PA totals, including zero
PA outings. Preserve the four prior excluded games, unknowns and multistint flags.
Do not infer pitches, rest, injuries, eligibility or the manager's reason.

For each selected pitcher/game/role, report terminal-participant PA proxy,
recorded outs while pitching (not official pitcher IP), hits and runs during the
outing; each observed continuation/change by cumulative PA, runs and score lead.
A last defensive event without a successor is **game-end censored**, not removal.
Condition on the state available at the decision; never use eventual outing runs
as the earlier state. Separate starts/relief and retain individual sample sizes.
Report terminal-PA decision cells separately from runner-only decisions. Reconcile
all opportunities, changes and censored endpoints. Repeat the report byte-exactly.
These observed associations cannot identify tactical/injury/fatigue causes.

### Diagnosis before correction

Independently reconstruct v2 entry budgets from its existing development seeds
1–1000 and stable manager words. Classify starter removal as below-budget
performance hook versus budget reached; distinguish final pitcher exhaustion,
early game ending and unused bullpen. Record each final entry's excess above its
sampled budget and observed role maximum; maxima are diagnostics, not stamina.
No policy coefficients may be tuned to this cohort.

After that diagnosis, record one minimal correction and its rationale here before
implementation. Correct overlapping removal mechanisms first if established;
do not invent additional bullpen personnel or hide irreducible exhaustion with
forced outs, fabricated endings, reentry or hard source-maximum caps.

### Candidate acceptance, fixed before evaluating any new candidate

- Test-first regression for the selected mechanism and boundary controls. Retain
  no-hit coverage/upper bound, score extension, legal substitutions, no-DH,
  inherited responsibility, runner-only exclusion and stable entry sampling.
- Development comparison: paired seeds1–1000, both supported casts. Show starter
  BF/outs quantiles, below18BF starts, complete games, after-six hitless/scoreless
  continuation, bullpen exhaustion and final-entry tails. No seed67 tuning.
- Final comparison: **fresh paired seeds1001–2000**, both casts and the immutable
  pre-change binary. All4000games must complete, independently reconcile boxes
  and available clean ER, and repeated seeds1001/1067 must be byte-identical.
- For adoption: remove the diagnosed extra below-budget hook exactly; reduce
  its count to zero; aggregate starts<18BF must not increase. For each cast,
  final-entry above-source-role-maximum fraction must not worsen by more than
  **2 percentage points** on fresh paired games. Report paired changes and
  uncertainty, not just a threshold verdict. This is a bounded regression budget,
  not historical-fidelity acceptance. If it fails, retain the candidate and raw
  failures without promoting it or changing the tolerance.
- Source mean comparisons are descriptive, not matched-opponent calibration
  targets. Report failure to improve them; do not adjust thresholds to fit them.
- Full offline/locked Rust suite, example tests/build, Python discovery with
  ResourceWarning errors, fmt, strict all-target Clippy and diff checks on frozen
  sources. Protect source datasets, exports, canonical engine, ratings and outcome
  kernels with hashes. Update PROGRESS with exact commands, results and failures.

## Recorded diagnosis and selected correction (before implementation)

Read-only source census repeats exactly: 482 training player/game outings,
3391 events /3250 terminal-participant PA;384 observed changes and98 game-end
censored endpoints. One zero-PA outing remains; three nontraining outings are
excluded unchanged; no selected multiple-appearance rows. Both roles retained
(70 starts/412 relief outings), not just the two exhibition starters.
For starters at9–17PA, >=4runs while pitching and tied/trailing, observed next
decisions after terminal PAs include35 continuations versus4 changes. These are
associations in sparse hindsight data, not evidence for a deterministic hook.

Independent v2 development-log reconstruction (`diagnosis.json` outside Git)
finds502 mean/478 empirical below-budget starter hooks. Final-entry above-source-
maximum tails:340 mean (all exhausted),521 empirical (520 exhausted,1 with unused
bullpen). Exhausted team-games:653 mean/914 empirical out of2000 each.

**Selected correction:** delete the additional >=9PA/>=4runs/tied-or-trailing
shortening rule for both cast formats. Both empirical lengths and their role
means already summarize observed removal, including short/ineffective outings;
a second deterministic cutoff imposes another removal process without estimating
conditional survival. Do not refit thresholds or claim this identifies fatigue.
Version the manager `workload-context-2019-v3`; retain stable words, budgets,
scoreless lead extension and certified no-hit upper bound. No export change.
CLI must disclose no additional early-performance hook. This one correction may
reduce bullpen pressure but does not eliminate its finite-cast cause.

## Reproduction and measured coverage

Local evidence root: `/tmp/bbsim-text-game-usage-v1/`. No datasets or generated
transcripts are committed. Use **new** destinations for repeats; never overwrite
accepted reports. Example with the existing private inputs:

```sh
shasum -a 256 /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb
# Must match d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528.
duckdb -init /dev/null -readonly -bail -batch -csv -nullvalue '' \
  -cmd 'SET enable_external_access=false' \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  -f audits/2019-pitcher-usage.sql > /tmp/usage-context-new.csv
python3 -B -m audits.pitcher_usage /tmp/usage-context-new.csv \
  /tmp/bbsim-pitcher-workload/report-null-fixed-first.events.csv \
  /tmp/bbsim-pitcher-workload/report-null-fixed-first.json \
  /tmp/usage-report-new.json
shasum -a 256 /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb
python3 -B -m unittest tests.test_pitcher_usage -v
```

Retain native stderr and before/after hashes as in `source.py`; the module binds
all CSV/report/recipe bytes but does not authenticate arbitrary hand-edited
context CSVs. Actual `source-reference.sql` independently computes windowed
counters, leads and successors directly from the pinned DB; all3391 decision
rows/nine fields agree with the Python census. Full existing participant/outcome
validation covers the192025-event export before selected-outing measurement.
Source context keys and dimensions are complete for this accepted export; a
missing/duplicate/orphan key or unknown inning/side refuses rather than inventing
context. Continuations to a next defensive event can cross an inning boundary.
Source appearance ranges label roles, not cause, stamina or verified availability.

The70 training starts include selected bullpen pitchers' other starts; they are
not all the two exhibition starters. Per-player start-role PA observations and
recorded outs while pitching (not official IP credits):

| Pitcher | Starts | PA mean / p10 / median / p90 | Recorded outs mean / p10 / median / p90 |
|---|---:|---|---|
| Gonzales |34|25.471 /22 /26 /29|17.912 /14 /18 /21|
| Fiers |32|22.813 /16 /23.5 /27|16.750 /9 /18 /21|

Nearest-rank indices are floor((n-1)*q); medians use the central-pair average.
These season/role summaries are not matched to this exhibition's opponents.
Missing physical fatigue, rest, injury and actual removal reasons remain unknown.

## Frozen-candidate comparison results

Development seeds1–1000: new mean policy starts<18BF285→0 and final tails340→162;
empirical starts406→145/tails521→344. No coefficient was adjusted afterward.
Seed67 is unchanged in both casts/formats except the manager disclosure.

Fresh paired seeds1001–2000:1000games for each policy/cast combination,
**4000 native games**, all complete and independently reconcile box counters,
pitcher entry order/notes and33628certifiable pitcher ER lines. Detailed/radio
repeats for seeds1001/1067 and experimental-rating repeat controls pass.
Denominator below is2000team starts per row; BF is the simulated box counter.

| Cast / manager | Gonzales BF mean | Fiers BF mean | Starts<18BF | Complete games | Exhausted team-games | Final entry above observed role max |
|---|---:|---:|---:|---:|---:|---:|
| Mean /v2 |23.293|21.298|323|0|709|395|
| Mean /v3 |26.155|23.139|0|0|472|168|
| Empirical /v2 |22.974|21.032|439|5|940|537|
| Empirical /v3 |25.814|23.010|140|6|727|326|

The diagnosed below-budget performance hooks go548→0 for each cast. Final-tail
fraction differences (v3−v2) are **−11.35percentage points** for mean (paired
Monte Carlo normal95% interval[−12.84,−9.86]) and **−10.55points** empirical
([−12.16,−8.94]); both clear the preregistered maximum +2point regression budget.
All mean tails are exhausted; empirical tails with unused bullpen increase1→4
(the other322v3 tails are exhausted), so not every tail is caused by exhaustion.
The deterministic mean fallback produces no short starts; that missing spread
is still not a faithful historical usage distribution. Empirical budgets retain
spread and sparse/zero-PA observations; the 1PA simulation minimum stays legal.

Narrated hitless/scoreless survivors after six continue for seventh: mean4/4→8/8,
empirical5/5→8/8; scoreless starters (hits permitted):244/269→249/274 and
217/233→223/236. These are descriptive surviving histories, not matched historical
opportunities or independent canonical hit-coverage certification.
Full BF/recorded-outs quantiles/histograms, relief distributions, final-budget
excess, per-game results and pins remain in `development-results.json`,
`fresh-results.json` and corresponding `*-records.json`. Separate raw-log checks
verify19252development/19604fresh removed-reliever budgets exactly; stable
entry-word sampling and runner-only PA exclusion remain intact. The old binary
and all input bytes are unchanged. Independent here means a separate arithmetic/
SQL/oracle path, not a separately delegated reviewer or independent data source.

The source-role means are descriptively closer, not calibration acceptance:
changing pitchers also changes actual matchup vectors and later game states.
No result establishes physical fatigue, rest, historical availability, predictive
validity or a representative full-season replay. Source-code/full-suite acceptance
and the local checkpoint are recorded separately in PROGRESS.

## Boundaries

Finite closed-cast exhaustion cannot be eliminated in all games without more
verified personnel or changing workload/entry policy. This task must quantify
that remaining limitation rather than certify 2019 usage, measured fatigue,
rest, predictive accuracy, full P-02 or G1–G5.
