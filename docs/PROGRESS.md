# Development progress and restart point

**Last updated:** 2026-10-10 UTC

**Current state:** F-01/F-02 verified; F-03 development-host transfer and offline spike verified but **blocked** on its target-OS packaging decision. G0 remains pending; no application code.

**Active task:** none while F-03 is blocked. **Next ready task:** F-03 target packaging assessment after maintainer chooses initial desktop OS(s).

**Blocker requiring input:** Which desktop OS/architecture(s) are initially supported? Only macOS arm64 was tested; F-03's definition of done requires a target packaging practicality decision (and SQLite comparison if DuckDB is impractical). Independent 2019 box-score team lines are absent; same-lineage reconciliation is not independent historical validation. Release/licensing and pilot choices remain for F-04; do not sign off G0.

This is the single status ledger. Task definitions and gate criteria are in [EXECUTION.md](EXECUTION.md); product/modeling decisions are in [PLAN.md](../PLAN.md). Statuses: `planned`, `ready`, `active`, `blocked`, `done`. `done` requires linked evidence and exact verification results, not just an implementation commit. A blocked task must name the blocker and decision needed. Update this file at each meaningful checkpoint and before pausing, so it remains usable without prior chat context.

## Task status

| ID | Status | Evidence / blocker |
| --- | --- | --- |
| F-01 | done | [Query](../audits/2019.sql) v1, [validator](../audits/verify_2019.py), [69-metric report](data-audits/2019.md) and [tests](../tests/test_f01_audit.py). Two independent read-only aggregate runs yielded byte-identical CSV SHA-256 `775e9629132013f1e448e1f5d62636a7da14bccb007315cb66568a3a8ee210f3` (outside Git); 7/7 unit tests and `git diff --check` passed. Metadata hash stable. G0 not signed off; limitations in report. |
| F-02 | done | [Fixture report](data-audits/2019-fixtures.md), [selection](../audits/2019-fixture-discovery.sql), [details](../audits/2019-fixture-details.sql), [reconciliation](../audits/2019-fixture-reconcile.sql), [diagnostics](../audits/2019-fixture-diagnostics.sql), validators/tests. Five events in three games pass state checks; 54 event-to-derived-summary cells match; zero independent 2019 team box rows. All four outputs reproduced byte-for-byte, 18/18 unit tests passed, `git diff --check` passed. The 414 BIP difference is definitional; one schedule row is marked `Not made up` but exact game-by-game schedule matching is not proved. G0 pending. |
| F-03 | blocked | [Host spike report](data-audits/2019-filtered-spike.md), [extract](../audits/2019-filtered-extract.sql), [offline verifier](../audits/2019-offline-verify.sql), [atomic wrapper](../audits/filtered_spike.py), [profile parser](../audits/profile_http.py), tests. 41.96 s, ~918.95 MiB profiled HTTPFS payload + 12.51 MiB preflight catalog (metadata/overhead excluded), 22,818,816-byte local DB; 12 direct-source/local count pairs match, F-01 event/credit totals match, offline read under empty HOME, synthetic injected-failure cleanup. Full 26/26 tests, offline verifier and `git diff --check` pass. **Decision needed:** initial target OS/architecture(s); only macOS arm64 host evidence, so target packaging/SQLite fallback cannot be settled yet. G0 pending. |
| F-04 | planned | Depends on F-01–F-03. |
| E-01 | planned | Depends on F-02, F-04. |
| E-02 | planned | Depends on E-01. |
| E-03 | planned | Depends on E-02. |
| P-01 | planned | Depends on F-04, E-03. |
| P-02 | planned | Depends on P-01. |
| P-03 | planned | Depends on P-02, F-03. |
| A-01 | planned | Depends on P-03. |
| A-02 | planned | Depends on A-01. |
| A-03 | planned | Depends on A-02. |
| A-04 | planned | Depends on A-01–A-03. |
| S-01 | planned | Depends on A-04. |
| S-02 | planned | Depends on S-01. |
| S-03 | planned | Depends on S-02. |
| X-01 | planned | Depends on S-03. |

## Gate decisions

| Gate | Status | Evidence and signoff |
| --- | --- | --- |
| G0 — data | pending | Await F-04 coverage/import report and maintainer go/no-go. |
| G1 — engine | pending | Await E-03 deterministic fixtures and accounting tests. |
| G2 — playable game | pending | Await P-03 offline full game and applicable checks. |
| G3 — mechanisms | pending | Await A-04 conditional diagnostics and preregistered thresholds. |
| G4 — managed season | pending | Await S-03 reproducible season ensemble. |
| G5 — expanded seasons | pending | Await X-01 second-season audit and checks. |

## Decision log

| ID | Status | Decision / reason | Revisit |
| --- | --- | --- | --- |
| D-01 | confirmed | Season-end ratings for historical **reconstruction**, not a forecasting claim. | Add a separate as-of/predictive mode only with leakage-safe evaluation. |
| D-02 | confirmed | Start with one audited pilot season; **2019 is a candidate, not a coverage claim**. | F-04. |
| D-03 | confirmed | Later season replays use actual schedule and date-appropriate team membership with modeled, labeled availability and autonomous managers. | S-01 source audit. |
| D-04 | confirmed | Canonical engine record is an event's before/after state plus delta, including PA and non-PA events. | E-01 schema review. |
| D-05 | provisional | 2026-10-10: Plain DuckDB local cache read offline without extensions on macOS arm64, but import took ~931.46 MiB measured lower-bound transfer to produce 21.76 MiB. Keep a storage-neutral repository boundary; other target packaging and SQLite fallback are unresolved. | Complete F-03 target-OS assessment, then F-04 storage decision. |
| D-06 | pending | Initial desktop OS/architecture(s) must be chosen now to complete F-03 packaging assessment. Pilot coverage minimums, release/storage choice and performance budget remain F-04 decisions. | Maintainer input for F-03; F-04 based on evidence. |
| D-07 | pending | Accuracy metrics/tolerances and held-out protocol for each model version. | Record baseline criteria before P-01, conditional criteria before A-01–A-03, and season criteria before S-03; see G2–G4. |

## Resume here

1. F-01 complete; read [2019 audit](data-audits/2019.md) and its exact reproduction commands before selecting a pilot. Changed: `.gitignore`, `audits/2019.sql`, `audits/verify_2019.py`, `tests/test_f01_audit.py`, `docs/data-audits/2019.md`, this ledger. No app scaffold or source rows. Public catalog SHA-256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583` was stable across the audit; source-model commit `b449f7bec49f852c27803e096303a4c78ac661b9`.
2. Exact verification: `duckdb :memory: -csv -noheader < audits/2019.sql > /tmp/bbsim-f01-2019.csv` plus `python3 audits/verify_2019.py /tmp/bbsim-f01-2019.csv` printed `validated 69 metrics; 2429 games; 222047 events`; independent rerun and `cmp` yielded identical SHA-256 `775e9629132013f1e448e1f5d62636a7da14bccb007315cb66568a3a8ee210f3`. `python3 -m unittest discover -s tests -v` passed 7 tests; `git diff --check` passed. Report preserves earlier parser/binder/denominator failures. **No G0 signoff**; no filtered-download/offline or licensing proof. Do not commit CSV/catalog metadata.
3. F-02 complete: new `audits/2019-fixture-{discovery,details,reconcile,diagnostics}.sql`, `audits/verify_fixture_{details,reconcile}.py`, `tests/test_f02_{details,reconcile}.py`, [fixture report](data-audits/2019-fixtures.md), this ledger. Exact commands/hashes in report. Two independent runs of each SQL file matched byte-for-byte (`cmp` exit 0); `python3 audits/verify_fixture_details.py /tmp/bbsim-f02-details-second.csv` validated 5 transitions; `python3 audits/verify_fixture_reconcile.py /tmp/bbsim-f02-reconcile-second.csv` validated 54 metrics, **0 independent box cells**; `python3 -m unittest discover -s tests -v` passed 18; `git diff --check` passed. Earlier expected red imports and limitations in report. G0 still pending.
4. F-03 **development-host evidence verified; task incomplete/blocked**. Changed `audits/2019-filtered-extract.sql`, `audits/2019-offline-verify.sql`, `audits/filtered_spike.py`, `audits/profile_http.py`, `tests/test_f03_{spike,profiles}.py`, [report](data-audits/2019-filtered-spike.md), this ledger. Reproduce with `OUT=$(mktemp -d /tmp/bbsim-f03.XXXXXX); /usr/bin/time -p python3 audits/filtered_spike.py --output-dir "$OUT"; python3 audits/profile_http.py "$OUT/f03-profiles.txt"; duckdb -readonly -bail -batch -csv -noheader -cmd 'SET enable_external_access=false' "$OUT/2019-spike.duckdb" -f audits/2019-offline-verify.sql`. First successful cold client run in `/tmp/bbsim-f03.57WsAQ`: 41.96 s, 321 GETs/~918.95 MiB rounded HTTPFS payload, plus 13,119,488-byte preflight catalog; local DB 22,818,816 bytes SHA-256 `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`. Direct source/local counts match 12 tables; strengthened offline check matched F-01 totals and F-02 fixture; empty-HOME offline reads passed. The remote catalog hash matched before/after. `python3 -m unittest discover -s tests -v` passed **26/26**; `python3 audits/profile_http.py /tmp/bbsim-f03.57WsAQ/f03-profiles.txt` reported 13 panels/321 GETs/963587584 displayed-unit bytes; fresh offline SQL and `git diff --check` passed. First preflight failed HTTP 403 on Python's default user agent (fixed with named audit user agent, no partial DB); exploratory `duckdb_extensions()` under disabled external access failed but pure offline reads passed. No raw data in Git. **Next exact action:** obtain maintainer's initial desktop OS/architecture choice, then test practical packaging on those targets (or compare SQLite if impractical); do not mark F-03 done or G0 approved yet. F-02 commit `7ded9a4` preceded this checkpoint; check `git log -2 --oneline` for its F-03 host-spike successor.
