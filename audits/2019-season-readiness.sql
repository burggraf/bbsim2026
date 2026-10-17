-- Accepted local F-03 snapshot only; native caller disables external access.
-- Raw schedule annotations are retained; no date or doubleheader guessing here.
WITH reference_values AS (
 SELECT b.batter_id AS player_id,b.batting_team_id AS team_id,v.metric,v.value
 FROM batting b CROSS JOIN LATERAL (VALUES
 ('PA',b.plate_appearances::BIGINT),('AB',b.at_bats::BIGINT),('H',b.hits::BIGINT),
 ('2B',b.doubles::BIGINT),('3B',b.triples::BIGINT),('HR',b.home_runs::BIGINT),
 ('BB',b.walks::BIGINT),('K',b.strikeouts::BIGINT),('HBP',b.hit_by_pitches::BIGINT),
 ('RBI',b.runs_batted_in::BIGINT)) v(metric,value)
 UNION ALL SELECT r.runner_id,r.batting_team_id,v.metric,v.value
 FROM running r CROSS JOIN LATERAL (VALUES ('R',r.runs::BIGINT),
 ('SB',r.stolen_bases::BIGINT),('CS',r.caught_stealing::BIGINT)) v(metric,value)
), profiles AS (
 SELECT 'batter' AS role,batter_id AS player_id,batting_team_id AS team_id,
 game_id,plate_appearance_result AS result FROM events WHERE no_play_flag=FALSE
 UNION ALL SELECT 'pitcher',pitcher_id,fielding_team_id,game_id,plate_appearance_result
 FROM events WHERE no_play_flag=FALSE
), records AS (
 SELECT 'schedule' AS record_type,to_json(s)::VARCHAR AS payload FROM schedule s
 UNION ALL SELECT 'game',to_json(struct_pack(game_id:=g.game_id,date:=g.date,
 away_team_id:=g.away_team_id,home_team_id:=g.home_team_id,
 doubleheader_status:=g.doubleheader_status,use_dh:=g.use_dh,park_id:=g.park_id,
 source_type:=g.source_type,results:=coalesce((SELECT list(struct_pack(
 finish_date:=r.game_finish_date,forfeit:=r.forfeit_flag,suspended:=r.suspension_flag,
 tied:=r.tie_flag,shortened:=r.is_shortened_game,duration_outs:=r.duration_outs))
 FROM results r WHERE r.game_id=g.game_id),[])))::VARCHAR FROM games g
 UNION ALL SELECT 'reference',to_json(struct_pack(player_id:=player_id,team_id:=team_id,
 metric:=metric,value:=sum(value),unknown_rows:=count(*) FILTER(WHERE value IS NULL),
 source_rows:=count(*)))::VARCHAR FROM reference_values GROUP BY player_id,team_id,metric
 UNION ALL SELECT 'profile',to_json(struct_pack(role:=role,player_id:=player_id,team_id:=team_id,
 games:=count(DISTINCT game_id),events:=count(*),terminal_pa:=count(result),
 ordinary_pa:=count(*) FILTER(WHERE result IS NOT NULL AND result NOT IN ('IntentionalWalk','Interference'))))::VARCHAR
 FROM profiles GROUP BY role,player_id,team_id
)
SELECT record_type,payload FROM records ORDER BY record_type,payload;
