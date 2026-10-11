# Shortest path from the script to a generated game

Scope assessment at local `33f84b3`; no probability implementation or gate signoff.
[PROGRESS](../PROGRESS.md) owns current status and the sequencing decision.

## What the actual code proves

The canonical reducer and immutable transitions can finish games. The terminal
script exercises them, but chooses its outcomes in advance. `derive_accounting`
folds supplied credits and marks initial-to-terminal physical coverage via
`complete_game`; that flag explicitly does not certify official box coverage.

## Main-path blockers, not another rare-rule list

1. **Complete scoring coverage is not implemented.** Required classifications,
   named fielding chances/credits and RBI causes need consistency and omission
   checks before supplied zeroes become complete official cells. Existing
   `rules::validate_credit_counts` bounds RBI by runs and PO by outs; those
   bounds alone do not establish coverage or cause.
2. **ER reconstruction is deliberately narrow.** `accounting::clean_play`
   supports reviewed bounded K/walk/HR/FC/WP histories, not generic contact or
   all misplay/PB histories. Even the demo's ordinary single/steal puts its
   scoring half outside that proof. A new model cannot call uncertain ER zero.
3. **Full-field game evidence is missing.** Current full-game fixtures prove
   selected PA/run/out totals, not all approved player/team/inning cells,
   justified zeroes, fielding/out conservation and omission controls. The
   [approved matrix](g1-accounting-scope.md) requires that evidence and a separate
   maintainer G1 decision. Passing more local corner-case tests is not a substitute.

These are substantive work packages, not one finishing tweak. Their remaining
size is not yet measured; there is no evidence-backed date for G1 completion.
Further catcher-tag/appeal expansion is not the selected next work.

## Recommended sequencing decision

Allow a bounded **experimental generated-game preview before G1**, while leaving
E-03/G1/P-01/P-02 acceptance requirements intact. This changes execution order,
not the definition of a finished product or the approved accounting matrix.
The existing roadmap forbids early model work, so explicit maintainer approval
is required before doing it. No approval is inferred from this recommendation.

If approved, select a small versioned outcome taxonomy and evaluation protocol
before estimating probabilities. Build a seeded generator/policy separately
from the deterministic engine; use audited league inputs and documented smoothing,
not arbitrary probability floors or fabricated real-player ratings. Preregister
which legal histories/mechanisms the preview represents and which remain
unimplemented, without treating the latter as impossible in the production model.

The next visible checkpoint should run a full game with varied outcomes, expose
seed/model version, reproduce the same event log for the same inputs, and pass
multi-seed legal-state/termination/continuity and independent available-stat
conservation checks. Report failures, partial fields and coarse advancement/
manager assumptions. It is neither calibrated 2019 player simulation nor P-01/
P-02/G1 completion. No desktop scaffold, new data download or distribution.

If the maintainer retains strict sequencing, implement scoring coverage and a
complete clean-game golden first, then the remaining approved ER/context evidence;
request G1 signoff before starting probabilities. Do not replace those requirements
with the scripted or experimental preview.
