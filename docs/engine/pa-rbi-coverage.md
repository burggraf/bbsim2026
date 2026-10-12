# E-03: observed-PA RBI coverage, not credited-counter completeness

Preregistered after verified local protected-award checkpoint
`a17b778a959bf2a0840c3df938256869b8a8a08b`, before tests or production edits.
[PROGRESS](../PROGRESS.md) owns live status. This contributes to the
[approved matrix](g1-accounting-scope.md); it does not complete E-03/G1.

## Actual flow and bounded selection

`validate_run_credits` matches named legally scored runners to `Delta.runs` and
Run credits. `scored_runners` applies third-out suppression; an unresolved
scoring third-out cannot produce a valid immutable transition. RBI credits are
bounded by legal runs and belong to the actual batter, but a supplied positive
RBI does not establish cause. `derive_accounting` currently accumulates only
`credited_rbi`, with no per-PA coverage. Its existing classified-H fold already
keeps unknowns irreversible and actor-local.

Cached 2019 Rule9.04 defines RBI as an action at bat causing a run to score.
Consequently **zero legal runs in a validated PA imply RBI0**, independently of
hit/AB classification or whole-play fielding review. A physical Home crossing
voided by an explicit force/batter-before-first third out is not a legal run.
This is not an inference from omitted RBI credits or a score-only snapshot.

Rule9.04(a)(2) plus the explicit exclusive protected-award facts independently
establishes the positive BB/HBP case already implemented. Rule9.04(a)(1) was
also inspected: an explicit out-of-park HR/net Home shape does not newly assert
that all scoring within a potentially multi-effect PA was solely that hit and
unaided by an error. Broader positive hit/HR/RBI causes need their own proof.
Choose this smaller coverage seam first; no new HR facts/wire schema here.
Primary rulebook SHA256 remains
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`,
Rule9.04 printed111/PDF124.

## Representation and unknowns

Add `EventTransition::rbi_contribution() -> Option<RbiContribution>`:

- Non-PA: None; it is not an RBI opportunity for the current batter.
- Validated PA with zero legally credited runs: named actual-batter RBI0.
- Positive scoring PA with existing protected-award proof: reuse its independent
  named contribution, not the supplied RBI counter.
- Other positive scoring PA: None even with NoMisplay/correct supplied RBI or an
  explicit HR type. No generalized positive hit/GIDP/sacrifice/fielding inference.

Keep `protected_award_rbi()` unchanged: missing exclusive cause stays None even
for a nonscoring award. The new generic zero-run evidence does not certify that
specific cause, E/PO/A/ER or complete boxes.

Add `BatterLine.classified_rbi: Option<u32>` for this actor's observed PAs.
Initialize Some(0) for zero observed PA opportunities (including a runner-only
row), not career/game completeness. Fold each actual PA's independent
contribution: sum known values, set None for any unknown and never restore it.
Leave `credited_rbi` unchanged. Non-PA events do not add/taint current-batter
coverage, and a scoring runner's carried pitcher does not receive a batter RBI
opportunity. No pitcher RBI field, new credit, Event DTO, codec or sampler change.

## Observable acceptance before implementation

1. Cartesian masks/outs/halves/review/vector order: nonscoring classified and
   unclassified PAs yield named RBI0 without filling unrelated H/AB/ER; validate
   reducer/direct constructor/codec, caller immutability and literal accounting.
2. Explicit third-out voided physical Home crossings yield RBI0, not physical
   crossing counts. Non-PA WP scoring yields None and no current-batter RBI row.
3. Existing protected BB/HBP scoring gives named RBI1; missing facts with correct
   supplied RBI and NoMisplay stays unknown. Positive generic hit and explicit
   out-of-park HR remain unknown; legacy supplied counters are unchanged.
4. Contiguous repeated-actor histories: known positive/zero accumulate, unknown
   before or after a known PA is irreversible, other batters remain independent,
   substitutions/runner-only appearances do not manufacture PA coverage.
5. Verify all three immutable construction paths and roundtrips. Witness missing
   interface then genuine contribution/fold reds before implementation. Add
   preview property assertions before changing any necessary presentation (none
   planned); keep accepted seeded presentation byte-identical.
6. Run full Rust/fmt/strict all-target Clippy/Python/native/offline audits,
   diagnostics/CLI, prose/diff and exact frozen ownership checks. Preserve raw
   failure evidence and inspect complete actual staged changes before local
   checkpoint. No push, new download or independent reviewer is authorized.

Complete RBI cause coverage, full-field literal goldens, broader Rule9.16,
production probabilities/UI and G1 remain required unfinished work.
