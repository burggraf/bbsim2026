# Player-specific Seattle–Oakland probability pilot — preregistration

## Authorized bounded scope

P-01-player-preview under D-17: maintainer approves the recommended player-specific
probability milestone and directs stopping after implementation so they can test.
Base `7488d53`, same audited `OAK201903200` starting personnel, closed DH roster,
source positional actors and existing viewer. This is selected-season hindsight
reconstruction using terminal-event participant **proxies**, not forecasting,
official batter PA/pitcher BF, calibrated abilities or production P-01/G1/G2.
No new transfer, dependencies, delegation, worktrees, manager/substitution/fatigue,
handedness/park/fielding-skill model, other matchups, desktop migration or push.

Exactly 12 owned paths: this document; `docs/PROGRESS.md`; `preview/README.md`;
new `preview/player_ratings.py`, `tests/test_player_ratings.py`,
`engine/examples/support/player_ratings.rs`, `engine/tests/player_probability_preview.rs`;
`engine/examples/support/joint_contact.rs`, `engine/examples/generated_game.rs`,
`preview/build_viewer.py`, `preview/serve.py`, `tests/test_live_preview.py`.
Other177 existing tracked files (including pure engine, canonical codec/accounting,
lineup loader/policy, previous audits and tests) stay byte-identical to HEAD.
Freeze source/input/contract evidence outside Git in `/tmp/bbsim-player-pilot/`.

## Inputs and narrow export

Reuse the accepted opportunity report, not a new query or a second PA classifier:
`/tmp/bbsim-player-opportunities/report-first.json`, SHA256
`382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e`.
Reuse lineup-readiness report SHA256
`5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b`,
accepted pooled/joint count artifacts and source snapshot/supplement pins from
[real-lineup contract](real-lineup-preview.md). Preserve Retrosheet attribution.
No new eligibility cutoff: export the 18 starting batter profiles and two starting
pitcher profiles, role-keyed; retain each nine-category count including zeros.
Missing profiles or malformed inputs fail; never borrow an identity or fall back
on bad data. Input hashes, cohort/provenance and selected-profile conservation
are checked before publication and reread immediately before atomic exclusive
publication. Existing outputs and outputs inside the repository are refused.

UTF-8 TSV `key\tvalue`, format `player-proxy-counts-v1`: fixed version/provenance,
prior strengths, nine pooled counts, and exactly20 `batter:<id>`/`pitcher:<id>`
rows with canonical decimal comma-separated nine-count vectors. No floats from
Python drive native sampling. Native Rust validates controls, unknown/duplicate/
missing fields, numbers, count bounds, fixed provenance and exact role/ID set
against the loaded lineup. Self-reported hashes alone do not authenticate edited
rows; pinned exporter plus source reconciliation and frozen artifact bind this
local experiment. All real rows/reports/exports remain outside Git; tests are
synthetic. Roles use literal terminal-event identities, not unavailable mid-PA
credit-responsibility reassignment or inferred opportunities.

## Formula fixed before fitting/measurement

Order: K, non-IBB BB, HBP, 1B, 2B, 3B, HR, ROE, contact out. Use the unchanged
pooled Jeffreys distribution `ell[k]=(league[k]+0.5)/(Nleague+4.5)`.
For **each** role set `kappa=200` included participant opportunities, chosen as a
transparent provisional prior strength before seeing selected-player estimates,
not fitted or asserted optimal. `alpha[k]=200*ell[k]`.
Posterior component mean `mu[k]=(n[k]+200*ell[k])/(N+200)`;
component SD `sqrt(mu[k]*(1-mu[k])/(N+200+1))`; data weight `N/(N+200)`.
Zero exposure returns the pooled mean, not empirical zeros or an arbitrary floor.
These SDs describe the assumed Dirichlet components, **not** validated uncertainty
in true ability, a 95% interval, or propagated matchup uncertainty.

For the current batter and opposing current starter, generalized-log5 baseline:
`w[k]=mu_b[k]*mu_p[k]/ell[k]`; `p[k]=w[k]/sum(w)` in left-to-right binary64
operations. Every one of the nine modeled legal categories stays finite/positive,
normalized within32 binary64 epsilons; IBB/interference/runner-only mechanisms
remain excluded rather than being invented as unconditional categories.
Independent player marginals retain opponent/park/defense confounding. No claim
of best method, held-out accuracy, or hierarchical opponent-adjusted estimation.
A future production comparison still requires preregistered held-out alternatives.

Keep all five pooled contact-proxy conditionals unchanged. Shared joint sampler
uses the first eight cumulative matchup probabilities then
`start+(1-start)*prefix(q)` for the first four tail edges, final edge exactly1,
strict `<` (edge equality belongs to next interval), multiply then add/no FMA.
Exactly one SplitMix64 word per PA, no retry/rejection/second fielding draw.
Only matchup probabilities change; engine/event physics, positional actors,
accounting, source IDs, DH and closed-roster limits remain. Three existing modes
(default, fictional joint, real pooled) and their unconfigured pages remain exact.

## Observable acceptance fixed before tests/fitting

- Test-first export, posterior/log5 and native-loader failures; distinguish missing
  interfaces/harness errors from genuine semantic reds. Literal synthetic counts
  give independent means/SDs/weights and log5 weights; test zero/sparse/full counts,
  bool/int rejection, overflow, input ordering, unknown/missing/duplicate roles,
  provenance, final drift, no overwrite/partial publication and no silent fallback.
- Independent posterior/CDF oracle for all18 actual matchups; neutral controls
  (both zero -> pooled, one zero -> other posterior), positivity/normalization,
  stronger synthetic hitter HR and stronger pitcher K direction, all13 edges
  and adjacent representable draws, exact one-word consumption. Seeded property
  coverage keeps base/out continuity, DH exclusion, positional actors, available
  stat conservation, codec round trips and terminal games.
- Fixed100000 draws per actual matchup at seed19; compare all13 counts to actual
  CDF widths with `abs(observed-M*p)<=6*sqrt(M*p*(1-p))+1`. Diagnostic sampler
  check, not historical calibration. Fixed seeds0..1999 paired/repeated rated
  and pooled games; report outcome/run/length changes descriptively, not accuracy.
  No seed/count/band/threshold/prior/filter retuning after measurement. Component
  sensitivity at kappa100/200/400 is descriptive only; deployment remains200.
- Repeat real export byte-for-byte, reconcile all20 role vectors to the pinned
  report and independent retained native aggregation (fresh upstream reconciliation),
  verify all input/non-owned hashes. No repeated fitting to obtain passing scores.
- CLI `--ratings` requires lineup plus joint counts. Server startup-only ratings
  path enables `model=rated`, keeps `model=real` pooled comparison and no-query
  default. Queries accept only configured enums/one seed, never paths; retain
  Host421/route404/no-store/CSP/timeout and visible500/no fallback on corrupt input.
- Native preamble exposes20 role posterior rows and18 matchup vectors, counts,
  strengths/weights/SDs/provenance and limitations. Viewer renders an escaped
  rating/matchup panel from native data, never a second probability implementation
  or RNG. Reject incomplete/inconsistent metadata and unrecognized rated records.
- Fresh full offline locked Rust build/test/fmt/all-target Clippy `-D warnings`,
  `python3 -Werror -m unittest discover -s tests -v`, compilation, existing audits,
  native controls and whole-record/page/codec compatibility. Freeze executable
  sources during full checks; preserve complete raw stdout/stderr and failures.
- Fixed browser seeds1/2/3/99: rated/pool selector, Generate/New/max-u64 wrap,
  Next/Previous/Restart/autoplay/Pause/terminal, rating disclosures and full native
  data. Compare every rated CALL/RECAP/body/score/base/final-stat field with an
  independent CLI. Owned browser/server closed/stopped; socket refusal witnessed.
- Complete scoped diff review and bounded parent/author acceptance, fresh prose
  links excluding fences, working/index/frozen/tested/non-owned equality; local
  accepted/precommit/committed exact diff checkpoint and clean main. No independent
  reviewer/gate/rights signoff implied. Document a one-command local testing launch,
  stop after this milestone and wait for maintainer feedback; no further scope.

## Pre-implementation caller-driven scope amendment

The lineup preamble writer embeds a pooled/no-player-fitting limitation. Rated
output must not repeat that false statement or omit/replace its factual source
metadata. Before selected-player estimation, add
`engine/examples/support/real_lineup.rs` solely for a backward-compatible
mode-aware limitations writer. Scope is now13 owned paths and176 frozen
non-owned tracked files. The original12-path contract remains this document's
byte prefix; formula, prior strengths, input selection and acceptance stay fixed.

## Implementation and fresh bounded evidence

Implemented native count-bound posterior/log5 sampling and an opt-in `--ratings`
mode. Existing three modes remain unchanged; the configured viewer offers the
player pilot beside the real pooled comparison. Native provenance supplies20
role count/mean/SD/weight rows and18 matchup vectors; the viewer only validates
shape/conservation and renders those numbers. Individual category counts are
included, not just totals. Rated preambles distinguish the league prior/reference
pooled CDF from actual dynamic matchup probabilities and do not repeat the old
no-player-fitting limitation. The pure engine/accounting/codec are untouched.

External exports repeat exactly:1605 bytes, SHA256
`c84eadf566f5ee88a2e2e1f5cb8a14fb46fb110c8e67ce11fc226cdbf0a792df`.
Fresh read-only native aggregation repeats all1,821 role profiles/12 bands and
matches the original audit; all20 selected nine-count vectors exactly reconcile
to native aggregation, the pinned report and export. No IDs/opportunities borrowed.
All seven source inputs and176 non-owned tracked files remain unchanged.

`/tmp/bbsim-player-pilot/full-checks.sh` on `proc_fea8` exit0/391s:
**601 Rust/212 Python** tests, offline locked build/test, fmt, all-target Clippy
`-D warnings`, complete Werror discovery/compilation, established source/fixture/
27 shared-upstream ER/54 external-box checks, native1,350 controls, repeated
source controls, default/joint/real ensembles and sampler tests. Complete raw
stdout1,008 lines/63,039 bytes and stderr394 lines/50,784 bytes were consumed;
expected negative-generation/HTTP500 controls only, no unexpected warning or
traceback. Ten executable/test files frozen through verification.

Independent literal-count binary64 oracle reproduces all20 means/SDs/weights,
all18 matchup distributions and **234 bit-exact CDF edges**. All234 category counts
from18×100,000 seed19 draws pass the unchanged six-sigma-plus-one diagnostic.
Fixed2,000 repeated/paired actual-lineup games terminate, preserve continuity,
codec/DH/available statistics. Descriptive rated versus pooled results:

| Diagnostic | Rated | Real pooled |
| --- | ---: | ---: |
| Mean combined runs/game | 7.855 | 8.291 |
| Ensemble runs SD | 4.1953 | 4.3692 |
| Mean PA events/game | 79.2685 | 80.9405 |
| Maximum PA events | 188 | 167 |
| Generated K fraction | 0.17976 | 0.23108 |
| Generated HR fraction | 0.03969 | 0.03748 |

These are descriptive differences, **not accuracy**, predictions or a successful
historical fit. Kappa100/400 sensitivity's largest absolute category-probability
shifts from200 are0.01846/0.02581; deployment remains200, no selection/retuning.
Ichiro's six included participant events receive only2.9% data weight; pitchers'
754/865 included events receive79.0%/81.2%. These are proxy exposures, not official
PA/BF. Full literal vectors/means/SDs/sensitivity/outcome totals are retained only
outside Git in `evaluation.json`/`diagnostics.stdout`/`ensemble.csv`.

Fresh base/current12 whole default/joint/real records,12 static pages and24
applicable old configured/unconfigured live pages are byte-exact. Actual rated
seeds1/2/3/99 repeat. Owned native Chrome captured **304 frames** (70/87/76/71):
every CALL/RECAP/body/score/status/base/final-stat field, preamble/team/20 roster,
20 count/mean/SD/weight and18 matchup rows matches independent CLI records.
Selector comparison/Generate/New/exact max-u64 wrap/reset/Next/Previous/Restart/
autoplay/Pause/manual/visibility/terminal stop, panel toggles and nested log scroll
were observed. Three verified screenshots retained. Browser code sandbox failed
before dispatch; native commands were used, no scripted activation/CDP workaround.
An unsubmitted max-seed edit did not change New game's loaded-seed increment;
Generate then New correctly wrapped max-u64 to0. Server retained ten GET200 and
ten ten-seconds-later read/socket timeout diagnostics; no packet-level cause claim.
Owned browser closed/server SIGTERM15;127.0.0.1:8765 refused, all frozen bytes
unchanged. Full stage/acceptance/checkpoint status is in the live ledger.

## Maintainer testing and stop boundary

With the currently verified local binary and external caches, run this one command
from the repository (Ctrl-C stops it):

```sh
python3 -Werror -m preview.serve --baseline /tmp/bbsim-preview-2019-counts.csv --contact-counts /tmp/bbsim-joint-preview/counts.csv --lineup /tmp/bbsim-real-lineup/lineup-first.tsv --ratings /tmp/bbsim-player-pilot/ratings-first.tsv --port 8765
```

Open `http://127.0.0.1:8765/?model=rated&seed=1`; compare the two Seattle–Oakland
options at the same seed. Expand/collapse the probability and component-detail
panels, try game controls and inspect source/native facts. The full rebuild/export
recipe is in [preview README](../../preview/README.md); `/tmp` evidence/caches are
local, not bundled or guaranteed durable. After the local checkpoint, **stop for
maintainer testing**. No manager/roster/desktop/season/production-model work begins
without feedback; G1/G2 and redistribution rights remain open.
