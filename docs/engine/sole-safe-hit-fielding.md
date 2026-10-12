# Reviewed sole-safe-hit zero PO/A proof

## Preregistration and scope

Selected after verified local sole-safe-hit RBI checkpoint `be3441a`, before
new tests or production behavior. This is an E-03 event-local fielding slice,
not E-03/G1 completion or a complete official box certificate.

Inspected the approved [matrix](g1-accounting-scope.md), canonical
`sole_safe_hit::validate`, existing `caught_strikeout_fielding()` and
`ordinary_ground_fielding()` getters, `FieldingContribution`, and the independent
safe-hit fixture/codec/rejection helpers. The accounting fold currently retains
supplied PO/A/E/PB counts; this slice does not change that fold or presentation.

Primary evidence: retained 2019 Official Baseball Rules PDF SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`.
Rule9.09(a)-(c), printed120–122/PDF132–134, awards actual and automatic putouts
for retirements. Rule9.10(a), printed122/PDF134, awards assists for a resulting
putout, one prevented by subsequent error, or interference/running-out-of-line
retirement. The error exception makes a safe final snapshot insufficient to
prove zero assists.

## Observable contract

Add `EventTransition::sole_safe_hit_fielding()` returning
`Option<accounting::FieldingContribution>`. It qualifies only a canonical
transition with both explicit `SoleSafeHitFacts` and
`FieldingReview::NoMisplay`. A qualified result has empty putout and assist maps:
each named actor in the transition's before-defense scope contributes zero.

This is independent of omitted or supplied PO/A credits. The existing exclusive
safe-hit validator requires classified 1B/2B/3B, complete safe held/forward
movements for the batter and each occupied runner, no retirements, no personnel
changes, and no competing delivery/balk/contact episodes or PO/A units. The
explicit full fielding review rules out the error-prevented-out exception and
unrecorded earlier misplays. Together these facts prove zero PO/A. Neither
NoMisplay alone, zero outs alone, hit classification, final bases, correct RBI,
nor a complete-game flag supplies the new proof.

Missing exclusive cause or Unknown fielding review returns None, including a
legal unknown-reviewed single with a prior foul-chance Error. Its independent
RBI proof remains available. Do not change canonical legality, broaden accepted
hit-review/walk-off shapes, erase known contradictions, or make unbound legal
histories illegal. Existing ground/caught getters retain their own semantics.

No new facts, codec revision, RNG/model/producer binding, credit units,
accounting-line fields, ER reconstruction or global completeness flag is needed.
The getter does not independently certify E/PB, any other event, an actor's
whole observed segment, or an official game box.

## Test-first acceptance checks

1. Witness missing-method interface failure with unchanged production hashes;
   add only a None stub, then witness genuine Some-versus-None semantic reds.
2. Derive literal empty PO/A maps over 576 independent value × mask × outs ×
   half × movement-order × credit-order frames. Confirm actual before-defense
   identities, zero retirements and absent supplied PO/A, rather than deriving
   expected maps by copying production output.
3. Exercise apply, canonical constructor and codec round trip. Preserve all
   existing independent wire controls and semantic rejection assertions.
4. Test missing cause, Unknown review, prior foul Error, held/unequal continuous
   advances and legal ordinary walk-offs. Unknown cases stay None and retain
   independent RBI where otherwise qualified; ER/fold remain unchanged.
5. Existing guarded wrong actor/role/units/retirement/competing-family cases
   remain rejected. Bare classified NoMisplay hits remain partial even when
   physically identical to the qualified play.
6. Run the complete applicable Rust/Python/native/audit/CLI/preview checks,
   formatting, strict all-target Clippy, fresh prose links and exact staged
   working/frozen equality. Retain raw failures and verify seeded legacy
   presentation and baseline bytes unchanged. Local checkpoint only; no push
   or independent-review/G1 claim.
