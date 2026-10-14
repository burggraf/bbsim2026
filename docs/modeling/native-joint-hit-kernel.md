# P-01 — standalone native joint-hit kernel prerequisite

## Preregistration — 2026-10-13, before new tests/code/measurements

D-20 requires mechanism solidity before managerial integration; D-21 permits
bounded continuation without routine testing pauses. The descriptive hit audit
`95150b6` and retrospective [comparison](hit-advancement-comparison.md), code
`f3a11d7`/verified handoff `e1ccd7c`, are prerequisites, not model/generator approval.
[PROGRESS](../PROGRESS.md) remains the only live status ledger.

Base `e1ccd7cf98ee3b4811997fc8d964838d55e22df9`, clean main, parent sole writer.
Exactly four owned paths:

- `docs/PROGRESS.md`
- `docs/modeling/native-joint-hit-kernel.md`
- `engine/examples/support/joint_hit.rs`
- `engine/tests/native_joint_hit_kernel.rs`

Preserve210 other tracked paths and all131 prior executable/test sources; freeze
base/main/owned/protected/source/protocol bytes before tests. Final source freeze133.
No dependencies, source transfer/extraction/fitting, delegation/worktrees, push,
UI/CLI/manager/ratings/canonical-engine changes or existing producer integration.
No new real-data measurement. All fixtures/counts/participants are synthetic.

## Purpose and ownership

Provide an isolated Rust example-support kernel, exercised directly by a native
integration test through the existing path-module convention. It owns a joint
conditional hit endpoint sample and constructs a canonical engine transition;
Python and a browser own neither outcomes nor game state. It is **not called by
any game producer**. Source-file independence and existing mode byte equality are
acceptance requirements. Actual-data loader/export and a complete-game opt-in
producer require later separate protocols; this slice provides neither.

Retain all three fixed comparison models as explicit experiment variants:
uniform-legal, pooled-mask Jeffreys and exact-base-out Jeffreys. No winner/default
selected by retrospective scores. A complete training-only table has48 typed
contexts (hit1/2 × before mask0..7 × outs0..2), including zero-exposure cells,
and full structural support, not just observed categories. A pure table constructor
checks shape/unique contexts/count domains/conservation; it does **not authenticate
source provenance**, infer missing counts or certify a real cohort.

## Fixed support, probability arithmetic and supplied-word sampler

Use the same full ordered safe joint geometry as the accepted comparison: batter
ends exactly at hit value; occupied runners appear once in ascending origin order,
hold/advance without backwards motion, collision or overtaking; non-Home targets
strictly increase; Home may repeat only as a suffix. Empty bases are deterministic.
Independently enumerate the support in tests;40 value-mask vectors/120 context
entries. Typed invalid contexts and count shape/aggregation errors fail closed.

Counts are u64 with checked aggregation; exact half-prior probabilities use u128
`(2*n_j+1)/(2*N+K)`, including unobserved structural categories. Pool only training
counts across outs for the pooled variant. Zero-training becomes uniform; no
hierarchy/parameter/player/trajectory/arm/park effects or new qualification.

The kernel consumes a supplied u64 word, never a global RNG or second hidden
roll. Use its high53 bits as integer draw0..2^53-1. Compute each strict-less-than
threshold as `ceil(exact cumulative numerator * 2^53 / denominator)` in checked
u128 arithmetic; final threshold exactly2^53. Expose exact probability ratios
and integer grid masses for testing. Quantization error per category is less
than one grid unit; a category must occupy at least one grid point. Reject
collapsed/unrepresentable intervals rather than clipping, removing categories,
retuning counts, borrowing probabilities or retrying a draw. All arithmetic
errors are explicit. This versioned sampler is not a compatibility promise with
an earlier floating-point sampler; existing producers/RNG streams stay unchanged.

## Explicit modeled projection and terminal boundary

A projection requires an explicitly named modeled unaided-hit/NoMisplay assumption,
not a fact inferred from source endpoints. Bind actual before-state next batter,
defensive pitcher, occupied identities and carried responsible pitchers. Construct
one PA, one hit plus exact Single/Double modifier, no outs/personnel effects,
runner moves including holds, and literal Run/RBI units for each modeled scorer.
Use the unchanged canonical `apply`; no secondary state model or permissive bypass.
Codec roundtrip and independent accounting must agree with the complete transition.
Nonstation/held movements may retain unknown earned runs; do not claim full boxes.

This prerequisite deliberately **refuses every bottom-half inning>=9** projection
before sampling. Terminal hit-value/run chronology is not supplied by the accepted
endpoint model; do not silently cap/reclassify a double, add winning runs, resample,
fallback to station advancement or broaden engine rules to make it legal. Top
halves and bottom innings1..8 remain testable, including nonterminal extra-inning
top frames. Over/invalid states, context mismatches and actor/charge errors refuse
without mutating caller state. A future playable integration needs its own explicit
terminal projection policy; this refusal is a temporary scope boundary, not an
accurate strategy, a source filter or a complete-game implementation.

## Observable acceptance, fixed before tests

1. Separate missing interfaces, genuine semantic reds and already-green controls.
   Independent support/probability arithmetic; exact literal uniform/pooled/base-out
   probabilities and training isolation. All48 contexts/full support/zero/sparse
   exposure/shape/duplicates/invalid contexts/checked u64 aggregation/u128 thresholds.
2. Exact grid count conservation, positivity and strict edge controls (0, edge-1,
   edge, max word), deterministic repeats, unrepresentable tiny intervals rejected.
   Quantization discrepancy strictly less than1/2^53 per category; no empirical
   performance threshold or source model selection.
3. Synthetic supplied-word frequency diagnostic only: fixed SplitMix64-v1 seed19,
   100,000 words per model/context, independent expected integer grid masses; each
   observed category within6*sqrt(N*p*(1-p))+1 of expectation. Fixed before execution;
   no seed/count/tolerance search. This is sampler implementation testing, not
   empirical calibration/generalization or evidence that source endpoints are causes.
4. Every legal synthetic projection across halves/masks/outs/values/support with DH
   and no-DH, actual batting cursor, current and inherited charges: continuity,
   legal bases/one PA/one BF/no outs/score/Run/RBI/hit-value credits, codec and
   independently folded accounting. Explicit unsupported terminal/unknown/invalid
   controls; known canonical contradictions remain errors, not fabricated history.
5. Freeze133 executable/test sources; full offline locked Rust build/test, fmt,
   all-target Clippy `-D warnings`, Werror Python discovery/compilation and **all**
   inherited source/native/property/preview controls. Four-mode16 complete records/
   four rated live pages remain byte-identical. Consume complete raw outputs with
   genuinely nontruncating reads; preserve failures and source hashes.
6. Exact four paths/base/main/work-index-tested/protected/input/protocol/links/tree/
   full-index diff before bounded author acceptance/local checkpoint; clean main
   afterward. No independent-review, empirical accuracy, production model/manager,
   G1/G2/G3, availability/fatigue or public redistribution-clearance claim.

Completion requires honest sampler/projection/refusal evidence, not an improving
league score or a generated complete game. No consequential maintainer-input
blocker identified for this isolated prerequisite; consequential later integration
or release choices remain outside this contract.

## Initial execution evidence — preregistration frozen before tests

`proc_5eed` exit0/8s, `python3 -Werror /tmp/bbsim-native-joint-hit-kernel/freeze.py`:
base/main/exact four-path scope/210 protected HEAD-work/131 prior sources/ten
inherited accepted inputs/original protocol frozen; new Rust module/test absent.
Complete short stdout consumed, stderr empty. Read actual baseline and joint
samplers/shared game loop/real-lineup caller/canonical hit-value and ending guards;
legacy station walk-off policy is not reused or silently modified by this kernel.
Two independent synthetic support/type tests added only after freeze.
`proc_9a3d` records cargo exit101 because `joint_hit.rs` is absent; interface only,
full short raw stderr/empty stdout consumed,131 prior/210 protected pins exact.
Shape-only empty-vector API then `proc_0458` witnesses two genuine assertions:
invalid value/mask accepted and empty support instead of literal `[[1]]`.
Ordered-construction support implemented only afterward; `proc_e248` exit0/5s
reports two methods/fmt/diff and preserved131 prior/210 protected pins. Four new
training-table/exact-probability methods added before their APIs; this later source
is not certified by the earlier two-method green or a final source freeze.
Current table/probability interface run pending. Full verification/acceptance remain
unperformed. No real data measured, production caller or source cause inferred.

## Test-first native implementation evidence — before diagnostic measurement

- Table APIs: `proc_df8e` missing imports; `proc_19ba` four unexpected-Err positive
  failures/two controls; `proc_5e64` three numeric/domain assertions/three controls;
  `proc_b9f1` six methods green. Permutation/isolation was already green on empty
  scaffolding; not misrepresented as a newly repaired defect.
- Grid APIs: `proc_4910`16 missing methods; `proc_cae2` three genuine grid/positive
  interval assertions/six controls; `proc_3661` nine green. Fixed frequency method
  compiled but deliberately skipped, not measured before final source freeze.
- Projection: `proc_043f` missing APIs; `proc_fae0` two positive errors/ten controls.
  First implementation `proc_f4f9`11 pass/one `InvalidState`, clippy not reached.
  `proc_34d5` wrong exact filter ran zero methods; no verification credit. Correct
  `proc_3b36` reproduces Single/mask1/out0/Top/no-DH/cursor0/current charge/[1,2],
  with runner a1 also queued as next batter. Existing canonical validation of the
  after-snapshot refuses this roster conflict. Only the synthetic occupied-ID
  recipe changed cursor+1→cursor+3, still exhausting all9 batting cursors/halves/
  DH roles/charges/support; all8640 assertions preserved. No engine relaxation.
- `proc_7d9b`12 focused pass, then `clippy::int_plus_one` at independent oracle21;
  exact equivalent `target>i` applied before any frequency diagnostic. All48-context
  independent exact-ratio comparison added as already-green coverage.
- `proc_e0b3` exit0/10s:13 focused methods/8640 canonical projections/codec/independent
  accounting/all65527 bottom innings>=9 refused, fmt/all-target Clippy `-D warnings`,
  131 prior/210 protected pins/diff pass. One frequency method skipped. Complete
  short raw logs consumed without truncation; larger missing-interface logs consumed
  in35-line ranges. Raw failures remain outside Git. This is not final verification.

Earlier comparison handoff claims of complete inspection are not substantiated by
truncated tool displays. Current ledger explicitly supersedes those inspection
claims: byte/test receipts remain evidence of exactly their checks, not complete
semantic content consumption or independent review. This slice does not upgrade
historical review evidence or any fidelity/model/manager gate.

## Frozen diagnostic and full verification underway

`proc_6311` exit0/1s, `python3 -Werror /tmp/bbsim-native-joint-hit-kernel/prepare-full.py`:
133 final sources/131 prior/210 protected/ten accepted inputs/original protocol/
base-main-exact-four-path scope/78 completed evidence pins exact; all inherited
suite commands/assertions and compatibility copied with only external routing/
current manifests/protected count changed. Complete short logs consumed/stderr empty.

`proc_ac5c` exit0/3s: exact one fixed `frequency_diagnostic_preregistered_seed19_100000_words_every_model_context`
method,14,400,000 supplied words (3×48×100,000), seed19 and fixed6sigma+1 category
bounds pass.133 source hashes before/after and78 completed evidence pins unchanged.
Complete short raw logs consumed. This is synthetic implementation testing, not
real-data fitting, predictive validation or approval of any variant/default.

`proc_434d` runs the entire frozen inherited suite and four-mode byte compatibility.
Outcome/full raw output inspection/author acceptance/local checkpoint pending;
no executable/test mutation after diagnostic freeze. No manager/fidelity gate upgrade.

## Complete frozen verification — author checkpoint preparation

`proc_434d` exit0/264s, `bash /tmp/bbsim-native-joint-hit-kernel/final/full-checks.sh`:
623 Rust tests in36 unfiltered summaries (14 new methods),356 Werror Python tests
(`28.871s`), offline locked build/fmt/all-target Clippy `-D warnings`/Python compilation
and every inherited control passed. Native1350 cases/source69 metrics/five fixture
transitions/54 metric comparisons/27 shared-upstream ER/54 boxes/2000 reproducible
games/100000 legacy diagnostic draws/read-only source repeats/full prior field
reconciliation passed. Four old traces/16 four-mode records/four rated live pages
byte-identical;133 tested/131 prior/210 protected/ten inputs/78 evidence/prefix exact.
Expected generation failures and HTTP400/404/421/500 remain negative-control output,
not hidden errors or fictional games.

Full stdout1840 lines/99728 bytes SHA256
`9aaebb9906cb21a7853c23270f8c5c4083650aa90b5b588c23c0ff354ce113cf`;
full stderr545 lines/77560 bytes SHA256
`5559ce823a2dcca777452aa17c0d6ad92cde1002961685df907095a65ed003de`.
`proc_4b7b` exit0/1s verifies completed raw logs/pins and creates100 lossless1800-character
fragments (56 stdout/44 stderr), concatenation exactly equals original bytes.
All100 fragments actually consumed in nontruncating visible responses, including
complete commands, failures-as-controls and unfiltered test summaries. Complete
source inspection is established by the subsequent52-part actual staged review,
including all323 module/654 test lines, not the earlier truncated direct source reads.
Fragment preparation/hashes alone would not establish inspection; these statements
apply only to the completed native run, not unsupported historical comparison claims.

No executable/test mutation or new real-data/model fitting after measurement.
The following exact staged checkpoint supersedes the preparation's pending status;
local commit/postcheck remain pending. No independent-review, production/default-model/
manager/G1–G3 approval.

## Bounded author acceptance of the frozen native prerequisite

`proc_52a1` exit0/1s: exact four paths/base/main/all214 work-index blobs/210
protected HEAD-index/133 tested/131 prior/ten inputs/78 predecessor/233 completed
stage pins/prefix/516 prose links passed; full-index diff1332 lines/93415 bytes,
SHA256 `ea24270ddc53c35ff248f1798b72971f16f040654fab0aaf18c3ccbc7c41a5b9`,
tree `e6ac2c36ab7577d0a8fd2f20a2d5c220fdc9caa2`. All52 lossless1800-character
parts actually read without truncation, including every new source/test line and
complete original protocol/ledger delta; exact reconstruction retained externally.
The preparation's source-inspection sentence was premature; this staged review,
not truncated earlier source displays, substantiates complete source consumption.

Parent/author accepts this exact bounded sampler/projection/refusal implementation
on complete frozen verification and actual staged inspection. No substantive kernel
finding; inspection chronology corrected in documentation only. No source/test
mutation, extra empirical measurement, model/default selection or gate signoff.
Acceptance-only delta is confined to the two owned Markdown paths; original
preregistration prefix, frozen executable/test bytes and staged code sections remain
exact. Receipt `/tmp/bbsim-native-joint-hit-kernel/acceptance.json` binds the reviewed
diff/tree. Fresh precommit/local commit/postcheck still required; no push.

## Verified local code checkpoint and documentation handoff

`proc_b905` exit0/1s confirms exact acceptance-only two-doc delta/all214 work-index
blobs/preserved frozen/protected/input/233 stage/15 completed checkpoint pins/prefix/
516 links, source/test staged sections byte-identical. All10 lossless acceptance-
delta parts actually consumed. Diff check caught `native-joint-hit-kernel.md:245:
new blank line at EOF`; removed only the extra newline before fresh checks. Original
failure/delta retained; no warning/test suppression or source/test mutation.

`proc_8275` exit0/2s fresh precommit/local commit/postcheck passes; code
`d417b7f2061bc3e6851d7c7277ea7b8682a75f12`, parent `e1ccd7cf98ee3b4811997fc8d964838d55e22df9`,
exact four paths/tree/all214 committed-index-work blobs/133 tested/131 prior/210
protected/ten inputs/78+233+15 completed pins/prefix/516 links/clean main ahead10;
no push. All four short commit/post stdout/stderr logs actually consumed, stderrs
empty. Source/tests unchanged from diagnostic/full verification through author
acceptance/commit. This establishes only the standalone synthetic prerequisite.

This two-doc handoff records the verified code checkpoint; its separate exact
stage/commit/postcheck must verify documentation-only scope. Next is a separately preregistered training-
count export/native loader, not automatic real-data fitting, model selection or
producer/manager integration. Original protocol prefix remains byte-exact. External
`/tmp` evidence can expire; no data/large caches/logs/helpers committed or published.
