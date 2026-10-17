# Singles/doubles allocation v3 — bounded preregistration

Written 2026-10-17 before any v3 fits or scores. Parent checkpoint `7422997`;
prior [v2 results](player-calibration-v2-results.md) and rejected v1 remain frozen.
Maintainer asks to continue the recommended singles/doubles follow-up. Interview
`904e45e4-96ee-44ac-ac18-bfeec32731a8` grants **up to60 minutes**, stopping with an
honest checkpoint if incomplete, and a verified local private evidence backup.
This pass closes by18:59UTC; reserve the final15 minutes for checks/handoff.
No further score-driven candidates, grids or threshold changes in this pass.

## Question and unchanged boundary

Can less shrinkage in the existing double-versus-single node improve individual
1B/2B reconstruction without losing the earlier tree's strengths? V2 selected
200/200 at that node without any separate 1B/2B constraint; its worst qualifying
batter deviations were +4.3776pp singles and −4.1172pp doubles. These are known
retrospective diagnostics, not a newly discovered independent test population.
The target is hindsight selected-season reconstruction, not forecasting.

Use only the same audited2019 literal terminal-participant counts, category order,
whole-game SHA256 split and actual opponent graph. Original denominators are all
nine included terminal outcomes, **not official PA/BF or conditional hit counts**.
No data transfer, source mutation, new dependencies, UI, mechanics, runtime
integration, artifact export/default-model switch, commit/push or gate signoff.
The existing v1/v2 commands and runtime artifact remain byte-compatible.

## Fixed candidate and selection rules

Use the identical eight-node probability tree and binary augmented-likelihood
solver:5000 cycles, correction≤1e-8,20 neutral-gauge substeps, unchanged priors.
Freeze strengths at20/20 for nodes0–4 and200/200 for nodes5 and7, the v2 choices.
Refit those fixed nodes on training counts, **without reselection**. At node6 only,
compare the16 pairs `lambda_b,lambda_p in {1,5,20,200}`; no added strengths.

For each role separately and players with≥200 original training opportunities,
require singles **and** doubles absolute rate-bias MAE≤.01 and maximum≤.03, and
MAE/maximum no worse than the training fixed200 multinomial baseline. Rates are
fractions (.01=1 percentage point). Qualify with complete nine-vector expected
counts over actual opponents, not neutral-opponent conditional rates. Minimum
validation conditional binary NLL chooses among qualifying pairs; ties within
1e-10 favor ascending strengths. Selection reads no test/full-refit counts.
Publish all16 candidate/convergence/qualification/validation receipts and chosen
strengths before test scoring. No qualified candidate means explicit refusal,
not fallback. Missing/cold roles keep the existing strict-input/neutral-prior rules.

Refit selected strengths on the full graph **without reselection**. Earlier nodes
are identical to separately refitted frozen-v2 nodes. Only singles/doubles may
change in a nine-vector; their sum and every other category stay unchanged within
binary64 tolerance1e-12. Build this entire positive normalized vector before one
parent draw; no tree-node rolls, hit/out conversion, repair or second reducer.

## Frozen development acceptance checks

1. Full graph: for batters and pitchers with≥200 original opportunities, singles
   and doubles each MAE≤.01 and maximum≤.03; both summaries no worse than full
   fixed200 **and** frozen-v2. Preserve all20 role/category bias summaries and
   worst identities, including failures; report exposure bands and sparse roles.
2. Preserve all original v2 full-reconstruction/relative-common-category bounds:
   K/BB/HR/combined hits MAE≤.01,max≤.03 and no worse than fixed200; HBP/3B/ROE
   MAE≤.005,max≤.02. Do not change a published v1/v2 acceptance rule.
3. Reused test benchmark: keep the original versus-league upper95% game-cluster
   difference<0 and mean NLL versus fixed200≤+.01. Additionally require v3 mean
   NLL minus frozen-v2≤+.001. Report NLL/Brier/category contributions and paired
   whole-game intervals versus both baselines, not just the acceptance booleans.
4. Retain the200 fixed-opportunity ensemble centering/variation checks unchanged:
   |mean−expectation|≤5sqrt(V/200)+.05; SD>0 for V≥5; SD within20% of sqrt(V) for
   V≥25. This is sampling/centering evidence, not realistic historical variance
   or propagated ability uncertainty. Positive zero-observed outcomes stay legal.
5. Independent Python arithmetic recomputes all acceptance cells, scores and
   probability products from source/effects, with bound input/result hashes.
   It is not an independent fitter or independent data. A disposable fold0
   perturbation must leave pretest selection/training effects unchanged.

Every requirement is conjunctive. Failure is retained; do not choose a different
candidate using full-data reconstruction or test scores. A statistically rejected
comparison can still have verified code. No automatic runtime integration follows.

## Honest validation design

The original train/validation/test split is retained solely for comparability.
V1/v2 already examined test scores and full-season player counts; this follow-up
was motivated by those results. Both validation and test are **reused development
partitions**. New random splits or cross-validation of this same examined corpus
would not create a genuinely untouched external evaluation. Intervals are
conditional descriptive diagnostics, not post-selection predictive certificates.

For a later predictive claim, freeze the complete candidate/selection pipeline
before evaluating genuinely unexamined, separately audited opportunities, or an
as-of-date forward evaluation with no future-outcome ratings. That needs a separate
scope/data decision; no new season or source transfer is authorized here. For this
product's primary hindsight reconstruction goal, label actual-opponent full-data
centering honestly and assess opportunity/schedule/parameter uncertainty separately.

## Test, verification and stopping contract

Before implementation, observe failing tests for: missing v3 CLI mode; separate
1B/2B bounds and fixed200 comparison (including exactly .03 versus over .03);
unchanged non-hit probabilities/1B+2B conservation; v3 scientific rejection when
v2-only checks would pass; independent version and comparison handling; no v3
runtime export. Preserve existing parser/refusal/source tests.

Run all applicable project checks: offline locked examples build; full Rust tests;
example tests; fmt; strict all-target Clippy; full Python discovery with
ResourceWarnings as errors; diff check. Rerun original v1/v2 comparators to NEW
external outputs and check exact old-result parity. Native/Python scores and
fixed-opportunity ensembles, source/effect pins, and explicit failures are kept
outside Git. New code changes require fresh verification; no historic receipt
is retroactively repaired by this pass.

Evidence root: `/tmp/bbsim-player-hit-mix-v3/`. Back up the prior calibration study
and eight named existing local inputs, with source/destination hashes and a Git
archive of7422997, under the private Application Support checkpoints directory;
leave originals unchanged. This is not a backup of every historical `/tmp` study.

At the cap, record exactly what passed/failed/remains unrun and one concrete next
action in `docs/PROGRESS.md`; do not claim completion from partial checks. No
additional reviewer delegation is authorized by this request.
