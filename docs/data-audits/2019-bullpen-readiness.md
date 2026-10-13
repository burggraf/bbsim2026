# F-01-bullpen — Seattle–Oakland offline readiness audit

## Preregistered scope and stop

D-18: maintainer selected “Audit bullpen readiness for Seattle–Oakland using
existing offline data, then stop with a concrete pitching-change implementation
proposal” in interview `962fd443-9786-4157-90b6-b1f14a091dd0`. Base `0c20cfc`.
Exactly four owned paths: this report, `docs/PROGRESS.md`, new
`audits/bullpen_readiness.py` and `tests/test_bullpen_readiness.py`.
All other187 tracked files remain byte-identical. No simulator/manager/ratings/
UI changes, transfers, dependencies, delegation, roster promotion or gate waiver.
No cohort measurements precede this contract. Source/caller inspection only:
existing lineup export provides games, people and fielding appearance ranges;
engine substitutions require explicit bench membership and preserve PA cursors.

Reuse the existing `audits/2019-lineup-readiness.sql` and its full source validator.
Read only the accepted F-03 snapshot SHA256
`d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`,
supplement SHA256
`554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82`,
and opportunity report SHA256
`382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e`.
Fixed target `OAK201903200`,2019-03-20; no matchup/score/count-based search.
Raw exports/reports/logs stay outside Git in `/tmp/bbsim-bullpen-readiness/`.

## Fixed source distinctions

- Validate the original whole-cohort game/person/appearance input using the
  existing lineup audit, including duplicate/orphan/byte-domain guards.
  Target must be a structurally supported starting-lineup candidate.
- For each target side, retain all literal same-game position1 appearances,
  including starter, player ID/name/raw bio hands/start/end and lookup gaps.
  Nonstarter position1 participants form an **observed pitching cast**, not a
  pregame bullpen. Preserve NULL/0/reversed endpoints; range order does not
  certify throw chronology, pitching responsibility, workloads or substitutions.
- Report every nonstarting same-game participant (all source positions, not just
  pitchers) with their literal rows; positions11/12 are batting roles, not
  pitchers. Count unknown side/position/range fields; preserve accepted0/NULL/
  full0..255 domains without invented exclusions or zero filling.
- Report all players with season position1 appearances for either target team,
  separate distinct pitching games/dates before/on/after target date, exact
  target-game support and source counts. Same-date other games are not prior
  appearances; no implied order from game IDs. Future observations, first–last
  dates and multi-team observations do **not** establish active-roster intervals,
  pregame eligibility, injuries, fatigue, readiness or rest.
- Join literal pitcher-role terminal-event counts from the accepted opportunity
  audit. Retain nine counts/N and exclusions/event totals as reported; no fitting,
  probabilities, exposure cutoff or ranking. Missing profile is explicitly
  unsupported, never borrowed/zero-for-unknown. Valid zero counts stay zero.
  Duplicate/malformed/negative/bool/nonconserving profiles fail visibly.
- `eligible_at_start` remains unknown for every proposed reliever. No verified
  transaction/active-roster, bench declaration or measured fatigue source exists
  in these inputs. Report unknowns as findings, not an empty known bullpen.

## Observable acceptance

Synthetic test-first controls cover exact pitcher-versus-batting-role selection,
starting/nonstarting separation, both sides, complete raw ranges/unknown hands,
missing/duplicate bios, missing/zero/malformed profiles, multi-team membership,
before/on/after dates without future eligibility inference, shuffled inputs and
nonmutation, duplicate/orphan appearances and unsupported target.

Publisher pins inputs before querying, DuckDB `-init /dev/null -bail -batch -csv`
with read-only attachments/external access disabled/SELECT stdout (no COPY),
retains raw stdout/stderr, rechecks DB/report/query/recipe hashes before exclusive
report publication. Refuse existing output/evidence and repository outputs;
failed input/native/drift checks cannot publish a report. Native projection tests
use synthetic local databases; do not install extensions or access the network.

Repeat actual exports/reports exactly. Independently reconcile target position1
rows, all nonstarting participants, season pitching-game/date partitions and
selected source profile vectors against read-only native SQL and pinned reports.
Freeze two tested Python sources; full Werror discovery/compilation, offline
locked Rust build/tests/fmt/strict all-target Clippy, applicable existing audit/
native/preview checks, source pins and unchanged simulator compatibility.
Inspect complete logs/scoped diff; exact four-path working/index/frozen/base/
non-owned/original-contract equality and fresh prose links; bounded parent/author
acceptance and verified local checkpoint. No independent review or G1/G2/rights
signoff. Stop with a concrete **proposal**, not an implemented manager.

## Decision to produce after measurement

Distinguish (1) verified historical pregame bullpen, expected unavailable, from
(2) a clearly labeled closed recorded-game pitching cast for a future internal
exhibition. State exact personnel/count support and missing facts. Propose the
smallest supported future opt-in change, explicit personnel/eligibility assumption,
versioned deterministic replacement policy, source-bound reliever profiles,
engine non-PA substitution/accounting/DH/one-draw invariants and viewer evidence.
Any modeled cast/bench/policy needs a separate maintainer choice; do not claim
historical availability or production P-02 readiness from appearances alone.

[Live status](../PROGRESS.md) · [Lineup source contract](2019-lineup-readiness.md)

### Pre-test ownership count correction

The executable `git ls-files` freeze counts **188**, not187, non-owned tracked
files at base `0c20cfc` (189 existing files minus the existing ledger; the other
three owned paths are new). Four-path scope and all predicates are unchanged;
the original preregistration byte prefix is retained externally.

## Actual source findings and bounded proposal

Actual `proc_541e` repeats byte-for-byte: report501,000 bytes, SHA256
`758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88`;
native export14,283,285 bytes, SHA256
`bef8108455784113702dd4003172ac99162d24dfb8522da4e2a0c53fa16dbf02`.
This is the unchanged original lineup export, not a new transfer/query taxonomy.
Source DBs/opportunity report/two frozen tested sources unchanged; native stderr
empty. Raw personnel, ranges/counts and complete reports remain outside Git.

Fixed target has35 literal appearance rows. All supplied side/position/start/end
fields are known and no reversed target range occurs. Seattle has five observed
pitchers (starter plus four nonstarters); Oakland has seven (starter plus six).
All12 have unique named bio support and conserved pitcher-role profiles, no
missing-profile fallback. Nonstarter included participant counts span9–327;
small exposure is a modeling uncertainty, not a selection cutoff. The six Seattle
and seven Oakland nonstarting participants include other roles: they are not all
pitchers or certified bench members. Season position1 participants number42/28,
including possible noncareer pitchers; this is an observed-position pool, not an
eligible bullpen or role classification.

For both target teams, **zero** prior-date2019 pitching-game observations support
pregame availability. Later-season observations and multi-team membership cannot
repair that unknown. These inputs have no active-roster/transaction/bench
certificate or measured rest/fatigue. Therefore **NO-GO for a verified historical
pregame bullpen**. No source facts have been converted into eligibility.

`proc_0e31` independent read-only SQL/fresh native terminal-participant aggregation
reconciles all35 target rows, every cast/nonstarting identity/name/raw-hand/range,
all joined count/N/event/exclusion fields and all70 season pitching-game/date/
multi-team profiles (95 detailed report rows, with overlapping views). All
eligibility fields remain NULL. Independent CSV962,495 bytes SHA256
`840d116f0a2bf5c0901aae26285d7c6a607df8034eb38dc49e7cea02cdd5e12c`;
fresh native profile CSV108,407 bytes SHA256
`80b7060e6cb7b769929ee9c69c60807d81ed906cb1ebd184d543fd3f12b5ee74`.
Full verification/acceptance/checkpoint status is in the live ledger, not inferred
from these diagnostic passes.

### Proposed next milestone — one modeled pitching change per side

Recommend a separate opt-in **closed recorded-game pitching-cast exhibition**,
not a historical roster replay. Explicitly assume that one source-supported
recorded nonstarter per side is on the modeled bench at game start. Choose the
lexicographically first literal player ID within each recorded nonstarter cast,
not a favorite, exposure threshold, skill ranking or historical entry timestamp.
Exact selected identities/names/profiles are in the external report; no borrowed
player or missing-data fallback. This would use only two extra pitcher profiles,
not all70 season position1 participants or a fabricated25-man roster.

Version a deterministic policy: at the start of each team's first defensive
half-inning numbered6, replace its starter once; the reliever finishes the game.
This is a simple visible policy, **not measured fatigue, optimal strategy or actual
historical timing**. Empty-base half boundaries avoid creating an unproved
mid-inning policy, while existing inherited-responsibility regression controls
must still remain green. Keep the DH/nine hitters, positional contact policy,
fixed player-probability formula and exactly one RNG word per PA unchanged.

Caller inspection identifies three real implementation seams:

1. `real_lineup::event_for` deliberately rejects a changed pitcher or nonempty
   bench/removed list. Keep that existing mode intact; add a separately versioned
   closed-cast identity/bench validator and initialization, not a relaxed guard.
2. `Ratings::read_tsv` accepts exactly20 starting roles. Add a separately versioned
   cast-bound exporter/loader for22 roles (18 hitters/four pitchers), keep old
   bytes/loaders and all old modes exact. Current probability lookup already uses
   `state.teams[defense].pitcher`; do not add browser-owned probabilities.
3. `preview::game_from` is currently a PA producer loop. Add a separate manager
   seam before PA sampling: deterministic `EventKind::Substitution` plus
   `PersonnelChange::Pitcher` goes through canonical `apply`, preserves batting
   cursor/PA totals/bases/score, consumes **zero** RNG words and creates an
   immutable before/after transition. Suppress repeated changes/reentry and
   never substitute after game over. Do not falsely encode the change as a PA.

Future acceptance: strict closed-cast/provenance/bench/profile guards, both
DH sides/half boundaries, no reentry/extra change, current-pitcher sampling/stat
credits and outgoing/incoming observations, inherited runner charges, full
continuity/codec/available-stat accounting, literal substitution narration/UI
frames, fixed seeded game ensembles, full suites and byte-exact old modes.
No pitch counts, workload effects, leverage, tactics, batting substitutions,
other matchups, date-correct active rosters, desktop/season or gate expansion.

**Stop now for a maintainer choice**: approve this deliberately modeled cast and
one-change policy, request a different policy, or require additional eligibility
sources first. The audit provides a concrete option, not permission to implement
it. Production P-02, G1/G2 and data distribution rights remain open.

### Reproduction

From the repository, choose a new external output (existing report/native paths
are refused):

```sh
python3 -Werror -m audits.bullpen_readiness \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  /tmp/bbsim-outcome-details/2019-outcomes.duckdb \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-bullpen-readiness/report-new.json
```

Exact independent SQL/helper/raw checks/full logs are retained under
`/tmp/bbsim-bullpen-readiness/`; they are local, unbundled and may expire.
