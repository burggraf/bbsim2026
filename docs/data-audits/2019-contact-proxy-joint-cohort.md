# 2019 contact family memberships in the existing joint source cohort

## Preregistered descriptive contract — 2026-10-15

**F-01-contact-proxy-joint-cohort**, base
`4b65bd53d29657b7d61315af4b2cecd425401281`, follows qualified descriptive code
`c008864`, handoff `a787917` and the explicit postcheckpoint reading correction
`4b65bd5`. [PROGRESS](../PROGRESS.md) is the sole live status ledger.

The [previous audit](2019-contact-proxy-qualification.md) already established
5,389 SF/SH/GIDP family memberships, 4,059 four-axis geometry matches, and zero
complete literal vectors. These are known descriptive findings, not a blinded
reservation or a mechanism/rate evaluation. This contract is written before this
task's tests, implementation and new membership/category counts. It asks where
those original memberships sit in the **unchanged existing joint source taxonomy**,
without repairing them, adding them to a sampler, or counting them a second time.

### Scope and observed callers

Only four repository paths are owned:

- `audits/contact_proxy_joint_cohort.py`;
- `tests/test_contact_proxy_joint_cohort.py`;
- this report;
- `docs/PROGRESS.md`.

Protect all other **258 existing paths** and all **163 prior executable/test
sources**. Expected final tree: **262 blobs / 165 executable/test sources**.
Parent is sole writer; no delegation, dependencies, worktrees, network transfer,
source repair, fitting, model selection, engine/accounting/codec/projector/producer/
UI change or push. Canonical Rust remains sole mechanics/after-state authority.

Actual source inspection found that `runner_transitions.inventory()` already
calls `joint_contact.inventory()` before census counters. The latter validates
both original exports and their complete real-event join, groups primary results,
excludes IBB/interference from the nine included PA categories, and partitions
only `contact_out` into five source-shape bins. Its `classify()` has a fixed
first-reason precedence: result; one-out/zero-run units; trajectory; flags;
credit shape; runner effects; literal notation; root/credit match. None of these
bins is a causal certificate. `contact_proxy_qualification.inventory()` calls the
unchanged literal inventory with all evidence before type-strict whole-baseline
equality and the four-veto descriptor. These callers must remain unchanged.

Actual Rust caller inspection found `joint_contact::event_for()` first obtains
`preview::event_for()`, which gives a contact out one batter retirement; the
residual transformation removes named ground handling/credits and returns that
coarse event. It does not supply historical SF/bunt/force-relay facts. This is
context for a read-only gap audit, not authority to change that policy or claim
that source shapes describe generated mechanisms.

### Baselines and validation before any new counter

Expose one pure `inventory(raw_rows, outcome_rows, supported_game_ids, *,
native_groups, expected_census, native_joint, cohort, expected_support,
expected_proxy, expected_joint)` API. No CLI/publisher, unchecked report-only
entry point, or alternative reducer is added. Every evidence keyword is mandatory;
`None` and non-dictionary accepted baselines fail closed. Materialize iterables
without mutating rows, children, baselines or caller containers.

1. Run unchanged proxy inventory with the complete census/cohort/native-child and
   full real-event native joint evidence. Require **entire type-strict equality**
   with the accepted proxy report before new counters. Its nested literal report,
   original reasons/ordering, all children/NULLs, PA-frame table, units and splits
   remain byte-equivalent as JSON data.
2. Run unchanged legacy joint inventory and require entire type-strict equality
   with the accepted legacy joint report, excluding **only its top-level publication
   provenance**, before new counters. Do not confuse that report with native
   full-event facts: accepted legacy `counts-first.json` is SHA
   `1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca`;
   SHA `dff340fd…` identifies the predecessor's **native CSV**, not this report.
3. Preserve full accepted prior reports as separate nested output values. JSON
   comparisons distinguish bool/int/float aliases and reject nonfinite supplied
   values; do not use Python's numeric-alias equality as accepted-report proof.
4. Only then derive and reconcile the new tables. Invalid source/join, native
   map or child-census disagreement, conflicting identity or baseline must never
   reach the new counter/projection function. Accepted missing/extra children are
   preserved facts, not automatic errors or fabricated endpoint evidence. Existing supported-game list validation,
   all four exclusions and whole-game SHA256 first-hex `012` split stay unchanged.

### Fixed output grains and conservation

**Source cohort table:** exactly one contribution for every validated **real
source event**, including non-PA events and the original IBB/interference exclusions;
no-play rows stay in validation/prior reports but contribute no new event units.
Use original typed native facts for `result`, `pa`, `base_mask`, `outs_before`,
`event_outs`, `event_runs`; preserve NULL versus zero. Add `split`, `joint_category`,
`contact_bin`, `residual_reason`. Primary category uses unchanged `joint.GROUPS`;
non-PA has explicit `non_pa`. Bin/reason are both NULL outside `contact_out`;
inside it retain the exact old classifier's bin and reason (including empty
candidate reason), not a trajectory alias or newly inferred cause.

Each source cell records `events`, `labeled_events` (distinct source events with
one or more inherited family labels), and `overlap_events` (more than one label).
A labeled event is counted once there even if it has multiple family memberships.

**Membership table:** one contribution per inherited family membership, not per
runner, child or literal vector. Retain the same source-cell fields and add
`family`, `event_geometry_proxy`, `literal_shape`, `literal_reasons` (original
ordered list), and positive integer `events`. Use unchanged literal classification
and exactly the previous four-veto conjunction; do not broaden labels, infer
missing endpoints, relax a predicate or assume labels necessarily mean contact out.

Require all of the following exact, zero-tolerance invariants:

- Source cells conserve all real events/non-PA/PA units and all original joint
  included categories, exclusions, five bins, residual first reasons and original
  `(bin,base_mask,outs_before)` contexts after collapsing added dimensions.
- Global distinct labeled/overlap source counts equal original literal totals;
  per-split labeled counts equal original candidate split totals. Membership
  weights conserve every original family total
  and every `(family,split,geometry)` total from the accepted proxy report.
- Summed membership weights for one family within a source cell cannot exceed that
  cell's labeled-event count; summed families may exceed it for original overlaps.
- Splitting/refining a cell never changes nested prior reports, original PA-frame
  counts or full literal diagnostics. No output includes a ratio, fitted parameter,
  tactical opportunity denominator, player ability or causal certification.
- Input ordering does not alter any complete output field. Preserve missing/extra
  native children and nullable endpoints in the prior report, never reconstruct,
  discard, normalize or count them as new before-state opportunities.

### Observable acceptance, tests and measurement

Write failing regression/property tests before implementation. Cover positive and
negative SF/SH/GIDP descriptors; supported/unsupported before masks and outs;
multiple memberships versus distinct source events; no-play, non-PA, included
non-contact and original excluded categories; all ordinary candidate bins and
residual first-reason precedence; training/heldout/excluded splits; duplicates,
orphans, changed joins, native evidence, NULL/extra children, absent and type-aliased
baselines, late mismatch before counters, no mutation, stable ordering and complete
prior-report equality. Use inherited real-schema fixture helpers, not weakened
validators; record any new fixture defect separately from behavior reds.

Focused command: `python3 -Werror -m unittest tests.test_contact_proxy_joint_cohort -v`.
Discovery/interface failure is not a behavior red; already-green controls and
assertions unreachable after an earlier failure must remain distinguishable.

After focused acceptance, authenticate all accepted inputs/source definitions and
original path/digest bindings, then repeat two fresh read-only complete original
support/native publications with **fresh exclusive stems**. Require entire accepted
literal/proxy/legacy-joint equality before new counts. Compare every complete new
cell, null, weight, split, and nested prior value; both reports must be byte-identical.
A separate standard-library aggregation reference may reuse inherited parsers and
classifiers but must construct/reconcile counters independently. Report that limit:
**independent aggregation, not independent classification, parsing, causal review
or blinded validation**. Do not falsely describe shared classifier output as an
independent mechanism check.

Freeze all 165 current sources and retain all 163 prior identities and all 75
inherited recipe rows. Run the **entire** inherited offline/locked Rust build/tests,
format and strict all-target Clippy, Werror Python discovery/compilation, native/
source/property controls, old failure/modified-export controls, mixed prefixes,
four-mode and exact raw-page compatibility, legacy and BBE27–29 payload parity.
Retain raw exits, failures, failing source/seed identities and original manifests;
helper-only adaptations must reverse to original command/assertion bodies. A failed
full run requires a fresh entire acceptance after its cause is corrected, not only
the unreachable tail. Do not reuse/delete reserved companions or rebaseline history.

Before acceptance, genuinely inspect current source/tests and changed documentary
spans in bounded visible requests. Hashes, ASTs and parsers prove identity, not
reading. Qualify truncated displays and timing; preserve original receipts. Mark
done only after fresh current-source suite and exact scoped code/handoff identity
checks, with a separate evidence-backed bounded descriptive GO. Parent-author
acceptance is not independent review or E-03/G1/G2/G3/manager/season/desktop/release/
licensing signoff. PROGRESS records failures, current files and one next action.

## Preregistration and initial TDD evidence — incomplete

Work root `/tmp/bbsim-contact-proxy-joint-cohort/`. `proc_dcd9` exits0/11s:
258 protected paths/163 prior sources/75 unchanged recipes/23 task inputs/13
inherited inputs/6 definitions/6 predecessor authorities authenticate; twelve
preregistration artifacts closed, stderr empty. No new task tests, implementation
or membership/category measurements preceded that authentication. The original
protocol is frozen as `protocol-original.md`; original historical path/digest
bindings, raw failures and qualified prior reading receipts remain unchanged.
`reading-qualification-v1.json` separately withdraws the preregistration receipt's
unsupported full166-line old-module visibility claim:four original responses were
truncated;selected taxonomy/caller inspection,not full-file reading,was established.
Original receipt/manifests retained,no retrospective reading certification.

Eight tests were written first. `proc_4c8c` exits1/0s with missing-module
`ImportError: cannot import name 'contact_proxy_joint_cohort' from 'audits'`:
one loader error, **zero behavior methods run**, not eight semantic reds.
A twelve-line typed-empty interface then allows `proc_d4fa` to run eight methods
in0.031s: **22 assertion failures, zero errors**, all eight methods fail. The
complete source/streams and hashes are retained; later assertions after an earlier
failure are not independently certified red. Prior-report copies and format checks
were already green controls, not proof of new tables. Only status/representative
failure spans and the summary were read; no complete227-line stream claim.

Minimal validation-only implementation calls unchanged proxy/legacy inventories
and requires full type-strict baselines before an empty projection. `proc_c01a`
runs eight methods in0.044s: **five table failures/zero errors, three guard methods
pass**. This is partial implementation, not task acceptance or a fresh full suite.
All required evidence, numeric-alias/nonfinite baselines and native mismatch guards
were observed red before this validation logic. No production counters existed at
that validation-only checkpoint.

The suite is now expanded to eighteen methods, including 64 mask/out cases,
nullable versus zero PA units, primary/excluded categories, all original splits,
weighted/order/iterator controls, missing/extra children, no mutation and nested
baseline/source failures. `proc_0767` exits1/1s: eighteen methods in0.314s,
**74 assertion failures/zero errors** against validation plus empty counters;
all64 source-fixture mask/out cases reach the intended table assertion. Existing
validation/no-mutation/metadata controls are already green, not manufactured reds;
assertions after an early empty-table failure remain independently untested.
No fixture error occurred. Complete snapshots/streams and machine summary retained;
no complete758-line reading claim. Production table construction/reconciliation was
then written. `proc_1633` exits0/1s: **eighteen Werror methods pass in0.264s**;
complete focused stderr actually displayed in three bounded requests. Current137
module/273 test lines actually displayed in bounded spans after this focused pass,
before actual task measurement/full suite;`source-inspection-v1.json` binds timing
and hashes. This is parent inspection,not independent review or full acceptance.
Reference typed-empty `proc_3d93`:four methods/67 assertion failures/zero errors;
`proc_7d97` exits0/1s:four controls in0.250s. Set-based distinct event aggregation
and direct four-axis geometry agree across64 contexts,overlaps,weights,NULL/zero
and splits;inherited parsers/labels/literal diagnosis/joint classification remain
shared,not independent causal validation. Current51-line reference/43-line test
actually displayed after those controls,before actual new task measurements.

`proc_6400` exits0/13s:258 protected/165 current sources/all75 original recipe rows/
23 task inputs/24 closed focused-reference artifacts authenticate. The prior epoch
helper is adapted reversibly only for live HEAD/tree/untracked scope and truthful
display;all original source/input/recipe/assertion bodies are retained. These are
identity/history checks,not a fresh executable suite. Complete freeze stdout and
empty stderr displayed. Production137/test273 sources remain unchanged since the
first focused green;`tested-source-v1.sha256` freezes165 current bindings.

Actual repeated offline measurement was started as **proc_1c40**:
`python3 -Werror /tmp/bbsim-contact-proxy-joint-cohort/measure-v1.py` with separate
`measurement-v1.stdout/.stderr`,exclusive literal/native and reference v1 stems.
Do not rerun occupied stems. Entire accepted literal/provenance then proxy/legacy
equality precedes new projection/reference cells;no new scientific findings or
completion are asserted until native lifecycle/result/closure inspection.
`proc_69fc` exits0/18s:full-v1 prepared with89 recipes including75 original rows,
all195 inherited driver lines reversibly preserved plus current source/reference
checks;private native-output AST control passes. Complete preparation streams
actually displayed. Preparation is not full execution;subsequent native run status
is below. Historical687Rust/508Python does not certify the new sources.

## Actual repeated source reconciliation — full acceptance pending

**proc_1c40 exits0/388s** (driver386.875s),stderr empty. Both fresh complete offline
literal/source/native publications match the entire accepted original support and
provenance before the task inventory. Each task inventory validates the entire
accepted proxy and legacy joint report before projection;all conservation checks
pass. Every full cell/ordered literal reason/typed nullable unit matches independent
set aggregation/direct geometry,with inherited classification shared. No independent
causal certification or blinded validation is claimed.

Both new reports are **1,666,493 bytes**,SHA256
`c871add3c72e6ee9f9ef489385eaf8f506542fb8f1b84e954525093ce2773f71`:
`/tmp/bbsim-contact-proxy-joint-cohort/report-1-v1.json` and`report-2-v1.json`.
There are **1,549 source cells/172 family-membership cells**,weighting192025 real
events and5389 distinct labeled events/5389 family memberships/zero observed
overlaps. These grains only coincide for this observed no-overlap population;
conflicting-label fixtures still have multiple memberships per distinct event.

The complete legacy source taxonomy remains unchanged:185703 included PA,753
excluded IBB,61 excluded interference,186517 total PA and5508 non-PA. Contact bins
remain `[17188,19528,5379,536,39614]`,sum82245. Original literal5389/zero full
vectors and prior family/split/geometry4059/1330 weights remain unchanged.

| Original family membership | Geometry | Legacy primary category/bin | Exact first reason | Memberships |
| --- | --- | --- | --- | ---: |
| SF | true | contact_out/residual | non_inplay_result | 1123 |
| SF | false | contact_out/residual | non_inplay_result | 20 |
| SF | false | roe/NULL | NULL | 7 |
| SH | true | contact_out/residual | non_inplay_result | 430 |
| SH | false | contact_out/residual | non_inplay_result | 313 |
| SH | false | roe/NULL | NULL | 33 |
| GIDP | true | contact_out/residual | non_single_scoreless_out | 2506 |
| GIDP | false | contact_out/residual | non_single_scoreless_out | 957 |

Thus **all4059 geometry-proxy memberships remain residual** in the unchanged old
taxonomy. All labeled memberships partition into5349 contact_out/residual and40
roe;no labeled membership occurs in an ordinary candidate bin. This does not
repair the literal vectors,justify treating every label as a clean retirement,
select a producer policy or authorize reassigning old bin weights. Real source
SF/SH first reasons are`non_inplay_result`;the synthetic fixture's SH
`runner_effect_or_unknown` exercises a different accepted primary result,not a
replacement source classification. No before-PA frame is promoted into a tactical
attempt/opportunity denominator and no rate is fitted.

Authority:`measurement-result-v1.json`,`measurement-artifacts-v1.sha256` closes52
artifacts,and`measurement-lifecycle-v1.json` binds the actual native exit. Prior
support SHA7442e3ef,proxy SHAe841457a,legacy JSON SHA1573fb84 and native CSV
SHAdff340fd remain distinct unchanged bindings. All165 sources/258 protected paths/
23 inputs authenticate before and after measurement;no historical rebaseline.
Raw stdout/stderr and both full reports retained. Inspection used the first24
result lines/first12 stdout lines and complete machine scalar/cross-split summaries;
**not** a claim of complete214-line result/full raw stdout reading.

## Fresh complete verification — scoped author checkpoint pending

**proc_e26f exits0/542s** for
`bash /tmp/bbsim-contact-proxy-joint-cohort/full-v1/full-checks.sh`,with separate
`full-v1.stdout/.stderr`. **proc_1523 exits0/13s**,independently inspecting the native
exit,source/recipe hashes,all machine summaries and exact existing compatibility
assertions;this is a parent-authored evidence check,not independent model review.

Fresh checks:offline/locked Rust build/tests/fmt/strict all-target Clippy;
**687 Rust tests across43 zero-failure summaries;526 Werror Python discovery tests
in38.966s**,plus separate3/3/1/1/1/1/3/1/4 controls and ten OK markers;Python
compilation;all inherited source/native/property/failure/modified-export,384-prefix,
four-mode/16-record/four-full-UTF8-page,legacy and BBE27/28/29 parity checks. All
existing exact record/payload/byte/hash assertions remain unchanged. Current165/
prior163 sources,258 protected paths and89 recipe rows (all75 originals retained)
authenticate;all195 inherited command lines reconstruct reversibly.

`full-result-v1.json` SHA256
`bf690ea541f6e1b790d88c070366eac7c3ca3d4a6c6403a7b34fc95fde81b9de`;
`completed-v1.sha256` closes **140 full-run artifacts**,after native lifecycle.
Raw stdout2642lines/157757bytes/SHA
`55c9094401085bf4df99465d78d03155e5b2b6e4b12d6da79c4d79df66dfd1e5`;
stderr864lines/124116bytes/SHA
`7c4db86322853330ca2a4cdd2f22242782526e0c01e8f48e155075210fc63183`.
Inspection displayed complete six-line inspector stdout/empty stderr,full stderr
839–864 and stdout2631–2642 plus complete machine scalar/test-summary fields.
**No complete raw-log reading claim.** Original streams/history remain untouched.

At that verification checkpoint the scoped code/author commit was still pending.
No executable source/test changed after focused green or measurement;later changes
are documentation only.

## Bounded parent-author acceptance — 2026-10-15

**GO for this descriptive reconciliation only.** Verified local code
`4b69d93141b1242c0aa5ad7cd7356f6716fad70f`,parent4b65bd5,
tree`db869c43a65500e9ee12e7c08b8a57d2e5149203`. **proc_ac58 exits0/22s**:
one scoped four-path commit followed by explicit existing-identity postcheck,
262 exact HEAD/index/work blobs/258 protected/165 sources/89 recipes/578 prose
links;clean main ahead23 at capture,origin804a72d unchanged,no push. Authority:
`/tmp/bbsim-contact-proxy-joint-cohort/code-post-v1.json`. Commit streams and
postcheck evidence retained;2 prose-link/1 tracking controls pass without weakening
broken-link checks. A private candidate is not actual staging. All55 bounded
before/after documentary views,including the entire324-line new report,were
actually displayed before that commit. Earlier137 module/273 test inspection
identity/timing is retained;no retrospective full-stream reading certification.

Observed source/family grains,whole prior reports,all full cells and exact
conservation are accepted with fresh687Rust/526WerrorPython/full inherited evidence.
The original5389/zero literal vectors are unchanged. All4059 geometry memberships
remain residual;40 other labels are roe. Shared classification is explicitly not
independent causal validation. No mechanism/rate/denominator promotion,fitting,
producer/manager integration,E-03/G1/G2/G3/G0–G5 signoff,desktop/season/release or
licensing approval follows. Missing evidence stays unknown.

Next ready **F-01-contact-fact-evidence-boundary** must first preregister a small,
zero-new-count map from exact canonical SF/bunt/force-relay required facts to the
already retained source definitions/available exported fields. It must distinguish
direct evidence,derivation,unavailable facts and modeled assumptions,not infer
retouch/handling/force/relay/timing or scorer judgments from these labels. No new
source transfer,endpoint repair,producer code or fitted rates are authorized by
this audit. Actual gaps should produce a precise source/licensing/modeling decision,
not another implicit gate waiver. PROGRESS remains the sole live next-task ledger.

Handoff preparation failure retained:`proc_440f` exits1/18s before staging/commit;
Python3.15 Werror raises`DeprecationWarning` for`os.path.commonprefix`. A text-prefix
regression reproduces seven runtime errors before the helper fix;all seven empty/
Unicode/path-like literal string cases then pass under unchanged Werror. A direct
character-prefix calculation replaces the deprecated function;filesystem commonpath
would not preserve text semantics. Original helper/streams/empty v1 views retained,
reversible private handoff-v2 uses fresh stems. Project source/tests,actual counts
and the fresh full-suite identity are unchanged;no old publication was overwritten.
