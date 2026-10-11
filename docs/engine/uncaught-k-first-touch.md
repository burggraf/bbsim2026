# E-03 uncaught-K legal First acquisition versus before-first third out

**Bounded parent-accepted implementation; contract selected before TDD. Commit status belongs to PROGRESS.**
[PROGRESS](../PROGRESS.md) owns live status. This is a small known-fact guard
within the [approved matrix](g1-accounting-scope.md), not positive qualification
of all eligible/after-first recovery histories or an E-03/G1 completion claim.

## Evidence and boundary

2019 Rule 5.08(a) distinguishes a batter-runner retired **before he touches
first** from a timing retirement after First. Rules 5.05(a)(2), 5.09(a)(3) and
9.15(a)(2),(3) distinguish automatic K retirement from an eligible batter-runner.
Cached primary passages/provenance belong to [2019 rules](2019-rules.md).

`SingleDeliveryFacts.advances` supplies handling-caused legal acquisition, not
an intended award or an exhaustive chronology. An explicit eligible batter's
legal `HandlingAdvance { reached: First }` cannot coexist with that same batter
as the named `ThirdOutKind::BatterBeforeFirst`. This contradiction does not
require deciding WP/PB, a recovery cause, a steal or a final box's completeness.
A retired actor cannot acquire a base legally after already being retired.

The actual shared path is reducer/constructor -> `rules::batter_charge` ->
`pitch::preflight/validate`; decoding calls the same constructor. Existing
qualification returns unknown for intermediate causal reaches, and generic
third-out validation knows only final movement/typed reason. Hypothesis: the
missing cross-fact check lets this represented contradiction become
`UnsupportedRule` with a unit, or be accepted as a no-unit partial input.
Parent reproduced both public APIs and full-layout decoding before production
edits: `proc_c3e3` accepted the minimal contradictory no-unit record through both
APIs; `proc_2b96` had49 green/3 semantic reds, including the full-record decoder.
Legal partial and canonical charge-priority controls were already green. Only
then added the narrow shared preflight guard; `proc_f131` passed all52 automatic-K
methods, fmt, strict Clippy and diff checks. Those initial focused greens do not
certify the later scope correction or replace the full verification below.

Scope is **explicit First acquisition**, the PA batter identity, supplied
uncaught-third-strike context and same-batter before-first third-out metadata.
Do not infer a First touch from a Second/Third endpoint, vector order, K credit,
protected award or an absent fact. Do not globally reject intermediate reaches,
retired batters, appeals or third-out events. Keep canonical malformed state,
participant/origin/charge/count and automatic-K eligibility checks ahead of
missing chronology. Use the established `IllegalCredit` taxonomy for the
contradictory supplied handling assertion, as for recovered-before-first facts.

## Observable acceptance

1. With two outs before the delivery, all eight before-base masks, both halves,
   both handling classes and canonical `Plate -> Out`, explicit legal batter
   First acquisition plus same-batter before-first third out is `IllegalCredit`
   in reducer, constructor and complete-record decoder. Missing recovery/steal
   review, no supplied unit, vector reversal and an unrelated causal witness
   must not mask it. Original before-state remains immutable.
2. A legal no-First-acquisition before-first K remains supported. The same
   physical ending with explicit First acquisition and after-first timing
   metadata remains a usable no-unit **partial** input; supplied WP/PB stays
   unsupported until richer qualification is separately selected. Neither
   absence nor retained partial accounting certifies an official zero.
3. Preserve the old runner-only safe-reach-then-retirement uncertainty test,
   special recovered-third-strike-before-first suppression, automatic-K and
   separate-recovery positives, before/after ordering, named runs, inherited
   responsibility and ending controls. Add literal PA/AB/BF/K/outs checks for
   the legal partial after-first control, without ER/completeness expansion.
4. Semantic wire tests splice complete valid event records with independently
   validated physical snapshots and positive round-trip controls. Keep tag/
   truncation corruption separate; no production codec/DTO/tag/version change.
5. Preserve actual semantic reds, run applicable full Rust/fmt/strict-Clippy,
   Python/native properties/audit/doc/diff checks, inspect the full diff and
   record targeted review before bounded acceptance. No data/model/UI changes,
   new matrix exclusion, probability floor or G1 signoff.

## Verification and targeted parent review

Parent review of the actual shared source/callers, primary rules, complete diff
and synthetic tests found one scope/priority issue: the initial predicate also
caught an actually safe batter falsely named as the third-out actor. That belongs
to canonical `DeltaMismatch`, not this retired-batter `IllegalCredit` guard.
`proc_458d` reproduced the paired-API mismatch; `proc_ef52` independently captured
all three paths as `IllegalCredit` versus `DeltaMismatch` before narrowing to
actual `Plate -> Out`. No assertion was relaxed. The positive physical/round-trip
control validates first, and the final regression exercises all three paths.

Corrected frozen `proc_466c` passed **333 unfiltered Rust (53 automatic-K),
59 Python, native1,350, 27 shared-upstream ER/54 box comparisons, fmt, strict
Clippy, 211 prose links and diff checks**, exact
`cargo fmt --manifest-path engine/Cargo.toml && shasum -a 256 engine/src/*.rs engine/tests/*.rs > /tmp/bbsim-e03-first-touch-source-sha256.txt && bash /tmp/bbsim-e03-first-touch-final-checks.sh`.
Parent consumed full stdout/stderr; all26 source/test hashes were identical
before/after. Earlier reds, initial snapshot and separate final native outputs
remain outside Git. The two matrices each execute1,024 frames, with explicit
frame counts, full-layout semantic wire tests and retained legal controls.

Parent inspected the complete corrected source/test diff, tracked doc diff and
new contract. The final12-line production addition binds exact context, First
acquisition, actual retired batter movement and same-actor before-first metadata;
existing canonical origins/charge/eligibility checks precede it. It does not
alter qualification, canonical DTO/codec/fold, ER, destinations or endings.
Parent accepts only this known-fact guard. **This is targeted author/parent review,
not an independent reviewer verdict.** No child reviewer or external semantic
service was launched without delegation authorization. It does not confer
E-03 completion, G1 signoff, complete boxes or historical/predictive accuracy.

## Not selected

Rule 9.12(f)(2)'s recovered/tagged batter wording must not be silently generalized
into a positive WP/PB result for every later retirement. The named existing
`lawful_safe_reach_then_retirement_does_not_get_a_blanket_illegal_ruling` fixture
is a runner-only intermediate-history case, not proof of a batter's First touch.
Keep it unchanged. Positive eligible/after-first retirement qualification and
more complete recovery/appeal/end-game/Rule9.16 histories remain required work.
