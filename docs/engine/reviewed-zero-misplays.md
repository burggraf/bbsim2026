# Reviewed event-local zero errors and passed balls

## Preregistration

E-03 slice selected after verified local `9b311f6`, before new tests/production.
The approved [accounting matrix](g1-accounting-scope.md) requires E/PB as well as
PO/A. Existing observed-defense exposure/eligibility and actor-local PO/A
coverage are accepted prerequisites, not a complete fielding box or G1 signoff.

Inspected the actual coverage fold, five PO/A getters, legacy credited-fielding
branch, canonical review/event-shape/role checks, passed-handling preflight,
existing PO/A literal-map tests and preview whole-object comparison. The existing
`FieldingReview` explicitly reviews **all** event fielding chances, including
muffed fouls, errors, PB, interference and obstruction. NoMisplay asserts none
occurred; it is not inferred from absent credits. Unknown remains unknown.
Audited historical source rows do not supply this whole-chance assertion.

Primary source: frozen 2019 Official Baseball Rules, SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`.
Rule9.12, printed123–128/PDF135–140, includes missed foul chances and scorer
judgment, not just final safe/out destinations. Rule9.13(b)/Comment,
printed128–129/PDF140–141, distinguishes catcher PB from pitcher WP and
advancement/recovery exceptions. This slice proves reviewed **zeroes** only;
it does not infer positive E/PB units, catchers or counterfactual reconstruction.

## Observable contract

Add `MisplayContribution { errors, passed_balls }` with event-local named maps
and `EventTransition::reviewed_misplay_zeros()`. On a canonically validated
non-substitution event with full NoMisplay return Some(empty E map, empty PB
map). Empty maps mean zero in this transition's actual **before-defense** scope.
A substitution or Unknown review returns None, even with absent/correct supplied
credits, a sole cause, known RBI, PO/A, physical completeness or favourable
final destinations. Existing canonical contradictions are not weakened.

Extend `FieldingCoverageLine` with independent Option<u32> `errors` and
`passed_balls`, initially Some(0) when an actor is newly observed. Reuse the
existing non-substitution before-defense actor loop and observed_events count;
no second opportunity clock/map or new credited rows. Deduplicate actual no-DH
pitcher, exclude DH/bench/removed/inherited-charge actors as already specified.

Accumulate each new axis only from this reviewed-zero contribution. Unknown
review irreversibly taints E and PB for each observed before-defense actor;
later NoMisplay does not repair it. Incoming actors are not tainted by a prior
fielder's history; substitutions create/taint/count nothing and removed rows
remain retained. Other team and PO/A axes stay independent: a generic/unbound
NoMisplay play can have E0/PB0 while its PO/A remains unknown.

Keep all supplied credited maps/counters/cardinality, canonical event DTO/BBE24,
validation/priority/legal partials, ER reconstruction, producers, ratings/RNG
and seeded presentation unchanged. No positive E/PB coverage or complete-game
certificate. Broad official boxes, richer ER and independent full-field goldens
remain mandatory before E-03/G1 completion.

## Test-first acceptance

1. Freeze production and witness missing API/field interface errors. Add only
   schema/defaults/None stub, migrate existing literal maps for the new axes,
   then witness genuine semantic reds before implementing behaviour. Distinguish
   fixture/compiler failures and retain all raw evidence.
2. Independent existing canonical fixture helpers: six families (five qualified
   PO/A causes plus unbound ground), two reviews, eight masks, three outs, both
   halves, DH/no-DH and two credit orders:2,304 new reviewed-misplay frames.
   Literal scope/count/E/PB and independent PO/A expectations; existing1,920
   movement/credit-order PO/A frames/assertions remain intact.
3. Unknown absence and positive prior Error/PB supplied-count controls remain
   None, not fabricated official zero/positive totals. Illegal NoMisplay with
   Error, PB kind/units or passed handling retains existing canonical rejection.
   Reducer/constructor/codec parity and caller immutability remain checked.
4. Known→unknown→known and incoming/removed actors, runner-only opportunities,
   substitutions, DH loss, inherited charges, half/terminal before-defense and
   physically complete/late-start logs demonstrate independent axes/irreversible
   unknowns. Existing credited-stat equality is preserved after independently
   asserting the newly added axes; no dropped assertions or weakened thresholds.
5. All applicable Rust/Python/native/audit/preview/CLI checks, strict all-target
   Clippy/formatting, byte-identical baseline/seeded presentation, fresh prose
   links and exact staged/working/frozen equality before local checkpoint.
   No push/independent-review/full-box/G1 claim; PROGRESS owns live status.
