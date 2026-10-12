# 2019 cached outcome inventory — F-01 follow-up

## Conclusion and scope

**The cache is substantially richer than the preview, but it does not establish complete coverage of every out mechanism.** Fly, line-drive, pop-up, bunt, sacrifice, fielders-choice and multi-out evidence exists. The preview deliberately collapsed much of this into a ground out; changing narration alone would not fix the simulation.

This is a read-only census of the existing [F-03 snapshot](2019-filtered-spike.md), not a new import, model fit, legality certificate or independent official-record reconciliation. **Correction after pinned source inspection:** the initial v1 census omitted the cached location-angle field. Version2 adds literal angles and2,593 recorded `Foul` tags; these are foul-territory evidence, not a universal fair/foul ruling or catch count. The earlier blanket absence of foul evidence is superseded. [Pinned definitions and supplement contract](2019-source-detail-scope.md) distinguish recorded locations from legal ball status.

All source rows, generated CSV/JSON, examples and raw verification logs remain outside Git. Simulation, RNG, baseline and UI are unchanged; historical test assertions are retained. Internal-use/licensing and G1/G2 limitations remain in [the ledger](../PROGRESS.md).

## Provenance and reproduction

- Source: `https://data.baseball.computer/baseball/v1/baseball.ducklake`; catalog SHA-256 `2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583`.
- Cached schema `f03-spike-1`, extraction `2019-filtered-extract-v1`, selected 2019 MLB regular season under the original F-03 filter. Original manifest/timestamp/Retrosheet attribution is retained in the JSON report. Neither census refreshed the remote catalog. The follow-up fetched only bounded pinned source-model/documentation text, not catalog or season data.
- Database `/tmp/bbsim-f03.57WsAQ/2019-spike.duckdb`: SHA-256 `d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528`, unchanged before/after every read-only reproduction.
- [Event-grain query](../../audits/2019-outcomes.sql), [separate-path controls](../../audits/2019-outcome-controls.sql), [inventory CLI](../../audits/outcome_inventory.py), [tests](../../tests/test_outcome_inventory.py).

Run from repository root with the existing DuckDB CLI and Python 3.11+; no Python DuckDB package or additional dependencies:

```sh
set -euo pipefail
DB=/tmp/bbsim-f03.57WsAQ/2019-spike.duckdb
OUT=$(mktemp -d /tmp/bbsim-outcomes.XXXXXX)
duckdb -readonly -csv -nullvalue '' "$DB" -c 'SELECT * FROM audit_manifest;' > "$OUT/manifest.csv"
duckdb -readonly -csv -nullvalue '' "$DB" < audits/2019-outcomes.sql > "$OUT/events.csv"
python3 audits/outcome_inventory.py "$OUT/events.csv" "$OUT/manifest.csv" "$OUT/report.json"
duckdb -readonly -csv -nullvalue '' "$DB" < audits/2019-outcome-controls.sql > "$OUT/controls.csv"
python3 -Werror::ResourceWarning -m unittest discover -s tests -v
```

Queries disable external access. `-nullvalue ''` is essential: DuckDB's default CSV display emits literal `NULL`, which is not the parser's absent-value convention. Repeat the exports/report and use `cmp`; preserve the original DB hash. Controls contain 19 scalar metrics and 17 result cells for comparison with JSON totals, supplied units and result partitions. They use different aggregation paths through **the same upstream cache**, not an independent baseball oracle.

Historical accepted **v1** outputs at `000e51a`, `/tmp/bbsim-outcomes-first-{events.csv,report.json,controls.csv}`, remain retained unchanged; reproduce them from that checkpoint. They omit angles:

| Artifact | SHA-256 |
| --- | --- |
| Event CSV, 34,047,180 bytes | `5b6db9c18ae275286348eca378866d6541cf68c5a660cb15abb1d9fbf191d6a6` |
| JSON, 721,334 bytes | `2aaa615208ee961dc2502bfd6d63bd72de74e7e269d0607803b929ad584f7934` |
| Controls CSV | `885eb26127a208e3cce2b37723a1ee1836ea8bab3e6274cad9acfc606b99ca59` |

Corrected **v2** outputs `/tmp/bbsim-source-detail-scope/verification-first-{events.csv,report.json,controls.csv}` add angles only; fresh verification status is in the ledger:

| Artifact | SHA-256 |
| --- | --- |
| Event CSV,35,059,942 bytes | `ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a` |
| JSON,724,035 bytes | `7685b4de62f69e6bacf09e8ce366b767dc13be3205b1dc064f7c4eff5e326aef` |
| Controls CSV,864 bytes,unchanged | `885eb26127a208e3cce2b37723a1ee1836ea8bab3e6274cad9acfc606b99ca59` |

JSON retains manifest/input/query hashes, event-grain accounting, raw tags, literal derived flags, context/position distributions, supplied opportunity units and deterministic lowest-event-key examples for every observed signature. There are 100 result/trajectory/out/DP/TP/GIDP signatures, 1,090 before-state/out contexts and 239 result/trajectory/position cells. These partition events, **not individual retirement mechanisms**.

## Accounting and denominators

2,429 games; 222,047 source events minus 30,022 no-play rows = **192,025 real events**, partitioned into **186,517 PA** and **5,508 non-PA**.

- Event/state/PA batting outs and fielding putouts agree on every applicable exported row. Summed actual outs = putouts = **130,270**.
- Event/state/supplied runner runs agree on every row and sum to **23,467**.
- Supplied team/event fielding credits: **43,968 assists**, **2,836 errors**. These do not identify individual credited fielders or defensive opportunities.
- Actual outs per event: **65,807 zero**, **122,169 one**, **4,046 two**, **3 three**. Separately, recorded DP flags = **4,034**, TP flags = **3**, GIDP units = **3,463**. Out multiplicity and official statistical labels are different dimensions.
- **43,551 real frame-ending events** have the documented nullable after-state convention. Original whole-event audit's 43,552 includes a no-play row; neither number is missing-play evidence. Active after-states cannot silently be absent.
- **72,447 events have no supplied runner rows**. Their supplied credit/exposure units are zero; this is not a universal claim about complete runner-history observability.

Observed PA-result vocabulary, without the preview's exclusions/coarsening:

| Result | Events | Result | Events |
| --- | ---: | --- | ---: |
| InPlayOut | 79,825 | StrikeOut | 42,823 |
| Single | 25,947 | Walk | 15,142 |
| Double | 8,117 | HomeRun | 6,763 |
| HitByPitch | 1,984 | ReachedOnError | 1,470 |
| SacrificeFly | 1,143 | Triple | 785 |
| IntentionalWalk | 753 | SacrificeHit | 721 |
| FieldersChoice | 556 | GroundRuleDouble | 414 |
| Interference | 61 | InsideTheParkHomeRun | 13 |

Batted metadata covers **125,754 events**. The cached `balls_in_play` flag selects **118,577**: it excludes 6,763 `HomeRun` and 414 `GroundRuleDouble` events, while the 13 inside-the-park HR remain included. Do not silently omit ground-rule doubles or apply a BIP denominator to all batted contact.

## Contact and out evidence

For the **79,825 InPlayOut-labeled events only** (not sacrifices, hits or all individual outs):

| Recorded trajectory | Zero outs | One out | Two outs | Three outs |
| --- | ---: | ---: | ---: | ---: |
| GroundBall | 75 | 35,147 | 3,453 | 3 |
| GroundBallBunt | 0 | 266 | 14 | 0 |
| Fly | 0 | 30,464 | 4 | 0 |
| LineDrive | 0 | 7,722 | 1 | 0 |
| LineDriveBunt | 0 | 2 | 0 | 0 |
| PopUp | 0 | 1,556 | 3 | 0 |
| PopUpBunt | 0 | 147 | 1 | 0 |
| Unknown | 3 | 583 | 381 | 0 |

A result label is not a fixed transition:

- StrikeOut: **92 zero-out**, **42,568 one-out**, **163 two-out** events.
- FieldersChoice: **268 zero-out**, **280 one-out**, **8 two-out** events.
- SacrificeFly: **1,132 one-out**, **11 two-out** events.
- SacrificeHit: **1 zero-out**, **717 one-out**, **3 two-out** events.
- ReachedOnError: **1,457 zero-out**, **13 one-out** events.
- All **78 zero-out InPlayOut events have error credits**. Preserve the source label and actual delta; do not automatically reinterpret these as a clean batter retirement or rewrite them as ROE without the detailed play facts.

Representative keyed evidence in the external JSON:

| Game / event / event key | Supplied evidence, not reconstructed chronology |
| --- | --- |
| `ARI201906040` / 9 / `1254607149` | InPlayOut, GroundBall, zero outs, error credit |
| `ANA201904040` / 96 / `1254090096` | StrikeOut, zero outs, passed-ball runner advance |
| `ANA201904080` / 25 / `1254091045` | StrikeOut, two outs, caught-stealing and DP flags |
| `CIN201909020` / 24 / `1257677364` | StrikeOut, two outs, no DP flag |
| `HOU201905220` / 19 / `1259705629` | GroundBall, three outs, TP flag, no GIDP unit |

A separate bounded census `/tmp/bbsim-outcomes-mechanism-checks.json` found zero K-zero-out events with first occupied below two outs, zero GIDP units without first occupied/below two outs, and zero DP/TP flags with non-two/non-three out counts. These consistency checks do **not** establish catching, throw order, retirement cause or complete legality.

### Unknowns and literal source-derived flags

Among 118,577 source-defined BIP:

| Field | Recorded known | Recorded unknown/missing | Derived known flag |
| --- | ---: | ---: | ---: |
| Trajectory | 115,914 | 2,663 | 116,859 |
| General location | 113,825 | 4,752 | 117,408 |
| Fielded-by position tag | 118,554 | 23 | 118,554 |

Depth is also explicitly unknown/missing for4,752 BIP; the other tags must not all be interpreted as measured depth. The source documents `Default` depth/angle as ambiguous between default location and unrecorded detail. Across all192,025 real events, raw angles are Foul2,593/FoulLine6,959/Default79,073/Left8,889/Middle14,898/Right8,581/Unknown4,761/absent66,271. FoulLine is not Foul. The Foul-angle rows include four two-out events and two recorded GroundBalls: location tags alone cannot establish individual catches or fair/foul rulings. **945 BIP have a derived-known trajectory flag while the recorded tag remains unknown**; those raw tags stay unknown in the inventory.

The four narrow trajectory flags are **not an exhaustive partition of source-known trajectory**. **1,126 known BIP have none of those flags**: recorded bunt subtypes. The separate statistical `bunts` field totals only **944**, not all 1,126 recorded bunt tags. A draft CASE classifier falsely collapsed part of this into Unknown. The accepted export therefore preserves each literal source-derived flag instead of inventing a unified categorical trajectory. `/tmp/bbsim-outcomes-derived-flags.csv` retains the cross-tab evidence. The [pinned follow-up](2019-source-detail-scope.md) re-inspected these definitions: bunt subtypes belong to a separate seed class, and broad inferred AirBall takes precedence in some cases before the statistical bunt flag. Exact published-row derivation remains to reconcile; literal flags and raw tags stay separate.

## Runner evidence and opportunities

Supplied runner credit units: outs **9,522**, unforced-out statistic **1,190**, steals **2,280**, CS **832**, pickoffs **318**, WP advancement **2,340**, PB advancement **503**, balk advancement **191**, defensive indifference **244**, unspecified advancement **27**, error advancement **1,045**. Advancement units are not counts of pitcher WP/catcher PB/balk occurrences. Individual `running.runner_id`/before-base role and credit rows remain in the cache; the inventory aggregates them before joining, so it cannot fan out events.

Simultaneous effects exist: **128 PA events** carry WP advancement credits, **27 PA events** PB credits, **205 PA events** steal credits and **145 PA events** CS credits. Counts are events here, not the unit totals above. Do not restrict these effects to separately sampled runner-only events.

Cached derived exposure units are available, not wholly absent:

| Supplied statistic | Units | Why this is not yet a validated rate denominator |
| --- | ---: | --- |
| `times_force_on_runner` | 148,337 | Force-pressure/exposure statistic, not observed force retirements. Do not subtract unforced outs to assert every force/tag cause. |
| `stolen_base_opportunities` | 71,961 | Source-defined actor-event opportunities; pitch-wise/manager-choice eligibility and derivation need verification. |
| `extra_base_chances` | 15,604 | Derived extra-base chance statistic, not every specific OF throw or eligible arm opportunity. |

JSON includes these units by result/recorded trajectory/before outs/base mask/actual outs, with runner-row and missing-row counts. No opportunity rate, smoothing choice or model is fitted. Fielding PO/A/E incidence still does not supply a full eligible fielder range/error/arm denominator.

## Capability matrix

Grades concern the **existing cache**, not every possible baseball event or the entire upstream publication.

| Mechanism | Grade | Available / missing |
| --- | --- | --- |
| Observed PA vocabulary and actual out/run counts | supported | Full observed partition and supplied accounting; not proof of universal legal-category coverage. |
| Ground/fly/line/pop/bunt contact with result/context | partial | Recorded and derived evidence, known gaps; detailed catch/muff/throw/retirement facts are not implied by tags. |
| Fair/foul catch, foul pop/line out, infield-fly detail | partial location evidence; explicit mechanisms unverified | Cached angle includes2,593 Foul and6,959 FoulLine tags, omitted in v1 and now preserved. Foul territory does not universally establish legal foul status; no complete catch/infield-fly modifier/retirement history in this snapshot. |
| Hits, ground-rule double, inside/outside-park HR | supported for recorded classifications | Types are distinguishable; exact paths, attempts and throws still partial. |
| Strikeout variants and K-plus-runner effects | partial | Actual outs, runner credits/context; no complete caught/uncaught/called/swinging/recovery chronology. |
| FC, force/tag/appeal outs | partial / explicit cause unavailable | Context, identities and basepath-out/derived exposure credits; no explicit retirement base/cause/order or complete appeals/fourth-out history. |
| DP/TP/GIDP | partial | Flags, real multiplicity and context; no complete ordered named retirement/throw chain. |
| Sacrifices, ROE and simultaneous runner outs | partial | Labels, error/out/run credits and context; detailed qualifying causes require play facts. |
| SB/CS/pickoff/WP/PB/balk/indifference | partial | Actor credit/exposure rows and before/after context; no complete causal episode/timing or verified policy denominator. |
| Individual PO/A/E and involved defenders | partial | Aggregate team/event credits and first-fielded position; opaque personnel key without cached defensive mapping/individual credit actors. |
| Range/error conversion and OF/catcher arm opportunities | unavailable as complete denominators | No full eligible individual fielder/throw opportunities or tracking/range/arm evidence; credits alone cannot identify skill. |

## Smallest next data supplement, before modeling

The [pinned source inspection and bounded supplement contract](2019-source-detail-scope.md) now scopes a **2019-only** supplement containing raw play text/modifiers or equivalent structured facts: fair/foul/contact modifiers, batter/runner origin/destination and retirement base/cause/order, scoring versus last-out timing, and fielding credit/throw sequences. Include defensive personnel-key mappings and relevant substitutions so position tags can resolve to actual eligible actors. Verify joins, transfer scope, provenance/attribution, unknowns and the 78 error-bearing zero-out InPlayOut examples before fitting a transition model.

This is a proposed follow-up, **not an implemented extraction or authorization to download/publish data**. It may resolve factual sequences but will not manufacture tracking-quality range/arm information. Keep necessary priors/uncertainty explicit. Do not assign a final hit and then independently roll an out. Preregister conditional mechanism/held-out checks before fitting; no calibration, full box, G1/G2 or release signoff follows from this census.

## Verification and preserved failures

Preregistered acceptance remains in [PROGRESS](../PROGRESS.md). `/tmp/bbsim-outcomes-{interface-red,semantic-red}.log` preserve the missing-module then six-method stub failures. New fixture initially expected two missing fielders while supplying one known fielder; corrected only that fixture and added a known-fielder control, not a weaker assertion. Native regression `/tmp/bbsim-outcomes-sql-null-red.log` witnessed SUM hiding one NULL runner-credit row beside a zero row; SQL now rejects missing supplied units before aggregation. Bunt regression/evidence `/tmp/bbsim-outcomes-bunt-red.log` and `derived-flags.csv` led to literal flags, not a recategorized model. CSV default-NULL display failure and `/tmp/bbsim-outcomes-null-display.csv` retained; corrected explicit export display convention, not input facts.

Fresh complete `bash /tmp/bbsim-outcomes-full-checks.sh`, managed `proc_c419`, exit 0: **548 Rust / 83 Python** (ResourceWarning-as-error), fmt/build/strict all-target Clippy/py_compile/diff; 69 original audit metrics, 5 fixture transitions, 54 fixture comparisons, 27 shared-upstream ER and 54 external box comparisons; native 1,350 cases; unchanged baseline; 2,000 actual-baseline seeded games/100,000 draws (exploratory, not accuracy); two byte-identical outcome/control exports and JSONs; all 19 control metrics/17 result cells and event/opportunity partitions; unchanged database and frozen four audit/test files. Complete stdout/stderr consumed at `/var/folders/d0/z9jph2ld4v9gw45bwg0f1j900000gn/T/pi-processes-YoEAQg/proc_c419-{stdout,stderr}.log`. Expected injected FileNotFoundError/HTTP500 is a passing negative test. No independent reviewer or fresh season-wide official comparison was run.

**Fresh v2 correction verification:** `bash /tmp/bbsim-source-detail-scope/full-checks.sh`, managed `proc_55af`, exit0:548 Rust/84 Python ResourceWarning-as-error plus the same full applicable checks/reconciliations/diagnostics above; three changed audit/test hashes frozen, repeated v2 CSV/JSON/controls byte-exact, all pre-existing CSV/example/JSON facts unchanged, original database unchanged. A separate literal cached-angle census matches the complete v2 distribution. `angle-semantic-red.log` retains two genuine missing-distribution failures. Earlier `proc_b193` passed suites but failed a new comparison harness on examples because they correctly gained angle; the harness now compares every prior field while preserving the declared new angle, with no implementation/test weakening. Both runs' complete raw logs remain under the same pi-processes directory; final717 stdout/170 stderr lines consumed. Original v1 artifacts/hashes above remain unchanged. Engine/model/UI/RNG and previews unchanged/stopped.
