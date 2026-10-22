# Pitcher usage policy v2 — preregistration (2026-10-18)

Live status: [PROGRESS](../PROGRESS.md#resume-here). This contract is recorded from
`4e9c005` **before v2 implementation or simulation**. The source-population preflight
is not a policy evaluation. [V1 results](managed-season-v1-results.md) and
[read-only diagnosis](pitcher-usage-diagnosis-v1.md) remain frozen, including NO-GO.

## Question, scope and hypothesis

Can allocating starts across **all supported club starters**, and replacing strict
relief-class priority with continuous role debt plus a small situation preference,
improve individual usage without changing outcomes, rest or legality?

V1 sends about 98% of starts to a group responsible for 76% historically. Its relief
pace saturates outside a dead band and remains subordinate to workload class/ID.
These motivate v2; the missing entry-state evidence does not prove the proposed
relief change will repair any particular appearance. This is a **new retrospective
development policy**, not a refit of v1, fresh predictive validation or S-03 signoff.

Only pitching opportunity selection changes. Keep the existing source snapshot,
season club pools, original-date/game-ID schedule, ordinary-full-game normalization,
observed-position assumptions, ratings/fixed200 default/calibrated-v2 opt-in,
canonical accounting, batting selection and PH tactical/usage rules. No acquisition,
daily injury/minor-league windows, invented positions, outcome adjustment, forced
appearances, exact usage quotas, dependency or UI work. Integer outs remain the unit.

V2 explicitly supersedes v1's principal-five priority/reservation, rotation pointer,
discrete pitching pace and strict workload-class-first ordering **only in its opt-in
mode**. The numerical choices below are fixed before evaluating them, not asserted
to be optimal or physiologically measured. V1 and historical modes remain available.

## 1. Clock and starter opportunities

For the club about to play, let `c` be accepted completed games, `N` its authenticated
played-game count (161 for DET/CHA, 162 otherwise), and `g = c + 1`. This one-game
look-ahead allocates the upcoming opportunity; it is not a prematurely advanced
ledger clock. V1/batting clocks remain unchanged. Full-season aggregate targets
are intentional hindsight inputs, but no decision reads a future simulated result
or a dated future game's actual starter/substitution to select today's pitcher.

1. Starter candidates are every existing club-pool actor with observed position 1
   and a positive historical club start count `S`; no top-five admission gate.
   Filter occupied identities and the **unchanged global rest rules first**:
   no same-calendar-day pitching, four full days after a start, no day after >6
   relief outs, no fourth consecutive pitching day. Underuse never overrides these.
2. Allocate the club with fewer rested supported starters first, stable Away-first
   tie, then exclude its chosen ID from the other club. Preserve existing joint
   positional matching and global cast/bench uniqueness. If no distinct legal
   starter exists, refuse explicitly and stop; do not borrow an unsupported actor.
3. For each candidate, `A` is its accepted simulated starts **for this club**.
   Choose the greatest start debt `S*g/N - A`. Compare integer numerators
   `S*g - A*N` within the club. Ties prefer the longest time since any global
   pitching appearance (never appeared first), then stable ascending player ID.
   Equal rest/debt must not depend on source row order. There is no principal/extra
   pointer in v2; the accepted ledger is sufficient state. V1 pointers stay intact.
4. Keep empirical starting terminal-PA budgets, existing clean-outing qualification
   and marked pooled priors unchanged. A selected starter whose accepted starter
   outs exceed `T_start*g/N` by **both >36 outs and >40%** gets the existing 0.8
   budget factor; otherwise 1.0. This uses the v2 clock but the same frozen numeric
   threshold and maximum shortening. No second overuse-driven starter substitution.
   Preserve existing scoreless/no-hit extensions, first-batter obligations and
   last-pitcher-stays exhaustion behavior. Never stop at an exact season target.

Small start counts are preferences, not dates or quotas. Starting opportunities
may remain unmet because of global rest, shared identities, support or schedule;
record them and fail the relevant checks rather than fabricating a start.

## 2. Relief roles, score and budgets

Build relief candidates with the existing primary-pitcher heuristic and role-support
rules: positive club relief outs, empirical relief outings observed in the source,
or the existing marked-prior case for a primary nonstarter with unknown relief
target. A known start-only actor without relief support is not silently converted.
**Remove the blanket principal-five bullpen exclusion**: a mixed-role starter may
relieve when supported and rested. Its later start remains subject to global rest.
Keep current deterministic cross-side cast allocation and native size bounds; no
new emergency roster or shared-player ownership algorithm in this version.

At an actual entry decision, filter legal unused bench pitchers/not removed before
ranking. For each, let `T` be known club relief-out target and `U` accepted club
relief outs. Define continuous debt and the bounded situation score:

```text
E = T*g/N
B = max(6, E/5)
d = (E - U)/B
q = clamp(d, -1, 1)
m = 1 if the candidate matches the desired workload class, else 0
score = q + m/4
```

- Unknown `T` gives `d = q = 0` with an explicit unknown flag. A known zero target
  uses the formula; it is not unknown and is not an eligibility ban.
- Long class remains the exact comparison `relief_pa_sum >= 6*relief_outings`, using
  the same empirical samples/marked priors as v1. Desired long: inning <=5 **or**
  absolute score gap >=4; desired short otherwise. Classes are coarse workload
  assumptions, not measured leverage, handedness suitability or fatigue.
- Choose maximum `score`, then maximum **unclipped** `d`, then ascending stable ID.
  Thus equal/dead-band usage no longer always falls back to ID, and a class match
  contributes at most 0.25, not an unconditional veto over usage. There is no neutral
  dead band for v2 pitching selection; unchanged batting/PH preferences retain theirs.
- Comparisons must be exact rationals, not rounded exported pace/float scores:
  `d = 5*(T*g - U*N) / max(30*N, T*g)`. For example, `q=-1,m=1` scores -0.75;
  `q=0,m=0` scores 0 and wins. If both `q=0`, the class match wins. Test exact
  ties, saturation and class boundaries. Validate canonical nonnegative counts,
  `0 <= c < N <= 162`, u32 target/usage bounds and checked wide arithmetic; reject
  malformed/overflowing/duplicate inputs rather than silently clamping bad data.
- The unused/no-reentry filter means an entry candidate has no earlier pitching
  usage in this game. Do not charge a reserve probe or outgoing pitcher to a
  candidate's `U`. Current-game outs remain canonical accounting, credited once.
- Preserve the sampled individual/marked-prior **relief terminal-PA budget**, its
  manager-word lifecycle, and existing removal rules. No new budget multiplier,
  remaining-target cap, second fatigue roll or forced early exit is introduced.
  Role budgeting here means separate start/relief target feedback and the existing
  per-role empirical outing budgets, not a new pitch-count or IP estimator.

The same selector must serve voluntary changes, PH reserve feasibility and forced
pitching restoration. A PH probe is not a reservation or an appearance; restoration
uses its actual later state, which can have a different score/class preference.
Exhaustion retains the current pitcher for an ordinary hook; an unfillable PH
vacancy refuses, without illegal reentry or an invented emergency pitcher.

## 3. Versioning and retained decision evidence

Expose Python `--pitcher-policy v2`, defaulting to the existing v1 behavior. Keep v1
constants, input bytes, RNG streams and historical CLI output unchanged. V2 uses:

| Binding | V2 identity |
|---|---|
| Season policy | `season-pool-pitcher-usage-v2` |
| Native manager | `season-pool-workload-context-2019-v2` |
| Personnel schemas | `season-pool-lineup-v2`, `season-pool-pitching-cast-v2`, `season-pool-bench-v2` |
| Machine accounting | `controlled-accounting-managed-v2` |
| Game/batch receipts | `managed-game-receipt-v2`, `managed-season-receipt-v2` |
| Batch manifest | `managed-season-v2` |
| Entry trace | `pitcher-entry-decisions-v2` |

All three personnel files must agree on mode/policy/game/source bindings; reject
mixed v1/v2/historical sets. Preserve canonical accounting metric order and unknowns.
Export club `c,N` and each reliever's nullable `T`/integer `U` in the v2 cast, not
v1's `relief_pace` as a disguised continuous score. Include the starter counts,
candidate debts/rest exclusions and 80/100 scale in the pregame decision receipt.
Freeze the exact field spellings/trace serialization with interface tests before
integration behavior code; they must convey all the above without unchecked extra fields.

A separate native `--manager-trace PATH` sidecar is mandatory for v2 acceptance,
not mixed into canonical accounting stdout. Retain actual entry selections,
successful PH reserve probes and exhausted-entry attempts. For exhaustion, emit
once per unchanged side/current-pitcher/candidate-eligibility/desired-class tuple,
not once per PA. Each choice record must bind game/seed/policy, side, event index,
reason, canonical before-state snapshot (including inning/half/outs/score, current
pitcher/obligation, bench and removed IDs), every cast candidate's eligibility/exclusion and
exact debt/class/rank components, and the chosen actor or explicit exhaustion.
For an applied entry, bind its canonical substitution transition. Record entry
manager word/sample/base budget/scale/extensions when actually evaluated; never
consume another RNG word merely to fill a trace field. A probe cannot advance
entry RNG, rotation/usage clocks or accounting appearances.

Use the existing canonical personnel transition and shared selector, not a second
rules engine or an explanatory reimplementation that can disagree with selection.
The independent checker must recompute rankings from pinned inputs and retained
before state and reconcile applied entries with starts/appearances/final native
accounting. Mechanism tests additionally compare against the actual event stream;
entry snapshots alone do not certify full intervening event continuity.

Pin sidecar/input/output/recipe/native hashes in receipt-last artifacts. Validate
native integer-zero exit, strict accounting **and trace** before ledger acceptance.
Tampered/missing traces, incomplete histories and malformed metadata fail closed.
Rebuild reproduces pregame choices, validates traces and reconstructs accepted
usage without executing games. Preserve original publication I/O failures rather
than reclassifying an accepted game as a native failure. Trace enabled/disabled
must yield identical v2 accounting and selected personnel on compatibility probes.

## 4. Frozen evaluation populations and thresholds

Use the same authenticated 2,429 played games/30 clubs and targets as v1:
`targets.json` SHA256
`f45cd37997e1027ac17e4e4d752c4aa44e2be96f871f510408f72e432108972a`.
Source-population preflight confirms 455 batter-team targets >=100 PA, 565
pitcher-team targets >=60 outs, 240 starter-role targets >=60 outs, 331 relief-role
targets >=60 outs, and 250 club entries with >=5 starts. These overlap; do not add
them as disjoint populations. Role totals are 75,473 starter/54,797 relief outs.
All 565 qualifying pitcher-team entries have existing position/primary-pitcher support.
Archive sorted eligible identities/targets plus hashes **before implementation**.

Every fixed seed must pass all rows below; no averaging a failed seed away.
Use exact integer cross-products for percentage/band boundaries, without rounding
historical counts, eligible populations or fractional tolerances:

| Check | Frozen per-seed criterion |
|---|---|
| Completion/legality | All 2,429 games accepted; zero refusals/native/trace failures; complete chronological history |
| Existing batter usage | >=80% of compared >=100-PA entries within max(20 PA,25% actual) |
| Existing pitcher usage | >=80% of compared >=60-out entries within max(18 outs,30% actual) |
| Additional coverage | 100% of the 455/565 eligible batter/pitcher entries have a real accounting appearance |
| Role coverage | 100% of the 240/331 eligible starter/relief entries have an actual appearance in that role |
| Role usage | Separately >=80% within max(18 outs,30% actual), for each of those two role populations |
| Starting opportunities | >=80% of the 250 entries with >=5 actual starts within max(2 starts,30% actual starts) |
| Aggregate usage | Absolute deviation <=5% for PA, total pitching outs, starter outs and relief outs separately |
| Reconciliation | Exact independent actor/team/role totals and selector/entry/receipt agreement; no unknown source usage hidden |

The original compared-only thresholds are unchanged; **coverage and role/start
checks are additional v2 gates** chosen because v1 exposed omissions and role
substitution. An absent qualifying appearance remains null in the original
comparison and **fails coverage**. It is not converted into a measured zero row or
silently removed from the frozen eligible population. For an observed actor, zero
credited outs is a known zero; role coverage still requires an actual role entry.
Keep all nonqualifiers/unknown targets, known-zero roles and actual-only/simulated-
only entries visible. Unavailable source targets never become actual zeros.

Report each entry's targets, starts, appearances, start/relief/total outs, band status,
cast/candidate exposure, rest exclusions and known/unknown/prior flags. Report
principal-versus-other starts descriptively for comparability, not as an allocation
quota. Describe negative-pace/debt selections, class tradeoffs, exhaustion and
worst errors by role/club/opportunity band, with exact entry witnesses and limits.

Master seeds **1,2,3** retain the existing SHA256 per-game seed recipe and separate
manager stream. Run each once after engineering gates; statistical failure of an
earlier seed does not permit tuning before later seeds. Retain all three results.
Use retained v1 seasons on exactly the same game IDs/seeds as the development
baseline; do not reexecute or overwrite them. Pairing seeds does not hold actor
histories/opportunities fixed once personnel diverge. Report changes in coverage
and compared populations, not an apples-to-apples claim from unlike denominators.
Three previously inspected seeds are reused development evidence, not fresh test
seasons, a predictive benchmark, uncertainty intervals or the 100+ S-03 ensemble.
Any policy/weight/threshold revision after evaluation requires another version and
preregistration; failed v2 stays failed.

## 5. Implementation gates and handoff

1. **Pure Python slice first:** new policy helpers/tests, no default/in-game switch.
   Test all-starter admission, fractional starts/ties, global rest/occupied identities,
   scarce shared starters, mixed roles, off days/doubleheaders, c=0/final game,
   missing versus zero targets, checked arithmetic, source row permutation and
   immutable caller inputs. Explicitly cover an underused rank-6 starter with a
   rested principal and a low-start principal competing with other supported actors.
   Use hand-calculated rational examples/property checks, not outputs as assertions.
2. **Opt-in producer/native integration:** strict formats; shared-selector RED/GREEN
   tests for opposite classes/debt, 5/6 innings, gaps 3/4, exact PA/outings class
   boundary, saturation/ties, known-zero/unknown targets, actual first-batter and
   no-reentry constraints, ordinary exhaustion, PH probe/restoration and trace tamper.
   Budgets/removal/extensions and outcome RNG must have independent regression tests.
   Validate first/mixed schema/overflow errors before any simulation is accepted.
3. **Mechanism/compatibility gate before seasons:** authenticate/freeze v1 controls
   and sources before edits; all 2,429 static inputs; first 512 chronological actual-
   source games with accepted-ledger feedback (DH/non-DH, off days/doubleheaders).
   Repeat the same first four game contexts, including trace on/off parity. Preserve
   the existing 12 historical byte controls and four managed-v1 prefix contexts.
   Independently reconcile boxes, traces, role totals and receipt-backed rebuild.
   Static inputs check support/schema, not full-history availability. Cohort checks
   are engineering checks, not grounds to fit weights or certify usage.
4. **Fresh applicable verification:** full Python Werror discovery, Rust tests,
   example tests, offline locked examples build, fmt, strict all-target Clippy,
   diff/source guards and targeted review of the integrated change when authorized.
   Preserve raw failures, exact commands, seeds, source/model/rules/native/contract
   hashes. No ignored/weakened assertions or smoke-test replacement for full suites.
5. **Then evaluate:** exclusive v2 artifact destinations, three production CLI
   seasons and all frozen metrics above, independent arithmetic/entry verification,
   failed metrics retained, private evidence seal and explicit go/no-go. Passing
   this development contract still does not sign off G1–G5, historical eligibility,
   defensive replacements, UI/desktop/offline packaging or public redistribution.

Blast radius to inspect before implementation: `preview/season_manager.py`,
`managed_inputs.py`, `managed_season.py`, `season_runner.py` and their tests;
`engine/examples/text_game.rs`, support `season_pool.rs`, `real_lineup.rs`,
`full_manager.rs`, `full_bench.rs`, `full_simulation.rs`, `season_accounting.rs` and
all example/test callers of those support modules. Reuse canonical ledger/stats and
personnel rules. No engine outcome-model/refit, source importer or database schema
migration is needed. The smallest next task is the pure policy slice, not all gates at once.

## Dated mechanism-gate reconciliation clarification — 2026-10-19

After the first full mechanism-gate attempt (`proc_04ff`) stopped because the
independent box parser required every fielder putout to be known, the maintainer
authorized this narrow clarification on 2026-10-19. It changes only the mechanism
gate's independent reconciliation method, not v2 policy, metrics, thresholds,
engine accounting or the approved E-03/G1 accounting matrix:

- Keep fielder putouts and assists as unknown (`?`/`None`) wherever canonical
  accounting lacks qualified event-local evidence. Never coerce these values to
  zero, require a complete fielder-PO total, or infer individual fielder credit.
- Independently decode each game's canonical event-transition stream, validate
  transition framing/canonicality and continuity, and sum each transition's
  `delta.outs` for its recorded pitcher participant and defensive team. Compare
  that per-game pitcher-ID/team map exactly with known pitcher-role outs in the
  accounting box.
- Retain every other existing mechanism check: known team/actor and role identity,
  batter PA versus opposing pitcher BF, batting/pitching run reconciliation,
  independent trace arithmetic/witness checks, receipt-backed rebuild equality,
  and trace-on/off parity. Unknown PO/A remain visible; they are not treated as
  evidence of complete fielding accounting.

This clarification does not pass the mechanism gate by itself or sign off G1/E-03,
season accuracy, or any publication/default change. The fresh gate must still pass
all 2,429 static input trios, all 512 chronological games and every remaining trace,
role, rebuild and parity check before the authorized three-season evaluation begins.
Preserve the failed `proc_04ff` evidence unchanged and use a new exclusive directory
for the clarified run.
