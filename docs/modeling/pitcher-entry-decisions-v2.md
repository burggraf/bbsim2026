# Pitcher entry decisions v2 — interface before live integration

This freezes the entry-record interface required by the
[pitcher-policy preregistration](pitcher-usage-policy-v2.md), alongside the
[native workload-value interface](pitcher-usage-v2-native-wire.md).
[PROGRESS](../PROGRESS.md#resume-here) remains the status ledger.
This preparation is not a completed sidecar validator or live v2 manager.

## Shared selection object

One native `select` function receives an immutable canonical before state, side,
validated club clock and the complete side's cast candidate profiles. Each profile
has ID, original-starter flag, raw relief usage/nullable target and empirical or
marked-prior relief PA sum/outings. Exactly one original starter is declared;
only that excluded starter may lack a relief outing. IDs are unique canonical
ASCII lower-case/digit/hyphen strings of 1–64 bytes. The complete cast/source and
cross-side bindings remain the full personnel reader's responsibility.

Candidate exclusions, in order: `starter`, `active`, `removed`, `not_benched`;
otherwise `eligible`. Eligibility does not invent role support or override rest:
pregame rest/support filtering still belongs to the producer. The native selector
ranks eligible candidates using the verified exact score, unclipped debt and
ascending ID. Evidence order is ascending ID; eligible rank is one-based.
The immutable returned object supplies both the chosen ID and record encoding.
A probe is not an appearance, reservation, budget draw or state mutation.

## TSV serialization

UTF-8, tab-separated, LF-terminated; no escaping or blank lines. Header rows have
exactly two cells, in this order:

```text
format\tpitcher-entry-decisions-v2
game\t<2019 game ID>
seed\t<u64>
season_policy\tseason-pool-pitcher-usage-v2
manager\tseason-pool-workload-context-2019-v2
events\t<u32 canonical event count>
decisions\t<u32 record count>
```

Each record has these rows, in order. Angle-bracket notation below is descriptive,
not literal wire text. Counts use canonical ASCII decimal. `?` means unknown;
`-` means not applicable, never zero. Integer ratios are exact and unreduced with
positive denominators; signed numerators use canonical signed decimal.

| Row | Ordered cells after row kind |
| --- | --- |
| `decision` | sequence, zero-based event index, Away/Home, reason, chosen ID or `-`, c, N, before-transition hex |
| `candidate` | sequence, ID, exclusion, U, T or `?`, PA sum, outings, debt numerator, debt denominator, clipped numerator, clipped denominator, long flag, match flag, score numerator, score denominator, rank |
| `budget` | sequence, outgoing ID, start/relief, manager word, sample index, base budget, scale, lead extension, no-hit floor or `-`, final budget |
| `applied` | sequence, canonical substitution hex |
| `enddecision` | sequence, candidate count |

Sequence is zero-based/consecutive. All cast candidates appear exactly once in
ID order. Flags are `0`/`1`. Every supported relief profile retains its exact
debt/class/score components even when excluded; only eligible candidates have a
rank position. An original starter with zero relief outings has exact debt but
`-` for undefined clipped/class/score components and rank, not an invented relief
profile. Its PA sum must also be zero. Exclusion never hides raw target/usage.
Reasons are `hook`, `ph_probe`, `ph_restore`. `chosen = -` explicitly records
exhaustion. Applied rows occur exactly when a hook/restoration selects a pitcher;
PH probes never have one. A probe witnessed by a substitution must bind an actual
PH transition, not an unrelated substitution or pitcher hook. Hook records have
exactly one budget row; probe and restoration records have none. The budget describes the **outgoing entry's**
already-evaluated sampled budget, not an invented draw for a proposed incoming
pitcher. Scale is 80/100, lead extension 0/3, base budget positive u32; no-hit
floor and final budget are u64 derived values. Final budget is the scaled base
(floored to at least one), plus lead extension, then maximized with the optional
positive no-hit floor. Sample index/word are checked against pinned role samples
and entry RNG history by the later full semantic checker, not inferred here.

The before-state witness is lower-case hex of the actual next canonical
`EventTransition::to_bytes()` at that event index. Its decoded **before** contains
the complete native state, including obligations, defense, lineup, bench and
removed IDs; its event/after are additional witness data, not a hypothetical
probe substitution. The encoder requires exact equality with the selection's
retained before state. Applied records additionally require that this actual
transition is a substitution installing the chosen pitcher on the recorded side;
its bytes are repeated explicitly in `applied`. Exhaustion cannot conceal an
actual entry on that side. A PH probe may witness a pinch hitter leaving a
pitcher vacancy only when the shared selector found a reserve; it cannot conceal
installation of another pitcher. No new state codec is introduced.

## Acceptance still required before live games

The record encoder is not whole-game acceptance. The final reader/checker must
reject missing/extra/duplicate fields, noncanonical values and broken sequence/
header/source/cast bindings; decode native witnesses through the canonical engine;
recompute eligible rankings independently; reconcile actual entries, sampled
budgets, appearances and accounting; and verify complete history. Mechanism tests
must compare witnesses with the actual event stream, including continuity between
records. Header counts or isolated legal witnesses alone do not prove that.

Record every successful PH reserve probe and actual entry selection. Suppress
repeated exhaustion only while the side/current-pitcher/candidate-eligibility/
desired-class tuple is unchanged, not once per PA; updating that tuple must also
observe intervening successful probes/selections. Whole-game capture, exhaustion
deduplication, native `--manager-trace`, receipt-last publication, tamper rejection
and trace-on/off parity remain integration obligations. No serializer or probe
may consume an RNG word to populate evidence. Historical/v1 paths stay unchanged.
