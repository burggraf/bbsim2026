# E-03 — reviewed sole unaided safe hits in clean earned-run halves

## Preregistration

Selected after verified local `cf288eb`, before new tests or production changes.
Inspected the complete sole-safe-hit validator, its declared exclusive continuous
unaided cause, existing station-hit clean proof/half certification and actual
observed-half fixtures. Reuse canonical cause facts, not just classified hits,
RBI units, favorable final bases or absent error credits.

Primary source: frozen 2019 Official Baseball Rules SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`,
Rule9.16 opening/(a), printed130/PDF142: reconstruct without errors/PB, give the
pitcher benefit of doubt; safe hits contribute to earned runs. The existing
[SoleSafeHitFacts contract](sole-safe-hit-rbi.md) asserts all represented advances
arise exclusively and continuously from the classified unaided 1B/2B/3B, not
an earlier/later episode or stopped/restarted misplay. Whole-chance NoMisplay
separately excludes prior missed chances/errors/PB. Together these facts permit
actual held/unequal advances in this bounded errorless play.

## Observable checks fixed before implementation

- Extend only the existing PlateAppearance clean-play branch to recognize
  canonical sole-safe-hit facts under its full-review guard. Keep the station
  proof and all other legacy vocabulary unchanged; no generic advancement,
  error-assisted hit, separate episode or counterfactual reconstruction expansion.
- Use actual reviewed HR/BB/WP/K origins to establish eight masks, 0–2 outs and
  both halves. Three classified values, two legal advancement policies and
  independent move/credit reversal:1,152 cases. Policy one sends before runners
  continuously Home; policy two uses legal nonpassing holds when available,
  otherwise station advances. The batter stays at its classified First/Second/
  Third destination; no runner passes a preceding held runner.
- Append a reviewed homer and remaining outs. Independently assert literal
  R/ER, BF/PA, AB/H/K/outs and codec/physical continuity. Positive opening run
  prevents unknown tests from vacuously succeeding through zero charges.
- Nonstation single controls: missing exclusive cause, missing current/earlier
  review, positive prior foul-chance Error with Unknown review, late origins
  and unfinished positive-run halves remain unknown. Correct Hit/value/RBI
  units do not substitute for proof. Retain old generic nonstation-hit unknown
  assertions and every historical accounting test byte-identically.
- Freeze production and witness genuine semantic reds before implementation;
  retain fixture/compiler failures separately. No assertion/threshold weakening.
- Run all applicable Rust/Python/native/audit/preview/CLI checks, strict all-target
  Clippy/formatting, unchanged baseline/seeded/legacy presentation, fresh prose
  links and exact staged/working/frozen evidence before local checkpoint.

## Limits

Not ordinary walk-off expansion, other RBI causes, positive misplay evidence,
full Rule9.16 reconstruction, official boxes or E-03/G1 signoff. Existing carried
charges/irreversible uncertainty and canonical priorities remain unchanged.
No producer/RNG/model/codec/presentation changes, independent-review claim or
push. Live status and evidence belong only in [PROGRESS.md](../PROGRESS.md).
