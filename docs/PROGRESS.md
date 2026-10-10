# Development progress and restart point

**Last updated:** 2026-10-10 UTC

**Current state:** F-01–F-03 verified for initial macOS Apple Silicon; F-04 evidence brief drafted but **blocked on maintainer decisions**. **G0 remains pending**; no application code.

**Active task:** none while F-04 is blocked. **Next ready task:** F-04 decision/signoff after maintainer review.

**Blocker requiring input:** Confirm/reject a limited 2019 internal pilot and coverage exclusions, local storage/import/distribution policy and preliminary budget; choose a path to clarify conflicting publisher license signals. No bundled data or public release pending rights review. A separate G0 go/no-go entry is required. The catalog's 2019 team box tables are empty; a bounded external Retrosheet box archive matched 54 fixture metrics but shares upstream and does not prove season-wide accuracy.

This is the single status ledger. Task definitions and gate criteria are in [EXECUTION.md](EXECUTION.md); product/modeling decisions are in [PLAN.md](../PLAN.md). Statuses: `planned`, `ready`, `active`, `blocked`, `done`. `done` requires linked evidence and exact verification results, not just an implementation commit. A blocked task must name the blocker and decision needed. Update this file at each meaningful checkpoint and before pausing, so it remains usable without prior chat context.

## Task status

| ID | Status | Evidence / blocker |
| --- | --- | --- |
| F-01 | done | [Query](../audits/2019.sql) v1, [validator](../audits/verify_2019.py), [69-metric report](data-audits/2019.md) and [tests](../tests/test_f01_audit.py). Two independent read-only aggregate runs yielded byte-identical CSV SHA-256 `775e9629132013f1e448e1f5d62636a7da14bccb007315cb66568a3a8ee210f3` (outside Git); 7/7 unit tests and `git diff --check` passed. Metadata hash stable. G0 not signed off; limitations in report. |
| F-02 | done | [Fixture report](data-audits/2019-fixtures.md), [selection](../audits/2019-fixture-discovery.sql), [details](../audits/2019-fixture-details.sql), [reconciliation](../audits/2019-fixture-reconcile.sql), [diagnostics](../audits/2019-fixture-diagnostics.sql), validators/tests. Five events in three games pass state checks; 54 event-to-derived-summary cells match; zero independent 2019 team box rows. All four outputs reproduced byte-for-byte, 18/18 unit tests passed, `git diff --check` passed. The 414 BIP difference is definitional; one schedule row is marked `Not made up` but exact game-by-game schedule matching is not proved. G0 pending. |
| F-03 | done | [Host and macOS arm64 packaging report](data-audits/2019-filtered-spike.md), [extract](../audits/2019-filtered-extract.sql), [offline verifier](../audits/2019-offline-verify.sql), [atomic wrapper](../audits/filtered_spike.py), [profile parser](../audits/profile_http.py), tests. 41.96 s; ~918.95 MiB profiled HTTPFS + 12.51 MiB preflight catalog (lower bound), 22,818,816-byte snapshot; 12 source/local count pairs and F-01 totals matched. Synthetic injected-failure cleanup passed. Maintainer selected **macOS Apple Silicon**; isolated copied CLI/extensions (93,823,524 bytes) read public source and local snapshot under empty HOME (offline read with network disabled). Fresh 26/26 tests, full offline SQL, copied-binary checks and `git diff --check` passed. CLI packaging practical for selected target; SQLite fallback not triggered. No signed/notarized/Tauri proof or G0 approval. |
| F-04 | blocked | [Draft decision brief](data-audits/2019-go-no-go.md) and [external box validator](../audits/verify_retrosheet_box.py)/[tests](../tests/test_f04_external_box.py): a separate season-specific Retrosheet box archive matched **54/54** F-02 game-team metrics, while the source catalog still has **0** independent box cells. 30/30 unit tests, F-01/F-02 validators, F-03 offline/profile and external-box validator pass; `git diff --check` passes. Retrosheet notice allows broad use with prominent attribution; baseball.computer homepage describes added data as CC BY-SA 4.0, but its repository LICENSE is CC BY-NC-SA 4.0 — scope unclear. **Decisions needed:** 2019 limited pilot/coverage minima, user-import DuckDB vs other store/distribution, preliminary budget and rights-clarification path. Then explicit G0 go/no-go, not inferred from test pass. |
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
| D-05 | provisional | 2026-10-10: Plain DuckDB cache read offline on macOS arm64; copied CLI/extensions loaded source under empty HOME. Import took ~931.46 MiB measured lower-bound transfer to produce 21.76 MiB. Keep a storage-neutral repository boundary; other target packaging is untested. | F-04 storage/distribution decision; revisit for new targets. |
| D-06 | partial | 2026-10-10 maintainer selected **macOS Apple Silicon (arm64)** only for the initial desktop release. Pilot coverage minimums, release/storage choice and performance budget remain F-04 decisions. | F-04; revisit targets if expanding beyond macOS arm64. |
| D-07 | pending | Accuracy metrics/tolerances and held-out protocol for each model version. | Record baseline criteria before P-01, conditional criteria before A-01–A-03, and season criteria before S-03; see G2–G4. |

## Resume here

1. F-01 complete; read [2019 audit](data-audits/2019.md) and its exact reproduction commands before selecting a pilot. Changed: `.gitignore`, `audits/2019.sql`, `audits/verify_2019.py`, `tests/test_f01_audit.py`, `docs/data-audits/2019.md`, this ledger. No app scaffold or source rows. Public catalog SHA-256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583` was stable across the audit; source-model commit `b449f7bec49f852c27803e096303a4c78ac661b9`.
2. Exact verification: `duckdb :memory: -csv -noheader < audits/2019.sql > /tmp/bbsim-f01-2019.csv` plus `python3 audits/verify_2019.py /tmp/bbsim-f01-2019.csv` printed `validated 69 metrics; 2429 games; 222047 events`; independent rerun and `cmp` yielded identical SHA-256 `775e9629132013f1e448e1f5d62636a7da14bccb007315cb66568a3a8ee210f3`. `python3 -m unittest discover -s tests -v` passed 7 tests; `git diff --check` passed. Report preserves earlier parser/binder/denominator failures. **No G0 signoff**; no filtered-download/offline or licensing proof. Do not commit CSV/catalog metadata.
3. F-02 complete: new `audits/2019-fixture-{discovery,details,reconcile,diagnostics}.sql`, `audits/verify_fixture_{details,reconcile}.py`, `tests/test_f02_{details,reconcile}.py`, [fixture report](data-audits/2019-fixtures.md), this ledger. Exact commands/hashes in report. Two independent runs of each SQL file matched byte-for-byte (`cmp` exit 0); `python3 audits/verify_fixture_details.py /tmp/bbsim-f02-details-second.csv` validated 5 transitions; `python3 audits/verify_fixture_reconcile.py /tmp/bbsim-f02-reconcile-second.csv` validated 54 metrics, **0 independent box cells**; `python3 -m unittest discover -s tests -v` passed 18; `git diff --check` passed. Earlier expected red imports and limitations in report. G0 still pending.
4. F-03 **done for the selected macOS arm64 CLI spike**, not a signed/notarized app. Changed `audits/2019-filtered-extract.sql`, `audits/2019-offline-verify.sql`, `audits/filtered_spike.py`, `audits/profile_http.py`, `tests/test_f03_{spike,profiles}.py`, [report](data-audits/2019-filtered-spike.md), this ledger. Reproduction commands and raw-log locations in report. Cold run `/tmp/bbsim-f03.57WsAQ`: 41.96 s, ~918.95 MiB profiled HTTPFS, 13,119,488-byte preflight catalog, 22,818,816-byte local DB SHA-256 `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`. Source/local counts equal for 12 tables; F-01 credit totals and F-02 fixture pass offline; catalog hash stable. Isolated copy `/tmp/bbsim-f03-bundle.EViWGA` under empty HOME loaded the public source for a bounded 2,429-game count and read 2,429 local games/222,047 local events with external access disabled. Fresh `python3 -m unittest discover -s tests -v` passed **26/26**, `python3 audits/profile_http.py /tmp/bbsim-f03.57WsAQ/f03-profiles.txt` printed 13 panels/321 GETs/963587584 displayed-unit bytes, full offline SQL and copied-binary reads passed, `git diff --check` passed. Early HTTP 403 and blocked extension-introspection attempt are preserved in report; no source rows in Git. F-03 host checkpoint `42cc787` and F-02 `7ded9a4` were pushed to `origin/main`.
5. F-04 **draft evidence verified but task incomplete/blocked**. Changed `audits/verify_retrosheet_box.py`, `tests/test_f04_external_box.py`, [decision brief](data-audits/2019-go-no-go.md) and this ledger. Bounded [2019 Retrosheet box archive](https://www.retrosheet.org/events/2019box.zip) (1,741,285 bytes, SHA-256 `34ec6b45fc2b88c6b7675d0c364bffc10e8d4f81e166ef2ad51dd65a1cd73444`) reproduced byte-for-byte on two GETs outside Git. `python3 audits/verify_retrosheet_box.py /tmp/bbsim-f04-2019box.zip /tmp/bbsim-f02-reconcile.csv` validated **54 external box comparisons**; F-02 catalog box cells remain zero. First comparison failed on `ANA201904050` Away `outs` (event 24 vs wrongly mapped offensive 27); F-02 defines **fielding-team** outs, confirmed against same-side putouts and regression-tested. Official [Retrosheet notice](https://www.retrosheet.org/notice.txt) requires prominent attribution; baseball.computer [homepage](https://baseball.computer/) says added data CC BY-SA 4.0, but its [pinned repository LICENSE](https://raw.githubusercontent.com/droher/baseball.computer/b449f7bec49f852c27803e096303a4c78ac661b9/LICENSE) is CC BY-NC-SA 4.0; scope unresolved, no redistribution clearance. Fresh `python3 -m unittest discover -s tests -v` passed **30/30**; F-01 validator (69 metrics), F-02 detail (5) and reconcile (54, 0 catalog box), F-04 external box (54), F-03 offline SQL/profile and `git diff --check` passed. No raw data in Git. **Next exact action:** ask maintainer to review [F-04 brief](data-audits/2019-go-no-go.md) and decide limited pilot, coverage minima, DuckDB user-import/distribution, provisional performance envelope and rights clarification; record explicit G0 go/no-go separately. Do not start E-01 or publish data while blocked. F-03 completion `c5a8e91` was pushed to `origin/main`.
