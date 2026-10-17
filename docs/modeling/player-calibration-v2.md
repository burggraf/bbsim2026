# Outcome-specific opponent calibration v2 — follow-up preregistration

Written2026-10-17 after the fixed[v1 comparison](player-calibration-v1.md),within
the same authorized15:48–17:48UTC pass/checkpoint16:48UTC. V1 selected joint20/20,
passed reconstruction/variation but **failed** its unchanged held-out guardrail:
NLL versus fixed200 +.0103893214 >+.010. No v1 runtime integration authorized by
that result. Preserve its inputs/results/failed acceptance;do not expand its grid,
change its tolerance or call this follow-up a v1 pass.

## Scientific boundary

A shared concentration across all outcomes forces rare-event and strikeout
shrinkage to move together. Compare one fixed outcome-tree point model with
outcome-specific regularization. No new data/season/covariates/dependencies,
fielding roll,manager or native mechanics. Same hindsight participant-proxy goal.

**The v1 test partition has been used.** Keep its game split for comparability,
train each new parameter on training rows and choose only on validation rows,
but report test scores as a reused **retrospective development benchmark**, not
fresh independent validation or predictive acceptance. No P-01/P-02/G1–G5 signoff.
No further candidate/model/threshold changes based on this follow-up's scores
within this pass. A failure remains failure and leaves the CLI model unchanged.

## Fixed probability tree (before any v2 fit/scores)

Compute a complete positive nine-category vector **before one CDF parent draw**.
No random roll at the individual tree nodes and no hit converted to an out:

1. K versus all other terminal outcomes.
2. Non-IBB BB,conditional on not K.
3. HBP,conditional on neither K nor BB.
4. Any hit(1B/2B/3B/HR),conditional on none of the above.
5. HR,conditional on a hit.
6. 3B,conditional on a non-HR hit.
7. 2B versus1B,conditional on a non-HR/non-3B hit.
8. ROE versus contact-out,conditional on a non-hit/non-K/non-BB/non-HBP.

Define each risk set from those literal source categories,not invented PA or
fielding denominators. Train league conditional rates from the same training
nine-category Jeffreys league (ratio of the relevant masses),with no validation
or test counts. For each binary node fit the same augmented likelihood/IPF and
neutral-prior gauge solver as v1,specialized to two categories. Pseudo-opportunities
remain centered on the node's training league rate. Use the fixed Cartesian grid
`lambda_b,lambda_p in {1,5,20,200}` at **each** node,with the same5000-cycle/1e-8
convergence refusal and20 gauge substeps. Generic array-size specialization must
preserve v1 behavior;test it against retained v1 source/binary/results.

For each node in the order above,consider all16 fits. Keep earlier selections
fixed;unfitted descendant nodes have neutral factors solely to construct a
complete vector for fitting diagnostics. Among fits satisfying the original v1
training reconstruction bounds for the newly fixed observable,select minimum
validation **conditional binary NLL**,ties1e-10 favor ascending strengths.
Conditional NLL decomposes the complete multinomial NLL for this fixed tree.
Newly fixed observables: K,BB,HBP,combined hits,HR,3B,none(no new bound for2B),ROE.
Check means over each player's original full terminal-opportunity denominator,
using the actual opponent graph and>=200 full included opportunities. For each
newly fixed **common** observable(K,BB,hits,HR),also require training MAE and
maximum bias no worse than the training fixed200 baseline:carry the final
no-worse requirement into selection rather than knowingly selecting a worse
centering candidate. This additional selection constraint is fixed before v2
scores;the v1 numerical bounds and final guardrails are unchanged. Earlier
observables cannot change when descendant probabilities are fitted. If no fit
qualifies at any node,the v2 model refuses;do not relax a bound or use a fallback.

Refit the **selected eight strength pairs** on the full2019 graph,without
reselection,and enforce the same full-refit bias/no-worse-common-category checks
as v1. Preserve all candidate/convergence/risk-set/validation receipts.

## Evaluation and conditional development integration

Reuse every v1 quantitative bound unchanged: league comparison upper95% clustered
CI<0;mean NLL versus fixed200<=+.01;full reconstruction bounds;each role/common
category MAE and max no worse than fixed200;200 fixed-opportunity ensembles with
original means/SD checks. Retain all failures,all source players/bands,all zeros.
Compare v1 and v2 results side by side,including Brier and category contributions.
The reused benchmark cannot provide a new independent uncertainty/accuracy claim.

If all bounds pass,allow an explicitly supplied,versioned **development** tree
artifact in the complete text CLI;do not silently replace legacy preview or the
reproducible v1 command. Source-count validation still requires all18 hitters/
all12 pinned pitchers;missing current role/effects refuse,no league/starter
runtime fallback. Conditional probabilities are finite,strictly between0and1;
normalized full parent vector;retain pooled interference as tenth category,
tactical IBB/runner-only paths and actual actors/word/vector/parent receipts.
Artifacts/provenance remain external,no source bundle or public publication.

A new empirical mean is not a Bayesian ability posterior. No propagated parameter
uncertainty,handedness/park adjustment or historically validated variance magnitude.
Positive zero-observed outcomes remain legal;report sparse profiles explicitly.

Tests before implementation: two-category literal fits/neutral/zero-cell and
confounded fixtures;all risk-set conservation and exact probability-tree products;
node sensitivity without a second outcome draw;version/role/cohort/control/bounds
refusals;current reliever behavior and independent actor/CDF reconciliation.
Retain full v1 comparison equality after generic specialization. Then full project
checks/source pins,independent numerical audit and bounded review before claiming
this development checkpoint verified. No gate or official-credit equivalence.

## Results

No v2 fit/scores measured at preregistration. Evidence root remains
`/tmp/bbsim-player-calibration-v1/`,separate versioned v2 outputs;live status in
`docs/PROGRESS.md`. Remaining budget and mandatory checkpoint are unchanged.

### Measured disposition (2026-10-17; appended after preregistration)

The fixed tree selects20/20 for the first five nodes and200/200 for the last three.
It meets the unchanged numerical bounds: reused test NLL1.5276428749, versus
fixed200−.0029259988; full reconstruction/no-worse-common checks and200 fixed-
opportunity ensembles pass. **Development acceptance only**, not fresh predictive
validation. A strict explicit-artifact CLI path is verified; default outputs remain
unchanged. Full866 Rust/620 Python checks and separate1000-game calibrated/default
cohorts pass. Singles/doubles allocation still has roughly4pp worst qualifying
batter biases and is a concrete follow-up, not a hidden completed objective.
See [full results, limitations, review and reproduction](player-calibration-v2-results.md).
No criteria/candidates above changed after scoring; original preregistration bytes
are retained in`/tmp/bbsim-player-calibration-v1/preregistration-v2-before-results.md`.
