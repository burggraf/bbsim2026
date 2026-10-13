# F-01 — bounded 2019 raw outcome-detail supplement

2026-10-12. **Internal offline pilot, not a production store or calibration/G1/G2 signoff.** This extends the [pinned source contract](2019-source-detail-scope.md) and [catalog preflight](2019-catalog-preflight.md). The maintainer directed continuation until an explicit decision is needed. No public redistribution, extra seasons, core/UI/RNG/rating changes, dependency installation or preview restart is authorized here. Current acceptance/results live in the [sole ledger](../PROGRESS.md).

## Acceptance preregistered before reads

Use the existing2,429-game2019 universe and immutable F-03 cache; retain no-play identities. First probe seven fixture games, then extend only if identity, credit, actor and transfer checks pass. Retrieve exactly the five minimal detail models plus a complete source-event identity bridge:29 active Parquet objects, no legacy/all-years-table fallback or largest event-player table. The bridge compares **all34 event columns bidirectionally**, including global/local identity, no-play, before context, actual results and literal batted fields. Original input bytes cannot change.

Shared concurrent response-body reservation caps:512 MiB probe,1 GiB season including failed/recovery/diagnostic attempts, cumulative upper ceiling1.5 GiB. Native time300s per stage/envelope, local file<250 MiB, at least1 GiB free space. Direct read-only catalog inspection found0 active delete files,0 candidate inline mappings and0 encrypted candidate files; native Parquet readers validate file format. The metadata has no data-file `format` column; its retained binder failure is an interface error, not evidence of a missing model.

The loopback-only relay permits exact pinned public HTTPS objects and bounded single byte ranges only. Native HEAD uses the catalog size; every actual range must return206, the exact Content-Range/Content-Length/total object size, identity encoding and a stable strong ETag. It reserves the whole requested span before origin body reads, never refunds failed reservations, has no redirect/retry/full-GET fallback and records body bytes/hash. No credentials are forwarded. Native extension autoinstall/autoload and startup scripts are disabled; installed httpfs is loaded explicitly. The catalog is not fetched again or attached remotely.

## Observed coverage and reconciliation

| Fact | Verified count |
|---|---:|
| Selected games |2,429|
| Source/cached event identities, all columns equal |222,047|
| Real events / no-play markers |192,025 /30,022|
| Unique, nonblank, unmodified raw audit rows |222,047|
| Sparse parsed flag rows |10,548|
| Raw baserunner rows |196,161|
| Real-event / no-play baserunner rows |182,718 /13,443|
| Ordered fielding-credit rows |177,670|
| Fielding appearance records |69,613|
| PO / A / E |130,270 /43,968 /2,836|
| Runs |23,467|
| Unknown recorded-credit positions |0|
| Unresolved required credited-position actors |0|
| Retained terminal empty appearance intervals |1|

Unique grains, orphan keys, raw game/event/file/line identities, mandatory fields and source/cached real-event runner roles/IDs/scoring/out units pass. Runner/credit children are independently aggregated before event joins. Assists are deduplicated by position per source-defined Putout/Error segment, not capped by outs. PO/A/E reconcile event-by-event to cached supplied credits; PO and runner scores also reconcile to actual event outs/runs. These are **same-source reconciliation checks**, not independent official box-score or physical chronology proof.

The derived personnel/statistics lookup excludes no-play markers. An initial unqualified raw/derived runner comparison therefore found13,443 unmatched no-play rows, not13,443 contradictory real-event actors. The raw rows stay intact; the real-event comparison excludes no-play by the pinned source definition and tests that omission explicitly. No unavailable derived no-play statistics are filled with zeros or fictional players.

Each supplied known credit position resolves to one distinct player on the opposite batting side and inclusive event range. No ANY_VALUE conflict suppression. This certifies required credit mappings, **not** complete defensive opportunity, appearance participation, roster availability or throw observation.

### Legal terminal interval, not a fabricated played appearance

The full gate initially rejected one range: `ATL201906110`, `culbc001`, Home position7, start92/end91. The game ends its supplied history at event91; events90–91 are NP substitutions, with no subsequent played event. The source row is preserved exactly, applies to **no** event and earns no inferred participation/credit.

Independent [MLB reporting](https://www.mlb.com/news/chris-archer-struggles-with-homers-vs-braves) confirms the game was called for rain before Pittsburgh's ninth-inning opportunity, after a1-hour48-minute delay. The source staging model passes through appearance ranges rather than repairing them. The validator accepts only a recorded empty terminal span `(last_event_id+1,last_event_id)`; all other reversed/skipped ranges still fail. Regression tests witness the positive terminal boundary, strict negative boundaries, ambiguous/missing actors and unknown positions. This is not a universal interval relaxation or an invented substitution/throw chronology.

### Keyed factual examples

| Game/event | Raw text, retained verbatim | What is supplied |
|---|---|---|
| ANA201904040/96 |`K+PB.B-1`|K with PB and batter advance;0 actual outs|
| ANA201904080/25 |`K+CS2(24)/DP`|K/CS compound and DP modifier;2 actual outs|
| CIN201909020/24 |`K/NDP.1X2(24)`|Explicit first-runner retirement and NDP;2 actual outs|
| ARI201906040/9 |`5E3/G56.B-2`|Error-bearing zero-out InPlayOut with batter advance|
| HOU201905220/19 |`5(2)4(1)3/GTP`|Runner-relative triple-play notation;3 actual outs|

These facts are inputs for a subsequent mechanism audit/parser. Do not invent clock-time ordering, pitch types, catches or all intervening throws from credit sequences alone. Sparse flags are literal **published parsed labels**, not every raw slash modifier; original text stays separate. Foul and FoulLine remain distinct supplied location categories, not universal legal rulings. Missing opportunity/range/arm/cause information continues to require labeled uncertainty/priors.

## Transfer and failure evidence

Source definition commit `b449f7bec49f852c27803e096303a4c78ac661b9`; catalog SHA256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583`, snapshot857. Original cache SHA256 `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528` remains unchanged. A source commit does not by itself pin publication; actual object URLs, sizes, strong ETags and range-body hashes are retained separately.

| Stage | Payload bytes received | Reserved bytes | Seconds | Result |
|---|---:|---:|---:|---|
| Seven-game five-table probe, `proc_8269` |248,384,369|248,384,369|92.34|Passed extraction/factual checks|
| Seven-game34-column bridge, `proc_987d` |39,339,919|39,339,919|19.55|Passed identity|
| Season first attempt, `proc_378e` |246,007,026|248,384,369|90.51|Four tables complete; one appearance range RemoteDisconnected|
| Recovery launch, `proc_8fe2` |0|0|<1|Stopped on retained WAL guard, before network|
| Isolated-copy recovery, `proc_88b4` |11,265,251|13,642,594|33.58|Same range timed out before headers|
| Controlled four-range diagnostic, `proc_d50b` |5,014,674|5,014,674|4.58|All unchanged requests passed|
| Final two-table stage, `proc_7f50` |52,982,513|52,982,513|20.04|Passed extraction/factual gate|

Probe total287,724,288 bytes/111.90s; season/recovery/diagnostic total315,269,464 actual bytes/320,024,150 conservatively reserved bytes, about149s including the no-network guard launch. Both original envelopes pass; no thresholds, seeds, request semantics or ETag checks were relaxed. The same previously failing full span returned CF HIT and a byte hash/ETag identical to the initial successful probe. Its exact intermittent upstream failure cause remains unknown; the diagnostic did **not** justify speculative chunking or blind automatic retries.

Failed database+WAL and every raw log remain frozen. Only isolated copies replay the WAL. The four completed tables are compared bidirectionally to the frozen failed attempt; missing appearance/bridge tables are fetched separately, not all completed tables again. The new local file is17,575,936 bytes before publication. Total session response payload602,993,752 bytes (607,748,438 conservatively reserved) is below the preregistered1.5 GiB cumulative ceiling; exact stage sums, attempts and raw evidence are retained in the manifest. Native profile bytes are loopback response bodies; relay bytes are actual approved-origin response bodies, **not total TLS/wire bytes**.

Small output does not mean small transfer: the seven-game detail probe already required about237 MiB. Source-event `season=2019` pushdown is visible; sparse-table key and appearance-game filters scan much larger physical row groups. Full-file metadata479,089,115 bytes is not a transfer measurement. The internal pilot is feasible but not an efficient production update strategy.

## Reproduction and retained evidence

Tools: installed DuckDB v1.5.6/httpfs and Python stdlib; no dependency installs. All raw data, native scripts/profiles, failed partials/WALs, hashes and range traces remain outside Git under `/tmp/bbsim-outcome-details/`. Actual staged extraction used the retained `run_extract*.py` scripts, then the tracked complete offline gate. The reusable extractor below is additionally verified by a native tiny-Parquet end-to-end fixture (mocked origin, real loopback/httpfs), not misreported as a second cold full-season transfer.

```sh
# New, nonexisting external directory. This command makes bounded data reads;
# do not run it merely to reverify the already retained offline supplement.
python3 -m audits.outcome_details \
  --catalog /tmp/bbsim-catalog-preflight/baseball.ducklake \
  --snapshot /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  --directory /tmp/bbsim-outcome-details-new

# Existing artifact: reproduce read-only checks with external access disabled
# after the local cache attachment and temporary expected-table setup.
duckdb -readonly -bail -batch -init /dev/null -csv \
  /tmp/bbsim-outcome-details/2019-outcomes.duckdb \
  -f /tmp/bbsim-outcome-details/season-verify.sql
python3 -Werror::ResourceWarning -m unittest \
  tests.test_bounded_ranges tests.test_outcome_details tests.test_outcome_detail_extract -v
```

Native live extraction profiles, staged/accepted ownership evidence and fresh complete project checks are linked in the ledger. Genuine semantic reds cover concurrent budget/allowed origins, source identity/grain/blank text/actors, per-segment assist deduplication, runner-role/statistic matching, unknown positions and terminal empty records; interface/harness/WAL/network failures remain distinct. No historical test pass certifies new code.

**Attribution:** The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".

## Next boundary

Raw/sparse/actor facts are available offline; universal force/tag/appeal/retirement timing and all physical throws are not thereby certified. Preregister a bounded raw-mechanism inventory with explicit unknown categories and keyed independent checks before a parser or richer outcome policy. Do not fit ratings, retag contact, reskin/restart the viewer or resume the deferred ER branch simply because the import passed. No public source-data redistribution, production/binary-license clearance, predictive accuracy or G1/G2 signoff is claimed.
