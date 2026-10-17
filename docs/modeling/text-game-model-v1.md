# Recovery model v1 — initial, explicitly provisional choices

Recorded 2026-10-16 **before implementing/comparing the integrated sampler**.
This is an initial integration model, not a fitted or calibrated baseball model.
The acceptance target is [text-game-recovery.md](text-game-recovery.md); current
status and failures remain in `../PROGRESS.md`.

## Source observations versus generated assertions

Reuse the validated 2019 baseline input and its exact provenance/exclusion checks.
Its nine PA counts are K/BB/HBP/1B/2B/3B/HR/ROE/other non-hit contact:
`42823,15142,1984,25947,8531,785,6776,1470,82245` (185,703 total).
The last category includes SF/SH/FC, **not just batter retirements**. Source result
labels separately report 79,825 InPlayOut, 1,143 SF, 721 SH and 556 FC, totaling
82,245. Preserve their identity as source summaries, not factual catch/throw proofs.

Restore the baseline's explicit `excluded_interference` count (61 for this source)
as a tenth PA parent. Use Jeffreys categorical weights `(n + 0.5)/(N + 5)` over
these ten parents. The generated subtype for that unresolved source label is an
**assumed catcher-interference first-base award**, not a finding about the 61
historical causes. It must be an explicit event/credit/error with forced advances,
never discarded or silently assigned to a hit/out. IBB's 753 historical labels
remain separately reported; intentional walks are a manager action, not a random
unconditional hitter outcome. No player-level rare-mechanism fitting in this slice.
Ground-rule doubles and inside-park home runs remain pooled source observations;
if not generated separately, disclose that limitation rather than label them as
implemented subtypes.

Every catch, contact direction, depth, clean handling, throw, retouch, force/tag
clock and named position assignment in these new games is generated/model-supplied.
No source-shape proxy proves those individual physical facts. The canonical
reducer owns state/legality; immutable model-level mechanism facts may carry the
additional geometry/clock selected by the constructor. Their tests and typed
constructor must bind them to that transition; old canonical validators must not
be advertised as validating facts they do not represent. A renderer only reads
that retained result and its canonical deltas. Do not weaken existing factual
validators merely to admit a broader model assertion.

## Provisional conditional policy (not tuned against output)

- K: 20% called, 78% swinging/direct caught, 2% uncaught/recovery family. Safe
  uncaught-third reach is possible only when first is empty or two are out;
  otherwise retire the batter under the appropriate legal recovery mechanism.
- Non-hit contact: retain all five old proxy parent counts
  `17188 ground,19528 fly,5379 line,536 pop,39614 unresolved`. Allocate the
  unresolved mass **as a generated prior**: 45% ground, 35% fly, 15% line, 5% pop.
  Add that allocation to the corresponding four modeled child weights. This
  consumes the whole parent mass; it does not establish historical subtypes.
- Ground fielder weights P/C/1B/2B/3B/SS: `5,1,12,25,22,35`.
  First-base unassisted has no self-assist. Direct outs otherwise name the
  thrower and first-covering receiver. Model assignment is separate from batting
  order and must exclude the DH and removed/bench players.
- Fly direction LF/CF/RF: equal; line-catch positions P/C/1B/2B/3B/SS/LF/CF/RF:
  `2,0,9,14,12,16,15,16,16`; popup positions: `2,23,15,15,15,25,2,1,2`.
  Catch descriptions cannot acquire an invented throw or extra out.
- Ground children include ordinary out, force/FC, force-relay DP and bunt
  consequences. Initial contextual masses: DP .18 when first occupied/<2 outs;
  FC .12 when a legal runner retirement is available; sacrifice/failed bunt
  .03 when first occupied, second empty/<2 outs. Ineligible child mass stays
  with ordinary legal ground-play alternatives; it is not dropped from the PA
  parent or retried after a failed constructor. Bunt failure share .20.
- Fly with third occupied/<2 outs: sacrifice/tag-up family .60; other eligible
  tag-up opportunities .20. Caught-ball double-play opportunity .02 with a
  before runner/<2 outs. Specify retouch, actual throw/tag and out/run timing in
  the modeled leaf; choosing a catch alone is not evidence for these effects.
- Hits keep their selected hit family. For a single, a second-base runner may
  score (.60), a first-base runner may take third (.30 if the lead runners
  clear it); for a double a first-base runner may score (.50). Resolve a joint
  legal movement vector, not independent colliding runner endpoints. An extra
  base attempt may produce a runner tag-out (.10 of the eligible extra-base
  attempts), **not turn the already-selected hit into a batter out**.
- ROE: fielding/throwing mechanisms .60/.40; select the actual modeled actor,
  error and causal movement before projecting. Do not assign every E to SS or
  claim whole-chance review where a canonical proof is not represented.
- At an eligible pre-PA decision: steal .025, pickoff .004, WP .012, PB .003,
  balk .001; use distinct transitions, not PA totals. Among legal steal targets,
  choose a lead unblocked runner; attempt success .75. Pickoff retirement .15;
  a failed non-error pickoff leaves state/cursor unchanged. No attempt with no
  eligible runner. Thresholds are disjoint categorical intervals, not successive
  independent tests that accidentally change the total attempt mass.
- IBB: .03 with first empty, second/third occupied, <2 outs, inning >=7 and
  score within two; otherwise ordinary PA. No future game result is inspected.
- Minimum manager policy: use only an explicitly supplied/model-assumed bench
  and bullpen; no invented historical eligibility. Starter target 27 batters,
  relief target 6; prefer changes at a half boundary from the seventh onward,
  permit threshold changes with inherited runners. Never re-enter removed
  players. If no reserve remains, the current pitcher continues; no forced
  winner or unlimited invented bench. Keep responsible pitchers on inherited
  runners. Recorded appearance-cast order/availability is disclosed hindsight.

These initial values are **not acceptance tolerances** and must not be adjusted
merely to obtain pleasing seed 1/2/3 transcripts or green mechanism coverage.
Deterministic constructed examples cover rare leaves separately. Record a dated
reason for substantive changes before any subsequent fitting/comparison. Never
hide a constructor failure by falling back to a generic retirement or resampling.

## 2026-10-17 — specification refinement and incomplete implementation checkpoint

Before the next integrated sampler run, fix the previously unspecified ordering
for caught-air children: consider the .60 scoring-retouch fly opportunity first,
then a .02 caught-ball runner-tag double play among open next bases, then a .20
other tag-up from first/second. For that last attempt use a provisional .10 tag-out
risk; this additional air-attempt risk is a **new generated prior**, not a measured
source rate. Prefer the lead unblocked runner. A completed catch precedes the
legal retouch and attempted advance; a subsequent tag is not a force or GIDP.
A receiver already holding the caught ball tags unassisted, never throws to self.
This resolves an unspecified joint sequence without fitting or selecting attractive
seed results. The old held-only canonical air proof does not validate this compound
geometry: the sealed model mechanism retains it and the canonical reducer checks
state, timing, actual deltas and supplied credits.

**Historical interim checkpoint, superseded by the dated refinements below:**
The connected command remains explicitly `text-game-recovery-v1-incomplete`.
It now implements concrete single-runner first-base force choices (including failed
bunts) and the .60/.40 error split. It still does **not** implement the full target
policy: multi-runner FC/failed-bunt cases, uncaught K, contextual hit advances/tag-outs
and broader personnel choices remain pending. In particular, the partial ground
sampler supports its DP/bunt interval only with first occupied/second empty; a
failed-bunt share is currently implemented only with a lone first-base runner.
These are integration gaps, not calibrated zero probabilities or a revised target.
The hit worker confirmed a canonical blocker: typed hit values currently refuse
any same-event out, including a separate runner tag. Do not erase that runner-out
mass or drop the hit classification to make the projector appear complete.

## 2026-10-17 — next contextual ground correction (before implementation)

Keep the existing priors; remove implementation-only exclusions of other occupied
bases from the selected First→Second force/FC and First-force/First-relay DP.
In these bounded leaves, other before-runners hold; after the First runner is
forced out their force is removed. No guessed additional force retirement,
scoring, advance or changed responsibility is supplied. The FC batter replaces
that one retired runner's carried pitcher charge under Rule9.16(g), with secondary
runners unchanged; more complex replacement chains still refuse. Failed bunts
use this same failed force-at-Second attempt with no sacrifice credit. Successful
bunts remain the separately supported First→Second/First-throw family. Checks
must cover all occupied-First masks, outs, halves, actual infield actors, held
secondary identities, canonical terminal clearing, credits and codec agreement.
This fills common contextual cells; it does not fit rates or waive the remaining
hit, uncaught-K and personnel recovery target. The sampler may choose the direct
DP/FC with any occupied First. The failed-bunt child must also reach all four
occupied-First masks; only successful bunt requires Second empty. Do not wrap
both children in the successful-bunt eligibility check. Preserve .03 × .20 = .006
failed-bunt mass and .03 × .80 = .024 successful-bunt mass by disjoint ground
intervals [.30,.306) and [.306,.33), respectively. The latter applies only with
Second empty; other eligible ground mass selects an ordinary concrete ground
out directly, never retries a constructor or relabels a sampled successful bunt.
Held Third suppresses neither child. Constructed sampler tests preregister
seeds1..=3000 for each mask1/3/5/7, outs0/1, both halves, using synthetic ground-only
contact counts; these are repeated opportunities, not selected ordinary games.

## 2026-10-17 — hit/uncaught-K joint policy refinement (before sampler integration)

The wired revision is `text-game-recovery-v1-joint-hit-k-incomplete`, distinct from
the earlier incomplete-cohort version. It is not acceptance or calibration.
After wiring, the first1,000 seeds completed their native/codec/accounting checks
and exercised all seven new hit/K contexts, but the unchanged rare failed-bunt /
held-Third coverage assertion failed. Keep that failure (`joint-hit-k-connected-first.*`)
and the assertion. Before further sampling, fix the ordinary verification range
to seeds1..=10,000 once, retaining every game, unchanged rates and all assertions.
This expands coverage evidence, not a fitted rate, relaxed tolerance or selected
success cohort. Constructed48,000 ground opportunities already pass all masks,
outs0/1 and both halves; they do not substitute for ordinary game verification.
If the fixed expanded range still misses the rare context, report the unmet check
rather than adjusting rates, excluding seeds or automatically expanding again.

The earlier hit advance priors now become complete supplied joint endpoints,
resolved lead-to-trail, not colliding independent rolls. Singles always bring Third
home; Second chooses Home (.60) or Third, then First may choose Third (.30) only
if it is clear, otherwise Second. Doubles bring Second/Third home and First chooses
Home (.50) or Third. Triples bring all before-runners home. Choose the lead eligible
extra-base attempt, with the existing .10 tag risk; only that before-runner can be
out, never the selected safe hitter. The other Home endpoints cross before the
throw/tag in lead-to-trail origin order, with an exact native timing set. A safe
late-bottom winning opportunity is stopped **before** optional throw/tag selection,
with explicit Third/Second/First crossing order and official hit value from the
actual winner. No retry, second batter-out roll, repaired invalid supplied vector
or suppressed hit credit is permitted.

Keep the .20/.78/.02 called/caught-swinging/uncaught K parent split. An ineligible
uncaught batter (First occupied, <2 outs) is automatically out with held runners
in this integration; automatic-out plus same-delivery advancement remains a
separately disclosed gap, not a canonical impossibility. Eligible uncaught K uses
provisional .75 pitcher-wild/.25 catcher-passed handling assessments. With existing
runners, choose joint one-base advancement (.75) before any recovery outcome; a
winning advance stops without a later recovery out. Otherwise among advancing
opportunities choose safe batter reach (2/3) or catcher-to-First recovery out (1/3).
The winning-stop check requires fewer than two prior outs: a later before-First
third out suppresses the physical Home crossing and is not a winning run.
The remaining .25 holds runners and recovers the batter before First, with no
WP/PB unit. With no before-runners choose safe reach/recovered hold .50/.50. These
new conditional weights resolve previously unspecified child policy, not fitted
rates or acceptance tolerances. All eligible recovered advances use the new typed
native chronology; missing/invalid chronology refuses rather than becoming HOLD.

## 2026-10-17 — manager v2 supersedes the initial fixed hook

The initial27/6PA/seventh-boundary manager policy above is superseded for the
complete text game by [workload/context v2](text-game-manager-v2.md). Seed67
exposed an unconditional6IP hook despite18BF/0H/0BB; this correction uses the
already-audited individual role workloads and bounded, explicitly provisional
context rules. No PA probabilities, scientific tolerances or historical records
are retuned. V1 cast bytes remain supported as an individual-mean fallback; an
explicit empirical v2 export adds training outing spread. Actual verification
and remaining fidelity limits are tracked only in [PROGRESS](../PROGRESS.md).
