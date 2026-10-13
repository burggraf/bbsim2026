-- Independently grouped literal runner-child facts, not inferred opportunity rates.
-- Read-only snapshot; caller attaches accepted supplement AS detail (READ_ONLY).
-- SELECT stdout only; preserve empty SQL-NULL CSV serialization (-nullvalue '').
SET enable_external_access=FALSE;
CREATE TEMP TABLE runner_transition_assertions AS
SELECT CASE WHEN
 (SELECT count(*)=count(DISTINCT event_key)
  AND count(*) FILTER(WHERE event_key IS NULL OR game_id IS NULL OR no_play_flag IS NULL)=0
  FROM events)
 AND (SELECT count(*)=count(DISTINCT event_key)
  AND count(*)=count(DISTINCT (game_id,event_id))
  AND count(*) FILTER(WHERE event_key IS NULL OR game_id IS NULL OR event_id IS NULL
   OR no_play_flag IS NULL OR outs_on_play IS NULL OR runs_on_play IS NULL)=0
  FROM detail.selected_events)
 AND NOT EXISTS(SELECT 1 FROM detail.selected_events s LEFT JOIN events e USING(event_key)
  WHERE e.event_key IS NULL OR s.game_id IS DISTINCT FROM e.game_id
   OR s.no_play_flag IS DISTINCT FROM e.no_play_flag
   OR CAST(s.plate_appearance_result AS VARCHAR) IS DISTINCT FROM CAST(e.plate_appearance_result AS VARCHAR)
   OR s.outs_on_play IS DISTINCT FROM e.outs_on_play
   OR s.runs_on_play IS DISTINCT FROM e.runs_on_play
   OR CAST(s.batted_trajectory AS VARCHAR) IS DISTINCT FROM CAST(e.batted_trajectory AS VARCHAR))
 AND NOT EXISTS(SELECT 1 FROM events e LEFT JOIN detail.selected_events s USING(event_key)
  WHERE s.event_key IS NULL)
 AND (SELECT count(*)=count(DISTINCT (event_key,baserunner))
  AND count(*) FILTER(WHERE event_key IS NULL OR baserunner IS NULL
   OR baserunner NOT IN ('Batter','First','Second','Third')
   OR is_out IS NULL OR run_scored_flag IS NULL)=0
  FROM detail.stg_event_baserunners)
 AND NOT EXISTS(SELECT 1 FROM detail.stg_event_baserunners r LEFT JOIN detail.selected_events s USING(event_key)
  WHERE s.event_key IS NULL OR r.game_id IS DISTINCT FROM s.game_id
   OR r.event_id IS DISTINCT FROM s.event_id)
 THEN TRUE ELSE error('unsupported runner transition group facts') END AS verified;
WITH child_facts AS (
 SELECT s.no_play_flag AS no_play,
  CASE WHEN s.game_id IN ('BOS201908070','CHA201905270','DET201905190','NYN201906130')
       THEN 'excluded'
       WHEN substr(sha256(s.game_id),1,1) IN ('0','1','2') THEN 'heldout'
       ELSE 'training' END AS split,
  coalesce(CAST(s.plate_appearance_result AS VARCHAR),'') AS result,
  coalesce(CAST(s.batted_trajectory AS VARCHAR),'') AS trajectory,
  CASE WHEN s.no_play_flag THEN NULL ELSE e.base_state END AS base_mask,
  CASE WHEN s.no_play_flag THEN NULL ELSE e.outs END AS outs_before,
  s.outs_on_play AS event_outs,s.runs_on_play AS event_runs,
  r.baserunner,r.runner_id IS NOT NULL AS identity_present,r.base_end,
  r.baserunning_play_type,r.attempted_advance_to_base,r.is_out,r.run_scored_flag,
  r.explicit_out_flag,r.advanced_on_error_flag,r.is_advance_attempt
 FROM detail.stg_event_baserunners r
 JOIN detail.selected_events s USING(event_key)
 JOIN events e USING(event_key)
 CROSS JOIN runner_transition_assertions a
 WHERE a.verified
)
SELECT to_json(struct_pack(no_play:=no_play,split:=split,result:=result,trajectory:=trajectory,
 base_mask:=base_mask,outs_before:=outs_before,event_outs:=event_outs,event_runs:=event_runs,
 baserunner:=baserunner,identity_present:=identity_present,base_end:=base_end,
 baserunning_play_type:=baserunning_play_type,attempted_advance_to_base:=attempted_advance_to_base,
 is_out:=is_out,run_scored_flag:=run_scored_flag,explicit_out_flag:=explicit_out_flag,
 advanced_on_error_flag:=advanced_on_error_flag,is_advance_attempt:=is_advance_attempt)) AS facts,
 count(*) AS rows
FROM child_facts
GROUP BY ALL
ORDER BY facts;
