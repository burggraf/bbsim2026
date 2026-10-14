# P-01 — fixed-opportunity joint safe-hit comparison

## Preregistration — 2026-10-13, before new tests, fitting or scores

D-20 prioritizes engine/mechanism solidity; D-21 requests bounded continuation.
The descriptive [hit qualification audit](../data-audits/2019-hit-advancement.md)
is verified locally at `95150b638304318de976c610fbe2eb1f59bf4312`. Its qualified
endpoints are not causes, NoMisplay certificates or a validated advancement policy.
This next experiment compares three deliberately simple **joint endpoint**
distributions on fixed observed opportunities. It does not alter the simulator.
[PROGRESS](../PROGRESS.md) is the only live status ledger.

The reserved game's descriptive counts/vectors have already been examined during
the preceding audit. These scores are therefore **retrospective reserved-split
diagnostics**, not fresh blinded validation or evidence that season-end ratings
forecast games. No new independent empirical validation claim or gate signoff.

Base: `95150b638304318de976c610fbe2eb1f59bf4312`, clean main, parent sole writer.
Exactly four owned paths:

- `docs/PROGRESS.md`
- `docs/modeling/hit-advancement-comparison.md`
- `audits/hit_advancement_comparison.py`
- `tests/test_hit_advancement_comparison.py`

Preserve207 other tracked paths and all129 existing executable/test sources.
Freeze base/scope/original protocol/protected bytes/accepted inputs before tests.
No new dependencies, transfers, delegation/worktrees, canonical Rust changes,
preview/UI/manager integration, seed/parameter/filter search, push or release.
No generated EventTransitions or Python-owned simulator outcomes. Model rankings
cannot authorize integration. Production accuracy/fidelity gates remain NO-GO.

## Read flow and accepted inputs

Inspected the actual accepted Python qualification/census/publisher flow, exclusive
final-reread writer, synthetic Rust ordered geometry, current station producer and
real-lineup caller. The existing producer supplies explicit modeled causal facts;
source endpoints cannot do so. The ordinary-hit experiment must stay separate.

Reuse the preceding nine accepted snapshot/supplement/lineup/manifest/census/native
CSV pins unchanged, plus the accepted hit report:
`6551fe2e0146d844362fb704121d7b79a64d95ac49834990c4a490d35432dcf0`
(2,207,071 bytes). No new source extraction/transfer. Before actual fitting, rerun
unchanged complete hit publication into a new exclusive external qualification
artifact and require byte-for-byte equality with the accepted report. This
revalidates all original source/native/full-census/provenance/cohort facts rather
than trusting a convenient count subset. All ten inputs and complete inherited/new
recipes/queries are pinned before execution and reread before final publication.

Use the already-fixed game-ID SHA256 reservation and four exclusions unchanged.
Only qualified literal Single/Double vectors in supported **training** games enter
model counts (17,467 events). Qualified supported reserved events (4,308) enter
scores only. Excluded qualified38 events enter neither fitting nor scoring; retain
them and every unresolved candidate in coverage summaries. Full source totals stay
34,064 candidates/21,813 qualified/12,251 unresolved. No imputation or relabeling.
Do not treat source-filtered coverage as a random sample of all hits.

## Fixed joint support and distributions

A context is `(hit value, before base mask, outs before)`: values1/2, masks0..7,
outs0..2. Preserve48 contexts even when a training or scoring count is zero.
Enumerate legal **ordered complete safe endpoint vectors**, not independent runner
advances: batter endpoint exactly hit value; every occupied before base appears
once in ascending origin order; each runner holds or advances to safe base/Home;
non-Home endpoints strictly increase; Home endpoints form a suffix and may repeat.
No Out, collision, backward endpoint or overtaking. Canonical source predicates
are unchanged. Empty bases have one vector; no artificial uncertainty is added.

Collapse only qualified vector endpoint shapes within each context; trajectory,
attempt labels and nullable fields are not explanatory variables in this bounded
experiment. Preserve their original audit unchanged and disclose this coarsening.
Reject a qualified vector outside independently enumerated support before fitting.
Endpoint identities do not enter the counts or serialized probabilities.

Fix all three models before new measurements:

1. `uniform-legal-v1`: every legal joint vector has probability `1/K`.
2. `pooled-mask-jeffreys-v1`: pool training counts across all three outs states for
   the same hit value/mask, then `q_j=(2*n_j+1)/(2*N+K)`.
3. `base-out-jeffreys-v1`: use only training counts in the exact value/mask/outs
   context, with the same fixed Jeffreys-half-per-legal-vector rule.

Here K is the full structural support size, not the observed number of categories.
Use exact integer/Fraction arithmetic for probabilities and their sum; zero-count
legal categories stay positive and zero-exposure contexts become uniform. Counts
are nonnegative checked u64 integers, with checked u64 aggregation (bool is not
an integer count); never fill a missing source field with zero. No hierarchical tuning, empirical prior strength,
player/arm/park/hand effects, trajectory conditioning, fitted qualification, rates
for unqualified hits or learned attempt policy. Neither learned model sees reserved
or excluded counts. No RNG or repeated draw approximation is needed for scoring.

## Metrics and numerical checks fixed before scores

For each model score the supported reserved qualified joint vectors exactly by
count. Report every48 context's training/scoring N, legal K and full probability
vector, with exact numerator/denominator and finite binary64 presentation.

- Negative log score: `-log(q_observed)` nats/event; count-weighted arithmetic mean.
- Multiclass Brier score: `sum_j((q_j - 1[j=observed])**2)` per event; count-weighted
  mean, **not** divided by K.
- For both scores report signed learned-minus-uniform and base-out-minus-pooled
  deltas. Zero is the fixed nonworsening boundary; report flags, not automatic
  model selection/integration approval. Report worsening as plainly as improvement.
- Report all qualified supported reserved events and the occupied-before-mask
  subset separately; empty bases have deterministic scores0 and must not dilute
  the running-mechanism diagnostic silently.
- Report per-hit-value and per-outs scores plus all48 cells; retain zero-N cells
  as unscored/None, not measured zero. Unsupported/unqualified/excluded data are
  coverage, never hidden scoring denominators.

Exact rational normalization/support is mandatory; probability presentation must
be finite/positive and sum within32 binary64 epsilons of1. Compute floating sums
in a fixed canonical order with `math.fsum`. Proper scores must be finite and
nonnegative (no clipping/retuning to pass); serialize finite JSON only. No confidence
interval, statistical significance, causal generalization or fresh heldout pass is
claimed from these previously examined aggregated vectors. This scope lacks game-
cluster resampling; do not manufacture independent runner/event uncertainty bands.
Computational task completion requires honest complete scores/conservation, not
an empirical model winning. Empirical failure does not change the protocol.

## Observable acceptance and execution order

1. Test first: missing interface versus genuine semantic reds versus already-green
   refusals explicitly separated. Independently enumerate support; test every48
   context, simultaneous scoring/holds/nonstation geometry, illegal vectors,
   empty/zero/sparse counts, bool/negative/overflow/type refusals, exact rational
   controls and normalization. Preserve test fixtures as synthetic only.
2. Training isolation: changing reserved or excluded endpoints/counts cannot alter
   learned probabilities; only scoring/coverage may change. Permutation/nonmutation
   and exact context/split/vector/count conservation. Scoring formulas independently
   calculated on literal tiny distributions; zero scoring N remains None.
3. Fail-closed publisher: complete accepted qualification regeneration/equality
   before fitting; early collision/repository/digest refusals and final input/query/
   recipe drift checks, including redirected output parents. Outputs external and
   exclusive; preserve qualification evidence if subsequent scoring fails.
4. Before real execution prepare separate standard-library probability/score
   controls without importing production fitting/scoring code. Reconcile all
   training counters,48 supports/three full distributions, context/subset/overall
   denominators and both proper scores on actual data. No source/protocol retuning.
   Two exclusive reports repeat byte-for-byte; all ten inputs/recipes unchanged.
5. Freeze131 old/new executable/test sources. Run full inherited offline locked
   Rust build/test, fmt/strict all-target Clippy, Werror Python discovery and
   compilation, source/native/property/preview controls and four-mode byte
   compatibility. Consume complete outputs and exact four-path full-index diff;
   verify base/main/index/work/tested/protected/input/prefix/links/tree equality.
   Bounded parent/author acceptance and local checkpoint only, not independent
   review or accuracy/generator/manager/G1/G2/G3 signoff.

No consequential maintainer-input blocker is presently identified. A subsequent
experimental Rust generator integration needs a separate protocol, explicit causal
assumptions and terminal projection policy; this comparison does not supply them.

## Implementation evidence — no actual fitting/scoring yet

`proc_eef2` exit0/6s freezes original protocol/base/four owned paths/207 protected
HEAD-work paths/129 prior sources/ten accepted inputs before new tests. Module/test
paths were absent at freeze. `proc_e885` missing-module ImportError is interface
evidence, then shape-only activation yields25 genuine support/type assertions
across two methods (`proc_16e0`),370-line raw failure retained; earlier visible
reads were truncated, so full inspection is not claimed. Independent
pairwise support versus ordered-combination implementation agrees on40 vectors
across16 value/mask states; `proc_82f5`two methods/compile/diff green. Five further
probability/proper-score methods are added first; `proc_1b50` witnesses22 missing
API errors/seven total methods,230-line raw log retained/tail inspected; earlier
visible reads were truncated. Minimal interfaces then `proc_b820` witnesses21
semantic assertions; full239-line raw semantic log retained. Earlier visible
reads were truncated; they do not establish complete inspection.
`proc_628d` seven focused methods/compilation/diff green. Synthetic distribution
and report work continues: `proc_4d2c`11 missing fit/inventory errors; a prefit guard
spy added during that interface cycle triggers a fresh run (`proc_d09f`,11 errors)
before shape activation. These are interface records, not acceptance of frozen
sources. `proc_d96f` nine genuine assertions across five methods, with training
isolation/shuffle controls already green; full raw semantic log retained,
without a complete untruncated-inspection claim.
`proc_5e7a`14 focused methods/compilation/diff green, raw log retained. Five further report-detail/typed-refusal controls added first:
`proc_e823` two missing-field KeyErrors, then `proc_6ac5` two semantic assertions;
coarsening/normalization/typed-prefit controls already green. `proc_1081`19 focused
methods/compilation/diff green, raw log retained. Full synthetic
publisher/CLI controls added first: `proc_c2d4`11 missing APIs plus one cascading
missing-artifact error. Drift subtest grouping corrected to keep independent
cases/checks together, with every assertion retained. Minimal publish/main
interfaces activated for a separate guard-semantic run: `proc_67e9` ten genuine
refusal assertions plus two missing-positive-artifact errors, not compiler reds.
Only then complete pinned/equal-qualification-before-fitting publisher added;
`proc_94d2`30 focused methods/compilation/diff green. Three additional prefit recipe,
qualification-artifact drift and late collision controls are already-green refusal
characterizations (`proc_d494`33 methods/compilation/diff); no implementation
repair is claimed. Raw green logs retained; earlier truncated reads are not
complete-inspection evidence. At this implementation checkpoint, no source fitting/
scoring, model selection or simulator outcome change has occurred.

## Independent computational verification — fixed before actual execution

External `/tmp/bbsim-hit-comparison/independent.py` imports standard libraries only,
not production fitting/scoring functions: Cartesian/pairwise joint support,
Fraction-half posterior, direct count-weighted log loss and exact-Fraction Brier
aggregation. `proc_7735`48 synthetic contexts/literal score/prior/zero-N controls
and Werror compilation pass; complete stdout/stderr inspected. Exact supports,
counts, rational probabilities and binary64 probability presentation must agree.
Independent proper scores may differ solely in binary64 accumulation order; the
fixed computational reconciliation bound is `32*epsilon*max(1,abs(reference))`,
not an empirical/model-acceptance tolerance. Diagnostic nonworsening stays exactly
zero, with no clipping or empirical retuning. Before source fitting, the separate
exclusive actual driver pins base/main/four paths/207 protected/129 prior/131 total
sources/ten inputs/protocol and both helper programs; complete unchanged hit audit
must regenerate/equal the accepted bytes before **each** comparison fit. It then
reconciles all48 contexts/three distributions/seven subsets and both proper scores,
retaining source qualification if any later check fails. Two comparison/qualification
pairs must be byte-identical. Actual execution remains pending this preregistered
verification; no new source transfer/extraction or simulator change.

## Actual observations — retrospective diagnostics, 2026-10-13

`python3 -Werror /tmp/bbsim-hit-comparison/actual.py`, `proc_0405` exit0/72s.
Complete short stdout inspected in bounded reads; stderr empty. Before each fit,
unchanged complete `hit.publish` revalidates source/native/full-census/cohort and
writes a new exclusive qualification artifact **byte-identical** to the accepted
2,207,071-byte report. Independent controls reconcile every48 context/120 joint
support entries/three full distributions/seven subsets/training and reserved counts,
exact rational probabilities/presentation/both proper scores and signed diagnostic
flags. Two comparison reports each176,048 bytes, SHA256
`934ca90488c622eedde7cda30708463ba22b883d5e1a03be8f7ddf43124eddc6`, byte-identical.
Both qualification artifacts byte-identical; all228 source/input/helper/completed-
evidence pins unchanged. Training17,467/reserved4,308/excluded-qualified38 unchanged;
all34,064 candidates/21,813 qualified/12,251 unresolved and outside/source coverage
retained. Every48 context has scoring exposure in this dataset; training N12–3,273,
reserved N3–797. Zero-N controls remain tested even though actual cells are populated.

Lower scores are better on these fixed, previously inspected qualified proxies:

| Subset | N | Model | Log score (nats/event) | Brier (not divided by K) |
|---|---:|---|---:|---:|
| All | 4,308 | Uniform | 0.434118632 | 0.264740019 |
| All | 4,308 | Pooled mask | 0.274167608 | 0.177963012 |
| All | 4,308 | Base-out | 0.267542579 | 0.172315051 |
| Occupied | 1,796 | Uniform | 1.041304604 | 0.635022272 |
| Occupied | 1,796 | Pooled mask | 0.657635887 | 0.426873416 |
| Occupied | 1,796 | Base-out | 0.641744673 | 0.413325856 |

The2,512 empty-base opportunities are deterministic zero-loss observations, not
missing/unscored cells. Showing occupied separately prevents them from silently
diluting the mechanism diagnostic. Base-out-minus-pooled overall deltas are
log `-0.006625028161450386` and Brier `-0.005647961104479099`; occupied deltas
`-0.01589121454316722` and `-0.013547559263973163`. All seven subset averages
nonworsen on both scores for each fixed comparison, but that does **not** mean all
contexts improve. Against uniform, each learned model worsens both scores in six
double contexts `(mask,outs)=(1,0),(1,2),(3,2),(5,0),(7,1),(7,2)`. Base-out versus
pooled worsens log score in20 contexts and Brier in18. For example Double/mask1/
outs0 has training280/reserved53 and base-out-minus-uniform log `+0.10721071223498002`,
Brier `+0.08901361144849873`; Double/mask5/outs0 has training17/reserved7 and
`+0.13224121973733105`/`+0.09942680776014112`. Full48 probabilities/score/delta/N/K
records remain in the external report, including all worsening and sparse cells.
No filter/prior/parameter change, model selection, significance/generalization,
forecast or automatic integration claim follows these averages.

`proc_4e13` exit0 preserves the two premeasurement docs SHA-exact in
`/tmp/bbsim-hit-comparison/actual-docs/` and maps only their historical pins to those
archives (`verified-actual-current.sha256`); source/data/helper pins remain exact.
Thirteen completed actual artifacts pinned **after** writer exit. This is historical
evidence preservation for subsequent Markdown observations, not source retuning or
recomputation. All131 executable/test sources stay frozen from measurement through
full verification/acceptance. Full inherited checks/compatibility and bounded author
acceptance/local checkpoint remain pending; no G1/G2/G3 or manager GO.

Reproduction requires the exact accepted local caches (native siblings beside the
census), unchanged query/recipe versions and an absent external output plus absent
same-stem `.qualification.json`. Do not overwrite accepted artifacts or silently
transfer replacement data if transient `/tmp` evidence expires:

```sh
python3 -Werror -m audits.hit_advancement_comparison \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb \
  /tmp/bbsim-outcome-details/2019-outcomes.duckdb \
  /tmp/bbsim-lineup-readiness/report-first.json \
  /tmp/bbsim-outcome-details/manifest.json \
  /tmp/bbsim-runner-transitions/report-first.json \
  /tmp/bbsim-hit-advancement/report-first.json \
  /tmp/new-absent-hit-comparison.json
```

## Fresh full verification — 2026-10-13

`proc_3a48` full preparation exit0/1s freezes131 sources equal to the measured
source manifest;129 prior/207 protected/ten accepted inputs/228 mapped historical
pins/13 completed actual artifacts exact. Compared inherited and new drivers:
only external artifact routing/current protected pins/new Python compilation and
added four-mode compatibility; no inherited check removed or replaced.

`bash /tmp/bbsim-hit-comparison/final/full-checks.sh`, `proc_a7f2` exit0/280s.
Raw1,810 stdout/544 stderr lines retained SHA-pinned. Earlier large tool views
were truncated and alone did not establish complete inspection; the local handoff
below records the later complete nontruncating reread, not a grep-only review. Offline locked build/test, fmt and
all-target Clippy `-D warnings` pass;35 full unfiltered Rust summaries total609
passing tests. Werror Python discovery356 methods (`29.172s`), compilation,
native1,350 individual cases, source69 metrics/five transition fixtures/54 metric
comparisons/27 shared-upstream ER/54 external box comparisons,2,000 seeded games/
100,000 fixed-diagnostic draws/source repeats and every inherited control pass.
Four historical default traces,16 full default/joint/real/rated records and four
rated full live pages remain byte-identical to accepted references. All131 tested
sources/129 prior/207 protected/ten inputs and actual evidence pins still exact.
Expected generation failures and HTTP400/404/421/500 lines are identified negative
controls followed by successful tests, not suppressed errors or an abandoned server.
No executable/test source changed after measurement or verification. Parent/author
review and local checkpoint remain pending; this diagnostic does not approve a
model, generator, manager, empirical accuracy or G1/G2/G3.

## Bounded author acceptance — 2026-10-13

`proc_cd3c` stage exit0/1s: exact four-path base/main scope, all211 work-index
blobs/207 protected HEAD-index/131 tested/129 prior/ten inputs/228 mapped actual/
13 completed actual pins/original protocol prefix/512 prose Markdown links and
110 completed evidence pins exact. The stage excludes its own active stdout/stderr;
completed evidence only. Full1,131-line/102,961-byte full-index diff inspected in
complete nontruncating ranges; executable/test sources identical to measured and
fully tested bytes. No source/test change or requested assertion weakened.

`proc_3860` author acceptance exit0/1s: accepted exact diff/tree/pins, retained
worsening and sparse contexts, all limits checked against the original contract.
Complete short stdout/empty stderr and `acceptance.json` consumed. Parent/author
acceptance only, **not independent review**, causal/model accuracy, automatic model
selection, generator/manager integration, empirical validation or G1/G2/G3 signoff.
Only Markdown handoff/acceptance observations follow frozen verification. Renew
exact staged diff/tree/completed stage pins before local commit; verify parent/
committed-index-work/tested/protected/input/evidence equality and clean main afterward.
No push, data publication or routine approval pause authorized by this checkpoint.

## Verified local checkpoint and inspection clarification — 2026-10-13

`proc_a434` doc-only accepted snapshot preparation exit0/1s; exact211 work-index
blobs/131 tested/129 prior/207 protected/ten inputs/512 links and12 completed
stage/acceptance pins. Inspected85-line Markdown delta equals retained acceptance
patch; both executable/test diff sections remain byte-identical. Fresh precommit,
local commit and postcheck `proc_0853` exit0/2s establish code
**`f3a11d7391350b714e27c1028473cb2946af6b54`**, parent `95150b6`, exact four paths,
accepted1,160-line/105,272-byte full-index diff/tree and all211 committed-index-work
blobs; tested/protected/input/evidence pins unchanged, clean main ahead8/no push.
All six short stdout/stderr logs consumed; stderrs empty. This handoff changes only
the two Markdown paths; no executable/test/input/query mutation or remeasurement.

Earlier large source/log/diff tool responses were truncated; requesting ranges
was not proof of complete visible inspection. During this handoff, all1,131 staged
diff lines are re-read in15 genuinely nontruncating chunks (maximum8,723 bytes),
and all1,810 stdout/544 stderr lines in17 chunks (maximum10,999 bytes). Complete
Rust609/Python356/native/source/compatibility evidence and expected negative-control
HTTP/generation failures inspected, not suppressed. This later inspection establishes
complete current evidence consumption; it does not retroactively certify the earlier
truncated views or expand parent/author acceptance into independent review.

Bounded comparison is done. Production generator/model selection and managerial
integration remain NO-GO. Next concrete action is separately preregister the bounded
native joint-hit projection/sampling prerequisite, including explicit modeled causal
facts, terminal hit-value/run restrictions and preservation of legacy modes, before
any new sampler test/code/measurement; this comparison authorizes none of those
changes by score ranking. No consequential maintainer-input blocker identified.
