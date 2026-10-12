# Reviewed protected-award and sole-HR zero PO/A

## Preregistered bounded E-03 slice

Selected after verified local safe-hit zero-PO/A checkpoint `8142a49`, before
new tests/production changes. Uses the approved [matrix](g1-accounting-scope.md)
and retained 2019 Rules9.09/9.10 (printed120–122/PDF132–134, PDF SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`).
Putouts require actual/automatic retirements. Assists require a resulting out,
an out prevented by subsequent error, or interference/out-of-line retirement.

Inspected `protected_award::validate`, `sole_home_run::validate`, common
`validate_credit_counts`/credit-role checks, canonical callers, existing scoped
fielding getters and both independent family fixture/wire/rejection helpers.
The two cause validators require zero retirements but currently do not prohibit
an otherwise eligible Assist under NoMisplay; common counts cap PO at outs but
not assists. Hypothesis to test: a bound reviewed no-retirement cause currently
accepts this known contradictory unit. Do not certify independent zero maps
without first rejecting that contradiction on all canonical paths.

## Contract and limits

Add `protected_award_fielding()` and `sole_home_run_fielding()` returning
`Option<accounting::FieldingContribution>`. Each qualifies only its respective
canonical exclusive cause plus `FieldingReview::NoMisplay`; return literal empty
PO/A maps, zero only for that event's before-defense scope. The protected case
uses existing exact BB/IBB/HBP first-base force geometry and optional unforced
holds. The HR case uses existing out-of-park subtype/all-occupant scoring and
zero retirements. These facts plus full review rule out all Rule9.09/9.10 causes.

Missing exclusive facts or Unknown review remains None. Correct supplied RBI,
classification, final bases, zero outs, or NoMisplay alone do not qualify.
Unknown-review prior foul-chance errors can retain independent RBI; preserve
existing legal/partial unbound histories. Do not globally cap assists by outs:
error-prevented-out assists and richer unbound histories remain legitimate.

Only bound causes with full NoMisplay review gain an Assist-unit contradiction
guard (IllegalCredit), before unrelated compound-history uncertainty. Existing
PO/actor/classification/origin/run/retirement checks remain. Unknown-review
bound records with prior-chance Error/Assist must retain their existing partial
semantics and must not gain independent zero maps. Reject no unbound record
merely for failing the new qualification. Do not weaken existing priority tests.

No new DTO facts, codec version/tags, event shape, RNG/model/producer binding,
accounting fold/fields, ER reconstruction or presentation changes. These scoped
PO/A proofs do not certify E/PB, complete actor segments or official game boxes.
No E-03/G1 completion, independent review or publication claim.

## Observable test-first checks

1. Freeze production; witness missing-getter interface errors, add only None
   stubs, then witness actual semantic reds for independent Some-empty results
   and acceptance-versus-IllegalCredit contradictory Assist units.
2. Literal zero maps over 192 HR frames (mask × outs × half × independent
   movement/credit order) and 1,152 protected frames (BB/IBB/HBP × mask × outs ×
   half × explicit/omitted unforced holds × independent order). Exercise apply,
   canonical constructor and codec, without copying expected production output.
3. Missing facts/Unknown review, prior Error/Assist partial controls, ordinary
   award walk-offs and HR ninth/extras endings, actual before-defense attribution
   and unchanged RBI/ER/credited fold. Keep all prior tests and assertion bytes.
4. Valid independent full-event wire controls for known Assist-unit rejection
   under both bound causes, including compound Unknown chronology priority.
   Preserve isolated encoding failures and foreign-actor/counter guards.
5. All applicable unfiltered Rust/Python/native/audit/preview/CLI checks, fmt,
   strict all-target Clippy, exact accepted baseline/seeded presentation bytes,
   fresh prose links and full owned/staged/frozen inspection/equality. Retain
   failures, seeds and manifests outside Git; local checkpoint only, no push.
