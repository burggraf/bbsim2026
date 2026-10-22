# Pitcher usage policy v2 — fixed-seed development results

**Evaluation completed:** 2026-10-19; results recorded 2026-10-20.  
**Disposition:** the preregistered three-seed retrospective development evaluation **passes its frozen per-seed usage criteria**. This is not a model promotion or gate signoff: v1 remains NO-GO and the default; v2 remains opt-in.

This report records the fixed evaluation in the [v2 preregistration](pitcher-usage-policy-v2.md), which was sealed before implementation and simulation. Corrected independent evaluator `proc_3204` exited 0 with all three seeds PASS, using retained production outputs only. No policy, threshold, outcome model or default was retuned in response to these results. Scope is the same 2019 retrospective development population and master seeds 1, 2 and 3 only.

## Result

Each seed accepted all 2,429 played game IDs with zero refusals, native failures or trace failures. Every frozen criterion passed independently for each seed; a pass on one seed did not offset another. The table reports simulated-minus-actual deviations. Aggregate usage tolerances were ±5%; role and per-player bands are the exact preregistered integer checks, not rounded approximations.

| Master seed | Batter usage band / appearance coverage | Pitcher usage band / appearance coverage | Starter role usage / coverage | Relief role usage / coverage | Starting opportunities | PA aggregate | Total pitching outs | Starter outs | Relief outs |
| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| 1 | 455/455; 455/455 | 564/565; 565/565 | 240/240; 240/240 | 331/331; 331/331 | 250/250 within band | +4,214 (+2.2593%) | +307 (+0.2357%) | −1,045 (−1.3846%) | +1,352 (+2.4673%) |
| 2 | 455/455; 455/455 | 563/565; 565/565 | 239/240; 240/240 | 331/331; 331/331 | 250/250 within band | +4,615 (+2.4743%) | +627 (+0.4813%) | −1,335 (−1.7688%) | +1,962 (+3.5805%) |
| 3 | 455/455; 455/455 | 564/565; 565/565 | 240/240; 240/240 | 331/331; 331/331 | 250/250 within band | +4,362 (+2.3387%) | +340 (+0.2610%) | −1,533 (−2.0312%) | +1,873 (+3.4181%) |

For the usage/coverage columns, the first fraction is within-band entries / compared entries; the second is real accounting appearances / eligible entries. The independent reconciler checked 2,429/2,429 game boxes and exact transition-derived pitcher outs in each seed, with 30 teams and 2,179 / 2,186 / 2,182 simulated player-team-role rows.

The frozen actual totals in every row are 186,517 PA, 130,270 pitcher outs, 75,473 starter outs and 54,797 relief outs. Pitching workload is integer outs. Seed 2's starter usage band is 239/240 (99.58%), and its total pitcher usage band is 563/565 (99.65%); both are reported as observed and pass the fixed 80% floor. Starter opportunity deviations never exceed 2 starts in absolute value (maximum 2, 1 and 1 by seed).

The frozen, overlapping target populations are 455 batter-team entries with at least 100 actual PA, 565 pitcher-team entries with at least 60 actual outs, 240 starter-role entries, 331 relief-role entries and 250 club entries with at least five actual starts. All qualifying batter and pitcher entries had a real accounting appearance; role appearance coverage was complete in both pitching roles in all three seeds. The full 455/565/240/331/250 denominators were retained rather than inferred from observed simulation coverage.

### Playing-time distribution and data visibility

Across 30 clubs, actual top-five starters accounted for 3,680 of 4,858 actual starts and the other supported starters for 1,178. V2 simulated 3,674 / 3,673 / 3,676 top-five starts and 1,184 / 1,185 / 1,182 other starts for seeds 1 / 2 / 3. Per-club top-five differences were at most 0–2 starts below actual; the complementary group was at most 0–2 above actual. This is a descriptive comparison, not a quota or a claim that player histories/opportunities remain paired after personnel diverge.

The reports also retain nonqualifiers and visibility differences. Each seed has 1,994 actual player-team-role rows; simulated rows are 2,179 / 2,186 / 2,182. Actual-only rows number 111 / 114 / 105 and simulated-only rows 296 / 306 / 293. There are zero unknown source-usage rows and two known-zero observed pitching roles in each seed. These all-role counts are not substitutes for the fixed eligible-cohort coverage checks and should not be read as missing or extra roster transactions.

### Relief selection and role errors

Known-target applied relief selections, class outcomes, negative-debt selections and rest exclusions are retained below. “Negative known debt” means the selected pitcher's accepted relief outs already exceeded that pitcher's prorated target at the actual decision; the score's class term and legal-candidate filters still apply. It is an observed policy tradeoff, not evidence of measured fatigue. Rest-exclusion counts are candidate-filter events and may count the same pitcher across multiple decisions. There were no exhaustion events in any seed.

| Seed | Known-target applied entries | Selected with negative known debt | Class match / mismatch | Relief rest exclusions | Exhaustions |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 19,647 | 5,252 (26.7%) | 14,259 / 5,388 (27.4% mismatch) | 12,212 | 0 |
| 2 | 19,831 | 7,218 (36.4%) | 14,650 / 5,181 (26.1% mismatch) | 12,179 | 0 |
| 3 | 19,763 | 5,955 (30.1%) | 14,347 / 5,416 (27.4% mismatch) | 12,312 | 0 |

The independent trace evaluator retained exact negative-debt witnesses: seed 1 `LAN201904030`, event/sequence 85, `stewb001`; seed 2 `NYA201904120`, event/sequence 73, `frarc001`; seed 3 `MIA201904010`, event/sequence 159, `chenw001`. Direct reads of the receipt-pinned traces also confirm a class-mismatch applied-entry witness per seed: seed 1 `SLN201905060`, event/sequence 36, `parkb001`; seed 2 the same game, event/sequence 42, `gantj002`; seed 3 the same game, event/sequence 69, `piven001`. Each is a `hook` decision. These are examples of the preregistered continuous-debt/class-preference tradeoff, not exceptions to the usage bands. Exhaustion has no witness because its count is zero.

Largest absolute total-pitcher-out deviations are reported by opportunity band, including the nonqualifying 1–59-out band for context. The two qualifying bands (60–179 and 180+) remain within their frozen limits in every seed.

| Seed | 1–59 actual outs | 60–179 actual outs | 180+ actual outs |
| --- | --- | --- | --- |
| 1 | NYN `flexc001`: 41 → 75 (+34) | CIN `bauet001`: 169 → 208 (+39) | CHA `novai001`: 561 → 500 (−61) |
| 2 | BOS `chacj001`: 44 → 73 (+29) | LAN `hillr001`: 176 → 220 (+44) | BOS `porcr001`: 523 → 595 (+72) |
| 3 | NYN `flexc001`: 41 → 76 (+35) | SEA `swane001`: 174 → 204 (+30) | DET `norrd002`: 433 → 360 (−73) |

For the separate role cohorts, the largest starter-out deviations were CHA `novai001` 561 → 500 (−61), BOS `porcr001` 523 → 595 (+72), and DET `norrd002` 408 → 331 (−77) in seeds 1–3. Largest relief-out deviations were MIA `chenw001` 205 → 237 (+32), TBA `beekj001` 276 → 297 (+21), and NYA `cortn001` 193 → 213 (+20). Each is within the preregistered role band. Exact per-entry targets, starts, appearances, start/relief/total outs, band status, cast/candidate exposure, rest exclusions, and known/unknown/prior flags remain in the per-seed reports and details below.

## Verification lineage and retained evidence

The production seasons used the v2 binary with SHA-256 `93da407a0f7cc68a67d5bf061361bf1cc07eec83b85acd6a0721edad8eda162d`, bound in production state to the original full-verification report. A later fresh full-verification build produced a different candidate binary (`4191c502a0dddc0c70894501ffc3cfc7e1531a7396658a4def6a8e5d593756f8`) because it was built in a separate isolated target. The first evaluator attempt correctly refused to equate those binaries. No seasons were rerun and no identity check was bypassed: the corrected lineage checker binds the production binary to its own green report, then compares both green reports' source pins (allowing only the updated `docs/PROGRESS.md` pin), external-input hashes, toolchain, unchanged original engine reference binary and all 16 compatibility-control command/output fingerprints. Four regression tests cover this lineage rule. The corrected external evaluator suite passes all 14 tests and `py_compile`.

The production and current full-verification candidates, the mechanism candidate and the protected original reference binary are distinct artifacts. The original reference remains unchanged at SHA-256 `f226b27b6562aafa9df5910a4c56ce0b58a8e2080d7b06d752279e63cabe04d8`. Final full verification passed all nine checks, all 16 frozen v1 byte controls and 26 worktree pins before this results/ledger documentation update; it is engineering verification, not a usage or gate signoff.

## Reproduction commands

- Production run: `python3 -B -W error /tmp/bbsim-pitcher-v2-evaluation-20261019/run_three_seeds_v1.py` (`proc_e24e`). Its `state.json` records each exact CLI argument vector, output root, seeds, input hashes and per-seed exit/counts.
- Fresh final full verification: `python3 -B /tmp/bbsim-pitcher-v2-integration/run-full-current-role-key-final-refresh-v1.py` (`proc_e51d`).
- Independent evaluation, from retained production outputs only: `python3 -B -W error /tmp/bbsim-pitcher-v2-evaluation-20261019/verify_v2_evaluation_v1.py` (`proc_3204`). Output root is the exclusive `verification-role-key-fix-v1/` directory.
- Corrected evaluator regressions: `python3 -B -m unittest discover -s /tmp/bbsim-pitcher-v2-evaluation-20261019 -p 'test_*_v1.py' -v` (14 passed); `python3 -B -m py_compile /tmp/bbsim-pitcher-v2-evaluation-20261019/verify_v2_evaluation_v1.py` (passed).

| Evidence | Location | SHA-256 |
| --- | --- | --- |
| Frozen target population | `/tmp/bbsim-season-usage-targets-v1/first/targets.json` | `f45cd37997e1027ac17e4e4d752c4aa44e2be96f871f510408f72e432108972a` |
| Archived frozen evaluation population | `.../verification-role-key-fix-v1/frozen-population.json` | `52b520ffa305351629f9f331260632f62022f03409b503523e1fc4f5cabf0d77` |
| Overall independent evaluation report (`all_seeds_passed: true`) | `.../verification-role-key-fix-v1/report.json` | `a1c8f3de2d3c1498b316282fe617bf83ad28cdade6b547a106cb2c0a90e9d372` |
| Seed 1 report / details | `.../verification-role-key-fix-v1/seed-1/{report,details}.json` | `8974781895dc6a4b365f6fd1012c2f407c9b55e1eeb26917ada64807518d6743` / `839b3d0db1e4ebd6d765501485c318d707e0f17979730b29d2dcc880729579ed` |
| Seed 2 report / details | `.../verification-role-key-fix-v1/seed-2/{report,details}.json` | `538e5eceb033243ae1eb9c91ad3867d6951cffa44729bf618afd8103acfc6835` / `f900ee892fa0ce6260ea9703fa067875014ce8d41b5dcb0918d0e12690aa515a` |
| Seed 3 report / details | `.../verification-role-key-fix-v1/seed-3/{report,details}.json` | `52bc6878152b667d9e727973d0cacf48fbd6549fc1ffccea9e97b9ae93e64f6d` / `1b62c87f7d65f0e151262cdb3bc994fda1114c723f2928f72cdcb9ee9c9a59ed` |
| Production run state | `/tmp/bbsim-pitcher-v2-evaluation-20261019/production-seed-runs-pythonpath-fix-v1/state.json` | `3aa087ed390b2e82437df1d9df7375bd7214aaca439587c85862f202ad500ed5` |
| Production-time full-verification report (binds binary `93da…`) | `/tmp/bbsim-pitcher-v2-integration/full-current-20261019-doc-refresh/report.json` | `9a07476be63f71cb812128c65d93975e0375666451e38995b3c7bd72d8997fa0` |
| Final fresh full-verification report | `/tmp/bbsim-pitcher-v2-integration/full-current-20261019-role-key-final-refresh/report.json` | `d02820c663d4ca03b047301ee17ffce556a96de6127bbc85ba477491f50bbc1e` |
| Mechanism gate PASS report | `/tmp/bbsim-pitcher-v2-integration/mechanism-v2-transition-outs-role-loop-20261019/report.json` | `74ded9f924e1b4c996cd26fe2a6b882ac1d982ad5da37f6468ebef7041318693` |

The overall evaluation report seals each per-seed report/details hash and the frozen-population digest. Full verifier lineage, prior production/evaluator reports and original traces/receipts remain in their separate retained `/tmp` roots. Earlier evaluator failures (`game_index` NameError, starter-role `start`/`starter` key mismatch, and the correctly blocking initial binary-lineage mismatch) are preserved as evaluator-failure evidence, not season outcomes; they did not change the frozen metrics or policy.

## Limits and gate impact

- This is a three-seed retrospective development result using season-end historical targets. It is not held-out predictive validation, a fresh test-season benchmark, uncertainty intervals, or the 100+ season S-03 ensemble. Pairing seeds does not hold actor histories or opportunities fixed after selections diverge.
- Existing pools, observed positions, and usage-ledger appearances/workload are not verified transaction histories, dated roster eligibility, injury boundaries, availability, or measured fatigue. The counts must not be read as claims about those facts.
- This result passes only the fixed v2 development usage evaluation. It does not sign off G1–G5, establish complete historical eligibility/defensive replacement handling, or authorize changing the default. Keep v1 NO-GO/default and v2 opt-in. No commit, push, retune, additional season run or public release is part of this result.
