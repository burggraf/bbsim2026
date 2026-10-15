# E-03 — explicit ordinary walk-off safe-hit facts

## Preregistered contract — 2026-10-14

D-20/D-21 continuation follows the accepted [source audit](../data-audits/2019-terminal-hit-source.md),
code804a72d/completioned70c6f. This is a small deterministic engine prerequisite,
not a terminal generator, stop/reclassification policy or a selected probability model.
Source endpoints cannot provide the facts below. Synthetic callers assert them;
no source authentication, chronology reconstruction or calibrated accuracy is implied.

### Owned scope and compatibility

Nine paths only: this report, PROGRESS, `engine/src/{lib,reducer,rules,sole_safe_hit,codec,walkoff_hit}.rs`
and `engine/tests/ordinary_walkoff_hit.rs`. Baseed70c6f;219 other existing blobs
protected. No dependencies, transfers, worktrees, delegation or push. Existing
Event literals and sole-hit causes remain unchanged. Preserve both Bottom>=9
kernel/dispatcher refusals, existing producers, RNG streams and legacy bytes.

Add optional facts to the immutable **EventTransition**, with a read-only accessor,
not a second transition model. Public `apply_with_walkoff_hit_facts` and
`EventTransition::try_new_with_walkoff_hit_facts` share the existing reducer and
canonical constructor via private optional-context implementations. Existing APIs
supply None and retain their precise current behavior. Existing Event payloads,
all callers, historical test prefixes and absent-facts Debug output stay exact.

`WalkoffHitFacts` contains named `winning_runner_id`, existing `NamedBaseTouch`
for this batter's final legitimate natural-course base, and an explicit
`NaturalCourseTouchTiming::{Unknown,ByWinningRun,AfterWinningRun}`. By includes
a touch at the scoring instant; After asserts the same-play natural-course
exception in 2019 Rule9.06(f), not arbitrary later running. No timing is inferred
from endpoint/credit/vector/JSON order. Unknown refuses.

Counted runs/official hit credit remain explicit Delta credits; physical final
positions remain explicit moves. Latent sampled hit value/reclassification are
**not** canonical facts selected here. This bounded family requires natural-course
batter base equal to his physical final safe base (First/Second/Third). Separate
later physical continuation, extra uncounted Home crossings, outs/appeals/awards/
misplays and automatic clipping remain unsupported. It does not certify a clock.

### Legality and error contract

Only Bottom inning>=9 with nonleading Home before, a positive exact winning-run
Delta, PlateAppearance, existing bound SoleSafeHitFacts, explicit Single/Double,
zero event outs, complete safe vectors and NoMisplay review. Existing validators
continue to own identities, origin/charge/full-vector conservation, PA/RBI/Run
units, lineup eligibility, all other bound families, final state and accounting.
The new context must not bypass these or infer a cause.

The named winner must be a before-base runner explicitly moving Home with a Run
credit, not the batter/held/absent/opponent actor: InvalidParticipants otherwise.
Known credit above physical batter destination or the named winner's advance
(First3/Second2/Third1) is IllegalCredit. Invalid batter-touch identity is
InvalidParticipants; incompatible touch endpoint, unknown timing, missing causal/
review/classification context and out-of-scope shapes are UnsupportedRule.
A named winner with another **counted** scorer originating behind him contradicts
this bounded continuous safe/no-passing family and refuses UnsupportedRule; do
not infer winner identity even where only one could qualify. Endpoints alone do
not certify that family. Neither moves nor credits Vec order is a clock.

Missing facts preserve all accepted/refused legacy cases. Valid explicit winner
facts may resolve the existing mixed-ceiling UnsupportedRule but cannot legalize
credit above the true ceiling, excess counted runs or an invalid canonical delta.
No mutation or partial transition on refusal. Post-win natural-course batter
Second is allowed when winner advance permits it; no winning-instant clipping.

### Encoding

Legacy BBE25/BBE26 records must encode/decode byte-identically with no facts.
BBE27 encodes the existing before/Event/after payload followed by required
WalkoffHitFacts; an air-catch combination is outside this family and must refuse.
Decode BBE27 through the same canonical facts-aware constructor. Reject invalid
magic, truncated/trailing payloads, invalid timing tags and semantically tampered
facts. No silent dropping or None default for a truncated new record.

### Observable acceptance checks fixed before tests/code

1. Test-first missing interface recorded separately, then meaningful semantic
   failures from a minimal conservative scaffold, then green. Already-green
   legacy/unknown controls get no regression credit.
2. Named mixed-ceiling two-run Double accepted only with a legitimate winner;
   wrong/held/absent/Plate identity, excess runs, unavailable cause/review,
   wrong inning/half, invalid touch/timing and malformed units refuse without
   changing before. Both constructor and reducer agree; supplied wrong after
   state still refuses. AfterWinningRun Double demonstrates Rule9.06(f).
3. Independent synthetic Cartesian ordered safe vectors, all masks/outs, both
   credited values, innings9/10, each candidate scorer, both known phases and
   reversed move/credit order. Expected predicate uses named origin/touch and
   no-passing/count conservation, not production helper output. Log counts,
   exact error partitions and successful immutable roundtrips.
4. New record roundtrips and targeted byte mutations fail; old record bytes and
   absent-facts Debug remain exact. Accounting/score/lineup deltas agree and
   accepted terminal transitions cannot be followed by another event.
5. Fresh offline locked full Rust tests/build/fmt/all-target Clippy -D warnings,
   Werror Python discovery/compilation and the full inherited native/source/
   property/preview/legacy16-record/four-page suite, not a smoke substitute.
   Preserve raw logs and failures; freeze new executable/query/test bytes first.
6. Exact nine-path index/work/HEAD scope,219 protected blobs, unchanged135 of140
   prior sources, original report prefix, accepted input/evidence pins and prose
   links/whitespace. Archive the five intentionally changed prior source bytes
   with explicit same-digest historical mapping; do not silently rebaseline.
   Parent-author inspection and acceptance only; no independent-review or gate
   signoff. Record completion and a concrete next action in the sole ledger.

No owner decision currently required. Manager remains NO-GO; E-03/G1 and all
production model/season/desktop gates remain incomplete.

## Test-first observations and clarification — 2026-10-14

`proc_25d0` exits101: five missing-interface compiler diagnostics, not semantic
regressions. Minimal legacy-delegating scaffold yields `proc_bdce` exits101:
9 semantic failures/2 already-green controls. First implementation `proc_8e26`
exits101:8/12 pass. The supplemental max-score test reproduces new u16 winning-
count overflow; widen new arithmetic to u32, not a saturating/clipping policy.
Three other failures are fixture/contract/caller issues, retained externally:
queued next batterh1 was also a before runner; move synthetic batting slots1/8,
not identities or validators. Reusing the old event after termination gives
legacy InvalidParticipants before the canonical over-state check; retain that
exact reducer refusal and separately assert canonical InvalidState. Finally,
**missing hit classification with bound SoleSafeHitFacts is IllegalCredit** under
the existing validator, not the protocol's proposed UnsupportedRule. Preserve
that stronger known contradiction; this dated clarification supersedes only
that error-label sentence. No relaxed validator or removed assertion.

`proc_59c9`:12 focused pass, then strict Clippy rejects cloned single-ref slice;
use `std::slice::from_ref` without suppressions. Added already-green empty-base/
Triple/retirement controls and strengthened valid Top-half refusal. `proc_8698`
exits0/12s:13 focused pass, fmt and strict all-target Clippy pass. Cartesian1,944
attempts partition1,152 accepted/264 known ceiling contradictions/528 impossible
winner orders; both known phases and Vec permutations included. Full suite and
acceptance remain pending on these changed executable bytes.

## Inspection correction before acceptance — 2026-10-14

Stopped full `proc_c0ab` intentionally: the new validator checked winner order
among scorers but omitted passing a preceding **held** runner.142 frozen v1
source snapshots/exact mapping, all recipes and partial raw logs remain external;
no completed full-suite credit. Supplemental `proc_2a99` reproduces unexpected
Ok on First-to-Home past Second held at Third. Add pairwise origin/endpoint order
validation only to the new facts family; include batter-versus-held-runner case.
No legacy policy or guard change. Fresh v2 freeze/full verification required;
prior focused results do not certify these changed bytes.

`proc_b1a5` exits0/17s:14 focused tests and strict all-target Clippy green; exact
Cartesian counts unchanged. New v2 staged whitespace/source freeze passes142
sources/219 protected/228 index-work blobs/542 links;42 complete staged-diff
fragments consumed.30 completed prefreeze pins and156 v1 pins preserve failures,
142 old frozen-source snapshots and interrupted raw logs; none receive full-pass
credit. Seven v2 recipes frozen; full `proc_2353` running, no acceptance yet.

Additional explicit legacy check compiles frozen baseed70c6f and current Rust
sources with `rustc --edition=2021 -D warnings`, same synthetic air/legacy driver:
288 BBE25/BBE26 records plus normal/pretty Debug,864 framed payloads/2,541,216
bytes equal, SHA `67a2ab57bbb66f404925164b6f534cebc260aec1b97f95b67d49c1b50a27520d`.
`proc_86b0` exits0/6s after correcting helper rlib filename prefix; initial37
cascading compiler diagnostics retained, not engine regressions. Header counts
144BBE25/144BBE26 independently parsed. External `v2/legacy-parity/post.json` and
completed pin manifest record actual results. Extra compatibility characterization,
not independent review or a replacement for the full suite.


## Fresh v2 verification and bounded acceptance — 2026-10-14

`proc_2353` exits0/618s on the unchanged142-source v2 freeze:

```sh
bash /tmp/bbsim-ordinary-walkoff-hit/v2/final/full-checks.sh
python3 -Werror /tmp/bbsim-ordinary-walkoff-hit/v2/inspect-completed.py
```

651 Rust tests/38 summaries,391 Werror Python tests42.871s; offline locked build,
fmt, strict all-target Clippy, Python compilation and the complete inherited
source/native/property/preview suite pass.16 four-mode seeded records and four
raw UTF-8 pages remain exact.14 focused tests pass again; independent Cartesian
oracle matches1,944 attempts=1,152 accepted/264 IllegalCredit/528 UnsupportedRule.
Inherited terminal diagnostic remains960 candidates/384 rows/1,920 permutations;
these are structural categories, not source frequencies or terminal policy.

Actually consumed109 lossless raw fragments:49 full stdout,48 full stderr,
9 diagnostic stdout,1 diagnostic stderr,1 focused stdout,1 focused stderr.
Exact reassembly and21 completed-full pins verified; actual consumption receipt
`/tmp/bbsim-ordinary-walkoff-hit/v2/full-consumed.json`. Full stdout87,593 bytes/
1,624 lines SHA `726bc5ea89454a2497eabd8cc26690187bc28d58eab53d3446ac00d02e2e1bb6`;
stderr85,231 bytes/591 lines SHA
`893d7cb01c5d1e15694af52ffed4278473a8a110b62bcf96559486391b630f2c`.
Generation-failure and HTTP-error lines belong to passing negative controls.
Frozen inputs,219 protected blobs,135 unchanged prior sources/five exact historical
snapshots, seven recipes,30 prefreeze/156 v1 pins and legacy parity pins verified.
Original manifest rows and preregistered prefix remain exact, not rebaselined.

**Parent-author GO for this deterministic supplied-facts prerequisite only.**
Reviewed the new canonical seam, validation/codec/legacy behavior and evidence;
42 complete preacceptance staged fragments were consumed, with final documentary
delta inspection/checkpoint still to follow. This is not independent review or a
claim every inherited source file/retained prefreeze failure log was read in full.
Supplied cause/chronology remain assumptions. No terminal generator, clipping,
reclassification, selected model, guard removal, manager or gate approval.
E-03/G1/full boxes and production/season/desktop/release gates remain incomplete.
Actual local commit/tree/clean-state receipt must be recorded after checkpoint;
no push authorized. External evidence may expire; synthetic tests are committed.


### Actual local checkpoint

`proc_ee3b` exits0/24s: code `efc81ae4fccf836c043e35e8e1b9ef2df088883c`,
tree `0561deea824a639c9409eb16e1763f4bbfc53612`, complete accepted/committed diff
SHA `5db3b3d19c7e38ec8b257b23e4b28cbf6f994a8d8649217a8b828c152529ab1b`.
Exact nine-path scope, all228 HEAD/index/work blobs,219 protected, clean main,
origin804a72d unchanged; no push. Author acceptance binds earlier whole stage
inspection plus wholly read documentary delta; executable bytes remain frozen.
Actual external authority `v2/code-post.json`, not a prospective receipt. This
subsequent two-doc handoff records the observed result; no test/code change.
Next inspect/preregister a common runner-only corrective prerequisite, keeping
terminal dispatch refused and manager NO-GO rather than inventing a clock/model.
