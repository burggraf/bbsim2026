# Ordinary direct caught-air prerequisite

## Preregistration

Selected after verified local `83366fe`, before tests/production. The
[observed support map](../modeling/observed-outcome-support.md) identifies a
high-volume main-path primitive for a later experimental contact-policy
milestone, not a complete historical catch parser or production model.

Add `AirContact::{Fly,LineDrive,PopUp}` and optional
`OrdinaryAirCatchFacts { batter_id, fielder_id, contact }` to canonical `Event`.
These assert one ordinary legal direct non-bunt catch retiring this batter
before legal First acquisition; before runners hold, no deflection/throw/other retirement or advancement in
this catch episode. No automatic infield-fly penalty, tag-up, sacrifice,
compound delivery, ground/K/hit/award/muff or ambiguous clock is inferred.
Whole-chance review remains independent; missing review/facts stays unknown.
Unknown events and valid prior partial logs remain usable.

Validate named current batter, eligible before-defense catching fielder,
ordinary PA/AB/no-hit classification, one physical out/no runs, held runners,
third-out before-first scoring and contradictory causal facts. With full
`NoMisplay` review, require one named PO and no assists; derive event-local
PO1/A0 independently from facts, union into observed-defense coverage. Do not
infer the fielder from supplied PO credit or equate credited totals with proof.
Reviewed E/PB0 and RBI0 reuse independent existing getters. Qualify only the
ordinary reviewed-clean catch in clean-half ER; no extension of deferred
misplay/two-out-prefix reconstruction, no repair of earlier uncertainty.

Keep existing no-air records byte-identical `BBE25`. Encode a new `BBE26`
record only when air facts exist; preserve/decode the old layout with None and
revalidate both versions. Preserve old no-air debug output; show the new field
when present. Mechanical additions of None to existing Rust event literals
must preserve every other byte/assertion. No probabilities, generator policy,
UI/radio, source importer, RNG, manager or roster changes in this slice.

## Fixed acceptance

- Freeze prior production/tests; distinguish missing-API/interface reds from
  semantic reds after schema-only stubs. Test every new function/getter.
- All three air classes ×eight masks ×three before outs ×both halves; ordinary,
  inning-boundary and terminal catches; optional held movements and independent
  movement/credit ordering. Constructor/reducer/codec agree and conserve PA/BF,
  AB/no-hit, physical outs, named PO and actor-local observed-defense coverage.
- Wrong/missing actors/PO, DH/bench/removed eligibility, extra/missing PO/assists,
  sacrifice/K/compound/advance/third-out/causal contradictions; no production
  assertion weakening. Missing facts/review does not certify PO/A or clean ER.
- Literal complete clean-half positive ER and unknown/reknown controls, with
  old ER uncertainty/failed seeds retained. No false official-box certificate.
- Exact parent/new no-air wire/debug compatibility; malformed/version/semantic
  codec forgeries fail. All prior tests/native/audit/baseline/2,000 games/100,000
  draws, formatting/build/strict Clippy/Python warnings-as-error checks run.
- Inspect exact owned stage, frozen/mechanical/non-owned equality and fresh
  prose links; local checkpoint only, no push/delegation/independent-review,
  G1/G2/rights/accuracy claim. [PROGRESS.md](../PROGRESS.md) is the live ledger.

## Implemented and verified slice

The API is a declared **before-legal-First** direct catch subtype, not a
universal clock assertion about every historical fly/line/pop out. Named legal
First touch contradicts this subtype even below two outs; without these facts,
otherwise legal incomplete acquisition/retirement histories stay partial. The
contact tag is supplied, never reconstructed from PO, text order or a fielder.
Known actor/credit errors precede unsupported compound-history resolution.

`air::validate` uses the existing held-retirement, credit, review and compound
preflight guards. `ordinary_air_fielding` derives the named PO independently;
observed-defense folding retains actor-local uncertainty and before-defense
attribution. The clean-play vocabulary recognizes the reviewed catch; existing
ER reconstruction algorithms and deferred positive-prefix gates are unchanged.
No parser, generator category/probability, sampler, manager or UI changed.

Fresh offline verification (`proc_5599`, exit0), with all774 stdout/227 stderr
lines consumed, ran `/tmp/bbsim-air-catch/full-checks.sh`:

- 567 Rust tests, including19 new focused tests; fmt check, generated-game build
  and all-target strict Clippy (`--offline --locked`, `-D warnings`).
- 124 Python tests with **all warnings as errors** (`python3 -Werror -m unittest
  discover -s tests -v`) and audited-module compilation. Expected negative
  FileNotFoundError/HTTP500 cases are retained, not unexplained failures.
- Original69 audit metrics, five fixture transitions,54 shared fixture cells,
  27 pitcher ER source cells and54 external box comparisons; native1,350 cases.
  Source ER cells do not certify engine reconstruction or independent G1 boxes.
- Baseline exact;2,000 repeatable legal games and100,000 fixed diagnostic draws.
  Outcome export/report/control repeats exact;19 independent scalar controls,
  17 result cells, all prior CSV fields/rows and non-added JSON unchanged.
  Both original/supplement input hashes and all33 changed Rust hashes passed.
- Four complete CLI traces (seeds1/2/3/99) are byte-identical to `83366fe`.
  Separately compiled parent/current engines (`proc_6a51`, exit0, empty stderr)
  encoded and independently decoded all316 transitions in those four games.
  Length-framed BBE25 bytes match exactly:304,463 bytes, SHA256
  `3c2e8afdbde8326bccfa76d43ed4b4871ecdddbcfc7d7ac28b2d5eee6dc76d43`.
- All27 prior migration files strip47 exact None-only lines back to HEAD;
  19 other prior Rust files unchanged; non-owned tracked bytes unchanged.

TDD evidence is retained outside Git at `/tmp/bbsim-air-catch/`: missing-API
`proc_6e33` (eight interface diagnostics), schema-only `proc_5269` (four semantic
reds/one control), extended `proc_fa07` (seven reds/three controls), and
`proc_125d` (known First-touch error priority red, generic partial control).
`proc_c8df`, `proc_8459` and `proc_56a0` are intermediate green evidence, not
substitutes for the final full run. No assertions, seeds or thresholds were
weakened. Exact owned staging/acceptance is recorded in the live ledger.

This is an engine prerequisite, not historical import support, a production
contact model, predictive accuracy, licensing clearance or G1/G2 signoff.
