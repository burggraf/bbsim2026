# E-03 — ordinary imposed exact balk awards in clean earned-run halves

## Preregistration

Selected after verified local `5115812`, before new tests or production changes.
Inspected the complete balk validator/preflight, exact-award predicate and facts,
clean-play/half-certification fold, observed-half fixture and actual inherited
balk/current-pitcher tests. Reuse existing canonical exact-award qualification;
do not infer it from a Balk credit, label, final bases or missing misplay units.

Primary source: frozen 2019 Official Baseball Rules SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`,
Rule9.16(a), printed130/PDF142: balks contribute to earned runs; retain the
opening requirement to reconstruct without errors/PB and benefit the pitcher.
Canonical PitcherAction602a + PenaltyApplied + ExactProtectedAwards binds each
before runner to exactly one next-base award with no out or separate action.
Whole-chance NoMisplay separately excludes earlier missed chances/misplays.

## Observable acceptance fixed before implementation

- Extend only the RunnerOnly(Balk) clean-play branch under its existing review
  guard, using the existing exact-award predicate (internal reuse, no public
  API). Unknown cause/disposition/resolution, play-proceeds progress and bare
  labels/units do not certify ER through this new branch. Existing clean-play
  proofs and canonical validation/priority remain unchanged.
- Reuse the actual observed half prefix through HR/BB/WP/K to obtain all seven
  nonempty masks, 0–2 prior outs, both halves and independent move/credit orders:
  168 cases. Append exact imposed balk, reviewed homer and the remaining outs.
  Assert literal R/ER, BF/PA, AB/H/K/outs and one current-pitcher balk. Non-PA
  award does not advance the batting order or BF/PA. Roundtrip/immutability apply.
- Positive run before the award makes missing facts/current or earlier review,
  late origin and unfinished halves observable as unknown ER, not zero. Remove
  the asserted unit with missing facts to preserve a legal legacy partial view;
  never change canonical rules just to manufacture a negative fixture.
- Observed relief after runner acquisition: old pitcher keeps inherited charged
  R/ER, reliever gets the balk and actual BF/outs; no current-participation/charge
  conflation. Repeated exact awards remain distinct occurrences, not deduplicated.
- Freeze production and witness genuine semantic reds before implementation.
  Retain all old accounting tests byte-identically and record any fixture/compiler
  failures separately. No weakened assertions, seeds or thresholds.
- Run all applicable Rust/Python/native/audit/preview/CLI checks, format/strict
  all-target Clippy, unchanged baseline/seeded/legacy presentation, fresh prose
  links and exact staged/working/frozen evidence before local checkpoint.

## Limits

Not interference/other balk causes, discarded balks, pitch recovery, richer
advance/error/PB reconstruction, complete official boxes or E-03/G1 signoff.
No producer/RNG/ratings/codec/presentation changes, independent-review claim or
push. Live status and evidence belong only in [PROGRESS.md](../PROGRESS.md).
