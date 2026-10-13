# F-01: offline 2019 real-team/lineup readiness audit

## Authorization and preregistered scope

Maintainer chose “Audit real-team/lineup readiness using existing 2019 data” in
interview `f0658376-806f-4713-b418-eb071b945bc2`, then explicitly directed proceeding
with the preserved answer despite timeout. This is a read-only F-01 follow-up for
one future real matchup, not authorization for a reskin, player fitting, a new
importer, season replay, UI restart, source transfer or release. No dependencies,
delegation, worktrees, engine changes or gate waiver.

Exactly five owned paths: this report, `docs/PROGRESS.md`,
`audits/2019-lineup-readiness.sql`, `audits/lineup_readiness.py`,
`tests/test_lineup_readiness.py`. Base `156ba0f`; all other tracked paths remain
byte-identical. Artifacts/data/logs stay outside Git in
`/tmp/bbsim-lineup-readiness/`. No cohort/coverage measurement precedes this
contract; schema inspection and existing accepted reports are prerequisites.

Accepted inputs only:

- [F-03 snapshot](2019-filtered-spike.md), 22,818,816 bytes, SHA256
  `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`.
  Source catalog `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583`;
  `f03-spike-1` / `2019-filtered-extract-v1`, source selection 2019 regular-season
  AL/NL games. It contains names, starting lineup/fielding maps, starter IDs,
  parks, results and people, but not a transaction/active-roster table.
- [Accepted supplement](2019-outcome-details.md), 17,575,936 bytes, SHA256
  `554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82`;
  snapshot857/source `b449f7bec49f852c27803e096303a4c78ac661b9`.
  `stg_game_fielding_appearances` supplies game/player/side/position/event ranges,
  not verified pregame eligibility. Preserve position0 and reversed terminal
  ranges; do not manufacture a bench or infer availability between appearances.

The command must pin both DB hashes before querying, use the installed DuckDB
CLI read-only (local second attachment read-only), disable external access before
SELECTs, preserve complete raw stdout/stderr externally, and recheck DB/query/
recipe hashes before exclusive output publication. No overwrite or silent
partial publication; no remote attachment/extension install/COPY. The SQL emits
literal projected game/person/appearance/manifest records with deterministic
ordering, not outcomes or fitted probabilities. Source duplicate grains cannot
be silently collapsed. Unknown metadata produces explicit coverage gaps.

## Fixed structural checks and interpretation

Selection is exactly F-03's 2019 regular-season AL/NL cohort. Check global game
and person lookup grains. Report all selected games/sides and denominators;
source IDs/maps/names are recorded source fields, not independent proof of
historical legality or official scoring. Preserve raw handedness labels and
count missing/Unknown labels; do not infer effective matchup hands from bios.

A **structural starting-lineup candidate** requires:

1. Nonempty game/team IDs, distinct teams, known date/boolean DH, source team
   names and `bat_first_side='Away'`; one matching result with non-NULL false
   forfeit/suspension/tie flags. No score/winner or final-duration criterion:
   this is a starting-lineup exhibition input, not replay-log eligibility.
2. Both sides have nonempty, distinct player IDs in their source maps, exactly
   lineup keys1..9 without DH or0..9 with DH, and fielding keys1..9 without DH
   or1..10 with DH. Slot0 is the separate DH-game pitcher; position10 is DH.
   These are source conventions to test, not positions assigned by batting order.
3. Fielding position1 equals the explicit starting pitcher. No-DH batting IDs
   equal the nine fielding IDs and include that pitcher exactly once. With DH,
   lineup slot0 equals that pitcher; batting IDs1..9 exclude him, equal fielding
   positions2..10, and the position10 DH occurs at exactly one batting slot.
4. Each starting ID has exactly one person lookup with nonblank first/last name;
   starting identities are disjoint between teams. Missing/ambiguous names stay
   unknown; do not fabricate names or borrow another player's lookup.

Collect **all** failed conditions per side/game (not a lossy priority bucket),
including unknowns. Separate source membership/lookup support from structural
compatibility. No appearance, role-exposure, rate or handedness cutoff gates the
candidate selection. Strict predicates are frozen before inspecting counts;
if a source convention differs, report failure and resolve the contract explicitly,
not retune until a desired game qualifies.

Choose the primary witness by ascending `(date, game_id)` among structural
candidates; additionally show the first DH and first no-DH witnesses by the same
ordering if present. No choice by score, favorite team, rate, player exposure or
model fit. Each witness lists literal team names/IDs/date/park, ordered batting
slots, explicit pitcher, positional fielding map, DH batting slot (engine
zero-based conversion labelled), lookup names/raw hands and current unsupported
capabilities. No generated game or modified Rust producer in this audit.

For appearance-based membership, count literal distinct `(team_id,player_id)`
pairs and observed game counts/first-last dates/positions; retain event ranges
and unknown positions as source facts. A player appearing later in a game is
not known to be eligible on its opening bench. The interval between first/last
appearances is **not** an active-roster interval; absence is not an injury and
multiple observed teams are not dated transaction records. Report how many
appearance IDs lack local bios and how many starting sides lack corroborating
same-game/side/position/player appearance records. Corroboration is shared-source
consistency, not independent historical proof, and never promotes an inferred
roster into a verified one. Do not reject the accepted terminal reversed range.

## Acceptance evidence required

- Test-first synthetic DH/no-DH cases; wrong/missing keys and pitcher/DH/defense
  mismatches, duplicate/cross-team identities, missing/ambiguous bios, unknown
  metadata, duplicate games/results/JSON keys, shuffled deterministic inputs,
  earliest witness choice, unknown position/reversed ranges and membership-gap
  controls. Positive controls must remain green; missing interfaces are not
  semantic reds.
- Real CLI provenance/read-only/no-overwrite/input-stability controls; complete
  native export retained. Independent read-only native SQL reconciles all
  candidate game IDs, every witness slot/name/position/pitcher/DH link, lookup
  coverage and membership counts/windows; repeated report/export bytes exact.
- Freeze tested sources before full checks. Run
  `python3 -Werror -m unittest discover -s tests -v`, full offline-locked Rust
  tests/build/fmt/strict all-target Clippy, existing audit/native/preview suites
  and frozen-input compatibility checks. Historical passes do not certify this
  new audit. No warning suppression or replacement smoke suite.
- Entire staged diff/retained logs inspected; exact five-path ownership,
  working/index/frozen equality, non-owned HEAD equality, fresh prose links,
  both DB hashes and preregistration preserved. Parent/author acceptance only;
  no independent-review claim. Commit locally only after verification; no push.

## Readiness decision (pending evidence)

The audit has not yet measured or certified a matchup. Even a structurally
supported witness does not supply ratings, a legal autonomous bench/bullpen,
date-appropriate availability, effective hands/park effects or full official
box accounting. The current preview still uses fictional identities, equal
pooled rates and ordinal synthetic fielders, **not actual positions**; swapping
names is insufficient. Production P-01/P-02/P-03, E-03/G1/G2 and rights remain open.

[Execution contract](../EXECUTION.md) · [live ledger](../PROGRESS.md) ·
[player-exposure boundary](../modeling/player-opportunity-support.md)

## Premeasurement source-semantic check (2026-10-13)

The cached source `doc_global_cols.md` describes `lineup_position=10` for the
separate DH pitcher, but that is not a promise about normalized game-start map
keys. A read-only schema/convention probe of the already accepted fixture
`ANA201904040` returned lineup keys0..9 and fielding keys1..10. The original
slot0 map predicate remains unchanged; the whole cohort still requires testing.
The same source description recognizes appearance positions11/12 as pinch
hitter/runner. Before any cohort counts, a regression exposed the new audit's
accidental ten-position cap; remove that cap, preserve the full UTINYINT domain,
and label0/NULL/undocumented13..255 unknown. Do not turn11/12 into fielders or
opening-bench certificates. These checks refine raw-field handling, not candidate
filters or measured thresholds. Original preregistration retained byte-for-byte
externally; this dated note and corrected sources are frozen before measurements.

## Measured source support

Frozen native runs `proc_f0a5` and independent reconciliation `proc_d5ca`:

| Check | Observed support / denominator | Interpretation |
| --- | ---: | --- |
| Starting-side structure and named identity lookups | 4,858 / 4,858 sides | Correct source map keys/uniqueness/pitcher/DH/defense relationships; no side-level failures. Shared-source support, not independently certified historical eligibility. |
| Named starting identities and same-game/side/position appearances | 46,150 / 46,150 entries | Includes separate DH pitchers and DH records; neither batting order nor position10 is an actual ninth/tenth defender. |
| Missing/ambiguous starting lookups | 0 / 46,150 entries | Exact single local first/last-name lookup; names are current source bio labels, not guaranteed as-of-2019 spellings. |
| Missing/Unknown raw bio hand labels | 0 / 92,300 starting bio hand slots | Literal labels, not effective matchup sides or fitted handedness effects. |
| Structural starting-lineup candidates | 2,425 / 2,429 games | Four source-suspended results excluded by the fixed flags, not poor lineups. Not a complete-game replay eligibility count. |
| Supplied appearance rows with local named lookup | 69,613 / 69,613 rows | Fielding/batting-role appearance records, not all season roster or PA participants. |
| Unknown position/side/date appearance records | 0 / 69,613 rows | Source records only; no active-roster interval inferred. |

Four `result_metadata` exclusions: `BOS201908070`, `CHA201905270`,
`DET201905190`, `NYN201906130`; each has `suspension_flag=true`, not missing maps.
No extra duration/winner/score filter was introduced. This differs intentionally
from the older 2,415 necessary replay candidates: starting-lineup exhibitions and
complete historical terminal logs have different requirements.

Appearance records yield **1,559 team-player pairs / 1,405 players** and
**145 players observed with multiple teams**. These are literal shared-source
memberships/windows, not a 1,559-person roster or 145 dated transactions. F-03's
1,410 participant-linked bios and F-01's 1,568 season-roster rows have different
populations; none implies a complete eligible opening bench. Supplied positions
are1–10 in this cohort; synthetic11/12/unknown-byte controls remain covered.
The accepted reversed range `ATL201906110 / culbc001 / Home / position7 /
92–91` is retained exactly, not expanded into active time or rejected.

Every native candidate ID, all76 witness identity/name/raw-hand/slot/position
links, lookup coverage and every1,559 membership/date/position profile reconciled
against separate set-based read-only SQL. Both reports/complete native exports
repeat byte-for-byte; native stderr empty, source/query/recipe and174 non-owned
tracked hashes unchanged. These are independent implementations on the same
source, not independent official data.

### Deterministically selected primary matchup

Earliest candidate **`OAK201903200`, 2019-03-20, `TOK01`**:
**Seattle Mariners (`SEA`, Away) at Oakland Athletics (`OAK`, Home)**, DH used.
Explicit pitchers: **Marco Gonzales (`gonzm005`)** and
**Michael Fiers (`fierm001`)**, respectively. Both DH batting slots are4,
converted to engine zero-based index3. Names below are literal first/last bio
labels; positions are source positional IDs, not batting ordinals.

| Slot | Seattle player ID / name / position | Oakland player ID / name / position |
| --- | --- | --- |
| 1 | `gordd002` / Dee Strange-Gordon / 4 | `laurr001` / Ramon Laureano / 8 |
| 2 | `hanim001` / Mitch Haniger / 8 | `chapm001` / Matt Chapman / 5 |
| 3 | `brucj001` / Jay Bruce / 3 | `piscs001` / Stephen Piscotty / 9 |
| 4 | `encae001` / Edwin Encarnacion / 10 (DH) | `davik003` / Khris Davis / 10 (DH) |
| 5 | `santd002` / Domingo Santana / 7 | `pindc001` / Chad Pinder / 7 |
| 6 | `narvo001` / Omar Narvaez / 2 | `olsom001` / Matt Olson / 3 |
| 7 | `healr001` / Ryon Healy / 5 | `profj001` / Jurickson Profar / 4 |
| 8 | `beckt001` / Tim Beckham / 6 | `semim001` / Marcus Semien / 6 |
| 9 | `suzui001` / Ichiro Suzuki / 9 | `hundn001` / Nick Hundley / 2 |

First no-DH control: **`CIN201903280`, 2019-03-28**, Pittsburgh Pirates at
Cincinnati Reds; both pitchers occur exactly once in the source batting order,
with no separate DH pitcher slot. Full literal lineups/names/positions and
checks are in the external report, not a new generated game.

### Reproduction and retained artifacts

```sh
python3 -Werror -m audits.lineup_readiness \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  /tmp/bbsim-outcome-details/2019-outcomes.duckdb \
  /tmp/lineup-readiness-new.json
# A new external filename is required; old reports/native evidence cannot be overwritten.
```

`/tmp/bbsim-lineup-readiness/report-first.json`:1,238,826 bytes,
SHA256 `5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b`.
Complete source projection `report-first.native.csv`:14,283,285 bytes,
SHA256 `bef8108455784113702dd4003172ac99162d24dfb8522da4e2a0c53fa16dbf02`.
Separate `native-independent.csv`:458,150 bytes,
SHA256 `73c7545ca170fc77aca0a58b8dc706d38d21ae984ec1c771ceee4952a32ea425`.
Independent recipe/reconciliation, raw logs and repeat files are in that external
artifact directory; temporary artifacts are not remotely durable.

**Bounded readiness:** recorded, named starting identities and positional/DH
links support preregistering a closed-starting-roster experiment for this matchup.
They do **not** supply a calibrated real-team game. The unchanged Rust preview
initializer is fictional/DH-only and its fielders are ordinals, not source
positions. A later approved implementation must bind identities and positional
actors explicitly, preserve defaults and label pooled probabilities. Bench/
bullpen eligibility, autonomous substitutions, date-appropriate availability,
player ratings/effective hands/park effects and complete accounting remain open.
No UI, Rust, RNG, fitting, download or release change occurred in this audit.

Source attribution retained verbatim in report provenance: The information used
here was obtained free of charge from and is copyrighted by Retrosheet.
Interested parties may contact Retrosheet at "www.retrosheet.org".

## Fresh verification (2026-10-13; staging/acceptance pending)

The original “pending evidence” paragraph above is retained as preregistration
history, not current status. The measured decision is the bounded readiness
statement above; no product/gate/eligibility/accuracy signoff is inferred.

- Test-first interface and semantic reds are retained separately. Final focused
  `proc_4936` passed26 tests; original preregistration prefix and the three tested
  audit sources were frozen before real cohort measurements.
- `proc_f0a5` repeated the real pinned CLI export/report byte-for-byte;
  `proc_d5ca` independently reconciled every2,425 candidate ID,76 witness links
  and1,559 membership profiles. All native stderr files are empty.
- `proc_0805`, `bash /tmp/bbsim-lineup-readiness/full-checks.sh`, exit0/164s:
  **586 Rust / 183 Python tests (26 new Python)**; `cargo fmt --manifest-path
  engine/Cargo.toml --check`, `cargo build --offline --locked --manifest-path
  engine/Cargo.toml --example generated_game`, `cargo clippy --offline --locked
  --manifest-path engine/Cargo.toml --all-targets -- -D warnings`, `cargo test
  --offline --locked --manifest-path engine/Cargo.toml`, `python3 -Werror -m
  unittest discover -s tests -v`, Werror compilation and `git diff --check` pass.
  Complete stdout1,160 lines/71,263 bytes and stderr337 lines/43,506 bytes retained
  and inspected; generation-failure/HTTP500 entries are expected negative tests.
- Full checks also rerun the established69-metric/five-transition/54-cell source
  audits,27 shared-upstream ER and54 external-box comparisons,1,350 native rule
  controls, repeated outcome inventories/19 scalar+17 result controls,
  2,000 default and paired joint games/100,000 draws, and prior1,821 role-profile
  reconciliation. These do not prove independent historical truth or calibration.
- Fresh HEAD/current compatibility:316 BBE25 transitions/304,463 framed bytes,
  pinned SHA/codec roundtrips,11 default CLI cases, complete default **and joint**
  traces for seeds1/2/3/99, repeated outputs and default static/unconfigured live
  pages are byte-identical. Joint native calls/recaps/states remain exact. No
  permitted CALL delta for this audit; no interactive browser/UI change.
- `PYTHONPATH=. python3 -Werror /tmp/bbsim-lineup-readiness/refusal-controls.py`
  passed actual CLI no-overwrite refusal, read-only CREATE refusal on both native
  attachments and all live report/manifest/DB/query/recipe/export provenance pins.
  Complete expected failure stderr retained; attempted statements were rejected,
  both DBs/report/native evidence stayed byte-identical. Synthetic final-input
  drift control proves refusal before report creation; source files stayed frozen.
- Report sizes/hashes, all manually listed primary lineup names/IDs/positions,
  source DH-slot conversions and all four suspension exclusions were checked
  directly against the retained literal export/report. All174 non-owned paths,
  both DBs/prior player report and the three tested audit sources remain unchanged.

Local parent/author staged acceptance and commit equality are recorded in the
[live ledger](../PROGRESS.md), not an independent review. Raw source exports,
reports and logs remain external; reproduce with a new output name. No new
network transfer, fitted ratings, Rust/UI changes, push or release.
