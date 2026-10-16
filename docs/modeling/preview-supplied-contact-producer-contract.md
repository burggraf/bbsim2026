# Standalone supplied-parent contact producer — contract

Task **P-01-preview-supplied-contact-producer-contract**,2026-10-16 UTC.
Documentary prerequisite only;no producer implementation or integration yet.
Own exactly `docs/PROGRESS.md` and this new report. Base
`47a5dfcc7419630f0f700a857a3702dc002b4a43`,tree
`10decd8cd494d536c39e0b678dc62b044737d563`:297 blobs/184 scientific sources;
protect296 paths/all184 scientific sources;expected298 blobs/184 sources.
Preserve the complete [transport implementation](preview-transition-transport.md),
[transport contract](preview-transition-transport-contract.md),
[provider boundary](contact-proxy-provider-boundary.md) and
[durable handoff](preview-transition-transport-durable-handoff.md),all existing
source/tests/callers/426 verification recipes and original scientific authorities.
This is not E-03/G1/P-01/P-02 completion,accuracy evidence or a release grant.

## Observed seam and deliberately small choice

The ordinary joint game samples a parent then uses the existing ordinary
`game_with` facade. Real-lineup and rated games still use ordinary `game_from`.
The provider adapter is standalone;its existing callbacks appear only in tests.
The typed transport loop accepts immutable complete transitions and consumes no
RNG words. These published seams can compose without changing an existing caller.

Do **not** add another automatic parent sampler or full-game loop. A new
standalone one-step helper will accept an already chosen `joint::Sample`,construct
its existing ordinary canonical parent,and refine only
`ContactOut + Some(Residual)`. The caller owns parent sampling and any parent
sampling words. This avoids quietly changing a joint/rated/default model and
permits all new mechanism checks to use scripted parents and synthetic proxy
counts rather than actual-source proxy draws.

## Prospective API fixed before implementation

Future implementation scope is four paths only:`docs/PROGRESS.md`,
`docs/modeling/preview-supplied-contact-producer.md`,
`engine/examples/support/contact_proxy_producer.rs`,and
`engine/tests/contact_proxy_producer.rs`. Do not edit this contract or existing
modules/tests/callers to make that implementation pass. Reuse existing module
registration/types and native APIs;no dependencies or second probability engine.
Prospective policy version is `supplied-contact-proxy-producer-v1`.

```rust
pub enum Produced {
    Ordinary(EventTransition),
    ContactProxy(adapter::Adapted),
}
impl Produced {
    pub fn transition(&self) -> &EventTransition;
    pub fn into_transition(self) -> EventTransition;
}
pub fn produce<F>(
    before: &GameState,
    parent_sample: joint::Sample,
    rng: &mut preview::Rng,
    bytes: &[u8],
    expected_source_sha256: &str,
    provide: F,
) -> Result<Produced, String>
where
    F: FnOnce(
        &GameState,
        &adapter::sampler::Selection,
    ) -> Result<Option<adapter::consequence::ContactDecision>, String>;
```

Names `adapter`,`joint`,`preview` denote the existing sibling modules;reuse the
adapter's nested consequence/sampler registration rather than compiling duplicate
native modules. The enum reuses `Adapted` intact instead of copying a second
selection/disposition schema. Both accessors expose/return the original native
transition without extraction/reapplication,codec roundtrip,reprojection or
additional RNG. Metadata is available before explicitly consuming the enum.
A fallback/Remaining result is still `ContactProxy`,not disguised as Ordinary.

## Ordered work and exact word/callback semantics

1. Use `joint::event_for(before,parent_sample)` and the native `apply` to build
   the ordinary parent. Do not accept an arbitrary supplied delta or reconstruct
   actors for special contact plays. Wrap event errors with
   `ordinary contact-proxy parent refused: {error}` and reducer errors with
   `ordinary contact-proxy parent refused: {error:?}`. Invalid outcome/contact
   pairs refuse here,before consuming a class word or invoking the callback.
2. Every valid non-residual parent returns `Ordinary(parent)` unchanged. Consume
   **zero** RNG words in this helper,invoke the callback **zero** times,and do
   not inspect proxy bytes/digest. This is an explicitly lazy boundary,not a
   claim that unused malformed artifacts were validated. Twelve valid parent
   categories are ordinary:eight non-contact outcomes and four non-residual
   contact kinds. Existing synthetic air-fielder ordinals/policies are reused,
   not newly inferred historical actors.
3. For a valid residual contact-out parent,consume exactly **one** `next_u64()`
   class word,then call existing `adapt_with_provider` exactly once with that
   parent/word/bytes/digest/callback. The adapter's existing independent canonical
   parent replay remains intact;it is validation,not a second sampled outcome
   or second retained event. Do not resample the parent/class or retry.
4. Full strict artifact loading/calculation/context selection remains in the
   adapter. A loader/selection refusal occurs after the one class word and
   invokes the provider zero times. An accepted selection invokes the FnOnce
   provider once,including Remaining and terminal contexts. Provider errors,
   including empty strings,and native adapter/projector errors propagate
   **verbatim**. Do not turn refusal into unavailable facts or fallback success.
5. Successful adapter output becomes `ContactProxy(adapted)` intact. Preserve
   selection class/context/grid/source tag/model/version,realized class,
   disposition and the complete canonical transition. None/Remaining/terminal
   fallback semantics are exactly the existing adapter's,not new policy.

No initial-state validator,panic catcher or rollback is added. The caller
supplies a valid state and uses the existing loop's ending/resource guards.
Consumed RNG and callback/capture external effects are not rolled back on error;
FnOnce/immutable inputs do not prove purity or fact truth. The helper does not
sample a parent,invoke a manager,insert non-PA events,alter lineup/accounting or
add a full-game resource policy. Caller-driven runner-only/substitution events
can still share the existing typed transport loop without becoming PAs.

## Observable implementation acceptance checks

Preregister implementation tests before writing its production body:

- All13 valid parent categories over the24 valid base/out contexts. Non-residual
  results match independent `event_for + apply` bytes and keep RNG/callbacks
  untouched. Invalid outcome/contact combinations refuse before class draws or
  callbacks. Preserve canonical legality rather than weakening fixture checks.
- Synthetic-only proxy artifacts;compare residual results/selection metadata/
  complete transition bytes with direct native adapter calls given the exact
  reference RNG word. Confirm exactly one class word,zero on ordinary/refusal
  before parent construction,and no extra word in accessors or transport.
- Explicit SF/bunt/GIDP facts preserve all native supplemental facts and codec/
  accounting consequences. Unknown,None,Remaining,terminal fallback,wrong
  family/ineligible/contradictory facts and arbitrary provider errors retain
  exact adapter distinctions. Selected class is not automatically realized.
- Whole malformed artifacts/mismatched tags/context boundaries retain strict
  loader refusal;ordinary calls may ignore unused malformed artifacts exactly
  as declared. Inputs stay byte-identical and move-owned FnOnce captures work.
- Compose scripted produced transitions through `game_from_transitions`;check
  full continuity,metadata-before-extraction,non-PA/PA independence,canonical
  ending,no post-ending callback and no transport-added reducer/projector pass.
  Keep existing predecessor/RNG/fact/cap tests byte-exact.

Record interface versus semantic red failures,invalid fixtures and already-green
controls separately. Preserve all raw failures and independent direct canonical
references. Run all applicable native/offline/locked/fmt/strict Clippy/Python/
scientific/refusal/parity checks with a new current source freeze. Do not reuse
744/601 historical passes as certification of future changed code. Historical
manifest bindings remain explicit authenticated snapshots,not spoofed current
hashes. Original predicates/tolerances/426 inherited recipes remain immutable.

## Scope and claim limits

No actual-source proxy class draws are executed in this task or its prospective
synthetic verification;the new helper remains unwired. No source query/transfer/
census,fitting,new actor/fact generation,handedness/location/range/arm inference
or new fielding roll.
An already decided hit cannot enter this residual-out refinement branch. No
mutation of selector/provider/projector/codec/accounting/engine/RNG algorithm or
existing producer/default/CLI/UI. No manager/season support,rights expansion,
bundled extract/app release or G0–G5 signoff. The externally supplied digest tag
and synthetic `'a'*64` fixtures are not source authentication or observed facts.
Unavailability is not a negative judgment,held runner,known actor or `Out`.

Before implementing this separate four-path helper,complete this documentary
checkpoint:authenticate published prerequisites/private archives/all184 unchanged
sources/426 recipes,run Werror actual-import/prose/tracking checks,inspect the
bounded two-path candidate,accept fixed bytes,commit once,postcheck existing
identity,and perform only an ordinary authorized push with actual remote closure.

## Prerequisite authentication observed — 2026-10-16

proc_f9a0 exits0/12s,empty stderr:published297 blobs/296 protected paths/all184
unchanged scientific sources/426 recipes/211+457+19 transport and53+18 handoff
bindings/both durable archives verified. Complete original162-line protocol SHA
`60b638ec10c1725cd22904a073bd88eb2bc140d0500b1d86d99ab5b68f98d5ce`.
The new document is not a producer implementation and does not certify future
changed code. Evidence root:`/tmp/bbsim-preview-supplied-contact-producer-contract/`;
exact command:`python3 -Werror /tmp/bbsim-preview-supplied-contact-producer-contract/preregister-v1.py`.
At preregistration,actual-import/prose/tracking/fixed candidate/publication were pending.

## Documentary verification observed — 2026-10-16

proc_41d6 exits0/9s:Werror helper compilation,actual-import2 methods0.106s,
prose-link2 methods0.008s,tracking1 method0.005s,`git diff --check`,296 protected
paths/all184 unchanged sources/426 recipes/published prerequisite bindings and
both private archive payload manifests pass. Actual prose inventory661.
The two-document controller is an exact-inverse administrative adaptation;
scientific sources,predicates,tolerances and426 recipes remain unchanged.
Inherited link controls already pass;these are not new semantic scientific reds.

Evidence:`runtime-v1.*`,`links-v1.*`,`tracking-v1.*`,`verify-v1.*` and
`checkpoint-adaptation-v1.json` in the private contract root. Commands:

```sh
python3 -Werror /tmp/bbsim-preview-supplied-contact-producer-contract/test-link-runtime-v1.py /tmp/bbsim-preview-supplied-contact-producer-contract/checkpoint-v1.py
python3 -Werror /tmp/bbsim-contact-endpoint-semantics/test-prose-links-v1.py /tmp/bbsim-preview-supplied-contact-producer-contract/checkpoint-v1.py
python3 -Werror /tmp/bbsim-contact-endpoint-semantics/test-prose-tracking-v1.py /tmp/bbsim-preview-supplied-contact-producer-contract/checkpoint-v1.py
python3 -Werror /tmp/bbsim-preview-supplied-contact-producer-contract/checkpoint-v1.py verify
```

Fixed candidate inspection/acceptance/once-only commit/existing-identity
postcheck/ordinary authorized push remain pending. No implementation,
independent review,comprehensive secrets/license certification or gate signoff
is claimed;no new scientific suite is claimed for documentary-only changes.
