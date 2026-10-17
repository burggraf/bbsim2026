# Singles/doubles v3 — stronger reconstruction, rejected development comparison

2026-10-17. [Preregistered contract](player-hit-mix-v3.md), parent`7422997`.
**Scientific disposition: rejected.** Software verification is recorded separately
below; completing a comparison does not accept its model or complete P-01/G1–G5.
The scientific pass closed without runtime integration, default change, source
transfer, commit or publication. Later source-only publication authorization and
receipts are tracked separately in the live ledger.

## Frozen study and result

Only double-versus-single node6 varies. All16 pairs from`{1,5,20,200}²` converged;
9 met the separate training singles/doubles constraints. Training-qualified
minimum validation conditional NLL selects **20/20**, rather than v2's200/200.
Training qualifiers:320 batters/276 pitchers with≥200 literal terminal proxies.
Full-data selected strengths are refitted without reselection. Nodes0–4 remain
20/20;nodes5 and7 remain200/200. No extra draw or hit/out roll is introduced.

Both v1 and v2 retain their historical results:all110 v1 output files and all19
v2 output files reproduced **byte-for-byte** in new external directories. V1
remains rejected. The separately refitted v2 reference training effects,full
effects and full-bias report match their original v2 files byte-for-byte.

### Individual full-data reconstruction

Population:359 batters/364 pitchers with≥200 original included opportunities.
Every figure below is **MAE / maximum absolute bias in percentage points**;
smaller is better. These are expected rates over actual opponents,not neutral
conditional rates,official PA/BF,or true-ability estimates.

| Role / outcome | Fixed200 | V2 | V3 |
| --- | ---: | ---: | ---: |
| Batter singles | .7820 / 4.3070 | .7622 / 4.3776 | **.3714 / 1.7193** |
| Batter doubles | .3257 / 2.6726 | .6238 / 4.1172 | **.2077 / 1.4218** |
| Pitcher singles | .7942 / 3.7124 | .7339 / 3.2996 | **.3740 / 2.0470** |
| Pitcher doubles | .4020 / 1.8175 | .6362 / 2.7579 | **.2370 / 1.1624** |

All four cells pass MAE≤1pp/max≤3pp and no-worse summaries versus both baselines.
V3 worst IDs:battersingles`solad001`(−1.7193pp),batterdoubles`buxtb001`(−1.4218pp),
pitchersingles`garcy001`(+2.0470pp),pitcherdoubles`obers001`(+1.1624pp).
K/BB/HBP/HR/3B/ROE/contact-out probabilities remain unchanged;1B+2B is conserved
to1e-12. Prior v2 K/BB/HR/total-hit and rare-outcome reconstruction checks pass.

### Reused development benchmark — the failed constraint

19,002 opportunities/249 whole games. Lower NLL/Brier is better.

| Model | NLL | Brier |
| --- | ---: | ---: |
| League | 1.5540884065 | .7210869325 |
| Fixed200 | 1.5305688737 | .7112969002 |
| V1 joint20/20 | 1.5409581951 | .7108561703 |
| V2 | **1.5276428749** | **.7100148394** |
| V3 | 1.5286528511 | .7101437268 |

- V3−v2 NLL **+.0010099762271294555**;descriptive paired-game95% interval
  `[+.0003094166069714309,+.0017105358472874802]`.
- The frozen bound is mean≤+.001. The excess is **.0000099762271294555**;
  it is small but a genuine failure,not a rounding tolerance or waived constraint.
- V3−fixed200:−.0019160225599318295;interval
  `[−.0038935227279965237,+.0000614776081328647]`. Its original≤+.01 mean bound passes;
  this interval does **not** establish superiority over fixed200.
- V3−league:−.025435555426506676;interval
  `[−.03007062432927528,−.02080048652373807]`;original league guardrail passes.
- The entire v3−v2 NLL loss is singles+.0002268063968743 and
  doubles+.0007831698302547;all other category contributions are unchanged.

The source was already examined in v1/v2 and motivated this follow-up. Validation
and test are **reused retrospective development partitions**,not independent
predictive validation. No new grid,threshold or candidate was added after seeing
this failure. Full reconstruction and fixed-opportunity variation pass,but
`heldout_guardrails:false` and **`accepted:false`** remain conjunctive rejection.

## Independent arithmetic, variation and isolation

`proc_7029` exits0/58s:independent Python arithmetic audits v1/v2/v3,including all16
node6 training-qualification/validation-loss receipts and the selected20/20.
It agrees on every acceptance decision,including v3 rejection. Source/result
hashes bind the audit;this is not an independent fitter or independent dataset.
A disposable fold0 count perturbation preserves **27 pretest files byte-for-byte**:
selection,training effects,all8 node tables,frozen-v2 training effects,and all16
candidate effect tables. Frozen-v2 train/full/bias references match old artifacts.
`proc_672a` exits0/76s:1000 fresh ordinary fixed200 games reproduce all2000
stdout/stderr files byte-for-byte against the prior cohort. Real rejected-v3
export refuses with no output;the existing opt-in v2 artifact remains SHA256
`905ee5c6153f67e474409b0f70e55cec1ffa913e511afe901cc1bbc4d820fb43`.

200 fixed-opportunity ensembles reconcile364,200 profile rows/16,389 variation
cells across1,821 role-profiles. Centering/SD checks all pass,no variance failures.
Minimum audited observed-matchup probability=.00003119380699539007;no lost mass.
24 batting/12 pitching profiles have zero training exposure and retain neutral
priors. Missing runtime profiles still refuse;zero exposure is not missing data.
120 supplemental role/exposure-band/category reconstruction rows reconcile the
whole185,703-opportunity population. Undefined zero-exposure rates are null.
None of this propagates rating uncertainty or proves realistic schedule/usage/
season variance. Literal terminal participants are not complete official credits.

## Verification and private evidence

Full project verification **passes**:managed`proc_eb92` exits0/593s,
18:17:26–18:27:19UTC,with197 equal before/after sourcepins still matching current
code:867 Rust tests/63 zero-failure summaries,3 example tests,15 focused tests,
623 Python tests in56.976s withResourceWarnings as errors,examples build,fmt,
strict all-target Clippy anddiff check. Exact commands:

```sh
cargo build --offline --locked --manifest-path engine/Cargo.toml --examples
cargo test --offline --locked --manifest-path engine/Cargo.toml --test full_player_game --test player_calibration --test player_calibration_tree
cargo test --offline --locked --manifest-path engine/Cargo.toml
cargo test --offline --locked --manifest-path engine/Cargo.toml --examples
cargo fmt --manifest-path engine/Cargo.toml --check
cargo clippy --offline --locked --manifest-path engine/Cargo.toml --all-targets -- -D warnings
python3 -B -W error::ResourceWarning -m unittest discover -s tests -v
git diff --check
```

API/semantic boundary failures remain in external`*-red.*`logs. New regressions
cover .03 versus over .03,each role/category's baseline constraint,.001 versus
over .001,nonfinite/ordered intervals,version/export isolation,complete/exact
candidate grids,and unchanged nonmix outcomes/conserved1B+2B. Closure also
reverifies all4653 prior private files against unchanged originals and714 relative
Markdown targets. Its first supplemental regex incorrectly interpreted a Python
call inside a fenced shell example as a link;raw failure is preserved and the
corrected code-fence-aware checker is self-checked. No project suite/criterion
was replaced,weakened or skipped.

A separate documentary commit`6bd2660` arrived while checks ran. It adds only
card-game research and its ledger entry;none of the197 tested sources changed.
That work is preserved. At scientific closure18:36UTC,the modeling pass was
**uncommitted and unpublished**,with HEAD6bd2660,not a v3 modeling commit.
The maintainer subsequently authorized source/documentation commit and push;
that publication does not change this study's rejection or runtime defaults.

External evidence:`/tmp/bbsim-player-hit-mix-v3/`:
`preregistration-before-fit.md`,`preregistration.json`,`red-results.json`,
`focused-results.json`,`comparison-lifecycle.json`,`comparison-v3/`,
`repeat-v1/`,`repeat-v2/`,`independent-{v1,v2,v3}.json`,
`independent-and-isolation.json`,`audit-lifecycle.json`,`supplemental.json`,
`verification/`. Raw native/Python stdout/stderr remain alongside receipts.
PreregistrationSHA256:`24457588b558b61aaf886760e50b71e7220df5ed1fe9ce78f6edb52d64be0036`.

Before changes,the prior calibration study plus eight named existing local inputs
and a Git archive7422997 were copied and verified privately:4653 files,
283,904,171 bytes;manifestSHA256
`99fa97db7499765eb1ac4c74960bbfb6c7a3833064d29e891522dd22355a8afc`.
Checkpoint:`/Users/markb/Library/Application Support/bbsim2026/work-checkpoints/player-calibration-7422997-private-20261017T180157Z`.
Originals remain untouched;this does not cover every older external study.
The follow-up private copy target is
`/Users/markb/Library/Application Support/bbsim2026/work-checkpoints/player-hit-mix-v3-7422997-private-20261017-v1`:
its verified`manifest.json`/`backup-current-result.json`is the receipt,with raw
follow-up evidence,seven working-tree source/test/documentation files and a
source archive of the preserved current HEAD. This is local,not off-device or
remote durability;see its copied`RESTORE.md`and retain both private checkpoints.

## Reproduce without overwriting retained outputs

Use existing audited inputs or privately restore the named inputs from that
checkpoint;no new download is needed. Choose **new** external output names:

```sh
cargo build --release --offline --locked --manifest-path engine/Cargo.toml --example calibrate_players
engine/target/release/examples/calibrate_players \
  /tmp/bbsim-player-calibration-v1/input/opportunities.tsv /tmp/NEW_HIT_MIX --hit-mix-v3
python3 -B -W error::ResourceWarning -m audits.player_calibration_review \
  /tmp/bbsim-joint-contact/context-first.csv \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-player-calibration-v1/input /tmp/NEW_HIT_MIX /tmp/NEW_HIT_MIX_AUDIT.json
```

Successful process exit means the comparison/audit completed,**not** that its
model passed;inspect`selection.json`/audit`accepted:false`. V3 effects have a
separate version and cannot export as the existing v2 runtime artifact.

**Next decision:** present this fidelity-versus-score tradeoff before any runtime
adoption or revised study. Keep v2 as the opt-in reference and fixed200 as default.
A different future reconstruction-first objective needs its own preregistration;
do not retroactively relabel this v3 comparison accepted. Broader individual
fidelity,ability uncertainty,official denominators,park/handedness/defense,
playing time and independently audited predictive evidence remain open.
