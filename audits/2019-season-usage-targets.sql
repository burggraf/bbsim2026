-- 2019-season-usage-targets-v1: accepted F-03 snapshot only, caller disables external access.
-- Batting owns PA credit; every live event owns current-pitcher outs, including non-PA.
WITH live AS (SELECT * FROM events WHERE no_play_flag=false),
usage_rows AS (
 SELECT game_id,batting_team_id AS team_id,batter_id AS player_id,'batter' AS role,
        plate_appearances::BIGINT AS usage FROM batting
 UNION ALL
 SELECT game_id,fielding_team_id,pitcher_id,'pitcher',outs_on_play::BIGINT FROM live
), records AS (
 SELECT 'manifest' AS kind,to_json(m)::VARCHAR AS payload FROM audit_manifest m
 UNION ALL SELECT 'person',to_json(struct_pack(player_id:=player_id)) FROM people
 UNION ALL SELECT 'game',to_json(struct_pack(game_id:=g.game_id,date:=g.date,
 away_team_id:=g.away_team_id,home_team_id:=g.home_team_id,
 starting_pitcher_ids:=map([g.away_team_id,g.home_team_id],
 [g.away_starting_pitcher_id,g.home_starting_pitcher_id]),
 suspended:=r.suspension_flag,shortened:=r.is_shortened_game,duration_outs:=r.duration_outs))
 FROM games g LEFT JOIN results r USING(game_id)
 UNION ALL SELECT 'batting',to_json(struct_pack(event_key:=event_key,game_id:=game_id,
 batter_id:=batter_id,batting_team_id:=batting_team_id,plate_appearances:=plate_appearances)) FROM batting
 UNION ALL SELECT 'event',to_json(struct_pack(event_key:=event_key,game_id:=game_id,
 batter_id:=batter_id,pitcher_id:=pitcher_id,batting_team_id:=batting_team_id,
 fielding_team_id:=fielding_team_id,no_play_flag:=no_play_flag,outs_on_play:=outs_on_play,
 result:=plate_appearance_result)) FROM events
 UNION ALL SELECT 'team_result',to_json(struct_pack(game_id:=game_id,team_id:=team_id,putouts:=putouts))
 FROM team_results
 UNION ALL SELECT 'target',to_json(struct_pack(game_id:=game_id,team_id:=team_id,
 player_id:=player_id,role:=role,usage:=CASE WHEN count(usage)=count(*) THEN sum(usage)::BIGINT END,
 unknown_rows:=count(*)-count(usage),source_rows:=count(*))) FROM usage_rows GROUP BY game_id,team_id,player_id,role
 UNION ALL SELECT 'check',to_json(struct_pack(
 games:=(SELECT count(*) FROM games),live_events:=(SELECT count(*) FROM live),
 pa:=(SELECT sum(plate_appearances)::BIGINT FROM batting),
 outs:=(SELECT sum(outs_on_play)::BIGINT FROM live),
 non_pa_outs:=(SELECT sum(outs_on_play)::BIGINT FROM live WHERE plate_appearance_result IS NULL),
 ibb:=(SELECT count(*) FROM live WHERE plate_appearance_result='IntentionalWalk'),
 ci:=(SELECT count(*) FROM live WHERE plate_appearance_result='Interference'),
 responsibility_switches:=(SELECT count(*) FROM batting b JOIN live e USING(event_key) WHERE b.batter_id<>e.batter_id),
 responsibility_pitcher_differences:=(SELECT count(*) FROM batting b JOIN live e USING(event_key) WHERE b.pitcher_id<>e.pitcher_id),
 sacrifice_credit_gap:=(SELECT sum(plate_appearances-at_bats-walks-hit_by_pitches-sacrifice_hits-sacrifice_flies-reached_on_interferences)::BIGINT FROM batting),
 invalid_cohort:=(SELECT count(*) FROM games WHERE season IS DISTINCT FROM 2019 OR is_regular_season IS DISTINCT FROM true
 OR home_league IS NULL OR home_league NOT IN ('AL','NL') OR away_league IS NULL OR away_league NOT IN ('AL','NL')
 OR source_type IS DISTINCT FROM 'PlayByPlay'),
 invalid_results:=(SELECT count(*) FROM results r LEFT JOIN games g USING(game_id) WHERE g.game_id IS NULL
 OR r.season IS DISTINCT FROM 2019 OR r.home_team_id IS DISTINCT FROM g.home_team_id OR r.away_team_id IS DISTINCT FROM g.away_team_id),
 starter_map_mismatches:=(SELECT count(*) FROM games WHERE home_starting_pitcher_id IS DISTINCT FROM fielding_map_home[1]
 OR away_starting_pitcher_id IS DISTINCT FROM fielding_map_away[1]),
 event_date_mismatches:=(SELECT count(*) FROM events e JOIN games g USING(game_id) WHERE e.date IS DISTINCT FROM g.date),
 state_out_mismatches:=(SELECT count(*) FROM live e LEFT JOIN states s USING(event_key) WHERE s.event_key IS NULL OR e.outs_on_play IS DISTINCT FROM s.outs_on_play),
 fielding_out_mismatches:=(SELECT count(*) FROM live e LEFT JOIN fielding f USING(event_key) WHERE f.event_key IS NULL OR e.outs_on_play IS DISTINCT FROM f.outs_played),
 duplicate_state_keys:=(SELECT count(*)-count(DISTINCT event_key) FROM states),
 duplicate_fielding_keys:=(SELECT count(*)-count(DISTINCT event_key) FROM fielding)))
 UNION ALL SELECT 'source_count',to_json(struct_pack(name:=name,source_rows:=n,local_rows:=CASE name
 WHEN 'games' THEN (SELECT count(*) FROM games) WHEN 'events' THEN (SELECT count(*) FROM events)
 WHEN 'batting' THEN (SELECT count(*) FROM batting) WHEN 'results' THEN (SELECT count(*) FROM results)
 WHEN 'team_results' THEN (SELECT count(*) FROM team_results) WHEN 'people' THEN (SELECT count(*) FROM people)
 WHEN 'states' THEN (SELECT count(*) FROM states) WHEN 'fielding' THEN (SELECT count(*) FROM fielding) END))
 FROM source_counts WHERE name IN ('games','events','batting','results','team_results','people','states','fielding')
)
SELECT kind,payload::VARCHAR AS payload FROM records ORDER BY kind,payload;
