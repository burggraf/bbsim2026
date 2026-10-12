# E-03: sole out-of-park HR — independent RBI cause

Preregistered after verified local PA RBI coverage checkpoint
`7815d19def01a7e44c8de6435456987c9f3fb84a`, before tests/production changes.
[PROGRESS](../PROGRESS.md) owns status. The [approved matrix](g1-accounting-scope.md)
still requires broader RBI causes, fielding/ER and independent complete goldens.

## Actual flow and primary rule

`validate_home_run` already binds the explicit out-of-park subtype to a safe hit,
zero outs and every before runner/batter reaching Home. Canonical origins/run
credits bind identities, carried pitcher charges and legal runs. They do not
assert that all scoring during a potentially multi-effect PA was by that hit.
`rbi_contribution()` currently keeps such positive histories unknown; supplied
RBI is not independent cause evidence. The preview's existing HR policy does
score every runner exclusively on the batter's out-of-park hit.

2019 Rule9.04(a)(1), printed111/PDF123, awards RBI for every run scoring unaided
by an error as part of the play begun by the batter's safe hit, including HR,
unless9.04(b) applies. Rule9.04(b) excludes force/reverse-force DP and the muffed
first-base throw that would complete a force DP. This slice is the zero-out sole
out-of-park HR case only; it does not infer general hit/error/sacrifice RBI.
Rulebook SHA256:
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`.

## Representation and scope

Append optional `Event.sole_home_run: Option<SoleHomeRunFacts>` in BBE23, after
protected-award facts, preserving existing tags. `SoleHomeRunFacts { batter_id }`
asserts that the actual batter's out-of-park hit is the exclusive cause of all
represented runner advancement/scoring in this PA, **unaided by an error**.
No earlier/separate advance, retirement, steal, delivery, balk, personnel change
or third-out history contributes to these net movements/runs.

Presence requires explicit OutOfParkHomeRun, actual-batter PA/Hit and the existing
exact all-to-Home shape/origins/charges/zero-out/run conservation. Independently
derive named batter RBI `1 + before occupied bases`, not supplied RBI counts.
Require exactly that many actual-batter RBI units. Missing facts stay None even
with NoMisplay, a favorable net shape and correct supplied RBI; old partial HRs
remain legal/unknown, including supplied-unit omission.

Expose `sole_home_run_rbi()` returning the existing `Option<RbiContribution>`;
add this positive proof to `rbi_contribution()` and the existing actor-local fold.
Unknown fielding review does not erase an independently asserted unaided scoring
cause. An unrelated earlier error/muffed foul chance may coexist with that cause
under Unknown, but still must not certify E/PO/A/ER or an errorless inning. Existing
NoMisplay/credit contradiction rules remain intact. No new ER reconstruction.

Known wrong actor/classification/unit/origin/charge/movement/out/other asserted
retirement contradictions precede unrelated uncertainty in all three canonical
entry paths. Reuse existing HR/credit/origin/run and other-family preflights;
unbound compound records retain existing legality/partial coverage, or
UnsupportedRule where their existing guards require missing chronology. They
are not universally illegal merely because this new proof excludes them.
Positive SB/CS/WP/PB/Balk cannot certify this exclusive-advancement fact.
No implicit HR RBI upgrade for missing causal input.

## Observable acceptance before implementation

1. Eight masks × three outs × both halves × both reviews × movement/credit order:
   literal independent RBI1–4, current-batter PA/AB/H/HR, named run/old versus
   current-pitcher charge, full constructor/reducer/codec equality/immutability.
2. Ordinary, top-ninth and multi-run walk-off/extra-inning HR cases: all permitted
   runs/RBI and carried charges preserved; no forced-winner/end-order invention.
3. Missing facts/correct or omitted supplied RBI remain partial. Bound wrong
   actor, missing/duplicate/foreign RBI or Hit, false origin/carried charge/out/
   Home/result assertions are rejected on all three paths using valid whole-event
   wire splice controls; encoding/version/tag failures stay separate.
4. Unknown review/prior foul-chance error preserve sole-HR RBI without ER certainty;
   known contradictions precede missing compound chronology, legal richer
   unbound episodes retain their existing partial/unsupported treatment.
   Actor-local RBI unknowns are irreversible.
5. Before producer change, assert HR-only sole-cause binding/contribution in its
   existing1,296-context property and witness a semantic red. Bind only existing
   HR policy; no new outcome/physical movement/scorer units/RNG/probabilities.
   Accepted seed1 presentation remains byte-identical.
6. Trace every None/header/tail migration without changing historical actors,
   counters or physical fixtures. Run all Rust/fmt/strict all-target Clippy/Python/
   native/offline/audits/diagnostics/CLI/prose/diff/frozen ownership checks; retain
   raw reds/failures, inspect exact owned/staged contents before local checkpoint.

No independent review, push/publication, new data, complete official RBI boxes,
G1 signoff or probability/UI work is authorized or claimed by this contract.
