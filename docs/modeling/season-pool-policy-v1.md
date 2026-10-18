# Season-pool usage policy v1 — component checkpoint

The maintainer clarified on 2026-10-18: use players in the existing data and aim
for approximately historical PA/IP. Daily active rosters, injuries and minor-league
dates are not the goal. This supersedes the earlier dated-roster prerequisite.
[Live status and next action](../PROGRESS.md#resume-here) remain authoritative.
[Overall usage contract and frozen criteria](season-usage-management-v1.md).

## Implemented boundary

`preview/season_manager.py` is a pure component, **not yet connected to season or
native game execution**. No new acquisition, dependencies, rating fit, outcome
adjustment, UI or public data distribution. Historical-lineup mode is unchanged.

- `build_pool(source, targets)` consumes `game_inputs.read_sources`' authenticated
  existing cohort and approved historical usage targets. It checks schedule binding,
  aggregates whole-season club membership and observed positions, separates club
  targets and start/relief outs, and preserves unknown targets. No date windows.
- Missing full-cohort rating observations use the existing zero-count league prior;
  cold flags are explicit. Unknown positions do not become invented positions.
- `pace(...)` uses actual completed club games / actual played schedule, with the
  frozen generous dead bands. Preference is bounded and neutral inside the band.
- `Policy(pool, ledger_snapshot, day)` requires complete simulated history and
  matching club denominators. It never advances clocks, spends targets or mutates
  source/ledger inputs. A missing actor in a complete simulated history has not yet
  appeared; that is distinct from an unknown historical target.
- `hitter(...)` makes a position-compatible usage preference relative to an incumbent.
  The integration caller supplies all other occupied IDs, including opponents.
- `starter(...)` follows a supplied rotation/pointer; rest precedes pace. Substantial
  overuse can prefer another rested rotation member, otherwise a 0.8 budget factor
  is returned. The scheduled pointer advances once, rather than following the
  substitute and consuming subsequent rotation slots. No hard season cap.
- `relievers(...)` preserves caller-supplied situation classes before usage ranking.
  This component does **not** invent native leverage classes or implement PH ranking.
- Recent pitching work is global across clubs. Four full calendar days follow a
  start; no same-day pitching, next-day pitching after >6 relief outs, or fourth
  consecutive pitching day. This is modeled rest, not measured fatigue/injury.

These helpers return proposed choices. Native legality/first-batter obligations,
2019 no-reentry rules and canonical accounting still own actual game transitions.
The caller must construct a complete unique positional lineup, resolve shared
multi-club IDs across sides, supply situation classes and apply returned rotation
state only after accepted games. Refused games cannot become complete policy context.

## Component evidence

External evidence: `/tmp/bbsim-season-pool-policy-v1/` (private, not durable storage).

- `red.stderr`: missing component assertions fail before implementation.
- `role-red.stderr`: combined pitching totals incorrectly hide relief overuse;
  independent reviewer reported the same defect. Role-specific target/usage fix
  has mixed-role, known-other-role/unknown-split and club-specific regressions.
- `green-v4.stderr`:19 focused tests pass, including810 pace-property cases,
  stable ties, position/occupied filters, global rest, rest-day boundaries,
  normal/skip rotation, exhaustion and bounded shortening. An initial fixture
  mistakenly labeled replacement pitchers as catchers only; retained green-v1
  errors correctly reject those positions, and the fixture was corrected.
- Read-only reviewer `b7b7cb51` reported the mixed-role P1. Narrow follow-up
  `58e69f41` confirms its resolution; broader final acceptance remains parent-owned.
- `audit.py`, `audit.json`, `pool.json` and raw stdout/stderr: authenticated existing
  2019 inputs, independent per-club/role sums, start+relief reconciliation, row-order
  repeat and before/after source hashes.30 clubs,1567 club-player pairs,1410 distinct
  people,146 multi-club people;186517PA/130270outs. Eight entries have no observed
  positional records; they remain unsupported for positional selection. No new
  identity mapping or roster source is needed. Cold flags include non-pitchers and
  pitchers without batting observations; they are not counts of failed imports.

Exact focused command:

```sh
python3 -B -W error -m unittest discover -s tests -p test_season_manager.py
```

Full verification driver: `python3 -B /tmp/bbsim-season-pool-policy-v1/verify.py`;
exclusive `verification-v1/` records exact commands, exits, raw logs and source pins.
Completed exit0 in747s:741 Python Werror/946 Rust/4 example tests plus build/fmt/
strict all-target Clippy/diff. All249 source/test pins match; `VERIFICATION_COMPLETE`
is present. This verifies the pure component, not its subsequent integration.

## Next integration (not delivered by this component)

Build explicit opt-in managed game inputs from these pools; preserve native metadata
truthfulness rather than relabeling participant-only cast/bench schemas. Connect
accepted canonical accounting back to the ledger, provide native bullpen situation
classes and PH usage tie preferences, and run chronological mechanism cohorts before
seed1/2/3 comparisons. Keep frozen bands and report failures/coverage. Full-season
usage success or S-01/S-02/G1–G5 signoff is not established by these component tests.
