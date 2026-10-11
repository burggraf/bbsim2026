# E-03 automatic strikeout with a separate recovery retirement

**Accepted bounded checkpoint; local commit status belongs to PROGRESS. Contract selected before TDD.** This follows the
[automatic-K delivery checkpoint](automatic-k-pitches.md) and the
[approved G1 matrix](g1-accounting-scope.md). [PROGRESS](../PROGRESS.md) owns live
status. E-03 remains incomplete and G1 pending.

## Primary rule and selected boundary

Cached 2019 Official Baseball Rules 5.09(a)(3), 9.15(a)(2), 9.13(a),(b) and
Comment, and 9.12(f)(2) distinguish an automatic batter out from a defensive
recovery retirement. The automatic batter cannot legally acquire first with
first occupied before the delivery and fewer than two outs. A catcher can
nevertheless recover that pitch and retire a different, already-on-base runner.
The actual recovery retirement, not subtraction of every credited K, controls
9.13's before/after-advancement comparison. The special recovered batter rule
in 9.12(f)(2) is not applied to this separate runner out.

The existing `SingleDeliveryFacts` can represent this bounded family without a
new tag, DTO, codec version, event kind or accounting counter:

- Actual PA/K and canonical automatic `Plate -> Out`, determined from **before**
  occupancy and outs; exactly one additional before-runner ends `Out`, for two
  event outs total.
- `BeforeAnyAdvance` or `AfterAdvance` names that separate recovery-retired
  before runner. `AfterAdvance` also names a different runner's handling-caused
  legal advance before recovery. That reach must reconcile with the final safe
  or Home movement and applicable pre-delivery steal review.
- Before-any recovery suppresses this episode's WP/PB. A qualifying named advance
  before recovery requires exactly one matching WP/current-pitcher or PB/actual-
  catcher unit. Additional causal reaches do not multiply the unit and cannot
  hide explicit unbound intermediate assertions.
- A third out is not globally excluded. With one out before the pitch, the
  automatic K precedes this pitch's defensive recovery retirement. Supplied
  third-out metadata must name the separate recovery runner, not the automatic
  batter. A named prior Home advance must agree with third-out timing and legal
  run credits. Existing canonical ending/conservation rules still apply;
  missing timing is not fabricated from movement-vector order. Rule 7.01(g)(1)
  compares the score **after** the top-ninth play: a tying run must allow the
  bottom ninth. Inspection exposed an existing pre-play-score check in the
  shared ending function; reproduce that independently with a legacy physical
  control before correcting it.
- An ordinary winning run ends the game immediately (7.01(g)(3)). Before-any
  recovery places its out before the eventual winning advance and can support a
  walkoff. An after-advance mixed recovery record does not bind the recovery out
  before the winning run or supply the needed end-game/appeal history; that
  terminal history remains unsupported, not an impossible outcome or a blanket
  terminal-event exclusion. Top-half terminal and ordinary inning boundaries
  with fully bound timing remain supported.
- Legal Home acquisition cannot leave a runner at an earlier base in a supported
  non-third-out frame (5.08(a) and Comment), even when a unit or review is missing.
  Legal Second/Third reach-and-return is unrepresented chronology, not an
  impossible advance or a positive qualification witness.

This is one delivery episode, not an exhaustive PA, error, steal or game history.
Missing recovery, unbound eligible/after-first batter retirement, additional
unordered retirements, intermediate reaches and independent completed steals
retain `UnsupportedRule` where facts are insufficient. NoRecoveryPutout does
not identify the cause of an additional final out. Removing a supplied unit
must not invent a complete official zero. Known actor, origin, charge, scalar,
credit, Home and explicitly bound ordering contradictions precede uncertainty.
No new exclusion, ER proof, box-completeness certificate or probability rule is
introduced.

## Observable acceptance selected before implementation

1. Reducer, direct constructor and round-trip decoder accept the bounded
   after-advance family for WP/PB, both halves, zero/one before outs, legal
   occupied-base/advancer/recovery pairs, and reversed movement/fact/credit
   order. Third-out cases preserve explicit timing and existing inning/game
   endings, rather than adding an arbitrary nonterminal cap.
2. The before-any controls suppress WP/PB without discarding K, PA/AB/BF,
   two outs, supplied PO/A, later FC advancement or carried run responsibility.
   They do not become batter-replacement `EventKind::FieldersChoice`.
3. Literal accounting separates current pitcher participation/WP from inherited
   scored R, catcher PB from fielding PO/A, and batting from fielding lines.
   Missing official scorer facts and ER remain unknown; no spurious pitcher or
   catcher batting line and no claim that this partial log completes a game.
4. Required-unit omission, opposite/duplicate/wrong-actor units, automatic
   batter safe reach/SB, wrong recovery actor, final-third-out order, and legal
   non-third-out Home-return contradictions are semantic errors before missing
   unrelated reviews. Use complete-record codec forgeries with unchanged valid
   physical snapshots and isolated encoding-corruption controls.
5. Missing recovery/steal reviews, an eligible retired batter, unresolved
   intermediate/held reaches, and richer unordered retirement/appeal histories
   preserve explicit uncertainty. A final out alone never disproves an earlier
   completed steal. Retain old single-out automatic and special recovered-batter
   suppression controls.
6. Preserve each actual semantic red before production edits. Run all applicable
   Rust/fmt/strict-Clippy/Python/native-property/audit/doc/diff checks and inspect
   the complete actual diff before bounded acceptance. No E-03/G1 signoff follows
   from this seam.

## Acceptance evidence and remaining work

Parent consumed the complete fresh read-only Sol-high review from
`9f5e61b1-86cc-4e6b-b002-2705cd73035c`: **No issues found; OK with notes** for
this bounded implementation and the shared post-play-score correction only.
Report: `/tmp/bbsim-e03-mixed-recovery-implementation-review.md`. The reviewer
inspected actual source/callers, synthetic tests, cached rules and supplied raw
logs; it did not run commands or independently certify source/hash binding.

Unchanged final `proc_f4de`, exact
`bash /tmp/bbsim-e03-mixed-recovery-final-checks.sh`, passed **327 Rust (47
automatic-K), 59 Python, native 1,350, 27 shared-upstream ER/54 box comparisons,
fmt, strict Clippy, 207 prose links and diff checks**. All 26 source/test hashes
were identical before/after and remain the reviewed snapshot. Separate baseline,
pre-review and final native outputs preserve earlier evidence. Semantic reds,
the separately corrected test-authored error-taxonomy mismatch and the independent
legacy ending red remain recorded in PROGRESS and raw logs, not rewritten.

The new positive matrix has 80 frames per handling class through both APIs and
round-trip decode. Paired API and complete-record wire tests cover unit omission,
last-out/type, Home and known timing-run contradictions; literal counts keep two
outs versus one PA/BF/K and separate inherited R/current WP/catcher PB/PO/A.
Before-any suppression and winning-run controls, eligible-batter/intermediate
uncertainty, special recovered-batter suppression and old encoding controls are
retained. The narrow ER proof is unchanged. These are synthetic mechanism checks,
not historical reconstruction or complete-box evidence.

Richer eligible/after-first batter retirement, independent steals, intermediate,
appeal/end-game history, Rule9.16 reconstruction and the full approved-field
coverage/goldens remain required. Legal unsupported histories remain legal
modeling requirements. **E-03 is incomplete; G1 pending.** No data/model/UI or
public release is authorized by this acceptance.
