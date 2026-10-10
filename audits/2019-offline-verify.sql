-- F-03 v1: run against local snapshot with -readonly and enable_external_access=false.
-- Any failed assertion exits the CLI nonzero under -bail; never promote that file.
WITH measured AS (
 SELECT name,n AS source_rows,
 CASE name
  WHEN 'games' THEN (SELECT count(*) FROM games)
  WHEN 'schedule' THEN (SELECT count(*) FROM schedule)
  WHEN 'results' THEN (SELECT count(*) FROM results)
  WHEN 'events' THEN (SELECT count(*) FROM events)
  WHEN 'states' THEN (SELECT count(*) FROM states)
  WHEN 'batting' THEN (SELECT count(*) FROM batting)
  WHEN 'running' THEN (SELECT count(*) FROM running)
  WHEN 'fielding' THEN (SELECT count(*) FROM fielding)
  WHEN 'batted' THEN (SELECT count(*) FROM batted)
  WHEN 'team_results' THEN (SELECT count(*) FROM team_results)
  WHEN 'parks' THEN (SELECT count(*) FROM parks)
  WHEN 'people' THEN (SELECT count(*) FROM people)
 END AS local_rows FROM source_counts
)
SELECT name,source_rows,local_rows FROM measured ORDER BY name;

SELECT CASE WHEN
 (SELECT count(*) FROM source_counts)=12
 AND (SELECT sum(abs(n - CASE name
  WHEN 'games' THEN (SELECT count(*) FROM games)
  WHEN 'schedule' THEN (SELECT count(*) FROM schedule)
  WHEN 'results' THEN (SELECT count(*) FROM results)
  WHEN 'events' THEN (SELECT count(*) FROM events)
  WHEN 'states' THEN (SELECT count(*) FROM states)
  WHEN 'batting' THEN (SELECT count(*) FROM batting)
  WHEN 'running' THEN (SELECT count(*) FROM running)
  WHEN 'fielding' THEN (SELECT count(*) FROM fielding)
  WHEN 'batted' THEN (SELECT count(*) FROM batted)
  WHEN 'team_results' THEN (SELECT count(*) FROM team_results)
  WHEN 'parks' THEN (SELECT count(*) FROM parks)
  WHEN 'people' THEN (SELECT count(*) FROM people)
 END)) FROM source_counts)=0
 THEN 'source/local counts equal' ELSE error('source/local count mismatch') END;
SELECT CASE WHEN
 (SELECT count(*) FROM games)=2429 AND (SELECT count(*) FROM schedule)=2430
 AND (SELECT count(*) FROM results)=2429 AND (SELECT count(*) FROM events)=222047
 AND (SELECT count(*) FROM states)=222047 AND (SELECT count(*) FROM batting)=186517
 AND (SELECT count(*) FROM running)=182718 AND (SELECT count(*) FROM fielding)=192025
 AND (SELECT count(*) FROM team_results)=4858
 THEN 'F-01 baseline counts equal' ELSE error('F-01 baseline mismatch / catalog drift') END;
SELECT CASE WHEN
 (SELECT count(DISTINCT event_key) FROM events)=(SELECT count(*) FROM events)
 AND (SELECT count(*) FROM events e LEFT JOIN states s USING(event_key) WHERE s.event_key IS NULL)=0
 AND (SELECT count(*) FROM events WHERE season<>2019)=0
 AND (SELECT count(*) FROM games WHERE season<>2019 OR NOT is_regular_season)=0
 AND (SELECT count(*) FROM audit_manifest WHERE schema_version='f03-spike-1'
      AND catalog_sha256='2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583')=1
 THEN 'identity, season and manifest checks equal' ELSE error('identity/season/manifest mismatch') END;
-- Cross-check selected source aggregates from the independent F-01 SQL run on the same catalog hash.
-- These are still source-derived, not independent historical box scores.
SELECT CASE WHEN
 (SELECT count(*) FROM events WHERE no_play_flag)=30022
 AND (SELECT count(*) FROM events e JOIN batted b USING(event_key)
      WHERE NOT e.no_play_flag AND b.balls_in_play>0)=118577
 AND (SELECT sum(errors) FROM fielding)=2836
 AND (SELECT sum(stolen_bases) FROM running)=2280
 AND (SELECT sum(caught_stealing) FROM running)=832
 AND (SELECT sum(extra_bases_taken) FROM running)=6394
 THEN 'F-01 event/credit totals equal' ELSE error('F-01 event/credit totals mismatch') END;
SELECT CASE WHEN
 (SELECT count(*) FROM events e JOIN states s USING(event_key)
  WHERE e.game_id='ANA201904040' AND e.event_id=36
    AND s.base_state_start=3 AND s.base_state_end=6
    AND s.outs_start=2 AND s.outs_on_play=0 AND s.runs_on_play=0)=1
 THEN 'offline fixture transition readable' ELSE error('offline fixture missing') END;
