# F-01 — terminal-hit source coverage

## Preregistration — 2026-10-14, before tests or new cohort measurements

D-20/D-21 authorize this bounded read-only prerequisite after the completed
[synthetic terminal diagnostic](../modeling/terminal-hit-compatibility.md).
Base `407f45705760388ae3388acd647e89a6293c5d8a`, clean main, parent sole writer.
At entry the local `origin/main` ref also equals that base (unlike the prior
receipt); no fetch or push performed here. [PROGRESS](../PROGRESS.md) owns status.

Exactly four owned paths:

- `docs/PROGRESS.md`
- `docs/data-audits/2019-terminal-hit-source.md`
- `audits/2019-terminal-hit-source.sql`
- `tests/test_terminal_hit_source.py`

Preserve all221 other existing tracked blobs and138 prior executable/test
sources. Freeze base, existing blobs, eleven accepted input pins, this original
protocol and evidence recipes before tests/measurement. No engine, dispatcher,
kernel, fitting, model choice, terminal policy, manager, UI, dependencies,
transfers, delegation, worktrees, push or gate signoff. Bottom>=9 refusals stay.

## Inspection findings (not a new coverage measurement)

Read the complete `audits/hit_advancement.py`, `audits/runner_transitions.py`,
`audits/2019-outcomes.sql`, `audits/2019-raw-mechanisms.sql` and
`audits/2019-runner-transitions.sql`; inspected the accepted cached staging
models for events/runners and the global-column definitions of ending/score.
Read-only native DESCRIBE of accepted `events`/`states` confirms available
inning/frame, score start/end, frame/game-end and truncated-frame fields.
Two old temporary event-state model paths are unavailable (ENOENT); do not
claim their implementation was read or fetch replacement source.

The existing outcome export omits inning/frame, scores, game-end/truncation,
and event-level RBI. It does retain frame_end, after-outs/mask, runs and named
before occupants. Raw runner children provide per-runner final endpoints and
run/error/out flags, **not crossing sequence or stop-time coordinates**. Their
JSON order is by baserunner role; fielding/flag sequence IDs order those records,
not runner crossings. The cached staging runner model includes an RBI flag,
but the accepted raw export omits it. RBI is credit evidence, not a clock.

`frame_end_flag` is defined as last event of the half-inning. A truncated frame
can include rain-shortened endings as well as walk-offs. Neither field alone
certifies a walk-off. Existing `terminal_or_out` combines PA!=1, event outs!=0
and frame_end!=false; its729 exclusions are not a count of walk-offs. The
accepted21,813 nonterminal qualified endpoints cannot be terminal exposure.
A literal final Single/Double is a recorded outcome, not a sampled latent hit
value or evidence of a reclassification decision. Missing clocks remain unknown.

## Fixed data, universe and measurements

Use the same eleven accepted offline input pins as the terminal diagnostic
(`/tmp/bbsim-terminal-hit-compatibility/inputs-before.sha256`), including snapshot,
supplement, lineup/manifest, census plus four exports, hit qualification and
comparison. Authenticate unchanged complete source/native/census/lineup/provenance
with `hit_advancement.publish` to a new exclusive external output; regenerated
qualification must equal the accepted report byte-for-byte **before** measuring.
Keep all2,429 games and34,064 literal non-no-play Single/Double candidates, including
the four previously excluded games. Preserve existing SHA256 game-ID splits.

New SQL is a read-only SELECT/CTE over accepted `events`/`states`, with session
external access disabled. It must refuse duplicate keys, orphan/missing state
rows, and conflicting supplied non-NULL start context or score deltas. Missing
new fields remain SQL NULL (CSV empty), never zero, false or a fabricated endpoint.
Do not change old validators/qualification or treat a boundary after-mask as
required. Export every candidate, sorted by event_key, with original result,
inning/frame/base/out/run facts, score start/end, ending flags, after-outs/mask,
and supplied event RBI. Do not label an official winning runner or infer timing.

Report separately by split/result and original exclusion reason:
- PA/non-PA, event-out and frame-end predicates and their overlaps;
- Bottom inning>=9; game-end; truncated-frame; known/missing scores and after-mask;
- arithmetic lead crossings: Bottom>=9, home start<=away start, event runs greater
  than the known deficit. Keep game-end/score-end confirmation separately, not as
  a hidden filter. Missing required arithmetic inputs yield unknown;
- on those arithmetic crossings, named complete endpoint coverage, all original
  qualification reasons, counts of Home children versus event runs, possible
  scorer origins and the recorded hit-value ceilings. These remain endpoint
  diagnostics, not authenticated unaided causes, crossing order or legal replay.

No fitted probabilities, rates used as policy, thresholds, sampled-versus-credited
conversion, clipping, resampling, guard removal or station fallback. No attempt
to reconstruct unobserved continuing-play endpoints after a stopped game.

## Observable acceptance, fixed before implementation

1. Test-first native SQL with synthetic fixtures: all masks/outs, both hit types,
   ordinary/early/extra halves, safe/out/ending/unknown fields; supplied score/context
   conflicts and duplicate/orphan/missing joins refuse; NULLs preserved; no-play/
   noncandidate exclusion; exact deterministic event rows; no source mutation.
   Missing-query/interface failures are distinct from semantic failures.
2. Two external native exports byte-identical. Independently join the already
   authenticated outcome/raw CSVs to every exported key; match all shared fields,
   candidate count/splits, original729 first-reason exclusions and reason overlaps.
   Independently enumerate arithmetic lead crossing and endpoint/ceiling diagnostics
   in an external frozen recipe, without changing production qualification.
3. Pin/recheck completed evidence, inputs, source/query/recipe bytes; never pin
   an active writer's own logs. Keep all failures/raw logs externally, synthetic
   committed fixtures only. Preserve this protocol as an exact prefix.
4. Stage new files and check whitespace before final freeze. Fresh full established
   offline locked Rust build/test/fmt/all-target Clippy, Werror Python discovery/
   compilation, inherited native/source/property/preview and exact four-mode bytes.
   Inspect raw outputs and full diff, verify owned/protected/index/work/tested
   equality and prose links. Parent-author acceptance only; local checkpoint with
   exact postverification. No independent review or G1/G2/G3/manager approval.

The deliverable is a source capability/coverage report and reproducible query,
not a terminal resolver. The next implementation, if justified, needs a separate
preregistered contract for explicit winning-crossing/stop-time facts and the
sampled-versus-credited hit-value distinction.

## Test-first observations

`proc_dacd` exits1: missing query FileNotFoundError; initial interface trace
retained externally (only first95 lines inspected at that checkpoint).
`proc_ef7a` exits1 after1s: minimal literal SELECT interface runs;14 assertion
failures across six methods expose absent ordering/filtering/join/context/score
guards. Four methods already pass, not regression credit. Complete semantic log
read. Before guards, corrected Cartesian Top fixture score endpoints to put its
run on the Away side; no assertion relaxed or source validator changed.
`proc_cc02` exits0/1s: all10 methods pass,288 Cartesian contexts. Additional NULL
projection assertions subsequently added as already-green coverage; fresh final
verification was pending at that checkpoint. No historical engine behavior changed.

## Authenticated source observations

`proc_22c4` final focused run exits0/1s,10 methods/288 Cartesian contexts,
including the added NULL assertions. `proc_be0d` exits0/44s:
`python3 -Werror /tmp/bbsim-terminal-hit-source/measure.py`. Unchanged complete
source/native/census/lineup/provenance validation runs first; regenerated
qualification is byte-identical to accepted `6551fe2e...`. Every34,064 exported
candidate key and all shared event/context fields reconcile against accepted
raw/outcome CSVs; original first/all-reason counts exactly retained. Separate
native aggregate SQL (not importing the new query) agrees on every result cell.
Two2,776,154-byte native exports are byte-identical, SHA256
`917364ba2bbb9e16b7d006b2e756d5e0041e6f721fd8a5d20902b30afe18f46e`.
Report61,420 bytes SHA256
`474c3659e9168100a902878aeeadcd973c5cbdca5cafcd69016e3432a31ac1bc`.
External `measure.py`, `summarize.py`, reports/CSV/raw logs and native controls
remain under `/tmp/bbsim-terminal-hit-source/` (may expire); no real rows committed.

| Recorded result | Candidates | With out | Frame end | Game end | Truncated | Bottom>=9 | Arithmetic lead crossing |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Single | 25,947 | 484 | 301 | 81 | 80 | 977 | 79 |
| Double | 8,117 | 150 | 92 | 21 | 15 | 284 | 15 |
| Combined | 34,064 | 634 | 393 | 102 | 95 | 1,261 | 94 |

These columns overlap, not a partition. All candidate PA flags are1.
The original729 `terminal_or_out` exclusions partition as336 out/non-frame-end,
95 zero-out/frame-end and298 out/frame-end;33,335 meet neither condition.
All393 frame-end after-base masks are missing; every other exported field is
known on these candidates. Do not insert an empty base mask or regard this
boundary convention as an impossible observed geometry. Ordinary qualification
correctly continues to exclude them without changing its predicates.

Splits (Single/Double): training20,733/6,510; reserved5,162/1,595;
excluded52/12. Arithmetic crossings: training57/14; reserved20/1; excluded2/0.
All94 have game_end/frame_end/truncated flags true, known final home lead,
complete correctly named endpoints, zero event outs and missing after-base mask.
83 have one Home child and one counted run;11 have two and two. On each, supplied
counted runs exactly equal known deficit+1 (Single71 one/8 two; Double12 one/3 two).
This is counted-score evidence, **not** proof that no additional physical runner
crossed or what every runner's position was at the stopping instant.

The other eight game-ending hits include seven with an out and one truncated
Bottom8 zero-out Single; this audit does not invent a rain/ending cause from
those flags. All94 arithmetic crossings fail original `terminal_or_out` and
`unsupported_geometry` (their missing boundary mask prevents the old full-mask
comparison);38 also fail notation, with one of those additionally carrying
fielding/runner-effect reasons. Failing the nonterminal proxy is not an invalid
source event and never grounds for deleting a row or retuning the filter.

Possible scorer origins among Doubles: six First-only, six Second-only,
two First+Second and one First+Third. The latter has recorded two needed/counted
runs and an empty fielding-credit list with known safe child endpoints; it mixes
possible origin ceilings. It is **not** evidence that Third was the winning
runner or that Double is an illegal official credit. No endpoint/notation order
is promoted to a crossing clock. No crossing identity, crossing timestamp,
stop-time vector, latent sampled hit value or reclassification decision is
present in this inspected export/schema slice; no claim about all upstream or
unfetched records is made. No source-cause/NoMisplay certificate is inferred.

A bounded reread of the cached2019 primary rulebook Rule9.06(a)–(g), canonical
`rules::validate_hit_value`/`rules::ending` and complete `sole_safe_hit::validate`
confirms another required distinction:9.06(f)'s comment allows the batter to
receive credit for a base touched in the natural course of play moments after
the winning run scores. Therefore a universal instant-of-winning-run clip is
not a correct substitute for richer terminal facts. Any future seam must keep
counted runs, winning-runner advance, authorized natural-course batter touches,
and physical continuation/stop facts distinct; this audit selects no such policy.

## Fresh verification at publication (historical acceptance-pending checkpoint)

Final140 executable/query/test sources frozen after staged new-file whitespace;
all138 prior sources and221 protected paths unchanged. `proc_2921` exits0/270s:
`bash /tmp/bbsim-terminal-hit-source/final/full-checks.sh`.
637 Rust tests/37 unfiltered passing summaries/zero failures;391 Werror Python
in30.159s; locked offline generated-game build, fmt/all-target Clippy `-D warnings`,
Werror compilation, native1,350, source69 metrics/2,429 games/222,047 raw events,
five fixtures/54 metrics/27 shared-upstream ER/54 external boxes,2,000 preview
seeds/100,000 draws/source export repeats, four legacy traces,16 four-mode records
and four raw UTF-8 pages exact. The inherited script's echo says219 non-owned;
its actual manifest contains and verifies221; the stale label is not a new count.
No executable/query/test or frozen recipe changed after this run.

Full stdout1,958 lines/104,272 bytes SHA256
`994f5a2a5cc068d804f09d57a3054310fb87bc4b3464105e6c26e6697cad5577`;
stderr586 lines/84,488 bytes SHA256
`fb21ac1f4a7295a4a5561d21af9284965fbfe0246a760127858382fd0faeee2c`.
Complete raw full logs are preserved; prior large tool reads were truncated and
**do not establish complete inspection**. This continuation actually consumed
stderr331–586 and all inherited393-line diagnostic stdout/two-line stderr.
All384 diagnostic rows independently reconcile960 candidates/1,920 orderings;
not new source probabilities. The successful suite includes expected negative
controls; complete raw-log and staged-diff consumption remains an acceptance
barrier. An initial external staging
helper falsely parsed a fenced Python call in unchanged replay documentation as
a Markdown link; raw `stage-v1.stderr` retained. Reused the established fence-aware
prose scanner,538 links pass. No product source or assertion changed for that
helper correction.38 completed measurement artifacts pinned, including historical
document snapshots matching the exact premeasurement freeze; no active self-logs.

Implementation/measurement/full verification pass, but parent-author acceptance
is **pending complete inspection**; there is no independent review, model
selection, terminal resolver or G1/G2/G3/manager signoff. Final documentary edits,
exact staged tree/all225 blobs and completed evidence are checked separately.
Publication checkpoint authorization — 2026-10-14: the maintainer explicitly
requests “commit and push -- then what's next?”. This supersedes the original
no-push scope only for publishing these four paths to the configured origin.
Commit this as an **in-progress checkpoint**, not completed audit acceptance or
a release. Full raw-log/diff inspection remains pending; task status stays active.
The resulting commit is identifiable in Git history; actual clean-state and
remote verification belong in external `publication-post.json`, not a promise
that a push succeeded. Committed report/query/synthetic tests are durable only
after publication is verified; large raw evidence remains external and may expire.

**Next bounded direction:** preregister explicit terminal winning-runner and
natural-course batter-touch evidence before changing canonical hit validation;
inspect payload/codec/callers first. Keep physical continuation, counted runs,
scorer credit and sampled hit type separate. The audit cannot supply a clock or
select an automatic terminal policy; both kernel/dispatcher refusals stay.

## Completion inspection and bounded acceptance — 2026-10-14

Published code checkpoint `804a72d7b08d02daaa66bee1669cb8d925916d18` is verified
on origin/main with clean225-blob committed/index/work equality in external
`publication-post.json`. It was deliberately published incomplete; the
acceptance-pending descriptions above record that historical checkpoint.

Continuation consumed **178 lossless approximately1,800-character fragments**:
58 full stdout,47 full stderr,31 missing-interface stderr,9 semantic stderr and
33 complete published-diff fragments. Exact reassembly/hashes verified in
`continuation-inspection/manifest.json`; actual consumption ranges in
`continuation-inspection/consumed.json`. This is actual reading, not credit for
fragment creation, hashes or an extracted status index. Inherited diagnostic
stdout393 lines/stderr2 lines were already fully consumed. All expected generation
failures/HTTP500/400 controls and the stale219-count echo are accounted for.
The missing-query run has32 interface/subtest errors, distinct from the14
semantic assertions across six methods; four methods were already green.
No assertion, validator, executable, query or evidence recipe changed.

`continuation-guard-probe.json` additionally records native synthetic duplicate/
orphan refusals with **no hit candidates**, plus a Single duplicate control;
all refuse with the intended join error. Already-green characterization, not a
new regression claim. Full query/tests and the external measurement caller were
read; complete consumption of every inherited source file is **not** certified
by this acceptance. In particular, the original protocol's blanket complete-read
wording cannot be substantiated by truncated historical responses; preserve its
exact bytes but qualify that inspection claim here. Frozen inherited validators
and authenticated/reconciled exports provide the separately stated evidence.

All140 tested sources,138 prior sources,221 original protected blobs,eleven
inputs,six recipes,38 completed measurement pins and52 completed-full pins
rechecked unchanged after publication. Only this document and PROGRESS change
for completion; continuation protects all223 other blobs. Original preregistration
prefix remains exact. Full executable verification is the unchanged `proc_2921`
run above; publication also reran10 focused tests/1.823s and Werror compilation.
No replacement smoke suite, regenerated outcomes or claim of independent review.

**Parent-author acceptance: GO for this descriptive F-01 source-audit deliverable
only.** No observed query/test defect; dataset timing/stop-position/cause gaps
remain reported. No model selection, terminal policy, canonical expansion,
producer integration, source transfer, release or G1/G2/G3/manager approval.
Completion doc commit/exact tree/225 committed-index-work blobs/clean-state
postverification are recorded in external `completion-post.json`; do not infer
a further push. Next scope requires its own preregistration before tests/code.
