# E-03 — explicit caught sacrifice-fly facts (bounded, nonterminal)

## Preregistration — 2026-10-14, before tests or implementation

D-20/D-21 continue common-mechanism fidelity,not manager integration or a gate
waiver. Runner-only projection is verified at local
`f965b77bb3a8746446e58b01505dbe050f2d611c`,clean main ahead6;origin804a72d
unchanged/no push. That supplied-decision projector does not create a steal policy.
The next concrete gap is a caught sacrifice fly:existing ordinary-air facts
require held runners and AtBat,and explicitly reject SF credits;legacy generic
SF credits exist but do not certify this causal episode/RBI/fielding coverage.

Exactly ten owned paths:
- `docs/PROGRESS.md`
- `docs/engine/caught-sacrifice-fly.md`
- `engine/src/lib.rs`
- `engine/src/reducer.rs`
- `engine/src/rules.rs`
- `engine/src/air.rs`
- `engine/src/accounting.rs`
- `engine/src/codec.rs`
- `engine/src/sacrifice_fly.rs`
- `engine/tests/caught_sacrifice_fly.rs`

Protect230 other existing blobs;148 prior executable/test sources,150 final
sources;240 final repository blobs. Parent sole writer. No transfers,dependencies,
delegation,producer/sampler/CLI/UI/manager changes,push or full-box/gate signoff.
No fitting,thresholds,source normalization or new real-data measurement.

## Inspected rules and real flow

Read PLAN/EXECUTION/current ledger,canonical constructor/reducer/batter-charge
flow,ordinary-air validator,codec,independent RBI/fielding/accounting/clean-ER
paths,and existing ordinary-air/accounting/walkoff fixtures. Cached2019 rulebook
(`/tmp/bbsim-2019-rulebook.txt`,no new transfer):9.08(d)(1),9.04(a)(1),5.09(a)(1)
Catch Comment/5.09(c)(1). SF requires before two outs,a ball in flight handled by
an outfielder or an infielder running in the outfield,and a runner scoring after
the catch. Fair/foul both qualify. Catch Comment permits leaving at the first
fielder touch,not only after completed possession. Therefore legal retouch and
scoring-after-catch are separate supplied facts;do not invent a stricter departure
clock or infer chronology from move/credit order. Dropped-ball9.08(d)(2),appeals,
multiple retirements and unaided/error/fielding chronology remain outside scope.

The before team stores eligibility and pitcher/DH,not all defensive positions or
ball location. Qualifying handling must be explicitly supplied,not guessed from
batting-order index,PO actor,trajectory,source label or absence of errors.

## Bounded representation and behavior

Add optional immutable transition context `CaughtSacrificeFlyFacts`,with named
scoring_runner_id and explicit handling,legal-retouch and scoring-timing evidence.
Proposed closed evidence enums distinguish Unknown from qualifying/legal facts;
scoring timing includes AfterCatch and a known BeforeCatch refusal control.
These are supplied modeled/reviewed assertions,not reconstructed source facts.

Public additive APIs:
- `apply_with_caught_sacrifice_fly_facts(before,event,facts)`
- `EventTransition::try_new_with_caught_sacrifice_fly_facts(before,event,after,facts)`
- `caught_sacrifice_fly_facts()` immutable getter and event-local RBI contribution.

Reuse Event's existing OrdinaryAirCatchFacts for named batter/fielder/direct
legal non-bunt catch. All three existing in-flight contact labels may qualify
when the separate9.08(d) handling criterion is supplied;do not equate SF with
only a source Fly label. The new context supports before outs0/1,one named before
Third runner scoring Home,other occupied runners explicitly holding with carried
pitcher responsibility,batter Out before legal First,one out/one run,one PA,
SF/NoHit/RBI1/Run1/PO1 and no AB/Assist/error/other scoring units. Whole-chance
NoMisplay review is mandatory. Actual current batter/pitcher and before eligible
fielder remain canonical requirements. Named scorer must be the actual Third
runner;do not use movement order as a winner/timing certificate.

Reject all Bottom inning>=9,including cases old generic events could represent.
No walkoff policy,score clipping,second fielding roll,third-out suppression or
resampling. Refuse outs2,missing/unknown evidence,wrong participants/handler,
wrong/missing/scaled credits,missing causal air facts,wrong review,runner-only/
substitution/compound/retirement/error shapes,other advances,non-Third scorer,
multiple scorers and incompatible terminal-hit facts. Strong existing canonical
contradictions keep their errors;fixture/caller priority differences must be
investigated rather than masking validation. Preserve caller state on refusal.

Optional facts absent:all legacy public constructors,Event literals,accepted
behavior,BBE25/26/27 bytes and normal/pretty Debug remain exactly unchanged.
No retroactive tightening of credit-only legacy SF events and no relaxation of
the ordinary-air SF refusal without the explicit new context. New facts use a
new BBE28 tail including bound air facts;decode revalidates canonical context,
actors,effects and evidence. Unknown tags/trailing bytes/tampered facts refuse.
Do not expose a mutable context or a validation-bypassing constructor.

## Independent accounting and ER boundary

For a validated supplied episode:PA1/BF1/AB0/H0/SF1/RBI1/R1/PO1/A0/E0/PB0,
current pitcher receives BF/out/SF while the scoring runner's carried pitcher
receives its run charge. RBI comes from validated context,not merely a supplied
RBI credit. Existing air proof supplies event-local named PO1/A0 and whole-chance
review E0/PB0,not official whole-game completeness. Coverage cannot restore a
prior None;multiple credited/logged effects must conserve independently.

Do not accidentally expand positive-prefix earned-run proof. `clean_play`
currently recognizes ordinary_air_catch;new SF advancement would otherwise
silently enter that held-runner ER family. Explicitly keep the new context outside
clean-ER proof,with regression tests through a completed half and inherited/current
pitcher charges. First-two-out positive-prefix ER and general9.16 remain deferred.

## Observable acceptance before implementation

1. Authenticate all237 committed/index/work blobs,230 protected,148 prior sources,
   predecessor recipes/evidence/input/export/report pins and cached rulebook.
   Before changes preserve six current-core same-digest snapshots and complete
   baseline engine source for fresh old/new compatibility. Preserve historical
   manifest rows;changed historical paths use explicit (path,digest)->same-digest
   snapshots,never silent rebaselining or dropping assertions.
2. Missing API compiler red,then genuine semantic reds against typed refusing
   scaffolding,with already-green refusal controls and fixture defects separate.
3. Independent mask4..7 x outs0/1 x half/nonterminal inning x all contact labels x
   queued cursor x DH/no-DH geometry/identity/charge checks;move and credit reversal,
   canonical apply/try-new/codec equality,one PA/BF/cursor advancement,one out/run,
   held-vector/charge/credit conservation. Exhaustive Bottom>=9 refusal plus Top
   late positives and score-ceiling failure. Unsupported geometries stay refused.
4. Missing/unknown/wrong facts,cause/review/credit units,NoHit/AtBat contradictions,
   bad/DH/foreign actors,malformed before/after,wrong scorer/charge,early run timing,
   two outs/compound/terminal facts and byte tampering have explicit controls.
   Absent new context does not certify an air+SF play;legacy generic SF behavior
   and existing held-air/walkoff controls remain exact.
5. Independent credited/classified batting,pitching,fielding,RBI/run accounting,
   contiguous prefixes,prior-None propagation and completed-half ER refusal.
6. Freeze all150 sources/recipes before full verification. Run offline locked
   full Rust tests/build/fmt/all-target Clippy -D warnings,Werror Python discovery/
   compilation and every inherited source/native/property/legacy record/page check.
   Historical checks may use only preregistered same-digest mappings;current150
   source manifest independently verifies the changed engine. Add fresh exact
   old/new BBE27 explicit-walkoff records+normal/pretty Debug parity,not only the
   legacy288 no-context records. Preserve failing seeds/raw evidence and counts.
7. Explicit bounded raw inspection scope,full staged-source/docs inspection and
   exact documentary-delta reconstruction;scope/HEAD-index-work/protected/input/
   evidence/recipe/prefix/links/tree/whitespace checks and verified local receipt.
   Generated fragments/hashes are not actual-reading proof. Parent-author
   bounded acceptance only,not independent review/calibration or E-03/G1/G2/G3 GO.

No owner-input blocker is currently identified. Unknown legal/representation
questions must be resolved from the canonical flow/rulebook or recorded before
implementation;do not invent factual certainty to keep progress moving.

## Authenticated TDD checkpoint — before behavior implementation

`proc_8216` exits0/15s:237 committed/index blobs,230 protected,148 prior
sources,33 full/32 prefreeze/19 acceptance pins,seven full/five projection
recipes,11 inputs/four exports/report and exact protocol authenticated. Six
current-core same-digest snapshots and16-source baseline engine preserved.
Cached rulebook SHA256 `bbbfee1a16b23cb1f5a3cb9a4a46864617698a122a756d59151f32c1c2e63090`.
Evidence root `/tmp/bbsim-caught-sacrifice-fly/`;external originals may expire.

`cargo test --offline --locked --manifest-path engine/Cargo.toml --test caught_sacrifice_fly`:
`proc_476d` exits101/2s with seven expected missing-API diagnostics. Typed
refusing scaffold (`UnsupportedRule`,gettersNone) then `proc_a939` exits101/7s:
three genuine semantic failures at refused valid projections,two already-green
legacy/refusal controls,no compiler warnings. The fully observed-half setup's
legacy Triple succeeds before its new SF refusal;not a fixture-setup red.
Scaffold/tests/raw logs preserved externally. Added explicit matched PO+handler
eligibility controls (so mismatched credits cannot mask the role predicate),
non-Third scorer/other-advance refusals,legacy-constructor refusal and known
Unknown/BeforeCatch codec-tag revalidation before behavior implementation.
No physical/causal/source certainty is inferred from these synthetic fixtures.
`proc_a788`:four genuine semantic failures/two controls. Explicit canonical error
priority before Unknown handling then makes `proc_b0e3` five genuine failures/two
controls across seven methods;raw logs/scaffolds/test snapshots retained.

## Initial implementation checkpoint — before freeze

Optional private transition context now goes through both reducer and immutable
constructor's shared batter-charge/air validator. New module validates explicit
handling/retouch/after-catch,full before-runner vector/exact six credit units and
canonical participants/roles/origins/outs/run conservation before unknown scope.
Strong contradictions precede unrelated Unknown handling. No new Event literal
field or mutation API;old constructors passNone. New SF has raw BBE28 tails for
bound air facts then named scorer/three evidence tags;old25/26/27 paths stay exact.
Event-local RBI comes from validated context,not merely a supplied RBI credit.
Existing named air PO/A and reviewed E/PB axes apply;SF context is explicitly
outside the older clean-ER proof. No producer/policy/RNG/source change.

`proc_0432` exits0/9s:seven new methods pass in1.14s,3456 independent supported/
permuted frames,all65527 late-Bottom refusals,named defense eligibility (including
matched fact+PO mutations),unknown/bad facts/units/movements,canonical priority,
BBE28 truncation/trailing/identity/valid-Unknown/BeforeCatch/invalid-tag controls,
legacy absent-context refusal/generic-SF/held-air controls and completed-half
ER refusal/prior-None fielding propagation. `proc_60ed` exits0/30s:all-target
Clippy -D warnings and157 unchanged accounting/air/walkoff/projector methods pass.

Fresh explicit-BBE27 parity `proc_6ed1` exits0/5s against preserved f965b77 engine:
1944 matrix attempts retain1152 accepted/264 IllegalCredit/528 UnsupportedRule;
1152 records/3456 binary+normal/pretty Debug payloads match byte-for-byte,
10238352 bytes SHA256 `e6edb91e97d0042e3123fc4b0c9248e6554ad37288a17850f19219853f7db189`.
The original walkoff caller remains exact;only an appended export test serializes
its accepted frames,retaining all its assertions and checking the exact counts.

## Verification recipe lifecycle and completed v3 full run

`proc_c5e6` successfully freezes150 current sources/all148 original raw prior
rows/six same-digest current-core snapshots/230 protected/five recipes/38 completed
prefreeze pins. Full preparation then fails before generating/running its script:
the manifest scan attempts to read its not-yet-created future full-recipes file.
Raw AssertionError and partial `final/` remain. This is a preparation lifecycle
defect,not an engine/test failure. Five exact v1 recipe snapshots,raw manifests,
closed logs and receipt are preserved in `preparation-v1/`;no blind exclusive rerun.

Isolated exact scan-loop regression `proc_8ce9` has one expected failure/two
controls;minimal known-future-file exemption makes `proc_b688` three passes.
Unknown missing input manifests remain errors;materialized row digests are still
checked. `proc_481e` exits0/2s:new six-recipe/72-pin v2 freeze preserves identical
150 application sources and all38 original prefreeze rows as exact prefix. Old
recipe rows remain bound to five explicit same-digest snapshots,not rebaselined.
Historical source verifier composes five older and six current source snapshots
using(path,digest),preserving all raw manifests and their original expected hashes.
Fresh `final-v2/` preparation reversibly reconstructs the entire inherited script,
retaining commands/assertions;12 full recipes/new explicit-BBE27 parity,baseline
record equality,predecessor pins and the lifecycle regression are checked. The
143-line script/22-line compatibility helper and the309-line core diff/165-line
module/591 original test lines had truncated earlier displays;complete reading
then was not established. Inspection nevertheless identified missing dedicated
preregistered compound/terminal controls;later complete current staged-diff reading
is recorded below,not retroactively attributed to those attempts. `proc_a65f` was intentionally stopped with SIGTERM
before changing frozen tests;its partial full outputs remain,pinned only after
confirmed stop. No full success is claimed. `verification-v2/` preserves all150
raw source rows,14 same-digest recipe bindings,the old SF test snapshot and the
closed-output manifest/stop receipt;no raw row or partial output is rebaselined.

An eighth method explicitly checks six competing causal fact families (also
against Unknown handling),personnel/multiple retirements/multiple scorers,fully
annotated two-out metadata and both incompatible terminal-hit APIs. `proc_630e`
exits0/10s:all eight pass (1.48s),then all-target strict Clippy passes. These are
additional already-green refusal controls,not a new semantic red or engine fix.
Only the test's appended method changed;production source/query bytes remain
exactly frozen. Original591-line test prefix remains exact. Historical mapping
adds its explicit same-digest old-test snapshot alongside prior mappings.

`proc_becd` exits0/3s:three lifecycle controls pass,fresh v3 freeze verifies150
current sources and six recipes,all148 prior+150 v2 original raw source rows,
230 protected and114 completed prefreeze pins (all72 v2 rows exact prefix).
Twelve source path/digest bindings include the old-test snapshot;14 v2 recipe
bindings plus v1 bindings preserve every original recipe row. New `final-v3/`
preparation reversibly reconstructs all inherited commands/assertions with12 full
recipes and valid bash syntax. `proc_eddf` exits0/290s on the full inherited
script. `proc_c091` exits0/1s:665 Rust tests/40 zero-failure summaries,424 Werror
Python tests in30.581s,all offline locked build/fmt/strict all-target Clippy,
Python compilation,inherited source/native/property checks,16 complete seeded
four-mode records/four raw UTF-8 pages,288 legacy records/864 payloads and the
fresh1152 BBE27 records/3456 payloads are verified. The384 mixed prefixes repeat
exactly through unchanged callers;no producer integration is inferred.
Forty-six completed full artifacts pinned,including raw112497-byte stdout SHA256
`af0190dd0a552862c98947d5855321c35660c2ab767ab854b2877801278eb963` and95323-byte
stderr `e1ec3f284af67f61cb56b947ae8c6885ac55813d86c0f69452f94259dbbbccf1`.
The70901-byte generated raw view remains intact. Actual direct bounded reads are
fragments000–023 and067–070 (each <=1500 bytes):all Rust result summaries,Python/
build/Clippy/compile markers,inherited controls,complete prefix/walkoff/parity
small logs and diagnostic aggregate/bookends. Middle024–066/large diagnostic row
table were deliberately not claimed read;neither the entire generated view nor
full raw logs/tables were manually consumed. Dataset/report/payload reconciliation
is machine evidence,not row-by-row manual validation or independent review.

## Bounded parent-author acceptance — 2026-10-14

`proc_1956` exits0/16s:ten owned paths,240 exact index-work blobs,230 protected,
150 current sources,46 completed full/114 prefreeze pins,12 full/six SF recipes,
11 inputs/four exports/report,original protocol prefix,553 prose links,whitespace
and tree checks pass. Preliminary tree `01f56393cc5f5182de9a295c43441da5180d5965`;
104770-byte diff SHA256 `ae49d65c65b05cedd56642b5455e4d3af8c7ab0715373d4e1ad0707dd6134a7a`.
All105 staged-diff fragments000–104 were actually read in complete nontruncating
displays,including all six changed core paths,new165-line module,709-line test
and scoped documentation. This establishes current diff inspection,not earlier
truncated-reading claims. Bounded raw scope remains000–023/067–070 only;large
raw logs/diagnostic tables and record payloads are not claimed manually consumed.

Parent-author GO for the supplied-facts nonterminal canonical SF family:explicit
handling/legal retouch/after-catch evidence,Third scorer/other holds,one out/run,
independent event-local accounting,immutable context/BBE28 revalidation,legacy
None-context compatibility and continued positive-prefix ER/late-Bottom refusal.
The eight tests/full v3 checks satisfy this bounded source contract;synthetic
supplied assertions are not source certificates or measured attempt probabilities.
No independent review,producer/sampler/manager integration,calibration,full-box
completion or E-03/G1/G2/G3/release signoff. Manager remains NO-GO.

No application source/test/query change after v3 freeze. Only these documentary
corrections/acceptance remain;exact temporary-index reconstruction plus repeated
source/input/evidence/recipe/protected/scope/index-work/prefix/link/tree checks
must precede the local commit. Commit/postreceipt are pending,not remotely durable;
no push is authorized. Next inspect/preregister a bounded sacrifice-bunt episode,
without extending this protocol or waiving outstanding gates.
