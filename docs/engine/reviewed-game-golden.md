# E-03 — literal reviewed-vocabulary 8½-inning game reconciliation

## Preregistration

Selected after verified local `a67e009`, before new tests. This is the next
main-path integration proof: combine the five qualified PA families and their
independent H/RBI/PO/A/E/PB/clean-ER coverage in one legally completed game.
Existing isolated proofs and the old physical 52-event game are not this proof.
No new production behavior, scoring facts, model, RNG or codec is planned.

## Fixed fixture and independent oracle

Start empty Top1, 0–0. Away first: ordinary BB; sole single with the before runner
Home; sole double with its before runner Third; sole triple with both before
runners Home; sole out-of-park HR. This scores five. Home first: HBP, ordinary BB,
IBB; sole single with all three before runners Home; sole double and triple each
with its before runner Home; sole HR. This scores seven. Each played half then
has ground/K/ground retirements. Finish at Top9, Home7–Away5, with Bottom9 unplayed.
All events have full NoMisplay and canonical bound cause/method facts. Use fixed
before-defense receiver slot0, catcher slot1 and pitcher as ground thrower.
No substitutions or runner-only episodes in this integration fixture.

- Exactly63 events: Away32 PA/31 AB/4 H/10 TB/5 R/5 RBI/1 BB/0 IBB/9 K;
  Home31 PA/28 AB/4 H/10 TB/7 R/7 RBI/2 BB/1 IBB/1 HBP/8 K.
- Literal per-player boxes use the fixed batting-order rotation, not event-credit
  accumulation or another call to the accounting fold as their oracle.
- Home pitcher:27 outs/32 BF/31 AB/4 H/5 R/5 ER/1 BB/0 IBB/9 K.
  Away pitcher:24 outs/31 BF/28 AB/4 H/7 R/7 ER/2 BB/1 IBB/8 K/1 HBP.
- Home defense: receiver18 PO, catcher9 PO, pitcher18 A; Away defense:
  receiver16 PO, catcher8 PO, pitcher16 A. Every eligible observed defender has
  known scoped E0/PB0/remaining PO0/A0; DH is absent. Eligible actor event clocks
  are32 Home and31 Away. Independently reconcile exact out/PO totals.
- Line score: first[5,7], second–eighth[0,0], ninth[0,unplayed]. Assert complete
  physical game, named charges, literal totals, full struct values, codec and
  constructor equivalence, continuity and move/credit-order independence.
- Paired complete-physical controls omit a sole-hit cause, current review,
  ground-method facts or HR cause without changing physical results/credited
  boxes. Show precisely which independent axes become unknown and which stay
  known; later complete events never repair missing coverage. Bound-method
  omission of required PO/RBI units must remain rejected, not silently complete.

## Limits and verification

This is a literal integration golden for these reviewed families, not the whole
approved G1 event vocabulary or certification of all absence-sensitive official
statistics. Full GameAccounting structs still distinguish credited lower bounds
from explicit coverage axes. The fixture has no steals/CS/GIDP/sacrifices/WP/balk/
errors/PB/interference; their credited zeros are asserted as credited zeros, not
new official-coverage APIs. The complete misplay-affected matrix, broader scoring
causes and final independent complete-field/gate evidence remain required.

Tests may pass on unchanged production: that is proof of existing integration,
not a semantic red or bug fix. Any genuine behavior defect needs a witnessed
regression red before its smallest production correction. Preserve every old
test byte-identically. Run all applicable checks, full retained logs, frozen
source/test equality and complete staged inspection before local checkpoint.
No push/independent-review/G1 claim. [PROGRESS.md](../PROGRESS.md) owns live status.
