# E-03 — known zero RBI for a bound two-out ground muff

## Preregistration

Selected after verified local `d84f4489da7edb0a3134915bda0ce061848c6c69`,
before new tests or production edits. The preview now supplies its ROE fielding
facts, but a positive-run two-out muff still leaves observed-PA RBI unknown.
The previous checkpoint deliberately implemented only the consistency guard;
this is a separately bounded coverage extension, not a reinterpretation of its
accepted proof.

Inspected `rbi_contribution()`, the independent actor-local RBI fold, canonical
`ground_muff::validate`, the producer and existing paired fielding/positive-RBI
controls. Frozen 2019 rulebook SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`,
Rule9.04(a)(1)–(3), printed111/PDF123: unaided causes and protected awards differ
from the error exception, which is explicitly **before two are out**.

## Bounded derivation

A canonically bound `MuffedGroundBallFacts` at exactly two before-outs asserts
that a mechanical muff lost the routine before-first batter retirement, with
all actual advances exclusively caused by that error and no independent episode.
That retirement would end the inning. Its named batter therefore has RBI0,
even if one actual legal run scores. The existing guard rejects positive RBI
units; this extension derives a named zero rather than treating absent credits
as evidence. Whole-chance review is independent: Unknown prior foul/fielding
chances do not erase this bound current causal exclusion, nor become covered
fielding or reconstructed earned runs.

Reuse the existing `rbi_contribution()` and irreversible actor-local fold; no
new DTO, wire version, public getter, supplied credit/default or producer change.
The existing no-legal-runs zero remains available for every observed PA. A
positive-run muff at zero/one outs remains unknown without the scorer judgment
required by9.04(a)(3). Missing muff facts with a positive run remain unknown at
all out counts, even with otherwise identical physics/credits. No blanket
error/RBI rule, hypothetical half reconstruction, ER certainty or G1 claim.

## Observable acceptance checks

- Independent384 half/out/mask/review/move-order/credit-order frames: literal
  named contribution and folded batter coverage, canonical constructor/codec,
  unchanged physical/credited counters. Both full and Unknown review qualify
  only the bounded two-out causal exclusion; zero/one-out positive cells stay
  None and existing zero-run cells stay Some(0).
- Paired missing-fact legacy controls with identical before/after/delta: positive
  runs stay None; reviewed fielding changes independently and is not normalized
  as an RBI proof. Prior-foul E2/Assist partial history retains RBI0 only with
  the causal bound while fielding axes remain None.
- Contiguous repeated batting-order history: known zero accumulation, missing
  positive-run cause irreversibly taints only its named batter, later bound
  opportunities cannot repair it. Other batters retain their own coverage.
- Existing two-out guard and generated-preview paired test expectations must
  explicitly assert only changed two-out RBI cells before normalization; all
  other statistics/assertions are retained. These declared coverage migrations
  follow the new semantic red, never weaken old malformed-input checks.
- Freeze production before new tests; witness genuine None-versus-named-zero
  reds. Run all applicable full Rust/Python/native/audit/diagnostic/CLI checks,
  consume complete logs, inspect actual staged snapshot and verify exact source
  equality before a local checkpoint. No push or independent-review claim.

[PROGRESS.md](../PROGRESS.md) owns status, commands and evidence.
