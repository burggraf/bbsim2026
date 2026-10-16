# Coarse contact-proxy fit contract — protocol before fitting

## Preregistration — 2026-10-15

Task: **P-01-preview-contact-proxy-fit-contract**. This is a documentation-only
successor to the verified [observable-target audit](../data-audits/2019-contact-proxy-observable-targets.md),
not permission to fit, select a model, implement a sampler or integrate a producer.
The previous [architecture contract](preview-contact-proxy-contract.md) and
[evidence boundary](../data-audits/2019-contact-fact-evidence-boundary.md) remain binding.

Base: `8288a8a9ebe8dae8898fb7ef0e788e595579b522`, tree
`b4ec1f5cced45c22b1a346aa5e03f2a470ffa06d`, parent `eb3928d`.
The audit's fresh full-v3 run passed 687 Rust and 548 Werror Python tests plus all
inherited checks; its identity-only postcheck verified 267 blobs and clean main.
That run certifies its unchanged scientific sources, not this later document.
No independent review or production gate signoff is claimed.

### Scope and immutable prerequisites

Only `docs/PROGRESS.md` and this new document may change. Preserve all other 266
existing paths, all 167 query/build/engine/audit/test sources and all 111 inherited
recipe bindings. Expected result: 268 tracked blobs, still 167 scientific sources.
Keep raw evidence and private verification receipts outside Git under
`/tmp/bbsim-preview-contact-proxy-fit-contract/`. Do not run historical live-epoch
helpers against this new HEAD or rebaseline their original path/digest bindings.

No new source transfer, query, population measurement, fitting, tuning, model
selection, dependencies, engine/accounting/codec/projector/producer/UI/manager
changes, gate waiver or push. No use of the 2256 descriptor matches as a silently
selected training population. No NULL-to-zero/false/held/Out conversion, endpoint
normalization, child reconstruction, identity repair or label alias.

### Observable acceptance checks

Before specifying the later fitting protocol, authenticate the actual code
checkpoint/postcheck, repeated measurement/full-suite evidence, protected paths,
source/recipe identities and existing controlled-shape reference. Freeze those
bindings and this protocol; record failures rather than overwrite occupied files.

Then document, before any fitting or comparison:

1. Exact source-event population, four response definitions, exclusion rules and
   treatment of conflicts, missing evidence and previously examined splits.
2. Sampling/transition boundaries: preserve residual-parent mass; source geometry
   labels are not imported causal facts. Rust continues to own legal transitions.
3. Any proposed estimator, prior, empty-context handling and artifact grammar,
   including integer/rational units, arithmetic and version/seed compatibility.
4. Numerical acceptance limits and separately labeled diagnostics. Distinguish
   computational correctness, coarse descriptive calibration and historical
   consequence fidelity. Do not use the first to claim the other two.
5. Full reporting of all 286 known mask differences, 1517 boundary-not-compared
   memberships and 1330 nongeometry memberships. Do not erase them with a new
   denominator or call the already examined reservation fresh blinded validation.
6. Explicit refusal/NO-GO rules and one concrete next work package. A proposed fit
   protocol may remain NO-GO if its observable or legal-fallback contract is not
   adequate; this task must not silently open the fit or integration gates.

Verify all documentary links and existing prose-link/tracking regressions, the
unchanged frozen protocol, protected/source/recipe/artifact identities and exact
owned diff. No new executable/test change is authorized, so do not describe the
predecessor full suite as a fresh run for this document. A coherent local checkpoint
requires exact committed/index/work identity and an updated resume entry. Never
blindly recommit, amend or roll back after a postcheck failure.

Status at preregistration: contract specification and verification pending;
no new counts, estimates, model, generator or gate decision.

## Specification recorded after prerequisite authentication

Preregistration `proc_e800` exited 0/2s. Its complete short stdout was inspected;
stderr is empty. It authenticated 267 base blobs/266 protected paths/167 sources/
111 recipes, all 145 full-suite/53 supplementary/55 measurement/14 canonical
artifacts and the original 139/27 failed artifacts. `protocol-original.md` and
`preregistration-v1.json` retain the protocol and exact original bindings. This
was hashing/identity verification, not a new source measurement or fit.

### 1. Population and response: event counts, not mechanism opportunities

Freeze the source population to the unchanged accepted audit's real source events
with `joint_category == 'contact_out'` and `contact_bin == 'residual'`.
Every included event must have `pa == 1`; an unexpected contrary row is a refusal,
not an additional filter. Reconcile the complete distinct parent mass to the
unchanged legacy residual bin before allocating response counts. Do not use all
plate appearances, runner children, labeled memberships, eligible before-frames
or descriptor matches as the denominator.

For each parent event, use the unchanged geometry predicates, not a new causal
classifier. The categorical response is exactly one of:

| Code | Response definition |
| --- | --- |
| `SF` | Exactly one geometry-qualified SF membership |
| `SH` | Exactly one geometry-qualified SH membership |
| `GIDP` | Exactly one geometry-qualified GIDP membership |
| `remaining` | None of those geometry-qualified memberships |

A geometry predicate is the old literal reason list with none of
`not_terminal_pa`, `conflicting_labels`, `unsupported_before`, `event_units`.
Keep every other original reason, including incomplete child vectors and unknown
causal facts. A conflicting-label event remains in the parent denominator and
`remaining`; preserve its original labeled memberships in diagnostics. More than
one geometry-qualified family for one event is a refusal, never priority-based
assignment or double counting. Geometry membership outside the residual parent
is also a refusal. Observed zero overlap is not a universal assumption.

Nongeometry labels, missing children/endpoints, known mask differences and
boundary rows are not dropped. ROE, IBB, interference, other legacy bins and
non-PA/no-play rows stay in their original outside-parent reports; they are not
relabeled as `remaining`. Preserve all original source, membership and split
folds. `remaining` means only “none of the three defined responses,” not a known
clean out, absent mechanism or invented causal certificate.

Use the original four excluded games and original SHA256 game split unchanged.
Export training, previously examined heldout and excluded tables separately.
Only training counts may enter a later estimator artifact. The other tables
remain diagnostics; no pooled fallback may read their counts. There is no fresh
blinded or prospective validation in this work.

### 2. Context domain and fixed estimator proposal

The only fitted-context proposal is `(base_mask, outs_before)`: masks 0–7 and
outs 0–2, exactly 24 rows in mask-major, then outs-major order. Geometry-class
structural support is frozen from the old predicates:

- `SF`: outs 0/1 and mask 4/5/6/7.
- `SH` and `GIDP`: outs 0/1 and mask 1/5.
- `remaining`: always supported.

This is geometry support, not historical canonical eligibility or legal API
availability in a current game. Do not infer inning, actor, retouch, timing,
handling or lineup facts from it. The controlled Rust reference still has exactly
16 fictional family/context rows; do not turn it into 24 observed game states.

Freeze a single, unoptimized Jeffreys proposal: prior mass **1/2 per supported
class**, including `remaining`, independently within each context. If `N` is the
training parent count, `c_j` a supported response count and `k` the number of
supported classes, the proposed exact ratio is `(2*c_j + 1)/(2*N + k)`.
Unsupported classes have exactly zero probability and must have zero observed
counts; otherwise refuse. No concentration search, empirical-Bayes tuning,
feature selection, player conditioning, pooled replacement or heldout adjustment.

An observed empty context has `N=0` and a uniform prior over its supported classes;
label it prior-only. This is not missing data becoming observed zero. Missing or
malformed counts, context keys or source evidence are refusals. Freeze an integer
safety cap of `2^31 - 1` for total artifact events, hence each row and count; this
is an overflow/representation bound, not a new source-population filter.

This proposal is specified before future count extraction. No counts, estimated
ratios or fitted artifact are produced by this documentation task.

### 3. Artifact boundary and deterministic arithmetic

Reserve format/model ID `contact-proxy-context-counts-v1` and
`geometry-proxy-context-jeffreys-v1`; neither is implemented or selected by the
current CLI. A future count artifact is strict UTF-8/ASCII, LF terminated, no BOM,
comments, blank/duplicate/extra rows, signed numbers or noncanonical integers.
Freeze tab-separated headers in this exact order:

```text
contact-proxy-context-counts-v1
model\tgeometry-proxy-context-jeffreys-v1
season\t2019
split\ttraining
source_sha256\t<64 lowercase hexadecimal characters>
prior\t1\t2
classes\tSF\tSH\tGIDP\tremaining
contexts\t24
```

The `source_sha256` is the complete authenticated source-report bytes, not only a
selected table or this document. Follow with exactly 24 tab-separated rows,
`C <mask> <outs> <N> <SF> <SH> <GIDP> <remaining>`, where each displayed separator
is a tab, then EOF. Require `N == SF+SH+GIDP+remaining`, complete context ordering,
structural zeros and exact training-source reconciliation. Integers carry counts;
no serialized floating-point parameters. Preserve all source/protocol/model/rules
versions and original provenance in the companion manifest, not in an altered
source digest. Do not invent a final artifact digest before an artifact exists.

A future native consumer computes the positive integer numerator/denominator
exactly, then one binary64 division per supported class. No binary32, FMA,
transcendental reweighting or probability clipping. Accumulate CDF left-to-right
in `SF,SH,GIDP,remaining` order; the final CDF is exactly 1.0. For `N>0`, report
smoothing's deviation from the empirical frequency; do not pretend it is an exact
frequency fit or an accuracy gain.

### 4. Sampling and canonical-transition boundary

No RNG is used in this task or the proposed read-only response-count audit.
Before a later standalone sampler is implemented, its versioned contract is one
externally supplied u64 word, its high 53 bits scaled by `2^-53`, strict `<` CDF
selection in the fixed order above, no redraw/rejection loop. A producer PRNG
schedule, snapshot/schema change, new CLI model ID or default change needs its
own scope and compatibility evidence; this document does not alter existing
seeds or formats.

A sampled geometry class is not a historical fact object. The ordinary producer
returns `(Outcome, Event)` for ordinary `apply`; the separate supplied projector
calls fact-bearing Rust APIs and returns `EventTransition`. Credits or matching
endpoints must not be used as an implicit bridge. A later modeled policy must
supply eligible actors and every required handling/retouch/timing/bunt-judgment/
force-relay/third-out/clean-held assertion explicitly and retain their modeled,
not observed, status. Rust alone validates legality and computes the after-state.

A finished game, invalid lineup or malformed before-state is a refusal, not a
fallback opportunity. For an otherwise legal active PA, distinguish an explicitly
unsupported canonical-policy domain (such as a documented late-bottom restriction)
from malformed artifacts, illegal supplied facts and programming errors. Proposed
unsupported-domain fallback is the unchanged residual-parent path with the already
supplied randomness; retain selected versus applied class in diagnostics. Do not
redraw, renormalize other classes, discard source rows or use a second defense roll.
Any programming error, malformed evidence or illegal supplied fact is a refusal,
not a concealed fallback. The exact ordinary-parent application and word ownership still require
a separate adapter contract before sampler/producer implementation; no such
adapter is authorized here.

### 5. Frozen numerical checks and honest evaluation

| Check | Acceptance limit or interpretation |
| --- | --- |
| Source parent allocation, all old folds, splits, repeated exports, artifact integer counts | Exact equality; zero lost/extra/double-counted events |
| Required contexts, structural zeros, schema, provenance, field-level unknowns | Exact; any contradiction or alias refuses |
| Exact proposed rational numerator/denominator | Exact integer equality; no fitting tolerance |
| Native/reference binary64 probabilities and CDF entries; probability sum | Absolute error at most `32 * 2^-52`; finite, nonnegative and monotone; final CDF exactly 1.0 |
| CDF grid-boundary/neighbor choices, context fallback and seeded bytes | Exact decisions/bytes; all supported contexts and every boundary/neighbor tested |
| Coarse source-label NLL/Brier and frequency residuals, by original split/context with sample sizes | Report diagnostics and ensemble variation if a later stochastic experiment exists; no predictive/model-selection pass is licensed |
| Claimed universal held-runner consequence-mask agreement | Zero known contradictions required; the existing 286 differences mean **NO-GO** for that claim |
| Historical canonical eligibility, causal handling or tactical opportunity | Unavailable; no numerical match can supply the missing facts |

The binary64 bound is computational parity, not a tolerance for historical rate,
run distribution, player accuracy or mechanism fidelity. No production accuracy
threshold is invented or waived here. No multiple-model leaderboard or tuning
based on already examined heldout results is permitted.

Always retain and report all 4059 geometry memberships: 2256 controlled-mask
matches, 286 known differences and 1517 boundary-not-compared. Preserve the family
and original split tables, all 1330 nongeometry memberships and the complete
source-event denominator. No normalization of boundary fields and no selection of
2256 matched rows. These are existing audit results, not new measurements here.

Even perfect occurrence-count reconstruction cannot repair an incorrect or
incomplete consequence model. The known differences remain a visible limitation
of held-other-runner modeled episodes; the boundary cases are censored, not
matches. Neither absent literal diagnostics nor a good proper score proves
historical handling, retouch, timing or force/relay causes.

### 6. Bounded decision and next package

This is a proposed, frozen route for an explicitly coarse descriptive experiment,
not an approved historical-cause importer or production simulator. **NO-GO remains
for rate fitting, model selection, a native sampler/adapter, producer/manager
integration and any production accuracy or release gate in this task.**

Next ready package after this document's verification/checkpoint:
**F-01-contact-proxy-response-counts**, a separately preregistered read-only
source-event census. Reproduce complete inherited evidence first; allocate the
four responses over the entire residual parent, preserve conflicts/missingness/
differences/boundaries and all original split/context folds, and cross-check every
cell independently. Publish counts and sample sizes only, not estimated rates,
artifacts with invented parameters, a sampler or gate approval. New executable
work will again require semantic regression/property controls and the entire
applicable fresh offline/locked suite, not this document's checks.

Documentary/protected-source verification and the two-path checkpoint remain
pending at this writing. No new scientific-source count, fit or source/API fact
has been manufactured by the specification above.

## Documentary verification before checkpoint

Fresh `python3 -Werror .../checkpoint-v1.py verify` exited 0: all 266 protected
paths, 167 unchanged scientific sources, 111 original recipes, 15 prerequisite
bindings, closed artifact groups and the unchanged protocol prefix authenticate;
599 current prose links resolve. The existing prose-link function is AST-exact,
not a replacement validator. Its two link regressions pass in 0.007s and its one
tracking regression in 0.005s, including real broken-link refusal and duplicate
tracking behavior. Werror compilation of the private helpers and `git diff --check`
pass. Raw verification/test streams remain under the task root.

No scientific executable or test changed, and no new full Rust/Python scientific
suite is claimed. The unchanged sources retain their preceding verified full-v3
identity. Scope is exactly the two documents; private candidate, actual changed-doc
inspection, once-only local commit and identity-only postcheck are the remaining
checkpoint actions. The rate-fitting, sampler/adapter, producer/manager and
production gates remain NO-GO in this task.
