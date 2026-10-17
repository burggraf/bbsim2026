# Bounded recovered uncaught K with earlier runner advances

## Observable acceptance (written before tests and behavior)

This is one canonical P-02 prerequisite, not P-02/G1 acceptance or generator
integration. The new typed recovery leaf asserts a catcher throw to a named,
distinct receiving fielder retiring the actual eligible PA batter **before
First**, after **all** supplied handling-caused advances by existing runners.
Existing recovery variants, Debug output, absent facts and bytes stay unchanged;
ordinary `AfterAdvance` with a retired batter remains insufficient proof.

- Actual before-state eligibility is First empty or two outs. Exactly one out,
  the PA batter from Plate to Out; no other outs, unrelated causes or personnel
  changes. Real active catcher/receiver roles, one receiver putout and one
  catcher assist, no self-assist. No new full-fielding, ER or RBI certainty.
- Nonempty, distinct handling advances bind actual before-runner identities,
  responsibility and final forward destinations; no held-only, intermediate,
  batter, invented actor, duplicate or unbound advancing move. Other runners
  may hold. This leaf supplies order, not an inferred steal history.
- WP/PB qualification continues through native handling/steal qualification;
  only real qualifying before-runner advancement earns the unit. Known malformed
  facts/credits take priority over missing steal history or winning chronology.
- A third out is explicitly `BatterBeforeFirst`; every physical Home crossing
  is named before that out, scores zero, and has no Run credit. Native reducer
  owns base clearing, cursor/accounting and ordinary inning/game endings.
- Bottom >=9 winning advances followed by recovery are refused without winning
  stop chronology; no post-win out is appended. Nonwinning and third-out
  no-score shapes remain supported.
- Append recovery tag 6 and BBE32 only for the new leaf. Reject old-header/new-tag
  forgeries and new-header/old-leaf records. Preserve exact old wire encodings.
- Helper adds an explicitly named joint one-base advance/recovery decision.
  Existing Automatic and RecoveredBeforeFirst HOLD; SafeReach stays joint safe
  reach. Empty advance requests fail, never silently fall back to HOLD.

## Verification plan

Retain separate interface compilation failures and semantic reds against a stub
before implementing behavior. Exhaust all eight base masks, three outs, both
halves and handling types. Check input immutability, inherited responsibility,
PA/AB/BF/K/outs/cursors, third-out physical Home/no-score, top ninth endings,
bottom ties/extras, nonwinning advances and winning refusal. Mutate roles,
actors, moves, credits, overlap, chronology and empty facts; combine known errors
with missing history. Verify apply/constructor/codec agreement, forged headers
and exact legacy wire preservation. Run all old pitch_occurrences_2019,
automatic_k_pitch_2019, strikeout_2019, full_strikeout and applicable contract
checks, owned rustfmt and strict Clippy. Raw receipts and baseline hashes live
outside Git at `/tmp/bbsim-text-game-recovery-v1/recovered-k-core-worker/`.

Only lib.rs, codec.rs, pitch.rs, the supplied helper/test and this document are
owned. The two old codec builders may receive only an appended mechanical enum
arm for compilation. Seeded rules/reducer/hit-runner-tag work stays unchanged.
No PROGRESS update, staging, commit, dependency/network or renderer/model work.

## Implemented API and bounded result

`RecoveryReview::RecoveredThirdStrikeBatterBeforeFirstAfterAdvances {
putout_fielder_id: String }` appends tag 6. `SingleDeliveryFacts.catcher_id` names
the thrower; the leaf names the distinct active receiver. The native PA binds the
batter, current pitcher and before-runner origins/charges. Its nonempty advance
vector is exhaustive for this episode's moving before-runners; omitted runners
hold. Credit requirements are one receiver Putout and one catcher Assist. The
native qualification function still handles steal-start review and one WP/PB
unit; no SB credit is inferred or accepted in this bounded leaf.

The supplied helper exposes
`StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { handling }`, jointly
advancing every before-runner one base and then retiring the batter before First.
It rejects an empty vector or an ineligible batter, never changes a requested
advance to HOLD, and lets canonical validation refuse unresolved winning stops.
The old `RecoveredBeforeFirst` helper and every old recovery encoding are intact.

All state application, scoring, cursor advancement and accounting still use the
native reducer/constructor. No new official ER, RBI or full-fielding proof was
added; inherited run charges stay with the original pitcher. A third-out Home
crossing is recorded physically but scores zero.

## 2026-10-17 — native omitted-leader correction

Independent review's Second→Home/Third-held counterexample was reproduced through
native apply before correction. This new leaf must also preserve joint endpoint
order of all surviving before-runners, treating omitted origins as holds. The
same bounded validator as hit/tag enforces this without touching legacy recovery
leaves or reducing a supplied reach. Native apply/constructor tests cover both
halves and all outs; the BBE32 forgery preserves the third-out after-state and
zero counted runs while changing only the prohibited passing chronology.
Historical worker receipts below predate this parent correction.

## Verification receipts

Raw receipts root:
`/tmp/bbsim-text-game-recovery-v1/recovered-k-core-worker/`.

- `02-interface-red.log`: missing decision/recovery interface (exit 101).
- `03-semantic-red-stub.log`: five behavioral failures against the helper stub.
- `04-semantic-red-canonical.log`: seven behavioral failures before implementation,
  including native UnsupportedRule for the positive new proof and erroneous
  acceptance of an empty new leaf. Ten inherited controls passed.
- `06-supplemental.log` and `07-supplemental.log` retain two test-author errors
  (missing BalkFacts disposition; u32 rather than native u64 wire-string length),
  corrected in tests without weakening production validation.
- Final `15-focused.log`: **146 tests / four suites**, including all 68 old
  automatic-K, 51 pitch-occurrence, five strikeout tests and 22 helper/core tests.
- Final `16-contracts.log`: **286 tests / 13 suites** (contract, reducer, rules,
  accounting, responsibility, caught-K, fielding, RBI, fixtures, defense, lineup,
  balks). `17-codec-regressions.log`: **170 tests / 13 suites**, including native
  BBE26–30 proof families and transition transport. **602 total tests passed**.
- `18-clippy.log`, `19-owned-fmt.log`, `20-diff-check.log`: strict all-target Clippy,
  owned rustfmt (skip_children protects seeded modules), diff check all pass.
- Exact final commands are in `15-focused.command` through `20-diff-check.command`
  and executable `final-checks.sh`; full failure chronology/commands in `README.md`.
- `legacy_wire_receipt` asserts three complete literal old encodings captured
  before interface changes, not hashes or re-encoded expectations. Old semantic
  codec suites pass unchanged except the two required appended tag-6 builder arms.
- `final-audit.log` verifies exactly eight allowed paths, all **294 non-owned
  baseline files unchanged**, the two six-line-only mechanical arms and no staged
  files. Baseline/worker-only patch/final hashes are retained outside Git.

All fixtures are synthetic and deterministic; no random failing seed or dataset
was used. Full repository Python/browser/generated-game suites were not run;
strict Clippy compiled all Rust targets. Integration, probability/model choice,
renderer behavior, stopped winning opportunities and P-02/G1 signoff remain the
parent's separate work and reviewer gate.
