# E-03 — sole ordinary ground-ball muff: positive fielding coverage

## Preregistration

Selected after verified local `2c6ceee`, before new tests/production. The actual
preview's remaining fielding gap is its ROE category: AB/NoHit and one supplied
Error at defensive slot3, all occupied runners advance one base, batter First,
but whole-chance review is Unknown. Credited E1 is not complete E/PB/PO/A evidence.
The new integration golden covers the other five PA families, not this gap.

Inspected actual preview event construction, coverage fold/getters, shared
fielding review/credit/role checks and the approved matrix. Frozen 2019 Official
Baseball Rules SHA256 `d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`:
Rule9.12(a)(1)/Comment, printed123–124/PDF135–136, mechanical ground-ball muff
with ordinary-effort lost retirement; slow handling/mental mistakes are not
this error. Rule9.12(a)(2) includes earlier muffed fouls; (d)/Comment allows
assists on other error methods, so no universal A0/error cap. Rule9.04(a)(3)
allows an RBI on some error plays: this slice must NOT infer positive-run RBI0.

## Bounded factual input

Add optional `MuffedGroundBallFacts { batter_id, fielder_id }` to Event, and a
whole-chance `FieldingReview::SoleGroundMuff` value. Facts assert one mechanical
muff of this batter's ordinary ground ball by the named eligible before-defense
fielder, losing a routine before-first batter retirement. There was no clean
fielding followed by throw, deflection/assist chain, recovery putout, or separate
advancement episode. All actual represented runner advances are exclusively
caused by this error: batter First, every before runner exactly one base forward,
no out/personnel change. This is deliberately the preview's station-one subset,
not a general rule forbidding other legal error histories.

SoleGroundMuff additionally reviews ALL chances in the event: this is the only
misplay, including prior fouls/PB/interference/obstruction. It requires these
bound facts and exactly one Error for this fielder; no other E/PB/PO/A units.
It cannot be used as an unchecked generic complete label. NoMisplay with the
muff/Error is contradictory. Unknown review stays Unknown: correctly supplied
muff facts/credits alone do not certify whole-event fielding. Earlier distinct
foul Error units remain legal partial history under Unknown, without a global
one-Error cap. Each separate qualified event may charge its own Error.

Reject known actor/role/PA/classification/origin/unit/geometry contradictions
through canonical reducer/constructor/semantic decode before unrelated missing
compound history. Retain legal unbound legacy error plays and their lower bounds.
Nonmatching typed geometry is a contradiction to the assertion, not a ban on
all other error plays. Unbound compound pitch/balk/contact facts need separate
binding and remain unsupported after their known contradictions are checked.

## Derived coverage and producer

- `muffed_ground_fielding()` gives scoped PO0/A0 only with bound facts and
  SoleGroundMuff; union into existing `fielding_contribution()`.
- `muffed_ground_misplays()` gives named E1/PB0 under the same qualification.
  `misplay_contribution()` unions existing reviewed zeroes with this positive
  proof; the existing `reviewed_misplay_zeros()` remains strictly zero-only.
- Fold via the existing eligible before-defense actor clock and independent
  irreversible Option axes. No new credited map/default completeness flag.
  Unknown previous events cannot be repaired; incoming actors retain independence.
- Annotate only the preview's already-chosen ROE with the actual chosen muff
  fielder and full sole-muff review. No source fact fabrication, random draws,
  probability/seed/physical-policy changes, new categories or calibration.
- Append the codec fact/review tag and increment BBE24 to BBE25; preserve all
  existing tags. Mechanically migrate literal Event fixtures/current-prefix and
  absence-tail offsets, retaining historical assertions and old-prefix rejection.

## Test-first acceptance

Freeze production. Witness interface errors, add only schema/None plumbing,
then witness semantic positive-coverage and contradiction reds before behavior.
Independent event-local matrix: eight masks, three outs, both halves, independent
move/credit order and both SoleGroundMuff/Unknown review; literal scoped maps,
observed actor clocks, credited equality, codec/constructor/immutability checks.
Test wrong/missing/duplicate unit, wrong batter/fielder/DH/bench, false movement,
missing facts, NoMisplay contradiction and semantic wire forgeries with parsed
controls. Preserve prior-foul Error multiplicity/other assist methods as partial
history, later repeated events, chronology/personnel and complete-physical
missing-evidence controls. Preview bindings must be tested first on frozen
producer; every existing unrelated statistic/seeded physical result stays exact.
Run all applicable checks/full retained logs/frozen and staged equality before
local checkpoint. No push or independent-review/full-box/G1 claim.

## Supplemental known-RBI consistency check (preregistered before test/fix)

After the first full verification and before staging, primary Rule9.04(a)(3)
(printed111/PDF123) confirms the error-RBI exception is explicitly **before two
are out**. This bound method loses a routine before-first retirement and all
represented scoring is exclusively error-assisted, with no independent episode.
At two outs a positive supplied RBI is therefore contradictory, not unrelated
missing reconstruction. Test reducer/constructor/parsed wire with a valid
positive-run/no-RBI control, both reviews, unchanged physics, and legal unbound
or zero/one-out positive-credit controls. Reject only the bound two-out positive
unit. Do not add a zero-RBI contribution for a positive-run PA, certify any
counterfactual ER, or impose a blanket error/RBI ban. Fresh full checks are
mandatory after the guard; preserve the initial full proof artifacts.

## Implemented proof surface (verification lives in PROGRESS)

The canonical charge flow validates the named muff before unrelated missing
compound history. Required AB/NoHit/PA and named Error units, before-defense
roles, all station-one origins/charges/runs and no-retirement geometry are
checked through reducer, constructor and semantic decoder. Full review additionally
requires exactly one Error and no other E/PB/PO/A; Unknown preserves prior-foul
Error/Assist lower bounds rather than imposing a universal cap. In particular,
a missing method binding cannot make an otherwise legal error-assist history
IllegalCredit: the unchecked sole-muff label is UnsupportedRule instead.

Thirteen appended accounting methods include384 review/order/mask/out/half frames,
parsed wire actors/binding/corruption controls, distinct chances/irreversible
taint, independent incoming-fielder coverage, known facts before unbound balk
history, definitely foreign steal-actor rejection before missing binding (while
present held actors retain intermediate-history uncertainty), the bounded two-out positive-RBI contradiction with pre-two/unbound
controls (no new zero coverage), and a53-event completed Away0–Home1 game with paired missing-evidence
controls and independently asserted fielding clocks/maps. The432-frame
all-category preview binding test checks unchanged before/after/credits and
normalizes only independently asserted new coverage cells before whole-object
accounting equality. Existing historical assertions and full suites remain
mandatory; a passing integration test is not fabricated semantic-red evidence.

## Explicit limits

No RBI or ER reconstruction expansion, no hypothetical outs/advances inferred
from final bases, no general positive PB/throw-error/foul-error/CS proof. Positive
charged runs in misplay-affected halves remain unknown ER. The counterfactual
advance/scorer judgments needed by Rule9.04/9.16 will be separately preregistered.
This closes a concrete preview fielding prerequisite, not the full approved
accounting matrix. [PROGRESS.md](../PROGRESS.md) owns live status/evidence.
