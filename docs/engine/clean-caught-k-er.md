# E-03 — reviewed ordinary caught strikeouts in clean earned-run halves

## Preregistered scope

Selected after verified local `6bd6b72`, before new tests or production changes.
Reuse the accepted [ordinary caught-K facts](ordinary-caught-k.md) and existing
[clean-half proof](clean-ground-er.md). This is a narrow vocabulary extension in
accounting, not new event facts, validation, codec, model or presentation.

Inspected `caught::validate`, the shared held-batter-before-First guard,
`clean_play`, half-origin/completion/cumulative-uncertainty handling and the
observed ground-half fixture/callers. The caught subtype proves one direct
caught third strike, actual batter retired before First, no runner advance,
optional explicit holds and no separate delivery/contact/balk episode. Full
whole-chance NoMisplay separately excludes prior missed chances/errors/PB.

Primary source: frozen 2019 Official Baseball Rules SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`,
Rule9.16 opening/(a), printed130–131/PDF142–143: reconstruct without errors/PB,
give the pitcher benefit of doubt, and count actual fielding chances. A reviewed
bound ordinary caught strikeout is one actual errorless retirement with held
runners, regardless of whether held movements are explicitly serialized.

## Observable checks fixed before implementation

- Extend only the existing PlateAppearance clean-play branch to recognize
  canonically validated ordinary caught-K facts under its existing full-review
  guard. Do not broaden generic held-vector K, uncaught/dropped K, advancement,
  pitch recovery, error/PB, sacrifices or counterfactual reconstruction.
- Reuse the observed ground-half prefix, replacing only its retirement with an
  explicit caught-K event with catcher PO1 and K/AB/NoHit. Preserve exact physics
  and before/after continuity. All eight observed masks, 0–2 prior outs, both
  halves, omitted/explicit holds and independent movement/credit reversal:
  384 cases. Bases must arise from actual observed reviewed awards/advances.
- Independently assert literal R/ER, outs/IP, BF/PA, AB/H and strikeout/catcher
  PO totals. A third-out K strands held runners, belongs to the ending half and
  must roundtrip unchanged through BBE24.
- Missing catch facts with explicit occupied holds must remain unknown despite
  K/PO/NoHit/AB; preserve the legacy simple one-movement K clean proof. Missing
  current or earlier review, late-start logs and unfinished positive-run halves
  remain unknown. Later caught facts cannot repair earlier uncertainty.
- Witness semantic reds on unchanged production before implementation. Retain
  fixtures/compiler/harness failures separately; no threshold/assertion weakening.
- Run the entire Rust/Python/native/audit/preview/CLI suite, strict all-target
  Clippy/formatting, unchanged baseline/seeded/legacy presentation, prose links
  and exact staged/working/frozen checks before a local checkpoint. No push.

## Limits

Not complete Rule9.16 reconstruction, positive E/PB proof, official boxes or
E-03/G1 completion. No independent reviewer or publication claim. Existing
legal partial histories, canonical contradictions and credited counters remain
unchanged. Live status and evidence belong only in [PROGRESS.md](../PROGRESS.md).
