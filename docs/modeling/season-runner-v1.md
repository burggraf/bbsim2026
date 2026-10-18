# P-02-season-runner-v1 — controlled headless historical-lineup batch

## Acceptance fixed before code and new season measurements

Base `e4d1bd9`; prerequisite [generic inputs](season-game-inputs-v1.md) is verified.
This is the next approved historical-lineup runner slice, not S-01/S-03, a free
season replay, official statistics, predictive accuracy or a gate signoff.
No new dependencies, transfer, fitting, outcome/manager/RNG changes, UI or push.

1. Authenticate/reconcile the same five complete cached source artifacts before
   constructing a manifest. Include every played game ID exactly once, ordered
   by literal source date then game ID. Preserve the entire readiness schedule
   reconciliation separately: ambiguous DH mappings remain ambiguous and the
   not-made-up slot remains visible. Do not assign it an invented played game.
2. Derive each unsigned 64-bit game seed from the first eight SHA256 bytes
   (big endian) of ASCII `bbsim-season-v1:{master_seed}:{game_id}`. Seeds depend
   on identity, not processing position, success, filtering or Python hash salt.
   Master seed must be an integer in [0,2^64-1]. Record seed algorithm, binary,
   recipes, inputs, model/rules provenance and all per-game bindings.
3. Existing export refusal rules remain binding. Suspended, shortened, tied,
   forfeited or unknown-result games are explicit `input_refused` manifest rows,
   never ordinary nine-inning simulations. Missing casts/profiles and appearance
   issues likewise refuse. This deliberately incomplete controlled batch must not
   be called a complete historical season. No new exception/replay rule is chosen.
4. Reuse the existing source validators and native text-game simulation. For
   practical season execution, compute the unchanged full-cohort workload inventory
   once, then select each game's actors. Target-only bullpen facts may replace
   unused whole-season descriptive fields, but produced lineup/cast TSV bytes
   must match the existing producer. Do not modify the pinned workload recipe.
5. Add an explicit machine-accounting output format to the existing native CLI,
   not another engine or a parser of human display names. Derive its player lines
   from canonical `derive_accounting`, keyed by stable team/player IDs and role.
   Emit known counts and `?` for unresolved AB/RBI/ER/coverage. Track pitching outs
   as integers, never sum decimal IP. Record batting, pitching and observed
   fielding coverage; do not invent pitcher W/L/S or official certification.
   Existing detailed/radio bytes, default format and simulations stay unchanged.
6. Strictly parse the machine schema; reject duplicate identities, malformed rows,
   mismatched seed/game/team bindings, negative/noncanonical counts, missing roles,
   incomplete games and failed team/player PA/BF/run/hit/outs conservation.
   Accumulate team standings and role/player totals with team splits; transferred
   players retain one stable identity. Any unknown contributing cell makes that
   cumulative cell unknown, rather than counting a partial sum as complete.
7. Exclusive external output only. Keep individual input TSVs and raw native
   stdout/stderr, including failures. Each played ID has `complete`, `input_refused`
   or `simulation_failed`; a native failure/timeout is not an input refusal.
   Unexpected producer errors abort without a final acceptance receipt. Publish
   the manifest, schedule and cumulative summary followed by a receipt only after
   rereading all source/recipe/binary pins and generated artifact hashes. An
   interrupted directory is unaccepted/nonreusable. Exit 2 for a recorded partial
   batch; exit 0 only if all manifest rows completed. No resume framework/parallel
   executor/config system in this version.
8. Tests first: stable seeds/order, shuffled source manifest, retained exceptions
   and schedule ambiguity, strict machine/schema refusals, unknown propagation,
   traded-player team splits, conservation, publication drift/overwrite, native
   synthetic DH/non-DH output and byte-compatible old output formats.
9. Actual proof: master seed 1 over **all 2,429 played IDs**, no success-rate target
   or hidden exclusions. Report exact completion/refusal/failure counts and reasons,
   all 30 teams' coverage, cumulative identity/conservation and measured elapsed
   time. Repeat selected supported DH/non-DH/cold games with identical seeds and
   inputs; compare machine totals to radio boxes and input bytes to the old
   producer. A second complete manifest/summary run must be byte-identical apart
   from explicitly external receipt paths. Preserve every failure without changing
   seeds, models, casts or thresholds just to obtain completion.
10. Fresh full Rust/example/Python suites, locked/offline build, fmt, strict
    all-target Clippy, diff and source/artifact preservation checks. No scientific
    rate tolerance is introduced or fitted here; pinch hitting and defensive
    replacements follow separately.

Expected scope: `preview/season_runner.py`, small preparation reuse in
`preview/game_inputs.py`, explicit accounting rendering in
`engine/examples/text_game.rs` and `engine/examples/support/season_accounting.rs`,
focused Python/Rust/CLI regressions, this document and `docs/PROGRESS.md`.

Private evidence: `/tmp/bbsim-season-runner-v1/` (local, not durable or redistributable).


## Run and inspect

Build the unchanged shared simulation plus explicit accounting renderer:

```sh
cargo build --offline --locked --manifest-path engine/Cargo.toml --examples
python3 -B -m preview.season_runner \
  /tmp/bbsim-season-readiness-v1/first.lineup.json \
  /tmp/bbsim-season-readiness-v1/first.lineup.native.csv \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-pitcher-workload/report-null-fixed-first.events.csv \
  /tmp/bbsim-season-readiness-v1/first.json \
  /tmp/bbsim-preview-2019-counts.csv \
  /tmp/bbsim-joint-preview/counts.csv \
  engine/target/debug/examples/text_game \
  /tmp/bbsim-controlled-season-new --seed 1
```

The destination must not exist and must be outside the repository. Existing source
artifacts are private local caches; do not download/repair them implicitly. Exit2
is the expected recorded partial-batch result when exceptions refuse; inspect the
manifest rather than treating this as either a crash or a complete-season pass.
Each native game has a30-second timeout. Unexpected producer/I/O errors abort.

Output:
- `manifest.json`: all played identities, source dates/teams, derived seeds,
  successful input actors/cast sizes/cold profiles, status and refusal/failure reason;
- `schedule.json`: every original schedule-reconciliation row, without resolving
  ambiguous doubleheaders or turning the not-made-up slot into a played ID;
- `games/GAME_ID/{lineup.tsv,cast.tsv,stdout.tsv,stderr.txt}` for each attempted native
  game, including native failures; refused inputs remain manifest entries;
- `summary.json`: team standings/counts and stable `role:player_id` counts with
  per-team splits. Counts derive only from `complete` rows. Unknown cells are JSON
  null, not zero; team splits may be known even if an overall cell is unknown;
- `receipt.json`, written last: source/count/binary/recipe and generated-file hashes.
  A receipt attests a stable recorded attempt, **not** that every game succeeded,
  that inputs establish daily eligibility, or that statistics are official.

### Explicit native accounting schema

`text_game --format accounting` uses the same command/input validation and canonical
transitions as detailed/radio. The tab-separated `controlled-accounting-v1` record
has `game`, `seed`, `engine`, `mechanics`, `ratings`, `manager`, `rng` and `rules`
metadata; `team SIDE TEAM_ID RUNS`; and these fixed role rows:

| Row prefix (then SIDE, PLAYER_ID) | Ordered integer/unknown cells |
| --- | --- |
| batter | PA, AB, R, H, RBI, BB, SO, 1B, 2B, 3B, HR, IBB, HBP, SH, SF, GDP, SB, CS, CI |
| pitcher | BF, outs, AB-against, R, ER, H, BB, SO, 1B, 2B, 3B, HR, IBB, HBP, SH, SF, WP, balk |
| fielder | observed events, PO, A, E, PB |

`SIDE` is Away or Home. `?` retains canonical unresolved credit/coverage; known zero
is `0`. Fielding observed-event counts are not official games played. Pitching outs
are integers; innings/ERA and pitcher W/L/S are not fabricated. The parser checks
identity, role/count shape, PA decomposition, opposing PA/BF/AB/hit/walk/strikeout/
HBP/sacrifice credits, run/fielding-out conservation and known credit bounds.
Permitted role IDs come from the exported starting lineup, declared pitching cast
and explicit batting vectors; a known game actor is not automatically allowed in
another role, and the DH is excluded from defensive bindings.

## Verified bounded results — 2026-10-18

Author acceptance only, not independent model review, overall P-02 completion,
S-01/S-03, statistical fidelity, official scoring or G1–G5 signoff.

Final `proc_cfda` exited0/639s, running the public CLI twice on master seed1:

| Disposition | Games |
| --- | ---: |
| Complete, canonical accounting parsed | 2,375 |
| Input refused: suspended/shortened exception policy needed | 14 |
| Input refused: already-active starting actor in pitching cast | 40 |
| Native simulation failures | 0 |
| Total played identities retained | 2,429 |

The two complete CLI attempts took285.596s and313.416s. Both exit2, intentionally:
this is an intact **partial controlled batch**, not a complete historical season.
All30teams are represented, with153–161 completed games per team. The entire
schedule is source-exact:2363 matched,66 ambiguous and one not-made-up row,
2430 rows total. No schedule ambiguity is resolved or row invented.

The40 personnel refusals are supported by a separate authenticated diagnosis:
each has a source-recorded pitcher already present in the starting lineup, with
matching names. The existing pitcher-only replacement path cannot safely enact
that personnel-role change; this is not a license to drop the actor or repair
source names. Actor-level facts are retained in the private refusal report.

Independent accumulation from raw machine rows agrees exactly with the summary:
1206 batter,802 pitcher and1391 fielder **role identities**, including319 multi-team
role identities. These are not counts of distinct players or certified official
appearances. Standings game/win/loss, league PA/BF and charged/scored-run totals
reconcile; any unknown contributing count remains unknown. All9503 generated
artifacts per attempt (inputs, native stdout/stderr, manifest, schedule, summary)
repeat byte-identically. Each receipt authenticates17 input/binary/recipe bindings.

Selected fixtures were the preregistered DH `KCA201903280`, non-DH `CIN201903280`
and cold non-DH `SFN201904060`, with identity-derived seeds3906761618317814889,
1561282304724806444 and9422362658346759055. Input bytes equal the prior accepted
producer bundles. Fresh detailed/radio output passes the corrected independent
narrative-to-box oracle; displayed batting/pitching cells match stable-ID machine
rows, including integer outs.404 old text-game pairs (mean/empirical/calibrated)
and8 old pooled/joint/lineup/rated preview pairs retain their bytes.

Fresh `proc_03a5` exited0/605s:902 Rust tests,4 example tests,662 Python tests;
locked/offline build, fmt, strict all-target Clippy and diff checks pass.
All230 executable/config source pins,356 protected tracked paths and14 unchanged
prior input bindings remain intact; only the owned producer recipe changed among
the original15 input/recipe bindings. Native outcome/rules/policy/RNG arithmetic
is unchanged. Three role-membership semantic REDs precede the final hardening;
14 focused Python regressions pass, and Rust serialization checks distinguish
unknown/zero, integer outs, incomplete-game refusal and output-error propagation.

Reproduction/evidence (private, local-only, not durable or licensed for release):
- `python3 -B /tmp/bbsim-season-runner-v1/final/prove.py`;
  `final/proof-results.json`, `final/first/`, `final/repeat/`, `final/comparisons/`,
  `final/legacy/` and raw stdout/stderr;
- `python3 -B /tmp/bbsim-season-runner-v1/final/verify.py`;
  `final/verification/results.json` records every exact cargo/Python/diff command,
  source/binary pins and raw logs;
- `python3 -B /tmp/bbsim-season-runner-v1/refusals.py`;
  `refusal-results.json` explains all40 personnel refusals;
- `final/integrity.json` records exact schedule and receipt/hash rechecks.

These scripts use exclusive evidence directories; a reproduction must choose new
external destinations, never overwrite the retained proof. The original proof
before role hardening, its logs, and the deliberately stopped initial full-suite
run remain preserved; neither is substituted for final changed-source evidence.
Early harness failures (pin tuple type, incomplete synthetic profile metadata,
radio team-label assertion and cast-rank indexing) remain retained as well.

Remaining: explicit exceptional-game replay, personnel-role changes, daily bench/
roster eligibility and rest/fatigue, pinch hitting, defensive replacements,
source24-PA-cell/62-sacrifice-credit reference discrepancy and statistical/player
fidelity. Next approved slice is pinch-hitting preregistration, then defensive
replacements. No transfer, source repair, new dependency, fitting, push or gate waiver.
