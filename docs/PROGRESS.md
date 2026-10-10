# Development progress and restart point

**Last updated:** 2026-10-10 UTC

**Current state:** F-01 read-only 2019 source inventory verified; no application code. G0 data gate remains pending.

**Active task:** none. **Next ready task:** F-02 — event/box-score fixture audit.

**Blockers:** none for F-02. The 2,430 candidate schedule rows versus 2,429 played-result/PBP starts, and 118,991 fielding BIP flags versus 118,577 batted-ball BIP flags, need reconciliation; do not infer a missing game or approve 2019, licensing, packaging or accuracy thresholds from F-01 alone.

This is the single status ledger. Task definitions and gate criteria are in [EXECUTION.md](EXECUTION.md); product/modeling decisions are in [PLAN.md](../PLAN.md). Statuses: `planned`, `ready`, `active`, `blocked`, `done`. `done` requires linked evidence and exact verification results, not just an implementation commit. A blocked task must name the blocker and decision needed. Update this file at each meaningful checkpoint and before pausing, so it remains usable without prior chat context.

## Task status

| ID | Status | Evidence / blocker |
| --- | --- | --- |
| F-01 | done | [Query](../audits/2019.sql) v1, [validator](../audits/verify_2019.py), [69-metric report](data-audits/2019.md) and [tests](../tests/test_f01_audit.py). Two independent read-only aggregate runs yielded byte-identical CSV SHA-256 `775e9629132013f1e448e1f5d62636a7da14bccb007315cb66568a3a8ee210f3` (outside Git); 7/7 unit tests and `git diff --check` passed. Metadata hash stable. G0 not signed off; limitations in report. |
| F-02 | ready | Next: fixture IDs and event/box reconciliation, especially schedule/start and BIP-definition differences identified by F-01. |
| F-03 | planned | Depends on F-01. |
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
3. Next exact action: start **F-02** by selecting small public source game IDs and mapping a non-PA runner event and multi-effect play to before/after states; reconcile box credits and investigate the 2,430/2,429 schedule/start difference and 414 BIP-flag difference. Record fixture IDs, queries and unexplained mismatches without committing raw extracts. Check Git status and publication state first.
