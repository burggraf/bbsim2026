# 2019 player-opportunity source support — preregistration

P-01-preview support audit only, after accepted joint sampler `7c20947`.
Internal offline facts, not ratings, calibration, production P-01, or a gate
waiver. [Product contract](../../PLAN.md), [execution dependencies](../EXECUTION.md)
and [live status](../PROGRESS.md) still apply. Read the historical
[matchup research note](matchup-probabilities.md) as planning, not live status.

## Scope fixed before measurements

Exactly four owned paths: this document, `docs/PROGRESS.md`, new
`audits/player_opportunities.py` and new `tests/test_player_opportunities.py`.
All other tracked bytes stay at `7c20947`, including canonical Rust, samplers,
existing exports/validators/count recipes, CLI, UI and dependencies. No network,
new data extraction, player fitting, lineup/roster work or preview restart.

Use the already accepted real-event outcome-v2 CSV and F-03 manifest, together
with the accepted joint-count JSON as a cohort certificate. Do not introduce a
second importer or change the outcome export. Pin these SHA256 inputs:

- Outcome CSV: `ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a`.
- F-03 manifest: `443e6a07341e381cd2b171599ac09658348e1462351ed41809fb70b4aaf60f9f`.
- Joint count JSON: `1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca`.
- Outcome SQL: `2422c4acb74702892607852b58b587c0efa59174676e8e25b5a20eaa8419e677`.
- Catalog/source/F-03 snapshot remain the accepted pilot's values, not a new
  source version. Retain exact Retrosheet attribution and input/recipe digests.

## Observable population and attribution boundary

`audits/2019-outcomes.sql` emits 192025 **real** source events, not no-plays:
186517 terminal PA rows and5508 non-PA rows. Reuse the complete existing
`outcome_inventory.inventory` validator, CSV field/header rules and manifest
reader; missing identities, grain/fanout, inconsistent PA/state/credits or
malformed data are errors, never silently dropped or replaced with zero.
Reuse `joint_contact.GROUPS`/`CATEGORIES` for the original nine coarsenings.

For each role separately, group by its literal exported `batter_id` or
`pitcher_id`. Population is every distinct role ID in the real-event export,
including IDs with zero **included** PA exposure. Retain all non-PA and excluded
PA rows in event/exclusion counters; do not increment included category counts
for them. Never treat an unfinished PA on an inning-ending runner play as a
terminal PA. Profiles are source-role observations, not all eligible MLB players.

**These are terminal-event participant counts, not official batting/pitching
credits.** Pinned `stg_events.sql` explicitly describes separate
`strikeout_responsible_batter_id` and `walk_responsible_pitcher_id` for mid-PA
substitutions. The accepted outcome CSV does not project them. Do not silently
substitute those identities, claim official PA/BF/box reconciliation, or call
these unexamined responsibility fields absent from the source. Hands, parks,
team membership, availability, opponent effects and credit reassignment are
not evaluated here. Non-PA current batter/pitcher occurrences are reported only
as source events, not invented opportunities or statistics.

## Fixed output and independent conservation

Format `2019-player-opportunity-support-v1`, deterministic JSON with sorted
keys and role profiles sorted by opaque player ID. Each profile records:

- `player_id`, `events`, distinct `games`, `selected_pa`, `non_pa`;
- all nine `counts` keys, including literal zero observed counts;
- `included_pa`, both `excluded_ibb`/`excluded_interference` counts;
- `zero_categories`, the number of the nine included categories with count0.

Counts are observed integers, **not empirical probabilities**, structural zeros,
posterior ratings or catch/fielding denominators. No smoothing or prior fit.
Per role, sum of profile events must be192025, selected PA186517, non-PA5508,
included PA185703, exclusions753/61 and nine counts exactly
`[42823,15142,1984,25947,8531,785,6776,1470,82245]`. Every profile must satisfy
`events=selected_pa+non_pa` and
`selected_pa=included_pa+excluded_ibb+excluded_interference`, with
`included_pa=sum(nine counts)`. Distinct games need not sum to league games.
The validator's independent outcome totals/games remain unchanged.

Report role summaries: number of observed IDs, number with zero included PA,
per-category number of profiles with zero observed counts, and exposure bands
based **only** on included PA: `0`, `1-9`, `10-49`, `50-199`, `200-499`, `500+`.
For every band, report players and included PA; sum players/exposure across bands
back to role totals. These fixed descriptive bands are not rating eligibility
filters or the research paper's opportunity threshold. Do not retune them after
viewing support. Same-season records are hindsight; no predictive claim.

CLI accepts outcome CSV, manifest, accepted count certificate, and a new external
output path. Validate duplicate/malformed JSON keys with the existing strict
object helper; pin cohort/certificate hashes, logical vector and metadata;
require the full source totals and original exclusions. Reread every input hash
before exclusive publication. Refuse overwrite and do not publish on invalid or
changed inputs. Synthetic `inventory` unit tests may use tiny valid cohorts, but
the real CLI must reject them rather than label them the accepted pilot.

## Acceptance fixed before tests/code

Witness interface failures separately from genuine semantic reds. Independent
literal fixtures include mixed PA/non-PA/exclusions, players sharing IDs across
roles, zero-included-exposure players, zero observed categories, all exposure
band boundaries, input-order invariance and exact integer conservation. Preserve
all existing validation: duplicate/header/grain/identity/unknown-result/state
errors, bad provenance/certificate, partial cohort, changed input and existing
output must not yield a published pilot report. No dataset/count/filter retuning.

After focused reds/greens, independently aggregate the accepted local F-03
snapshot with native DuckDB `-readonly`, external access disabled, SELECT stdout.
Reconcile every role/profile/category/event/exclusion/non-PA/distinct-game count
and band summary, not just league totals. Native queries/artifacts remain
external; no filesystem COPY or new production SQL. Repeat exports/reports
byte-for-byte and prove input/database/source hashes unchanged. Run all applicable
Rust/Python/audit/native/preview checks (including Python `-Werror`) with tested
sources frozen, then inspect complete four-path stage, fresh prose links,
working/index/frozen and non-owned HEAD equality. Record acceptance only in
PROGRESS. No independent-review, accuracy, G1/G2 or distribution-rights signoff.

## Observed support from the fixed native/repeat check

`proc_29b4` executed the preregistered offline controls without retuning. All1821
role profiles (990 batter IDs,831 pitcher IDs), every profile's nine categories,
events, distinct games, selected/included/excluded PA and non-PA cells, zero
category counts and12 native bands matched independent DuckDB aggregation.
Both reports and native exports repeated byte-for-byte with empty native stderr;
source/database/input hashes stayed unchanged. Each role conserved192025 real
events,186517 selected PA,185703 included PA,5508 non-PA and753/61 exclusions.
There were no zero-included-exposure IDs in this observed population; the zero
band and zero-exposure behavior remain positively tested on literal fixtures.
This does not establish a complete player population or official credit lines.

| Included-PA band | Batter IDs | Batter included PA | Pitcher IDs | Pitcher included PA |
| --- | ---: | ---: | ---: | ---: |
| 0 | 0 | 0 | 0 | 0 |
| 1–9 | 261 | 771 | 64 | 363 |
| 10–49 | 162 | 4488 | 136 | 3861 |
| 50–199 | 208 | 22063 | 267 | 31324 |
| 200–499 | 225 | 77788 | 256 | 74937 |
| 500+ | 134 | 80593 | 108 | 75218 |

| Category | Batter profiles with zero observed counts | Pitcher profiles with zero observed counts |
| --- | ---: | ---: |
| K | 61 | 52 |
| BB (not IBB) | 303 | 49 |
| HBP | 527 | 248 |
| 1B | 234 | 34 |
| 2B | 370 | 90 |
| 3B | 652 | 434 |
| HR | 441 | 86 |
| ROE | 503 | 311 |
| Contact out | 95 | 6 |

Artifact `/tmp/bbsim-player-opportunities/report-first.json`:1078681 bytes,
SHA256 `382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e`.
Raw profiles/native queries/repeat evidence remain outside Git. Full project
verification and exact acceptance are recorded only in PROGRESS, not inferred
from native equality. Reproduce to a **new** external output path:

```sh
python3 -Werror -m audits.player_opportunities \
  /tmp/bbsim-joint-contact/context-first.csv /tmp/bbsim-outcomes-manifest.csv \
  /tmp/bbsim-joint-contact/counts-first.json /tmp/player-opportunity-report-new.json
python3 -Werror -m unittest tests.test_player_opportunities -v
```

The JSON records provenance and the participant-attribution limitation. No
probabilities, shrinkage strength, credit reassignment, player selection cutoff,
matchup fit or simulation change was selected from these measurements. Do not
use empirical zero counts as zero legal probabilities or describe these
hindsight source observations as predictive or historical box certificates.
