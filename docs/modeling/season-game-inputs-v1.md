# P-02-season-game-inputs-v1 — strict generic historical game inputs

## Acceptance fixed before implementation or new game measurements

Base `d7b6cde` completes the [readiness audit](../data-audits/2019-season-replay-readiness.md).
The maintainer approved audit → headless historical-lineup season runner → pinch
hitting → defensive replacements. This is the first runtime prerequisite, not a
whole-season runner or statistical-fidelity signoff. Reuse the existing engine,
accounting, full-game simulation, manager, rating arithmetic and source validators.
No new dependencies/downloads, source repair, probability/RNG/default-policy change,
calibration experiment, UI, push or S/G gate waiver.

Observable checks:

1. Keep the existing pinned lineup v1 / cast v1-v2 commands, strict refusals and
   fixed-fixture outputs unchanged. Introduce explicit generic lineup v2 and cast
   v3 formats rather than silently interpreting a damaged old file as generic.
   Generic and legacy pairs cannot be mixed. Validate matching game IDs, starters,
   source/basis metadata, complete declared cast sizes and all role/identity sets.
2. Generic lineups carry game/date/park/team metadata, DH choice and all starting
   identities/positions. DH has ten distinct players per side (pitcher slot0);
   non-DH has nine, with the pitcher occupying exactly one batting slot. No DH
   player/extra slot0 in non-DH; no inferred/duplicated actors. Correct metadata
   drives actual team labels, narration and scores, not Seattle–Oakland literals.
3. Source-bound offline export consumes the already accepted full lineup native
   export/report, opportunity report, event CSV and accepted readiness report
   (the older lineup export lacks shortened-game flags). Verify their hashes, full
   lineup/person/appearance and opportunity reconciliation; reuse existing
   bullpen/workload inventory. Select any supported game by stable ID, not ordinal.
   No SQL/network transfer is needed. Keep all data outside Git and publish only
   exclusive, source-reread-checked files with a receipt after both inputs exist.
   Failed/partial destinations must not be reused or claimed accepted.
4. The historical closed cast includes every recorded position1 actor for each
   side, starter first, relief order by the existing descending role-mean/ID rule.
   Use the existing empirical outing budgets and manager stream, without new
   performance hooks, stamina caps or actual-result intervention. The workload
   audit's eligible-game role pools are not calibration training folds or measured
   fatigue. A missing selected workload role refuses; no invented workload prior.
5. Ratings stay the same fixed200 full-selected-season participant counts. A
   source-observed player with no batting events gets an **explicit all-zero**
   batting vector (existing neutral prior), never a silent runtime fallback.
   Non-DH needs batting vectors for incoming cast pitchers too. Missing/invalid
   actors, lookup failures and malformed count sets refuse. Unknown profile data
   is not zero data. No other full-season or held-out fitting is authorized.
6. Existing no-DH pitcher + batting-slot substitution must conserve the cursor,
   not consume PA/outcome RNG or reenter a removed player. Incoming pitcher's own
   batter and pitcher profiles must be used in subsequent matchups. A recorded
   nonstarter already in a fielding/batting role, reversed/unknown appearances,
   unsupported result flags or absent workload cannot be quietly excluded to
   make a game run; refuse and identify the unsupported input. Pinch hitters,
   defensive repositioning/double switches and eligibility stay separate work.
7. Tests first: shuffled generic DH/non-DH structures, metadata mismatch,
   duplicate/missing/extra actors, changed source input, unknown keys, bad counts,
   zero versus missing profiles, complete-cast truncation, legal pitcher batting
   replacement and canonical accounting. Preserve old schema rejection tests.
8. Actual-source proof uses: earliest supported DH game outside the original
   SEA–OAK pairing; earliest supported non-DH game; and earliest supported non-DH
   game with a full-season-zero starting batter, if its complete closed cast is
   representable under the stated restrictions. Run seeds1–200 each, requiring
   completion, legal contiguous canonical accounting, correct actor/rating use
   and repeatability. Unsupported inputs are explicit findings, not hidden seed
   or game exclusions. At least one incoming pitcher must bat in the non-DH
   proof; certify opportunity rates only later, after pinch hitting exists.
   Independently recompute each consumed ten-way vector from the supplied actor
   counts (absolute floating-point tolerance1e-12; no outcome-distribution fitting).
   Reuse the existing integer narrative→box oracle unchanged in credit assertions;
   adapt only team/start metadata and insertion of entering pitchers at their
   non-DH batting slot. Keep unknown RBI/ER unknown; this is not official scoring.
9. Freeze existing native commands before edits. Compare unchanged legacy detailed
   and radio output across mean/empirical casts for seeds1–100 (plus seed67), and
   retain broader old-preview controls. Run all applicable Rust/Python suites,
   locked/offline example builds, fmt, strict all-target Clippy and diff checks.
   Preserve failures/raw outputs and input/source pins. Review findings against
   actual callers; do not retune thresholds or replace the suite with smoke tests.

Expected touched boundaries: `preview/real_lineup.py`, `preview/player_ratings.py`,
`preview/pitching_cast.py`, a small `preview/game_inputs.py` exporter;
`engine/examples/support/real_lineup.rs`, `full_manager.rs`, `engine/examples/text_game.rs`;
focused Python/Rust tests plus this contract and the progress ledger. Widen scope
only for a demonstrated caller/validation need, and record it before the change.
No second simulation engine or general roster-management framework.

## Offline reproduction

Restore the accepted cached artifacts documented by the readiness audit first;
these private `/tmp` paths are not durable distribution artifacts. The producer
requires a **nonexistent child directory outside the repository**, not an existing
`mktemp -d` directory itself. It publishes `lineup.tsv`, `cast.tsv`, then the final
`receipt.json`; an interrupted partial directory is not accepted or reusable.

```sh
parent=$(mktemp -d /tmp/bbsim-game-inputs.XXXXXX)
python3 -B -m preview.game_inputs \
  /tmp/bbsim-season-readiness-v1/first.lineup.json \
  /tmp/bbsim-season-readiness-v1/first.lineup.native.csv \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-pitcher-workload/report-null-fixed-first.events.csv \
  /tmp/bbsim-season-readiness-v1/first.json \
  CIN201903280 "$parent/game"
cargo run --offline --locked --manifest-path engine/Cargo.toml --example text_game -- \
  --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup "$parent/game/lineup.tsv" --cast "$parent/game/cast.tsv" \
  --seed 1 --format radio
```

For code regression checks, first build the examples, then run the Rust suite,
example tests and Python discovery (including `tests/test_season_game_inputs.py`),
fmt, strict all-target Clippy and `git diff --check`. Raw source/game cohort proof
is additional to those suites, not a replacement.

## Execution note: independent oracle repair (before complete cohorts)

The first run stopped at `KCA201903280`, seed21. Preserved detailed/radio
outputs show Palka reaches against Keller (play51), McCarthy enters52,
Anderson replaces Palka on a force choice53 and scores56. The engine correctly
retains Keller's inherited run responsibility; the older narrative oracle
incorrectly assigned every reaching batter to the current pitcher. Its expected
KellerR2 disagreed with the correctR3. This is an oracle defect, not a new model
failure or a reason to loosen assertions. A separately preserved oracle copy now
transfers ownership for that explicitly narrated force replacement. The full
saved box passes; deliberately changing KellerR3 toR2 still fails. Original
oracle/raw seed21 failure retained in `/tmp/bbsim-season-game-inputs-v1/`;
`box-oracle-repair.json` pins both versions and the regression. Restart the same
registered games/seeds in `cohort-v2/`; all integer credit assertions remain exact.
No engine or probability/policy change was made for this repair.

## Approved regular-full-game policy — 2026-10-18

The maintainer explicitly chooses ordinary full simulations for abnormal played
schedule games, including shortened and suspended games. This supersedes the
historical-result refusal below for simulation inputs, not source auditing or
exact historical replay. Start once from the recorded starting lineup; use normal
2019 nine-inning/extra-inning/walk-off endings. Never stop at historical duration,
resume at the suspension point, force a historical tie/forfeit/result, or manufacture
an unplayed schedule slot. Retain original source flags unchanged in the audit.

Acceptance before code: valid supported lineup/cast inputs are identical under
shortened/suspended/tied/forfeit flags, historical durations and absent/ambiguous
result records; no source mutation. Unknown game IDs, unsupported personnel and
source/publication guards still refuse. Export receipts and season manifests label
`simulation_policy: regular-full-game-v1`; no engine/RNG/rating/default CLI byte
changes. Run regression tests and the full Python suite; rebuild all cached game
inputs to distinguish result-policy refusals from genuine personnel refusals, and
run newly admitted inputs through native completion/accounting checks. Preserve
prior proofs as historical evidence; do not overwrite their counts.

### Verified policy correction

All 2,429 played IDs rerun at master seed 1: **2,384 complete / 45 input_refused /
zero native failures**, intentional CLI exit 2, `complete_season=false`.
Nine abnormal games newly completed and repeated byte-identically; each defense
recorded at least eight innings, and replacing their historical result metadata
with ordinary-result metadata produced identical input bundles. All 2,375 formerly
completed games retained identical lineup/cast/stdout/stderr bytes (9,500 artifact
pairs); all 2,430 schedule rows and five source hashes stayed unchanged.

Remaining refusals are **40 pitching identity/eligibility conflicts, four unsupported
starting lineups and one cast with unknown appearance/workload evidence**. The five
newly exposed data refusals are DET201905190, CHA201905270, NYN201906130,
BOS201908070 (lineups), and ATL201906110 (cast). These are no longer refusals merely
because the historical games were abnormal; this policy does not invent personnel.
Independent accumulation from raw native rows reconciles every team/player split,
unknown propagation and league PA/BF/runs/wins/losses across all 30 teams.

Fresh **27 focused tests / 678 full Python tests** pass with ResourceWarnings as
errors. Test-first failures, complete raw logs, source/binary pins, proof and
verification are retained in private/local `/tmp/bbsim-regular-game-policy-v1/`.
No Rust source, native binary, outcome RNG, manager or probability arithmetic
changed; Rust suites were not rerun for this Python-only policy correction.
This is not complete-season/fidelity/official-stat acceptance or gate signoff.

## Verified bounded results

Completed on base `d7b6cde` without changing probability arithmetic, manager policy,
RNG streams, source data or the legacy command defaults. The authenticated exporter
reconciled the complete cached native lineup/opportunity inventories before
selection and reread all13input/recipe pins before exclusive publication. A public
CLI repeat produced byte-identical lineup/cast TSVs. Accepted source/report hashes
and raw outputs are retained privately in `/tmp/bbsim-season-game-inputs-v1/`.

| Registered selection | Cast Away/Home | Seeds completed | Consumed PA vectors | Incoming pitcher batting draws | Zero-count batter draws |
| --- | --- | --- | --- | --- | --- |
| `KCA201903280` — CHA at KCA, DH | 4/6 | 1–200 | 15,838 | not applicable | 0 |
| `CIN201903280` — PIT at CIN, non-DH | 3/6 | 1–200 | 15,279 | 602 | 473 |
| `SFN201904060` — TBA at SFN, cold-starter non-DH | 4/6 | 1–200 | 15,475 | 1,032 | 858 |

All600games completed with the existing canonical legality/continuity/conservation
and accounting checks. All600 detailed→radio boxes passed the independent integer
reconstruction (the documented inherited-FC oracle correction retained). All46,592
consumed vectors matched the actual batter/current pitcher counts within1e-12;
all raw-word CDF parent choices matched. The cold starter `stanr002` actually batted
in107recorded PA draws in the200-game cold cohort; cold input vectors were explicit
all-zero observations, not runtime missing-data fallbacks. Tactical IBB overrides
have no modeled draw and are not included in these draw counts. Both output formats repeat
exactly at seed67 for each fixture. These are controlled-game proofs, **not**
registered player opportunity/rate tolerances or predictive validation.

Legacy mean/empirical detailed/radio output is byte-identical across400seed-format
pairs, plus4calibrated-v2 pairs and8pooled/joint/lineup/rated preview pairs. A
same-game four-pair schema matrix passes both matching versions and rejects both
legacy/generic mixtures at the explicit pair gate before stdout. Generic metadata,
hyphenated source IDs, arbitrary pitcher batting slot, missing versus explicit-zero
profiles, malformed/truncated casts, exceptional-game refusal, changed-source
publication and exclusive-output behavior have regression coverage.

Fresh full commands (raw logs in `verification/`, exit0 each):

- `cargo build --offline --locked --manifest-path engine/Cargo.toml --examples`
- `cargo test --offline --locked --manifest-path engine/Cargo.toml` —899tests,
  63zero-failure summaries, no ignored tests.
- `cargo test --offline --locked --manifest-path engine/Cargo.toml --examples`
  —4tests, no ignored tests.
- `python3 -B -Werror::ResourceWarning -m unittest discover -s tests -v`
  —648tests.
- `cargo fmt --manifest-path engine/Cargo.toml --check`
- `cargo clippy --offline --locked --manifest-path engine/Cargo.toml --all-targets -- -D warnings`
- `git diff --check`

All226 executable-source/config pins stayed unchanged through verification and
cohorts; all non-owned prior code and six frozen old input artifacts stayed exact.
Evidence: `source-results.json`, `verification/results.json`, `cohort-v2/results.json`,
`box-oracle-repair.json`, source/binary pins and raw outputs. Initial red/harness
failures remain preserved. Inspection was direct author review, not a delegated
independent model review. No remote query, push, redistribution or gate signoff.
The code checkpoint is local; `/tmp` evidence is **not remotely durable**.

## Limits carried forward

This does not resolve the source's62SH/SF reference-credit gaps, certify all
AB/RBI/ER/IP, implement daily eligibility/rest or make suspensions/shortened games
ordinary completed nine-inning fixtures. Full-season manifest/orchestration,
cumulative stable-ID/team-split stats, explicit exception policy and reference
certification follow. Structural TSV loading is not authentication of arbitrarily
edited files; the source-bound producer authenticates complete cached artifacts.
Completing controlled games is not player opportunity/rate
fidelity, autonomous management, or predictive accuracy.
