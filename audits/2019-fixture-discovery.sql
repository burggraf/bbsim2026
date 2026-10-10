-- F-02 candidate selection v1. Read-only, 2019 AL/NL regular-season only.
INSTALL ducklake;
LOAD ducklake;
ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS baseball (TYPE ducklake, READ_ONLY);
CREATE TEMP TABLE games AS
SELECT game_id FROM baseball.main_models.game_start_info
WHERE season = 2019 AND is_regular_season = TRUE
  AND home_league IN ('AL','NL') AND away_league IN ('AL','NL');
CREATE TEMP TABLE candidates AS
SELECT e.game_id, e.event_id, e.event_key, e.plate_appearance_result,
       e.outs_on_play, e.runs_on_play,
       r.runner_id, r.baserunning_play_type, r.base_end, r.is_out
FROM baseball.main_models.stg_events e
JOIN games g USING (game_id)
JOIN baseball.main_models.stg_event_baserunners r USING (event_key)
WHERE e.season = 2019 AND e.no_play_flag = FALSE
  AND ((e.plate_appearance_result IS NULL AND r.baserunning_play_type IN ('StolenBase', 'CaughtStealing', 'WildPitch', 'PassedBall', 'Balk'))
    OR (e.plate_appearance_result = 'StrikeOut' AND r.baserunning_play_type IS NOT NULL)
    OR (e.runs_on_play > 0 AND e.outs_on_play > 0));
WITH ranked AS (
 SELECT *, CASE WHEN plate_appearance_result IS NULL THEN 'non_pa'
                WHEN plate_appearance_result = 'StrikeOut' THEN 'strikeout_runner'
                ELSE 'run_and_out' END AS kind,
        row_number() OVER (PARTITION BY CASE WHEN plate_appearance_result IS NULL THEN 'non_pa'
                       WHEN plate_appearance_result = 'StrikeOut' THEN 'strikeout_runner'
                       ELSE 'run_and_out' END ORDER BY game_id,event_id) AS rn
 FROM candidates
)
SELECT kind,game_id,event_id,event_key,plate_appearance_result,outs_on_play,runs_on_play,
       runner_id,baserunning_play_type,base_end,is_out
FROM ranked WHERE rn <= 3 ORDER BY kind,rn;
