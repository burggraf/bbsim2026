# P-02-text-game-manager-v2 — workload and context, 2019 pilot

**Follow-up:** the [conditional usage audit/v3 correction](text-game-usage-audit-v1.md)
removes the extra early-performance shortening rule; the original v2 contract
and results below remain historical evidence. PROGRESS owns adoption status.

## Contract fixed before implementation or examining new cohort results

Maintainer approved replacing the automatic seventh-inning hook after seed67
removed Fiers at6IP/18BF/0H/0BB. This is a bounded manager improvement, not an
outcome-model refit, new season, official fatigue model or G1–G5 signoff.
Existing audited 2019 training-role workloads and assumed closed cast only.
No downloads, dependencies, new eligible personnel or measured pitch counts.

### Workload inputs and compatibility

Keep the existing cast-v1 exporter bytes/default and prior artifacts unchanged.
An explicit `--empirical` export produces `closed-pitching-cast-v2` with
`policy=empirical-pa-closed-cast-v2` and `usage:start:ID` / `usage:relief:ID`:
sorted training outing PA counts, comma-separated, or `-` for an absent role.
These must reconcile exactly with the existing S,G sums/denominators. Reuse
existing provenance, training filters, identity/ability and atomic export checks.
Missing selected-role exposure refuses; no neutral workload fabrication.

Both formats load. V1 uses its individual role mean, explicitly disclosed as a
mean-only fallback, not a fitted distribution. V2 samples one observed training
outing budget per pitcher entry, uniformly over outings, minimum1PA. Retain zero
PA observations; never silently substitute another role. Use the start role for
the starter and relief role for replacements. No across-season extrapolation.

A separate seeded manager RNG supplies one stable word per entry; multiply-high
indexing selects the budget without consuming outcome RNG words. Repeat a seed,
input snapshot and policy version exactly. No resampling every decision.

### Context policy (explicit provisional heuristics, not fitted coefficients)

- Remove the universal inning>=7 boundary hook and fixed27/6PA targets.
- Normally replace when the individual entry budget is reached, before the next
  event, provided an unused modeled reliever is available. Mid-inning hooks stay
  legal and inherited responsibility stays canonical.
- A starter with no observed runs conceded and *known* zero hits can continue to
  `max(entry_budget, ceil(1.5 * individual start-role mean))`. Missing hit coverage
  or partial game origin is not a no-hitter certificate. This is workload-limited,
  not an unconditional promise to finish every no-hitter.
- A scoreless starter with a lead greater than3runs can receive3extra PA beyond
  the entry budget. Close games receive no such low-leverage extension.
- A starter with at least9PA and4runs conceded while tied/trailing can be removed
  before the budget. Count runs conceded while pitching, including inherited
  runners; do not use ER or future game events to judge current performance.
- Keep the existing recorded relief order/assumed availability, no reentry,
  no-DH batting-slot replacement and last-pitcher exhaustion behavior.

These numeric choices are declared heuristics, not learned historical decision
probabilities. Empirical outing lengths themselves include tactical/injury/poor
performance removals; sampling them plus context does not identify physical
stamina. Individual relief samples can be sparse. Real rest, pitch counts,
handedness/leverage roles, injuries and other eras remain unsupported.

## Observable acceptance and diagnostic plan

1. Test-first: seventh-inning boundary alone cannot hook; individual role S/G
   changes decisions; no-hit extension survives an exhausted nominal budget but
   not its upper limit; positive runs/missing coverage disable that extension.
   Low-leverage and struggling controls differ; relief uses relief exposure.
2. Empirical export/loader tests reject mismatched sums, counts, missing roles,
   malformed/overflow samples and incompatible policy/format. Legacy export
   remains byte-compatible. Test sampler endpoints and entry stability.
3. Preserve substitution legality, responsibility, runner-only PA exclusion,
   no-DH/exhaustion and actual-current-pitcher probability tests.
4. Seed67: independently show the old18BF/no-hit state no longer triggers removal.
   Run complete new games with v1 and empirical casts and repeat each; report
   actual starter lines and final results, without requiring a no-hitter.
5. Fixed seeds1..1000: all games terminate/reconcile; independently parse PA/IP,
   hits/walks/runs, pitcher entry order and usage. Report starter BF/IP mean,
   spread/quantiles, early hooks, complete games, continuation after six hitless
   or scoreless innings, and source-role distribution comparisons. Preserve raw
   failures. No numerical historical-fidelity pass tolerance or predictive claim:
   this change's acceptance is behavioral/legal, and bias diagnostics stay visible.
6. Run full Rust, examples, Python (ResourceWarning errors), fmt, strict all-target
   Clippy, build and diff checks on a pinned final source snapshot. No push.

Status, exact commands, failed evidence and actual results live in PROGRESS.md.

## Native diagnostic snapshot (not historical-usage acceptance)

Evidence root: `/tmp/bbsim-text-game-manager-v2/`. `usage-results.json` preserves
full source/old/new distributions and binary/input pins; `usage-summary.json` is
only a concise view, not a replacement. Both v1mean and empiricalv2 ran fixed
seeds1..1000:2000new complete games, independently reconciled box counters/order/
notes and17020certifiable pitcher ER lines. Existing uncertainty is not official
scoring recertification. Outcome kernels, rating data and PA formulas are
unchanged; changing the active pitcher still changes the matchup vector.

Per-policy denominator is2000team starts (1000games):

| Policy | Gonzales mean BF | Fiers mean BF | Starts<18BF | Complete games | Final entry above observed source-role maximum |
|---|---:|---:|---:|---:|---:|
| Old fixed/boundary |25.009|24.705|0|0|170|
| Individual-mean fallback |23.505|21.629|285|4|340|
| Empirical outings/context |22.914|21.393|406|9|521|

Source training-role PA-proxy means are25.471/22.813, not matched-opponent official
BF targets. Starter IP distributions are stored as outs (divide by3 for decimal
mean innings, not baseball tenths). Source workload reports lack measured pitch
counts/fatigue and source IP distributions. Observed maxima are not hard stamina
limits. **Shorter outings and increased final-entry tail warnings remain visible;
this is not a validated2019usage model.** Sampling observed lengths plus an early
performance hook can overlap removal causes; the finite cast can force its last
pitcher to continue. No coefficient was retuned against this cohort. Future
conditional usage/bullpen modeling needs its own evidence and acceptance plan.

`context-usage-results.json` independently reconstructs SplitMix64 manager words
and4760mean/5237empirical removed-reliever budgets, confirming stable entry
sampling andrunner-only PA exclusion. Narrated hitless/scoreless six-inning
starters return for seventh: old0/13, mean15/15, empirical17/17. Scoreless starters
(hits permitted): old0/285, mean259/276, empirical222/235. These are surviving-
starter descriptive histories, not independent canonical hit-coverage certificates
or matched historical/predictive cohorts.

Seed67 repeats in detailed/radio with both casts: eight perfect innings by Fiers;
Healy singles opening ninth, then Dull replaces Fiers. Fiers8IP/25BF/1H/0BB/7K;
Oakland2–0. No no-hitter requirement or outcome reroll. Files:
`seed67-{mean,empirical}-{detailed,radio}.txt`, `seed67-results.json`.
