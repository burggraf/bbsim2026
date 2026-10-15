# E-03 — explicit sacrifice-bunt throw (bounded, nonterminal)

## Preregistration — 2026-10-14, before tests or implementation

D-20/D-21 continue engine fidelity without managerial integration or gate waiver.
Base is verified local `a3e7f39d3e3a6e1588ef2f62e99a27b7944c2022`:240 repository
blobs/150 executable-test sources,clean main ahead7;origin804a72d unchanged.
Caught-SF acceptance is separate;its code-post receipt was observed after native
`proc_ea2b` exited0/25s. No push,source transfer,dependencies or delegation.

Exactly nine owned paths:
- `docs/PROGRESS.md`
- `docs/engine/sacrifice-bunt-throw.md`
- `engine/src/lib.rs`
- `engine/src/reducer.rs`
- `engine/src/rules.rs`
- `engine/src/accounting.rs`
- `engine/src/codec.rs`
- `engine/src/sacrifice_bunt.rs`
- `engine/tests/sacrifice_bunt_throw.rs`

Protect234 existing non-owned blobs;150 prior/152 final executable-test sources,
243 final repository blobs. Parent sole writer. No producer,sampler,CLI,UI,
manager,probability fit,source normalization,new measurement or release changes.

## Inspected rules and source boundaries

Read the product/execution contracts,current ledger,canonical reducer/constructor/
batter-charge flow,ground-throw facts/validator/fielding proof,credit/accounting/
clean-ER flow and existing generic SH and ordinary-ground tests. Cached2019
rulebook (`/tmp/bbsim-2019-rulebook.txt`,no transfer):9.08(a) and Comment,9.08(b)/(c),
9.09(a)(2),9.10(a)(1)/Comment and9.10(b). Rule9.08(a) requires fewer than two outs,
a bunt advancing runners and the batter put out at First (or an error exception),
unless the scorer judges exclusive bunting for a base hit. That judgment considers
circumstances;it cannot be derived from endpoints,SH labels or lineup position.

Existing `OrdinaryGroundThrowFacts` explicitly means a NON-BUNT ball with held
runners. Do not reinterpret it as a bunt,relax its validator,or attach it to this
new episode. Generic SH credits already support conditional AB/counters,but do
not establish contact method,advance cause,scorer judgment or reviewed PO/A/RBI.

## Smallest separate factual episode

Add immutable optional transition context `SacrificeBuntThrowFacts` with named
batter_id,advancing_runner_id,thrower_id,receiver_id and explicit
`SacrificeBuntJudgment::{Unknown,SacrificingOwnChance,ExclusivelyForBaseHit}`.
The subtype asserts a fair grounded bunt,controlled and directly thrown by the
named thrower to a distinct receiver catching it on First before this batter's
first legal touch. This bunt exclusively advances the named before-First runner
to an initially clear Second;any before-Third runner holds throughout. No other
touch,relay,deflection,retirement,appeal,misplay,runner episode or personnel change.
These are supplied modeled/reviewed assertions,not inferred source certificates.

Support only masks1/5,before outs0/1,one batter retirement,zero runs,full explicit
runner vector and unchanged carried pitcher responsibility. Require whole-chance
NoMisplay,actual queued batter/current pitcher,before-eligible distinct fielders,
and exactly PA1/NoHit1/SH1/receiverPO1/throwerA1. AB0/H0/RBI0/R0/E0/PB0;current
pitcher BF1/out1/SH1. RBI zero is a qualified event-local proof,not missing data.
A pitcher may field/throw or cover First when eligible;DH/bench/removed/offense
actors and duplicate fielding actors refuse. No guessed permanent positions.

Unknown judgment refuses after stronger canonical contradictions;known exclusive
base-hit judgment contradicts this SH claim. Reject missing/wrong/scaled credits,
NoHit/AtBat/hit contradictions,wrong participants/charges,missing runners,other
advances,multiple outs/scorers,two outs and competing contact/air/hit/award/pitch/
balk facts. All Bottom innings>=9 refuse,even this zero-run shape. Do not clip,
resample,change game-ending rules or fall back to a generic/ordinary ground play.
Dropped/caught bunts,error exceptions,safe-batter9.08(b),failed preceding-runner
attempts9.08(c),squeeze/home advances and compound plays need separate evidence.

## Canonical APIs,accounting and compatibility

Add `apply_with_sacrifice_bunt_throw_facts` and
`EventTransition::try_new_with_sacrifice_bunt_throw_facts`,immutable
`sacrifice_bunt_throw_facts()` plus qualified event-local fielding/RBI proofs.
One canonical validator is shared by reducer,constructor and semantic decoding;
no Event literal field,mutable metadata,unchecked constructor or second outcome
roll. Existing immutable walkoff/SF contexts are incompatible with this family.

Keep every None-context API,accepted/refused behavior and BBE25–28 binary/normal/
pretty Debug output unchanged. New records use BBE29 with their own complete
named-context tail,not an ordinary non-bunt throw or air tail. Decode revalidates
actors,judgment,geometry,credits and supplied after-state;unknown enum tags,
trailing/truncated/tampered bytes refuse. Valid Unknown judgment is not certainty.

Existing `clean_play` proves held ordinary-ground retirements,not bunt advancement.
Do not broaden positive-prefix Rule9.16 proof. Explicitly keep this new context
outside clean-ER proof,with a fully observed completed-half control where all
other events qualify and an old held-ground comparison proves the fixture can
otherwise establish ER. Preserve prior-None coverage;no official full-box claim.

## Observable acceptance fixed before implementation

1. Authenticate all240 committed/index/work blobs,234 protected,150 prior sources,
   all predecessor evidence/recipe/input/export/report pins and cached rulebook.
   Preserve five changed current-core same-digest snapshots and the full17-source
   baseline engine before editing. Keep raw manifest rows/expected digests and
   compose explicit(path,digest) mappings;no silent rebaseline or skipped checks.
2. Missing-API compiler red,then genuine semantic reds against a typed refusing
   scaffold. Keep already-green refusal controls,fixture/caller/compiler failures
   and recipe failures distinct;preserve exact raw evidence/failing seeds.
3. Independent masks1/5 x outs0/1 x both halves/nonterminal innings x all nine
   cursors x DH/no-DH x eligible fielding roles and move/credit reversal. Assert
   unchanged before/exact after,PA/BF/SH/AB/H/RBI/out/run/PO/A/E/PB units,held Third,
   carried responsibility and exactly one cursor advance. Apply/try-new/codec agree.
4. Explicit unknown/exclusive-hit judgments,competing causes,all wrong/missing/
   duplicate credits and actors,DH/bench/removed/offense fields,malformed states/
   supplied after-state,non-First/occupied-Second/extra movement,personnel/multi-out/
   multi-run/two-out contexts. Exhaust all65527 Bottom>=9 innings,late-Top positives.
   Strong canonical errors precede unrelated Unknown judgment;refusals are atomic.
5. BBE29 every truncation,trailing data,invalid/valid-Unknown/exclusive-hit tags,
   identity/version/after tampering;immutability. Legacy generic SH remains accepted
   without promoting fielding/RBI evidence;old non-bunt throw+SH remains refused.
   Completed-half ER and prior-None coverage controls have independent positive
   controls,not fixtures already incapable of proving the compared field.
6. Freeze all152 sources before offline locked Rust build/tests/fmt/all-target
   Clippy -D warnings,Werror Python discovery/compilation and every inherited
   source/native/property/legacy record/page comparison. Add fresh baseline/current
   BBE28 SF binary+normal/pretty Debug parity through unchanged original SF tests
   plus appended export,alongside inherited BBE27/legacy/prefix checks. Preserve
   original callers/assertions/counts and all historical raw rows/snapshots.
7. Declare actual bounded raw reading scope;inspect all scoped staged source/docs
   in small nontruncating pieces and any final documentary delta. Recheck scope,
   HEAD/index/work/protected/input/evidence/recipes/original prefix/links/tree/
   whitespace and verify the local commit/postreceipt. Receipts/hashes alone are
   not reading evidence. Parent-author bounded acceptance is not independent
   review,calibration,manager approval or E-03/G1/G2/G3/release signoff.

No owner-input blocker identified. Unresolved consequential rules/scope decisions
must be recorded/resolved before behavior changes;D-21 is not a gate waiver.

## Authentication and TDD observations — before behavior implementation

`proc_e677` exits0/16s:all240 committed/index blobs/234 protected/150 prior sources,
predecessor recipes/evidence/11 inputs/four exports/report/rulebook authenticated;
five same-digest current-core snapshots and17-source baseline preserved before
tests. Exact original protocol remains the prefix;external evidence root
`/tmp/bbsim-sacrifice-bunt-throw/` may expire.

`proc_ad2e` exits101/2s on the eight-method test:seven expected missing-interface
compiler diagnostics,no semantic result. Typed refusing scaffold then `proc_d06a`
exits101/7s:six genuine semantic failures,one oracle defect and one already-green
refusal method. The completed-half control establishes ER Some(2) before reaching
the expected new-bunt refusal. No compiler warnings;raw logs/scaffold/tests retained.

**RBI oracle correction,2026-10-14,before behavior:** existing `rbi_contribution`
explicitly proves named batter RBI0 for EVERY zero-run PA by legal-run conservation;
it does not require an SH cause. The legacy fixture incorrectly expected None,
and the positive helper incorrectly expected an empty map rather than a named
zero entry. Correct both to assert the exact existing one-entry named-zero map,
without changing production accounting or weakening coverage. This qualifies the
original protocol's ambiguous "without promoting ... RBI evidence" wording:
legacy zero-run RBI stays known zero;only new cause-specific metadata/PO/A are
absent. A new context-specific RBI getter must agree with that existing proof.
This is a fixture/oracle correction,not a production bug or newly added mechanism.
`proc_0202` exits101 on the corrected refusing scaffold:six genuine semantic
failures/two already-green controls,no warnings. All original failure logs and
both test snapshots remain;no weakened or silently dropped assertion.

## Initial implementation — not full accepted

Separate canonical context now validates the named bunt/First throw,scorer
judgment,actors,full vector/exact units and scope;strong canonical errors precede
unrelated Unknown judgment. Optional private metadata,additive reducer/constructor/
getters,BBE29 and exact reviewed PO/A are implemented. The RBI getter exposes the
existing named-zero proof;the main RBI fold is unchanged. Explicit clean-ER guard
keeps advancement out of the held-ground proof. The existing five-argument private
constructor forwards to one validating body with absent bunt context,so protected
SF code/callers and ordinary non-bunt source/tests need no edits. No Event literal
field,mutable context or bypass constructor. None-context Debug fields stay absent.

`proc_dc4f` exits0/8s:all eight methods pass in1.08s,1728 supported/permuted frames,
all65527 late-Bottom refusals,eligible matched fielders,credits/causes/geometry/
canonical-priority controls,BBE29 tampering/immutability,and completed-half ER None
versus the held-ground ER Some(2) control/prior-None fielding propagation. Complete
short focused logs read;234 protected blobs/original protocol prefix remain exact.
`proc_fd66` exits0/39s:strict all-target Clippy `-D warnings` passes and six
unchanged focused suites pass177 methods (accounting118,SF8,air19,ground12,
walkoff14,runner6). Earlier exhaustive-reading claim for207-line stdout/eight-line
stderr is withdrawn in the qualification below;Clippy output is clean. No source changed.

`proc_795d` exits0/10s:fresh cached17-source baseline/current compilation with
`rustc --edition=2021 -D warnings`;the unchanged original SF caller plus additive
export runs all9/9 tests on each. 3456 records/10368 BBE28/normal/pretty payloads
match exactly,33558912 bytes SHA
`d331e6f9c10df51269dd95a6975f9e615faaf8246892d96c488c2169c42a690f`.
Original caller prefix SHA `d2c9677dbd383887771c2ee08ed356d39a720921e5b4f904deb7fc80abfdf280`;
all original assertions retained. Both complete short outputs/empty stderr read.
This is backward-compatibility proof,not independent review/integration approval.
New preparation-loop regression retains all three controls:0.003s pass. Only the
known future self-recipe manifest is exempt;unknown missing inputs still fail.

Full inherited/source/native/property/legacy/prefix checks and full staged
inspection/acceptance remain pending.

## First frozen full verification — failed on missing external artifact

`proc_3bd3` exits0/3s:152 frozen sources/six recipes/39 closed prefreeze artifacts;
150 prior+148 historical+150 v2 original rows and17 (path,digest) bindings verified,
234 protected blobs/protocol prefix/inputs/evidence/base/origin exact. No source
changed after focused green. Preparation reconstructs the complete inherited
full script exactly under only declared output/checker/manifest paths
and protected-count text,then adds fresh SF parity/predecessor pins/planning
controls. No command or assertion removed;14 full recipes frozen;bash syntax
passes. Future-manifest loop has the same exact exception,not a missing-input skip.

`proc_fca4` was started with:
```sh
bash /tmp/bbsim-sacrifice-bunt-throw/final/full-checks.sh \
  > /tmp/bbsim-sacrifice-bunt-throw/full.stdout \
  2> /tmp/bbsim-sacrifice-bunt-throw/full.stderr
```
**Observed `proc_fca4` exit1/359s:**Rust673/41 zero-failure summaries,Werror
Python424 in31.447s,fmt/build/strict Clippy/compilation and the first fixture
checks pass. The next inherited command
`python3 -Werror -m audits.verify_pitcher_er /tmp/bbsim-f04-2019box.zip /tmp/bbsim-e03-pitcher-er-fixtures-durable.csv`
fails `FileNotFoundError: [Errno 2] No such file or directory` for the source CSV.
No later native/property/source/page/legacy checks or full-success inference.
Failure stack/tail displayed;some displays truncated as qualified below,not proof
of complete stack reading. Original raw evidence remains exact.

Failure records/raw exact copies/closed recipes/source manifest are preserved in
`/tmp/bbsim-sacrifice-bunt-throw/failed-full-v1/` with14 artifact pins in
`failed-full-v1.sha256`. Raw stdout65721 bytes SHA
`fb972a2ed2d2632fd7fcb3c747c0107cd751c771e41699d8fd33b06c8ada2076`;
stderr87251 bytes SHA
`c3f322f9e274dab480c78bb5e9996d7a234f600267a0b479563658d0de7a894b`.
Frozen152 application sources/six recipes/39 prefreeze/14 full recipes/234
protected/original protocol/input pins remain exact. No application change.

**Preflight gap,not engine failure:**the eleven inherited input pins did not
include this separately consumed ER CSV. Its original identity digest was not
found in local historical manifests. A finite offline search checks412 bbsim CSV
headers,no matching `game_id,player_id,earned_runs` header;neither cached DuckDB
has an ER/pitcher table. Persistent audit cache contains only original F01/details/
reconcile/Retrosheet-box/DB artifacts,not the source ER CSV. A broad initial path
search included long historical Markdown and truncated;no exhaustive raw-reading
claim. Subsequent targeted manifest and CSV-header searches were bounded/complete.
Original protected SQL attaches remote DuckLake,so it is not an offline recovery
recipe. Do not fabricate source ER from its Retrosheet comparator,which would
make a circular equality check;do not weaken/skip the required comparison.

**Blocked pending owner input:**restore/provide the original source-ER CSV,or
explicitly authorize separate preregistered,bounded read-only source reacquisition
and provenance audit. Current no-transfer/no-new-measurement scope remains intact;
no network action. If reacquisition is authorized,record snapshot/version/transfer
caps and missing-old-digest limitation before any query;do not silently label a
new extract byte-identical to the lost original. On recovery,pin/validate the
input before a fresh full run;preserve v1 failure/recipes/rows and do not blindly
rerun exclusive helpers. No stage/acceptance/commit or independent/gate approval.
Prepared stage/inspection/documentary helpers have not been run.
Recovery interview `93878c4f-7e9c-4a7d-b6a1-0dd5d4b34cc7` timed out after600s
without an answer;questions saved under `~/.pi/interview-recovery/`. No recovery
or transfer authorized. Await explicit chat choice:original local CSV,separate
read-only/provenance-audited recovery with at most64MiB additional metadata/data
transfer,or remain blocked. Silence/defaults are not permission;no automatic reopen. Complete cached table-name lists/blocked-freeze output read;all152
sources/234 protected/17 historical bindings remain exact. No background job.

## Explicit recovery amendment — preregistered before network access

Owner chat: **“Authorize bounded recovery”**,following the stated original-three-
game/three-field,read-only,at-most64MiB additional metadata/data option. This is
an explicit narrow exception to the earlier no-transfer prerequisite,not a gate
waiver,source-normalization/engine expansion,or permission to fit/measure gameplay.
The lost original source CSV has no recovered identity digest;the new extract
will have separate provenance/version/digest,never an invented byte-parity claim.

- Scope:2019 MLB regular-season `ANA201904040`,`ANA201904050`,`ANA201904070`;
  only source `stg_game_earned_runs` fields `game_id`,`player_id`,`earned_runs`.
  Reuse authenticated cached2019 game selection instead of transferring another
  game-start table. Record the fixed original SQL and its exact adaptation.
- Use cached native catalog SHA
  `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583`,
  snapshot857,offline with external access disabled. Identify active table/schema/
  columns/file paths/stats and any delete/encryption/unsupported metadata before
  selecting allowlisted physical files. If missing/incompatible,stop;do not
  silently switch snapshots or fetch unrelated tables/catalogs.
- Additional cumulative cap **67108864 bytes** for all metadata/data response
  payloads,with conservative reservations before every request. Cached metadata
  costs zero additional transfer. Existing `RangeRelay` is loopback-only,public
  HTTPS allowlist/range/status/length/encoding/ETag checked;no redirects. Disable
  native extension auto-install/autoload/retries/unrestricted upstream reads.
  Failed requests retain reservations. No implicit retry or budget reset;an
  unaccounted hard-killed attempt consumes its full allocated allowance.
- No full-source season/table materialization. Only three filtered columns and
  three game keys may be exported;physical column chunks/row groups and footers
  can include adjacent rows,still fully counted against the cap. Do not output
  unrelated values. Preserve every request/failure and native query/profile log
  outside Git. Do not install dependencies or publish artifacts.
- Before acceptance:exact header/strict CSV/unique named keys/nonnegative explicit
  values/all three games;original independent comparison must validate all27
  pitcher-game cells,including zeros,against authenticated Retrosheet boxes.
  Retrosheet is comparator only,NEVER the source used to construct this extract.
  Refuse missing/extra/mismatched keys/values;no fill,drop,normalization or tuning.
- Record raw catalog/schema/file provenance,ETags/ranges/digests/bytes/query/native
  version/elapsed time and source attribution. Promote only complete validated
  output to an external persistent content-addressed cache and the old required
  filename;pin it before ANY fresh full run. Preserve original11-input manifest
  exactly and add a separate supplemental input pin,not a silent rebaseline.
- Application sources/tests stay frozen152/protected234. Keep failed full-v1 and
  all recipe/source manifests immutable. Use fresh output/recipe names for any
  later full verification;do not blindly rerun exclusive helpers. Full Rust/
  Python/source/native/property/legacy/SF/prefix checks must pass freshly before
  staged inspection/acceptance/local checkpoint;no historical-pass substitution.

Concrete next:perform offline catalog authentication/metadata inspection,then
prepare/pin the bounded extraction recipe before any request.

Offline recovery inspection authenticates snapshot857/catalog SHA and table663;
one 4750238-byte Parquet object1727,785-byte footer,three physical source columns;
no deletes,partitions,mapping,encryption or inlined table. No additional transfer.
The first metadata helper rejected a present default as if every default must be
absent;raw traceback/helper/outputs preserved. Exact inspection shows only literal
SQL `NULL` defaults in DuckDB dialect,not non-null values or expressions. A separate
v2 validator accepts ONLY these exact metadata values,requires physical named
columns (no default is applied),and passes three controls including expression/
non-null/reordered/missing-column refusals. This is a tooling assumption correction,
not a source normalization or application change. Existing relay's seven adversarial
unit methods pass freshly in5.574s;all complete short outputs read. Extraction
recipe/protocol/metadata/native controls are pinned before the first request.

## Observed bounded recovery and fresh full-v2 lifecycle

Authorization/recovery observed2026-10-15;checkpoint time12:47:37Z. This dated
entry supplements the frozen prerequest protocol;it does not rewrite its bytes.

`proc_e960` exits0/3s. Native extraction1.887267834s,DuckDBv1.5.6;five complete
requests with stable ETag `47bb0c90c90c5d8ef3e64f0eaeff76e2`,total received and
reserved **4446175 bytes**,below67108864. No other catalog/source download or
retry. Each exact range/response digest is preserved in `requests.jsonl`.
New source CSV678 bytes/27 cells/three games/three fields,SHA
`4d3c5342e0045e6cc26ca1831c01159d9aa135ea3b90365afb38b6e367e06890`.
Unchanged strict comparator validates all27 separately extracted source-vs-box cells;
this remains shared-upstream reconciliation,not engine ER reconstruction. Native
selection control true;complete short logs/five raw request records read,not a
manual source-row-reading claim. Original lost digest stays unknown;no byte-parity
claim. Read-only0400 content-addressed external cache and old required-path hardlink
are exact;separate two-path `supplemental-input.sha256`,original11 rows untouched.

48 recipes/metadata/protocol inputs pinned before network;56 closed recovery
artifacts in `completed-recovery.sha256`. Failed metadata assumption/helper/raw
trace retained;no source normalization. Application152/protected234 unchanged.
`proc_53ca` exits0/2s:full-v2 recipe reconstructs every v1 command/assertion under
only new output/checker/recipe paths,then appends recovery/failure pins. Eighteen
recipes frozen;new checker validates recovered input BEFORE full verification.
All v1 outputs/14 failure artifacts/six original recipes/39 prefreeze rows/fourteen
original full recipes remain unchanged. No resumed suffix or historical-test-pass
substitution.

`proc_6f20` full command:
```sh
bash /tmp/bbsim-sacrifice-bunt-throw/final-v2/full-checks.sh \
  > /tmp/bbsim-sacrifice-bunt-throw/full-v2.stdout \
  2> /tmp/bbsim-sacrifice-bunt-throw/full-v2.stderr
```
Native completion observed: **exit0/342s**,all673 Rust tests/41 zero-failure
summaries,424 Werror Python in32.593s,offline locked build/fmt/all-target Clippy
`-D warnings`/compilation and EVERY inherited source/native/property/page/legacy/
prefix/assertion check. Fresh1152 BBE27 and3456 BBE28 records/10368 SF payloads
remain exact;SF bytes33558912/SHA`d331e6f9c10df51269dd95a6975f9e615faaf8246892d96c488c2169c42a690f`;
384 mixed prefixes,16 four-mode records/four pages/288 legacy records unchanged.
Recovered source ER27-cell comparison passes freshly within this full run.

`proc_4e79` exit0/2s pins60 closed full artifacts;`full-result.json` records actual
counts. Full stdout2292 lines/128872 bytes/SHA
`cf2e8c11aa4dedb8b951d81322f688c5c9dc12549a23a5c85e063d3b6fac31e3`;
stderr692 lines/97580 bytes/SHA
`e83c7e9290be4775cacacb46183c93dbd027ca65dd0ce65331897ae96ef863a8`.
V1 failed logs remain exact,no omitted failure or changed source expectation.

**Reading qualification:**complete small inspector/recovery/native/request outputs
read;raw-view fragments000–003/071–074 and direct stdout2262–2292/stderr656–692
actually read,not the other fragments/all raw logs/large tables. Earlier wording
about complete207-line inherited-focused stdout/eight-line stderr/full failure
stack is withdrawn because some displays were truncated. This correction does
not retroactively establish reading;raw originals/digests/machine reconciliation
remain unchanged. `raw-inspection-read.json` binds only the bounded actual scope.

## Bounded parent-author acceptance — 2026-10-15

`proc_c151` exits0/22s:exact nine-path stage,243 index/work blobs/234 protected,
152 frozen sources,all448 historical raw rows/17 same-digest bindings,555 prose
links,HEAD/origin/prefix/whitespace/input/evidence/recipe checks pass. Preliminary
tree`ff85a6c076f88834b5a5f963c974a10ed8f4cd4b`,112329-byte staged diff SHA
`04b571c38add5854a796e4652543075777ea78bd66e8e3d21234d6d2fb601254`.

**Inspection correction,2026-10-15:**the earlier claim to have completely read
all23 grouped views/113 fragment contents without truncation is WITHDRAWN.
Several actual tool displays were truncated. `stage-inspection-read.json` records
an unsupported reading assertion;its hashes/reassembly do not prove inspection.
Preserve that original pinned receipt and raw outputs,not a rewritten substitute.
The full-v2 test evidence/source hashes remain intact. Fresh actual scoped
reinspection is recorded below;it does not retroactively make the original
receipt true. No independent review or gate approval is implied.

**Recorded functional GO is limited to the preregistered supplied-facts
nonterminal bunt/context/BBE29/qualified PO-A/existing named-RBI0/ER-refusal and
legacy-compatibility slice;fresh complete parent inspection is recorded below.**
No source cause certificate,production sampler/manager integration,calibration,
independent review,full boxes or E-03/G1/G2/G3/release approval. Separately authorized
27-cell ER recovery remains provenance-qualified and does not supply engine ER.

Only this report and `docs/PROGRESS.md` may now change for documentary acceptance;
all152 executable/test/query sources remain frozen. Next:read the EXACT two-doc
acceptance delta,prove final staged tree by temporary-index reconstruction from
the authenticated preliminary tree (not proof of complete reading),rerun ALL scope/pin/HEAD/index/work/protected/
recipe/input/prefix/link/whitespace checks,then make/verify local checkpoint.
No local commit/push claim before native completion and `code-post.json`.
After that checkpoint,inspect the existing ordinary GIDP canonical/fielding path
and preregister its next bounded direct force/relay prerequisite before any new
test/code/measurement. Do not reinterpret this bunt or integrate manager policy.

## Verified local checkpoint and documentary handoff — 2026-10-15

`proc_cf32` observed exit0/21s. Exact two-document acceptance delta22082 bytes,
SHA`ae7f736346e32b64fd3cc296eb14d6dcb8a5ec4d6b3a7b21b34f363e92aaf0d4`,
prepared as five grouped views/23 fragments. The claimed complete reading is
WITHDRAWN because displays were truncated;the pinned original
`acceptance-doc-read.json` is preserved,not treated as proof of reading. Final
tree proved by temporary index,not a standalone reread of the composed final diff:
`760d9a933cd5b578cf3693c07fd39a5022af3d0a`;115573-byte diff SHA
`c6b4ff2d10948f7ec5498275d7290ac4298aba912aecf9e980f5a478b47671a2`.
21 closed acceptance artifacts pinned;all scope/source/input/evidence/recipe/
protected/prefix/link/index/work/HEAD/origin/whitespace checks pass.

`proc_06f6` observed exit0/26s. Verified local commit
`6f367c0928b2b28df2522e87cc4d83a16a9fd703`,parenta3e7f39,subject
`Validate explicit nonterminal sacrifice-bunt throw facts`. All243 committed/index/
work blobs exact,234 protected,clean main ahead8/origin804a72d unchanged/no push.
Machine acceptance/commit/postreceipt evidence is preserved;external authority
`/tmp/bbsim-sacrifice-bunt-throw/code-post.json`. Full-v2 application's152 sources
remain exact;no test/behavior changes after verification. No source data committed.

This final two-document handoff records actual postreceipt and marks the bounded
task done;only source-equality/documentary checks are newly applicable. Historical
HEAD-specific freeze helpers intentionally binda3e7f39:do not blindly rerun them
at6f367c0,waive assertions or invent a new full-test pass. Verify preserved raw
manifests plus the actual code-post tree/blob proof and exact documentary delta.
The next ready task is E-03-ground-force-double-play inspection/preregistration;
no implementation or new measurement yet. Existing qualified GIDP/contact facts
and causal/fielding unknowns must be read before choosing scope. Parent-author
acceptance remains bounded;manager NO-GO and E-03/G1/G2/G3/release status unchanged.

**Current inspection corrective action,2026-10-15:**handoff `proc_fc23` actually
completed exit0/40s at `8fafda78a81fb351198033f70b6917a75ff0d91b`,parent6f367c0,
tree`cfc014f124da4daac5670e82ef4f652606ec574d`.243 HEAD/index/work blobs and152
application sources match,clean main ahead9/origin804a72d unchanged/no push.
Short handoff commit stdout/stderr and post.json were read. However,the original
`handoff/read.json`/post.json complete-two-view reading assertion is also
WITHDRAWN:the first actual grouped display was truncated. Preserve original
receipts/pins. Fresh complete small views establish only NEW inspection,never
retroactive proof that old displays were complete.

**Fresh actual inspection:**read all65 complete <=1800-character accepted-final
views000–064,all115573 bytes/SHA`c6b4ff2d10948f7ec5498275d7290ac4298aba912aecf9e980f5a478b47671a2`,
and all19 handoff views000–018,all32723 bytes/SHA
`256625d1ba532246a5c22c9be6a44683a3cdc27e7be11cd134a305d86f082c99`.
These displays were actually complete,not the old truncated grouped views.
Fresh handoff diff uses `git diff --binary --full-index --no-ext-diff 6f367c0
8fafda7 --`;its format/hash differs from the old compact6013-byte documentary
substitution artifact. Both are preserved,not silently rebaselined. Preparation
rechecked all152 frozen source hashes/HEAD/origin/two-doc scope/clean index.
`/tmp/bbsim-sacrifice-bunt-throw/inspection-correction/read-v1.json` binds the
new views and original receipt digests. All scoped code/tests and documentation
of the accepted final diff were read;not all raw test logs/source tables,not an
independent review. No application/test/recipe/data changes or invented new
full-suite pass. Only final correction-delta/source/pin/documentary verification
and its local checkpoint remain before the force-double-play draft can proceed.
