# Pitcher usage diagnosis v1 — frozen-season, read-only attribution

## Disposition and scope

`P-02-pitcher-usage-diagnosis-v1` diagnoses the retained managed seasons at code
`fb2cde0`, inspected from clean `44fe0d0`. **Managed v1 remains a pitcher-usage
NO-GO**, not a validated usage model. [Frozen evaluation](managed-season-v1-results.md)
and its thresholds are unchanged. No games were executed, sources acquired,
policies/ratings/outcome probabilities changed, or gates signed off.

The strongest identified mechanism is the restricted starter opportunity set.
Relief evidence establishes continued use despite negative pace and the priority
rules permitting it, but does not identify each within-game selection's cause.

## 1. Top-five priority displaces the historical extra starters

`managed_inputs.rotations` sorts club starters by historical starts, then stable ID.
`build_game` offers only the top five whenever any principal is rested and unoccupied.
Only exhaustion of that set opens the remaining rotation. `Policy.starter` can
substitute for substantial overuse **within the offered set**; it cannot promote an
underused extra past a rested principal. The accepted scheduled pointer advances
once per game. Global rest and scarce shared-starter allocation remain enforced.

An integer-day reconstruction using raw preceding pitching appearances reproduced
**all 14,574 starter decisions**, reasons, budget scales and accepted rotation turns
across the three retained seasons. Rest logic also matches the unchanged production
helper on 4,096 six-day histories, for both starter/reliever argument values
(8,192 comparisons), plus explicit boundary controls.

| Start allocation | Historical | Seed 1 | Seed 2 | Seed 3 |
|---|---:|---:|---:|---:|
| Principal top-five club entries | 3,680 | 4,753 | 4,749 | 4,753 |
| All other supported club starters | 1,178 | 105 | 109 | 105 |
| Total | 4,858 | 4,858 | 4,858 | 4,858 |

Thus approximately 98% of simulated starts go to a group that historically made
approximately 76%. This is direct opportunity-allocation evidence, not an estimate
of how many pitcher-band failures a changed policy would repair.

TEX Kolby Allard (`allak001`) is principal rank 5 despite only 9 historical starts:
he receives 29/30/29 simulated starts and 378/366/358 outs versus 136 actual outs.
The 0.8 starter-budget signal already applies to 25/27/25 of his starts. Bounded
shortening does not undo the excess starting opportunities.

All 24 uncomparable qualifying pitcher-team entries in each seed are outside the
principal five. Among these, 18/18/17 entries were rested, unoccupied extra candidates
on at least one spot-start day but never selected; earlier extra candidates still
precede them. Extra candidates restart their offered order at rank 6, rather than
receiving the principal rotation's advancing pointer. Underuse alone does not
change that ordering. The remaining missing entries had no such spot-start day.

SDN Dinelson Lamet (`lamed001`), rank 6, is a particularly clear trace: 14 actual
starts/219 outs; rested on all 162 SDN game dates in every seed, **zero days on which
he reached the extra-starter candidate branch**, zero cast inclusions and no
simulated pitching appearance. It is not his own modeled rest that excludes him.

## 2. Missing appearances have two distinct paths

| Missing qualifying entries | Seed 1 | Seed 2 | Seed 3 |
|---|---:|---:|---:|
| Never present in a pitching cast | 24 | 22 | 20 |
| Offered as relief but never appear | 0 | 2 | 4 |
| Total | 24 | 24 | 24 |

Never-cast entries cannot receive native pitching opportunities. But cast inclusion
alone is insufficient: seed 2 offers KCA Mike Montgomery (`montm002`) in 146 casts
and SEA Matt Wisler (`wislm001`) in 143, without a pitching appearance. Seed 3 offers
ATL Bryse Wilson (`wilsb003`), Montgomery, SEA Sam Tuivailala (`tuivs001`) and Wisler
in 162/141/162/145 relief casts respectively, again without appearances. All are
in the native short-workload class. Tuivailala's seed-3 pace is positive in 146
casts and Wisler's in 118: a positive preference does not guarantee selection.

These remain **uncomparable/null**, not measured zero-out rows in the frozen usage
comparison. Counting zero cast inclusions is a separate observed input census.
The diagnosis does not change the registered denominator or fabricate appearances.

## 3. Relief pace is subordinate and is not a cap

Python first excludes principal starters from relief, filters supported roles and
global modeled rest, then exports pace. Native `Cast::next_available` orders legal
unused pitchers by `(workload-class mismatch, -pace, stable ID)`. Long means the
exact integer comparison `relief_pa_sum >= 6 * relief_outings`; long is preferred
through inning 5 or at an absolute score gap of at least 4, short otherwise.
A matching-class negative-pace pitcher outranks an opposite-class positive-pace
pitcher. Equal class/pace still uses stable ID, not a continuous target deficit.
`pace` itself is neutral within its dead band and saturated at -1/+1 outside it.

| Retained relief appearances / outs | Seed 1 | Seed 2 | Seed 3 |
|---|---:|---:|---:|
| Long class | 5,990 / 27,552 | 6,045 / 27,933 | 6,098 / 27,775 |
| Short class | 11,575 / 29,306 | 11,602 / 29,191 | 11,538 / 29,192 |
| Appearances with negative pregame relief pace | 3,993 | 4,084 | 4,115 |

Examples (outs are integers, not decimal IP):

- KCA Brian Flynn (`flynb001`), rotation rank 10: 88 actual outs, 470/449/383
  simulated, no simulated starts. His relief samples total 122 terminal PA across
  9 clean training outings, hence long class. He appears with negative pace
  62/61/59 times; the negative signal is present, not missing from the export.
- ARI Jon Duplantier (`duplj001`), rank 10: 110 actual outs, 463/484/480 simulated,
  no simulated starts. His 106 PA / 12 relief samples also make him long class.
  Negative-pace appearances number 86/92/87.

Retained examples include seed-1 `ARI201904050`: negative-pace long Duplantier
appears, while positive-pace short Andrew Chafin (`chafa001`), Yoshihisa Hirano
(`hiray001`) and T.J. McFarland (`mcfat001`) are in the pregame cast but do not pitch.
The diagnostic records ten bounded examples per seed. **These are not proof of the
exact alternatives at an entry:** the accounting TSV contains final actor totals,
not entry order, inning/score, first-batter obligation or contemporaneous bench state.
It cannot establish which class was preferred at each entry, attribute a numerical
share of failures to class versus ID ordering, or separate budget/tail effects.
No native replay was used to fill that evidence gap.

## 4. Club-specific targets coexist with global role support

ATL Kevin Gausman (`gausk001`) has 16 actual starts, 240 starter outs and zero actual
ATL relief outs, but is only rotation rank 6. Across seeds he has no spot-start
candidate day and no simulated start. He is nevertheless offered in 152/151/151
relief casts, producing only 7/8/7 relief outs. Relief admission accepts an empirical
relief role observed anywhere in the source, not just a positive club relief target:
Gausman also has CIN relief history (61 actual relief outs), and his exported short
relief workload is 82 PA / 14 clean training outings. Global role support therefore
lets him pitch for ATL in relief without restoring ATL starter opportunities.
This is a trace of existing code, not an invented ATL relief observation or a new
eligibility/role rule. Rest remains global; targets remain club-specific.

## Verification, artifacts and reproduction

External root: `/tmp/bbsim-pitcher-usage-diagnosis-v1/`.

- `diagnose-first.py`, `first.stdout/.stderr`, `first/pool.json`: preserved initial
  analysis-script failure (`KeyError: 'pitcher'` at an aggregate lookup). The actual
  aggregate key is `outs`; seed-1 starter/native-ledger checks had already passed.
  This was not a simulator failure or a reason to change any acceptance assertion.
- `diagnose.py`, `second.stdout/.stderr`, exclusive `second/`: process `proc_9a47`,
  exit 0/38s, `DIAGNOSIS_COMPLETE`. Three pinned batch receipts, all consumed batch
  artifacts/input pins, full source authentication and **43,765 unchanged pinned
  paths**; all 7,287 native accounting outputs parsed and pitcher histories/club
  totals/starts/appearances/outs reconciled against ledgers and comparisons.
  JSON retains every pitcher row, missing entry, exposure count and class summary.
- `check.py`, `independent.stdout/.stderr`, `independent.json`: process `proc_b2d5`,
  exit 0/12s, `INDEPENDENT_CHECK_COMPLETE`. Separate literal TSV census importing no
  simulator/diagnostic helpers independently reconciles appearances, BF, outs,
  starts, class/pace exposure and summary counts; rechecks all pinned paths.

Exact commands:

```sh
PYTHONPATH=. python3 -B -W error /tmp/bbsim-pitcher-usage-diagnosis-v1/diagnose.py
python3 -B -W error /tmp/bbsim-pitcher-usage-diagnosis-v1/check.py
```

Completed destinations are exclusive: copy the two current recipes into a **new
external directory** before reproducing; their output paths derive from that
recipe directory. Do not overwrite the retained original directories. All raw
season inputs remain read-only and separately sealed with managed v1. The private
diagnosis seal and exact document checks are recorded in the live progress ledger.

No engine/Python production code changed. Full project suites, new native seasons,
full-seed repeats, UI checks and counterfactual policy evaluations were **not run**
for this documentary analysis. Historical engineering passes are not new proof.

## Next bounded step

Preregister a separately versioned successor contract for starter-opportunity
allocation across supported club starters and relief competition/role budgeting,
with explicit coverage checks and retained entry-decision context. Freeze its
metrics and comparison populations before implementing/evaluating it. The current
trace motivates that design; it does not prove any successor will pass or authorize
relaxing v1 thresholds, rest, legality, or changing outcome probabilities.
