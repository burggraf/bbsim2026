-- 2019-outcomes-v1: read-only offline F-03 inventory, not a fitted model.
-- duckdb -readonly -csv -nullvalue '' SNAPSHOT < this file > EVENTS.csv (outside Git).
-- Event-grain source facts; aggregate runner credit rows BEFORE joining.
SET enable_external_access=FALSE;
WITH guard AS (
 SELECT CASE WHEN
  (SELECT count(*) FROM audit_manifest) <> 1
  OR (SELECT catalog_sha256 FROM audit_manifest LIMIT 1) IS DISTINCT FROM
     '2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583'
  OR EXISTS(SELECT 1 FROM events WHERE season IS DISTINCT FROM 2019 OR no_play_flag IS NULL)
  OR (SELECT count(*) FROM events) <> (SELECT count(DISTINCT event_key) FROM events)
  OR (SELECT count(*) FROM states) <> (SELECT count(DISTINCT event_key) FROM states)
  OR (SELECT count(*) FROM batting) <> (SELECT count(DISTINCT event_key) FROM batting)
  OR (SELECT count(*) FROM fielding) <> (SELECT count(DISTINCT event_key) FROM fielding)
  OR (SELECT count(*) FROM batted) <> (SELECT count(DISTINCT event_key) FROM batted)
  OR (SELECT count(*) FROM running) <> (SELECT count(*) FROM (SELECT DISTINCT event_key,runner_id FROM running))
  OR EXISTS(SELECT 1 FROM states s LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
  OR EXISTS(SELECT 1 FROM batting b LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
  OR EXISTS(SELECT 1 FROM fielding f LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
  OR EXISTS(SELECT 1 FROM batted b LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
  OR EXISTS(SELECT 1 FROM running r LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
  -- SUM ignores NULL: reject missing supplied units before they can disappear in aggregation.
  OR EXISTS(SELECT 1 FROM running WHERE runner_id IS NULL OR runner_id=''
    OR runs IS NULL OR outs_on_basepaths IS NULL OR unforced_outs_on_basepaths IS NULL
    OR stolen_bases IS NULL OR caught_stealing IS NULL OR picked_off IS NULL
    OR advances_on_wild_pitches IS NULL OR advances_on_passed_balls IS NULL
    OR advances_on_balks IS NULL OR advances_on_unspecified_plays IS NULL
    OR advances_on_defensive_indifference IS NULL OR advances_on_errors IS NULL
    OR times_force_on_runner IS NULL OR stolen_base_opportunities IS NULL OR extra_base_chances IS NULL)
 THEN error('outcome inventory: invalid scope, provenance, orphan, grain or missing supplied units')
 ELSE TRUE END AS ok
), runner_units AS (
 SELECT event_key,count(*) AS runner_rows,sum(runs) AS runner_runs,
  sum(outs_on_basepaths) AS runner_outs,sum(unforced_outs_on_basepaths) AS unforced_runner_outs,
  sum(stolen_bases) AS steals,sum(caught_stealing) AS cs,sum(picked_off) AS pickoffs,
  sum(advances_on_wild_pitches) AS wp,sum(advances_on_passed_balls) AS pb,
  sum(advances_on_balks) AS balk,sum(advances_on_unspecified_plays) AS unspecified,
  sum(advances_on_defensive_indifference) AS indifference,sum(advances_on_errors) AS error_advances,
  sum(times_force_on_runner) AS force_exposure,sum(stolen_base_opportunities) AS steal_opportunities,
  sum(extra_base_chances) AS extra_base_chances
 FROM running GROUP BY event_key
)
SELECT e.event_key,e.game_id,e.event_id,e.season,e.plate_appearance_result AS result,
 b.plate_appearances AS pa,e.batted_trajectory AS trajectory,
 e.batted_location_general AS location,e.batted_location_depth AS depth,
 e.batted_location_angle AS angle,
 e.batted_to_fielder AS fielder,
 -- Preserve literal flags: the narrow categories/bunt statistic do not partition all known trajectories.
 c.trajectory_known AS derived_known,c.trajectory_unknown AS derived_unknown,
 c.trajectory_fly_ball AS derived_fly,c.trajectory_ground_ball AS derived_ground,
 c.trajectory_line_drive AS derived_line,c.trajectory_pop_up AS derived_pop,c.bunts AS derived_bunts,
 c.batted_location_known AS derived_location_known,c.fielded_by_known AS derived_fielder_known,
 c.balls_in_play AS bip,e.outs AS outs_before,e.base_state AS base_mask,
 e.outs_on_play AS outs,e.runs_on_play AS runs,s.outs_on_play AS state_outs,
 s.runs_on_play AS state_runs,b.outs_on_play AS batting_outs,
 f.putouts,f.assists,f.errors,f.double_plays AS dp,f.triple_plays AS tp,
 b.grounded_into_double_plays AS gidp,s.frame_end_flag AS frame_end,
 s.outs_end AS after_outs,s.base_state_end AS after_base_mask,
 e.batter_id,e.pitcher_id,s.runner_first_id_start AS first,
 s.runner_second_id_start AS second,s.runner_third_id_start AS third,
 f.personnel_fielding_key AS personnel_key,
 coalesce(r.runner_rows,0) AS runner_rows,r.runner_runs,r.runner_outs,r.unforced_runner_outs,
 r.steals,r.cs,r.pickoffs,r.wp,r.pb,r.balk,r.unspecified,r.indifference,r.error_advances,
 r.force_exposure,r.steal_opportunities,r.extra_base_chances
FROM events e
LEFT JOIN batting b USING(event_key)
LEFT JOIN states s USING(event_key)
LEFT JOIN fielding f USING(event_key)
LEFT JOIN batted c USING(event_key)
LEFT JOIN runner_units r USING(event_key)
CROSS JOIN guard
WHERE e.no_play_flag=FALSE AND guard.ok
ORDER BY e.event_key;
