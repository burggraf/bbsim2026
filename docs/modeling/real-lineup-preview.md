# P-03-real-lineup: bounded Seattle–Oakland preview

## Authorization and boundaries (preregistered 2026-10-13)

Interview `e4166831-3e06-40fc-b901-3b0102de3315` timed out but preserved
“Build a bounded Seattle–Oakland real-lineup preview”; maintainer explicitly
instructed proceeding with that answer. D-16 authorizes an opt-in native game
and existing-viewer integration using the [audited starting witness](../data-audits/2019-lineup-readiness.md):
`OAK201903200`,2019-03-20,Seattle Away/Oakland Home,DH,TOK01. This is a generated
exhibition using recorded starting personnel, **not replaying that game's results**.
Closed starting roster: nine hitters plus a separate pitcher on each side;
no substitutions, eligible bench/bullpen claims, workload/fatigue or active-roster
inference. Starting pitchers stay all game, an explicitly unrealistic experiment
policy. Player names/IDs/positions are source facts; all players retain identical
pooled joint probabilities. No player fitting/effective hands/park effects,
new source transfer/dependencies/delegation/worktrees/push/release or gate waiver.
Temporary loopback/native browser verification is authorized and must be stopped.
Default fictional/ground and fictional/joint behavior must remain byte-exact.

Base `f048bf4`. Exactly15 owned paths:

- `docs/PROGRESS.md`, this contract, `preview/README.md`;
- `preview/real_lineup.py`, `tests/test_real_lineup.py`;
- `engine/examples/support/real_lineup.rs`, `engine/tests/real_lineup_preview.rs`;
- `engine/examples/support/{preview,joint_contact,game_trace}.rs`;
- `engine/examples/generated_game.rs`;
- `preview/{serve,build_viewer}.py`;
- `tests/{test_live_preview,test_visible_viewer}.py`.

All other tracked files (including pure engine/rules/codec/accounting, samplers'
count input, audits and their tests) remain byte-identical. Artifacts/source rows,
lineup export, logs/captures/fresh parent copies live outside Git in
`/tmp/bbsim-real-lineup/`. No source roster fixture is committed; tests use
synthetic identities with shuffled batting/fielding positions.

## Input and producer contract

Reuse the accepted audit report, SHA256
`5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b`, as the
sole lineup data input. A small standard-library Python exporter verifies its
exact bytes, format, primary witness and DB provenance before exclusively writing
an external UTF-8 narrow TSV (no overwritten outputs, repo outputs or partial
success; recheck report bytes before publication). No new query/importer or DB
writes. Accepted snapshot/supplement hashes remain
`d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528` /
`554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82`.
Retain exact Retrosheet attribution and report SHA. Export repeats byte-exact;
independently compare all20 rows to the audited literal witness.

Format `closed-starting-lineup-v1`: `key\tvalue` header; exactly10 metadata keys
`format,game_id,date,park_id,away_team,home_team,audit_sha256,snapshot_sha256,
supplement_sha256,attribution`, plus20 player keys `Side:slot:position:player_id`
with full bio name as value. Side is Away/Home; slot0 is pitcher/position1;
slots1..9 are batting order, positions2..10 occur once per side, position10 DH.
Strict duplicate/unknown keys, row/column counts, missing/blank/control values,
invalid IDs (lowercase ASCII letters/digits only), slot/position domains,
cross-team identities, unsupported metadata and provenance are errors. Names
are display metadata, never engine identities. TSV has no quoting/escape syntax;
tab/newline/control fields are rejected rather than lossily normalized.
The native parser validates supplied structural/provenance fields, not a claim
that a self-reported digest authenticates arbitrary rows. The Python export and
frozen-input/native reconciliation bind this actual local experiment to the
accepted report. Only an operator startup path, never a URL-supplied file path.

CLI adds `--lineup FILE.tsv` only with `--contact-counts FILE.csv`; absent lineup
retains exact existing CLI/default output. Preserve flag-looking legacy values
and errors. Reject repeated/missing lineup or lineup without joint counts before
game generation. No malformed lineup fallback. Model remains
`pooled-jeffreys-joint-proxy-preview-v1`; real policy version
`station-held-positional-starting-preview-v1`. Sampler boundaries/RNG unchanged:
one joint draw per PA, no rejection/retry/second fielding roll. Shared existing
engine loop gets an explicit initial state; legacy initialization unchanged.

## Positional experimental policy (fixed before tests/games)

Use source fielding positions, **never batting order ordinals**:

| Modeled event | Supplied actor(s) |
| --- | --- |
| Caught K | catcher2 |
| Ordinary ground out | shortstop6 throws to first baseman3 |
| Sole ground muff/ROE | shortstop6 |
| Held fly catch | center fielder8 |
| Held line-drive catch | right fielder9 |
| Held pop-up catch | third baseman5 |
| Residual contact out | no invented fielder or type; Unknown review retained |

These are deterministic coarse assignments, not historical locations or
fielding-skill estimates. Keep existing station-hit/award/HR/held-runner/endings
physics and causal facts, review-qualified statistics and residual unknowns.
DH never fields; pitcher only occupies source position1. Position-bound named
credits/facts must agree and remain eligible before terminal/half boundaries.
Reject roster/pitcher/DH/bench/removal drift rather than invent substitution
position maps; batting cursors/DH first-PA obligation evolve normally.
No pure-engine schema/codec changes. Actual IDs stay in all native snapshots;
named radio calls use a token-exact supplied lookup with stable ID shown alongside
name, no substring replacement or fictitious type/throw/skill. Default calls and
native traces untouched.

## Viewer contract

Optional startup `--lineup` requires configured contact counts and an existing
local file. Existing configured model select adds `real` only when configured;
no query still selects default. Request accepts only configured enum values;
invalid/duplicate/unknown/extra keys fail400 before engine. Foreign Host421,
unknown route404/no-store/CSP/30s generation timeout remain. Native load failure
is visible500, never synthetic/default fallback. Seed/New Game max-u64 wrap,
playback/reset/manual/visibility/terminal stopping remain unchanged.

Native preamble lists TEAM/PLAYER source records, policy/provenance and limitations.
Viewer derives actual team labels/starting roster/names only from this recognized
real policy's complete metadata, escapes all supplied text, exposes20 roster rows
with slot/position/IDs, verbatim provenance/policy and closed-roster/pooled-rate
limits. Calls/recaps and complete native before/event/after bodies remain the
Rust record, not a second simulation. Default/joint pages remain byte-exact when
real mode absent. Do not rename fictional IDs into real teams after simulation.

## Observable acceptance checks

1. Genuine test-first reds: exporter pins/exclusive/input-drift/strict format;
   native initial order/pitcher/DH/positional actors, invalid grains/provenance,
   cross-team IDs/DH fielding/roster drift and opt-in CLI constraints; named calls
   and residual fallback; visible real-team/roster/limits/escaping and request
   enum/default/failure/startup paths. Separate missing interfaces from semantic
   reds; preserve positive controls and raw failures. No assertion weakening.
2. Independent synthetic batting/position permutations; every13 joint outcomes
   × eight masks × three outs × both halves, compatible current batter distinct
   from base runners; direct engine apply/codec roundtrips/credit actor and
   facts checks, terminal/third-out/partial accounting. No inferred position
   from name, lookup order or DH-slot ordinal.
3. Fixed seeds0..1999: real-mode repeated games terminate, continuity/codec and
   available-stat conservation; all personnel stay bound, all fielders match
   position policy, residual unknowns persist. Scores/outcomes/runner physics
   match same-seed fictional joint control after identity/positional relabeling;
   contact bins/sampler boundaries unchanged. No seed/filter/count retuning.
4. Freeze all executable/test sources before full applicable checks: full
   offline-locked Rust build/tests/fmt/strict all-target Clippy, `python3 -Werror
   -m unittest discover -s tests -v`, Werror compilation, established native
   audits/diagnostics and current pinned inputs. Fresh parent/current default
   and fictional-joint whole traces, CLI/wire/pages byte-exact. Run native
   real repeated traces for seeds1/2/3/99 and independently bind20 source rows.
5. Native browser: real selector/team/roster/limits, return default/joint,
   generate/New Game/exact seeds/reset/Next/Previous/Restart/autoplay/Pause/
   visibility/terminal stopping/log scroll/details. All recorded real game
   frames' calls/recaps/native bodies/score/status/bases/statistics match
   independent CLI. Retain failures/limitations/screenshots; close owned browser
   and stop server, confirm socket refusal and unchanged inputs/sources.
6. Parent/author bounded acceptance only: consume complete logs and staged diff;
   exact15-path ownership, working/index/frozen/tested equality, all non-owned
   HEAD bytes, original preregistration and fresh prose links; acceptance-only
   ledger restage preserves non-ledger sections. Local accepted/precommit/
   committed diff equality/clean main; no push, independent review, accuracy,
   complete box, manager readiness, production P-01/P-02/P-03/G1/G2/rights claim.

[Execution](../EXECUTION.md) · [live status](../PROGRESS.md) ·
[joint sampler contract](joint-contact-preview.md)

## Implementation and verification evidence

The original preregistration above is preserved byte-for-byte. Evidence stays
outside Git under `/tmp/bbsim-real-lineup/`; no source fixture or dataset is added.
The pinned export repeats at1205 bytes, SHA256
`92910884f608a9282c6213da8144e02fc1ac73e9b8b5c68e89e8d155fcf1375c`.
Native initial state—not a post-game reskin—uses all20 source identities/order/
pitchers/DH/positions. Shared policy/loop/contact application preserves legacy
physics and one-draw probabilities; source-position actors replace only opt-in
assignments. Pinned export/independent20-row comparison binds the actual input;
a native self-reported digest alone does not authenticate edited TSV rows.

Test-first logs retain interface/semantic failures and positive controls. A Rust
u32/u16 harness comparison, inefficient scaffold run stopped deliberately,
HTTPError/BrokenPipe test-body consumption and an already-green C1 probe are not
misrepresented as implementation reds. Matching-name C1 and boolean/int alias
reds were independently observed. Eight native tests include624 permutation
frames and2,000 repeated paired real games;18 added Python tests cover export,
CLI, visible strict metadata/escaping and configured-only HTTP/startup failures.
`proc_c6df` full check exit0/263s:594 Rust/201 Python, offline-locked build/fmt/
strict all-target Clippy/Werror discovery and compilation, established audits,
1,350 native controls, ensembles and100,000-draw diagnostics. Both complete raw
logs were consumed (stdout1,187 lines/72,975 bytes; stderr382 lines/48,588 bytes).
Fresh HEAD/current default and fictional-joint whole traces/CLI/316 BBE25 records/
304,463 wire bytes/static/live pages remain exact when real configuration absent.
All12 executable/test sources,169 non-owned tracked files and five inputs frozen.

Native Chrome session `bbsim2026-lineup-20261013` verified real/default/joint
selection,20 roster rows/provenance/limits, seed/New Game/max-u64 wrap/reset,
Next/Previous/Restart, autoplay/explicit Pause/manual/visibility/terminal stopping,
scrollable log/full facts/final statistics. Four complete raw captures for fixed
seeds1/2/3/99 contain75/89/76/76 frames (391,688/488,441/400,462/402,156 bytes);
all316 CALL/RECAP/full bodies/score/status/bases/preamble/teams/roster/final supplied
statistics exactly match independent CLI. Actual-source role checks also verify
all316 bodies, no DH fielding and residual unknowns. No pop occurs in these fixed
traces; synthetic permutations/ensemble cover it, without selection retuning.
Six screenshots retain five successful states plus a statistics closing toggle;
source inspection confirms those final details were already open by default.

`browser-infrastructure-and-limitations.txt` records envelope KeyError, origin/
stale-ref/guidance guards, unverified CSS clicks,25s Restart and5s scroll-condition
timeouts; fresh refs/accessible locators and documented native selector scrolling
then established the application postconditions. No scripted click/CDP/browser
shell workaround. Page errors zero; complete server logs retain10 HTTP200 GETs
and10 ten-seconds-later `Request timed out: TimeoutError('timed out')` messages.
No packet-level cause claim. Owned browser closed, `proc_8516` stopped by SIGTERM15;
8765 socket explicitly refused and all inputs/sources/original contract unchanged.

Exact15-path staged-diff review and bounded parent/author acceptance are recorded
separately in the live ledger. Verification alone is not independent review,
calibration, complete box accounting,
active-roster/bench/bullpen proof, production manager readiness or any gate/rights
signoff. See [live status](../PROGRESS.md) for the current checkpoint and next action.
