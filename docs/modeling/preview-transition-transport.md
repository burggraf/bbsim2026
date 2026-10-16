# Native immutable preview transition transport — preregistration

Task **P-01-preview-transition-transport**,2026-10-16 UTC.
Published prerequisite `b3bf6a1dc441f12a982926f0ee5d508dfbfc5c18`,tree
`f1a663e2775572a5dc561888199f76b7828bf439`;294 blobs/183 scientific sources.
The separate [transport contract](preview-transition-transport-contract.md) is
preserved byte-for-byte,including its original146-line contract and findings.
Exactly four owned paths:

1. `docs/PROGRESS.md`
2. `docs/modeling/preview-transition-transport.md`
3. `engine/examples/support/preview.rs`
4. `engine/tests/preview_transition_transport.rs`

Protect292 other paths/182 other scientific sources. Expected296 final blobs/
184 scientific sources. Preserve the entire existing generated-preview test
file,all other tests/callers and the complete published transport-contract bytes,
including its146-line original prefix. Authenticate actual line counts/hashes;
never overwrite or rebaseline these authorities. Preserve the preview module's
complete prefix before the `pub fn game_from` declaration. Record its actual
line count/sha256 in prerequisite evidence before tests or runtime edits. Only
the old `game_from` body/signature's unneeded state mutability and the new typed
loop may change. Constants,
RNG,models,ordinary event policy and all existing upstream functions stay exact.

## Observable behavior fixed before implementation

Add `game_from_transitions<F>(seed:u64,state:GameState,produce:F)` returning
`Result<Vec<EventTransition>,String>`,where
`F:FnMut(&GameState,&mut Rng)->Result<EventTransition,String>`.
Keep a single seeded loop:before each invocation enforce existing10000 accepted-
transition cap;propagate producer errors unchanged;require the whole returned
before-state to equal current state;clone only after into current state and
append the original transition. Do not extract/reapply the event,reencode as a
substitute for retaining it,or rerun a projector. Immutable typed construction/
decoding retains canonical legality;the loop adds continuity,not fact truth.
A valid ended initial state returns an empty log without a callback. No new
initial-state validator is promised. Successful game end stops immediately.

Error strings remain:

- new mismatch:`seed {seed}, play {one_based_index}: supplied transition before-state mismatch`;
- cap:`seed {seed}: resource ceiling reached, no winner fabricated`;
- old apply:`seed {seed}, play {one_based_index}, {outcome:?}: {engine_error:?}`.

The old `game_from` signature and callback contract remain unchanged;wrap its
producer,increment a local attempted-play counter,apply the returned Event once
and delegate to the typed loop. Preserve old error sequencing/seed/output bytes.
No new RNG word is consumed by transport;the callback controls its own existing
RNG. External effects,consumed words and panics are not rolled back or caught.
No retries,fallback-on-error,successful truncated logs or fabricated winner.
Non-PA/substitution events remain non-PA;transport makes no independent outcome,
batting-order,statistical credit or manager decision.

## Test-first matrix

Write a new independent test file without editing old tests. First demonstrate
missing-symbol interface failure separately,then use a compiling empty-log stub
to demonstrate semantic failures before implementing the loop. An already-ended
empty case and legacy-only controls may already pass;report that honestly. First
failure does not prove a method's later subcases ran. Do not relax old assertions
or adjust scientific tolerances,inputs or authority to pass.

1. Independent copy of the predecessor loop as test-only reference,synthetic
   baseline counts `[20,8,1,15,4,1,4,2,45]`,seeds0..127 plus `u64::MAX`:
   compare every complete ordered canonical transition byte and callback RNG
   word against both old facade and new typed path. Real/default/joint/rated
   compatibility also remains under the entire inherited verification recipes.
2. Canonical supplied SF/SH/GIDP grids:SF masks4..7/outs0..1;SH/GIDP masks1/5/
   outs0..1,Top inning2 with explicit eligible fictional actors and responsibility.
   Supply one complete transition,then legally finish the game through ordinary
   callbacks;assert first transition identity,BBE28/29/30 bytes,full continuity,
   supplementary getters,available accounting and exactly one PA for that play.
   Explicit walkoff BBE27 from tied Bottom9/Third-only/single,known winner/batter
   touch/timing:exact retention and one callback. No new terminal contact facts.
3. Scripted mixed prefix using supplied SF/bunt/runner-only safe steal/K/walk/
   GIDP/K plus explicit bench substitution,then a legal deterministic ending.
   Assert byte retention,non-PA cursor/PA/BF invariants,pitcher responsibility and
   accounting before/after runner-only/substitution. This is a script,not a
   generated frequency,actual-source draw or fitted actor/manager policy.
4. Whole-before discontinuity:score,cursor,bench/removed identity,responsible
   pitcher,inning and outs;construct each returned transition canonically from
   the modified state. Check first and later callback mismatch index/refusal,
   no retry or successful partial log. Keep before/transition inputs immutable.
5. Exact errors including empty/reason-like strings at first and later callbacks;
   closure-owned non-Copy state;zero calls for valid ended state;no post-ending
   callback;repeated seed/script/inputs remain exact.
6. Exactly10000 accepted transitions then cap/no10001st callback,including legal
   explicit held-runner pickoff attempts that do not advance PA/order. Verify
   fixture legality rather than introducing a new NoPlay event kind or a
   fabricated state reset. If such a fixture is refused,retain the failure and
   inspect existing canonical mechanisms before changing the fixture;do not
   weaken the cap or legality assertions. Preserve old all-K cap/error behavior.
7. Old invalid-event engine errors keep exact seed/play/outcome Debug formatting
   at first and later calls. Existing models/policy/RNG constants and full
   generated-preview test prefix remain unchanged.

Use real reducers/projectors/accounting/codec,no mocks of baseball legality.
Supplementary fixture setup must validate outside the loop too. Fix interface
mistakes without labeling them semantic reds;retain raw stdout/stderr and exits.

## Verification/publication boundary

Authenticate published294 blobs/292 protected/183 prior sources/182 other
sources/372 inherited recipes/all331 driver lines/197 completed artifacts/348
callback-checkpoint bindings/14 callback publication bindings and43 documentary
checkpoint/14 documentary publication bindings. Preserve original failures and
partial artifacts. Freeze finished current source/helper/input/recipe bytes.
Apply only administrative exact-inverse epoch/scope/owned-path/private-output
aliases;retain all scientific premises,thresholds and inherited recipe/driver
bytes. Do not pin a running writer or blindly rerun an occupied preparer/root.

Focused Rust tests/fmt/offline locked strict all-target Clippy first,then the
entire applicable fresh offline/locked Rust build/tests/fmt/Clippy,Werror Python
discovery/compilation and inherited scientific/native/property/refusal/
publication/legacy/BBE/prefix/four-mode/raw UTF8-page checks. Compare old/new
ordinary output bytes,not merely legal summaries. Inspect completed artifacts
before fixed candidate inspection/bounded author acceptance/once-only commit/
existing-identity postcheck/authorized ordinary push/actual remote clean-main
closure. Historical passes do not certify changed preview code.

No engine/accounting/codec/projector/selector/adapter/provider/caller/test-default
changes except the specified transport module and new test file. No dependency,
delegation,source query/transfer/census,fitting,actual-source class draw,automatic
facts/actor policy,gameplay integration,CLI/UI/manager/season/release change or
G0–G5 signoff. Unknown is not false/held/Out. Preserve286 mask differences/1517
boundary-not-compared/1330 nongeometry memberships/2256 descriptor matches;
retrospective evidence is not predictive validation. Bounded author acceptance
is not independent review,comprehensive secrets/license certification or new
rights clearance. Internal G0 permission remains limited;public-data/app and
fidelity/manager/product holds remain unchanged.

## Focused evidence — 2026-10-16;whole verification still pending

Original131-line protocol remains byte-exact. Prerequisite proc_114a exits0/8s:
294 published blobs/292 protected/183 prior/182 other sources/372 recipes/331
lines/197 completed/348+14 callback/43+14 documentary bindings;16 prerequisites
closed. Actual unchanged prefix428 lines,old generated-preview tests912 lines,
complete published contract165 lines/original146-line prefix authenticated.

New11-method test file precedes implementation. proc_8170 exits101/3s with6
E0425 missing-symbol errors/no test execution,an interface failure. Empty-log
stub proc_aebe exits101/8s:6 transport semantic failures/1 pre-transport
InvalidParticipants fixture failure/4 passes,including a vacuous repeat check.
The new supplied helper incorrectly assumed Away batter/Home defenders after a
half switch;compare the existing projector tests' current offense/defense
selection. Correct only new fixture actors and require actual first transition/
ending in the repeat test. proc_fd1c exits101/5s:8 semantic failures/3 already-
green controls over11 methods,0.71s. First failures do not cover later subcases.

Initial implementation proc_86c7 exits101/7s:10 pass/1 fixture InvalidState,
2.49s;fmt/Clippy not reached. Diagnostic proc_f07b exits101/4s identifies outs
change6/attempt3:preventing cleanup leaves arbitrary runner a3 as next batter,
which existing canonical snapshot validation correctly refuses. Change only that
new fixture's supplied runner to a5,retain all14 mismatch pairs and strengthen
its explicit canonical-validity assertion. No engine rule or old test changes.
proc_84df exits0/11s:11 Rust methods2.40s/fmt/strict all-target Clippy/diff pass.

Final corrected tests replay against the retained stub in proc_4947:8 semantic
failures/3 already-green controls,native101 preserved;final implementation bytes
restored exactly. Same managed process exits0/14s after fresh11-method Rust
2.38s/fmt/offline locked strict all-target Clippy/diff pass. BBE27–30 retention,
16 supplied contact contexts,129 synthetic seeds,14 discontinuity pairs,mixed
non-PA/substitution accounting,exact errors and10000-step caps are exercised.
All final focused assertions run green,not merely the first red subcase.

Current-scope proc_b305 exits0/2s:7 inherited refusal controls1.715s;administrative
root/scope-observation aliases invert exactly,not a new semantic-red cycle.
One private preparation attempt failed before writing the frozen-control copy:
its short inverse target already occurred in setUp. Retain all5 partial copies;
V2 verifies them without overwrites and aliases the whole poison call. Original
inline failure remains in the transcript;the metadata is not a reconstructed
native log. Numerical/scientific assertions and tolerances stay unchanged.

Freeze proc_e068 exits0/2s:184 current sources/292 protected/75 helpers/eight
original inputs/372 inherited recipes/331 actual driver lines. Five-entry closed
freeze authority SHA
`4ede35372956086f539e8553a02225e501b90ed46f50c431fcf4efbceb5e8d78`.
proc_c286 exits0/4s with5 frozen-refusal methods3.464s;three before-full bindings
closed after observed completion. Inherited focused proc_5edb exits0/22s with
5 reference/3 native-gate methods and21 adapter/9 selector Rust methods.

Historical guards now need an explicit old-preview (path,digest) resolver.
The compiling raw-file placeholder fails;the first implementation also fails
because its new count premise guessed453 lines. The measured original is452
lines and independently matches both inherited SHA and published Git blob.
Correct only that new premise;proc_183c exits0/12s with7 methods11.282s.
Current473-line preview remains separately authenticated in the184-source
freeze;the shared428-line prefix is exact. Unknown digest/blob/count/duplicate
bindings and poisoned snapshots refuse;no changed-working-file hash is spoofed.

proc_5485 exits0/22s prepares4 exact-inverse historical cores and verifies the
177-source historical application subset while requiring the current184-source
freeze. proc_be55 exits1/3s after eight-copy preparation/live-epoch2/real-gate3
controls pass:an unadapted nested qualification validator still checks actual
current preview against its old digest. Retain every V1 output/failure. V2
explicitly aliases that validator to the same authenticated snapshot,leaving
its AST/test-delta/measurement/numerical/report predicates untouched.
proc_d53f exits0/10s:5 historical cores/8 exact-inverse versioned epoch plans,
2 live-epoch controls and the positive predecessor guard pass. No overwrite.

A historical-manifest CLI labels archived preview bytes separately from other
actual raw bytes. Compiling no-check stub proc_34c4 exits1 with5 semantic
assertion failures over2 methods;proc_5caa exits0/3s with2 methods2.681s and
py_compile. Unknown/current preview digests,duplicate rows and missing arguments
refuse. Only the inherited non-owned manifest's old preview binding needs this
CLI;the fresh184-source manifest remains a native raw-current shasum check.
Seventy-four inherited direct-check commands were inspected;an initial inline
inventory used the wrong argv slice and found none,then its corrected strict
five-token parse identified the two manifests:old non-owned/current tested.
This diagnostic mistake did not change source or evidence authorities.

Full-suite preparation proc_e2b0 exits0/42s:426 recipe bindings retain the
372-entry inherited prefix/all331 actual prior lines by exact inverses;driver342
lines. Seven inherited preparation-control methods/bash syntax/whole source and
alias guards pass;not a whole-suite execution.

Prelaunch proc_fb48 exits1/1s:the new closure accidentally pins its redirected
empty stdout before its final print. Its195-binding V1 closure is invalid and
retained,not accepted;the launcher correctly refuses the changed raw hash.
Exactly that one binding mismatches. No full-start/native streams/native-exit
artifact exists and no source/426-recipe byte changes. V2 keeps control streams
in the managed process rather than the closed root and refuses regular stdout/
stderr before arguments or output mutation. No-op-gate proc_b7f3 exits1 with3
assertion failures over2 failed/1 already-green methods;proc_926c exits0 with3
methods0.054s/py_compile. Pipes/sockets/terminals remain allowed. V1 retained.

proc_9100 observes actual `START entire fresh transport` after authenticating
V2's213 finished preparation bindings,closed SHA
`e4ca8b309aed3552ad8fe75774f825879aa8447e07fb19fba7d60f87dbd836e6`.
The entire fresh native suite is running;readiness is not completion. The75-
helper freeze/all426 recipes remain immutable;native raw streams are active
writers and must not be pinned early.

Inspector preparation proc_bae6 fails before any output inspector is written:
a later filename-only inverse also reverses that filename inside an earlier
whole pin-call target. Regression proc_9dfa reproduces1 semantic failure and1
already-green hexadecimal-authority control. V2 separates numeric scope and
common filename aliases;proc_f0bc exits0 with2 methods0.001s/exact-inverse
preparation/py_compile. The inspector is prepared,not executed.
744 Rust/48 Rust summaries/601 Python discovery/48 Python summaries remain
forecasts,not observed results:733 prior+11 new Rust methods and three appended
private Python suites7/2/2. No statistical threshold/report predicate changes.
Those were launch-time forecasts;the observed completion below supersedes
that pending status.

## Observed fresh verification and bounded acceptance — 2026-10-16

proc_9100 exits0/1560s(native1557.828s):744 Rust/48 zero-failure summaries;
601 Werror Python56.461s/all48 summaries;offline/locked build/tests/fmt/strict
all-target Clippy/compilation and all inherited scientific/native/property/
refusal/publication/compatibility checks. Raw stdout461713 bytes SHA
`3cbf5731cb8b6d4dd8128743a1076ec6d90f162a51a5f92ed3ac14fee817ed58`;
stderr211164 bytes SHA
`d213d96af33ac4ba7347c84766d6e83eaf93eb79448db06fddd35c9eda1cdf49`.

First inspector proc_3f03 exits1/42s before result/manifest writes:the frozen
native selector helper authenticates the actual184-source scope before/after
execution,but emits an inherited hard-coded `source_count=183`. The original
75-helper freeze/426 recipes/raw native metadata/artifacts stay immutable.
The separate validator requires that exact raw183 literal and independently
frozen current184-source proof;it does not promote old metadata into current
scope. proc_5581 yields2 semantic failures/1 already-green method;
proc_f0c8 exits0 with3 methods0.003s/py_compile. All failed outputs are retained.

Versioned inspector preserves scientific predicates,original input authorities,
digests and tolerances. proc_45cf exits0/42s,empty inspector stderr:184 sources/
292 protected/426 recipes/211 completed artifacts;all75 certificates/two exact
retrospective diagnostic repeats/two synthetic selector repeats;walkoff/SF/bunt/
legacy full-byte parity/384 prefixes/16 four-mode records/four raw UTF8 pages.
Postfull control-stream refusal3 methods pass0.055s. No scientific source changed
after the full run. Result SHA
`d24ee06ed2b57e106a75fe9a6b8cc2bd2a45984b00e8c813ca1037ec2eb631b4`;
211-artifact closure SHA
`e3ae959279570071a807cd4e931dc47835d91a3ef6f2ede08c517bbe8fb4ca38`.

Bounded author acceptance closes249 finished root files with managed
unredirected control streams. It is not independent review,exhaustive visible
raw-log reading,predictive validation,source-data rights clearance,gameplay
integration or a gate signoff. Candidate/publication remain pending. Historical733 Rust/601 Python
cannot certify changed preview code. Raw failures/large debug assertions remain;
visible raw reading is bounded,not exhaustive. No independent review,automatic
provider/gameplay integration or gate signoff is claimed.
