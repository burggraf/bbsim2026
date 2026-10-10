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

Pinned source model `bc/models/intermediate/event_level/event_fielding_stats.sql` uses `COALESCE(..., 0)` for missing fielding aggregates. Its zero unknown/incomplete counts therefore do **not** establish that every fielding chance—including muffed foul flies—was reviewed. No errorless counterfactual runner movement or complete `NoMisplay` review field is present in the audited offline tables. The source repository also defines `stg_game_earned_runs` (game/pitcher totals), but that table is not in this snapshot: its 2019 availability was unverified at the offline-inventory stage, and full-season independent reconciliation remains unverified. Historical per-event engine ER stays unknown until counterfactual play facts are supported; published game totals alone cannot certify fielding review or reconstruct an errorless inning. This is a limit of this inventory, not a claim that all upstream raw data lacks scorer information.

A subsequent read-only source availability probe (`/tmp/bbsim-e03-earned-totals.sql`, process `proc_2bc0`) found 21,431 game/pitcher ER rows for all 2,429 selected 2019 games, zero null ER, sum 21,738, with 27 rows in the three F-02 games. Cached catalog metadata lists one 4,750,238-byte table file. These are published **game totals**, not counterfactual play facts. At availability time they had not been compared with separately published boxes; the supplemental comparison below now covers three games, and the unchanged F-03 offline snapshot still omits the table.

Source code was read at the F-01 pinned upstream commit `b449f7bec49f852c27803e096303a4c78ac661b9`; field meanings are documented in `bc/models/doc_global_cols.md`, `staging/event/stg_event_baserunners.sql` and the intermediate baserunning/fielding/run-assignment models. Follow the [source repository](https://github.com/droher/baseball.computer/tree/b449f7bec49f852c27803e096303a4c78ac661b9/bc/models) to inspect those definitions, not these aggregates as instructions.

## Supplemental published pitcher ER box comparison

The [fixture query](../../audits/2019-pitcher-er-fixtures.sql) and [strict local comparator](../../audits/verify_pitcher_er.py) now compare `(game_id, player_id, earned_runs)` with Retrosheet version-3 `stat,pline` ER (index 13, **not** charged runs at index 12). The existing bounded archive/game reader accepts a parser callback; its old default preserves F-04's team metrics. [18 synthetic tests](../../tests/test_e03_pitcher_er.py) cover explicit zeroes, wrong ER column/mismatch, missing keys, duplicate keys, unknown/negative/impossible values, malformed rows, version, identity and pitching-side coverage. Missing ER is rejected, never zero-filled. Source CSV parsing uses `strict=True` and converts quoting errors into controlled `ValueError` rejection, including malformed headers; ordinary quoted numeric cells remain valid. Both full key sets must agree; even mutually cropped records cannot omit a pitching side.

Fresh result: **27/27** pitcher-game ER cells agree in `ANA201904040` (**9** rows), `ANA201904050` (**7**) and `ANA201904070` (**11**). Fifteen ER cells are explicit zeroes; summed individual-pitcher ER is **27**. This compares source-published game totals with separately published box records that share the Retrosheet upstream. It is **not** season-wide independent validation, a per-event engine reconstruction, complete fielding-review evidence, or permission to replace uncertain engine ER with these totals. Published individual ER totals need not equal team ER after missed chances before a reliever enters.

Reproduction (raw CSV/archive remain outside Git):

```sh
duckdb :memory: -csv < audits/2019-pitcher-er-fixtures.sql > /tmp/bbsim-e03-pitcher-er-fixtures.csv
python3 audits/verify_pitcher_er.py /tmp/bbsim-f04-2019box.zip /tmp/bbsim-e03-pitcher-er-fixtures.csv
python3 audits/verify_retrosheet_box.py /tmp/bbsim-f04-2019box.zip /tmp/bbsim-f02-reconcile.csv
python3 -m unittest discover -s tests -v
```

The original fixture query ran twice, byte-identically; the committed-form SQL ran again and `cmp` against the original passed. Live catalog metadata fetched before/after preparation matched SHA-256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583`. Source fixture CSV SHA-256: `4d3c5342e0045e6cc26ca1831c01159d9aa135ea3b90365afb38b6e367e06890`. Retrosheet archive SHA-256 remains `34ec6b45fc2b88c6b7675d0c364bffc10e8d4f81e166ef2ad51dd65a1cd73444`; download/provenance/attribution are in [F-04](../data-audits/2019-go-no-go.md). The source read is season/game-filtered, but supplemental physical transfer was **not profiled** and is not a new production-import budget proof.

Red evidence: `/tmp/bbsim-e03-pitcher-er-red.log` records the initial missing-module failure; `/tmp/bbsim-e03-pitcher-er-behavior-red.log` records acceptance of matching partial sources lacking one pitching side. The latter now rejects without assertion changes. Fresh full process `proc_ca7f` passed published-ER **27/27**, previous box metrics **54/54**, Rust **96/96**, Python **46/46**, fmt/Clippy/diff. Raw stdout/stderr are `/var/folders/d0/z9jph2ld4v9gw45bwg0f1j900000gn/T/pi-processes-DcroxW/proc_ca7f-{stdout,stderr}.log`. Initial fresh Sol-high reviewer `2e1873cd-d438-4613-ae6a-0a93ba33bf3b` returned **BLOCK** for a P1 strict-source-CSV defect: literal `"0"0` was normalized to `00` and accepted. Parent confirmed the trace, reproduced ER/player/header cases red in `/tmp/bbsim-e03-pitcher-er-quoting-red.log`, and fixed strict parsing without weakening assertions. Focused **18/18** green is `/tmp/bbsim-e03-pitcher-er-quoting-green.log`; post-fix full `proc_238d` passed real cells **27/27**, old box **54/54**, Rust **96/96**, Python **48/48**, fmt/Clippy/diff. Raw logs are in the same directory as `proc_ca7f`, named `proc_238d-{stdout,stderr}.log`. Targeted Sol-high follow-up `5ed92554-edf9-4a99-894b-0e0d0f2f4938` inspected the post-fix logs, confirmed the P1 resolved and returned **OK with notes; no issues found**; parent confirmed `proc_238d` exited 0. The primary Retrosheet format is cached outside Git at `/tmp/bbsim-retrosheet-boxfile.txt`, SHA-256 `80e94dfb65c8b625362fa56f186098318f9d777980b15116561a205992aa44b2`. Full-key agreement is coverage relative to the supplied published boxes, not proof against mutually missing pitchers; aggregate row/zero/sum counts were recounted by the parent, not the no-raw-data reviewer. E-03/G1 are not complete.

## Bounded engine contract and acceptance

2019 Rule **9.16(g), example (2)** explicitly charges a batter-runner who replaces a forced-out inherited runner to the prior pitcher. Before this checkpoint the engine forced Plate-origin responsibility to the current pitcher, so that supported case could not be represented. The new typed fielder's-choice PA supports a deliberately bounded **one occupied runner, runner out, batter safely to first, one out, no runs** shape. The batter carries the retired runner's responsibility; no vector-order inference or independent fielding roll occurs. Multi-runner replacements, tag-vs-force cause, charge chains, mid-PA count exceptions and full ER reconstruction remain separate work; unrepresented subtype shapes must return `UnsupportedRule`, not fabricated charges.

Acceptance: replacement from each occupied base and each 0–2 prior-out state; original pitcher rather than reliever charged when the replacement later scores; repeated replacement preserves charge; third-out clearing preserves non-scoring/PA accounting; forged responsibility, FC hit/walk credit or contradictory caught-stealing retirement is rejected by reducer, validator and codec; unsupported multi-runner shapes rejected; existing hit-plus-runner-out remains a non-FC play. Full applicable Rust/Python checks and fresh substantive review precede commit. E-03 remains active and G1 pending.
