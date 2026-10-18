# Managed season v1 — integration contract (2026-10-18)

Status/results: [single live ledger](../PROGRESS.md#resume-here).
[Verified pure policy](season-pool-policy-v1.md),
[frozen usage criteria](season-usage-management-v1.md).
This contract precedes integration tests/code; no dated roster acquisition.

## Acceptance and boundaries

Add an explicit opt-in Python managed runner, not a change to historical mode.
Read existing authenticated sources and independently pinned PA/outs targets.
Use all existing season club pools with observed positions, soft targets and global
simulated pitcher rest. No daily injuries/minors reconstruction, outcome adjustment,
hard season quotas, rating refit, invented positions or new source transfer.

1. Per-game inputs must be complete, deterministic and immutable: one unique actor
   per selected slot/position; no actor belongs to both sides including cast/bench.
   Use positional matching rather than a greedy choice that can strand a required
   position. Match positions jointly across both clubs (scarce slots first, pace/
   incumbent/target/ID candidate preferences, free candidates before augmenting an
   occupied assignment); no exponential search or claim of global utility optimum.
   Source game batting-slot/position layout is retained when valid; malformed layouts
   complete deterministically from selected observed positions. Starting pitcher
   occupies the non-DH pitching batting slot; DH follows
   game rules. Stable source batting-slot preferences are permitted hindsight data.
2. A five-member principal rotation follows descending historical starts/stable ID.
   Rest/occupied constraints precede usage. Allocate the club with fewer globally
   rested starting candidates first (stable Away-first tie), so a shared starter
   cannot strand a club when the other has a supported alternative.
   Extra observed starters may cover an
   exhausted principal rotation; advance the scheduled principal turn once only,
   and persist accepted rotation pointers with receipts. Never advance on failure.
   Reserve the current club's five principal starters from bullpen use. Other
   pitchers need positive observed relief outs or supported relief outings (including
   zero-out outings); a primary nonstarter with unknown relief target may use a marked
   empirical prior. A known start-only role is not silently converted into relief.
3. Preserve source empirical terminal-PA role budgets where supported; if a role is
   unavailable or internally unresolved, use an explicitly flagged pooled empirical
   role prior from clean source players (not invented individual outings). Keep
   rating cold flags and role-target unknowns explicit. Prior sample multiplicities
   must fit native bounds; the producer will publish the deterministic selection
   recipe and selected samples, not claim unavailable individual measurements.
   Frozen producer recipe: sort terminal-PA samples across clean source players for
   the role; take at most101 midpoint-rank quantiles (`(2i+1)*N//(2n)`). These are
   representative observed values, not preserved full-distribution multiplicities.
   Only the currently assigned role needs a prior when individual support is absent.
   No tuning against usage-comparison outcomes. Primary bullpen actors have observed
   position1 and either starts, no other observed position, or season pitching outs
   >= batting PA; incidental position-player pitching is not a normal bullpen role.
4. New `season-pool-lineup-v1`, `season-pool-pitching-cast-v1` and
   `season-pool-bench-v1` formats explicitly state season-pool availability and
   `season_policy=season-pool-soft-usage-v1`. Keep historical formats unchanged;
   reject mixed modes. Manager metadata/accounting must distinguish managed games.
5. Native policy retains separate manager RNG, first-batter/reentry/positional rules,
   empirical PA budgets and bounded scoreless/no-hit extensions. Optional starter
   shortening is 80% or100% before extensions. Unused relievers use workload-role
   classes first: mean relief PA>=6 preferred in innings<=5 or score gap>=4, otherwise
   shorter-outing roles preferred; bounded relief pace then stable ID resolve ties.
   These classes are modeling assumptions, not measured fatigue or leverage skill.
6. Managed PH preserves gain>0.10. Only candidates within0.05 of the best qualifying
   expected-bases value compete on PA pace; then value/stable ID. No outcome change.
7. Managed machine format `controlled-accounting-managed-v1` preserves canonical
   rows and adds explicit availability/usage policy, manager version and PH mode.
   Parser conserves all existing counts and actor checks. Accept into the existing
   exactly-once ledger only after native success and strict parsing; reproduce its
   snapshot/comparison from retained per-game inputs/output/receipts. No refused or
   failed game may be hidden or followed using incomplete policy context.
8. Preserve actual schedule and all ambiguity/exception flags. Modeled chronological
   ordering remains original game date then ID; suspended games are normalized full
   games on that modeled date, not literal historical continuation/availability.
   Stop on first refusal/failure; pending slots stay visible, not fabricated wins.
9. Freeze source/target/recipes/native hashes before computation; publish exclusively
   outside repository with receipt last. Verify sources unchanged at acceptance.
   Same seed/state must yield byte-identical inputs/native output and choices.

## Verification gates

- Semantic RED/GREEN tests: matching traps/exhaustion, DH/non-DH, cross-side shared
  IDs, rest/rotation advancement, unknown/cold/prior profiles and source immutability;
  strict managed wire/mixed-mode parser cases, native situation/pace/gain boundaries,
  first-batter obligations and unchanged default behavior.
- Actual-source chronological DH/non-DH and doubleheader mechanism cohorts; reconcile
  native rows independently and retain all refusals, fallback counts and failures.
- Fresh independent review of integrated changes, then all Python Werror/Rust/example
  suites, offline locked build, fmt, strict all-target Clippy and diff checks.
- Only after the mechanism/compatibility gates run fixed seeds1/2/3 usage comparisons,
  matched cohorts and full errors/coverage against the existing frozen tolerances.
  A failed metric stays failed; component success does not certify a full season.

## Ownership

Two independent contracts: native input/runtime bridge (isolated worker based on
`4a3c76c`, native files/tests only) and Python producer/runner (parent/main checkout).
The parent owns ledger/docs, reviews, native handoff integration, full verification
and acceptance. Native worker returns an owned commit and raw focused evidence;
parent does not edit that worktree. No push or G1–G5/public data approval.
