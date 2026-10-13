# D-19 — pitcher-specific PA-workload managed preview

## Authorized milestone and preregistration

2026-10-13 maintainer rejected the arbitrary sixth-inning proposal as a realistic
hook. It was an unimplemented plumbing test, not a fatigue model. Interview
`b3ec6a67-afb5-41ae-b739-2d71de84007c` confirms: audit outing workloads, implement
and verify a separate opt-in preview, then stop for testing; observed mean outing
lengths separated into starts/relief; model all four Seattle/six Oakland recorded
nonstarters as available at game start; longer-average relief outings first with
stable ID ties. This is a closed exhibition, not certified historical availability.

Base `7c42374`, main/parent sole writer. No outing-profile measurements precede
this contract. Schema/caller inspection only: accepted caches have terminal PA
participants, starting-pitcher IDs, appearance ranges and ball/strike snapshots,
**not pitch totals or pitch sequences**. Do not fabricate pitch counts from count
snapshots, event endpoints, innings or a constant pitches-per-PA multiplier.
No transfers, dependencies, delegation, worktrees, push, release or gate waiver.

### Exact20 owned paths

- `docs/PROGRESS.md`
- `docs/data-audits/2019-bullpen-readiness.md` (append-only proposal supersession)
- `docs/modeling/pitcher-workload-preview.md`
- `audits/2019-pitcher-workload.sql`
- `audits/pitcher_workload.py`
- `tests/test_pitcher_workload.py`
- `preview/pitching_cast.py`
- `tests/test_pitching_cast.py`
- `engine/examples/support/managed_pitching.rs`
- `engine/tests/managed_pitching_preview.rs`
- `engine/examples/generated_game.rs`
- `engine/examples/support/player_ratings.rs`
- `engine/examples/support/real_lineup.rs`
- `engine/examples/support/preview.rs`
- `engine/examples/support/game_trace.rs`
- `preview/build_viewer.py`
- `preview/serve.py`
- `tests/test_live_preview.py`
- `tests/test_visible_viewer.py`
- `preview/README.md`

All180 other existing tracked files, including pure engine/rules/codec/accounting,
remain byte-identical. Verify executable ownership counts before tests; append a
count correction rather than rewrite this preregistration if necessary. Preserve
the original contract prefix, input pins and full raw evidence outside Git under
`/tmp/bbsim-pitcher-workload/`. Real source personnel, outing rows, reports, TSVs,
logs and screenshots stay outside Git; committed test personnel are synthetic.

## Frozen source/measurement contract

Read only accepted offline snapshot/supplement and pinned opportunity/lineup/
bullpen reports. Fixed matchup `OAK201903200`,2019-03-20; no exposure/result/
score/seed search. Reuse the unchanged full lineup validator and accepted outcome/
participant validation, not a replacement counting parser. Source provenance:

- F-03 snapshot SHA256 `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`.
- Supplement SHA256 `554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82`.
- Opportunity report SHA256 `382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e`.
- Lineup report SHA256 `5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b`.
- Bullpen report SHA256 `758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88`.

An **outing** here is a distinct player/game with a literal position1 appearance,
across all observed teams, not an active-roster interval or individual mound stint.
Retain duplicate-range/grain/lookup/unknown-side/unknown-role findings; do not use
appearance endpoint differences as workload. Multiple position1 stints in one game
remain labeled, not split into invented pitch chronology. Count each real terminal
PA against its literal terminal pitcher, including IBB, interference and sacrifices;
exclude no-play and runner-only events from workload. This is a participant-PA
proxy, **not official BF or mid-PA responsibility**. Preserve zero-PA appearances.

Independently reconcile full-cohort pitcher totals to the accepted opportunity
profiles before fitting. Report unjoined terminal participants/appearance gaps,
zero appearances, non-PA counts and all excluded/unknown observations explicitly.
For training use the pinned lineup audit's structurally supported candidate games
(the known four suspended-game exclusions remain excluded), not a new outcome/
exposure filter. Classify starts by corroborated source starting-pitcher identity;
only known other pitchers for that side are relief. Unknowns cannot become relief
by default. Keep full-source versus training partitions and their conservation.

For each of the fixed12 pitchers, retain start and relief outing count, literal
workload sum and individual outing workloads. No winsorization, injury inference,
minimum sample cutoff, league borrowing or role mixing. Selected role with no
supported outings is a blocker, not zero/neutral fallback. Publish sparse sample
counts; historical mean usage includes tactical/short outings and is not a
physical capacity or measured fatigue estimate. Selected-season reconstruction
uses hindsight, not a pregame forecast. Means are measured only after this fixed
contract; do not retune selection or policy after seeing them.

Publisher: pin DB/reports/query/recipe before read, read-only DuckDB attachments,
`-init /dev/null -bail -batch -csv`, disable external access, SELECT stdout/no COPY,
retain native stdout/stderr, reread pins before exclusive publication. Refuse
existing evidence and repository outputs; failed validation/drift cannot publish.
Actual repeats/native independent reconciliation must agree exactly. Closed-cast
export binds the accepted20 starters and10 recorded nonstarters,30 rating-role
vectors (18 batters/12 pitchers), names and both workload-role summaries. Reuse
fixed200 Dirichlet/log5 ability formulas, not a second fitted probability model.
Stage/fsync/exclusive export publication and final input reread; source hashes
are provenance checksums, not authentication of arbitrary hand edits.

## Deterministic manager policy — fixed before means

Version the manager separately from outcome generation and canonical engine.
For the starting pitcher select his **start** role mean; for each bench pitcher
select his **relief** role mean. With workload sum S and positive outing count G,
threshold is `max(1, ceil(S/G))`, calculated using checked integer arithmetic.
One completed PA minimum follows 2019 MLB Rule5.10(f)/(g), not the National
Association-only three-batter provision (retained rulebook printed53/PDF65).
No simulated pitch counter, injury, performance hook or outcome-rate degradation.

Count only modeled completed PAs against the current defensive pitcher. Before
his next PA, if his counter reached threshold and an unused modeled reliever
exists, emit a canonical `Substitution`/`PersonnelChange::Pitcher` transition.
Decide before the incoming pitcher's first PA: between innings if the threshold
was reached at the third out, otherwise between PAs within the half. Do not
replace a pitcher while his team bats or after game over. Each incoming pitcher
faces at least one completed PA; no mid-PA changes or fabricated exceptions.

Order unused relievers by **descending exact rational relief mean**, then ascending
literal player ID for exact ties. This is a long-to-short usage heuristic, not a
skill/leverage ranking or historical entry order. A player is used at most once,
removed pitchers cannot return. When the finite recorded cast is exhausted,
**the last pitcher stays beyond his target**, visibly disclosed; do not invent
another player, loop/reenter, fabricate a winner, or claim calibrated workload.
This exhaustion rule is a bounded exhibition limitation, not realistic season
availability. No rest-day or full-season manager in this milestone.

Substitutions consume zero RNG words; every PA still consumes exactly one
SplitMix64 outcome word. Preserve the ability prior/formulas, pooled contact
conditionals, positional actors, current hitter/opposing pitcher lookup and
canonical legal before/after state. Non-PA substitutions preserve batting cursor,
PA/BF counters, outs/score/bases and carried runner responsibility. Bench/removed/
DH changes go through canonical engine validation; do not patch GameState around
`apply`, transfer inherited runs to a reliever, or weaken the old closed-roster
validator. Add an explicit closed-cast mode; old loaders/modes remain strict.

## Observable acceptance before implementation

- Synthetic audit tests: both sides/start-relief separation, role mixing prevented,
  zero versus missing, duplicate/orphan/unknown joins, multiple stints counted once,
  suspended/source-training exclusions, conserved terminal totals including IBB/
  interference/sacrifices, non-PA/no-play exclusions, shuffled input/nonmutation,
  native read-only projection, provenance/exclusive publication/final drift.
- Export/loader tests: exact modeled cast/role/name/workload binding, raw sums/G,
  no exposure borrowing, zero means/positive minimum, integer/bool/control/duplicate/
  overflow/unknown-field guards, provenance, final reread and atomic refusal.
- Native independent checks: ceil/minimum and exact rational ordering, separate
  starter/relief limits, no hook before limit, middle-of-half and next-defensive-half
  hooks, no offensive/postterminal hook, zero extra draws, no reentry/exhaustion
  disclosure, closed-cast/DH/current-batter/inherited-charge/substitution credits,
  continuity/codec/available-stat conservation and all13 contact/outcome policies.
- Fixed seeds0..1999: repeated managed games and paired unchanged rated games,
  termination, legal state/continuity/codec/accounting, each removed pitcher's PA
  count equals its limit, early-game-end and final-exhausted-pitcher exceptions
  explicitly distinguished. No seed/count/resource-ceiling retuning.
- Actual30 role vectors/12 workload summaries/limits/order independently reconcile;
  verify all108 hitter/opposing-cast probability vectors/CDF boundaries using the
  unchanged formula and one-word sampling. Preregistered fixed six-sigma-plus-one
  draw diagnostic:100,000 seed19 words for each matchup; descriptive, not accuracy.
- Native truthful disclosures: modeled availability, workload basis/S/G/limit/order,
  sparse samples, exhaustion, no pitch/fatigue measurement; substitution calls name
  outgoing/incoming, active pitcher/workload visible during playback. Native player
  probabilities remain Rust-owned; label initial matchup tables as initial or show
  validated active native values, never misrepresent starter probabilities as live
  relief probabilities. Escape metadata; preserve unavailable accounting labels.
- Startup-configured `model=managed` with a local cast file, alongside old modes;
  no query paths/new URL file access, no-query default unchanged, visible500 on
  malformed inputs/no fallback. Old CLI errors, records, wire and static/live pages
  stay byte-exact. New opt-in/native presentation tests precede behavior.
- Full offline locked Rust build/tests, fmt, strict all-target Clippy `-D warnings`,
  **`python3 -Werror -m unittest discover -s tests -v`**, compilation and every
  applicable established source/native/preview check. Freeze tested executable
  sources during full verification; preserve complete failures/logs/failing seeds.
- Browser seeds1/2/3/99: independent native/frame/substitution/active-workload/panel/
  final-stat parity, selector/Generate/New/large-seed wrap/reset/navigation/autoplay/
  pause/manual/visibility/terminal stop/log scroll; inspect screenshots and cleanup
  only owned resources. No witness/seed search or claims from unchecked clicks.
- Complete actual full-index scope/diff inspection and exact base/main/work/index/
  frozen/tested/non-owned/input/original-prefix equality; fresh prose links outside
  fences, bounded parent/author acceptance, accepted/precommit/committed diff/tree
  equality and clean verified local checkpoint/no push. Not independent review.

Stop with a runnable testing recipe after this bounded milestone. Production
P-02, G1/G2, statistical calibration, historical eligibility, full season management
and public redistribution rights remain open. No fixed-sixth-inning hook will be
implemented.

[Live status](../PROGRESS.md) · [Source cast audit](../data-audits/2019-bullpen-readiness.md)

## 2026-10-13 priority amendment — D-20

After asking whether managerial work meant full simulation fidelity was already
complete, maintainer explicitly trusts implementation judgment but states engine
accuracy/solidity is the most important requirement. It is not complete. Finish
the in-flight read-only workload foundation, then establish a test-backed engine/
generator/model gap checkpoint before managerial integration. Legal/reproducible
transitions and mechanically correct probability sampling do not certify realistic
advancement, generated steals, complete outcomes or calibrated matchups. Prioritize
common-game fidelity and preregistered mechanism/accuracy evidence over additional
management/UI sophistication. Select any corrective implementation as a separate
bounded work package; no new transfer/dependency/delegation or gate waiver inferred.
The original source filters, zero/missing rules, role means, ceiling/order/exhaustion
policy and acceptance prefix remain unchanged; no post-measurement retuning.

## 2026-10-13 foundation checkpoint and reproduction

D-21 now authorizes continuing without routine testing pauses until explicit owner
input is needed. It does not authorize managerial integration before D-20 fidelity,
additional source transfers, production calibration, release or gate signoff.
The foundation is separately testable before those later steps. Live verification
and acceptance remain exclusively in [PROGRESS](../PROGRESS.md).

[Audit CLI](../../audits/pitcher_workload.py), [native grouped query](../../audits/2019-pitcher-workload.sql)
and [synthetic/native regression tests](../../tests/test_pitcher_workload.py) retain
full-source versus2,425-candidate training partitions,485 selected outing rows,
all24 start/relief summaries and explicit unmatched/unknown findings. All selected
roles have support, but relief samples range from3 to72 outings; a measured mean
is not measured fatigue or true physical capacity. Three source outings excluded
from training account for30 literal participant PAs. Multiple appearances remain
one outing with literal rows/ranges retained. Full native/outcome/profile validation
precedes means. An independent SELECT based on batting PA flags and separate mound
appearance grouping reconciles every selected outing and role summary; it does
not import the production audit. Reports repeat exactly (413,830 bytes,
SHA256 `125768bd82ce54ead277d92301fbb7434e823a8fc4432777087c15b8e46ad2f9`).

The initial actual run correctly failed closed when SQL NULL base identities were
serialized as literal `NULL`. Canonical native CSV requires **`-nullvalue ''`**;
fix the command, not the source or validator. Real literal `NULL` identities are
not blanked or postprocessed. Original failure/export/SQL evidence remains local.
A separate independent-helper parser failure used reserved CTE name `cast`;
renaming the helper CTE preserved source filters and original failed artifacts.
Neither failure published a report or changed selection/policy.

```sh
python3 -Werror -m audits.pitcher_workload \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  /tmp/bbsim-outcome-details/2019-outcomes.duckdb \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-lineup-readiness/report-first.json \
  /tmp/bbsim-bullpen-readiness/report-first.json \
  NEW_EXTERNAL_WORKLOAD_REPORT.json
python3 -Werror -m preview.pitching_cast \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-lineup-readiness/report-first.json \
  /tmp/bbsim-bullpen-readiness/report-first.json \
  NEW_EXTERNAL_WORKLOAD_REPORT.json \
  NEW_EXTERNAL_CAST.tsv
python3 -Werror -m unittest discover -s tests -p 'test_pitch*.py' -v
```

Replace `NEW_EXTERNAL_*` with new paths outside the repository. Audit output and
all native evidence siblings must not exist; cast export refuses existing files,
stages/fsyncs, rereads all four input hashes, then publishes an exclusive hard link.
No source download is performed. `/tmp` caches/evidence may expire; the recipe
requires the five accepted pinned inputs, not an arbitrary substitute dataset.
All30 ability vectors retain original kappa200/model ownership. No probability
fitting, engine modification, manager, browser outcomes or new playable mode is
introduced by these commands.

[Exporter](../../preview/pitching_cast.py) and [synthetic tests](../../tests/test_pitching_cast.py)
produce sorted `key\tvalue` TSV fields with `format=closed-pitching-cast-v1`:

- Existing20 starting ability vectors plus ten recorded nonstarter pitcher vectors;
  each `batter:ID`/`pitcher:ID` stores nine integer category counts in the unchanged
  rating order, not Python-computed probabilities.
- `workload:start:ID` and `workload:relief:ID` store literal `S,G`; `0,0` is missing
  support and differs from supported zero `0,G`. Selected-role missing support
  blocks export. Each summary independently agrees with retained outing rows.
- `cast:SIDE:RANK:ID` stores the literal name, rank0 starter, then relief candidates
  descending exact rational relief means and ascending ID ties. No floating-point
  ordering, role mixing, exposure cutoff or historical-entry-order inference.
- Fixed provenance/model/prior/count metadata, separate `policy=pa-mean-closed-cast-v1`,
  workload basis, `exhaustion=last_pitcher_stays`, and explicit
  `availability_basis=recorded nonstarters assumed available at start; not verified eligibility`.

Actual exclusive exports repeat (3,577 bytes,
SHA256 `d284339066222212dc79c57b284bb764d49a156937b2b6762ac943a7ccec2bec`).
All30 source vectors/24 raw summaries/12 names reconcile; integer cross-product
ordering is separately checked against the export. Cast artifacts remain local;
the old strict20-role ratings loader intentionally cannot load this new format.
A cast export is **not** a manager implementation or engine-fidelity signoff.

### Bounded foundation acceptance scope

D-20 deliberately splits the in-flight source foundation from later manager work.
The current checkpoint changes only eight of the original20 owned paths:
`docs/PROGRESS.md`, `docs/data-audits/2019-bullpen-readiness.md`, this contract,
`audits/2019-pitcher-workload.sql`, `audits/pitcher_workload.py`,
`tests/test_pitcher_workload.py`, `preview/pitching_cast.py` and
`tests/test_pitching_cast.py`. Original20-path authorization and180 non-owned
pins remain; all ten existing authorized-but-untouched paths also equal HEAD.
The two future manager paths are not created. Acceptance verifies the exact
current eight-path diff,122 frozen source hashes, all accepted/source/artifact
pins and original preregistration/supersession prefixes before a local coherent
foundation checkpoint. F-01-workload completion applies only to this source/export
foundation; P-02-managed-preview stays planned behind D-20 fidelity, without any
manager/native probability/UI/gate completion inferred. No push or independent
review is authorized or claimed.
