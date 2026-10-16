# Full-transition preview transport — documentary prerequisite

Task **P-01-preview-transition-transport-contract**,2026-10-16 UTC.
Published prerequisite `7aac1249400016990202895771fca11ff14909fa`,tree
`f32034b354e54dbdd45a61c460f1ed6e028b052d`;293 blobs/183 scientific sources.
Exactly two owned paths for this documentary task:

1. `docs/PROGRESS.md`
2. `docs/modeling/preview-transition-transport-contract.md`

Protect292 existing paths/all183 scientific sources;expected294 final blobs.
No runtime/test/codec/accounting/projector/selector/provider/producer/CLI/UI
change. No dependencies,delegation,source queries/transfers,rate fitting,actual-
source draws,automatic actor assignments,manager work or gate signoff. This is
not implementation permission to silently modify old caller behavior. A later
implementation needs its own bounded scope,test-first evidence and fresh suite.

## Observed transport gap

`preview::game_from` currently receives `(Outcome,Event)`,calls ordinary `apply`,
and stores its returned immutable `EventTransition`. `game_with` supplies its
fictional initial state;real-lineup and participant-rating producers also call
`game_from`. These callers must retain their signatures,seed behavior,errors,
transition bytes and default output. No new command-line option follows here.

`EventTransition` has private fields and validating constructors/decoder. Its
immutable getters expose whole before/event/after and supplementary transition
facts. Current codec versions include BBE27 walkoff,BBE28 caught-SF,BBE29 bunt
and BBE30 force-relay facts. Extracting only `transition.event().clone()` and
calling ordinary `apply` is not a full-transition transport:the supplementary
facts are outside `Event`. The supplied-contact projector/adapter already
returns the complete canonical transition. It must not be reconstructed from
credits,descriptors or inferred chronology,or counted as a second PA.

## Proposed narrow native API

A later separately preregistered implementation may add:

```rust
pub fn game_from_transitions<F>(
    seed: u64,
    state: GameState,
    produce: F,
) -> Result<Vec<EventTransition>, String>
where
    F: FnMut(&GameState, &mut Rng) -> Result<EventTransition, String>;
```

This typed transport has no `Outcome` label because runner-only/substitution
transitions are not plate appearances. Their legal deltas and credits belong to
the canonical transition;transport must not fabricate an outcome,PA,batting-order
advance or additional accounting. It neither samples a class nor supplies facts.

The shared loop seeds the existing `Rng` once and invokes the producer only while
`state.over` is false and fewer than10000 transitions have been accepted. For
each successful producer return,require complete `transition.before()==state`,
including personnel,cursor,runner identities/responsibility,score and boundary
state. Refuse mismatch before advancing local state or appending anything. Then
clone only `transition.after()` into current state and append the original owned
transition,retaining every supplementary fact and byte representation. Do not
extract/reapply its event,decode/reencode it as a substitute for retaining it,
add a second reducer/projector,or infer a different outcome. Validated typed
construction establishes event legality;the loop adds whole-state continuity.

A valid already-ended initial state yields an empty log and zero callbacks.
The caller remains responsible for a valid supplied initial state. This task
does not promise a new pre-callback initial-state validator or change the old
ended-initial-state behavior. No transition can be accepted from a different
before-state;invalid typed transitions remain unconstructable through the
existing safe constructors/decoder. Preserve their existing tests unchanged.

Preserve the exact resource-ceiling message:
`seed {seed}: resource ceiling reached, no winner fabricated`.
The ceiling counts every accepted transition,not only PAs,and runs before the
next callback. A successful ending transition suppresses all later callbacks.
Do not fabricate a winner,truncate to a successful partial game,or retry.

New whole-before mismatch error:
`seed {seed}, play {one_based_index}: supplied transition before-state mismatch`.
Producer `Err(String)` propagates verbatim,including empty/reason-like strings;
no partial successful log is returned. Panics are not caught. External producer
side effects and RNG words already consumed before an error are not rolled
back;no purity/authentication claim follows from `FnMut` or immutable inputs.
The transport consumes no RNG words itself beyond constructing the existing
seeded generator. Allocation of parent/contact/provider words is a separate
producer-policy contract;no automatic provider is enabled here.

Keep one shared loop rather than two independently maintained termination/
continuity implementations. Existing `game_from` may wrap its ordinary producer,
call `apply` exactly once for each returned `Event`,and delegate to the typed
loop. Preserve its current canonical error exactly:
`seed {seed}, play {one_based_index}, {outcome:?}: {engine_error:?}`.
A local attempted-play counter in that wrapper is acceptable if test evidence
proves indexing,callback/error order,seed behavior and all old output bytes.
An ordinary caller must not invoke the supplementary projector or selector.
The new typed-loop entry point itself must not invoke any reducer or sampler.

## Acceptance fixed before later implementation

A separate implementation should own only the existing preview support module,
a new transport test file,its protocol/findings and the ledger. Preserve the
complete old generated-preview test file and all other tests/callers. Append or
add test-first controls;distinguish missing-symbol interface errors from a
compiling placeholder's semantic failures and disclose already-green cases.

Required observable checks:

- Exact ordinary-producer equivalence across registered seeds,including the full
  ordered transition bytes and externally observed callback RNG words. Existing
  default/joint/real/rated output and raw UTF8 pages remain byte-identical.
- Retain supplied canonical SF/bunt/force-relay and explicit walkoff transitions
  byte-for-byte;confirm supplementary getters,codec identity and available
  independent accounting. One supplementary PA remains one PA. No claim that
  unsupported Bottom>=9 contact projectors become supported by transport.
- Mixed PA/runner-only/substitution logs retain complete transitions,continuity,
  event/PA counts,lineup cursor and pitcher responsibility. Do not label a
  non-PA event as a sampled PA or expand manager policy.
- Whole-before mismatches for score,cursor,personnel,runner responsibility and
  inning/outs refuse at the offending one-based index,without another producer
  call. A canonical transition built from a different state is still refused.
- Producer errors at the first and later callbacks remain exact;move-owned
  captured state works;no retry,successful partial log or post-ending callback.
- Valid ended initial state means zero calls;canonical endings stop immediately.
  A scripted nonterminating legal sequence reaches the unchanged10000-transition
  guard and does not call the producer a10001st time. Count non-PAs toward cap.
- Existing old-path invalid-event errors and their seed/play/outcome context
  remain exact. No old test,assertion,tolerance or source authority is relaxed.
- Freeze final sources/helpers/inputs/recipes,then the entire applicable fresh
  offline/locked Rust build/tests/fmt/strict all-target Clippy,Werror Python
  discovery/compilation and inherited scientific/native/property/refusal/
  publication/legacy/BBE/prefix/four-mode/raw-page checks. Retain all372 inherited
  recipe bindings/all331 driver lines by exact-inverse administrative aliases.

For this documentation-only checkpoint,authenticate the published293-blob base,
all183 unchanged scientific sources,372 recipes/331 driver lines,197 completed
artifacts,348 checkpoint bindings and14 publication bindings. Inspect actual
callers/constructors/codec;verify local prose links,tracking and real helper
imports,exact two-path diff/index/tree identity and ordinary publication. These
are documentary controls,not a fresh scientific run or new semantic red cycle.

The previously observed286 mask differences/1517 boundary-not-compared/1330
nongeometry memberships/2256 descriptor matches remain unchanged. Retrospective
scores are not predictive validation. This transport contract does not supply
missing historical causes,fit actor policy,join a live generator,render new UI,
complete a manager/season,clear public-data rights or sign G0–G5. The current
limited internal G0 permission and fidelity/product/release holds remain.

## Documentary evidence — 2026-10-16

The original146-line contract above is byte-exact. Prerequisite proc_d848 exits0/
9s with empty stderr:published293 blobs/292 protected/all183 scientific sources/
372 inherited recipes/331 driver lines/197 completed/348 checkpoint/14 publication
bindings authenticated. Source/caller/constructor/codec inspection and closed13
prerequisites are private under `/tmp/bbsim-preview-transition-transport-contract/`.
No runtime source or test changes have been made.

Documentary proc_32a5 exits0/9s:292 protected/all183 unchanged scientific sources/
372 recipes/331 lines/645 local prose links;actual-import runtime2/prose-link2/
tracking1 controls and Werror helper compilation/diff check pass. These are
already-green documentary controls,not new semantic reds or a fresh scientific
suite. The published callback's733 Rust/601 Python result is predecessor evidence,
not certification of future transport code. The exact two-document candidate,
bounded acceptance,once-only commit,existing-identity postcheck and normal push
still require verification on fixed bytes. This contract does not enable the
new API,automatic provider/actor policy,producer integration,manager or gates.
