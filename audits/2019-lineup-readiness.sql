-- F-01-lineup v1: local F-03 main + accepted supplement attached as detail READ_ONLY.
-- Caller disables external access. SELECT only; no source writes, COPY or network.
WITH cohort AS (
 SELECT * FROM games WHERE season=2019 AND is_regular_season=TRUE
 AND home_league IN ('AL','NL') AND away_league IN ('AL','NL')
), guard AS (
 SELECT CASE WHEN EXISTS (SELECT game_id FROM games GROUP BY game_id HAVING count(*)>1)
 THEN error('Duplicate game grain') ELSE TRUE END AS ok
), records AS (
 SELECT 'game' AS record_type, to_json(struct_pack(
   game_id:=g.game_id,date:=g.date,season:=g.season,is_regular_season:=g.is_regular_season,
   home_league:=g.home_league,away_league:=g.away_league,use_dh:=g.use_dh,
   bat_first_side:=g.bat_first_side,park_id:=g.park_id,
   home_team_id:=g.home_team_id,away_team_id:=g.away_team_id,
   home_team_name:=g.home_team_name,away_team_name:=g.away_team_name,
   home_starting_pitcher_id:=g.home_starting_pitcher_id,
   away_starting_pitcher_id:=g.away_starting_pitcher_id,
   lineup_map_home:=g.lineup_map_home,lineup_map_away:=g.lineup_map_away,
   fielding_map_home:=g.fielding_map_home,fielding_map_away:=g.fielding_map_away,
   results:=coalesce((SELECT list(struct_pack(home_team_id:=r.home_team_id,
     away_team_id:=r.away_team_id,forfeit_flag:=r.forfeit_flag,
     suspension_flag:=r.suspension_flag,tie_flag:=r.tie_flag))
     FROM results r WHERE r.game_id=g.game_id),[])))::VARCHAR AS payload
 FROM cohort g
 UNION ALL SELECT 'person',to_json(struct_pack(player_id:=player_id,
   first_name:=first_name,last_name:=last_name,bats:=bats,throws:=throws))::VARCHAR FROM people
 UNION ALL SELECT 'appearance',to_json(struct_pack(game_id:=game_id,player_id:=player_id,
   side:=side,fielding_position:=fielding_position,start_event_id:=start_event_id,
   end_event_id:=end_event_id))::VARCHAR FROM detail.stg_game_fielding_appearances
 UNION ALL SELECT 'manifest',to_json(m)::VARCHAR FROM audit_manifest m
)
SELECT record_type,payload FROM records CROSS JOIN guard WHERE guard.ok
ORDER BY record_type,payload;
