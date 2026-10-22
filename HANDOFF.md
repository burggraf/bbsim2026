# Restart handoff

**Updated: 2026-10-22 UTC.** This is a compact restart snapshot, not a second live
status ledger. Keep ongoing task status and evidence in
[docs/PROGRESS.md](docs/PROGRESS.md), especially its first **Resume here** entry.
Refresh this file when handing off another session.

## Start here

1. Read [AGENTS.md](AGENTS.md), [PLAN.md](PLAN.md),
   [docs/EXECUTION.md](docs/EXECUTION.md), and the current summary/first Resume in
   [docs/PROGRESS.md](docs/PROGRESS.md). Older entries preserve historical states;
   their old “uncommitted/no push” notes do not describe the latest checkpoint.
2. Run `git status --short --branch` and `git log -5 --oneline`.
3. Check publication verification below before doing new implementation work.
   Do not infer a completed suite from an old result or a running process.

## Saved and pushed

- `3c9a59b` — opt-in pitcher-usage v2 integration, native entry evidence and
  validation, Python orchestration, regression tests, retrospective results,
  and 1976 readiness notes. Pushed to `origin/main`, together with the 17 earlier
  local commits. No force push, default change, new season run, or release.
- This handoff and the canonical ledger's publication entry are a separate
  documentation follow-up. Find its commit with `git log -5 --oneline`.

## What is implemented / what happened

- Pure Rust canonical event engine and seeded full games, real lineup/cast
  inputs, opt-in pinch hitting and season-pool management, Python batch runner,
  and a bounded internal preview exist. This is not the finished Tauri/Svelte app.
- Managed-season v1 completed all 2,429 played 2019 games at seeds 1/2/3, but its
  individual-pitcher usage evaluation failed. **V1 remains NO-GO and the default.**
- V2 distributes starts among supported starters and uses continuous relief
  workload debt with exact native rankings and trace/accounting validation.
  **V2 remains opt-in.** The recorded fixed seeds 1/2/3 completed 2,429 games each
  and passed their frozen retrospective usage criteria. Details, denominators,
  caveats and original artifact hashes:
  [v2 results](docs/modeling/pitcher-usage-v2-results.md),
  [policy](docs/modeling/pitcher-usage-policy-v2.md),
  [wire](docs/modeling/pitcher-usage-v2-native-wire.md), and
  [entry evidence](docs/modeling/pitcher-entry-decisions-v2.md).
- Do not retune or rerun the fixed evaluation simply to regenerate missing proof.
  This is retrospective development evidence, not predictive validation, the
  100+ season S-03 ensemble, default-promotion approval, or G1–G5 signoff.

## Verification at this handoff

Publication check process: `proc_0a75` (`publication-full-checks-20261022`). At the
handoff write, **fmt and all Python discovery tests exited 0; Rust and subsequent
checks were still running**. No full-verification claim is made for this commit.

Durable local logs (not in Git):

`~/Library/Application Support/bbsim2026/work-checkpoints/publication-20261022/`

Read `results.json`, each raw `*.stdout` / `*.stderr`, and final
`verification.json` if present. `verify.py` records exact commands and source pins;
`target/` is isolated from protected `engine/target` binaries. The verification
started before the documentation follow-up: its final source guard will report
`HANDOFF.md` / `docs/PROGRESS.md` changes; distinguish those expected documentation
changes from any unexpected executable-source change and from command failures.

Checks scheduled, from repository root:

```sh
cargo fmt --manifest-path engine/Cargo.toml --check
python3 -B -Werror -m unittest discover -s tests -v
cargo test --locked --offline --manifest-path engine/Cargo.toml
cargo test --locked --offline --manifest-path engine/Cargo.toml --examples
cargo clippy --locked --offline --manifest-path engine/Cargo.toml --all-targets -- -D warnings
cargo build --locked --offline --manifest-path engine/Cargo.toml --examples
git diff --check
```

Use an explicit isolated `CARGO_TARGET_DIR` for reruns. Toolchain at publication:
Rust/Cargo 1.99.1, Python 3.15.0. The outgoing tree/history scan found no
credential-pattern matches or files over 2 MB; that is not a comprehensive
security review. No scientific seasons, browser checks, or unavailable external
compatibility/evaluation harnesses were rerun for this publication.

## Important evidence gap

On 2026-10-22 the following referenced roots were **absent**:

- `/tmp/bbsim-pitcher-v2-integration/`
- `/tmp/bbsim-pitcher-v2-evaluation-20261019/`
- `/tmp/bbsim-season-usage-targets-v1/`
- `/tmp/bbsim-1976-audit-v1/`

Historical hashes/results in the ledger are preserved records, not proof that raw
reports, scripts, binaries, or traces remain recoverable now. Other private
checkpoints exist under `~/Library/Application Support/bbsim2026/work-checkpoints/`;
inspect manifests before claiming any one restores the missing final artifacts.
Do not commit datasets, caches, raw run logs, credentials, or private archives.

## 1976 exploration — audit only, not a replay

The prior read-only audit recorded 1,939 regular games, **147,598 PA**, and
**104,777 integer pitcher outs**, reconciled between Retrosheet event and box
values. Its missing temporary scripts/reports need recovery or a newly scoped
reproducible audit before relying on them for implementation.

Recorded issues: final-makeup-date/unordered-pair grouping reconciles schedule
counts but does not uniquely identify postponed games; two site changes reverse
home/away; schedule `Num` is not gameinfo `number`. Roster identity does not prove
daily availability. Twelve roster player/team keys lack fielding value rows;
`d_pb/d_wp/d_sb/d_cs` are blank in 40,582 of 45,099 fielding value rows, not zero.

`BOS197607060` and `CHA197609230` have CHA pitcher-in-order lineups opposite a DH
lineup despite a game-level DH flag. Core per-team DH state can represent this;
the current lineup adapter uses one DH flag and 2019-only dates. Club maps/game
counts are also 2019-specific. The applicable primary 1976 DH rule text was not
captured. `CHN197604210` and `TEX197605262` were next-day suspended completions.

**Scope nuance:** PLAN already approves modeled season-pool availability and
normalizing abnormal played games to ordinary full games for the pilot. Do not
reimpose dated-roster reconstruction as a universal prerequisite. For a proposed
1976 experiment, confirm whether those existing choices extend to that year;
date-aware suspension simulation is not automatically required. Preserve source
flags and distinguish normalized simulation from exact historical replay.

Archive identities recorded by the prior audit (not currently present):

| Archive | SHA-256 |
| --- | --- |
| `1976csvs.zip` | `33832efb42f530e24c1aaeb8cc05f2a33a8be6e65456c3f746aaec8bee9cfdba` |
| `1976eve.zip` | `cbe3b30e00ab18e0598be6b14426b9128e9183d41ea3b120810fb08046d788b2` |
| `1976SKED.zip` | `cc1188f0f12e05b026028241aad3d8963eb372d25f12629882be96d5fcd47a0b` |

## Next concrete action

**First:** collect the publication check's final results, investigate any actual
failures without weakening tests, and update the canonical ledger. If the process
is gone, inspect the durable logs before choosing missing checks to rerun.

**Then:** ask the maintainer to choose the next bounded work package. For 1976,
confirm replay scope/availability/normalization assumptions, recover or reproduce
the audit, resolve mixed-DH and 2019-only adapter/rating inputs, and preregister
comparison measures before simulation. No 1976 replay is approved by this
save-and-handoff request. No further v2 tuning/default promotion is approved.

**Release hold remains:** written source-scope clarification and licensing review
are outstanding. This authorized source-code push is not permission to distribute
source datasets, bundled extracts, an app, or a release. Preserve Retrosheet
attribution and uncertainty/unknown accounting; do not turn unknown PO/A into zero.
