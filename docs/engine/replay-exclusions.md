# E-03 — 2019 historical replay/golden exclusions

This is a **diagnostic candidate audit**, not a historical replay importer or a source-completeness certificate. The internal first deliverable is a head-to-head exhibition, not exact date-correct season playback. No ratings/calibration sample filter is changed by this audit.

## Policy and necessary conditions

2019 MLB Rules 7.01(a),(b),(g), 7.02 and 7.03 distinguish ordinary regulation/extra-inning endings from called, suspended, tied and forfeited games. Primary [rulebook provenance](2019-rules.md) is unchanged. Before historical replay, exclude special called/suspended/tied/forfeit games unless their handling is separately implemented and verified. Postponed/no-game schedule entries are not played games and must never become fabricated zero-stat games.

The [read-only SQL](../../audits/2019-replay-exclusions.sql) classifies selected 2019 AL/NL regular-season **game-key groups** using original result facts, not derived nine-inning hints. Its mutually exclusive primary diagnostic reasons are ordered:

1. `unknown_metadata`: empty/null identity, duplicate game-key or result grain (including a same-key game row outside the selection), missing result, or null original forfeit/suspension/tie flag, duration or either score.
2. `forfeit`, then `suspended`, then `tied`: respective source flags.
3. `shortened`: fewer than **51 total outs**, the ordinary nine-inning minimum (27 visiting offensive outs plus 24 home offensive outs). A normal 8½-inning home win is **not** excluded.
4. `unsupported_finish`: no decisive score or a duration/winner combination outside the current engine's terminal shapes. Visiting wins need a completed bottom inning (`outs % 6 == 0`); home wins use an uncompleted bottom inning or completed visiting ninth (`outs % 6` in `3,4,5`). Ordinary walk-offs and supported extra innings remain candidates.
5. `candidate`: passes these necessary metadata conditions only.

Multiple flags receive one primary reason, not multiple counted exclusions. `MIN` projections of result fields are never trusted when result grain is not exactly one. Source model `bc/models/intermediate/game_level/game_results.sql` at pinned commit `b449f7bec49f852c27803e096303a4c78ac661b9` defaults **unknown outs to nine innings** (`COALESCE(..., TRUE)`) and unknown shortened/extra status to false; this audit intentionally does not use those hints.

**Remaining boundary:** source summary flags do not independently identify every called-game circumstance, and false registry flags are not a full provenance/completeness review. A candidate needs a source-supported, contiguous canonical event log with a supported terminal transition before replay/golden certification. Existing unsupported appeal/fourth-out/end-game ordering, lineup/count and scorer cases can still reject a candidate. Duration/winner arithmetic alone does not prove a legal ending, roster, runner responsibility, complete credits or ER reconstruction. There is no fallback to treating an unknown duration as normal, and no automatic promotion of these rows into a playable cache.

## Observed offline result

On the unchanged F-03 snapshot:

| Primary category | Source game-key groups |
| --- | ---: |
| Selected | 2,429 |
| Candidate only | 2,415 |
| Shortened | 10 |
| Suspended | 4 |
| Forfeit / tied / unknown metadata / unsupported finish | 0 each |

Thus **14 groups are excluded by this audit**, without claiming the remaining 2,415 are fully reconstructable. Counts refer to this snapshot and this necessary-condition policy, not all possible 2019 source rights or accuracy.

## Reproduction and verification

```sh
duckdb -readonly /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb -csv < audits/2019-replay-exclusions.sql > /tmp/bbsim-e03-replay-exclusions.csv
duckdb -readonly /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb -csv < audits/2019-replay-exclusions.sql > /tmp/bbsim-e03-replay-exclusions-second.csv
cmp /tmp/bbsim-e03-replay-exclusions.csv /tmp/bbsim-e03-replay-exclusions-second.csv
python3 -m unittest discover -s tests -v
# Same decision body, native DuckDB BOOLEAN/UTINYINT types, synthetic data only:
python3 -c "import runpy; from pathlib import Path; h=runpy.run_path('tests/test_e03_replay_exclusions.py'); Path('/tmp/bbsim-e03-replay-native-property.sql').write_text(h['native_property_sql']())"
duckdb :memory: -csv < /tmp/bbsim-e03-replay-native-property.sql > /tmp/bbsim-e03-replay-native-property.csv
python3 -c "import runpy; h=runpy.run_path('tests/test_e03_replay_exclusions.py'); print(h['verify_native_property_csv']('/tmp/bbsim-e03-replay-native-property.csv'))"
```

The [test harness](../../tests/test_e03_replay_exclusions.py) also evaluates the identical decision body with standard-library SQLite (omitting only the DuckDB session network guard); it is **not a replacement** for the required native DuckDB runs. Cartesian nullable metadata cases are checked **individually**, not just through aggregate category counts. Eleven tests cover unknown duration despite a true derived nine-inning hint, unknown flags/scores, source selection, ordinary/extra endpoints, special/short/incomplete endings, empty/null IDs, missing/duplicate grains, primary reason precedence, out-of-selection same-key ambiguity and deterministic database cleanup. The synthetic corpus contains **1,350 cases**.

Both native read-only audit runs matched byte-for-byte; CSV SHA-256 `30eb42d0be83a7fa5cf13a2b3c41c85f020b3e6c58b1011d0942b6c86190b9e9`. Snapshot before/after SHA-256 stayed `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`. No network, snapshot mutation or source rows in Git.

Initial red `/tmp/bbsim-e03-replay-exclusions-red.log` records missing query; focused green `/tmp/bbsim-e03-replay-exclusions-green.log` records the initial nine tests. Early full `proc_3748` and native-property `proc_146b` passed **99 Rust / 57 Python** and real boxes **27/27**, **54/54**, but exposed `ResourceWarning`s: SQLite's transaction context manager does not close its connection. Parent added an explicit close regression (red `/tmp/bbsim-e03-replay-resource-red.log`), switched to standard-library `closing`, and focused **10/10** now pass (`/tmp/bbsim-e03-replay-resource-green.log`). Raw early logs remain unchanged under `/var/folders/d0/z9jph2ld4v9gw45bwg0f1j900000gn/T/pi-processes-DcroxW/`.

Durable-helper `proc_f882` passed **99 Rust / 58 Python**, native **1,350/1,350**, fmt/Clippy/diff and real **27/27**, **54/54**, with no SQLite warnings. Parent then inspected selection/grain ordering and reproduced a second defect: a same-key source game row outside the chosen year/regular-season/leagues was filtered before counting, so an ambiguous key became a candidate. Four subcases failed red (`/tmp/bbsim-e03-replay-grain-red.log`); separate global key-grain counting fixes them without promoting outside-selection games. Focused **11/11** passed (`/tmp/bbsim-e03-replay-grain-green.log`). Final post-grain `proc_7464` passed repeated read-only audit/output and snapshot `cmp`, native **1,350/1,350**, **99 Rust / 59 Python**, fmt/Clippy/diff and real **27/27**, **54/54**. Raw logs use the same directory above, `proc_7464-{stdout,stderr}.log`. Fresh Sol-high reviewer `6e90597a-f6cf-4aa2-ad75-40d207720d8b`, workflow `05f02b6d-c2ae-409e-8a05-a91d69c3ca01`, returned **OK with notes; no issues found**. Review was read-only and did not independently recount data. Global key grain is limited to the local snapshot; modulo-two endpoints and native missing/duplicate/null-key structural fixtures remain optional missing proof, not defects. Maintainer paused for laptop shutdown after review, then explicitly resumed. Fresh post-resume `proc_9231` passed the same full Rust/Python/real comparison checks (**99 / 59 / 27 / 54**), native **1,350/1,350**, repeated read-only audit matching prior output and unchanged snapshot hash. Raw logs use the directory above, `proc_9231-{stdout,stderr}.log`. Implementation is unchanged after review; this checkpoint is ready for commit. E-03 remains active and G1 pending.
