# E-03: sole protected BB/HBP award — RBI cause evidence

Preregistered after local caught-K checkpoint `2071bbf`, before new tests or
production edits. [PROGRESS](../PROGRESS.md) owns live status. This is a narrow
step toward the [approved accounting matrix](g1-accounting-scope.md), not G1.

## Actual flow and primary rule

The preview's `event_for` advances only the uninterrupted occupied-First chain
for BB/HBP, counts Home crossings, and supplies that many batter RBI units.
Canonical `validate_credit_counts` currently bounds RBI by scored runs; it does
not establish the scoring cause or require an omitted RBI. `clean_forced_award`
checks the exact shape for a different purpose: reviewed clean-half ER, not RBI
causality. The credited accounting fold remains a partial supplied-counter view.

Cached 2019 Official Baseball Rules, Rule9.04(a)(2), printed page111/PDF124,
credits RBI when the run scores **by reason of** the batter becoming a runner
with the bases full on BB or a first-base award for HBP. Rule5.05/5.06 supplies
the protected first-base/force-chain advances. Do not infer "by reason of" from
final bases: earlier WP/PB/balk/steal within a PA can produce the same net shape.
The retained rulebook SHA256 is
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`.

## Bounded representation and meaning

Add optional `ProtectedAwardFacts { batter_id }` to Event, appended in BBE22
without changing existing credit tags. Presence asserts that the actual batter
receives the classified BB/HBP first-base award and **all represented runner
advancement/scoring is solely its protected force chain**. The fact excludes intermediate or
separately caused runner advancement/retirement within this PA. Unforced runners
stay at their before bases, either omitted or explicitly held. Batter finishes
First; no out, personnel change, extra base, timing/third-out or extra run.

This is causal input, not an inference from BB/HBP/RBI counters. Mandatory
current-batter PA and exactly one BB/HBP classification bind the fact. Walk
classification follows existing ordinary/intentional-walk rules; an intentional
BB has the same protected RBI cause. Current-pitcher participation stays distinct
from an inherited scorer's carried run charge.

Expose independent event-local `protected_award_rbi()` as
`Option<RbiContribution>` with a named batter map: one RBI only when the before
bases are full; otherwise an explicit named zero. Missing facts remain None.
Facts make the corresponding supplied RBI mandatory and exact, including zero;
wrong actor/omission/duplication must not be accepted as complete evidence.
Rule9.04(a)(2) does not require whole-PA NoMisplay: Unknown review does not erase
an independently asserted protected award cause, and does not establish E/PO/A
or earned-run completeness. Do not widen ER reconstruction in this slice.

Known participant/classification/role/unit/origin/carried-charge/result
contradictions must precede unrelated uncertainty on reducer, direct constructor
and valid whole-record semantic decode. Explicit richer delivery/contact/balk/
fielding retirement records first receive existing family preflights; unbound
legal compound histories stay UnsupportedRule, not categorically illegal.
Positive WP/PB/balk/SB/CS or another asserted retirement cannot itself supply
this exclusive protected-advance cause. Preserve bare legacy award histories.

## Observable acceptance before implementation

1. Cartesian eight masks × three outs × both halves × BB/HBP × optional
   unforced holds × movement orders × Unknown/NoMisplay; independent literal
   cause-derived RBI0/1 plus PA/AB/H/BB/HBP/BF/run/charge/codec checks.
2. Bases-loaded inherited scorer, deliberate IBB and ordinary walk-off controls:
   correct named batter RBI1 and old-pitcher charged R, no participation transfer.
3. Missing facts keep None even with correct supplied RBI/NoMisplay. Wrong named
   batter, omission/duplication/wrong actor, misclassified PA, unforced/excess/
   missing force moves, carried-charge and third-out forgeries exercise all three
   semantic entry paths with valid whole-event wire splice controls. Distinguish
   interface/compiler red, genuine semantic red and fixture/encoding failure.
4. Qualified known contradictions precede missing compound history; legal richer
   unbound episodes remain partial. Unknown review must not fabricate other box
   fields, zero errors or positive ER.
5. Before changing preview production, add BB/HBP-only facts and contribution
   assertions to its existing1,296-context property and witness a semantic red.
   Bind only the existing producer cause; RNG/categories/physical moves/scorer
   units remain unchanged. Existing seed1 presentation must stay byte-identical.
6. Trace all mechanical None constructors and whole-event codec-tail migrations;
   no changed historical actor/counter/physical fixture. Run full Rust, strict
   all-target Clippy, fmt, Python, native audits, offline baseline, generated
   diagnostics/CLI, prose/diff and frozen owned/non-owned checks. Preserve raw
   reds/failures/logs; inspect exact owned/staged contents before local commit.

No complete RBI segment/box coverage, full golden, broad error reconstruction,
calibration, publication, new download or independent reviewer is claimed here.
