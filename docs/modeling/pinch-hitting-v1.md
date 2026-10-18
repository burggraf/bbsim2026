# P-02-pinch-hitting-v1 — source and legal-state prerequisite

Base `10353a1`; next approved feature after the [controlled runner](season-runner-v1.md),
before defensive-replacement policy. No UI, download, dependency, fitting, push or
gate change. This prerequisite does not itself implement automatic pinch hitting.

## Acceptance fixed before new bench measurements or implementation

1. Authenticate the same five complete cached artifacts with the existing game-input
   reader. Reuse full lineup and opportunity reconciliation, not a partial extract.
2. Retain every played ID and both sides, including unsupported/exceptional games.
   Report literal appearance-position counts, starting identities, nonstarting
   observed actors, recorded pitching actors, ambiguous cross-side identities,
   missing people, missing/unknown position/range and reversed ranges. Preserve
   problem actors and all their reasons; do not repair or silently delete them.
3. A possible controlled bench is only a **labeled same-game participant proxy**:
   nonstarting, nonpitching observed actors with unambiguous side, complete identity,
   supplied nonreversed ranges and known position codes. It is NOT an active roster,
   opening bench, injury/rest clearance, or evidence of actors who never appeared.
   Do not infer zero available bench from zero observed candidates. Do not use an
   actual future PA result or entry inning to make a managerial choice.
4. Reuse existing batting-count validation. Complete-cohort profile absence may be
   labeled zero observed batting counts/neutral prior, never missing person or role
   evidence. Report profile opportunity size, explicit cold status, and multi-team
   identities without collapsing players by display name. No new ratings arithmetic.
5. Report per-game DH/no-DH, existing exceptional-game refusal, candidate counts,
   raw positions and all exclusions; aggregate by team and DH mode. Mark counts of
   opening-bench availability unknown, not zero. No target success rate or retuning.
6. Synthetic tests precede code: starters/pitchers excluded from the candidate proxy;
   cross-side IDs, missing people, unknown/reversed ranges retained and quarantined;
   cold profiles, deterministic shuffled input, team splits, exceptions and source
   immutability; exclusive publication, drift and unsupported source refusal.
7. Actual read-only proof runs twice on all2429 IDs. Reports must be byte-identical,
   original five input hashes unchanged, and independent direct-source set/count
   calculations must reconcile candidates/positions/reasons. Keep raw failures and
   proof outside Git. Run full Python discovery with ResourceWarnings as errors,
   diff/local-link/source-preservation checks. No Rust behavior changes in this
   prerequisite; prior Rust passes are historical evidence, not new certification.

## Legal-state finding to resolve before automatic management

Current `rules::valid_team` requires a non-DH current pitcher to occupy a batting
slot. A batting-slot-only pinch hitter for that pitcher therefore cannot be a legal
canonical transition. Pretending that the hitter becomes a pitcher and replacing
him before facing a batter is not a valid workaround. Keeping a removed pitcher
active or changing batting order without an event is also forbidden.

The native simulation additionally retains starting nonpitcher defensive identities;
only its pitcher position is refreshed. Ordinary fielder pinch hits need a coherent
subsequent defensive assignment, not credits to a removed starter. DH substitution
has a different, already-supported starting-DH first-PA obligation.

After the source proof, preregister the smallest explicit canonical representation
for a pinch hitter and any outstanding pitching/defensive assignment, its resolution
before the next defensive play, one-way eligibility and no re-entry, and wire-format
compatibility. Then implement the automatic policy separately from outcome draws.
Do not silently narrow the requested feature to DH-only or weaken existing role
validation. New state, loaders, rating-required actors, simulation hooks, narration,
machine role bindings, independent accounting and their actual callers must agree.
Defensive-replacement **policy**, daily roster inference and statistical usage/rate
fidelity remain separate work; necessary pinch-hit legality is not optional.

## Source proof — 2026-10-18

The read-only prerequisite is verified; **automatic pinch hitting is not implemented**.
`audits/pinch_hitting_readiness.py` reuses the authenticated full-cohort reader and
count validator. It cannot be passed to the simulator as a bench roster. Original
acceptance was frozen at `/tmp/bbsim-pinch-hitting-v1/preregistration.md` before code
or new measurements; results below append to that contract, not replace it.

Final source-bound proof: `/tmp/bbsim-pinch-hitting-v1/final-v2/proof-results.json`.
Two CLI runs covered all 2,429 played IDs, 4,858 sides and 30 teams in 10.797423s and
11.333279s. Both 53,632,711-byte reports are byte-identical, including provenance.
Direct raw-source calculations independently reconciled game/side identity sets,
positions, every exclusion-reason set, candidate sets, counts and per-team splits.
Five input hashes, all 230 prior executable/config pins and 363 protected tracked
paths were unchanged; the proof additionally binds 89 Python source/test paths.

| Source observation | Result | Meaning |
| --- | ---: | --- |
| Observed actor-side-game entries |67,273| Not unique players or all batting participants |
| Starting actor-side-game entries |46,150| Not bench candidates |
| Recorded pitcher actor-side-game entries |21,433| Excluded from this nonpitching proxy |
| Participant-proxy entries |4,575|1,808 DH /2,767 non-DH; repeated players/games, not eligible roster slots |
| Zero-observation batting proxy entries |1| Explicit cold vector/neutral prior, not missing identity or zero ability |
| Multi-team player IDs |145| Source fielding-participant identities, not simulated role identities |
| Unsupported-starting-map actor entries |121| All remain in the report with reasons |
| Reversed-range actor entries |1| Original source fact retained; no source repair |

Reasons overlap and must not be summed as a partition. Cross-side, person-lookup,
unknown-side/position/range issues are absent in this authenticated extract; their
refusal/quarantine paths have synthetic regressions. Exceptional result records
are retained verbatim and are **not** cleared by candidate counts.

**Coverage finding:** the fielding-appearance extract contains only positions 1–10;
no 11/12 rows. This does not prove absence of PH-only/PR-only appearances. Players
who batted but never fielded may be missing from this proxy. Opening-bench and
pregame availability remain JSON null throughout, including teams with no proxy.
Before treating a bench pool as controlled game input, preregister a same-game,
stable-ID comparison/union with the already-cached terminal-batter participants;
retain PA-source exclusions, side ambiguities and missing evidence. No new source
transfer is implied. Neither source establishes a daily eligible roster.

### Safeguards, failures and fresh checks

Interface/semantic REDs are retained at the evidence root. Inspection added a
failing disclosure regression before explicitly labeling fielding-only coverage.
A private-copy publication regression also proved that checking the destination
only before measurement allowed a renamed parent/symlink redirect. Final code
rechecks the canonical external destination before the existing exclusive writer;
redirects into a private stand-in repository or another outside directory refuse.
No live source/module was modified for these negative controls. Earlier proof and
suite runs remain separate and do not certify the changed final source.

Final checks: **9 focused / 671 full Python tests passed**, with ResourceWarnings
as errors; full suite 56.319s. Raw logs are `focused-final-v2.*` and
`final-v2/python.*`. A supplemental log checker initially miscounted 12 HTTP tests
whose success markers follow interleaved server logs; its separate failure and
correction are retained in `final-v2/verification-checker-initial-failure.txt`.
No test output or repository test assertion was changed for that checker.

Reproduction (use a **new exclusive outside-repository output**, never overwrite
retained evidence):

```bash
python3 -B -Werror::ResourceWarning -m audits.pinch_hitting_readiness \
  /tmp/bbsim-season-readiness-v1/first.lineup.json \
  /tmp/bbsim-season-readiness-v1/first.lineup.native.csv \
  /tmp/bbsim-player-opportunities/report-first.json \
  /tmp/bbsim-pitcher-workload/report-null-fixed-first.events.csv \
  /tmp/bbsim-season-readiness-v1/first.json /NEW/EXCLUSIVE/report.json
PYTHONPATH=tests python3 -B -Werror::ResourceWarning -m unittest test_pinch_hitting_readiness -v
python3 -B -Werror::ResourceWarning -m unittest discover -s tests -v
git diff --check
```

The source proof driver and raw per-run stdout/stderr are in `final-v2/`; they use
exclusive artifact creation too. `/tmp` evidence is private/local, not durable or
licensed for redistribution. No fitting, source repair, native policy/RNG/rating/
accounting/default/schema change, independent model review or gate signoff occurred.
Rust suites are not rerun for this Python-only audit; unchanged pins preserve the
boundary, not a new Rust acceptance claim. The runner's 54 explicit refusals and
previous PA/credit discrepancy remain unresolved.

## Cached terminal-batter comparison — preregistration, 2026-10-18

Before implementation or measurement, extend this audit only through explicit
`--participants` / `inventory_participants(source)` / `export(..., participants=True)`.
Default v1 inventory and CLI semantics remain unchanged. V2 retains all v1 findings
and adds literal per-game/side fielding-only, terminal-batting-only, intersection
and union stable-ID sets. These are participation sets, **not candidate/eligible
bench rosters**. Opening availability stays null. No native/manager changes.

The authenticated cached event CSV has no batting-side column. Attribute a terminal
batter only to the opposite of the event pitcher's **unique recorded position-1
side in that same game**. Unknown/missing/cross-side pitching evidence leaves the
terminal event unassigned, with game/event/batter/pitcher IDs and all reasons;
never guess from team names, event order, inning or batter identity. Retain source
fielding/terminal-side conflicts as explicit quarantine reasons. Unknown-side
position-1 rows poison otherwise unique pitching evidence. Unknown game IDs remain
unassigned. Non-PA rows are counted separately and do not establish terminal batting
participation. IBB and interference establish participation even though excluded
from modeled batting counts; retain their distinct exclusion categories. No actual
PA outcome or entry timing is supplied to a manager.

Terminal actor records retain sorted event keys and inclusion/exclusion counts,
identity and validated existing full-cohort batting profile/cold-zero status.
Missing fielding evidence means unknown position/range/defensive eligibility, not
an invented PH position or clearance. Preserve all prior fielding quarantine and
exceptional-game records. Same display names never merge IDs; team splits remain.

Acceptance: test first terminal-only/fielding-only/both sets, IBB/interference,
non-PA exclusion, unknown/ambiguous pitcher side, cross-source side conflict, missing
people, cold profiles, deterministic shuffle, source immutability and opt-in export
safeguards. On all cached games, independently derive literal sets from raw CSV
and appearance rows; reconcile every side and aggregate set cardinalities and
exclusion counts. Run CLI twice into new exclusive external destinations with
byte-identical reports; recheck five source hashes and all non-owned tracked files.
Compare unchanged v1 inventory with its accepted report excluding provenance. Run
full Python discovery with ResourceWarnings as errors and `git diff --check`.
Preserve raw failures/logs under a new private evidence root. No coverage target,
new transfer, rating fit, eligibility claim, native proof or gate promotion.

After this bounded comparison, the next task is the explicit canonical pitching
vacancy/defensive-assignment and wire-compatibility preregistration above, not
automatic-policy implementation.

### Cached comparison results — 2026-10-18

Verified the bounded source follow-up, not pinch-hitting management. Two complete
CLI runs with `--participants` took 12.420620s / 12.394297s and produced identical
98,262,582-byte reports. Evidence: `/tmp/bbsim-pinch-participants-v1/proof-results.json`,
`prove.py`, `first.*`, `repeat.*`, `proof.*` and `batting-only-findings.json`.
All 2,429 games / 4,858 sides / 30 teams remain present. The existing v1 report
content is unchanged (provenance necessarily identifies the updated audit recipe).

| Actor-side-game participation set | Total | DH | Non-DH |
| --- | ---: | ---: | ---: |
| Fielding only |19,649|11,201|8,448|
| Fielding and terminal batting |47,624|23,105|24,519|
| Terminal batting only |4,027|647|3,380|
| Union |71,300|34,953|36,347|

This confirms that fielding-only proxies omitted observed batting participants:
4,027 actor-side-game entries across 556 stable player IDs. All have known people
and positive full-cohort batting opportunities; all lack same-side fielding evidence,
and five retain unsupported-starting-map reasons. None is thereby cleared for
defense or certified available at game start. Zero cross-side terminal actor entries
were observed; synthetic tests quarantine that condition. PR-only and never-used
participants remain unproven. These totals include starters and pitchers; they are
not an expanded count of eligible bench players or simulated appearances.

All 186,517 terminal rows receive a unique opposite-pitcher side; none is unassigned
in this authenticated extract. They include 185,703 modeled-count observations,
753 IBB exclusions and 61 interference exclusions. The separate 5,508 non-PA rows
do not establish terminal-batter participation. Independent raw-row calculations
reconciled every side's four sets, actor/event/category/profile/reason records,
team and DH aggregates, and unassigned records. Five source hashes, 363 protected
tracked paths and 89 Python source/test pins remained unchanged during proof.

Fresh checks: **15 focused tests passed** (0.041s), **677 full Python tests passed**
(57.977s), ResourceWarnings as errors; raw `focused-final.*` / `python.*` logs.
Interface/semantic, vector-format and terminal-only cross-side REDs are retained.
Publication redirect checks cover both modes; the shared exclusive/drift guards
remain in place. `git diff --check` and local documentation links pass. An initial
supplemental link checker mistook historical inline Python for a Markdown link;
its failure/root cause is retained at `link-checker-initial-failure.txt`; source,
assertions and raw test outputs were not changed for it.

Reproduce with the preceding CLI command plus `--participants` and a **new**
external output path; the default remains v1. The proof script is exclusive too,
so choose a new evidence root rather than rerun over its accepted artifacts.
Evidence is still private/local `/tmp`, not remotely durable or licensed for
redistribution. No native suites were rerun or native/policy behavior changed.
Next: preregister canonical pitcher-vacancy/defensive-assignment legality and wire
compatibility before implementing substitutions or automatic decisions.
