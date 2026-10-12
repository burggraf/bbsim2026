# E-03 — explicit ordinary caught-strikeout fielding evidence

## Selected before TDD

Following the [ordinary ground-throw proof](ordinary-ground-throw.md) and bounded
clean-ER checkpoints, close another common generated-path fielding prerequisite.
The preview already chooses caught K and supplies its fictional catcher's PO;
bare K/PO credits still do not bind the actual catch/retirement method.

2019 Rule 9.09(b)(1) credits the catcher when a batter is called out on strikes;
Rule 9.10(b)(1) does not credit the pitcher an assist on a strikeout unless he
fields an uncaught third strike and throws for a putout. This selected subtype
asserts an ordinary legal called/swinging third strike received directly by the
named catcher, retiring the actual batter before First. It is not uncaught-K
recovery, third-strike foul bunt, deflection, appeal or another out method.

## Bounded factual payload and canonical checks

- Proposed `CaughtStrikeoutFacts { batter_id, catcher_id }` and optional
  `Event.caught_strikeout`. Append its encoding after existing ground facts in
  `BBE21`; preserve prior credit tags. Historical constructors supply None.
- Actual PA participants; catcher is an eligible before-defense fielder, not
  the current pitcher/active DH/offense/bench/removed player. This is an asserted
  event-local catcher role, not inferred permanent defensive positions.
- One actual batter Plate→Out, no other retirement/scoring/personnel change;
  before runners hold with identities and carried charges intact. Held moves
  can be omitted or explicit. Third out names this before-first retirement.
- Exactly one actual-batter K/PA; existing optional AB/NoHit and all canonical
  actor/classification/count/origin/run/review guards remain authoritative.
  Incompatible outcomes and known delivery/touch/retirement contradictions
  precede unrelated missing compound chronology.
- With whole-chance NoMisplay, require exactly one named-catcher PO and no A.
  Unknown earlier chances can retain otherwise eligible sparse/extra assist
  credits and missing PO; do not label richer legal histories impossible.
  Explicit additional episode facts need their own binding and stay unsupported
  after known guards rather than gaining qualified certainty.

## Independent event-local contribution

Proposed `EventTransition::caught_strikeout_fielding()` returns an optional
`FieldingContribution`: catcher PO1 and an empty assist map only for canonically
bound facts plus whole-chance NoMisplay. Derive it from facts, not supplied
counters or the after-half defense. Missing facts/review yield None, not known
zero maps. Existing credited lines and clean-ER proof remain unchanged.

Only the preview's selected caught-K outcome supplies these assertions; no RNG,
probability, movement, score, rating or historical-source fact changes.

## Observable checks fixed before implementation

1. Literal PA/AB/BF/K/H/out/PO/A and independent contribution across masks,
   0–2 outs, halves, DH/no-DH, held-vector/order/catcher choices; caller immutability,
   direct construction, reducer and codec roundtrip/semantic validation.
2. Wrong batter/catcher/role, contradictory result, missing/duplicate qualified
   K/PA/PO/A, movement/charge/third-out and review forgeries on all entry paths.
   Whole-event byte controls must parse positively before a semantic mutation.
3. Missing facts/review and otherwise legal uncaught/automatic/recovery or richer
   histories retain their current partial behavior. Preserve current ground,
   pitch, balk, caught-stealing, generic-K and ER controls.
4. Preview declares facts only for chosen K, not contact/hits/awards/ROE; same
   seeded outcomes/score/credited boxes, with only the stronger event-local proof.
5. Interface/compiler reds are distinct from genuine behavior reds. Full Rust,
   Python, fmt, strict all-target Clippy and existing native/audit/preview/CLI
   checks, complete owned/staged inspection and frozen equality precede acceptance.

This does not certify E/PB/WP/RBI or complete official fielding/whole-game boxes,
expand full Rule9.16 reconstruction or complete E-03/G1. Live evidence and status
belong only in [PROGRESS.md](../PROGRESS.md).
