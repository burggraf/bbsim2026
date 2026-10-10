# F-03: 2019 development-host filtered transfer/offline snapshot spike

**Result:** a reproducible **development-host** import and a minimal relocatable CLI/extension test are viable on the maintainer-selected **macOS Apple Silicon (arm64)** initial target; this is **not** a signed/notarized app, a production store decision, an independent data-quality validation, a redistribution-license decision, or G0 approval. Public baseball.computer DuckLake v1 is a preview. The local snapshot is an uncommitted temporary research artifact. All source access was read-only; selected rows, not the full catalog, were imported.

## Method and provenance

On Darwin 27.0.0 arm64, DuckDB CLI v1.5.6 (`069cc9f9b5`), `ducklake`/`httpfs` extensions installed in the host profile. `audits/filtered_spike.py` first streams the public 13,119,488-byte catalog metadata over HTTPS and checks SHA-256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583` (the F-01/F-02 version). `audits/2019-filtered-extract.sql` v1 attaches that catalog `READ_ONLY`, disables DuckDB's external file and HTTP metadata caches, and materializes only the selected 2019 AL/NL regular-season games, 2019 AL/NL schedule, their events/derived details/results, and selected park/person lookups. It stores source counts computed by **separate direct source aggregates** under the same selection predicates, plus source URL, catalog hash, query version, timestamp and Retrosheet attribution in the local DB. Child-table aggregates join the locally selected game/event key sets, so these count comparisons test transfer/completeness for that selection; the separate F-01 baseline checks constrain parent-key omissions but are still source-derived. The catalog hash fingerprints metadata, **not** individual Parquet objects. A second catalog GET after the run had the same hash, but concurrent Parquet-object mutation remains unexcluded.

The wrapper writes to a unique temporary file **in the destination directory**, verifies it with a read-only DuckDB CLI and `enable_external_access=false`, then atomically links it into place without overwriting an existing snapshot. Failures remove the staging directory; text profiles/errors remain in the external output directory. The default 1 GiB free-space preflight is only a spike safeguard, not a production capacity forecast. No raw data/cache/logs are in Git. Query, wrapper, profiles, validator output and tests are the reproduction evidence; the 22 MB snapshot remains outside Git.

```sh
OUT=$(mktemp -d /tmp/bbsim-f03.XXXXXX)
/usr/bin/time -p python3 audits/filtered_spike.py --output-dir "$OUT"
python3 audits/profile_http.py "$OUT/f03-profiles.txt"
duckdb -readonly -bail -batch -csv -noheader \
  -cmd 'SET enable_external_access=false' "$OUT/2019-spike.duckdb" \
  -f audits/2019-offline-verify.sql
shasum -a 256 "$OUT/2019-spike.duckdb"
python3 -m unittest discover -s tests -v
# Inspect $OUT/f03-catalog-check.txt, f03-profiles.txt, f03-verify.csv and stderr files.
```

The recorded first successful cold client run (2026-10-10 UTC, `/tmp/bbsim-f03.57WsAQ`) exited 0 after **41.96 s** wall (**25.34 s** user, **3.98 s** sys); file size **22,818,816 bytes** (21.76 MiB); SHA-256 `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`. The first attempt stopped **before import** with HTTP 403 because Python's default urllib user agent was rejected; a named audit user agent got HTTP 200, and the retry succeeded. The failed attempt left no partial DB. A post-run fresh catalog GET again hashed to the same SHA-256 and matched the F-02 catalog byte-for-byte. All three raw CLI logs/CSV are outside Git.

DuckDB's `EXPLAIN ANALYZE` HTTPFS panels on the 13 remote statements reported **321 GETs** and approximately **963,587,584 received bytes (918.95 MiB)**, from displayed rounded KiB/MiB values. The preflight catalog GET added **13,119,488 exact bytes (12.51 MiB)**; measured data+preflight is therefore approximately **931.46 MiB**, a **lower bound** for the process, not a wire-level total. DuckLake attachment's catalog/metadata requests, HTTP/TLS overhead and retries outside the query panels were not measured. “Cold” means a fresh DuckDB process with DuckDB external cache disabled; the publisher/CDN and OS caches were not controlled. The `source_counts` verification query alone reported **244.3 MiB / 134 GETs**; all-column event/state/fielding scans account for much of the remainder. This is season-filtered **output**, not a claim that Parquet range reads were proportional to 2019 rows. Raw plans are in `f03-profiles.txt` outside Git; `audits/profile_http.py` only summarizes those rounded panels.

## Reconciliation and offline checks

The fresh direct-source counts and local counts in `f03-verify.csv` agreed for all **12** selected tables:

| Table | Source = local rows |
| --- | ---: |
| games / results | 2,429 each |
| schedule | 2,430 |
| events / states | 222,047 each |
| batting | 186,517 |
| running | 182,718 |
| fielding | 192,025 |
| batted | 125,754 |
| team_results | 4,858 |
| parks / people | 35 / 1,410 |

The offline verifier also checked unique event IDs, complete event-to-state joins, 2019 selection, manifest version/hash, a pinned F-02 transition (`ANA201904040` event 36), F-01 row-count baselines, and selected F-01 source totals: 30,022 no-play events, 118,577 BIP events, 2,836 fielding errors, 2,280 steals, 832 caught stealing and 6,394 extra bases taken. A **fresh** run of the strengthened offline SQL (after the initial import) exited 0 and printed `F-01 event/credit totals equal`. Those prior totals came from a separate source query on the same catalog hash; they do **not** provide independent historical box-score validation. The F-02 catalog had zero independent 2019 team box-score lines, and G0 remains pending.

A second read-only CLI session with an **empty HOME** and `enable_external_access=false` read 222,047 events, 222,047 states and 2,429 games without DuckLake/httpfs installed under that HOME. Thus **reading this plain DuckDB file** does not require remote access or those extensions on this macOS host; building/refreshing it does. One exploratory `duckdb_extensions()` introspection under `enable_external_access=false` produced a permission error when it tried to inspect the empty HOME; the plain-count retry succeeded, so no extension-installation conclusion relies on that introspection.

## Initial target packaging assessment (macOS arm64)

On 2026-10-10 the maintainer selected **macOS Apple Silicon only** for the initial desktop target. An isolated copy in `/tmp/bbsim-f03-bundle.EViWGA` used a copied DuckDB v1.5.6 arm64 CLI (50,191,144 bytes) plus copied matching `ducklake` and `httpfs` extension binaries (27,601,662 and 16,030,718 bytes). The three files total **93,823,524 bytes (89.48 MiB)**, outside Git. The CLI dynamically links only macOS system `libc++` and `libSystem` per `otool -L`. With a fresh empty HOME and an explicit extension directory, the copy loaded DuckLake, attached the public source read-only and counted **2,429** selected games; the **same copied binary**, with external access disabled and no extension directory, read **2,429** local games and **222,047** local events. This makes a development-host macOS arm64 CLI/extension bundle **practical enough for the F-03 spike**, so a SQLite comparison is not triggered. The proof is reproducible without embedding raw season data:

```sh
BUNDLE=$(mktemp -d /tmp/bbsim-f03-bundle.XXXXXX)
mkdir -p "$BUNDLE/bin" "$BUNDLE/extensions/v1.5.6/osx_arm64" "$BUNDLE/home"
cp -L "$(command -v duckdb)" "$BUNDLE/bin/duckdb"
cp ~/.duckdb/extensions/v1.5.6/osx_arm64/{ducklake,httpfs}.duckdb_extension \
  "$BUNDLE/extensions/v1.5.6/osx_arm64/"
HOME="$BUNDLE/home" "$BUNDLE/bin/duckdb" :memory: -csv -noheader -c \
  "SET extension_directory='$BUNDLE/extensions'; LOAD ducklake; \
   ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' \
   AS source (TYPE ducklake, READ_ONLY); \
   SELECT count(*) FROM source.main_models.game_start_info \
   WHERE season=2019 AND is_regular_season \
   AND home_league IN ('AL','NL') AND away_league IN ('AL','NL');"
HOME="$BUNDLE/home" "$BUNDLE/bin/duckdb" -readonly -csv -noheader \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb -c \
  'SET enable_external_access=false; SELECT count(*) FROM games; SELECT count(*) FROM events;'
```

This **does not** prove a signed/notarized `.app`, embedded/Tauri runtime integration, automatic extension loading after relocation, Windows/Linux packaging, future DuckDB version compatibility, or that publishing a season snapshot is licensed. Those remain explicit later integration/release checks, not implied by this CLI spike.

## Failure and decision boundaries

`tests/test_f03_spike.py` exercises the same wrapper with a **small local synthetic SQL**: successful promotion only after offline verification, injected failure after extraction, a failing verification, refusal to overwrite an existing snapshot, and a catalog-hash mismatch before staging. These cases left no playable partial DB. They do not simulate power loss, disk exhaustion mid-write, concurrent imports, network interruption during a full remote scan, or complete production packaging. The wrapper intentionally remains a scoped spike rather than a general importer.

**F-03's scoped evidence is now present for the selected target.** F-04 must decide pilot coverage minimums, attribution/licensing and distribution/storage policy and record a separate maintainer go/no-go for G0. New desktop targets need their own packaging assessment; if DuckDB proves impractical there, compare SQLite behind the same repository boundary. This CLI spike does not lift G0, authorize redistribution, or justify scaffolding the app.
