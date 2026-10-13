# Observed 2019 outcomes versus current engine support

## Scope and preregistration

F-01 follow-up after the accepted [raw-mechanism inventory](../data-audits/2019-raw-mechanisms.md),
commit `668fe61`. Preregistered in [PROGRESS.md](../PROGRESS.md) before measurements:
read constructors/accounting/preview callers; conserve the disjoint real PA/non-PA
cohorts and before contexts; distinguish physical representation, required causal
facts, statistical coverage and simulated assumptions. Documentation only; no
parser, engine, generator, RNG, rating or UI changes. No new downloads, dependencies,
production-model selection, G1/G2 signoff or redistribution permission.

Inputs are the accepted supplement SHA-256
`554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82` and the unchanged
original cache. Pinned source/snapshot and attribution remain in the
[supplement report](../data-audits/2019-outcome-details.md). Native read-only queries
aggregate each child before joining; zero coalescing denotes an absent child
**record count**, never an unknown supplied value. Missing runner endpoints are
counted separately. No-play rows are excluded rather than assigned statistics.

## Three different meanings of support

1. `apply`/`EventTransition::try_new` validates a supplied physical/scorer event.
   It does not recover the event from a source string or certify all statistics.
2. Named causal facts and whole-chance review qualify event-local contributions.
   Correct supplied PO/A totals and `NoMisplay` alone do not qualify all fielding.
3. A complete game still needs the [approved accounting matrix](../engine/g1-accounting-scope.md).
   Historical missing facts stay unknown. For a simulated play, a documented,
   versioned policy may generate causal history; that is not recovered history.

Actual inspected paths: [event facts/getters/constructor](../../engine/src/lib.rs),
[caught-K validation](../../engine/src/caught.rs), [accounting](../../engine/src/accounting.rs),
[preview producer](../../engine/examples/support/preview.rs), and
[preview contract](generated-preview.md). `fielding_contribution()` currently
unions ordinary ground throw, caught K, sole safe hit, protected award, sole
out-of-park HR and reviewed ground muff. There is **no ordinary caught-air fact
or qualified caught-air PO/A contribution**. `clean_play` also does not qualify a
generic non-K air out merely because it has `NoMisplay` and a putout.

| Observed family | Real source events | Current preview / engine distinction |
| --- | ---: | --- |
| K |42,823| Preview declares a direct caught K with held runners; source includes zero-out and compound/two-out K. Credits alone do not prove direct receipt/recovery history. |
| Ordinary BB / HBP |15,142 /1,984| Preview uses the protected force chain. Extra action needs separately supplied delivery/advance facts; no blanket exclusive-award inference. |
| Intentional BB / interference |753 /61| Excluded from preview denominator. Engine classifications/awards are not a universal interference-history or complete-fielding proof. |
|1B /2B / ground-rule2B /3B |25,947 /8,117 /414 /785| Preview uses station advancement and pools the doubles. Explicit hit value is available; exclusive safe-hit facts cannot be attached to a hit plus retirement/error/independent action. |
| Out-of-park / inside-park HR |6,763 /13| Preview pools both as out-of-park. Do not attach exclusive out-of-park facts to historical inside-park action. |
| ROE |1,470| Preview generates only a bound station-one ground muff, not every historical throwing/recovery/error history. Unknown earlier chances and richer ER histories remain visible. |
| In-play out / FC / SH / SF |79,825 /556 /721 /1,143| Preview collapses all to one ground throw, runners held. Generic physical events and bounded two-out/FC rules exist, but do not establish catch/tag-up/sacrifice/multi-out history or complete statistics. |
| Non-PA |5,508| Engine has runner/pitch/personnel event vocabulary; preview emits none. Source lexical atoms/labels are not occurrence/opportunity rates or sufficient clocks. |

These disjoint counts conserve **192,025 real events =186,517 PA +5,508 non-PA**.
The nine preview categories still contain185,703 included PA after753 IBB and61
interference exclusions. No categories/probabilities were fitted or changed here.
Every before base mask0–7/outs0–2 is valid in the source; the grouped context table
also conserves130,270 physical outs and23,467 runs. Contact/foul tags remain supplied,
not retagged from outcome or location.

## Smallest high-volume candidate: ordinary caught air, runners held

Before running the query, define a **source credit-shape proxy**, not a catch
classifier: `InPlayOut`, supplied trajectory `Fly`/`LineDrive`/`PopUp`, one out,
zero runs, one supplied PO row, zero supplied Assist/Error rows, zero non-batter
out rows, zero listed non-batter destination changes/missing endpoints, zero
sparse flag rows. Absence of a listed change is **not proof that runners held
throughout**; absent flags do not prove absence of interference or a missed chance.
The proxy is a prioritization count, not a historical import certificate.

-39,750 non-bunt air-tagged in-play outs:30,468 Fly,7,723 LineDrive,1,559 PopUp.
-38,552 match the simple credit shape, spanning all24 before mask/out cells.
-1,198 do not:1,190 one-out events and8 two-out events. Preserve these separately.
  Tag-up advances, assist/deflection chains, infield-fly/other flags and multi-outs
  cannot be silently folded into the simple catch.
- Bunt trajectories, sacrifice-result rows, unknown/missing trajectory and
  other results remain outside this proxy, not legally impossible.

Keyed controls (game/event/source key):

| Key | Literal play / supplied facts | Consequence |
| --- | --- | --- |
| ANA201904040/22/1254090022 | `7/F7D`, empty/two outs, Fly, PO1/A0/E0 | Simple proxy; synthetic ordinary catch needs a named catching fielder and explicit held-runner/catch history. |
| ANA201904040/37/1254090037 | `8/L8RD+`, mask6/two outs, LineDrive, PO1/A0/E0 | Literal terminal '+' is a contact modifier, not an extra play; source order is not a clock. |
| ANA201904050/46/1254090301 | `5/P`, mask1/two outs, PopUp, angle Unknown | Unknown angle stays Unknown. Not permission to infer fair/foul or catch certainty from location. |
| ANA201904040/75/1254090075 | `4/IF/P4D-`, mask3/no outs, InfieldFlyRule | Excluded flagged shape; automatic infield-fly retirement is not an ordinary catch declaration. |
| ANA201904050/63/1254090318 | `343/L3S+.1-2`, mask1/one out, Assist rows2, listed runner change | A line-drive tag does not imply direct receipt by a single fielder. |
| ANA201904070/33/1254090798 | `9/F9D.2-3`, mask3/one out, listed runner change | A one-out fly still needs advancement/touch history; holding runners would change the event. |

### Candidate implementation contract, not implementation or model approval

A concrete next main-path prerequisite would be a named **ordinary direct
caught-air** fact (non-bunt Fly/Line/Pop), batter retired, before runners held,
no other touch/throw/retirement/action, explicit whole-chance review. Validate
actor/eligible defender, PA/AB/no-hit, one out/no runs, exact named PO and zero
assists under reviewed-clean scope; preserve generic partial events when facts
are missing. Reject contradictory ground/K/compound facts rather than fabricate
history. Round-trip the immutable record and cover all24 contexts/both halves,
third-out endings, move/credit-order permutations and wrong/omitted-fact cases.
Any clean-ER qualification must be independently tested, without extending the
deferred two-out-prefix error reconstruction. No tag-ups, sacrifices, DP, IF,
range/arm or source parser completeness are smuggled into this primitive.

Only after that prerequisite should a separately preregistered experimental
contact policy decide how to sample joint contact/catch/advancement history.
Preserve category/RNG/version evidence and report changes/coarsenings. Never
sample a hit then independently roll to convert it to an out. Marginal contact
counts are not player opportunities or a fair-ball/catch probability model.
D-09's production taxonomy/positive-support requirement, full accounting,
manager policy and conditional/held-out accuracy gates remain open.

## Reproduction and verification

Copy the following into an external query file. Native stdout exports preserve
all four result tables (assertion, context census, air groups, keyed examples).

```sql
SET enable_external_access=FALSE;
CREATE TEMP VIEW credits AS SELECT event_key,
 count(*) FILTER(WHERE fielding_play='Putout') po,
 count(*) FILTER(WHERE fielding_play='Assist') assist_rows,
 count(*) FILTER(WHERE fielding_play='Error') errors
 FROM stg_event_fielding_plays GROUP BY event_key;
CREATE TEMP VIEW runners AS SELECT event_key,
 count(*) FILTER(WHERE baserunner!='Batter' AND is_out) runner_outs,
 count(*) FILTER(WHERE baserunner!='Batter' AND base_end IS NOT NULL AND base_end!=baserunner) listed_changes,
 count(*) FILTER(WHERE baserunner!='Batter' AND base_end IS NULL) missing_endpoints
 FROM stg_event_baserunners GROUP BY event_key;
CREATE TEMP VIEW flags AS SELECT event_key,count(*) flag_rows FROM stg_event_flags GROUP BY event_key;
CREATE TEMP VIEW context AS SELECT e.*,
 coalesce(c.po,0) po,coalesce(c.assist_rows,0) assist_rows,coalesce(c.errors,0) errors,
 coalesce(r.runner_outs,0) runner_outs,coalesce(r.listed_changes,0) listed_changes,
 coalesce(r.missing_endpoints,0) missing_endpoints,coalesce(f.flag_rows,0) flag_rows
 FROM selected_events e LEFT JOIN credits c USING(event_key)
 LEFT JOIN runners r USING(event_key) LEFT JOIN flags f USING(event_key)
 WHERE NOT no_play_flag;
SELECT CASE WHEN count(*)=192025 AND count(DISTINCT event_key)=192025
 AND count(*) FILTER(WHERE plate_appearance_result IS NOT NULL)=186517
 AND count(*) FILTER(WHERE plate_appearance_result IS NULL)=5508
 AND count(*) FILTER(WHERE base_state IS NULL OR base_state>7 OR outs IS NULL OR outs>2)=0
 THEN TRUE ELSE error('Real-event cohort/context conservation failed') END verified FROM context;
SELECT base_state,outs,plate_appearance_result,batted_trajectory,batted_location_angle,
 outs_on_play,runner_outs,listed_changes,missing_endpoints,count(*) events,
 sum(runs_on_play) runs FROM context GROUP BY ALL ORDER BY ALL;
CREATE TEMP VIEW air AS SELECT *,
 outs_on_play=1 AND runs_on_play=0 AND po=1 AND assist_rows=0 AND errors=0
 AND runner_outs=0 AND listed_changes=0 AND missing_endpoints=0 AND flag_rows=0 simple_credit_shape
 FROM context WHERE plate_appearance_result='InPlayOut'
 AND batted_trajectory IN('Fly','LineDrive','PopUp');
SELECT simple_credit_shape,batted_trajectory,base_state,outs,outs_on_play,
 count(*) events,sum(runs_on_play) runs FROM air GROUP BY ALL ORDER BY ALL;
SELECT a.event_key,a.game_id,a.event_id,base_state,outs,batted_trajectory,batted_location_angle,
 outs_on_play,runs_on_play,po,assist_rows,errors,runner_outs,listed_changes,missing_endpoints,flag_rows,s.raw_play
 FROM air a JOIN stg_event_audit s USING(event_key)
 QUALIFY row_number() OVER(PARTITION BY simple_credit_shape,batted_trajectory ORDER BY a.event_key)=1
 ORDER BY a.event_key;
```

External artifacts: `/tmp/bbsim-outcome-support/{contexts.sql,air.sql,air-query.sql,contexts.csv,air.csv}`.
Execute native queries with `/opt/homebrew/bin/duckdb -readonly -bail -batch -init /dev/null`
against the supplement, CSV/header stdout redirection, `SET enable_external_access=FALSE`.
Independent child aggregates count supplied PO/Assist/Error, non-batter outs,
non-batter `base_end != baserunner`, NULL endpoints and flags by event key before
joining. Group real events by base mask, before outs, result, trajectory, angle,
play outs, runner outs, listed changes and missing endpoints. This supports an
exact conditional census without claiming legal mechanism classification.

Fresh queries, repeats and the verbatim documented-query run exit0, five empty
stderr files; CSVs byte-identical;
input hashes unchanged. Full rows parsed with exact column widths; disjoint
cohort/outs/runs conservation and separately grouped air totals agree. Context
CSV116,050 bytes/SHA `abf32c76695578d060dcbbdbf0c8396920407df20db6125cf661c13dc76ae297`;
air-query CSV119,682 bytes/SHA `385f718d6854e5c4c10ea5fb915ce51f77136edcb61d4881abed2341e99c76f9`.
Contexts SQL SHA `817c38243cbfb760cc72ddf4ba4fd0219544b2d4fb0b709c3f56c45f552c33b7`;
air suffix SQL SHA `1e1587351068caf3afc0939b179ae662849e26e280aeb25ec781e872ee4b2d49`.
The air-query CSV intentionally retains the preceding context/control tables;
no source rows or diagnostics are filtered away.

No executable behavior changed. Prior full checks548 Rust/124 Python are the
accepted inventory's evidence, **not a fresh suite for this documentation**.
Fresh document ownership/link/diff/index/frozen acceptance is recorded in the
ledger. Raw artifacts remain outside Git; this is internal observed support,
not in-sample calibration, official historical scoring, public data or release
rights clearance.
