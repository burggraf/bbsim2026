# Experimental generated-game preview v1 — preregistration

D-12 approves this internal preview before G1; production gates remain pending.
This contract is recorded **before aggregation/estimation or generator implementation**.
No calibration or predictive claim, player ratings, new season download or UI scaffold.

## Inputs and model

Use the existing read-only F-03 2019 regular-season snapshot. Aggregate actual
PA rows (batting.plate_appearances=1 joined one-to-one to events), with explicit
reconciliation and rejection of unknown PA result labels. No missing value is
coalesced to zero. Exclude deliberate IBB and interference awards from this
preview's sampling denominator and report them separately.

Nine ordered categories: K, ordinary BB, HBP, 1B, 2B, 3B, HR, ROE, other contact.
Ground-rule doubles pool with2B; inside-park HR pools withHR; FC/SH/SF/in-play
outs pool as other contact. That last category is rendered as an ordinary
one-out ground/contact play with a declared shortstop assist/first-baseman PO,
**not** as faithful trajectory/FC/sacrifice/double-play management. Fly/line/pop
subtypes are not estimated in this first preview.
K is rendered caught and declares [ordinary caught-K facts](../engine/ordinary-caught-k.md) for its existing fictional slot0 catcher/PO; only this selected outcome supplies the facts. This adds event-local evidence, not a new roll, outcome, stat counter or whole-box certificate. BB/HBP alone supplies [exclusive protected-award RBI cause facts](../engine/protected-award-rbi.md) for the existing force-chain policy; independent RBI0/1 evidence does not change credits, physical outcomes or sampling. HR is out-of-park. All such mechanism coarsenings stay
explicit; absent mechanisms are unimplemented, not structurally impossible.

Probability for category i: `(count_i + 0.5)/(N + 9*0.5)`, a symmetric Jeffreys
Dirichlet posterior predictive baseline, no arbitrary epsilon. This is a pooled
league-only first baseline for fictional equal teams, not a fitted player model,
chosen-best matchup method or calibrated run environment. Preserve the nine
counts, exclusions, source/query/model versions and denominator outside Git.

Version a deterministic SplitMix64 RNG and fixed cumulative sampling order;
convert the top53 bits to a uniform [0,1) variate. No network/clock RNG or
platform hash-map ordering. Seed+counts+rules/model/policy version reproduce logs.

## Coarse engine policy

Use named fictional DH lineups and pitchers. No substitutions/fatigue, IBB,
steals, WP/PB/balk, sacrifices, DP or manager tactics yet. BB/HBP force only
required runners; hits advance runners station-to-station by hit value; ROE
uses a coarse one-base advancement with a named defender error. Contact outs
hold runners, never independently convert a decided hit to an out. Fielding
opportunity/arm/range and realistic advancement are not estimated.

Ordinary winning hits must respect the existing canonical scoring/ending rules.
Because the current codec lacks winning-run order facts, use a conservative hit
value valid for **every supplied scorer**, not an unrecorded presumed winner.
Score only the needed leading runners (third toward first), hold other runners
at distinct trailing bases and shorten the batter's credited value as needed;
HR keeps its exception. This is an explicitly coarse terminal policy, not a
second fielding roll or a declaration that richer legal histories are impossible.
Initial mechanical checks reproduced an unsupported winning-order frame and
seed335 before this correction; original mask/termination assertions retained,
with literal loaded/two-scorer walk-off controls added. Stop on the engine's
terminal state. A resource ceiling aborts with an explicit error, never invents an out/winner.

## Checks fixed before seeing results

- Parser rejects duplicates/missing categories/invalid counts/versions; counts
  reconcile selected PA, classified included PA and explicit exclusions.
- Probabilities finite/positive, sum1 (1e-12 tolerance); all-zero input rejected.
- Fixed RNG vectors and same-seed byte-identical logs; different seeds vary.
- Per-category event tests across eight masks/three outs/both halves and ordinary
  versus winning contexts; supplied PA/AB/H/K/BB/HBP/run/out/fielding credits agree.
- At least2,000 seeds: games terminate within an explicit10,000-event resource
  budget, no silent retries, legal transitions, continuity, codec round trips,
  named-run/line/charge totals, PA/BF, H partitions and physical outs conserve.
-100,000 standalone draws: each category frequency within six binomial standard
  deviations plus1/N of its fixed probability (mechanical sampler diagnostic,
  not calibration or a threshold tuned after results).
- Terminal step/autoplay CLI prints seed/version, varied outcomes and partial
  statistics; unknown ER stays `?`, not zero. Existing scripted demo remains.

Full-game scoring distributions and contact coarsening bias are reported as
exploratory diagnostics, not fitted/held-out accuracy. P-01/P-02/G1 acceptance
and the approved complete-box matrix are unchanged.
