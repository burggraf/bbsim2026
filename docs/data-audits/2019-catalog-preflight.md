# Published outcome-detail catalog preflight — F-01

## Result

**Schema preflight passes.** The public catalog is byte-identical to the catalog used for the existing F-03 snapshot. All **12 candidate tables and 136 declared columns** from the [pinned source inspection](2019-source-detail-scope.md) are present at its active revision, with matching primitive types and source ENUMs published as VARCHAR.

This proves catalog schema availability, **not 2019 row coverage, grain uniqueness, physical object availability or retirement chronology**. No Parquet/season rows were fetched, no remote DuckLake was attached, and no engine/audit/test/model/UI/RNG files changed. [Internal-use and G1/G2 limitations](../PROGRESS.md) remain unchanged.

## Authorized retrieval and provenance

Maintainer approved the proposed read-only catalog/schema preflight capped at16 MiB, with no Parquet/season-data download. One HTTP GET, no redirect/retry/fallback:

- URL: `https://data.baseball.computer/baseball/v1/baseball.ducklake`
- Retrieved2026-10-12 UTC; HTTP200, identity body, Content-Length **13,119,488 bytes** (12.512 MiB), below16,777,216-byte cap; streamed bytes and file size match exactly. Cap is on response-body bytes, not a measurement of TLS/header overhead.
- SHA-256 **`2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583`**, identical to the [original F-03 catalog](2019-filtered-spike.md). No catalog drift found.
- Last-Modified `Sun, 04 Oct 2026 23:54:19 GMT`; ETag `"c8accc65527641ebb7d643de96a0d26b-2"` (not treated as SHA-256); cache status DYNAMIC and `public, max-age=0, must-revalidate`.
- Single retained snapshot **857**, snapshot time `2026-10-04 15:37:19.265588-07`, schema version857. These are catalog identifiers, not proof of the Git source commit or verified data object bytes.
- Managed fetch `proc_a859`, `python3 /tmp/bbsim-catalog-preflight/fetch_catalog.py`, exit0;2.43 seconds in the fetch manifest. Complete stdout/stderr retained by pi-processes.

Raw catalog, selected response headers, exact scripts, SQL, CSV outputs and hashes are outside Git at `/tmp/bbsim-catalog-preflight/`. The retrieval manifest describes the download; separate verification proves its schema. Never commit this catalog or claim it makes the underlying files durable/reachable.

## Published candidates

Counts/sizes below are **all-year catalog file metadata**, not queried source rows or 2019 coverage. File record counts do not independently account for logical deletes or prove source grain. All136 inspected columns permit NULL in catalog schema; required factual values must still be checked on actual selected rows.

| Candidate | Files | Metadata records | Full file bytes | Columns |
| --- | ---: | ---: | ---: | ---: |
| `stg_events` | 14 | 18,141,020 | 173,675,325 | 34 |
| `stg_event_audit` | 3 | 18,141,020 | 97,205,905 | 6 |
| `stg_event_flags` | 1 | 1,656,399 | 6,437,503 | 3 |
| `stg_event_baserunners` | 7 | 17,411,649 | 140,023,850 | 21 |
| `stg_event_fielding_plays` | 3 | 15,719,352 | 47,622,098 | 6 |
| `stg_game_fielding_appearances` | 1 | 5,071,581 | 14,124,434 | 6 |
| `calc_fielding_play_agg` | 3 | 15,232,398 | 120,541,280 | 12 |
| `event_fielders_flat` | 7 | 16,321,784 | 281,503,168 | 13 |
| `event_personnel_lookup` | 2 | 16,321,784 | 177,860,551 | 5 |
| `personnel_fielding_states` | 4 | 15,380,410 | 166,081,743 | 8 |
| `personnel_lineup_states` | 4 | 18,273,617 | 49,676,954 | 8 |
| `calc_batted_ball_type` | 7 | 12,041,398 | 67,797,560 | 14 |

Notable verified columns: `stg_event_audit.raw_play` VARCHAR with game/event/key/file/line provenance; sparse flag and fielding-credit `sequence_id` UINT8; fielding position UINT8; event keys UINT32; runner start-role/destination/end-base/play-type VARCHAR, with explicit out/scoring/RBI/error flags; defensive appearance actor/side/position and event ranges; separate recorded/inferred batted-ball fields. Every declared column name/type for the12 models was compared with the pinned definitions, not just these highlights. Catalog presence does not validate encoded enum vocabularies or factual nulls.

## Transfer and coverage limits

- No active partition definitions were found for these12 candidates. Do **not** assume files are season-partitioned or that a game/year filter guarantees small transfer.
- `stg_events` file-level season bounds can exclude **4 of14 files** for2019; the10 potentially eligible files total **124,751,178 bytes**. This is a coarse metadata bound, not actual filtered/projection HTTP traffic or proof that each file contains2019. Row-group layout and predicate/semijoin pushdown remain unmeasured.
- The five minimal raw/sparse/appearance candidates total **305,413,790 full-file bytes**. This is neither a download request nor an expected season transfer estimate. The bridge and optional controls add more possible reads; retain the compact appearance route rather than bulk event-player expansion.
- `raw_play` file statistics report zero NULLs in all three files. This does not establish nonblank2019 raw text, complete play semantics or even that the current public Parquet objects are retrievable. Preserve this as metadata evidence only.
- Identical catalog bytes remove the detected catalog-version mismatch risk for the existing snapshot. Future reads must still pin/recheck this revision and verify actual game/event identities, child grains, no-play boundaries, source-to-local credits and unknowns. No keyed data reconciliation was performed here.

## Reproduction and checks

No additional dependencies were installed. The existing DuckDB CLI v1.5.6 opened the downloaded catalog as a **native local read-only database**, with startup scripts disabled and external access disabled. Queries read `information_schema` and native `ducklake_*` metadata tables only. They did not ATTACH DuckLake, evaluate source model views or open data objects.

```sh
set -euo pipefail
python3 /tmp/bbsim-catalog-preflight/inspect_metadata.py
python3 /tmp/bbsim-catalog-preflight/verify_metadata.py
duckdb -readonly -bail -batch -init /dev/null \
  -cmd 'SET enable_external_access=FALSE;' \
  /tmp/bbsim-catalog-preflight/baseball.ducklake -csv \
  -f /tmp/bbsim-catalog-preflight/tables.sql
git diff --check
```

A self-contained candidate schema check against the downloaded native catalog (same offline CLI flags):

```sql
WITH rev AS (SELECT max(snapshot_id) AS id FROM ducklake_snapshot)
SELECT s.schema_name,t.table_name,c.column_name,c.column_type,c.nulls_allowed
FROM ducklake_table t
JOIN ducklake_schema s USING (schema_id)
JOIN ducklake_column c USING (table_id)
CROSS JOIN rev
WHERE s.schema_name='main_models'
  AND t.table_name IN ('stg_events','stg_event_audit','stg_event_flags',
      'stg_event_baserunners','stg_event_fielding_plays','stg_game_fielding_appearances')
  AND s.begin_snapshot<=rev.id AND (s.end_snapshot IS NULL OR s.end_snapshot>rev.id)
  AND t.begin_snapshot<=rev.id AND (t.end_snapshot IS NULL OR t.end_snapshot>rev.id)
  AND c.begin_snapshot<=rev.id AND (c.end_snapshot IS NULL OR c.end_snapshot>rev.id)
ORDER BY t.table_name,c.column_order;
```

Inspection emits `revision`, `schemas`, `tables`, `columns`, `partitions` and `file_bounds` CSVs, with exact SQL and empty stderr. Verification checks the body/file/hash/cap, same original catalog, active revision, all12 tables/136 mapped columns, positive file metadata, no candidate partitions, potential2019 file bounds, original snapshot hash and all non-owned tracked bytes. All six repeated offline metadata outputs matched byte-for-byte on managed `proc_9999`; the self-contained documented SQL also ran successfully and its76 minimal-candidate columns matched the full inspection. An initial foreground repeat hit its10-second tool timeout; the managed rerun completed in15 seconds without changing assertions or repeating the network GET. Exact acceptance evidence and checkpoint are in the sole ledger. Repository behavior tests were not rerun for this documentation-only checkpoint; prior548 Rust/84 Python results remain historical, not new certification.

## Next bounded action

**Obtain authorization for a small keyed-game factual probe before a full2019 supplement.** Use the existing fixture game(s) to verify raw play text, sparse credits/runner facts and defender mappings; preregister network/object scope, transfer ceiling, runtime, failure cleanup and reconciliation before reads. Measure actual filtered transfer—metadata file sizes cannot replace that evidence. The [supplement contract](2019-source-detail-scope.md) remains the acceptance framework. If objects/joins/coverage fail, stop with the precise blocker. No modeling, season extraction, public redistribution or gate signoff is authorized by this preflight.
