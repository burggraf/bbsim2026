# Pitcher usage v2 — native workload value interface

This appendix fixes raw cast-value spellings before native integration behavior.
It supplements, and does not revise, the [preregistered policy](pitcher-usage-policy-v2.md).
Live status and evidence are in [PROGRESS](../PROGRESS.md#resume-here).

## Cast fields

The eventual `season-pool-pitching-cast-v2` retains the existing cast, ability,
empirical workload/sample and marked-prior bindings. Its additional fields are:

| Key | Exact value | Meaning |
| --- | --- | --- |
| `club_clock:Away` | `c,N` | Away accepted games and authenticated scheduled count |
| `club_clock:Home` | `c,N` | Home accepted games and authenticated scheduled count |
| `relief_usage:<player_id>` | `U,T` | Club relief outs used and historical relief-out target |

Each cast actor has exactly one `relief_usage` row, including the selected starter;
ranking still considers only supported legal unused nonstarters. Unknown `T` is
`?`, never an omitted value or zero. `U` cannot be unknown. Counts are canonical
ASCII unsigned decimal u32 values (no whitespace, sign or leading zeros).
Exactly two comma-separated cells are required. Clocks require `0 <= c < N <= 162`,
where the 2019 scheduled count is 161 for CHA and DET and 162 for every other
supported club. Examples: clocks `0,162`, `161,162`, `0,161`; usage `0,?`,
`17,0`, `120,144`.

V2 must not accept `relief_pace` as a replacement for these counts.

## Complete v2 personnel schemas

These markers are distinct from the historical, assumed, and managed-v1 inputs:

| File | Required `format` | `season_policy` | Retained policy field |
|---|---|---|---|
| Lineup | `season-pool-lineup-v2` | `season-pool-pitcher-usage-v2` | — |
| Cast | `season-pool-pitching-cast-v2` | `season-pool-pitcher-usage-v2` | `policy=empirical-pa-closed-cast-v2` |
| Bench | `season-pool-bench-v2` | `season-pool-pitcher-usage-v2` | `policy=late-close-expected-bases-ph-v1` |

The lineup and bench retain their complete managed-v1 row schemas, changing only
`format` and `season_policy`. The bench retains its per-actor `pace` preference;
that is not pitcher `relief_pace`. The cast retains its generic managed-v1
identity, ability, empirical workload/sample, marked-prior, and `starter_scale`
fields. It replaces every `relief_pace:<id>` with exactly one
`relief_usage:<id>` for every `cast:<side>:<rank>:<id>` actor, and adds exactly one
`club_clock:Away` and `club_clock:Home`. Unknown, missing, duplicate, or extra
fields and mismatched usage/cast actor sets are errors.

All three files must be present and agree on v2 mode, `season_policy`, and
`game_id`. Their source pins must match the existing audited inputs: lineup
`audit_sha256` equals cast `lineup_report_sha256`; all three carry the same pinned
snapshot and supplement hashes; cast and bench carry the same pinned opportunity
report hash; attribution is unchanged. The two clock `N` values are checked against
the corresponding lineup team IDs. The cast's two side groups and selected starters
must agree with that lineup, and bench actors may not overlap lineup or cast
identities. Mixed v1/v2/historical sets fail closed.

The native lineup/cast/bench parsers and Python season-runner validator check this
v2 schema. The text-game command validates and then deliberately stops before
simulation: v2 selector, hook, probe, and restoration integration is not part of
this preparation. There is no new CLI option or default change.

## Native exact ranking interface

`Clock::read` and `ReliefUsage::read` validate those value cells. `relief_rank`
accepts a validated clock/usage plus raw u32 empirical/prior `pa_sum,outings`,
u32 positive inning and signed score gap whose absolute value fits u32.
Zero outings are invalid. No profile or eligibility is inferred.

The returned `Rank` retains unreduced exact integer ratios: unclipped debt,
clipped debt and final score, plus long-class, match and explicit target-known flags. Denominators are
positive; unreduced ratios are not distinct scores. `Rank::compare` compares score,
then unclipped debt, exactly. It deliberately excludes ID: the eventual shared
selector resolves equal ranks by ascending stable player ID after legality.

All intermediates and cross-products use i128. With u32 raw counts and N<=162,
debt numerator magnitude is below 2^42, denominator below 2^40, and score/debt
comparison cross-products below 2^85, safely within signed i128. This arithmetic
bound is tested with maximum input counts; it is not permission to accept wider
or malformed wire values. Unknown targets use debt/clipped `0/1`; the same
situation bonus still applies. Known zero targets use the registered formula.

## Remaining integration boundary

This is preparation, not a completed native manager. The Cast/Lineup/Bench
readers validate v2 personnel schemas, the Python season runner binds the trio for
accounting, and the text-game CLI refuses before simulation; no v2 selector is
wired into game execution. Whole `pitcher-entry-decisions-v2` trace acceptance and
live hook, PH-probe, and restoration integration remain to be completed. No
mechanism gate, season evaluation, or pitching-accuracy improvement is claimed.
