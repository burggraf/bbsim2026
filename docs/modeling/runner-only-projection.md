# P-01 — explicit runner-only steal projector (standalone, synthetic)

## Preregistered contract — 2026-10-14, before tests/implementation

D-20/D-21 prioritize the common generated runner-only mechanism gap. Existing
engine tests show supplied steals/CS are mechanically representable, while the
current generator emits neither. The accepted CS audit identifies literal NULL
endpoints, not missing targets; it does not authorize historical cause inference
or empirical attempt/success fitting. This prerequisite isolates projection of
an already supplied modeled decision, not choosing whether/when to attempt.

Base `278ac659df6f4578b0f4cc90f98508f934fd97d4` is postverified:clean main ahead5,origin804a72d unchanged. Exactly four owned paths:
`docs/PROGRESS.md`, `docs/modeling/runner-only-projection.md`,
`engine/examples/support/runner_only.rs`, `engine/tests/runner_only_projection.rs`.
Parent sole writer. Protect233 other existing blobs and146 prior executable/test
sources;final148 sources. No old source/test/query modifications,dependencies,transfers,delegation,
production/CLI/UI/manager integration,push or gate signoff. No real-data measurement.

Proposed interface: a supplied named before-runner, explicit attempted next base,
explicit ordinary reviewed modeled cause, and Safe or Caught result. A caught
result supplies one named putout fielder and zero or more explicitly named assists;
actors must remain eligible under the canonical before defense, with no invented
catcher/position inference or fallback actor. Reject unsupported/unknown cause.
Clear Second/Third only; no home, multi-steal, pickoff, error-saved attempt,
compound PA/pitch, intermediate chronology, substitution or terminal-policy scope.
Reject all Bottom inning>=9,even where the old canonical engine could represent
a runner-only event;do not relax inherited kernel/dispatcher guards.

The helper consumes no RNG and never fits a model. A caller has already supplied
an attempt/result; this module does not define a time-at-risk denominator, attempt
hazard, success probability, selection policy or game-loop clock. No NoAttempt
transition: not choosing this helper leaves caller state untouched. Separate
manager policy remains NO-GO.

Construct one RunnerOnly(StealAttempt) event with full explicit before-runner
moves in canonical origin order: the selected actor goes to the next clear base
with exactly one SB, or Out with exactly one CS and supplied PO/assists. Other
runners explicitly hold with their existing carried pitcher responsibility; no
imputed source holds. These are synthetic modeled states,not reconstructed source
records. A third-out caught attempt supplies explicit modeled nonforce timing
metadata with no home crossings; canonical apply decides half normalization and
end state. The attempted base is validated by this versioned bounded projector;
the legacy event DTO does not claim a richer stored throw/target/touch history.
No PA/AB/BF/lineup-cursor movement; outs increment only on caught result. Credited
units are testable but do not create full official SB/CS/fielding/ER coverage.

Acceptance before implementation:
- Missing API red, then genuine semantic reds versus a conservative refusing
  scaffold,distinguished from already-green refusal controls/fixture mistakes.
- Independent clear-target oracle across all masks/outs/halves and both target
  roles,Safe/Caught outcomes,ordinary/extra Top halves,DH and no-DH before-defense
  membership,inherited charges and explicit held moves.
- Canonical before/after/codec equality,independent credited SB/CS/PO/A/out and
  zero PA/BF/cursor conservation;third-out survivor clearing is normalization,
  not a second retirement. No caller-state mutation on success/refusal.
- Unknown cause,foreign/next-batter runner,occupied/non-next/home target,ended or
  malformed state,Bottom>=9,wrong/DH/foreign/duplicate fielders and unsupported
  credit shapes refuse without station fallback or actor substitution. Do not
  impose a blanket same-fielder PO+assist ban if canonical rules permit it.
- Bounded mixed PA/runner-only prefixes use the existing explicitly selected
  ConditionalHits model and explicitly supplied advancement word/cause. No new RNG
  consumption;the test caller's supplied word sequence/count must be unchanged
  by inserting this no-RNG projector. Do not claim production integration.
- Freeze all sources/recipes before full checks;offline locked full Rust/build/
  fmt/all-target strict Clippy,Werror Python discovery/compilation,all inherited
  source/native/property controls and exact legacy records/pages remain mandatory.
  Preserve all historical manifest rows and same-digest source mappings.
- Complete bounded raw/staged inspection,scope/index-work/protected/source/input/
  evidence/prefix/links/tree/whitespace verification and local checkpoint receipt.
  Parent-author acceptance only,not independent review or E-03/G1/G2/G3 approval.

## Actual inspected contracts and bounded clarifications

Read `PLAN.md`/`docs/EXECUTION.md`,D-20/D-21 and existing canonical participant,
credit-role/out/third-out/destination/reducer paths,CS and fidelity fixtures.
Canonical `validate_participants` requires the actual queued batter even for
runner-only events:the event carries that named participant but no PA/BF/cursor
advance. Use the actual before pitcher;carried responsibility remains unchanged.
`apply` remains the sole transition constructor;no new core DTO/fact/codec.

`StealDecision { runner_id, target: Destination, cause, result }` is proposed;
`StealCause::{OrdinaryReviewed,Unknown}` and
`StealResult::{Safe,Caught { putout, assists }}`. A supplied target is Second or
Third and must be the selected before runner's next clear base for either result.
Cause OrdinaryReviewed is an explicit modeled assertion,not source inference.
At most one assist per supplied actor in this bounded projection;repeated assists
require unmodeled credit chronology and refuse. PO+assist by the same eligible
actor is not blanket-banned. Safe has no defensive credit fields. Empty/foreign/
DH actors refuse through canonical credit eligibility;no fallback fielder.

Third-out Caught uses explicit modeled Timing/no home crossings;canonical apply
normalizes survivors and determines any Top-ninth game ending. Bottom>=9 remains
outside this standalone family. No replay eligibility,official coverage or cause
certificate is inferred from source flags/NULL endpoints.

Authentication before tests checks all234 committed/index/work blobs,233
protected,146 old sources,completed predecessor evidence/recipes and accepted
inputs,exports/reports using preserved manifests. Do not rerun predecessor
HEAD-specific historical check-freeze helpers after their accepted commit.

## Checkpoint — authentication and TDD, before freeze

Evidence root `/tmp/bbsim-runner-only-projection/`;parent remains sole writer.
`proc_895e` exits0/14s:all234 committed/index blobs/233 protected/146 prior sources,
11 accepted inputs/four exports/prior report and31 full/nine measurement/33
prefreeze/four description/seven full/six measurement predecessor recipe pins
verified before tests. Protocol original remains an exact prefix.

`cargo test --offline --locked --manifest-path engine/Cargo.toml --test runner_only_projection`:
`proc_bbba` missing-module compiler red;conservative typed/refusing scaffold
`proc_786e` produces four genuine semantic-failing methods/one already-green
refusal method. `proc_8706` exits0/6s,five methods/format;version constant compiler
red `proc_5062`,then `proc_7c51` verifies six methods and all-target Clippy `-D warnings`.
192 supported frames,2880 independent geometry attempts/96 accepted,all65527
Bottom>=9 refusals plus Top9 ending/Top10 continuation,named roles/DH/no-DH,
unique assists,same eligible PO+assist,full held-vector charges,canonical
apply/try-new/codec equality,independent credited accounting and no PA/BF/cursor
advance. No new official fielding coverage certificate is produced.

Standalone module has explicit policy version `ordinary-runner-only-projection-v1`.
It performs bounded cause/target/unique-assist checks,builds the full supplied
runner vector and calls canonical apply. It does not modify reducer/rules/codec,
producer,dispatcher,sampler or legacy interfaces. During interface-only phase,
the Top-late fixture was rebuilt with the correct offense rather than flipping
half over old offense's runners;independent third-out half/inning checks were
added before the semantic red.

Supplemental native mixed-prefix control preserves the complete unchanged
`engine_fidelity_checkpoint.rs` source,rewriting only module paths,then appends
`prefix-test.rs`. Its first run (`proc_7c51`) fails86 vs83 caller words;the preserved
instrumented repeat `proc_6f92` confirms seed1:69 ordinary PA+3 OutOfParkHomeRun+
27 runner frames/14 hit words. The independent test oracle incorrectly excluded
HR PAs,not a projector/RNG defect. Correct its PA taxonomy to canonical PA,
OutOfParkHomeRun and FieldersChoice;do not alter production or count runner frames
as PA. `proc_53e4` exits0/18s:384 prefixes/three explicitly selected models/128
seeds each/two exact repeats. Uniform13331 total/4165 runner-only(2244 caught,
1921 safe);Pooled13332/4158(2241/1917);BaseOut13352/4189(2251/1938). All caller PA
and hit words/next check-word,continuity,codec,permuted move/credit after-state,
PA/BF/out/SB/CS/run accounting agree. These are synthetic fixture schedules,
not empirical rates,a management algorithm or production integration.

Reproduction after source freeze uses the complete inherited full-check script
plus `python3 -Werror /tmp/bbsim-runner-only-projection/prefix.py NEW_EXTERNAL_DIR`.
The native driver/recipes/logs are external and may expire;committed six-method
synthetic regression/property tests run through normal Cargo tests. Full checks,
final staged inspection and bounded acceptance are still pending here.

## Frozen full verification — 2026-10-14

`proc_22c0` exits0/2s:148 frozen sources (all146 prior manifest rows retained),
233 protected blobs,five projection/seven inherited full recipes,32 completed
prefreeze artifacts. Exact inherited script path/count adaptation and shell
syntax verified. No source/test/query change after this freeze.

`bash /tmp/bbsim-runner-only-projection/final/full-checks.sh` (`proc_4bb8`)
exits0/286s:657 Rust tests/39 summaries,424 Werror Python tests/30.824s,
offline locked build,fmt,all-target Clippy `-D warnings`,Werror compilation and
all inherited source/native/property controls. This includes native1350,
2000 games/100000 sampler draws,69 source metrics,5 fixture transitions/54
metrics,27 shared-upstream ER/54 external boxes;16 four-mode records/four raw
UTF-8 pages byte-exact,288 legacy records/864 binary+normal/pretty Debug payloads
exact to fresh baseline and preserved evidence. No official full-box certificate.

Fresh terminal diagnostic:384 rows/960 candidates/1920 permutations,combined
720 continue/120 walkoff/18 IllegalCredit/102 UnsupportedRule. Independent
ordinary-walkoff oracle1944 attempts=1152 accepted+264 IllegalCredit+
528 UnsupportedRule. Structural synthetic checks,not probabilities or policy.
Fresh native mixed-prefix control repeats all384 prefixes with the same40015
frames/12512 runner-only counts and exact caller words/conservation as before
freeze. Source inputs/exports/reports,original protocol and inherited historical
same-digest mappings remain authenticated,not silently rebaselined.

`proc_9620` exits0/1s:33 completed full artifacts pinned;fresh full stdout103318
bytes SHA256 `d50390364dde4e2a50a104078d33a9f79a949d7f2f2b26b7064736ea9a6768b4`;
stderr93457 bytes `f89ac79a37e5ffb27cc38e3087082a33dffb92dd13b41d8316543aa7991a942f`.
New module SHA256 `1c1288668e86decc380dcdd430a871421011fa34d12972c39c5a35dc468b8e15`;
tests `4862e927caf9c161086928cc14215a1e91963a18f575b355e3057a0b127b8019`.

Inspection scope is explicit:complete originals retained and machine-checked;
all ten <=2200-character v2 bounded-view fragments actually tool-read (raw
summaries/markers/contexts,complete prefix/walkoff output and diagnostic summary).
The detailed diagnostic row table and every full-log row were not manually read;
the separate47KB table remains pinned and independently machine-reconciled.
The larger initial inspection view is retained as created-only,not claimed read.
`raw-inspection-manifest-v2.json`/`raw-inspection-read.json` record this boundary;
no hashes/fragment-generation receipt substitutes for actual reading. Final
complete staged-diff inspection,scope/index/tree/links and acceptance/checkpoint
are still pending at this checkpoint.

## Bounded parent-author acceptance — 2026-10-14

`proc_feed` exits0/8s:exact four-path scope,237 index/work blobs,233 protected,
all148 frozen sources/all146 prior rows,33 full/32 prefreeze pins,five projection/
seven full recipes,11 inputs/four exports/report,original prefix,551 prose links,
tree and whitespace pass. Complete67884-byte staged diff actually read across
31 <=2200-character fragments;creation/reassembly alone was not counted.
The small following documentary delta is separately inspected and must reconstruct
exactly before final acceptance/commit;no source/test/query changed after freeze.

**GO for this bounded standalone supplied-decision projector only.** It handles
explicit safe/caught next-clear Second/Third choices with canonical rules,
credits,continuity/accounting and no RNG/PA/BF/cursor side effects. Unsupported
cases stay refused;no source NULL normalization,attempt clock/rates,result fitting,
fielding/ER/full-box certificate,production/CLI/UI/manager integration,independent
review or E-03/G1/G2/G3/release signoff. All Bottom>=9 and legacy byte/record/page
contracts remain. Local commit and exact postreceipt are the next checkpoint;
no push is authorized. Subsequent fidelity work must receive its own preregistration,
not silently extend this helper into a policy or historical adapter.
