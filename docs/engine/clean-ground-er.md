# E-03 — reviewed bound ground retirement in clean earned-run halves

## Preregistered scope

This follows the locally accepted [ordinary ground-throw contract](ordinary-ground-throw.md)
and the existing [bounded clean safe-PA ER proof](clean-safe-pa-er.md). It extends
only the errorless-play vocabulary in the independent accounting fold; no event,
codec, canonical validation, model, RNG or source-import change is proposed.

2019 Rule 9.16 requires reconstruction without errors and passed balls, including
missed foul chances and benefit of doubt to the pitcher. The existing clean-half
proof requires an observed origin, a legal completion and whole-chance
`FieldingReview::NoMisplay` on every event. The bound ground subtype supplies the
otherwise missing method: one direct ordinary fair non-bunt throw retirement of
the actual batter before First, with before runners held and charges preserved.
Canonical validation already reconciles actual participants, movements, one out,
no runs, third-out disposition, PA/AB/NoHit and qualified PO/A. In conjunction with
whole-chance NoMisplay, this one retirement is also one errorless out; there is
no erased misplay or advancement to reconstruct in this bounded play.

## Observable acceptance checks fixed before implementation

- Both halves, all eight observed base masks and 0–2 prior outs: establish the
  half from an observed boundary through reviewed HR/BB/WP/K events, then a bound
  reviewed ground retirement. Occupied masks must come from actual observed
  awards/advances, not a fabricated initial snapshot. Include held-vector
  omission, explicit held movements and movement-order permutations.
- Literal independent charged R/ER, PA/BF, AB, H and outs remain correct. Retained
  runners strand on a ground third out; its after snapshot may switch halves,
  but its retirement and PO/A belong to the ending half. Codec roundtrips retain
  facts/review and do not change the fold.
- A positive charged run before the ground retirement must be certified only
  after legal completion. Missing ground facts, missing ground review, missing
  earlier review, omitted observed origin or an unfinished half must leave that
  pitcher's positive ER unknown. Supplied PO/A/NoHit/AB alone must not repair it.
- Existing generic-contact, non-station hit, error/PB, charge-lineage and
  cumulative-uncertainty regressions remain unchanged and pass. Later bound
  ground evidence must not repair uncertainty from an earlier event.
- Run full Rust tests, formatting, strict all-target Clippy, Python tests and
  existing native/audit/preview/CLI checks. Preserve genuine semantic reds and
  fixture/harness failures separately; record fresh results in PROGRESS.

## Limits

This is not full Rule 9.16 reconstruction, fielding/WP/PB/RBI completeness,
source scorer inference, complete official boxes or E-03/G1 signoff. Unknown
review remains legal/partial; optional facts without NoMisplay do not suffice.
Generic ground labels, endpoints and credits do not imply this method. Richer
contact, advance, recovery, appeal and misplay histories remain separate work.
Acceptance and publication status belong only in [PROGRESS.md](../PROGRESS.md).
