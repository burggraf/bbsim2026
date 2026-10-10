# E-03 — scorer evidence and responsibility inventory

2026-10-10; read-only inventory of the pinned **F-03 offline 2019 snapshot**, not a new season extract or independent accuracy check. Snapshot SHA-256 `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`; source/model provenance and retention limits are in [F-03](../data-audits/2019-filtered-spike.md) and [F-04](../data-audits/2019-go-no-go.md). No rows are committed.

## Reproduction and observed aggregates

```sh
duckdb -readonly /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb -csv < audits/2019-scorer-coverage.sql > /tmp/bbsim-e03-scorer-coverage.csv
duckdb -readonly /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb -csv < audits/2019-scorer-coverage.sql > /tmp/bbsim-e03-scorer-coverage-second.csv
cmp /tmp/bbsim-e03-scorer-coverage.csv /tmp/bbsim-e03-scorer-coverage-second.csv
shasum -a 256 /tmp/bbsim-e03-scorer-coverage.csv /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb
```

Both runs exited 0 and matched byte-for-byte; aggregate CSV SHA-256 `9ae2a99ae2ca6710dca00cbf5409fc0fc43272c24f06383bbd6dd4e2c9f5136a`. The [query](../../audits/2019-scorer-coverage.sql) disables external access and outputs only aggregates.

| Metric | Observed count/sum |
| --- | ---: |
| Runner-event rows | 182,718 |
| Runner run credits | 23,467 |
| Scoring rows missing a charge-event join | 0 |
| Scoring charge/reached-event game mismatches | 0 / 0 |
| Scoring rows with an explicit charged-pitcher override | 0 |
| Scoring rows whose charge-event differs from original reach-event | 516 |
| Scoring rows charged to a prior pitcher | 2,274 |
| Runner error advances / error-avoided outs | 1,045 / 77 |
| Non-no-play events with missing team-unearned count | 0 |
| Sum of source `team_unearned_runs` | 54 |
| Sum of fielding incomplete-event / unknown-putout flags | 0 / 0 |
| Forfeit / suspension / tie / shortened game flags | 0 / 4 / 0 / 10 |

These are source-derived fields, not a completeness certificate. The 516 differing charge/reach keys are **not** asserted to all be fielder's-choice replacements or cross-pitcher transfers; this inventory does not classify their causes. Zero explicit overrides does not mean zero inherited responsibility.

## Available facts versus missing proof

The local `events` table includes `walk_responsible_pitcher_id`, and `running` includes `reached_on_event_key`, `charge_event_key`, `explicit_charged_pitcher_id`, error-advance and error-avoided-out counts. This supports checking responsibility against a charge lineage instead of blindly assigning every new batter-runner to the current pitcher. Pinned upstream model `bc/models/intermediate/event_level/event_run_assignment_stats.sql` resolves charge from the charge-event pitcher (or explicit override). Its `team_unearned_runs` aggregation joins an event-level team count onto scoring-runner rows; **do not treat that value as per-pitcher earned-run reconstruction**. Upstream `doc_stat_list.md` defines this field specifically as runs earned to an individual pitcher but unearned to the team after prior missed chances, not all unearned runs.

Pinned source model `bc/models/intermediate/event_level/event_fielding_stats.sql` uses `COALESCE(..., 0)` for missing fielding aggregates. Its zero unknown/incomplete counts therefore do **not** establish that every fielding chance—including muffed foul flies—was reviewed. No errorless counterfactual runner movement or complete `NoMisplay` review field is present in the audited offline tables. The source repository also defines `stg_game_earned_runs` (game/pitcher totals), but that table is not in this snapshot: its 2019 availability was unverified at the offline-inventory stage, and independent reconciliation remains pending. Historical per-pitcher ER must stay unknown until separately supported. This is a limit of this inventory, not a claim that all upstream raw data lacks scorer information.

A subsequent read-only source availability probe (`/tmp/bbsim-e03-earned-totals.sql`, process `proc_2bc0`) found 21,431 game/pitcher ER rows for all 2,429 selected 2019 games, zero null ER, sum 21,738, with 27 rows in the three F-02 games. Cached catalog metadata lists one 4,750,238-byte table file. These are published **game totals**, not counterfactual play facts or yet independent comparison evidence; supplemental box reconciliation remains pending, and the unchanged F-03 offline snapshot still omits the table.

Source code was read at the F-01 pinned upstream commit `b449f7bec49f852c27803e096303a4c78ac661b9`; field meanings are documented in `bc/models/doc_global_cols.md`, `staging/event/stg_event_baserunners.sql` and the intermediate baserunning/fielding/run-assignment models. Follow the [source repository](https://github.com/droher/baseball.computer/tree/b449f7bec49f852c27803e096303a4c78ac661b9/bc/models) to inspect those definitions, not these aggregates as instructions.

## Bounded engine contract and acceptance

2019 Rule **9.16(g), example (2)** explicitly charges a batter-runner who replaces a forced-out inherited runner to the prior pitcher. Before this checkpoint the engine forced Plate-origin responsibility to the current pitcher, so that supported case could not be represented. The new typed fielder's-choice PA supports a deliberately bounded **one occupied runner, runner out, batter safely to first, one out, no runs** shape. The batter carries the retired runner's responsibility; no vector-order inference or independent fielding roll occurs. Multi-runner replacements, tag-vs-force cause, charge chains, mid-PA count exceptions and full ER reconstruction remain separate work; unrepresented subtype shapes must return `UnsupportedRule`, not fabricated charges.

Acceptance: replacement from each occupied base and each 0–2 prior-out state; original pitcher rather than reliever charged when the replacement later scores; repeated replacement preserves charge; third-out clearing preserves non-scoring/PA accounting; forged responsibility, FC hit/walk credit or contradictory caught-stealing retirement is rejected by reducer, validator and codec; unsupported multi-runner shapes rejected; existing hit-plus-runner-out remains a non-FC play. Full applicable Rust/Python checks and fresh substantive review precede commit. E-03 remains active and G1 pending.
