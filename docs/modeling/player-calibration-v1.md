# Individual-player calibration v1 — preregistration (2026-10-17)

## Authority, purpose and limit

Maintainer: “continue working on this — let's get the model right!”; interview
`30bb9104-9902-4b15-a196-299c3a054e16` approves up to2 hours with a60-minute
checkpoint. Start15:48UTC; checkpoint16:48UTC; stop/report by17:48UTC.
P-01/P-02 remain incomplete. Selected-season **hindsight reconstruction**, not
forecasting or proof of true ability. Quantitative reconstruction is the primary
objective; retrospective held-out checks guard against a pathological model, not
an authorization to replace the reconstruction objective with prediction.

Scope: reuse audited offline2019 participant events/counts, compare small fixed
models, test fitting/sampling, and integrate only a qualifying version into the
complete text game. Preserve legacy/starter-preview and explicit pooled behavior,
strict current-role/full-cast refusals, interference mass, actual draw receipts,
canonical transitions/accounting and all existing mechanics. No UI, new events,
handedness/park/fielding estimates, dependencies/install/network/source transfer,
source mutation, season expansion, release/push or gate signoff.

Likely seams: new `audits/player_calibration.py`, `tests/test_player_calibration.py`,
`engine/examples/{calibrate_players.rs,support/player_calibration.rs}`,
`engine/tests/player_calibration.rs`; bounded changes to complete CLI/shared
probability combination and their tests if a candidate qualifies. This contract
and `docs/PROGRESS.md` record evidence. Native fitting stays outside the canonical
rules engine. Real source/effects/diagnostics remain outside Git.

## Fixed source and partition (before model calculations)

Reuse, do not reclassify: `audits.joint_contact.GROUPS/CATEGORIES` and the complete
`audits.player_opportunities.inventory`/outcome validator. Source
`/tmp/bbsim-joint-contact/context-first.csv`, SHA256
`ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a`;
original report `/tmp/bbsim-player-opportunities/report-first.json`, SHA256
`382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e`.
Require full192025-event validation,185703 included outcomes,753 IBB/61 interference
exclusions and every990 batter/831 pitcher nine-vector matching that report.
Retain original snapshot/provenance/attribution and hash inputs before/after.

Partition whole games using unsigned big-endian first8 SHA256 bytes of
`b'player-calibration-v1\0'+game_id.encode('utf-8')`, modulo10:
0=test,1=validation,2..9=training. Keep every source game/opportunity, including
cold-start identities. This is a new **retrospective** split, not a previously
blinded sample: full-season summaries/selected profiles have already been seen.
Train league/player parameters only on training rows; select only on validation;
inspect test scores only after selection. Refit the selected specification on
all2019 rows only for hindsight reconstruction/export. No parameter selection
from test results, no changed split/filter/threshold after measurement.

## Fixed candidates and fitting

Nine mutually exclusive categories: K,non-IBB BB,HBP,1B,2B,3B,HR,ROE,contact.
Training league `ell[k]=(C[k]+.5)/(N+4.5)`. Every legal category positive;
zero-exposure training roles use neutral league effects, not another identity.

1. League-only Jeffreys baseline.
2. Existing Dirichlet/generalized-log5 baseline: kappa200 per role.
3. Same formula with separate batter/pitcher kappas from fixed Cartesian grid
   `{1,5,20,50,100,200,400,800,1600}`. No continuous optimization/additional grid.
4. Opponent-adjusted regularized multinomial log-linear challenger:
   `P(k|b,p)=ell[k]*a[b,k]*c[p,k]/sum_j(ell[j]*a[b,j]*c[p,j])`.
   Fit the joint observed batter–pitcher opportunity graph, not independent
   player marginals. Add each batter versus a neutral pitcher and each pitcher
   versus a neutral batter synthetic **league-distributed pseudo-opportunities**
   of strength lambda_b/lambda_p from `{1,5,20,200}` Cartesian grid. These are
   regularization, not invented source events or official opportunities.
   Maximize the augmented multinomial likelihood with cyclic iterative
   proportional fitting: normalize each pair to its fixed exposure, match
   batter/category margins, then pitcher/category margins; neutral factors fixed1.
   Initialize all factors1; positive normalized factors; max5000 cycles;
   converge when both sets of multiplicative margin corrections differ from1
   by at most1e-8. Synthetic pre-measurement refinement16:06UTC: plain cyclic IPF
   leaves a weakly identified category gauge (`a*g,c/g`,which preserves every
   real-matchup probability) and missed the fixed tolerance after5000 cycles on
   the literal confounding fixture (correction5.3824e-8). Add20 neutral-case
   gauge-balancing substeps per cycle: with fixed neutral case normalization,
   maximize each category's concave Poisson surrogate
   `D*log(g)-A*g-B/g`, positive quadratic root `A*g^2-D*g-B=0`, then renormalize
   neutral cases. This changes solver coordinates,not model/objective/grid,
   tolerance or maximum cycles;fix and verify on synthetic data before real fits.
   Retain convergence/iteration/margin-error receipts. A failed
   convergence/finite check is a failed candidate, never silently accepted.

The challenger is a regularized multinomial point estimator, **not a claimed
Bayesian posterior**. Neutral-opponent distributions may be stored as normalized
nine-vectors and combined through the same product-over-league arithmetic.
Only existing standard libraries/native Rust; no new dependency for optimization.

## Selection and observable acceptance

Record **all** candidates, failures, parameters and validation/test metrics.
Selection is constrained by the training reconstruction objective, then minimizes
validation mean negative log likelihood (nats/PA); numerical ties within1e-10
prefer the earlier simpler candidate order above, then ascending parameter order.
No iterative test-based adjustment. A rejected selection leaves runtime unchanged.

### Reconstruction constraints

For each role/category, expected counts are sums of matchup probabilities over
that role's **actual observed opponent opportunities**, not a neutral opponent
or the opening exhibition bullpen. Assess players with at least200 included
opportunities, separately for batters and pitchers. For K,BB,HR and combined hits:
player-unweighted mean absolute rate bias <=.01 and maximum absolute bias <=.03.
For HBP,3B,ROE: mean absolute bias <=.005, maximum<=.02. Report all nine categories,
all exposure bands from the original audit, signed biases and worst identities;
no event-count filter that hides zeros. These are provisional reconstruction
acceptance bounds (percentage-point units), not predictive guarantees.

Candidates must meet these constraints on training opportunities to enter
selection. The selected all-season refit must meet them again on full-season
opportunities, with each role/common-category MAE and maximum bias no worse
than the existing200 baseline.
If it fails, report failure; do not switch to the next test-favored candidate.

### Retrospective held-out guardrails

For every candidate report held-out NLL, multiclass Brier score, category O/E,
role/exposure-band scores and cold-start exposures. For the selected candidate,
paired game-cluster95% normal CI for NLL difference versus league-only must have
upper endpoint<0; mean NLL versus existing200 must be no more than+.01 nats/PA.
The+.01 guard permits a small predictive tradeoff for the explicit reconstruction
objective; it is fixed before scores, not a claim of the best predictive model.
Report both differences/intervals even on failure. No forecasting claim.

### Variation and native integration

For a qualifying full-season refit, evaluate fixed historical opportunities and
200 fixed seed1..200 independent categorical season-length ensembles (not full
schedule/game replays). Record every role/category expected count, summed
Bernoulli variance, empirical mean/SD and percentiles, including zero results.
Monte Carlo means: `abs(Omean-E)<=5*sqrt(V/200)+.05` for every cell;
nonzero empirical variance where V>=5; empirical SD within20% of sqrt(V) for
V>=25 (diagnostic failures retained; no cohort expansion). This validates the
sampler/model-implied variation, not true ability uncertainty or playing time.

Expose shrinkage/regularization, role exposures, provenance and lack of propagated
parameter uncertainty. Repeat full-game seeds1/2/3 and fixed1..1000; retain all
stdout/stderr, no retry/discard/tuning. Independently recompute actual current
actors/probabilities/CDF words; every game executes native accounting/continuity.
Ensure a changed reliever's effects act only after entry. Pooled interference
alone remains tenth parent; no IBB or runner-only rating draw.

### Tests and verification

Test before implementation: split/game isolation, source/vector conservation,
invalid/changed inputs/output collision; literal posterior/neutral and joint
fit fixtures; training-only parameters/heldout independence; convergence failure;
finite positive normalized cold-start/zero-cell behavior; exact missing/extra-role
and source/version refusals; native-versus-independent numerical predictions;
CDF boundaries/determinism and legacy compatibility. Preserve semantic reds and
compiler/harness failures separately.

After integration freeze source, run offline locked full Rust/examples/build,
fmt,strict all-target Clippy `-D warnings`,full Python discovery with ResourceWarnings
as errors and diff checks. A bounded independent review may use the existing
authorized reviewer lane; verify findings before correction. No fitting result,
cohort or green suite independently signs P-01/P-02/G1–G5 off.

## Results

No new model scores/fits measured when this preregistration was written.
Evidence root: `/tmp/bbsim-player-calibration-v1/`. The live ledger owns status,
60-minute checkpoint, exact commands/results/failures and the concrete next action.

### Measured disposition (2026-10-17; appended after preregistration)

Validation selected `joint-20-20`, but its test NLL difference versus fixed200 is
+.010389321421514129, exceeding the unchanged+.01 limit. Reconstruction and
variation pass; **v1 remains rejected and was not integrated**. All110 comparison
files repeat byte-for-byte after the generic2/9 specialization. Independent
Python recomputation confirms the rejection. The separately preregistered v2
follow-up passes development bounds on the reused benchmark, not fresh validation.
See [measured comparison, limitations and reproduction](player-calibration-v2-results.md).
The original specification above is unchanged; original bytes are retained in
`/tmp/bbsim-player-calibration-v1/preregistration-v1-before-results.md`.
