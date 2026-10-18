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
Exactly two comma-separated cells are required. Clocks require `0 <= c < N <=162`.
Examples: `0,162`, `161,162`, `17,60`, `0,0`, `17,?`.

V2 must not accept `relief_pace` as a replacement for these counts. The eventual
full cast reader must reject missing/duplicate/unknown fields and mismatched actor
sets; the small value parsers in this slice do **not** certify a complete cast.
There is no standalone context file, new CLI option or default change here.

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

This is preparation, not a completed native manager. The existing Cast, lineup,
bench, CLI and season runner do not consume these primitives yet. Before wiring
live selection, freeze/test the **full** personnel and `pitcher-entry-decisions-v2`
entry-trace interfaces required by the parent contract, then share one selector
across hooks, PH probes and restoration. No trace acceptance, mechanism gate,
season evaluation or pitching-accuracy improvement is claimed here.
