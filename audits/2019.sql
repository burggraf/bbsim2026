-- F-01 query v1. Read-only remote DuckLake; only 2019 AL/NL regular-season keys/columns enter TEMP tables.
-- Reproduce: duckdb :memory: -csv -noheader < audits/2019.sql > /tmp/bbsim-f01-2019.csv
INSTALL ducklake;
LOAD ducklake;
ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS baseball (TYPE ducklake, READ_ONLY);

CREATE TEMP TABLE games AS
SELECT game_id, season, date, source_type, home_league, away_league,
       home_team_id, away_team_id, home_starting_pitcher_id, away_starting_pitcher_id,
       lineup_map_home, lineup_map_away, fielding_map_home, fielding_map_away,
       park_id, use_dh, game_type
FROM baseball.main_models.game_start_info
WHERE season = 2019 AND is_regular_season = TRUE
  AND home_league IN ('AL', 'NL') AND away_league IN ('AL', 'NL');

CREATE TEMP TABLE schedule AS
SELECT date, home_team, visiting_team, home_team_league, visiting_team_league,
       postponement_indicator, park_id
FROM baseball.main_models.stg_schedule
WHERE date >= TIMESTAMP '2019-01-01' AND date < TIMESTAMP '2020-01-01'
  AND home_team_league IN ('AL', 'NL') AND visiting_team_league IN ('AL', 'NL');

CREATE TEMP TABLE events AS
SELECT e.game_id, e.event_key, e.event_id, e.no_play_flag, e.plate_appearance_result,
       e.specified_batter_hand, e.specified_pitcher_hand,
       s.inning_start, s.frame_start, s.outs_start, s.base_state_start,
       s.inning_end, s.frame_end, s.outs_end, s.base_state_end, s.frame_end_flag,
       s.score_home_start, s.score_away_start, s.score_home_end, s.score_away_end,
       s.outs_on_play, s.runs_on_play, s.batter_id, s.pitcher_id,
       s.batter_hand, s.pitcher_hand, s.runner_first_id_start,
       s.runner_second_id_start, s.runner_third_id_start
FROM baseball.main_models.stg_events e
JOIN games g ON e.game_id = g.game_id
LEFT JOIN baseball.main_models.event_states_full s ON s.event_key = e.event_key
WHERE e.season = 2019;

CREATE TEMP TABLE non_pa_runners AS
SELECT r.event_key, r.runner_id, r.stolen_bases, r.caught_stealing, r.picked_off,
       r.advances_on_wild_pitches, r.advances_on_passed_balls, r.advances_on_balks
FROM baseball.main_models.event_baserunning_stats r
JOIN events e ON r.event_key = e.event_key
WHERE e.no_play_flag = FALSE AND e.plate_appearance_result IS NULL;

-- CSV fields: metric, denominator, count, unknown, note. Unknown is not silently counted as zero.
-- A count can be a subset or a sum of outcomes, as stated in the note.
WITH rows AS (
  SELECT 'games_source_rows' metric, count(*) denominator, count(*) count_value, 0 unknown_value,
         'Selected game_start_info rows; NOT an independent schedule' note FROM games
  UNION ALL SELECT 'schedule_rows', count(*), count(*), 0, '2019 stg_schedule AL/NL rows, including postponements; no game_id, not directly reconciled to starts' FROM schedule
  UNION ALL SELECT 'schedule_postponements', count(*), count(*) FILTER (WHERE postponement_indicator IS NOT NULL AND postponement_indicator <> ''), 0, 'nonempty postponement indicator; null means no indicator (not missing); semantics require fixture review' FROM schedule
  UNION ALL SELECT 'schedule_team_slots', count(*)*2, sum((home_team IS NOT NULL)::INT + (visiting_team IS NOT NULL)::INT), sum((home_team IS NULL)::INT + (visiting_team IS NULL)::INT), 'team codes on scheduled rows' FROM schedule
  UNION ALL SELECT 'schedule_park_id', count(*), count(*) FILTER (WHERE park_id IS NOT NULL), count(*) FILTER (WHERE park_id IS NULL), 'scheduled park ID' FROM schedule
  UNION ALL SELECT 'games_distinct_ids', count(*), count(DISTINCT game_id), count(*) - count(DISTINCT game_id), 'game_start_info IDs' FROM games
  UNION ALL SELECT 'teams_distinct', count(*), count(DISTINCT team_id), sum((team_id IS NULL)::INT), 'distinct team IDs among two slots per game; denominator is slots' FROM (SELECT home_team_id AS team_id FROM games UNION ALL SELECT away_team_id FROM games)
  UNION ALL SELECT 'players_distinct_event_participants', count(*), count(DISTINCT player_id), sum((player_id IS NULL)::INT), 'distinct batter/pitcher IDs among two slots per event' FROM (SELECT batter_id AS player_id FROM events UNION ALL SELECT pitcher_id FROM events)
  UNION ALL SELECT 'games_with_events', count(*), count(e.game_id), count(*) - count(e.game_id), 'selected starts with at least one stg_event row' FROM games g LEFT JOIN (SELECT DISTINCT game_id FROM events) e USING (game_id)
  UNION ALL SELECT 'games_with_non_pa', count(*), count(e.game_id), 0, 'selected games with at least one real non-PA event; absence is not missingness' FROM games g LEFT JOIN (SELECT DISTINCT game_id FROM events WHERE no_play_flag = FALSE AND plate_appearance_result IS NULL) e USING (game_id)
  UNION ALL SELECT 'games_with_results', count(*), count(r.game_id), count(*) - count(r.game_id), 'game_results row; played proxy' FROM games g LEFT JOIN baseball.main_models.game_results r USING (game_id)
  UNION ALL SELECT 'games_pbp', count(*), count(*) FILTER (WHERE source_type = 'PlayByPlay'), count(*) FILTER (WHERE source_type IS NULL), 'source_type=PlayByPlay' FROM games
  UNION ALL SELECT 'games_box_or_event', count(*), count(*) FILTER (WHERE source_type IN ('BoxScore','Event')), count(*) FILTER (WHERE source_type IS NULL), 'source_type=BoxScore or Event' FROM games
  UNION ALL SELECT 'games_other_source', count(*), count(*) FILTER (WHERE source_type NOT IN ('PlayByPlay','BoxScore','Event')), count(*) FILTER (WHERE source_type IS NULL), 'source_type outside known categories' FROM games
  UNION ALL SELECT 'games_team_ids', count(*)*2, sum((home_team_id IS NOT NULL)::INT + (away_team_id IS NOT NULL)::INT), sum((home_team_id IS NULL)::INT + (away_team_id IS NULL)::INT), 'two team slots per game' FROM games
  UNION ALL SELECT 'games_starter_ids', count(*)*2, sum((home_starting_pitcher_id IS NOT NULL)::INT + (away_starting_pitcher_id IS NOT NULL)::INT), sum((home_starting_pitcher_id IS NULL)::INT + (away_starting_pitcher_id IS NULL)::INT), 'two starter slots per game' FROM games
  UNION ALL SELECT 'games_dh_used', count(*), count(*) FILTER (WHERE use_dh), count(*) FILTER (WHERE use_dh IS NULL), 'use_dh true; adds one lineup/fielding map slot per side in source' FROM games
  UNION ALL SELECT 'lineup_slots', sum(18 + 2*use_dh::INT), sum(len(map_keys(lineup_map_home)) + len(map_keys(lineup_map_away))), sum(18 + 2*use_dh::INT - len(map_keys(lineup_map_home)) - len(map_keys(lineup_map_away))), 'expected 9 per side without DH, 10 with DH; map entry count only' FROM games
  UNION ALL SELECT 'fielding_slots', sum(18 + 2*use_dh::INT), sum(len(map_keys(fielding_map_home)) + len(map_keys(fielding_map_away))), sum(18 + 2*use_dh::INT - len(map_keys(fielding_map_home)) - len(map_keys(fielding_map_away))), 'source maps include DH as tenth slot; not all nine actual fielders' FROM games
  UNION ALL SELECT 'lineup_player_ids', sum(len(map_keys(lineup_map_home)) + len(map_keys(lineup_map_away))), sum(len(list_filter(map_values(lineup_map_home), lambda x: x IS NOT NULL)) + len(list_filter(map_values(lineup_map_away), lambda x: x IS NOT NULL))), sum(len(map_keys(lineup_map_home)) + len(map_keys(lineup_map_away)) - len(list_filter(map_values(lineup_map_home), lambda x: x IS NOT NULL)) - len(list_filter(map_values(lineup_map_away), lambda x: x IS NOT NULL))), 'nonnull player IDs among lineup-map entries' FROM games
  UNION ALL SELECT 'fielding_player_ids', sum(len(map_keys(fielding_map_home)) + len(map_keys(fielding_map_away))), sum(len(list_filter(map_values(fielding_map_home), lambda x: x IS NOT NULL)) + len(list_filter(map_values(fielding_map_away), lambda x: x IS NOT NULL))), sum(len(map_keys(fielding_map_home)) + len(map_keys(fielding_map_away)) - len(list_filter(map_values(fielding_map_home), lambda x: x IS NOT NULL)) - len(list_filter(map_values(fielding_map_away), lambda x: x IS NOT NULL))), 'nonnull player IDs among fielding-map entries, including DH' FROM games
  UNION ALL SELECT 'games_park_lookup', count(*), count(*) FILTER (WHERE EXISTS (SELECT 1 FROM baseball.main_models.stg_parks p WHERE p.park_id = g.park_id)), count(*) FILTER (WHERE g.park_id IS NULL OR NOT EXISTS (SELECT 1 FROM baseball.main_models.stg_parks p WHERE p.park_id = g.park_id)), 'park ID resolves in park metadata, not geometry' FROM games g
  UNION ALL SELECT 'games_park_id', count(*), count(*) FILTER (WHERE park_id IS NOT NULL), count(*) FILTER (WHERE park_id IS NULL), 'park ID not park geometry' FROM games
  UNION ALL SELECT 'games_dh_rule', count(*), count(*) FILTER (WHERE use_dh IS NOT NULL), count(*) FILTER (WHERE use_dh IS NULL), 'recorded use_dh; other rules not encoded here' FROM games
  UNION ALL SELECT 'games_dates', count(*), count(*) FILTER (WHERE date IS NOT NULL), count(*) FILTER (WHERE date IS NULL), 'played/start date not independent scheduled date' FROM games
  UNION ALL SELECT 'events_rows', count(*), count(*), 0, 'all stg_events for selected games including no-play' FROM events
  UNION ALL SELECT 'events_distinct_keys', count(*), count(DISTINCT event_key), count(*) - count(DISTINCT event_key), 'duplicate event keys' FROM events
  UNION ALL SELECT 'events_no_play', count(*), count(*) FILTER (WHERE no_play_flag), count(*) FILTER (WHERE no_play_flag IS NULL), 'no-play flag' FROM events
  UNION ALL SELECT 'events_pa', count(*) FILTER (WHERE NOT no_play_flag), count(*) FILTER (WHERE plate_appearance_result IS NOT NULL AND NOT no_play_flag), count(*) FILTER (WHERE no_play_flag IS NULL), 'PA result among real events' FROM events
  UNION ALL SELECT 'events_non_pa', count(*) FILTER (WHERE NOT no_play_flag), count(*) FILTER (WHERE plate_appearance_result IS NULL AND NOT no_play_flag), count(*) FILTER (WHERE no_play_flag IS NULL), 'no PA result among real events; may include substitutions' FROM events
  UNION ALL SELECT 'events_state_before', count(*), count(*) FILTER (WHERE inning_start IS NOT NULL AND frame_start IS NOT NULL AND outs_start IS NOT NULL AND base_state_start IS NOT NULL AND score_home_start IS NOT NULL AND score_away_start IS NOT NULL), count(*) FILTER (WHERE inning_start IS NULL OR frame_start IS NULL OR outs_start IS NULL OR base_state_start IS NULL OR score_home_start IS NULL OR score_away_start IS NULL), 'complete inning/frame/outs/bases/score before' FROM events
  UNION ALL SELECT 'events_state_after', count(*), count(*) FILTER (WHERE inning_end IS NOT NULL AND frame_end IS NOT NULL AND outs_end IS NOT NULL AND base_state_end IS NOT NULL AND score_home_end IS NOT NULL AND score_away_end IS NOT NULL), count(*) FILTER (WHERE inning_end IS NULL OR frame_end IS NULL OR outs_end IS NULL OR base_state_end IS NULL OR score_home_end IS NULL OR score_away_end IS NULL), 'complete inning/frame/outs/bases/score after; inning end may have null bases' FROM events
  UNION ALL SELECT 'events_state_after_non_boundary', count(*) FILTER (WHERE frame_end_flag = FALSE), count(*) FILTER (WHERE frame_end_flag = FALSE AND inning_end IS NOT NULL AND frame_end IS NOT NULL AND outs_end IS NOT NULL AND base_state_end IS NOT NULL AND score_home_end IS NOT NULL AND score_away_end IS NOT NULL), count(*) FILTER (WHERE frame_end_flag = FALSE AND (inning_end IS NULL OR frame_end IS NULL OR outs_end IS NULL OR base_state_end IS NULL OR score_home_end IS NULL OR score_away_end IS NULL)), 'complete after state except frame-ending transitions; null base at boundary may be intentional' FROM events
  UNION ALL SELECT 'events_frame_end', count(*), count(*) FILTER (WHERE frame_end_flag = TRUE), count(*) FILTER (WHERE frame_end_flag IS NULL), 'frame-ending transitions' FROM events
  UNION ALL SELECT 'events_delta', count(*), count(*) FILTER (WHERE outs_on_play IS NOT NULL AND runs_on_play IS NOT NULL), count(*) FILTER (WHERE outs_on_play IS NULL OR runs_on_play IS NULL), 'outs/runs on play' FROM events
  UNION ALL SELECT 'events_batter_pitcher_ids', count(*)*2, sum((batter_id IS NOT NULL)::INT + (pitcher_id IS NOT NULL)::INT), sum((batter_id IS NULL)::INT + (pitcher_id IS NULL)::INT), 'participant IDs including non-PA/no-play' FROM events
  UNION ALL SELECT 'events_effective_hands', count(*)*2, sum(COALESCE(batter_hand IN ('L','R'), FALSE)::INT + COALESCE(pitcher_hand IN ('L','R'), FALSE)::INT), sum((batter_hand NOT IN ('L','R') OR batter_hand IS NULL)::INT + (pitcher_hand NOT IN ('L','R') OR pitcher_hand IS NULL)::INT), 'effective hands L/R only; source-derived, not observed matchup measurements' FROM events
  UNION ALL SELECT 'events_explicit_hands', count(*)*2, sum((specified_batter_hand IS NOT NULL)::INT + (specified_pitcher_hand IS NOT NULL)::INT), 0, 'explicit event-side annotations; other effective hands are bio/opponent-derived' FROM events
  UNION ALL SELECT 'occupied_runner_ids', sum(((base_state_start & 1) != 0)::INT + ((base_state_start & 2) != 0)::INT + ((base_state_start & 4) != 0)::INT), sum(((base_state_start & 1) != 0 AND runner_first_id_start IS NOT NULL)::INT + ((base_state_start & 2) != 0 AND runner_second_id_start IS NOT NULL)::INT + ((base_state_start & 4) != 0 AND runner_third_id_start IS NOT NULL)::INT), sum(((base_state_start & 1) != 0 AND runner_first_id_start IS NULL)::INT + ((base_state_start & 2) != 0 AND runner_second_id_start IS NULL)::INT + ((base_state_start & 4) != 0 AND runner_third_id_start IS NULL)::INT), 'occupied before-state base slots; after runner IDs not in event_states_full' FROM events
  UNION ALL SELECT 'events_after_runner_state_join', count(*), count(b.event_key), count(*) - count(b.event_key), 'event_base_out_states row for each source event' FROM events e LEFT JOIN baseball.main_models.event_base_out_states b USING (event_key)
  UNION ALL SELECT 'occupied_after_runner_ids', sum(((b.base_state_end & 1) != 0)::INT + ((b.base_state_end & 2) != 0)::INT + ((b.base_state_end & 4) != 0)::INT), sum(((b.base_state_end & 1) != 0 AND b.runner_first_id_end IS NOT NULL)::INT + ((b.base_state_end & 2) != 0 AND b.runner_second_id_end IS NOT NULL)::INT + ((b.base_state_end & 4) != 0 AND b.runner_third_id_end IS NOT NULL)::INT), sum(((b.base_state_end & 1) != 0 AND b.runner_first_id_end IS NULL)::INT + ((b.base_state_end & 2) != 0 AND b.runner_second_id_end IS NULL)::INT + ((b.base_state_end & 4) != 0 AND b.runner_third_id_end IS NULL)::INT), 'occupied after-state base slots outside frame resets; end runner IDs in event_base_out_states' FROM events e LEFT JOIN baseball.main_models.event_base_out_states b USING (event_key)
  UNION ALL SELECT 'bip_events', count(*) FILTER (WHERE NOT e.no_play_flag), count(*) FILTER (WHERE b.balls_in_play > 0 AND NOT e.no_play_flag), 0, 'BIP among real events; no-BIP is not unknown'  FROM events e LEFT JOIN baseball.main_models.event_batted_ball_stats b USING (event_key)
  UNION ALL SELECT 'bip_trajectory_known', count(*) FILTER (WHERE b.balls_in_play > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND b.trajectory_known > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND (b.trajectory_known IS NULL OR b.trajectory_known = 0)), 'source-derived trajectory on BIP; unknown distinct from no BIP' FROM events e LEFT JOIN baseball.main_models.event_batted_ball_stats b USING (event_key)
  UNION ALL SELECT 'bip_location_known', count(*) FILTER (WHERE b.balls_in_play > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND b.batted_location_known > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND (b.batted_location_known IS NULL OR b.batted_location_known = 0)), 'source-derived location on BIP' FROM events e LEFT JOIN baseball.main_models.event_batted_ball_stats b USING (event_key)
  UNION ALL SELECT 'bip_recorded_trajectory', count(*) FILTER (WHERE b.balls_in_play > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND c.recorded_trajectory IS NOT NULL AND c.recorded_trajectory <> 'Unknown'), count(*) FILTER (WHERE b.balls_in_play > 0 AND (c.recorded_trajectory IS NULL OR c.recorded_trajectory = 'Unknown')), 'recorded (not inferred) trajectory on BIP' FROM events e LEFT JOIN baseball.main_models.event_batted_ball_stats b USING (event_key) LEFT JOIN baseball.main_models.calc_batted_ball_type c USING (event_key)
  UNION ALL SELECT 'bip_recorded_location', count(*) FILTER (WHERE b.balls_in_play > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND c.recorded_location IS NOT NULL AND c.recorded_location <> 'Unknown'), count(*) FILTER (WHERE b.balls_in_play > 0 AND (c.recorded_location IS NULL OR c.recorded_location = 'Unknown')), 'recorded (not fielder-inferred) location on BIP' FROM events e LEFT JOIN baseball.main_models.event_batted_ball_stats b USING (event_key) LEFT JOIN baseball.main_models.calc_batted_ball_type c USING (event_key)
  UNION ALL SELECT 'bip_trajectory_deduced', count(*) FILTER (WHERE b.balls_in_play > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND c.is_trajectory_deduced), count(*) FILTER (WHERE b.balls_in_play > 0 AND c.is_trajectory_deduced IS NULL), 'deduced flag on BIP; joins calc_batted_ball_type' FROM events e LEFT JOIN baseball.main_models.event_batted_ball_stats b USING (event_key) LEFT JOIN baseball.main_models.calc_batted_ball_type c USING (event_key)
  UNION ALL SELECT 'bip_fielded_by_known', count(*) FILTER (WHERE b.balls_in_play > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND b.fielded_by_known > 0), count(*) FILTER (WHERE b.balls_in_play > 0 AND (b.fielded_by_known IS NULL OR b.fielded_by_known = 0)), 'known fielder category on BIP, not an opportunity model' FROM events e LEFT JOIN baseball.main_models.event_batted_ball_stats b USING (event_key)
  UNION ALL SELECT 'fielding_bip_rows', count(*) FILTER (WHERE NOT e.no_play_flag), count(*) FILTER (WHERE f.plate_appearances_in_field_with_ball_in_play > 0), count(*) FILTER (WHERE NOT e.no_play_flag AND f.event_key IS NULL), 'fielding BIP flag among real events; may differ from batted-ball BIP'  FROM events e LEFT JOIN baseball.main_models.event_fielding_stats f USING (event_key)
  UNION ALL SELECT 'fielding_bip_incomplete', count(*) FILTER (WHERE f.plate_appearances_in_field_with_ball_in_play > 0), count(*) FILTER (WHERE f.plate_appearances_in_field_with_ball_in_play > 0 AND f.incomplete_events > 0), count(*) FILTER (WHERE f.plate_appearances_in_field_with_ball_in_play > 0 AND f.incomplete_events IS NULL), 'incomplete fielding credits, not missing opportunities' FROM events e LEFT JOIN baseball.main_models.event_fielding_stats f USING (event_key)
  UNION ALL SELECT 'fielding_errors', count(*) FILTER (WHERE f.event_key IS NOT NULL), sum(f.errors), count(*) FILTER (WHERE NOT e.no_play_flag AND f.event_key IS NULL), 'error credits summed; denominator event-fielding rows, may exceed one per play'  FROM events e LEFT JOIN baseball.main_models.event_fielding_stats f USING (event_key)
  UNION ALL SELECT 'fielding_unknown_putouts', count(*) FILTER (WHERE f.event_key IS NOT NULL), sum(f.unknown_putouts), count(*) FILTER (WHERE NOT e.no_play_flag AND f.event_key IS NULL), 'unknown putouts summed; missing real-event joins separately'  FROM events e LEFT JOIN baseball.main_models.event_fielding_stats f USING (event_key)
  UNION ALL SELECT 'fielder_opportunity_rows', count(*), count(*) FILTER (WHERE balls_hit_to > 0), count(*) FILTER (WHERE player_id IS NULL), 'event_player_fielding_stats rows; balls_hit_to proxy, not all eligible opportunities' FROM baseball.main_models.event_player_fielding_stats p JOIN games g USING (game_id)
  UNION ALL SELECT 'non_pa_runner_rows', count(*), count(*), count(*) FILTER (WHERE runner_id IS NULL), 'runner-event rows attached to real non-PA events; not all non-PA events have runners' FROM non_pa_runners
  UNION ALL SELECT 'non_pa_steals', count(*), sum(stolen_bases), count(*) FILTER (WHERE stolen_bases IS NULL), 'steal successes on non-PA runner rows, not all steal events' FROM non_pa_runners
  UNION ALL SELECT 'non_pa_caught_stealing', count(*), sum(caught_stealing), count(*) FILTER (WHERE caught_stealing IS NULL), 'caught stealing on non-PA runner rows' FROM non_pa_runners
  UNION ALL SELECT 'non_pa_pickoffs', count(*), sum(picked_off), count(*) FILTER (WHERE picked_off IS NULL), 'pickoff outs on non-PA runner rows' FROM non_pa_runners
  UNION ALL SELECT 'non_pa_wild_pitch_advances', count(*), sum(advances_on_wild_pitches), count(*) FILTER (WHERE advances_on_wild_pitches IS NULL), 'wild-pitch runner advances on non-PA events' FROM non_pa_runners
  UNION ALL SELECT 'non_pa_passed_ball_advances', count(*), sum(advances_on_passed_balls), count(*) FILTER (WHERE advances_on_passed_balls IS NULL), 'passed-ball runner advances on non-PA events' FROM non_pa_runners
  UNION ALL SELECT 'non_pa_balk_advances', count(*), sum(advances_on_balks), count(*) FILTER (WHERE advances_on_balks IS NULL), 'balk runner advances on non-PA events' FROM non_pa_runners
  UNION ALL SELECT 'roster_2019_rows', count(*), count(*) FILTER (WHERE player_id IS NOT NULL AND team_id IS NOT NULL), count(*) FILTER (WHERE player_id IS NULL OR team_id IS NULL), 'stg_rosters year=2019; season-level records, NOT date-specific active roster' FROM baseball.main_models.stg_rosters WHERE year = 2019
  UNION ALL SELECT 'runner_rows', count(*), count(*), count(*) FILTER (WHERE runner_id IS NULL), 'event_baserunning_stats runner-event rows; includes batter' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
  UNION ALL SELECT 'runner_steal_opportunities', count(*), sum(stolen_base_opportunities), count(*) FILTER (WHERE stolen_base_opportunities IS NULL), 'sum of modeled opportunity flags; denominator runner-event rows' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
  UNION ALL SELECT 'runner_steals', count(*), sum(stolen_bases), count(*) FILTER (WHERE stolen_bases IS NULL), 'sum of steal successes; denominator runner-event rows' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
  UNION ALL SELECT 'runner_caught_stealing', count(*), sum(caught_stealing), count(*) FILTER (WHERE caught_stealing IS NULL), 'sum of CS; denominator runner-event rows' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
  UNION ALL SELECT 'runner_extra_base_chances', count(*), sum(extra_base_chances), count(*) FILTER (WHERE extra_base_chances IS NULL), 'source-defined limited single/double advance chances' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
  UNION ALL SELECT 'runner_extra_bases_taken', count(*), sum(extra_bases_taken), count(*) FILTER (WHERE extra_bases_taken IS NULL), 'source-derived successful extra bases on limited chances' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
  UNION ALL SELECT 'runner_picked_off', count(*), sum(picked_off), count(*) FILTER (WHERE picked_off IS NULL), 'pickoff outs, not all attempts' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
  UNION ALL SELECT 'runner_wild_pitch_advances', count(*), sum(advances_on_wild_pitches), count(*) FILTER (WHERE advances_on_wild_pitches IS NULL), 'runner advances, not distinct events' FROM baseball.main_models.event_baserunning_stats r JOIN games g USING (game_id)
)
SELECT metric, denominator, count_value, unknown_value, note FROM rows ORDER BY metric;
