-- F-02 transition details v1: five specific events in three selected 2019 AL/NL games.
-- Result is five bounded rows, not a source extract. Catalog attached READ_ONLY.
INSTALL ducklake;
LOAD ducklake;
ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS baseball (TYPE ducklake, READ_ONLY);
CREATE TEMP TABLE events AS
SELECT e.game_id,e.event_id,e.event_key,e.no_play_flag,e.plate_appearance_result,
       e.batter_id,e.pitcher_id,e.batting_team_id,e.fielding_team_id
FROM baseball.main_models.stg_events e
WHERE e.season=2019 AND
 ((e.game_id='ANA201904040' AND e.event_id IN (36,96,99))
  OR (e.game_id='ANA201904050' AND e.event_id=4)
  OR (e.game_id='ANA201904070' AND e.event_id=3));
WITH movement AS (
 SELECT e.event_key,
        string_agg(r.baserunner || ':' || r.runner_id || '->' ||
                   coalesce(r.base_end::VARCHAR,'?') ||
                   coalesce('[' || r.baserunning_play_type || ']','') ||
                   CASE WHEN r.is_out THEN '(out)' ELSE '' END,
                   '; ' ORDER BY r.baserunner) AS runners
 FROM events e JOIN baseball.main_models.stg_event_baserunners r USING (event_key)
 GROUP BY 1
)
SELECT e.game_id,e.event_id,e.event_key,e.no_play_flag,e.plate_appearance_result,
       e.batter_id,e.pitcher_id,e.batting_team_id,e.fielding_team_id,
       s.inning_start,s.frame_start,s.outs_start,s.base_state_start,
       s.runner_first_id_start,s.runner_second_id_start,s.runner_third_id_start,
       s.outs_on_play,s.runs_on_play,s.inning_end,s.frame_end,s.outs_end,s.base_state_end,
       s.runner_first_id_end,s.runner_second_id_end,s.runner_third_id_end,
       s.score_home_start,s.score_away_start,s.score_home_end,s.score_away_end,
       s.frame_end_flag,b.plate_appearances,b.hits,b.strikeouts,
       f.putouts,f.assists,f.errors,f.passed_balls,m.runners
FROM events e
JOIN baseball.main_models.event_base_out_states s USING (event_key)
LEFT JOIN baseball.main_models.event_batting_stats b USING (event_key)
LEFT JOIN baseball.main_models.event_fielding_stats f USING (event_key)
LEFT JOIN movement m USING (event_key)
ORDER BY e.game_id,e.event_id;
