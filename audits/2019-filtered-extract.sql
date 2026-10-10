-- F-03 v1: development-host season-filtered DuckDB snapshot, NOT a production importer.
-- Run ONLY via audits/filtered_spike.py into an external temporary directory.
-- Profiling reports HTTPFS received payloads per CTAS; catalog ATTACH bytes are separate.
INSTALL ducklake;
LOAD ducklake;
SET enable_external_file_cache=false;
SET enable_http_metadata_cache=false;
ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS source (TYPE ducklake, READ_ONLY);

EXPLAIN ANALYZE CREATE TABLE games AS
SELECT * FROM source.main_models.game_start_info
WHERE season=2019 AND is_regular_season=TRUE
  AND home_league IN ('AL','NL') AND away_league IN ('AL','NL');
EXPLAIN ANALYZE CREATE TABLE schedule AS
SELECT * FROM source.main_models.stg_schedule
WHERE date>=TIMESTAMP '2019-01-01' AND date<TIMESTAMP '2020-01-01'
  AND home_team_league IN ('AL','NL') AND visiting_team_league IN ('AL','NL');
EXPLAIN ANALYZE CREATE TABLE results AS
SELECT r.* FROM source.main_models.game_results r JOIN games g USING(game_id)
WHERE r.season=2019;
EXPLAIN ANALYZE CREATE TABLE events AS
SELECT e.* FROM source.main_models.stg_events e JOIN games g USING(game_id)
WHERE e.season=2019;
EXPLAIN ANALYZE CREATE TABLE states AS
SELECT s.* FROM source.main_models.event_base_out_states s JOIN events e USING(event_key);
EXPLAIN ANALYZE CREATE TABLE batting AS
SELECT b.* FROM source.main_models.event_batting_stats b JOIN events e USING(event_key);
EXPLAIN ANALYZE CREATE TABLE running AS
SELECT r.* FROM source.main_models.event_baserunning_stats r JOIN events e USING(event_key);
EXPLAIN ANALYZE CREATE TABLE fielding AS
SELECT f.* FROM source.main_models.event_fielding_stats f JOIN events e USING(event_key);
EXPLAIN ANALYZE CREATE TABLE batted AS
SELECT b.* FROM source.main_models.event_batted_ball_stats b JOIN events e USING(event_key);
EXPLAIN ANALYZE CREATE TABLE team_results AS
SELECT t.* FROM source.main_models.team_game_results t JOIN games g USING(game_id)
WHERE t.season=2019;
EXPLAIN ANALYZE CREATE TABLE parks AS
SELECT p.* FROM source.main_models.stg_parks p
JOIN (SELECT DISTINCT park_id FROM games) g USING(park_id);
EXPLAIN ANALYZE CREATE TABLE people AS
WITH ids AS (
 SELECT batter_id AS player_id FROM events UNION SELECT pitcher_id FROM events
 UNION SELECT runner_id FROM running
)
SELECT p.* FROM source.main_models.people p JOIN ids USING(player_id);

-- Direct source counts use the same season/game predicates, independently of local table counts.
EXPLAIN ANALYZE CREATE TABLE source_counts AS
SELECT 'games' AS name,count(*) AS n FROM source.main_models.game_start_info
 WHERE season=2019 AND is_regular_season=TRUE AND home_league IN ('AL','NL') AND away_league IN ('AL','NL')
UNION ALL SELECT 'schedule',count(*) FROM source.main_models.stg_schedule
 WHERE date>=TIMESTAMP '2019-01-01' AND date<TIMESTAMP '2020-01-01'
 AND home_team_league IN ('AL','NL') AND visiting_team_league IN ('AL','NL')
UNION ALL SELECT 'results',count(*) FROM source.main_models.game_results r JOIN games g USING(game_id) WHERE r.season=2019
UNION ALL SELECT 'events',count(*) FROM source.main_models.stg_events e JOIN games g USING(game_id) WHERE e.season=2019
UNION ALL SELECT 'states',count(*) FROM source.main_models.event_base_out_states s JOIN events e USING(event_key)
UNION ALL SELECT 'batting',count(*) FROM source.main_models.event_batting_stats b JOIN events e USING(event_key)
UNION ALL SELECT 'running',count(*) FROM source.main_models.event_baserunning_stats r JOIN events e USING(event_key)
UNION ALL SELECT 'fielding',count(*) FROM source.main_models.event_fielding_stats f JOIN events e USING(event_key)
UNION ALL SELECT 'batted',count(*) FROM source.main_models.event_batted_ball_stats b JOIN events e USING(event_key)
UNION ALL SELECT 'team_results',count(*) FROM source.main_models.team_game_results t JOIN games g USING(game_id) WHERE t.season=2019
UNION ALL SELECT 'parks',count(*) FROM source.main_models.stg_parks p JOIN (SELECT DISTINCT park_id FROM games) g USING(park_id)
UNION ALL SELECT 'people',count(*) FROM source.main_models.people p JOIN
 (SELECT batter_id AS player_id FROM events UNION SELECT pitcher_id FROM events UNION SELECT runner_id FROM running) ids USING(player_id);

CREATE TABLE audit_manifest AS SELECT
 'f03-spike-1' AS schema_version,
 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS source_url,
 '2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583' AS catalog_sha256,
 '2019-filtered-extract-v1' AS query_version,
 current_timestamp AS extracted_at,
 'The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".' AS attribution;
CHECKPOINT;
