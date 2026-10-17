# Design findings and matchup mathematics

Recorded 2026-10-17. Read the [requirements and summary](README.md) first.
Source IDs below refer to [SOURCES.md](SOURCES.md). Statements labeled
**proposal** or **estimate** are not tested rules or empirical accuracy findings.

## 1. Physical cards and playability

### Two-sided handedness layout

| Card | One face | Other face | Face-selection rule |
| --- | --- | --- | --- |
| Batter | VS LHP | VS RHP | Pitcher's effective throwing hand |
| Pitcher | VS LHB | VS RHB | Batter's effective hitting side |

A switch hitter normally batting left against a right-handed pitcher uses his
**VS RHP** face; that pitcher uses his **VS LHB** face. Do not select both faces
from a player's nominal `Bats S` label. Ambidextrous-pitcher exceptions need an
explicit era-appropriate procedure, not an ambiguous face choice.

Repeat the name, season, own handedness, running and defensive information on
both faces. Distinguish the player's own hand from the opponent-hand face label.
Multi-position players may need separate position ratings; a short team fielding
strip is a possible overflow aid, not permission to erase position differences.
A pitcher who bats or another two-way player may have separate batting/pitching
cards tied to one identity; that product detail remains open.

**Estimate:** a poker-size face can hold roughly 10–14 main result rows plus a
compact identity header and rating strip if artwork and descriptive statistics
are limited. A range such as `137–264` represents 128 chances in one line. More
possible rolls do not require printing every number. No actual-size layout,
font-size/accessibility test, duplex alignment test or handling test has yet
proved this estimate. Check ordinary lighting and practical reading distance.

### Proposed complexity budget

- Most PA should finish with one main roll and one result lookup.
- Some provisional chances may require one opponent or defensive check.
- Meaningful steals/advances may require a separate decision and roll.
- Aim for one compact double-sided shared aid, not many specialized charts.
- Treat lookup count, arithmetic, rule recall and card handling as the main costs;
  dice count alone is a poor measure of complexity.

These are design targets. We have not measured the fraction of PA requiring a
second check or established a game-duration claim. A modest accuracy improvement
may not justify a substantial increase in interruptions.

## 2. What established games reveal

### Strat-O-Matic: chance allocation plus conditional rules

Official explanations describe a first d6 selecting a batter column (1–3) or
pitcher column (4–6), then the sum of two d6s selecting the row [S01, S02]. Each
side is selected half the time. The 2d6 rows are not equally likely: 7 has six
ordered combinations, while 2 and 12 each have one. Some results use a further
d20 split; fielding and other conditional rules add detail.

The card chance is therefore not simply the player's raw statistical rate.
Unofficial Bundy cardmaking research describes compensating for contributions
from the average opposing card, fielding charts and other rules [S04, S05]. Those
articles explicitly warn that the company's complete formulas are not public.
Their historical constants are not timeless league baselines.

**Qualification to the initial discussion:** real Strat is not merely two
independent tables with every result unconditional and final. Advanced N/W power
is an existing cross-card interaction. Under reproduced rule 12.1, a W hitter
receiving a pitcher-card HR chance instead receives the corresponding SINGLE**
chance; a split's other outcome remains as printed [S02, S03]. Ballpark and other
rules also matter. A general criticism of a pure mixture is not proof that every
commercial edition lacks exceptions, or evidence that our design is superior.
The official digital help also describes an electronic "Correct Board Game
Excesses" option; an electronic correction cannot be assumed to exist in the
physical game or satisfy our offline requirement [S02].

### APBA: batter result interpreted using pitcher/context ratings

Public Basic Game walkthroughs describe two ordered d6s yielding 36 combinations
(11 through 66 with only digits 1–6). Read a number on the batter's card, then use
the relevant base-occupancy board. Pitcher grades affect designated results, while
defensive ratings affect other resolutions [S09]. Some cards use a second column
and reroll, so 36 initial combinations are not the entire game's resolution.

Publicly indexed Master Game instructions describe numerical pitching grades
1–30, strikeout/control symbols and further modifiers [S08]. This is not a
50/50 independent batter/pitcher outcome-table architecture. Do not infer APBA's
complete current card-generation or grading algorithm from these mechanics.
The full official PDF could not be retrieved in this research; the evidence
boundary is recorded in the source register.

### DYNASTY League: a direct three-d10 precedent

The publisher describes ordered red/white/blue decimal dice producing 0–999,
with 0–499 on the hitter and 500–999 on the pitcher, and opponent-handedness
columns [S10]. This is direct evidence that the proposed primary dice structure
is practical in an established physical game. It is not independent evidence of
its accuracy, speed, or suitability for our particular card size.

### Public GitHub generator: useful implementation evidence, not authority

Inspected `mbutler/strat-o-rama` at revision
`08e5e53286d58f13255f5b814bca01f474fb4e91` [S06]:

| File | Observed role |
| --- | --- |
| `player.js` | Computes estimated batter/pitcher chances from historical statistics using Bundy-derived formulas |
| `card.js` | Allocates chances across 33 weighted card positions, including d20 splits |
| `notes.txt` | Explains chance/subchance units and formulas |
| `README.md` | Describes PDF generation and explicitly disclaims official compatibility |
| `config.js` | Selects the input CSV files |

This is a genuine statistics-to-card program, not merely a renderer. Its batter
HR formula apportions the required overall HR chances and subtracts an expected
pitcher-card contribution. It also contains hardcoded league constants and
zero-clipping of negative chances. Its configuration selects 2018 CSV files while
the calculation file labels its baseline constants as 2016 statistics. These
are reasons to investigate units, calibration and applicability, not reuse the
code blindly. The generator was **read, not installed or run**; no probability
reconciliation or benchmark of its generated cards was performed.

Other located tools render existing card text or parse game reports [S07]. A
card printer does not establish how player statistics become valid probabilities.
The bibliography also records outside opinion pieces that were not treated as
proof of proprietary formulas or numerical accuracy.

## 3. League-relative matchup construction

### Consistent inputs first

For one event in one defined opportunity set, let:

- `B` be the batter's event rate against neutral opposition;
- `P` be the pitcher's event-allowed rate against neutral opposition;
- `L` be the matching league/context baseline;
- `C_B` and `C_P` be conditional chances printed on the respective selected cards.

Ideally B/P are stabilized estimates adjusted for opponent strength, park and
other modeled effects, not raw marginals assumed to be neutral automatically.
Match season/league and the relevant handedness context. Do not apply an
additional generic platoon bonus after encoding that same effect in both faces.

Keep denominators consistent: K/PA, BB/PA and HR/PA are not interchangeable with
batting average H/AB. Define how intentional walks, sacrifices, interference and
runner-only events enter or leave each risk set. Deliberate IBB is a manager
choice. Runner-only events must not consume a completed PA or advance the batting
order, and compound plays such as K plus a runner advance remain possible.

### Why averaging raw rates fails

A raw 50/50 average gives:

```text
q_raw = (B + P) / 2
```

If B = 30% K and an average pitcher has P = L = 20%, this predicts 25%, not the
batter's intended 30%. It pulls the player back toward average solely because of
card selection, on top of any intentional statistical shrinkage already applied.

The remembered pitcher-versus-league adjustment is the additive model:

```text
q_add = B + (P - L) = B + P - L
```

Example: B = 30%, P = 25%, L = 20% gives 35%. The pitcher is five percentage
points above the baseline, so add five percentage points to the batter's rate.
A lower pitcher event-allowed rate subtracts instead. This is a model assumption,
not a proof of the true probability of a particular matchup.

### Encoding the additive model in equal-selection cards

A symmetric centering is:

```text
C_B = 2B - L
C_P = 2P - L
q   = (C_B + C_P) / 2 = B + P - L
```

For that example, print 40% K on the batter card and 30% on the pitcher card;
the overall K probability is 35%. Against a neutral 20% pitcher card, the batter
gets 30%. Against a neutral 20% batter card, the pitcher gets 25%.

More generally the batter card must supply `2B - mean_opposing_pitcher_card`
and the pitcher card must compensate for its corresponding opponent-card mean.
The allocation need not be symmetric, and actual opposing exposure and conditional
rules matter. Do not label `2B - L` as Strat's full proprietary construction rule.

Changing the card-selection weight does not by itself remove additivity. If the
batter card is selected with probability `w` (strictly between zero and one), a
symmetric-baseline allocation for the same additive target is:

```text
C_B = L + (B - L) / w
C_P = L + (P - L) / (1 - w)
q   = w*C_B + (1 - w)*C_P = B + P - L
```

This only works as a direct table when the resulting entries are valid
probabilities. Under a common normalized multi-outcome taxonomy their unrounded
card entries sum to one, but individual entries may still be invalid.

### The negative-chance limitation

With B = 5% K, P = 5% K, L = 20%, the additive result is **−10%**. The symmetric
batter card alone would require `2*5% - 20% = -10%` K. Such an entry cannot be
printed as a probability. Other extremes can require a value above one.

Another way to see the constraint: a pitcher table containing 20% K and selected
half the time contributes 10% K to every opponent unless another rule can modify
that contribution. A zero-K batter table cannot subtract those strikeouts.

Clipping negatives, then filling or renormalizing the table, changes the intended
matchup rates. Record the resulting error; do not silently call it calibrated.
Changing baseline allocation can move constraints around but cannot make an
unrestricted nonlinear matchup surface exactly separable into two fixed tables.
More digits improve granularity, not this structural limitation.

## 4. Nonlinear reference models and estimation

A binary odds-ratio/log5 baseline combines batter and pitcher relative to league
odds rather than adding raw percentage-point deviations:

```text
q_log5 = (B*P/L) / ((B*P/L) + ((1-B)*(1-P)/(1-L)))
```

With the 5% / 5% / 20% inputs, it gives `4/365`, approximately **1.095890%**, not
−10%. For strictly interior inputs its output is between zero and one. Raw zero
counts still need defensible statistical smoothing; the formula is not itself a
solution to sparse-data estimation.

For mutually exclusive outcomes k, one generalized-log5 baseline is [S11]:

```text
weight_k = B_k * P_k / L_k
q_k      = weight_k / sum_j(weight_j)
```

That is one normalized multi-outcome model. Do not compute unrelated binary
probabilities and assume they automatically sum to one. An explicitly defined
conditional outcome tree is another coherent construction; its risk sets and
fallback paths must be included in the full joint probability calculation.

Recommended comparisons, not a declaration of the best algorithm:

- Smoothed league-only and player baselines.
- League-adjusted additive/card-mixture baseline with its invalid cells exposed.
- Smoothed generalized log5.
- Joint hierarchical or regularized multinomial/conditional alternatives that
  account for opponent effects and share information for sparse players.

Published Bayesian log5 work studied binary on-base events; regularized
multinomial work addresses multiple outcomes [S12, S13]. A newer linked-matrix
factorization paper targets matchup batting averages [S14]. None alone proves
that its method is the best complete baseball simulator or best printable game.
Newness and model complexity are not selection criteria by themselves.

Fit and select shrinkage strengths on appropriate training/validation data.
Player-specific platoon effects, especially on rare events, often require strong
pooling rather than literal reproduction of a few L/R opportunities [S15]. Keep
legal zero-observed outcomes positive through justified priors; structurally
impossible events remain zero. Do not use an unexplained fixed epsilon.

Selected-season cards can aim at hindsight reconstruction. That is different
from underlying-ability estimation or forecasting. Record that choice before
judging bias or tuning shrinkage. No claim here establishes any player's true
matchup probability.

## 5. Exact decimal-dice resolution

Three ordered fair d10 digits give 1,000 equally likely numbers. Concatenate,
**do not add**, the dice. Use distinct colors or fixed positions for hundreds,
tens and units. For dice marked 1–10, read 10 as zero. Treat 000 as the lowest
number, not as a special extra result or an accidental 1,000.

### Split selection within the primary number

```text
000–499 -> batter card
500–999 -> pitcher card
```

Each selected card has 500 equally likely entries. A single entry represents:

- `1/500 = 0.20%` conditional on that card being selected;
- `1/1000 = 0.10%` of all primary PA rolls.

Example: 40 batter-card K entries plus 160 pitcher-card K entries give
`(40 + 160)/1000 = 20%` K overall. Do not divide their combined count by 500.
One slot's 0.10-percentage-point overall contribution corresponds to **0.6
expected events in 600 opportunities**. That is a step size, not a promised
maximum multi-outcome rounding error.

### Independent selector with a full table on each card

A fair d6 with 1–3 selecting batter and 4–6 selecting pitcher, independent of the
three d10s, permits a full 000–999 table on each card. One entry then contributes
`(1/2)*(1/1000) = 1/2000 = 0.05%` overall, or 0.3 expected events per 600 PA.
The 6,000 raw joint die combinations reduce to 2,000 equally weighted
card-plus-number entries if three selector faces do the same thing on each side.

| Primary arrangement | Entries per selected card | Overall increment from one entry |
| --- | ---: | ---: |
| Three d10s split into two halves | 500 | 0.10 percentage point |
| Independent 50/50 selector plus three d10s | 1,000 | 0.05 percentage point |
| Four d10s split into two halves | 5,000 | 0.01 percentage point |

Four full primary digits are not the same arrangement as a fourth die used only
as a redundant two-way selector. All figures assume fair, independent dice and
no subsequent outcome modifiers; full mechanics require their own enumeration.

### Conditional checks and precision tradeoffs

A rare event can be finer than the primary grid without adding permanent dice.
For example, trigger on one of the 1,000 initial numbers, then require a fresh
000–299 reroll: probability `1/1000 * 300/1000 = 0.03%`. The other reroll outcomes
need explicit consequences. Fit that branch jointly with the rest of the model.
Do not remove every tiny positive legal probability merely by rounding to zero.

Strat's primary three-d6 roll has 216 equally likely elementary combinations;
a d20 split can refine appropriate entries on a theoretical 4,320-combination
joint grid [S02, S04, S06]. Not every row has the same weight or is split. Thus
1,000 primary decimal results are not automatically finer than all Advanced
Strat rules, and slot count alone is not an accuracy benchmark.

**Recommendation:** begin with three d10s and the split-card primary number.
Try an independent selector only if its extra resolution or another well-defined
function earns its handling cost. Use a fourth primary digit only after measuring
rounding losses. Typical statistical uncertainty can exceed these slot sizes;
rare outcomes and extreme players still need explicit diagnostics.

## 6. Candidate fully analog interaction mechanics

The simplest baseline is selected-card lookup with league-relative chance
construction. Compare it against a limited-interaction version rather than
assuming every PA needs arithmetic, a reroll or a universal pitcher grade.

| Provisional reading | Candidate additional information |
| --- | --- |
| Routine resolved result | None |
| K chance | Opposing contact/strikeout information |
| BB chance | Opposing discipline/control information |
| HR chance | Opposing power/HR-prevention information |
| Grounder toward SS — range | Appropriate shortstop range/error information |

These are **candidate branches, not finished rules**. Define the consequence of
every failed check and fit the primary chance and confirmation probabilities
together. Adding an ad hoc pitcher "save" after a finalized result can double-count
an effect or destroy calibration. Targeted interactions are not a novel discovery;
existing games already use versions of them.

Alternative standalone designs can combine signed ratings through a small shared
lookup. This may better approximate a nonlinear model but costs addition and
chart use. An outcome-specific K/BB/HR representation has more expressive power
than assuming one pitcher grade equally describes every skill. The preferred
mechanic remains an empirical accuracy-versus-playability decision, not an
approved implementation.

Do not reserve a generic d6 face for "fielding" without modeling the implied
frequency and context. A 1-in-6 unconditional branch imposes a 16.7% primary
frequency, not a measured baseball fielding-opportunity distribution. Let contact
and context identify a defensive chance. If the same die controls several things,
enumerate their joint effects rather than assuming independence.

Generate fresh secondary randomness when required. A digit already conditioned
by the triggering primary range is not automatically an independent secondary
roll. A deliberately derived conditional mapping is possible, but must be proved
and may be harder to play than a reroll.

## 7. Defense, running and baseball consequences

Keep distinct mechanisms even if their printed scales are compact:

| Ability | Appropriate effect |
| --- | --- |
| Running/advancement | Eligible extra-base decisions and outcomes |
| Steal attempt tendency | Frequency of eligible attempts, separate from success |
| Steal ability | Success conditional on attempt and context |
| Range by position | Conversion of appropriately located contested contact |
| Error tendency | Mishandling per eligible opportunity |
| Outfield/infield arm | Eligible advancement decisions and throw outcomes |
| Catcher throwing/blocking | Eligible steal and pitch-related outcomes |
| Pitcher hold/endurance/WP tendency | Corresponding runner/workload/pitch mechanisms, not all-purpose hitting penalties |

A fast runner need not be an aggressive stealer. Raw assists do not identify arm
strength; raw steals do not identify attempt rate per opportunity. `1 - fielding
percentage` is not the chance of an error on every ball in play.

Resolve **contact type/location/depth -> fielding opportunity -> out/hit/error
-> hit value and eligible advancement**. A "grounder toward shortstop — range
check" is unresolved contact. A finalized single is already an outcome; do not
independently convert it to an out as an extra range roll. Clean hits, routine
outs and contested chances can coexist only if their frequencies and neutral
fielding consequences are calibrated jointly.

Shared rules can cover force eligibility, ordinary advancement, third-out scoring,
double plays, tag-ups, substitutions, walkoffs and era differences without printing
every consequence on every player card. Rare rules must remain correct even if
infrequent; speed is not permission to ignore scoring or lineup legality.

## 8. Existing data, uncertainty and rights

The repository has an audited internal 2019 play-by-play foundation, player
outcome-count pipelines and experimental matchup/calibration work. Start by
checking whether the accepted local snapshot supports the card-specific
aggregations; a new source download is not automatically necessary.

This is not a claim that current ratings are finished or card-ready. The live
[progress ledger](../docs/PROGRESS.md) governs current verification. Historical
and current artifacts explain the limits:

- [2019 coverage inventory](../docs/data-audits/2019.md).
- [Internal data-use decision](../docs/data-audits/2019-go-no-go.md).
- [Matchup research](../docs/modeling/matchup-probabilities.md), an older planning
  note whose historical status text does not override current progress.
- [Player calibration v2 results](../docs/modeling/player-calibration-v2-results.md),
  which distinguish development evidence from independent predictive validation.

Retrosheet-derived data provide excellent outcome evidence, not a complete ball
tracking system. Recorded trajectory/location, deduced location, actual fielders
and eligible difficult opportunities are different evidence. Inferring location
from who fielded a ball does not independently establish the fielder's range.
Positioning, teammates, parks, scorer judgments and missing opportunity quality
can confound defensive estimates. Use neutral/group priors with uncertainty when
individual range or arm is not identified; do not fabricate precision.

Public/commercial rights remain a separate decision. The baseball.computer site
states CC BY-SA 4.0 for additional contributions, while its repository LICENSE
states CC BY-NC-SA 4.0 [S16, S17]. Scope needs clarification before distribution;
this is not a legal conclusion about derived cards. Preserve required Retrosheet
attribution [S16]. Public GitHub code is not a grant to copy commercial card art,
logos, proprietary card sets or every embedded data source. No publisher outreach,
data/card publication or release is authorized by archiving this research.

## 9. Recommended validation and next experiment

Separate three sources of error:

1. **Estimation/model error:** how well the reference predicts appropriate held-out
   outcomes or reconstructs the selected season under its stated objective.
2. **Mechanical approximation error:** the gap between reference probabilities and
   the complete physical procedure, including rounded entries, confirmation
   checks, failed checks, fielding and conditional runner outcomes.
3. **Sampling/usage variation:** dice luck, opportunities, lineup and tactical
   decisions in actual games and replays.

Preregister data versions, exposure definitions, training/selection/evaluation
splits, metrics and tolerances before fitting or comparing new candidates. A
reused development benchmark is not fresh validation. Use log loss/Brier and
conditional calibration for predictive comparisons, and fixed-opportunity
reconstruction diagnostics for hindsight cards. Do not confuse matching league
totals with correct individual or conditional mechanisms.

For an authorized prototype:

1. Select a small contrasting player set: low-K contact hitter, high-K slugger,
   low-power hitter, platoon specialist, elite and ordinary pitchers, plus sparse
   profiles and representative defenders/runners. Only then expand to two teams.
2. Build a league-adjusted baseline and coherent nonlinear reference comparison.
   Record invalid additive/card cells; do not fix them by silent clipping.
3. Generate a minimal physical procedure. Enumerate all primary rolls and weight
   every conditional branch to calculate exact probabilities. For complicated
   stateful procedures, supplement with reproducible simulation; a smoke test
   does not establish the full probability distribution.
4. Report errors by event, opportunity band, handedness and extreme matchup, not
   just the mean. Check legal rare outcomes, probability normalization and that
   every printed range is covered exactly once. Evaluate fielding counterfactuals
   and steal/advance attempts as well as successes.
5. Print at 100% scale. Test face selection, contrast, legibility, dice ordering,
   lookup count, second-check frequency, teaching time and full-game pace.
6. Before a full-season claim, run fixed-opportunity and free-play comparisons,
   accounting/legal-state checks and many independent replay replications. The
   existing product plan calls for at least 100 season replications, target 200
   for rarer events; that remains a future gate, not work done by this archive.

Potential improvement over a specified commercial edition must name the metrics
and comparison conditions and show evidence. Access to rich public data, more
dice outcomes, or a more recent paper does not demonstrate superiority.

The immediate recommendation is **a three-d10 analog baseline, compact L/R faces,
one small shared aid and only evidence-justified interactions**, not a complete
season's artwork or a mandatory second check on every PA. The final mechanism,
thresholds, rare-event branches, parks/endurance rules and rating scales remain
open. This archive does not authorize implementing or fitting that prototype.

## Reproducible arithmetic

This exact-rational self-check covers the illustrative equations and fair-dice
counts, not player fitting, production code or complete game behavior. The
[README](README.md) gives the extraction command. It writes no files and makes
no network calls.

```python
from fractions import Fraction as F
from itertools import product

B, P, L = F(30, 100), F(25, 100), F(20, 100)
cb, cp = 2 * B - L, 2 * P - L
assert (B + P) / 2 == F(275, 1000)
assert (B + L) / 2 == F(25, 100)
assert (cb + cp) / 2 == B + P - L == F(35, 100)
assert (cb + L) / 2 == B
assert (L + cp) / 2 == P
for w in (F(1, 3), F(1, 2), F(2, 3)):
    bw = L + (B - L) / w
    pw = L + (P - L) / (1 - w)
    assert w * bw + (1 - w) * pw == B + P - L
print('PASS: raw averaging, league compensation and weighted additive identities')

low = F(5, 100)
assert low + low - L == -F(1, 10)
assert 2 * low - L == -F(1, 10)
log5 = (low * low / L) / (low * low / L + (1 - low) ** 2 / (1 - L))
assert log5 == F(4, 365) and 0 < log5 < 1
print(f'PASS: invalid additive example; binary log5 = {float(log5):.6%}')

rolls = [100 * h + 10 * t + u for h, t, u in product(range(10), repeat=3)]
assert len(rolls) == len(set(rolls)) == 1000
assert min(rolls) == 0 and max(rolls) == 999
assert sum(r < 500 for r in rolls) == sum(r >= 500 for r in rolls) == 500
assert F(40 + 160, 1000) == F(20, 100)
assert F(1, 500) == F(2, 1000)
assert F(1, 1000) * 600 == F(3, 5)
print('PASS: 1000 unique primary rolls, 500 per card, 20% combined K example')

joint = list(product(range(1, 7), range(1000)))
entries = {('batter' if d <= 3 else 'pitcher', r) for d, r in joint}
assert len(joint) == 6000 and len(entries) == 2000
assert sum(d <= 3 and r == 0 for d, r in joint) == 3
assert F(3, len(joint)) == F(1, 2000)
assert F(1, 2000) * 600 == F(3, 10)
assert F(1, 10000) == F(1, 2) * F(1, 5000)
assert F(1, 1000) * F(300, 1000) == F(3, 10000)
print('PASS: independent selector, four-digit and rare conditional resolutions')

strat = list(product(range(1, 7), repeat=3))
assert len(strat) == 216
assert sum(a <= 3 for a, b, c in strat) == 108
assert sum(a <= 3 and b + c == 7 for a, b, c in strat) == 18
assert sum(a <= 3 and b + c == 2 for a, b, c in strat) == 3
assert len(strat) * 20 == 4320
print('PASS: weighted Strat primary rows and theoretical d20 refinement grid')
print('All documented arithmetic checks passed; no game-accuracy claim.')
```
