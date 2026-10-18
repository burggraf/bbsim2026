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

## Canonical personnel prerequisite — preregistration, 2026-10-18

**Documentary contract, not implemented behavior.** This prerequisite remains
inside active `P-02-pinch-hitting-v1`; it does not enable an automatic manager or
clear source eligibility. Freeze this section before new native tests/code. The
first implementation checkpoint is canonical state/rules/reducer/codec plus their
callers and regressions, not a DH-only pinch-hitting shortcut.

### Verified rules and existing gaps

The [2019 Official Baseball Rules](https://content.mlb.com/documents/2/2/4/305750224/2019_Official_Baseball_Rules_FINAL_.pdf)
was retrieved again into private evidence and matches the existing rulebook SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`
(880,133 bytes). PDFKit text extraction retains all 188 pages. Relevant printed
pages 50–54 and 58–61 were inspected: 5.10(a–d) requires dead-ball substitutions,
the replaced batting slot and no re-entry; 5.10(f) requires the starter to finish
his first batter; 5.10(g) lets a reliever finish that batter **or retire the side**.
The three-batter paragraph is explicitly National Association only, not 2019 MLB.
5.10(i)'s taking-the-mound obligation and injury/illness exceptions are not modeled
by the between-events manager; do not claim that this prerequisite implements
all pitching-substitution rules. 5.11(a)(2),(4–10),(12),(14) governs starting-DH
obligation, substitute DHs, locked batting position and irreversible DH loss.

Actual flow inspected:

- `TeamState.pitcher: String` cannot represent a pitching vacancy;
  `rules::valid_team` requires a non-DH pitcher in exactly one batting slot.
  Both `reducer::apply` and `EventTransition` reconstruct personnel effects;
  `rules::finalize_personnel` owns one-way membership transfers.
- `rules::fielding_eligible` establishes broad active membership, not a canonical
  position map. `full_simulation::simulate_with_probabilities` clones starting
  defense and refreshes only position 1. A batting-slot edit alone would leave
  stale nonpitcher defense or require an invented pitching role.
- `Cast::prepare`, `next_change_with_word` and `substitution` assume an active
  string pitcher. `accounting::derive_accounting` infers fielding participants
  from batting membership and the pitcher; machine actor binding likewise needs
  new role-specific actors. Optional batting IDs alone cannot solve these gaps.
- Codec BBE25–BBE32 validates complete snapshots and selects the oldest applicable
  encoding. Adding fields to every old record would break byte compatibility.

### Selected canonical representation and transitions

1. Replace the pitcher string with a typed active/vacant role. Active stores the
   stable ID and a first-batter obligation: **untracked legacy, starting batter,
   relief batter, satisfied**. Vacant stores the retained non-DH batting-slot
   anchor, not an outgoing or fictitious active pitcher. Newly configured games
   start with the starting-batter obligation; a new pitching assignment creates
   the relief-batter obligation. A completed batter clears either obligation;
   retirement of the side also clears a relief obligation. Do not use terminal-PA
   workload as proof of the exact rule: verify reaches/outs from canonical events.
   Strict new substitutions cannot treat untracked legacy usage as satisfied.
2. Add canonical defense: explicit legacy-unassigned state, or nine position-indexed
   optional stable IDs. An empty position is a pending assignment, not eligibility
   or a zero-valued player. Filled IDs are unique, active, not bench/removed; position
   1 agrees with the active pitcher. In a DH team the other eight defenders are
   batting-order members excluding the DH; without DH, all nine are batting-order
   members. Completed assignments cover exactly that set. Caller-supplied roster
   assumptions are separate from historical availability/position certification.
3. Add an explicit offensive pinch-hitter change for the **current** batting slot.
   Incoming must be an unused eligible bench member; outgoing becomes removed,
   and cursor/score/outs/bases/PA/BF remain unchanged. A player currently on base
   needs the existing explicit pinch-runner change instead. Replacing the non-DH
   pitcher vacates pitching and position 1, after his obligation is satisfied;
   replacing another defender vacates that defender's position. Replacing the DH
   retains the DH role and starting-DH restriction. Never assign the hitter a
   pitching/fielding role merely because he entered the batting order.
4. Resolve vacancies using explicit substitution deltas: bench replacement in its
   inherited batting slot, pitching assignment, and/or position assignment for
   retained active players. Validate outgoing assignments against the before-state
   and final membership atomically, independent of delta ordering. Retained players
   cannot change batting slots; double-switch policy and already-active-player
   pitching-role changes remain explicit unsupported cases in this checkpoint.
   Defensive assignment alone neither consumes bench membership nor removes a
   retained active player. An outgoing removed player cannot be assigned again.
5. One shared canonical personnel projection must serve reducer and constructor;
   decoding must revalidate it. No eventless roster edits. New pure personnel events
   may have absent batter/pitcher participants rather than invented IDs; present
   IDs must still be nonempty and appropriately bound. Legacy participants retain
   their old meaning/bytes. A non-personnel event still requires the real current
   defensive pitcher and complete ready defense in explicit-defense mode.
6. A pending offensive vacancy may persist while that team bats, through a half
   boundary snapshot, or at game end if it never fields again. Before **any** PA,
   pitch or runner-only event on its defense, resolve every position and pitcher;
   otherwise refuse before an outcome/RNG draw or credit. Personnel events may
   resolve that state. A pinch hitter batting again in a long inning remains in
   his inherited slot; no forced substitution simply because the order wraps.
7. Keep DH loss irreversible. PH/PR for DH retain the DH; assigning the DH to
   defense requires the corresponding legal pitcher batting placement in the same
   atomic event, with no arbitrary retained-slot permutation. Unsupported DH-loss
   combinations refuse, not silently retain DH or fabricate a tenth defender.
   No pitcher-as-DH rule from later seasons. Existing supported DH-loss paths stay
   supported; the new contract does not narrow them to obtain green tests.

### Wire and caller compatibility

Reserve **BBE33** for explicit personnel state, new changes and optional personnel
participants. Older records decode to active pitchers with untracked obligations
and legacy-unassigned defense; these are explicit compatibility states, not newly
proven usage or assignments. Do not infer positions from batting order. Keep the
BBE25–BBE32 layouts, old decoder checks and oldest-applicable writer decision exact
for entirely legacy-representable transitions. Extended state in **either** snapshot
requires BBE33, including ordinary later plays after all vacancies resolve; never
silently downcast/erase it. Reject corrupt/duplicate/inconsistent assignments,
unsupported tags, trailing bytes and extended data relabeled as older versions.

The Rust source API migration must reach every actual caller and struct literal;
wire/CLI compatibility does not promise an unchanged Rust struct field type. Scope:
`engine/src/{lib,rules,reducer,codec,accounting}.rs`; mechanism validators that bind
pitcher/fielding actors; example/support initializers, real lineup, cast, probability
loaders, simulation, narration and machine accounting; `preview/season_runner.py`
role binding when new bundle actors are introduced; affected tests/fixtures and
engine contract/rules/accounting documentation. Search counts found TeamState
literals in 56 Rust files and `.pitcher` uses in 89; these are inspection counts,
not a mandate to edit all those files or an exhaustive compatibility proof.
Reuse existing model, rating arithmetic, immutable transitions and accounting.
Preserve lineup v1/cast v1–v2, generic lineup v2/cast v3, detailed/radio/accounting
defaults and mixed-schema refusal. No new input schema/automatic bench importer
belongs in this canonical checkpoint. Existing APIs need safe optional-pitcher
handling even while no production caller opts into explicit defense yet.

### Frozen implementation acceptance

Tests must fail before behavior changes. Required examples/properties:

- Non-DH pitcher → PH → PA or runner-only sequence → third out → explicit relief
  replacement in the same slot → defense. Outgoing pitcher is removed immediately;
  PH gets no pitching BF/outs/appearance or fielding event, cursor advances only
  on a PA, removed players never re-enter, runner responsibility survives. Repeat
  with PH reaching/scoring, batting again in a long inning, and a game ending
  before a replacement pitcher is needed.
- Nonpitcher PH leaves an empty defensive position; retain-and-assign versus
  replace-and-assign are distinct. No subsequent credit reaches the removed
  starter. Duplicate defenders, bench/removed/opponent IDs, stale outgoing IDs,
  role contradictions and unresolved defense refuse without mutating the caller.
- Starting-DH mandatory first turn, opposing pitching-change exception, substitute
  DH, existing legal DH loss and irreversible loss; no illegal restoration or
  batting-slot permutation hidden inside simultaneous changes.
- Starting/relief first-batter obligations: no immediate removal, completed-batter
  clearance, reliever's non-PA third-out exception, no three-batter restriction.
  Untracked legacy obligation is not accepted as evidence for new strict actions.
- Constructor/reducer/codec agree on final snapshots, conservation and refusals.
  Personnel deltas add no PA/BF/stat credits, outcomes or outcome RNG draws.
  BBE33 round-trips vacancies, pending/nonpending defense and ordinary follow-up
  plays; forged role/position states and old-version relabeling refuse.
- Freeze native records and default CLI outputs from the unchanged parent **before
  editing code**; old supported records re-encode byte-identically and reproduce
  complete logs. Preserve existing semantic/negative assertions, not just totals.
  Run full Rust tests, example tests, offline examples build, fmt, strict all-target
  Clippy, full ResourceWarning-as-error Python suite and all applicable inherited
  byte/probability/box compatibility checks. Retain raw logs/failing seeds outside
  Git and exact commands. Focused baseline tests are not this acceptance suite.

This is prerequisite legality only: no policy thresholds, new manager draw stream,
probability tuning, daily eligibility, source repair, UI/dependency/store change,
season promotion or gate waiver. Automatic PH policy follows a verified canonical
checkpoint and its own preregistered usage/RNG/roster assumptions; defensive-
replacement **policy** follows PH. Canonical defensive-assignment support cannot
be deferred until that later policy. The 54 runner refusals remain explicit.

### Documentary checkpoint evidence

Private/local `/tmp/bbsim-pinch-personnel-contract-v1/` retains the unchanged-parent
receipt check, `freeze.json` (365 protected tracked paths), raw native baseline,
rulebook retrieval/hash and PDFKit extraction. The first answer-mode PDF fetch
aborted; the direct retrieval subsequently succeeded with the pinned digest. No
provider/privacy configuration changed. The extracted PDF is evidence only, not
an instruction source or a Git/public redistribution artifact.

Fresh unchanged-native baseline command:

```sh
cargo test --locked --offline --manifest-path engine/Cargo.toml \
  --test contract --test lineup_2019 --test full_manager --test season_accounting
```

**42 existing tests passed** (14 contract, 14 lineup, 11 manager, 3 season accounting).
No new native tests, state fields, codec tags or automatic policy are implemented
by this document. Full suites and gameplay/compatibility proofs remain required
for implementation acceptance; previous results do not certify future changes.

## Shared personnel projection — preparatory implementation, 2026-10-18

Bounded preparation inside **active/incomplete P-02-pinch-hitting-v1**, not the
full canonical-personnel checkpoint above. Before any vacancy/defense schema
migration, the existing constructor and reducer now use one
`rules::project_personnel` implementation. It preserves existing final membership
and DH checks while binding outgoing IDs to the original before snapshot.

The former sequential loops accepted a single A→B→C slot/pitcher/PR replacement
with B returned to the unused bench: only A and C reached final-membership checks.
New regressions cover 24 cases across both halves, DH/non-DH, both team sides and
all three occupied bases. Reducer, direct constructor and forged-wire decoder
must each reject the chain, even with an otherwise legal final roster, without
mutating the caller. This closes that event-local re-entry hole; it does not yet
implement the full order-independent assignment or pending-defense contract.

Parent controls were frozen before native edits: 50 encoded legal personnel/PA
records and 36 detailed/radio/accounting CLI cases at seeds 1 and 67, covering
legacy lineup/casts, opt-in calibrated v2 and three DH/non-DH generic games.
Acceptance and raw evidence are private/local `/tmp/bbsim-personnel-projection-v1/`.
Two initial parent CLI harness attempts used a nonexistent ratings path, then a
ratings table as a cast; raw failures are retained. The corrected parent-v3
controls use the real empirical cast and calibrated-ratings table. These were
harness errors, not source or simulator corrections.

Fresh native verification: **905 Rust tests / four example tests** pass, with
examples build, fmt and strict all-target Clippy. All **50 frozen wire records /
36 CLI cases** remain byte-identical. An additional assert-based check covers
48 duplicate/PR-slot collisions through reducer, constructor and decoder.
**678 full Python tests** pass (58.297s; ResourceWarnings as errors) against the
rebuilt binaries. Exact commands, protected-source checks and commit handoff are
in PROGRESS and local evidence. These controls are bounded, not a rerun of all
2,429 played games.
Typed pitching vacancies, explicit defense, first-batter obligations, BBE33 and
automatic PH remain unimplemented. No roster/data repair or probability/RNG/policy
change belongs in this slice. The earlier 54-refusal statement is historical;
the [regular-full-game correction](season-game-inputs-v1.md#approved-regular-full-game-policy--2026-10-18)
now records 45 personnel/data refusals, not exceptional-result gating.

### Canonical personnel implementation — verified local prerequisite

The preparatory paragraph above is historical. The uncommitted successor now
implements active/vacant pitcher roles, pending/assigned positional defense,
first-batter obligations, PH/assignment transactions and optional substitution
actors across the canonical engine, codec, accounting and real Rust callers.
See [current type/wire contract](../engine/contract.md#canonical-personnel-extension-bbe33),
[rules](../engine/2019-rules.md) and [accounting](../engine/accounting.md).
`PROGRESS` remains the single live status/next-action ledger.

BBE33 retains explicit state in both snapshots, including ordinary follow-ups;
legacy conversion remains untracked/unassigned. Compatibility is event-wide:
optional/new/mixed strict events cannot use one legacy team to bypass an unknown
pitcher's first-batter obligation. Assignment projection binds to original
personnel, not sequential intermediate entrants. Ordinary producers reject
unready defense before callbacks or RNG; supplied positional inputs bind to the
canonical map. Explicit catcher facts and PB credits require assigned C, not any
active fielder. Offensive vacancies can survive long innings or a walk-off.

Raw local evidence is `/tmp/bbsim-canonical-personnel-v1/`. Semantic regressions
cover consumed RNG on refusal, already-active pitching, overflowing out counts,
untracked-usage bypass, standalone vacancy panics, stale supplied positions and
wrong assigned catcher identities. The first catcher test draft omitted a
required third-out clock in its positive control; that fixture failure is not
semantic RED. Corrected `assigned-catcher-red-v2.*` supplies behavioral RED;
`assigned-catcher-green.*` verifies reducer/constructor/decoder and delivery paths.

Earlier full attempts are preserved, not counted as acceptance: v2 stopped on an
illegal-frame fixture built through a newly guarded producer; v3 records932 Rust
passes/two validation-order failures. Fixtures now independently assert producer
and native refusals using legally built events, preserving callback/RNG checks.
Ended-game readiness intentionally precedes stale-participant diagnostics; the
walk-off test retains exact reducer and constructor errors. Only those two test
files changed after v3; production source did not.

Fresh **v4 passes934 Rust tests, four example tests,77 focused tests,15 additional
regression methods and678 Python tests** (`-B -Werror`,83.627s), plus example build,
fmt and strict all-target Clippy. Full Rust is unfiltered (`--no-fail-fast`),
including the10,000-game continuity/independent-accounting simulation; no failures
or ignored tests. Authenticated50wire/36CLI parent controls remain byte-exact.
Source-before/after pins match current files;244 protected paths and HEAD stay
unchanged. Raw logs/commands are `full-v4/{checks.json,*.stdout,*.stderr}`;
`verification-v4.json` SHA256 is
`c6cc191518d8b63b4de88c598e56122fd6b3e1459c46508eb1b90c667985d01c`.
Reproduce native checks with `python3 -B /tmp/bbsim-canonical-personnel-v1/run-full-v4.py`
using a NEW exclusive evidence directory (the existing driver refuses to overwrite
its completed evidence), or the exact Cargo argv in `full-v4/checks.json`.
Python uses `python3 -B -Werror -m unittest discover -s tests -v`; compatibility
uses the private `run-compat-v4.py` with a fresh output root. These are private
local artifacts, not durable publication. The bounded canonical prerequisite is
verified locally; source remains uncommitted and no gate promotion is claimed.

Automatic PH, bench eligibility/import, defensive-replacement policy, double
switches and already-active pitching-role changes remain deferred. No production
input schema, probability/default/RNG policy, source data or dependency changed.

## Automatic-PH policy — preregistration, 2026-10-18

The canonical prerequisite is verified above. The next bounded implementation is
**a standalone deterministic decision helper**, not CLI integration or completion
of automatic management. Policy/version: `late-close-expected-bases-ph-v1`.
Freeze this contract before policy tests/code or new source measurements. These
are provisional, unfitted tactical choices, not inferred historical PH rates.

### Bench and decision inputs

- The caller explicitly supplies an **assumed-available closed hitter pool**, with
  stable IDs and supplied assumed defensive positions2–9. Empty position evidence
  is unknown, not fielding eligibility. Every live candidate must be on the current
  offensive team's unused canonical bench. Reject duplicate IDs/positions,
  malformed IDs, position1/out-of-range positions, cross-side membership and
  entirely absent actors. A pool member already active/removed is unavailable,
  not a new entrant; do not re-enter him. Validate metadata before rating callbacks.
- Recorded pitching-cast actors are excluded from this hitter pool. A separate
  optional caller-declared reserve pitcher must be a distinct unused own bench
  member, not a hitter-pool member or opponent. This declaration is an explicit
  availability/role assumption, not proof of historical pitching eligibility.
- No current input schema is extended in this first helper slice. Future controlled
  export may use the audited same-game participant union only with its assumptions,
  exclusions and provenance retained. An empty observed pool does not establish
  zero historical eligible bench. No daily roster/injury/rest inference, source
  entry-inning/outcome selection or new data transfer.
- Require valid canonical explicit personnel, not upgraded legacy uncertainty.
  Ready current defense is mandatory. If the offense has pending assignments,
  decline another PH until resolved; no fictitious pitcher or stale defender.
  Ended games decline without a rating callback. Earlier/lopsided valid frames
  decline without rating callbacks or RNG.

### Frozen decision rule and canonical action

Consider all current batting slots, including non-DH pitcher, DH and fielders,
from inning7 onward when the absolute score difference is at most3, at any out/base
state. These are versioned policy thresholds, not fitted usage targets.

For each currently unused candidate, first build the legal substitution through
canonical `apply`, before querying probabilities. For a fielder, require supplied
coverage of his current position and include the corresponding explicit defensive
assignment in the same event. A DH needs no invented defensive position. A
non-DH pitcher requires the declared unused reserve pitcher; PH vacates pitching
and position1, to be resolved by the future integration before that team's next
ordinary defensive event. The reserve is not assigned now or credited an appearance.
Canonical first-batter/starting-DH restrictions remain authoritative; an
`IllegalStateChange` candidate is ineligible, while other canonical errors surface.

Only if at least one legal candidate exists, query the outgoing batter and those
candidates against the **current opposing pitcher** using the selected existing
rating model, not sampled outcomes. Require finite strictly positive normalized
nine-category vectors (sum tolerance1e-12). Rank by the explicit coarse score
`BB + HBP + 1B + 2*2B + 3*3B + 4*HR + ROE`, with K/BIP-out weights0. This is an
expected-bases proxy, not run expectancy or a new PA distribution. Choose the best
candidate only when his score exceeds the outgoing batter's by **strictly more
than0.10**. Stable-ID ascending breaks equal-score ties; input order must not matter.
No policy RNG is needed; do not consume outcome or pitching-entry words. Return
only the winning validated canonical transition, or no change. Never mutate the
supplied frame. Substitution adds no PA/BF/outs/runs/fielding/pitching appearance.

### Scope, integration order and acceptance

First slice: new `engine/examples/support/full_pinch.rs`, its focused native test
and this contract/`PROGRESS`. No current caller/default/output change. Subsequent
integration must separately cover bench preparation without `Cast::prepare`
conflicts, typed starting obligations/positions, offensive PH before the next draw,
mandatory defense restoration **before** the loop's readiness guard, workload
entry tracking, current-position fielders, replacement ratings (including calibrated
v2), names/narration, machine role bindings and receipt-last controlled exports.
No silent fallback to pooled ratings, legacy role uncertainty or unqualified defense.
Do not mislabel a passing standalone helper as completed automatic PH.

Before implementation, tests must expose absent behavior (interface absence is
not semantic RED). Then cover both halves/DH modes, pitcher/DH/fielder PH,
first-batter/DH obligations, pending and game-over refusal, inning/score/gain
boundaries, unused reserve requirements, unknown positions, duplicate/foreign/
used IDs, malformed probability vectors, stable ties/shuffles and immutable input.
Verify selected IDs/current pitcher callback binding, BBE33 roundtrip, preserved
batting slot/DH/runner charges and independent zero-stat substitution accounting.
Run all applicable native/Python/build/fmt/strict-Clippy and parent byte controls;
authenticate the122-source canonical prerequisite before edits and distinguish
new policy paths from untouched core/default callers. No fitting, source repair,
full-season acceptance, public redistribution or gate promotion.

### Standalone helper verification — 2026-10-18

Implemented only `engine/examples/support/full_pinch.rs` and its native test;
no existing native source or CLI caller changed in this policy slice. Private
`/tmp/bbsim-pinch-policy-v1/preregistration.md/json` preserve the pre-code contract.
Interface absence is retained separately from five behavioral no-op failures.
An additional semantic RED caught a malformed reserve ID accepted before rating
callbacks; shared ID validation corrects that boundary. Nine focused methods pass,
including96 base/out/side/DH cases, canonical BBE33 and zero-stat substitution checks.

Fresh managed `proc_e539` exits0: `python3 -B
/tmp/bbsim-pinch-policy-v1/run-full.py` runs943 Rust tests/66 result blocks, zero
failed/ignored, including the10,000-game continuity/accounting test. All10 native
stages pass, including examples build,86 focused methods, four example tests,
fmt and strict all-target Clippy. Managed `proc_c0ef` exits0: `python3 -B -Werror
-m unittest discover -s tests -v` passes678 tests (61.843s); `python3 -B
/tmp/bbsim-pinch-policy-v1/run-compat.py` preserves50 wire records/36 CLI cases
byte-exact, with unchanged native binary/rlib hashes. Driver output directories
are exclusive; use a fresh evidence root rather than overwrite completed logs.

`verification.json` authenticates124 native pins (original122 unchanged),244
protected paths, seven input/database pins, raw logs and unchanged HEAD; SHA256
`5e5b041e614dceeb30fefc6cd2dae11796ae6b982526ba064f8b3f14a4dfcd20`.
This supersedes `checkpoint-pending.json`, not the retained RED evidence.
The helper trusts explicitly supplied availability/role assumptions; it cannot
certify their historical provenance or completeness. Future import/integration
must cross-check pitching-cast exclusions and preserve unknown roster evidence.
**Automatic PH remains unintegrated**: no default/input/RNG change, independent
review, fresh historical full-season run, commit/push or gate promotion. Evidence
and source remain private/local, not remotely durable.

## End-to-end PH integration acceptance — 2026-10-18

Continue P-02 through actual `text_game` games, not another standalone checkpoint.
Add optional `--bench` with strict `assumed-game-bench-v1` key/value TSV: matching
`game_id`, frozen PH `policy`, explicit `availability_basis` (same-game participant
proxies assumed available at start; not verified eligibility), `defense_basis`
(supplied observed positions assumed usable; not verified defensive eligibility),
snapshot/supplement/opportunity hashes and attribution. Per-side stable actor rows
supply name, positions2–9 (or `?`) and nine batting counts. Empty pools remain
unknown historical availability, not known empty rosters. Reject extra/duplicate/
malformed fields, cross-side/starting/cast identities and name/count conflicts.
No pitching-cast member enters the hitter pool. No future results or entry innings
are runtime manager inputs. No source data repair, new dependencies or fitting.

Prepare the cast once, then add hitters and explicitly initialize complete starting
positions, starting-batter obligations and mandatory starting-DH turn at a full-game
origin. Keep the old path unchanged without `--bench`. On each iteration restore a
defensive pitching vacancy from the next unused cast pitcher BEFORE readiness/RNG;
replace the PH in that original batting slot with that pitcher atomically. Do not
assign a pitcher or invent an appearance while the PH's team is still batting.
Preserve first-batter obligations for ordinary pitching hooks too. After any opposing
pitching decision, run the frozen PH policy before play draws; evaluate legal
candidate **canonical after-states** using the same selected rating model. PH events
have no draw receipt. Fielders come from current assigned positions. Current and
incoming hitters must have complete count profiles; calibrated artifacts must also
cover the full expanded role/count set, never silently use standard ratings instead.

Acceptance: complete deterministic DH and non-DH games that actually contain PH;
PH reaches/bats/scores, replacement ratings appear in the next consumed PA vector,
removed players do not re-enter or make defensive plays, vacancy survives offense
and resolves before the next defensive event, inherited charges/cursors and independent
accounting reconcile. Explicitly render PH, associated position assignment and forced
pitcher entry in detailed/radio/box and machine accounting without false pitching
appearances. Malformed input refuses before output. Retain exact no-bench parent
50wire/36CLI controls; run full native/Python/examples/build/fmt/strict Clippy.
P-02 stays active until this evidence exists; no full-season/fidelity/gate promotion.

Supplemental stress acceptance (frozen before that run): reuse the exact full-game
regression with seeds1–256, both DH modes and both rating models, each repeated:
1,024 distinct configurations/2,048 complete games. Change only its seed bound in
an outside-repository driver; retain every assertion and raw output. This supplements,
not replaces, the unfiltered native suite and its 10,000-game check. These are synthetic
mechanism controls, not historical usage or predictive-accuracy evidence.

## End-to-end PH game results — 2026-10-18

**Bounded opt-in PH v1 is verified**, not just its helper. `--bench` connects strict
bench preparation, canonical candidate-state ratings, deterministic policy, explicit
vacancy/restoration, current fielders and named detailed/radio/box/accounting output.
No-bench behavior remains unchanged. Accounting uses `controlled-accounting-ph-v1`
with policy/availability/defense disclosures and the actually selected ratings model;
it does not masquerade as the no-PH season-runner schema. The old cast-only calibrated
artifact is correctly refused when expanded actor/count coverage is missing.

Fresh `/tmp/bbsim-pinch-integration-v1/full-v2/` checks pass:

- Unfiltered946 Rust tests,66 zero-failure summaries, including the10,000-game method;
  four example tests,130 focused tests, build/fmt/strict all-target Clippy.
- `PYTHONWARNINGS=error python3 -B -m unittest discover -s tests`:680 tests pass.
- Authenticated50 old wire records and36 no-bench CLI cases remain byte-identical.
- All125 native before/after source pins,244 protected paths and seven source/DB
  input pins match; the eight prior native files changed by this integration are
  explicitly enumerated. No source-data repairs, default-rating changes or policy-threshold retuning.

Selected **fixed200 simulated games using2019 source inputs**, not historical event replays:

| Fixture, seeds1–32 | Completed | PH events | Forced pitching restoration | PH PA / H / R |
| --- | ---: | ---: | ---: | ---: |
| `PIT201904220`, non-DH |32|27|16|33 /7 /4|
| `BAL201905230`, DH |32|6|0|10 /3 /3|

All64 named boxes reconcile against independently checked machine actors/counts,
with76 malformed-schema/membership/missing-artifact/actor/count/header controls.
For this verification only, the three PH disclosures are checked exactly and the
version header is normalized in memory so the existing strict conservation checker
can be reused unchanged; raw PH artifacts remain intact. This does **not** extend
the production season runner to PH. First PH games repeat byte-for-byte in all three
formats. Non-DH PIT seed4 also repeats all three formats and explicitly shows:
**Ketel Marte bats for Zack Godley; Matt Koch subsequently takes the mound and slot9.**
PH actors never receive fictitious pitching rows. Their current batting ratings and
legal defensive continuity are separately exercised in the full-game regression.

The supplementary unchanged-assertion stress driver passes1,024 DH/model/seed
configurations, each repeated:2,048 complete games. The first executions contain
1,472 PH selections and68 pitching restorations; both models and both offensive
halves are exercised. These synthetic controls are not historical usage estimates.

**Negative evidence retained:** initial CIN/OAK probes and LAN/MIL demonstration
attempts select no PH under the frozen policy. Positive fixtures were deliberately
selected for substitution coverage, not representativeness. Metadata/origin semantic
REDs caught incorrect PH model labeling and46 rating callbacks from a non-origin
frame; fixes now refuse nonzero batting cursors/removed personnel before callbacks.
The first full native run was stopped before these refinements and is not claimed
as passing. Evidence-index v1 mistakenly hashed its own open stdout; v2 corrects that
publication mistake without altering raw tests or weakening assertions.

Final evidence: `/tmp/bbsim-pinch-integration-v1/verification-v2.json`, SHA256
`e157baeb86752716adffcf881fd2d2168e9a8761d81def264bd7b4c92654425d`;
`artifact-pins-v2.json` authenticates975 completed artifacts. The coherent local
checkpoint/private handoff do not authorize a push or public data publication.
Bench availability and positions remain explicit assumptions, not roster eligibility.
Full-season PH orchestration, defensive-replacement policy, usage fidelity, licensing
clearance and G1–G5 remain outside this bounded completion.

### Play the pitcher-PH example

After building examples with `cargo build --locked --offline --manifest-path engine/Cargo.toml --examples`:

```sh
engine/target/debug/examples/text_game \
  --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/bbsim-joint-preview/counts.csv \
  --lineup /tmp/bbsim-pinch-integration-v1/source-pit/lineup.tsv \
  --cast /tmp/bbsim-pinch-integration-v1/source-pit/cast.tsv \
  --bench /tmp/bbsim-pinch-integration-v1/source-pit/bench.tsv \
  --seed 4 --format radio
```

Readable saved output:
`/tmp/bbsim-pinch-integration-v1/actor-check-v1/pitcher-ph-004.radio.first.stdout`.
Use `--format detailed` or `--format accounting` with the same inputs/seed.
