# E-03 eligible uncaught-K First acquisition and later catcher throw

**Bounded parent-accepted implementation after reproduced priority corrections. Contract selected before TDD. Commit/publication status belongs to PROGRESS.**
[PROGRESS](../PROGRESS.md) owns live status. This advances the
[approved matrix](g1-accounting-scope.md), not a new exclusion or G1 waiver.

## Primary rule and selected interpretation

Cached 2019 Official Baseball Rules 9.13(a),(b) require K plus WP/PB when
handling permits the eligible batter to reach First. Rule9.13 Comment suppresses
handling credits when the defense makes an out before any runner advances.
Rule9.12(f)(2) separately describes the catcher recovering a third strike and
throwing the batter out **at First**, or tagging him out, while other runners
advance. Rules5.05(a)(2),5.09(a)(3),5.08(a) and9.15 distinguish legal First
acquisition, automatic retirement and after-first timing outs. Provenance is
in [2019 rules](2019-rules.md); cached text5072–5140 and5169–5173 inspected.

Select a narrow positive family that does not resolve all of the tag wording:
the eligible batter legally acquires First **because of the handling**, then
the catcher recovers that same delivery and throws to a **different named
fielder**, who retires the batter at Second/Third/Home. The throw-at-First and
catcher-tag cases in9.12(f)(2) are not this family. Rule9.13's batter-First
qualifier and before-advance exception can therefore be evaluated from supplied
facts, not from a putout total or final Out. Do not generalize this reasoning
to a catcher tag, return-to-First retirement, independent later episode or appeal.
Those lawful histories remain unresolved until their own facts/contract exist.

External search `mv3z57sc8kxyf3` found rule republications but no authoritative
same-play after-first case ruling. The MLB Means/Haggerty example concerns a
**later separate CS**, not this episode; it is not used as proof. No forum or
newer-year explanation substitutes for the inspected2019 primary text.

## Minimal interface and bindings

Append `RecoveryReview::ThirdStrikeBatterAfterFirstThrow {
putout_fielder_id: String, at: Destination }` at recovery tag5. Its factual
assertion is that the PA batter legally touched First before this delivery's
catcher recovered/threw to the named receiving fielder for the batter's out.
It is not a caller-provided WP/PB certificate, full path or complete credit list.
`HandlingAdvance { runner_id: actual_batter, reached: First }` must separately
bind that legal First acquisition to this handling before positive qualification.
No First touch is inferred from Second/Third endpoints or a PO/A credit.

- Context must be actual PA/K with `UncaughtThirdStrike`; canonical actual
  batter movement is `Plate -> Out`, charged to the current pitcher. Automatic
  ineligibility conflicts with this actual legal-First assertion even if the
  handling witness/unit is missing. Preserve earlier canonical error priority.
- Receiver must be defensively eligible, distinct from the asserted catcher;
  the pitcher can receive a throw, a known DH cannot. Invalid identity/Out-as-
  location is contradictory. First as location is a lawful unrepresented
  return/throw history, not a new positive result or illegal outcome.
- Positive support has exactly one event out, the batter's. More actual outs
  require additional retirement order/cause facts, not arbitrary K subtraction.
  For one out, any supplied PO must belong to the named receiver. PO omission
  stays partial; it is not inferred. Supplied catcher assist stays distinct from
  PB, and PB/WP attribution remains catcher/current pitcher respectively.
- At the third out, the batter is an after-first timing out, not a before-first
  or force out. Other legal Home reaches and run credits need the existing
  explicit timing facts. Canonical run/origin/charge/scalar contradictions must
  not become missing-pitch uncertainty.
- Every other supplied handling reach must retain its existing final-destination
  binding. Unbound intermediate/held/steal histories remain unresolved. The
  batter's explicit causal First reach is the selected intermediate exception;
  the supplied throw review establishes its ordering before the batter's out.
- A known qualifying family requires exactly one matching WP/PB unit, even with
  no pre-delivery steal review or no PO/A units. Missing First cause, unknown
  recovery or old bare AfterAdvance cannot acquire new positive certainty.
- Winning-run order is not represented by this first-touch/throw assertion.
  Bottom-ninth-or-later winning histories with this recovery remain unsupported
  without binding the recovery out before game termination, even without units.
  Do not globally prohibit terminal halves or ordinary third-out timing runs.

Internal codec becomes `BBE18`, rejects prior prefixes, preserves all existing
credit/enum tags and fields. New recovery payload is receiver string then
Destination; use the existing Destination codec. No fielding-fold, DTO field,
new event kind, stat counter, ER vocabulary, data/model/UI or public compatibility
promise. Historical checkpoint documents retain their historical versions.

## Observable acceptance selected before implementation

1. Both APIs and round-trip decode accept the explicit family for WP/PB,
   both halves, outs0–2, every eligible mask, Second/Third/Home receiver location,
   pitcher/noncatcher receivers and movement/credit/fact permutations. Original
   before-state is immutable; independent legacy physical snapshots validate.
2. Add literal PA/AB/BF/K/one-out, WP/PB, PO/A and carried-run comparisons.
   Omitted PO/A is partial, not a complete official zero or full box proof.
3. Required-unit omission/opposite/duplicate/wrong actor, wrong context,
   automatic first occupancy, wrong/foreign/DH/catcher receiver, false retirement,
   Out location, supplied wrong PO and before-first/force third-out facts have
   regression cases. Known errors precede unrelated missing review.
4. Missing handling-First cause, return-to-First, additional unordered outs,
   independent steal/intermediate causes and winning-run order retain uncertainty.
   Preserve legacy lawful partial records, recovered-before-first suppression,
   automatic-K/separate-recovery and the accepted First-touch contradiction guard.
5. Full-layout semantic wire forgeries use separately validated physical controls
   and positive round trips; prove appended tag/payload, older-prefix rejection,
   unknown tags/truncation/trailing bytes separately from semantic failures.
6. Preserve behavior reds separately from compilation/layout/helper failures.
   Inspect complete actual source/test/doc diff; run all applicable Rust,
   fmt/strictClippy/Python/native/audit/link/diff checks on a frozen snapshot.
   Record targeted parent review and its independence limitation. No delegation
   authorized, no E-03 completion, G1 signoff, official box/ER/accuracy claim.

## Verification and targeted parent review

Layout/version red `proc_fe3b` returnedInvalidEncoding before production edits.
Only then appended the type/tag/prefix boundary; `proc_1486` parsed the complete
new record but returnedUnsupportedRule versus the expected positive result.
Expanded unchanged-pitch-source `proc_4d91` had54green/7 semantic reds; isolated
`proc_4bb8` accepted an impossible automatic-First assertion without cause/unit.
Only then implemented qualification and factual recovery checks.

Keep non-semantic failures separate: `proc_9e53` had62green/two test-helper
failures. A context helper asserted its old Uncaught-only subset, and a false
Force reason was used to normalize its own physical snapshot. Fixed the helpers
only, keeping all expected errors and original valid physical/round-trip controls.
`proc_e065` passed64 methods then hit an E0004 helper migration in the separate
pitch suite; explicitly encoded the appended variant, no wildcard/suppression.

Parent inspection found canonical third-out run/actor errors could be masked by
unit omission. `proc_fc2b` reproduced paired-API IllegalCredit versus DeltaMismatch;
`proc_d7d3` captured that same mismatch through all three canonical paths. Only
then enabled early canonical run/actor checking for the explicitly bound one-out
batter throw. `proc_0c74` passed66 focused methods, fmt and strictClippy. This
priority change does not apply arbitrary multi-out ordering or invent force facts.

Initial full `proc_714e` passed346 Rust/59 Python and all supplemental checks.
Parent inspected the complete source/test diff and extended new-family opposite/
duplicate/wrong-actor unit controls plus meaningful simultaneous runner/touch/
credit reversals in the literal accounting cases. No production behavior or
assertion was weakened. Initial hashes/native artifacts and raw logs preserved.
Pre-priority **`proc_b5d8` exit0**, exact
`cargo fmt --manifest-path engine/Cargo.toml && shasum -a 256 engine/src/*.rs engine/tests/*.rs > /tmp/bbsim-e03-after-first-throw-source-sha256.txt && bash /tmp/bbsim-e03-after-first-throw-final-checks.sh`,
passed **346 unfiltered Rust (66 automatic-K),59 Python,native1,350,27
shared-upstream ER/54 box comparisons,fmt,strictClippy,216 prose links and diff**.
All26 frozen source/test hashes identical before/after; parent consumed complete
stdout/stderr. The native cases concern replay metadata, not this throw model's
historical accuracy. The new positive matrix executes768 API/round-trip frames.

Targeted parent/author inspection covers actual source/callers, primary passages,
full-layout semantic/encoding controls, complete corrected diff and the contract.
Acceptance is limited to the explicit one-out noncatcher receiving-throw family,
canonical-priority fixes and internal codec extension. **Not an independent
reviewer verdict**: no delegated reviewer or external semantic service launched.
No complete-box, ER-vocabulary, probability, historical accuracy, E-03 completion
or G1 signoff follows. Catcher tags, return-to-First, additional unbound outs,
intermediate/steal and winning-order histories still need their own facts.

Final inspection reopened acceptance: early unsupported Home timing could mask
known receiver/unit contradictions. `proc_0e42`'s no-Home control passed and did
not reproduce a bug; no production change followed. Actual `scored_runners`
requires timing only when Home is present. Corrected `proc_10f3` and control
`proc_a017` reproduced all-three-path UnsupportedRule versus IllegalCredit
before the fix. The bound one-out family now propagates known canonical errors
immediately, defers only UnsupportedRule run history until factual and known
unit checks finish, and still rejects valid-unit unknown timing as unsupported.
No run or timing fact is manufactured. `proc_6f0d` passed68 focused methods and
strictClippy, including all retained canonical actor/run-priority controls.

Final unchanged **`proc_8e9d` exit0**, exact
`shasum -a 256 engine/src/*.rs engine/tests/*.rs > /tmp/bbsim-e03-after-first-throw-source-sha256.txt && bash /tmp/bbsim-e03-after-first-throw-final-checks.sh`,
passed **348 unfiltered Rust (68 automatic-K),59 Python,native1,350,27
shared-upstream ER/54 box comparisons,fmt,strictClippy,217 prose links/diff**,
all26 hashes identical before/after. Complete stdout/stderr consumed. Parent
inspected complete corrected source/test differences, including the64-line
source and164-line test priority refinement. Previous346-test snapshots and
separate native artifacts remain preserved; they do not certify this correction.
Parent accepts the bounded corrected implementation on this final evidence,
not independent review, complete boxes, expanded ER or a gate waiver.
