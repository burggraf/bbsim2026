# Season usage and rotation management — implementation plan v1

Status: implementation authorized; work and verification in progress, not statistically validated. User-requested
extension of season management: historical PA/IP pace should softly influence usage,
with leeway; normal starter rotation and bullpen situation suitability remain primary.
Live task status belongs only in [PROGRESS.md](../PROGRESS.md).

## Maintainer clarification — 2026-10-18 (supersedes dated-roster requirements below)

Use the existing selected-season player/team data, not verified daily active rosters.
Do not acquire more roster data or infer injury/minor-league availability windows.
A player observed for a club belongs to its modeled season pool; a multi-team player
has separate club targets, spread over each club's actual played-game denominator.
Recent simulated pitching workload follows the player across clubs. Positions are
observed season positions assumed usable, not certified defensive eligibility.
Unknown positions/targets remain explicit. Do not select one actor on both sides of
a game. No new identities, invented appearances, hard quotas or outcome adjustment.
Season-end totals/roles/positions are intentional hindsight reconstruction inputs.
The earlier dated-roster interview and phase-3/stint prerequisites below are retained
as history, not current blockers. Existing numerical bands remain frozen.

**Next bounded checkpoint (`P-02-season-pool-policy-v1`):** build deterministic club
pools from authenticated existing appearances and reconciled targets; implement pure
PA pace, compatible hitter preference, rotation/rest and relief-order helpers.
Observable acceptance: full source/target schedule binding; per-club/role totals
reconcile (integer outs); no source mutation; unknown-vs-zero distinction; no daily
windows; position compatibility and occupied-ID exclusion; inside-band neutrality;
correct under/overuse direction; rest before pace, four full starter rest days,
same-day exclusion, >6 relief outs next-day exclusion and fourth consecutive day
exclusion; stable ties and bounded starter shortening. Audit real-data coverage and
run regression suites before acceptance. This checkpoint does not enable these
helpers in the native season runner: integrated managed inputs, in-game bullpen/PH
hooks and seed1/2/3 usage comparisons follow, with fresh evidence. Historical mode
and its artifacts remain unchanged.

## Contract

- Count actual simulated batter PA and pitcher outs from canonical game accounting.
  Store IP as integer outs: 5.2 IP means 17 outs, never decimal 5.2.
- Compare usage with historical selected-season targets prorated by team progress.
  For a full-season team member, expected usage before a game is
  `historical target * completed team games / scheduled played team games`.
  Use the authenticated actual played schedule denominator, not an assumed 162.
- Inside a generous tolerance band, leave ordinary decisions alone. Outside it,
  introduce a bounded preference, not an exact quota or automatic eligibility ban.
- Maintain separate cumulative pace and recent workload/rest. Underused does not
  mean rested; overused does not mean injured or ineligible.
- Normal rotation order is the default. Substantial overuse may shorten an outing
  or justify skipping/resting a turn when a suitable rested replacement exists.
- Relief role/situation suitability comes before season pace; pace distinguishes
  suitable options and discourages repeatedly overusing the same pitcher.
- Batting pace influences compatible starting-lineup/rest choices and PH choice;
  it must not manufacture a tactically poor PH simply to spend someone's PA target.
- Do not alter outcome probabilities, simulated results, PA credits or outs to make
  targets match. Explain unavoidable over/underuse rather than forcing a match.

This is hindsight reconstruction, not forecasting. Season totals are permitted
hindsight inputs; later game outcomes and actual later substitution decisions are
not decision inputs. Explicit membership/availability assumptions remain necessary.

## Existing integration points and the important gap

`preview/season_runner.py` currently runs historical lineups and recorded closed
pitching casts, parses `controlled-accounting-v1`, then accumulates accepted games.
`preview/game_inputs.py` builds those inputs; `preview/bench_inputs.py` separately
exports assumed PH pools. Neither observed same-game cast nor observed bench is a
club roster from which to choose alternate starters, rested hitters or relievers.

Native integration reaches `engine/examples/text_game.rs` and support modules
`full_manager.rs`, `full_pinch.rs`, `full_simulation.rs`, `full_bench.rs` and
`season_accounting.rs`. The manager currently selects the first unused cast
pitcher; PH ranks legal candidates by current-matchup expected bases. Reuse these
boundaries and the canonical personnel reducer; do not build a second rules engine.

## Delivery order and acceptance checks

### 1. Reconcile targets and freeze the policy contract

Audit authenticated source PA and pitcher-out totals by player/team and, where
supported, membership stint and starting/relief role. PA must include the canonical
PA definition, not just the nine-category ratings sample. Preserve and explain the
known PA/credit discrepancies; missing values remain unknown, not zero. Reconcile
player-team totals with player and club totals and retain source hashes/recipes.

Use a whole-season club clock: divide each player-team target across that club's
actual played games, including players who historically arrived late or left early.
This is the maintainer-approved simplification, not inferred stint eligibility.
Never spend one club's target on another. Carry recent simulated workload globally
by player, without duplicating cumulative totals.

Before policy comparisons, freeze numerical tolerance bands, preference weights,
recent-rest rules, substantial-overuse thresholds and evaluation tolerances.
Suggested form: `allowance = max(absolute allowance, fraction * expected usage)`,
with larger absolute allowance for starter innings than reliever innings. Exact
values are proposed settings to preregister, not approved or empirically validated
numbers. Cover early season, zero/missing targets, late arrivals and tiny samples.

**Accept:** reproducible target table, discrepancy/coverage report, policy version,
explicit clock/rest/fallback rules and numerical acceptance criteria recorded before
running policy comparisons. No roster-completeness claim from participation proxies.

### 2. Deliver an observational season usage ledger first

Add the smallest ledger alongside existing runner accumulation. Track PA, pitching
outs, appearances/starts and dated recent workload by actor, team and supported stint.
Support both old and PH accounting explicitly, with strict version and expanded actor
bindings; do not accept PH output by simply relabeling its header.

Apply each reconciled completed game exactly once. Refused/failed/partial games
change neither usage nor completed-game progress. Pin pregame context, result and
policy versions. Resume by validating accepted game receipts and rebuilding ledger
state; a checkpoint cache is optional, not authoritative. Corruption/version drift
must refuse acceptance. The stateful runner must not silently skip a missing game
and call the later workload history a complete chronological season.

**Accept:** independent sums equal ledger PA/outs; duplicate/crash/resume tests;
refused games do not advance the clock; resumed and uninterrupted runs agree exactly;
old no-usage/no-bench controls retain exact bytes. Deliver a readable pace report
before the policy changes any personnel choices.

### 3. Supply selectable season-pool personnel

Build an authenticated personnel input from existing season club participation,
observed positions, pitcher roles and individual rating coverage. All club pool
members are modeled as available throughout its schedule, subject to simulated
workload/rest and legal positions; no daily roster reconstruction. Keep historical-
lineup replay as its existing mode; add explicit opt-in managed selection.

Resolve actual schedule order, including doubleheaders and suspended-game treatment,
before enabling recent-rest decisions. Use calendar dates for rest and team-game
counts for pace. Game two sees game-one workload; an off day creates rest, not PA/IP.
Use a normal five-man rotation as the initial policy with explicitly supported
roster changes/spot starts, not a promise that every 2019 club used exactly five.

**Accept:** no unsupported club identities or same actor on both sides; legal
complete positional lineup and sufficient pitching coverage; all selected actors
have ratings. Unknown positions, ambiguous modeled order or exhausted legal pools
produce an explicit refusal/diagnostic, not invented positions or identities.
This phase is a real prerequisite for changing starters or resting starting hitters.

### 4. Integrate batting usage preferences

Generate legal position-compatible lineups from the selectable pool, using the
historical lineup as an anchor rather than replacing all lineup strategy at once.
Outside-band usage can favor a compatible underused hitter or rest an overused one;
inside-band changes should not occur solely because of pace. Retain stable-ID ties.
Extend PH ranking only after legal/tactical filters and current-matchup evaluation;
freeze the maximum permitted tactical tradeoff so usage cannot dominate it.

**Accept:** tests for within-band neutrality, direction of under/overuse preferences,
catcher/position/DH constraints, no compatible replacement and legal PH restoration;
full games prove changed decisions and exact accounting without changing ratings.

### 5. Integrate rotation, rest and bullpen usage

Persist rotation order and last appearances. Choose the normal rested starter;
consider a shorter budget or a rested replacement only at the frozen substantial
usage threshold. Specify rotation-pointer behavior after a skip so it cannot cause
repeated skips or starvation. Insufficient rest remains distinct from season pace.

For relief, rank legally available/rested pitchers by declared situational role and
current matchup, then apply bounded pace preference among suitable choices. Feed
current-game workload into decisions as well as the pregame ledger. Preserve 2019
first-batter obligations, no re-entry and the absence of a three-batter rule.
Forced PH pitching restoration must use the same available-pitcher selection logic,
not bypass rest/usage by taking the first reserve. Retain empirical outing budgets
unless an explicitly versioned usage adjustment changes them; avoid duplicate hooks.

**Accept:** normal rotation, justified skip and return, no needless inside-band skip,
underused-but-tired refusal, off days/doubleheaders, mixed starting/relief roles,
situation-before-pace bullpen selection, exhausted bullpen and PH vacancy tests.
Selection explanations identify rotation/rest/situation/pace and any unmet target.

### 6. Prove the integrated behavior, then scale

Start with deterministic synthetic scenarios and an authenticated chronological
multi-team cohort covering DH/non-DH, trades, bench use, off days and doubleheaders.
Compare usage-enabled and disabled runs with frozen seeds, without retuning outcome
ratings. Report per-player and per-role pace deviations, band exceedances, starts,
rests/skips, relief workload concentration, legal-pool failures and tactical tradeoffs.
Report unknown/excluded players and incomplete schedules separately. Different game
lengths and total opportunities can make all historical targets jointly unattainable.

Run all applicable Rust/Python suites, examples/build/fmt/strict Clippy and protected
wire/default CLI controls; preserve raw failures and fresh exclusive evidence.
Advance to broader seasons only after cohort mechanics/coverage checks pass. A full
season ensemble belongs to S-03 and its preregistered 100+ season gate, not a substitute
for the smaller regression suite or a claim automatically earned by this feature.

## Roadmap boundary and first implementation checkpoint

This plan connects P-02 game policies with future S-01 personnel/schedule inputs and
S-02 season management; it does not waive A-04/S-01/S-02 dependencies or sign off G1–G5.
Strategic defensive replacements remain a separate unfinished P-02 slice. Target
reconciliation and the observational ledger can be prepared before that slice;
autonomous season-pool selection waits for integrated positional/rest legality checks.

**First concrete checkpoint:** phase 1 target/clock audit and frozen contract, followed
by phase 2 ledger + explicit PH accounting support. Do not start rotation selection
against the existing same-game-only pitching casts.

No new dependency, UI, injury generator, exact-usage quota, outcome-model refit or
source-data repair is needed for this scope. Club participation and positional
coverage remain visible; daily roster/stint coverage is no longer required.

## Historical first delivery and availability decision — superseded

Interview `b10ff106-df5f-409b-89c0-ec8bd811a033`: the user requires verified dated
roster inputs before autonomous selection, and approves the ledger plus historical-
lineup comparison first. Therefore phases 1–2 are the current delivery; phases 3–5
remain blocked/deferred. Do not introduce inferred roster windows to bypass this.
The first comparison measures the existing historical-lineup/closed-cast manager,
not an implemented season-pace policy or a managed full season. Implementation,
retained failures and measured results are in
[the observational ledger report](season-usage-ledger-v1-results.md).

Read-only source audit `/tmp/bbsim-season-usage-source-v1/` finds186,517 credited PA
and130,270 active-pitcher outs (936 on non-PA events). Sixteen mid-PA responsibility
switches mean terminal-participant rating profiles are not credited PA targets.
Preserve the62-credit sacrifice discrepancy. Suspended-game participants can carry
the original start date despite later arrival; Diekman's KCA/OAK2019-05-19 collision
shows why observation bounds cannot establish actual dated membership.

An observational ledger may record an explicit refused/failed schedule entry and
continue collecting later completed games, but that operation changes neither
usage nor completed-game clocks. It must mark history incomplete, preserve the gap
and refuse to present it as complete rest/availability context. Unrecorded gaps
still refuse chronological acceptance. Compare only source targets for accepted
IDs; always disclose excluded IDs and matched coverage. This is not authorization
for a stateful selection policy to continue through unknown workload history.

## Preregistered development settings — before usage-policy comparisons

The user authorizes implementation and historical-usage testing. These initial
settings are development choices, not medically measured fatigue or validated MLB
manager behavior. Freeze them for the first comparison; any later revision needs
its own recorded rationale and new version, not rewritten first-run results.

- PA pace dead band: `max(12 PA, 20% of expected PA)` on either side.
- Pitcher pace dead band: `max(18 outs for starter role / 6 outs for relief role,
  20% of expected outs)`. Combined player outs must also remain visible for mixed roles.
- Substantial starter overuse: excess greater than both 36 outs and 40% of expected
  outs; consider a rested replacement, otherwise reduce the normal outing budget
  by at most 20%. Do not stop at an exact outs target or override legal obligations.
- Normal starter rest: four full calendar days after a start; relief eligibility
  and any mixed-role return to starting must also consider intervening relief work.
  Rest after relief: do not enter on the day after more than six outs, or on a
  fourth consecutive pitching day. Pitching on the same calendar day is excluded
  from the initial managed policy. No-injury exception is invented.
- Batting replacements: only position-compatible legal alternatives; inside the
  band, no change solely for pace. Preserve lineup slots where practical.
- PH: retain the late/close and gain-greater-than-0.10 filters. Usage may choose
  among otherwise legal candidates within 0.05 expected bases of the best candidate;
  all selected candidates must still independently pass the original gain filter.
- Relief: situation/role class first, bounded pace preference next, stable-ID ties.
  The native policy contract must define the situation classes before evaluation.
- Missing target: neutral pace preference and an explicit unknown flag. A zero
  target is not a missing target, but is not an absolute legal-use prohibition.

First-run usage closeness criteria (report pass/fail; do not assert they will pass):
80% of compared batter-team records with at least 100 historical PA within
`max(20 PA, 25% of matched historical target)`; 80% of compared pitcher-team records
with at least 60 historical outs within `max(18 outs, 30% of matched historical target)`.
Report all players, opportunity bands, worst deviations, role splits and coverage
outside these headline groups. Report aggregate PA and outs deviations against a
5% bound. Full-season targets must not be compared to partial-run totals: use
matched chronological/cohort targets and explicitly distinguish partial results.
Use master seeds 1, 2 and 3 for the initial integrated development comparison if
coverage/runtime permits; one seed is a mechanism/development result, not an ensemble
or predictive interval. No seed selection by favorable results. Preserve baseline
and usage-enabled results, source hashes, schedule IDs and exact commands.

These thresholds test whether usage is reasonably close, not exact reproduction.
A failed threshold is a result to report, not permission to modify source credits,
exclude a bad result or retune the thresholds after seeing it.
