# Development progress and restart point

**Last updated:** 2026-10-10 UTC

**Current state:** planning documents prepared; no application code or data audit has started.

**Active task:** none. **Next ready task:** F-01 — read-only 2019 source coverage inventory.

**Blockers:** none for F-01. Do not assume 2019, DuckDB packaging or any accuracy threshold is approved for release.

This is the single status ledger. Task definitions and gate criteria are in [EXECUTION.md](EXECUTION.md); product/modeling decisions are in [PLAN.md](../PLAN.md). Statuses: `planned`, `ready`, `active`, `blocked`, `done`. `done` requires linked evidence and exact verification results, not just an implementation commit. A blocked task must name the blocker and decision needed. Update this file at each meaningful checkpoint and before pausing, so it remains usable without prior chat context.

## Task status

| ID | Status | Evidence / blocker |
| --- | --- | --- |
| F-01 | ready | Next: reproducible 2019 source-coverage report. |
| F-02 | planned | Depends on F-01. |
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

1. Read `AGENTS.md`, `PLAN.md`, `docs/EXECUTION.md` and this file; check Git status and the latest committed work. Verify any `active` task's artifacts rather than trusting its label.
2. Begin **F-01**: inspect the public source read-only, create reproducible coverage queries/CLI and `docs/data-audits/2019.md`. Include denominators, unknowns, source version and data-quality caveats. Do **not** download a full catalog, commit data extracts or scaffold the desktop app yet.
3. Before ending, update the F-01 row and this handoff with files changed, exact commands/results, blockers, and the next actionable step. Do not mark F-01 `done` until its report can be reproduced and reviewed.

**Documentation checkpoint:** `PLAN.md` now links the execution, status and methodology files; `AGENTS.md`, `docs/EXECUTION.md` and this ledger are new. `git diff --check` passed. An inline Python structural check passed for all four files: local links/newlines/whitespace, matching 18 task IDs, G0–G5 and exactly one ready task (F-01). No software tests exist or have been run; this is not G0 evidence. Check Git history/status for the publication state of this documentation checkpoint.
