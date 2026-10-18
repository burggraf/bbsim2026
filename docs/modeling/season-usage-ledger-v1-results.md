# Observational 2019 season usage — first delivery

This implements phases 1–2 of [the usage-management plan](season-usage-management-v1.md),
not autonomous personnel selection. The user chose verified dated roster inputs as
a prerequisite for phases 3–5. No inferred availability windows, rest controls,
rotation changes, bullpen pacing, ratings refit or UI changes are included.

## What is implemented

- `audits/season_usage_targets.py` and pinned SQL export historical credited batter
  PA and **active/current pitcher outs**, including non-PA outs, from the authenticated
  read-only local snapshot. They reconcile game durations, team putouts and a
  separate SQL aggregation. Exports are exclusive; the receipt is published last.
- `preview/usage_ledger.py` accepts complete canonical accounting exactly once in
  schedule order. Identical replay is idempotent; altered receipts/payloads refuse.
  Usage, BF, starts, appearances and start/relief outs have player/team splits.
  Zero-out pitching appearances are retained; unused personnel are not invented.
- Explicit refusals advance observation only, **not usage or completed-game clocks**.
  Gaps permanently mark history incomplete. Later collection is observational,
  never complete workload/rest context for a selection policy.
- `preview/season_usage.py` verifies retained batch/target receipts and bytes,
  derives starter maps from selected lineups, reparses raw accounting, checks
  manifest identities/seeds/actors/versions/totals and rebuilds the ledger from
  scratch. It publishes ledger, all game witnesses, matched comparison and source
  receipt references into a new external directory, with its own receipt last.
  No checkpoint cache is authoritative. A missing or changed source receipt refuses.
- The runner parser explicitly supports opt-in `controlled-accounting-ph-v1` and
  expanded declared bench actors with the exact policy/availability/defense
  metadata. Default accounting stays distinct. The comparison CLI currently
  consumes existing **no-bench controlled-season-v1 batches**, not PH batch runs.

Counts use integer outs: 5.2 IP means17 outs, not5.2 decimal innings. Unknown values
remain null; known partial subtotals are separate. Actual-only/sim-only actors and
all opportunity bands remain visible in the machine-readable comparison.

## Historical target evidence

`/tmp/bbsim-season-usage-targets-v1/{first,second}/targets.json` are byte-identical,
14,730,670 bytes; SHA256
`f45cd37997e1027ac17e4e4d752c4aa44e2be96f871f510408f72e432108972a`.

- 2,429 played IDs,73,073 per-game/team/player/role records.
- **186,517 PA**, including753 IBB and61 interference PA.
- **130,270 outs**, including936 outs on non-PA events.
- All4,858 team-game putout cells and2,429 game durations reconcile.
- DET/CHA have161 played games; the other28 teams have162.
- Separate SQL and the preceding read-only audit agree on all1,065 batter-team
  and929 pitcher-team totals. No source correction was made.

Preserved caveats:62 missing derived sacrifice credits;16 mid-PA batter responsibility
switches; four suspended and ten shortened games; six supplementary position1
appearances with no live events not synthesized as workload. Recorded game dates
are not event resumption dates or transaction/IL evidence. Putout/duration agreement
is same-lineage reconciliation, not independent official-stat certification.

## Fresh fixed-seed comparison

Seeds were fixed to1,2,3 before comparison. Each fresh baseline completed2,384 games
and explicitly refused45 of2,429 inputs, with **zero native failures**. Full observed
schedule coverage is not a complete simulated season. The excluded IDs/reasons
remain in each retained batch manifest and ledger; no clock or target is charged
for them. Matched historical targets are **182,762 PA and127,791 outs** for each run,
not the full-season totals above.

| Master seed | Batter-team within band | Pitcher-team within band | Simulated PA | PA deviation | Simulated outs | Outs deviation |
|---|---:|---:|---:|---:|---:|---:|
| 1 |442/454 (97.36%)|515/562 (91.64%)|187,483|+2.583%|128,342|+0.431%|
| 2 |445/454 (98.02%)|518/562 (92.17%)|188,039|+2.887%|128,508|+0.561%|
| 3 |440/454 (96.92%)|517/562 (91.99%)|187,327|+2.498%|128,482|+0.541%|

All three meet the preregistered descriptive thresholds:80% of compared batter-team records
with at least100 historical matched PA within `max(20 PA,25%)`;80% of compared pitcher-team
records with at least60 matched outs within `max(18 outs,30%)`; aggregate errors
within5%. This does **not** mean every player passes or justify tighter post-result
thresholds. These thresholds were not retuned after results. Eligible but
uncomparable records are disclosed separately and do not turn unknowns into failures;
there are zero such qualifying records in these three actual cohorts.

For example, seed1 still has substantial errors:

| Role | Team/player | Historical matched | Simulated | Difference |
|---|---|---:|---:|---:|
| batter | ATL/culbc001 |140 PA|59 PA|−81 PA|
| batter | ATL/joycm001 |235 PA|165 PA|−70 PA|
| pitcher | TOR/mayzt001 |154 outs|328 outs|+174 outs|
| pitcher | CLE/pereo002 |119 outs|240 outs|+121 outs|
| pitcher | MIL/claua001 |186 outs|297 outs|+111 outs|

Seed1 includes40 actual-only and262 sim-only role/player/team records; seeds2/3
have36/40 actual-only and269/271 sim-only records. Absent historical observations
are not fabricated as known zero targets. Each complete comparison includes all
records, bands, coverage, starts and start/relief outs, including failed bands.

**Interpretation:** this is selected-season hindsight reconstruction using actual
starting lineups and recorded closed pitching casts/budgets. The schedule and
personnel constrain opportunity strongly. These three seeds demonstrate repeatable
measurement, not predictive accuracy, autonomous usage management, a100+ season
S-03 ensemble, verified roster eligibility, or an independent calibration win.

## Reproduction and evidence

Existing authenticated target export (use a new destination every time):

```sh
python3 -B -W error -m audits.season_usage_targets \
  /tmp/bbsim-f03.57WsAQ/2019-spike.duckdb /tmp/NEW-usage-targets
```

Replay any accepted controlled batch against the accepted target directory:

```sh
python3 -B -W error -m preview.season_usage \
  /tmp/bbsim-season-usage-v1/baselines/seed-1 \
  /tmp/bbsim-season-usage-targets-v1/first /tmp/NEW-usage-comparison
```

The baseline recipe is `/tmp/bbsim-season-usage-v1/run-baselines.py`: a frozen54a52ba
source archive, native SHA256
`90f8da0c86edb22c4cb8046e7ebc62e915d1a889fb8f61b88e2734a225121c1d`, fixed200
individual ratings, existing regular-full-game policy and SHA256 game seeds. It
produced three exclusive accepted baseline directories, not a new manager policy.
`replay-all-v3.py` generated final reviewed `comparison-v3-seed-{1,2,3}/` and seed1
repeat; prior v2 results are retained, not overwritten. All numerical headlines,
coverage and aggregates agree across v2/v3; the final reports additionally disclose
eligible/uncompared denominator counts. The four
payloads—ledger, comparison, game receipts and source receipts—are byte-identical
on separate seed1 rebuild/publication. All reports are internal, not cleared for
redistribution. `/tmp` alone is ephemeral.

Failures are retained rather than relabeled as passes:

- Target worker's initial SQL/JSON trials failed; no receipt was produced. The
  corrected accepted repeats and separate SQL verification are retained.
- Its isolated full Python suite lacked native binaries (4 failures/14 errors);
  this is not the integration-checkout result.
- Parent integration initially wrote a draft before its first complete test run;
  that ordering did not satisfy test-first practice. The first run exposed a
  fixture seed mismatch. `integration-red.stderr` then preserves a semantic
  duplicate-slot failure before its correction.
- `replay-1.stderr` rejected a selected starter without a batting accounting row.
  Root-cause evidence `missing-starter-diagnostic.txt` identifies ARI201904070:
  starter velah001 was removed before his first PA. A focused regression failed
  before removing that incorrect appearance requirement. Starting pitcher rows
  remain required; **no batting appearance is synthesized**.

The independent review found two receipt-integrity issues and a denominator
contract mismatch. Semantic regressions first reproduced all three. Corrections:
anchor target bytes to the independently approved payload SHA rather than a
replaceable receipt; pin replay recipes before computation; count only known
compared deviations in headline fractions and disclose uncomparable coverage.
The same reviewer verified all three corrections with no remaining findings.
Original and follow-up reports are retained with the acceptance evidence. The
initial full verification process was stopped for these corrections after its
Python suite passed; that interrupted run is not final acceptance evidence.

Final verification: `python3 -B /tmp/bbsim-season-usage-v1/verify-all-v2.py`
completed with exit0 and `VERIFICATION_COMPLETE` (source bytes unchanged):
722Python Werror tests,946Rust tests,4example tests, examples build, fmt,
strict all-targetClippy and diff check. Exact invocations/exits are retained in
`verification-v2/results.json`, with unmodified raw stdout/stderr. The71 focused
Werror tests and50wire/36defaultCLI byte controls also pass. The public comparison
CLI completed against seed1 in `public-cli-final/`. These checks cover the
observational slice, not the deferred autonomous manager. This result note does
not sign off G1–G5 or mark all of P-02 complete.
