-- E-03 v1: diagnostic historical replay/golden candidates in the F-03 snapshot.
-- Run: duckdb -readonly SNAPSHOT -csv < audits/2019-replay-exclusions.sql
-- Necessary conditions only, NOT an importer or complete terminal-log proof.
-- Do not use source is_nine_inning_game: missing duration is coalesced to TRUE.
SET enable_external_access=false;
WITH selected AS (
 SELECT game_id
 FROM games
 WHERE season=2019 AND is_regular_season=TRUE
   AND home_league IN ('AL','NL') AND away_league IN ('AL','NL')
 GROUP BY game_id
), game_facts AS (
 -- Grain is global to the key; an out-of-selection duplicate is still ambiguous.
 SELECT game_id, count(*) AS game_rows FROM games GROUP BY game_id
), result_facts AS (
 SELECT game_id, count(*) AS result_rows,
        min(forfeit_flag) AS forfeit_flag,
        min(suspension_flag) AS suspension_flag,
        min(tie_flag) AS tie_flag,
        min(duration_outs) AS duration_outs,
        min(home_runs_scored) AS home_runs_scored,
        min(away_runs_scored) AS away_runs_scored
 FROM results GROUP BY game_id
), classified AS (
 SELECT s.game_id,
 CASE
  WHEN s.game_id IS NULL OR trim(s.game_id)=''
    OR coalesce(g.game_rows,0)!=1 OR coalesce(r.result_rows,0)!=1
    OR r.forfeit_flag IS NULL OR r.suspension_flag IS NULL OR r.tie_flag IS NULL
    OR r.duration_outs IS NULL OR r.home_runs_scored IS NULL OR r.away_runs_scored IS NULL
   THEN 'unknown_metadata'
  WHEN r.forfeit_flag THEN 'forfeit'
  WHEN r.suspension_flag THEN 'suspended'
  WHEN r.tie_flag THEN 'tied'
  WHEN r.duration_outs<51 THEN 'shortened'
  WHEN (r.home_runs_scored>r.away_runs_scored AND r.duration_outs%6 IN (3,4,5))
    OR (r.away_runs_scored>r.home_runs_scored AND r.duration_outs%6=0)
   THEN 'candidate'
  ELSE 'unsupported_finish'
 END AS reason
 FROM selected s LEFT JOIN game_facts g USING(game_id)
 LEFT JOIN result_facts r USING(game_id)
), categories(reason) AS (
 VALUES ('candidate'),('unknown_metadata'),('forfeit'),('suspended'),('tied'),
        ('shortened'),('unsupported_finish')
), metrics AS (
 SELECT c.reason AS metric,count(g.reason) AS n
 FROM categories c LEFT JOIN classified g ON c.reason=g.reason GROUP BY c.reason
 UNION ALL SELECT 'selected_games',count(*) FROM selected
)
SELECT metric,n FROM metrics ORDER BY metric;
