# F-02: 2019 event-transition fixture and source-summary audit

**Status:** four queries reproduced byte-for-byte; F-02 fixture evidence complete, but G0 still pending. This is not an independent box-score validation or G0 signoff. Source: baseball.computer public read-only DuckLake v1, catalog SHA-256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583` (unchanged from [F-01](2019.md) on 2026-10-10 UTC); source-model revision `b449f7bec49f852c27803e096303a4c78ac661b9`. DuckDB CLI 1.5.6. Queries are version 1; hashes below identify exact text.

## Reproduce without raw extracts

Run from repository root. Each query attaches the remote catalog `READ_ONLY`, filters to 2019 AL/NL regular-season games or explicit fixture IDs, and writes only bounded aggregate/detail results under `/tmp`, **not Git**. The queries do not make an offline snapshot or measure network transfer.

```sh
duckdb :memory: -csv -noheader < audits/2019-fixture-discovery.sql > /tmp/bbsim-f02-candidates.csv
duckdb :memory: -csv -noheader < audits/2019-fixture-details.sql > /tmp/bbsim-f02-details.csv
python3 audits/verify_fixture_details.py /tmp/bbsim-f02-details.csv
duckdb :memory: -csv -noheader < audits/2019-fixture-reconcile.sql > /tmp/bbsim-f02-reconcile.csv
python3 audits/verify_fixture_reconcile.py /tmp/bbsim-f02-reconcile.csv
duckdb :memory: -csv -noheader < audits/2019-fixture-diagnostics.sql > /tmp/bbsim-f02-diagnostics.csv
python3 -m unittest discover -s tests -v
shasum -a 256 /tmp/bbsim-f02-*.csv
```

| Query | SQL SHA-256 | Result rows | CSV SHA-256 |
| --- | --- | ---: | --- |
| `2019-fixture-discovery.sql` | `db83285ca757e26862ae90ded0b3131d7b9de388a514372427e40c440cf733ff` | 9 candidate rows | `a625901a854b637a2c0f29b38601a0e972af864ae5f37e3eeddb83ab43c56722` |
| `2019-fixture-details.sql` | `9d1de9430cb6023ca1651e6aef3c118611e624fa5e351e3f9c2bcff14401d100` | 5 event rows | `9bfa50ac7871c50365e3471a6a2503e77b967517432c44d7ce282f67e339fb24` |
| `2019-fixture-reconcile.sql` | `0022e301d06fae28a74b15f1b762a8cc3ce0e7c0ac313c36ce7d5ebf258ae805` | 54 game/team/metric rows | `05f7e5d01207a44624b390207b2e8d718241c4e6deef87168f8501fe5f15be39` |
| `2019-fixture-diagnostics.sql` | `b4d5a0408fa2af9582aa51003d7528fb9de64ef537a98bd5656fa05605a7a1cc` | 6 aggregate rows | `df99d3f209f2ee8d9457494f8485c44c351e26fbea78821676e091d299f457a7` |

## Five bounded transition fixtures

Source base mask: 1=first, 2=second, 4=third. The `after` base/runner slots in `event_base_out_states` are derived by looking at the **next event in the same half-inning**; a frame-ending event has no next base state. All five below have a non-boundary after-state. Scores are ANA home–opponent away where relevant. Names here are stable Retrosheet-style IDs, not human-identifying claims beyond the source record.

| Game/event (key) | Participants and before | Outcome, runner movement and after | Credits |
| --- | --- | --- | --- |
| `ANA201904040` #36 (`1254090036`) | Bottom 3, 2 outs, mask 3: `pujoa001` on 1st, `fletd002` on 2nd; batter `lastt001`, pitcher `volqe001`. | **Non-PA wild pitch**: `pujoa001` 1st→2nd and `fletd002` 2nd→3rd; mask 6, 2 outs, score 2–6 unchanged. | PA absent, 0 runs/outs/fielding credits. |
| `ANA201904040` #96 (`1254090096`) | Bottom 9, 0 outs, empty, batter `bourp001`, pitcher `dowdk001`; score 3–11. | Strikeout plus passed ball; batter `bourp001` reaches 1st, mask 0→1, outs remain 0, score unchanged. | **1 PA, 1 K, 1 passed ball, 0 putouts**; not a separate runner-only play. |
| `ANA201904040` #99 (`1254090099`) | Bottom 9, 1 out, mask 6: `bourj002` 2nd, `bourp001` 3rd; batter `pujoa001`, pitcher `dowdk001`; score 3–11. | In-play out and `bourp001` scores; 2nd runner holds. Mask 6→2, outs 1→2, score **4–11**. | 1 PA, 0 hits, 1 putout and 1 assist, 1 run; simultaneous effects. |
| `ANA201904050` #4 (`1254090259`) | Top 1, 1 out, mask 5: `andre001` 1st, `odorr001` 3rd; batter `mazan001`, pitcher `penaf002`. | Strikeout **and** `andre001` steals 2nd; mask 5→6, outs 1→2, no run. | 1 PA, 1 K, 1 putout, 1 SB. |
| `ANA201904070` #3 (`1254090768`) | Top 1, 1 out, mask 1: `odorr001` 1st; batter `andre001`, pitcher `strac001`. | **Non-PA caught stealing**: `odorr001` out at 2nd, mask 1→0, outs 1→2, no run. | PA absent, 1 CS, 1 putout and 1 assist. |

The bounded details validator checks exactly these five IDs, PA/non-PA credit, outs and score deltas, and occupant identity versus each before/after base mask. The discovery query's first three candidates per category are illustrative; the CS fixture was chosen separately by filtering `stg_event_baserunners.baserunning_play_type LIKE '%CaughtStealing'` in `ANA201904070` and is pinned explicitly in the details/reconciliation queries. It is not a proof of continuity or credit correctness for every 2019 event; F-02 provides fixtures for later E-02/E-03 tests.

## Three-game event totals versus source summaries

The table combines **two teams per game**, each row reporting event-derived `PA, runs, outs, hits, SB, CS, putouts, assists, errors` in that order. `audits/2019-fixture-reconcile.sql` emits the 54 individual comparisons, and the validator rejects missing, duplicate or differing summary/available box values. Runs/outs come from `stg_events`; PA/hits from `event_batting_stats`; SB/CS from `event_baserunning_stats`; fielding credits from `event_fielding_stats`. Compared source summaries are `team_game_offense_stats`, `team_game_fielding_stats`, and `team_game_results` (including `game_results` scores through that model).

| Game | Team (side) | PA | Runs | Outs | Hits | SB | CS | PO | A | E |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ANA201904040` | TEX (Away) | 45 | 11 | 27 | 15 | 0 | 0 | 27 | 11 | 2 |
| `ANA201904040` | ANA (Home) | 42 | 4 | 27 | 8 | 0 | 0 | 27 | 9 | 1 |
| `ANA201904050` | TEX (Away) | 35 | 1 | 24 | 6 | 2 | 0 | 24 | 5 | 0 |
| `ANA201904050` | ANA (Home) | 30 | 3 | 27 | 6 | 0 | 0 | 27 | 7 | 2 |
| `ANA201904070` | TEX (Away) | 36 | 2 | 24 | 6 | 0 | 1 | 24 | 5 | 1 |
| `ANA201904070` | ANA (Home) | 40 | 7 | 27 | 9 | 1 | 0 | 27 | 11 | 0 |

**All 54 event-to-summary cells matched, but this is mostly a same-lineage reconciliation, not independent validation**: team summaries aggregate the event models. The published `stg_box_score_team_batting_lines` and `stg_box_score_team_fielding_lines` have **zero rows for all 2,429 selected 2019 games**, hence zero independent box cells for the fixtures. The validator preserves `NULL` for unavailable box values, rather than converting them to zero. Independent historical box-score cross-checking remains **unproved** and must be addressed before G0 if required.

## F-01 discrepancy diagnoses and residual risk

- Candidate schedule: 2,430 date/AL/NL rows versus 2,429 selected regular-season starts. The schedule includes one row for `2019-07-02 CHA–DET`, postponement `Rain`, with `makeup_dates = '20190927 - Not made up'`. This is a **candidate explanation** for the one-row gap; the schedule lacks `game_id`, and other postponed/moved/doubleheader rows require game-by-game matching before claiming exact schedule reconciliation.
- BIP flags: the 414 differences are **all** `GroundRuleDouble` with null `batted_to_fielder`, `event_batted_ball_stats.balls_in_play=0`, and `event_fielding_stats.plate_appearances_in_field_with_ball_in_play=1`. These have different definitions, not evidence of 414 missing source events: [batted-ball model](https://github.com/droher/baseball.computer/blob/b449f7bec49f852c27803e096303a4c78ac661b9/bc/models/intermediate/event_level/event_batted_ball_stats.sql) uses a nonnull fielder for BIP, while [fielding model](https://github.com/droher/baseball.computer/blob/b449f7bec49f852c27803e096303a4c78ac661b9/bc/models/intermediate/event_level/event_fielding_stats.sql) uses the plate-appearance result's `is_in_play`. Do **not** force the two counts to agree or call a ground-rule double a fielder opportunity.
- No standalone 2019 box-score team lines in this catalog, no independent event/box parity test, no assurance every event credit is historically correct. Source preview caveats, license/attribution review and offline filtered transfer are still F-03/F-04/G0 work. No raw event extracts are committed.

## Checks and failures

Two independent runs of all four SQL files produced byte-identical result CSVs (all SHA-256 values in the table matched; four `cmp` commands exited 0). The public catalog SHA-256 matched before and after the runs. First and repeat runs: `python3 audits/verify_fixture_details.py /tmp/bbsim-f02-details.csv` printed `validated 5 fixture transitions and state invariants`; `python3 audits/verify_fixture_reconcile.py /tmp/bbsim-f02-reconcile.csv` printed `validated 54 fixture metric comparisons; 0 independent box cells present`; `python3 -m unittest discover -s tests -v` passed 18/18; `git diff --check` exited 0. The report's team table was checked against all 54 metric rows. An expected red test run initially failed to import `audits.verify_fixture_reconcile` and later `audits.verify_fixture_details` before those validators existed; both modules and tests now pass. No production/source data mutation, no dataset download or app scaffold.
