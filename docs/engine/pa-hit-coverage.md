# E-03: PA hit/no-hit classification coverage

Selected before implementation after local checkpoints `eefba3a`, `7eb5160`
and handoff `3a99b52`. [Approved matrix](g1-accounting-scope.md) and
[field coverage requirements](scoring-coverage.md) stay unchanged.

## Actual caller gap and smallest prerequisite

The generated contact-out producer supplies AtBat/PO/A; its ROE supplies
AtBat/Error. Neither AtBat nor missing Hit declares a scorer-reviewed no-hit
outcome. Before adding factual retirement/throw chains and full-box coverage,
make this missing PA decision representable and expose its actor-local coverage.
This slice covers **H classification only**, not ordinary-out method or fielding
completeness, and does not extend clean ER.

- Append actual-batter `CreditKind::NoHit`, a scorer decision/modifier (zero H),
  not another PA/AB/BF unit or a claim about contact/retirement/fielding.
- Bind it to a PA and its actual batter. Duplicate NoHit and Hit+NoHit are
  `IllegalCredit`, including before unrelated inherited-charge uncertainty.
  NoHit does not itself identify BB/HBP/sacrifice/FC or resolve pitcher lineage.
- Add `BatterLine.classified_hits` and
  `PitcherLine.classified_hits_allowed`: optional **observed-segment** H counts
  only while each of that actor's observed PAs has explicit Hit/no-hit outcome.
  Hit establishes one; NoHit, K, BB, HBP, SH, SF, interference award and canonical
  FC establish zero. Bare AtBat/GIDP/contact/endpoints/NoMisplay do not establish
  no-hit coverage. Do not infer absent opportunities or complete official boxes.
- A generic supplied Hit can establish H1 while scored breakdown/TB remains
  unknown. Preserve existing `credited_*` lower bounds, AB and ER semantics.
  Missing PA classification irreversibly makes only that batter/current PA
  pitcher's count None; subsequent facts, relief, endings cannot restore it.
  Current PA attribution remains separate from carried run charges.
- Append wire credit tag25, version BBE19; preserve prior tags/layout and reject
  earlier version headers. Decoder shares canonical validation.
- Once canonical validation/folding passes, the preview may explicitly supply
  NoHit on its **chosen** contact-out/ROE categories. No new outcome sampling,
  ratings, probability, policy, fielding roll or historical data inference.

## Acceptance checks fixed before TDD

1. Classified hit/no-hit totals/zeroes versus unknown bare AtBat in both halves;
   no second PA/BF/AB/H, no value inference, exact literal segment totals.
2. Actor-local irreversible uncertainty, relief/inherited charges, runner-only
   non-PA behavior, complete physical game with one omitted no-hit declaration.
3. Reducer/direct constructor/semantic decode reject wrong actor, non-PA,
   duplicate and contradictory markers; valid partial/no-marker controls remain.
4. Missing-coverage cases do not certify official PO/A/E/PB/RBI/WP/balk/ER boxes;
   ordinary-contact ER uncertainty remains intact. Legal incomplete histories
   stay usable/unsupported, not structurally excluded from G1 obligations.
5. Generated games retain seeded physical outcomes and qualified prior stats;
   explicit H coverage equals supplied H without implying full-box coverage.
6. Fresh full applicable suites/supplemental audits/diagnostics, frozen hashes,
   complete owned/staged inspection and local-only checkpoint before acceptance.

## Reproduce the portable checks

```sh
cargo fmt --manifest-path engine/Cargo.toml --check
cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path engine/Cargo.toml
python3 -m unittest discover -s tests -v
```

The focused methods are in `engine/tests/accounting.rs`; preview coverage is
asserted over all 1,296 producer contexts and 2,000 synthetic seeded games in
`engine/tests/generated_preview.rs`. The literal full physical game is
Home9–Away8; omitting just one NoHit preserves terminal continuity but loses
only the affected batter/current pitcher's H coverage. Even the positive
classified version keeps generic-contact ER unknown and lacks fielding credits.
The retained partial relief/runner-only and generic Hit-value tests distinguish
actual PA opportunity from carried R and classified H from unknown hit value.

[PROGRESS](../PROGRESS.md) owns status, exact commands and raw evidence. Ordinary
retirement/field-credit facts and other per-field coverage remain subsequent
E-03 work, not a gate waiver or an unchecked complete-box boolean.
