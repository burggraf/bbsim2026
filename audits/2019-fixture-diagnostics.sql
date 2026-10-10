-- F-02 aggregate diagnostics v1. Read-only 2019 source comparison; no raw rows exported.
INSTALL ducklake;
LOAD ducklake;
ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS baseball (TYPE ducklake, READ_ONLY);
CREATE TEMP TABLE games AS
SELECT game_id FROM baseball.main_models.game_start_info
WHERE season=2019 AND is_regular_season=TRUE
  AND home_league IN ('AL','NL') AND away_league IN ('AL','NL');
CREATE TEMP TABLE schedule AS
SELECT date,home_team,visiting_team,makeup_dates,postponement_indicator
FROM baseball.main_models.stg_schedule
WHERE date>=TIMESTAMP '2019-01-01' AND date<TIMESTAMP '2020-01-01'
  AND home_team_league IN ('AL','NL') AND visiting_team_league IN ('AL','NL');
CREATE TEMP TABLE bip_mismatches AS
SELECT e.plate_appearance_result,e.batted_to_fielder,
       coalesce(b.balls_in_play,0) AS batted_bip,
       coalesce(f.plate_appearances_in_field_with_ball_in_play,0) AS fielding_bip
FROM baseball.main_models.stg_events e JOIN games g USING(game_id)
LEFT JOIN baseball.main_models.event_batted_ball_stats b USING(event_key)
LEFT JOIN baseball.main_models.event_fielding_stats f USING(event_key)
WHERE e.season=2019 AND e.no_play_flag=FALSE
  AND coalesce(b.balls_in_play,0) IS DISTINCT FROM
      coalesce(f.plate_appearances_in_field_with_ball_in_play,0);
WITH metrics AS (
 SELECT 'candidate_schedule_rows' AS issue,'date + AL/NL codes' AS detail,count(*) AS rows FROM schedule
 UNION ALL SELECT 'selected_game_starts','game_id + regular-season flag',count(*) FROM games
 UNION ALL SELECT '2019_batting_box_rows','selected regular-season games',count(*) FROM baseball.main_models.stg_box_score_team_batting_lines JOIN games USING(game_id)
 UNION ALL SELECT '2019_fielding_box_rows','selected regular-season games',count(*) FROM baseball.main_models.stg_box_score_team_fielding_lines JOIN games USING(game_id)
 UNION ALL SELECT 'not_made_up_marker',cast(date AS DATE)::VARCHAR || ':' || home_team || '-' || visiting_team || ':' || makeup_dates,count(*) FROM schedule WHERE makeup_dates ILIKE '%not made up%' GROUP BY 1,2
 UNION ALL SELECT 'bip_definition_mismatch',coalesce(plate_appearance_result,'NULL') || ':fielder=' || coalesce(batted_to_fielder::VARCHAR,'NULL') || ':batted=' || batted_bip::VARCHAR || ':fielding=' || fielding_bip::VARCHAR,count(*) FROM bip_mismatches GROUP BY 1,2
)
SELECT issue,detail,rows FROM metrics ORDER BY issue,detail;
