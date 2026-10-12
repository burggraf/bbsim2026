# Observed-defense PO/A coverage

## Preregistration

E-03 slice selected after verified local `c41e55a`, before tests/production.
Prerequisites are the validated event-local ground throw, caught K, reviewed
sole safe-hit, protected-award and sole-HR PO/A proofs. This does not complete
E-03/G1 or certify E/PB/ER, all supported event families or official boxes.

Inspected `derive_accounting`, all five event getters, `FieldingContribution`,
`rules::fielding_eligible`, snapshot/lineup constraints, and actual fielding
callers/assertions in the engine tests and preview diagnostics/presentation.
The credited `fielders` map deliberately contains only supplied-credit actors;
existing tests assert its cardinality and old credited lines. Preserve that
map and every credited counter; do not weaken those assertions to add coverage.

## Observable contract

Add `EventTransition::fielding_contribution()` as the canonical union of the
five existing qualified PO/A getters. No new legal transitions or scoring facts.
Substitutions and other unqualified events return None. Do not infer universal
zero PO/A from zero outs, missing credits, NoMisplay, RBI or a complete-game flag.

Add a separate `GameAccounting::classified_fielding` map per team, with
`FieldingCoverageLine { observed_events, putouts, assists }`. The latter two
are Option<u32>, initially Some(0) for a newly observed actor only. No new rows
are created by substitutions or by unobserved/offensive roster membership.

Every non-substitution transition is an observed defensive opportunity for the
eligible actors in its **before-defense**, including runner-only events. Use
existing eligibility: exclude DH/bench/removed actors, include actual pitcher,
and count a no-DH pitcher appearing in the batting order only once. Never use
carried run-charge pitchers or after-boundary defense to identify these actors.

For each such actor increment observed_events once. A qualified contribution
adds that actor's named PO/A, absent names meaning zero only in this qualified
scope. If the event is unqualified, both totals become None for every actor in
that before-defense scope. Unknown is irreversible for that actor across later
known events; it does not taint an incoming/newly eligible fielder or the other
team. Keep removed actors' previously observed rows without inventing later
opportunities. Personnel changes alone do not add or erase coverage.

Some totals mean only the actor's covered observed defensive events, not its
whole game/career. A later-start/unfinished log can have known observed totals
without complete_game. Conversely a physically complete log can have unknown
PO/A. E/PB, ER and full scoring completeness remain independent and unchanged.

No canonical-event codec/DTO/validation, producer, probabilities/RNG, presentation or
existing credited-fold behavior changes. The new fold reuses canonical facts,
not supplied PO/A counters. Partial logs remain usable and honestly unknown.

## Test-first acceptance

1. Freeze production; witness missing APIs, add only schema/empty getter/map
   stubs, then witness semantic reds before behavior.
2. Independently built canonical ground/caught/safe-hit/award/HR events test
   getter union and literal observed-defense maps across DH/no-DH, both halves,
   masks/outs and independent movement/credit orders. Constructor/codec share
   identical immutable transitions; legacy credited maps/counters stay exact.
3. Contiguous known/unknown/reknown sequences, personnel changes/new actors,
   inning-boundary and terminal defense, no-DH pitcher deduplication and carried
   charge versus actual participation. Counts and unknowns are actor-local.
4. Runner-only unknown opportunities taint before-defense even without fielding
   credits; substitutions do neither. Omitted facts/review stay partial, cannot
   be repaired by complete_game or correct supplied counts.
5. Regression guard: preserve all existing credited-map cardinality/line tests,
   codec semantics, fielding positive/negative controls and seeded presentation.
   Run all applicable Rust/Python/native/audit/CLI/preview checks, formatting,
   strict all-target Clippy, fresh prose links and exact owned/staged/frozen
   equality. Record raw failures and seeds; local checkpoint only, no push or
   independent-review/G1 claim.
