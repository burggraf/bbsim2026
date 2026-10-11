-- preview-baseline-v1: read-only, offline F-03 snapshot only; no fitting/calibration.
-- Run duckdb -readonly -csv SNAPSHOT < this file. Keep output outside Git.
WITH selected AS (
 SELECT b.event_key, e.plate_appearance_result AS result
 FROM batting b JOIN events e USING(event_key) WHERE b.plate_appearances=1
), guard AS (
 SELECT CASE WHEN
  (SELECT count(*) FROM batting)<>(SELECT count(DISTINCT event_key) FROM batting)
  OR (SELECT count(*) FROM events)<>(SELECT count(DISTINCT event_key) FROM events)
  OR (SELECT count(*) FROM batting WHERE plate_appearances=1)<>(SELECT count(*) FROM selected)
  OR EXISTS(SELECT 1 FROM batting WHERE plate_appearances IS NULL OR plate_appearances NOT IN (0,1))
  OR EXISTS(SELECT 1 FROM events e LEFT JOIN batting b USING(event_key)
    WHERE e.plate_appearance_result IS NOT NULL AND b.plate_appearances IS DISTINCT FROM 1)
  OR EXISTS(SELECT 1 FROM events WHERE season IS NULL OR season<>2019)
  OR (SELECT count(*) FROM audit_manifest)<>1
  OR (SELECT catalog_sha256 FROM audit_manifest LIMIT 1) IS DISTINCT FROM '2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583'
 THEN error('preview baseline: invalid grain, season, manifest or missing/inconsistent PA row') ELSE TRUE END AS ok
), classified AS (
 SELECT CASE result
  WHEN 'StrikeOut' THEN 'strikeout' WHEN 'Walk' THEN 'walk'
  WHEN 'HitByPitch' THEN 'hbp' WHEN 'Single' THEN 'single'
  WHEN 'Double' THEN 'double' WHEN 'GroundRuleDouble' THEN 'double'
  WHEN 'Triple' THEN 'triple' WHEN 'HomeRun' THEN 'home_run'
  WHEN 'InsideTheParkHomeRun' THEN 'home_run' WHEN 'ReachedOnError' THEN 'roe'
  WHEN 'InPlayOut' THEN 'contact_out' WHEN 'FieldersChoice' THEN 'contact_out'
  WHEN 'SacrificeFly' THEN 'contact_out' WHEN 'SacrificeHit' THEN 'contact_out'
  WHEN 'IntentionalWalk' THEN 'excluded_ibb' WHEN 'Interference' THEN 'excluded_interference'
  ELSE error('preview baseline: unknown or missing PA classification') END AS category
 FROM selected
), keys AS (
 SELECT unnest(['strikeout','walk','hbp','single','double','triple','home_run','roe','contact_out','excluded_ibb','excluded_interference']) AS key
), counts AS (
 SELECT keys.key, (SELECT count(*) FROM classified WHERE category=keys.key)::VARCHAR AS value FROM keys
), rows AS (
 SELECT 0 AS position,'format' AS key,'preview-counts-v1' AS value
 UNION ALL SELECT 1,'catalog_sha256',catalog_sha256 FROM audit_manifest
 UNION ALL SELECT 2,'query_version','preview-baseline-v1'
 UNION ALL SELECT 3,'selected_pa',count(*)::VARCHAR FROM selected
 UNION ALL SELECT 4,key,value FROM counts
)
SELECT key,value FROM rows CROSS JOIN guard WHERE guard.ok ORDER BY position,key;
