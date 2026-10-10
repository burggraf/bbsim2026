# Development progress and restart point

**Last updated:** 2026-10-10 UTC

**Current state:** F-01 inventory and F-02 fixture audit verified. G0 remains pending; no application code.

**Active task:** none. **Next ready task:** F-03 — filtered import/storage spike.

**Blockers:** none for the development-host F-03 spike. Independent 2019 box-score team lines are absent; same-lineage event/summary reconciliation is not independent historical validation. Target desktop OSes, release/licensing and pilot choice require later maintainer decision at F-04; do not sign off G0.

This is the single status ledger. Task definitions and gate criteria are in [EXECUTION.md](EXECUTION.md); product/modeling decisions are in [PLAN.md](../PLAN.md). Statuses: `planned`, `ready`, `active`, `blocked`, `done`. `done` requires linked evidence and exact verification results, not just an implementation commit. A blocked task must name the blocker and decision needed. Update this file at each meaningful checkpoint and before pausing, so it remains usable without prior chat context.

## Task status

| ID | Status | Evidence / blocker |
| --- | --- | --- |
| F-01 | done | [Query](../audits/2019.sql) v1, [validator](../audits/verify_2019.py), [69-metric report](data-audits/2019.md) and [tests](../tests/test_f01_audit.py). Two independent read-only aggregate runs yielded byte-identical CSV SHA-256 `775e9629132013f1e448e1f5d62636a7da14bccb007315cb66568a3a8ee210f3` (outside Git); 7/7 unit tests and `git diff --check` passed. Metadata hash stable. G0 not signed off; limitations in report. |
| F-02 | done | [Fixture report](data-audits/2019-fixtures.md), [selection](../audits/2019-fixture-discovery.sql), [details](../audits/2019-fixture-details.sql), [reconciliation](../audits/2019-fixture-reconcile.sql), [diagnostics](../audits/2019-fixture-diagnostics.sql), validators/tests. Five events in three games pass state checks; 54 event-to-derived-summary cells match; zero independent 2019 team box rows. All four outputs reproduced byte-for-byte, 18/18 unit tests passed, `git diff --check` passed. The 414 BIP difference is definitional; one schedule row is marked `Not made up` but exact game-by-game schedule matching is not proved. G0 pending. |
| F-03 | ready | Development-host cold filtered transfer, local reconciliation/offline reads, atomic failure handling and packaging evidence; target OS decision reserved for F-04. |
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
| D-05 | provisional | DuckDB is the initial local-cache candidate because the source publishes DuckLake/Parquet; keep a storage-neutral engine/repository boundary. | F-03 packaged/offline measurement. |
| D-06 | pending | Target desktop OSes, pilot coverage minimums, storage choice and performance budget. | F-04, based on evidence. |
| D-07 | pending | Accuracy metrics/tolerances and held-out protocol for each model version. | Record baseline criteria before P-01, conditional criteria before A-01–A-03, and season criteria before S-03; see G2–G4. |

## Resume here

1. F-01 complete; read [2019 audit](data-audits/2019.md) and its exact reproduction commands before selecting a pilot. Changed: `.gitignore`, `audits/2019.sql`, `audits/verify_2019.py`, `tests/test_f01_audit.py`, `docs/data-audits/2019.md`, this ledger. No app scaffold or source rows. Public catalog SHA-256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583` was stable across the audit; source-model commit `b449f7bec49f852c27803e096303a4c78ac661b9`.
2. Exact verification: `duckdb :memory: -csv -noheader < audits/2019.sql > /tmp/bbsim-f01-2019.csv` plus `python3 audits/verify_2019.py /tmp/bbsim-f01-2019.csv` printed `validated 69 metrics; 2429 games; 222047 events`; independent rerun and `cmp` yielded identical SHA-256 `775e9629132013f1e448e1f5d62636a7da14bccb007315cb66568a3a8ee210f3`. `python3 -m unittest discover -s tests -v` passed 7 tests; `git diff --check` passed. Report preserves earlier parser/binder/denominator failures. **No G0 signoff**; no filtered-download/offline or licensing proof. Do not commit CSV/catalog metadata.
3. F-02 complete: new `audits/2019-fixture-{discovery,details,reconcile,diagnostics}.sql`, `audits/verify_fixture_{details,reconcile}.py`, `tests/test_f02_{details,reconcile}.py`, [fixture report](data-audits/2019-fixtures.md), this ledger. Exact commands/hashes in report. Two independent runs of each SQL file matched byte-for-byte (`cmp` exit 0); `python3 audits/verify_fixture_details.py /tmp/bbsim-f02-details-second.csv` validated 5 transitions; `python3 audits/verify_fixture_reconcile.py /tmp/bbsim-f02-reconcile-second.csv` validated 54 metrics, **0 independent box cells**; `python3 -m unittest discover -s tests -v` passed 18; `git diff --check` passed. Earlier expected red imports and limitations in report. G0 still pending.
4. Next exact action: check Git status and start **F-03** with a season-filtered cold transfer to an external temp cache. Measure actual network bytes, elapsed/disk, source-to-local counts and offline read; test atomic failure cleanup. Do not commit a season dataset or choose target OS/storage release policy without F-04 evidence and maintainer input.
