-- E-03 scorer/responsibility inventory of the external F-03 offline snapshot.
-- Reproduce: duckdb -readonly SNAPSHOT -csv < audits/2019-scorer-coverage.sql
-- Aggregate output only; no source rows, network access or snapshot mutation.
SET enable_external_access=false;
WITH charge AS (
 SELECT r.*, e.game_id AS charge_game, e.pitcher_id AS charge_pitcher,
        reached.game_id AS reached_game
 FROM running r
 LEFT JOIN events e ON e.event_key=r.charge_event_key
 LEFT JOIN events reached ON reached.event_key=r.reached_on_event_key
), metrics AS (
 SELECT 'running_rows' AS metric, count(*) AS n FROM charge
 UNION ALL SELECT 'run_credits', sum(runs) FROM charge
 UNION ALL SELECT 'run_rows_missing_charge_event', count(*) FROM charge WHERE runs=1 AND charge_game IS NULL
 UNION ALL SELECT 'run_rows_charge_game_mismatch', count(*) FROM charge WHERE runs=1 AND charge_game!=game_id
 UNION ALL SELECT 'run_rows_reached_game_mismatch', count(*) FROM charge WHERE runs=1 AND reached_game!=game_id
 UNION ALL SELECT 'run_rows_explicit_pitcher', count(*) FROM charge WHERE runs=1 AND explicit_charged_pitcher_id IS NOT NULL
 UNION ALL SELECT 'run_rows_charge_differs_from_reached', count(*) FROM charge WHERE runs=1 AND charge_event_key!=reached_on_event_key
 UNION ALL SELECT 'run_rows_charged_to_prior_pitcher', count(*) FROM charge WHERE runs=1 AND coalesce(explicit_charged_pitcher_id,charge_pitcher)!=current_pitcher_id
 UNION ALL SELECT 'runner_error_advances', sum(advances_on_errors) FROM charge
 UNION ALL SELECT 'runner_outs_avoided_on_errors', sum(outs_avoided_on_errors) FROM charge
 UNION ALL SELECT 'events_missing_team_unearned', count(*) FROM events WHERE NOT no_play_flag AND team_unearned_runs IS NULL
 UNION ALL SELECT 'team_unearned_run_sum', sum(team_unearned_runs) FROM events WHERE NOT no_play_flag
 UNION ALL SELECT 'fielding_incomplete_event_sum', sum(incomplete_events) FROM fielding
 UNION ALL SELECT 'fielding_unknown_putout_sum', sum(unknown_putouts) FROM fielding
 UNION ALL SELECT 'result_forfeit_games', count(*) FROM results WHERE forfeit_flag
 UNION ALL SELECT 'result_suspended_games', count(*) FROM results WHERE suspension_flag
 UNION ALL SELECT 'result_tied_games', count(*) FROM results WHERE tie_flag
 UNION ALL SELECT 'result_shortened_games', count(*) FROM results WHERE is_shortened_game
)
SELECT metric,n FROM metrics ORDER BY metric;
