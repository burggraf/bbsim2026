-- Local factual checks only. Caller supplies source_events, selected_games/events,
-- the five stg_* detail tables, independently aggregated expected_fielding
-- and expected_running from the cached real-event actor/statistics table.
-- Attach native local databases before disabling external access. No remote reads.
CREATE TEMP TABLE detail_assertions AS
SELECT CASE WHEN NOT EXISTS (
 (SELECT * FROM source_events EXCEPT ALL SELECT * FROM selected_events)
 UNION ALL (SELECT * FROM selected_events EXCEPT ALL SELECT * FROM source_events)
) THEN TRUE ELSE error('Source/cached event columns differ; reconcile explicitly') END AS identity_ok,
CASE WHEN NOT EXISTS (
 SELECT event_key FROM selected_events GROUP BY event_key HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT game_id,event_id FROM selected_events GROUP BY game_id,event_id HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT 1 FROM selected_events WHERE event_key IS NULL OR game_id IS NULL OR event_id IS NULL
 OR no_play_flag IS NULL OR batting_side NOT IN ('Home','Away') OR batting_side IS NULL
 OR outs_on_play IS NULL OR runs_on_play IS NULL
) THEN TRUE ELSE error('Invalid selected event identity/state') END AS selected_ok,
CASE WHEN NOT EXISTS (
 SELECT event_key FROM stg_event_audit GROUP BY event_key HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT 1 FROM selected_events e FULL JOIN stg_event_audit a USING(event_key)
 WHERE e.event_key IS NULL OR a.event_key IS NULL
 OR e.game_id IS DISTINCT FROM a.game_id OR e.event_id IS DISTINCT FROM a.event_id
 OR a.raw_play IS NULL OR trim(a.raw_play)='' OR a.filename IS NULL OR trim(a.filename)=''
 OR a.line_number IS NULL
) THEN TRUE ELSE error('Missing/duplicate/blank/mismatched raw audit') END AS raw_ok;

CREATE TEMP TABLE child_assertions AS
SELECT CASE WHEN NOT EXISTS (
 SELECT event_key,sequence_id FROM stg_event_flags GROUP BY ALL HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT 1 FROM stg_event_flags f LEFT JOIN selected_events e USING(event_key)
 WHERE e.event_key IS NULL OR f.sequence_id IS NULL OR f.sequence_id<1
 OR f.flag IS NULL OR trim(f.flag)=''
) THEN TRUE ELSE error('Invalid sparse flag grain/identity') END AS flags_ok,
CASE WHEN NOT EXISTS (
 SELECT event_key,baserunner FROM stg_event_baserunners GROUP BY ALL HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT 1 FROM stg_event_baserunners r LEFT JOIN selected_events e USING(event_key)
 WHERE e.event_key IS NULL OR e.game_id IS DISTINCT FROM r.game_id
 OR e.event_id IS DISTINCT FROM r.event_id OR r.runner_id IS NULL OR trim(r.runner_id)=''
 OR r.baserunner IS NULL OR r.baserunner NOT IN ('Batter','First','Second','Third')
 OR r.is_out IS NULL OR r.run_scored_flag IS NULL
 OR (r.baserunner='Batter' AND r.runner_id IS DISTINCT FROM e.batter_id)
) THEN TRUE ELSE error('Invalid baserunner grain/identity') END AS runners_ok,
CASE WHEN NOT EXISTS (
 SELECT event_key,sequence_id FROM stg_event_fielding_plays GROUP BY ALL HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT 1 FROM stg_event_fielding_plays f LEFT JOIN selected_events e USING(event_key)
 WHERE e.event_key IS NULL OR e.game_id IS DISTINCT FROM f.game_id
 OR e.event_id IS DISTINCT FROM f.event_id OR f.sequence_id IS NULL OR f.sequence_id<1
 OR f.fielding_position IS NULL OR f.fielding_position NOT BETWEEN 0 AND 9
 OR f.fielding_play IS NULL OR f.fielding_play NOT IN ('Putout','Assist','Error','FieldersChoice')
) THEN TRUE ELSE error('Invalid fielding credit grain/identity') END AS credits_ok,
CASE WHEN NOT EXISTS (
 SELECT game_id,player_id,fielding_position,start_event_id
 FROM stg_game_fielding_appearances GROUP BY ALL HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT 1 FROM stg_game_fielding_appearances a LEFT JOIN selected_games g USING(game_id)
 WHERE g.game_id IS NULL OR a.player_id IS NULL OR trim(a.player_id)=''
 OR a.side IS NULL OR a.side NOT IN ('Home','Away') OR a.fielding_position IS NULL
 OR a.fielding_position NOT BETWEEN 0 AND 12 OR a.start_event_id IS NULL
 OR a.end_event_id IS NULL OR (a.start_event_id>a.end_event_id AND NOT (
  a.end_event_id=(SELECT max(e.event_id) FROM selected_events e WHERE e.game_id=a.game_id)
  AND a.start_event_id=a.end_event_id+1
 ))
) THEN TRUE ELSE error('Invalid fielding appearance grain/range') END AS appearances_ok;

-- The source personnel/statistics lookup excludes no-play markers. Raw no-play
-- runners are retained; do not manufacture absent derived rows or drop history.
CREATE TEMP VIEW real_raw_runners AS SELECT r.* FROM stg_event_baserunners r
 JOIN selected_events e USING(event_key) WHERE NOT e.no_play_flag;
CREATE TEMP TABLE runner_assertions AS
SELECT CASE WHEN NOT EXISTS (
 SELECT event_key,baserunner FROM expected_running GROUP BY ALL HAVING count(*)<>1
) AND NOT EXISTS (
 SELECT 1 FROM real_raw_runners r FULL JOIN expected_running c USING(event_key,baserunner)
 WHERE r.event_key IS NULL OR c.event_key IS NULL
 OR r.runner_id IS DISTINCT FROM c.runner_id
 OR r.run_scored_flag::INTEGER IS DISTINCT FROM c.runs
 OR r.is_out::INTEGER IS DISTINCT FROM c.outs_on_basepaths
) THEN TRUE ELSE error('Real-event runner roles/actors/units do not reconcile') END AS runners_ok;

-- Source-defined assist segments end at a Putout or Error, not physical throws.
CREATE TEMP VIEW credit_init AS
SELECT *,count(*) FILTER (WHERE fielding_play IN ('Putout','Error')) OVER (
 PARTITION BY event_key ORDER BY sequence_id ROWS UNBOUNDED PRECEDING
) AS group_id_init FROM stg_event_fielding_plays;
CREATE TEMP VIEW credit_segments AS
SELECT *,lag(group_id_init,1,0) OVER (PARTITION BY event_key ORDER BY sequence_id) AS group_id
FROM credit_init;
CREATE TEMP VIEW credit_totals AS
SELECT event_key,count(*) FILTER (WHERE fielding_play='Putout') putouts,
 count(DISTINCT (fielding_position,group_id)) FILTER (WHERE fielding_play='Assist') assists,
 count(*) FILTER (WHERE fielding_play='Error') errors FROM credit_segments GROUP BY event_key;
CREATE TEMP VIEW runner_totals AS
SELECT event_key,sum(run_scored_flag::INTEGER) runs FROM stg_event_baserunners GROUP BY event_key;
CREATE TEMP TABLE credit_assertions AS
SELECT CASE WHEN NOT EXISTS (
 SELECT 1 FROM selected_events e LEFT JOIN credit_totals c USING(event_key)
 LEFT JOIN expected_fielding f USING(event_key) LEFT JOIN runner_totals r USING(event_key)
 WHERE NOT e.no_play_flag AND (
  coalesce(c.putouts,0) IS DISTINCT FROM f.putouts
  OR coalesce(c.assists,0) IS DISTINCT FROM f.assists
  OR coalesce(c.errors,0) IS DISTINCT FROM f.errors
  OR coalesce(c.putouts,0) IS DISTINCT FROM e.outs_on_play
  OR coalesce(r.runs,0) IS DISTINCT FROM e.runs_on_play
 )
) THEN TRUE ELSE error('Recorded PO/A/E/outs/runs do not reconcile') END AS totals_ok;

-- Resolve only supplied credit positions; unknown position0 remains unknown.
-- Count distinct candidates, never ANY_VALUE over competing actors.
CREATE TEMP VIEW credit_actors AS
SELECT f.event_key,f.sequence_id,f.fielding_position,
 count(DISTINCT a.player_id) candidates,min(a.player_id) player_id
FROM stg_event_fielding_plays f JOIN selected_events e USING(event_key)
LEFT JOIN stg_game_fielding_appearances a ON a.game_id=e.game_id
 AND a.fielding_position=f.fielding_position
 AND a.side=CASE e.batting_side WHEN 'Away' THEN 'Home' WHEN 'Home' THEN 'Away' END
 AND e.event_id BETWEEN a.start_event_id AND a.end_event_id
WHERE f.fielding_position BETWEEN 1 AND 9
GROUP BY f.event_key,f.sequence_id,f.fielding_position;
CREATE TEMP TABLE actor_assertions AS
SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM credit_actors WHERE candidates<>1)
 THEN TRUE ELSE error('Missing or conflicting required defensive actor') END AS actors_ok;

SELECT 'games' metric,count(*) n FROM selected_games
UNION ALL SELECT 'events',count(*) FROM selected_events
UNION ALL SELECT 'real_events',count(*) FROM selected_events WHERE NOT no_play_flag
UNION ALL SELECT 'no_play_events',count(*) FROM selected_events WHERE no_play_flag
UNION ALL SELECT 'raw_audit_rows',count(*) FROM stg_event_audit
UNION ALL SELECT 'flags',count(*) FROM stg_event_flags
UNION ALL SELECT 'baserunners',count(*) FROM stg_event_baserunners
UNION ALL SELECT 'real_event_baserunners',count(*) FROM real_raw_runners
UNION ALL SELECT 'no_play_baserunners',count(*) FROM stg_event_baserunners r
 JOIN selected_events e USING(event_key) WHERE e.no_play_flag
UNION ALL SELECT 'fielding_credits',count(*) FROM stg_event_fielding_plays
UNION ALL SELECT 'fielding_appearances',count(*) FROM stg_game_fielding_appearances
-- A recorded empty terminal span applies to no event; never activate its actor.
-- It is not evidence of fielding participation (e.g. a rain-called game).
UNION ALL SELECT 'terminal_empty_appearances',count(*) FROM stg_game_fielding_appearances
 WHERE start_event_id>end_event_id
UNION ALL SELECT 'putouts',coalesce(sum(putouts),0) FROM credit_totals
UNION ALL SELECT 'assists',coalesce(sum(assists),0) FROM credit_totals
UNION ALL SELECT 'errors',coalesce(sum(errors),0) FROM credit_totals
UNION ALL SELECT 'runs',coalesce(sum(runs),0) FROM runner_totals
UNION ALL SELECT 'unknown_position_credits',count(*) FROM stg_event_fielding_plays WHERE fielding_position=0
UNION ALL SELECT 'unresolved_credit_actors',count(*) FROM credit_actors WHERE candidates<>1
ORDER BY metric;
