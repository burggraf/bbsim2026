# E-03: bounded clean safe-PA earned-run proof

Contract selected before behavior changes, following local generated-preview
checkpoint `eefba3a`. [Approved matrix](g1-accounting-scope.md) and
[scoring coverage requirements](scoring-coverage.md) remain unchanged.
This is an ER proof extension, not complete official boxes or G1 signoff.

## Actual gap and selected boundary

`accounting::clean_play` currently supports bounded reviewed K, BB, HR, FC and
WP shapes. Even an explicitly classified, reviewed clean triple followed by a
reviewed WP and three K-outs leaves its charged run unknown. The cached 2019
Rule 9.16(a), including example (2), explicitly makes that run earned.

Extend only:

- **Safe classified 1B/2B/3B:** one batter Hit with its canonical value modifier,
  no physical outs, batter safely to the corresponding base, and all before
  runners advancing station-to-station by that value (Home when appropriate).
  Each relevant runner must have exactly its expected movement; no retirement,
  passing, omitted origin, extra throw/steal/pitch/balk or personnel episode is
  certified through this shape. Canonical validation remains authoritative.
- **Simple HBP award:** the existing single Plate→First/no-outs/no-runs shape,
  with explicit HBP, alongside the already supported simple BB. Forced
  multi-runner awards and other award histories remain outside this extension.

Every event still needs explicit whole-chance `FieldingReview::NoMisplay`, not
merely absence of supplied Error/PB. The half must have an observed origin and
legal end; earlier unknowns cannot be restored by later clean plays. Carried
pitcher responsibility stays independent of current PA/BF participation.

## Deliberately not changed

No codec/DTO, outcomes, RNG, manager, data or UI changes. Ordinary generic
contact outs (even an AtBat credit), unclassified hits, non-station advancement,
runner retirements, misplay/PB reconstruction and unrepresented episodes remain
uncertain under the existing proof. In particular, preserve the current
`reviewed_generic_contact_is_not_yet_an_errorless_play_proof` control. An ordinary
out will require its own factual contract; supplied PO/A totals alone must not
be treated as complete fielding/scoring evidence.

## Observable checks before implementation

1. Literal Rule 9.16(a)(2)-shaped triple→WP→three K: R1/ER1, one triple/TB3,
   exact PA/BF/outs, observed half score, codec and continuity.
2. Positive safe-hit and simple-HBP cases in both halves. Station hit patterns
   exercise all masks/outs where a complete observed prefix can establish them;
   retain actual legality/charge and hit-value validation, not fabricated states.
3. Unknown review at any contributing play, omitted hit value, unsupported
   contact, unobserved origin and unfinished half keep positive charged ER
   unknown. Error/PB cases remain unknown rather than being forced to zero.
4. Legal non-station advances remain usable but outside this positive proof.
   No new structural impossibility or accounting-matrix exclusion is invented.
5. Fresh full Rust/Python/fmt/strictClippy/native supplemental checks, complete
   owned diff inspection and literal/property evidence before acceptance.

Raw failures/seeds and exact verification belong in [PROGRESS](../PROGRESS.md).
The canonical engine/fold owns all statistics; the generated preview is only a
caller, and its ordinary-contact/error frames may still show unknown ER.
