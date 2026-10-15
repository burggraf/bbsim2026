-- Descriptive source-representation cells, not reconstructed retirements or rates.
-- Prepend the EXACT frozen assertion prefix from 2019-contact-consequence-support.sql
-- before WITH flag_vectors; caller opens both accepted databases READ_ONLY.
-- The materialized guard is mandatory even when no child records exist.
WITH real AS MATERIALIZED (
 SELECT e.event_key,e.base_state,e.batter_id,
  CAST(e.plate_appearance_result AS VARCHAR) AS result,b.plate_appearances AS pa,
  s.runner_first_id_start AS first_id,s.runner_second_id_start AS second_id,
  s.runner_third_id_start AS third_id,
  CASE WHEN e.game_id IN ('BOS201908070','CHA201905270','DET201905190','NYN201906130')
   THEN 'excluded' WHEN left(sha256(e.game_id),1) IN ('0','1','2') THEN 'heldout'
   ELSE 'training' END AS split
 FROM events e LEFT JOIN states s USING(event_key) LEFT JOIN batting b USING(event_key)
 CROSS JOIN contact_consequence_assertions proof
 WHERE NOT e.no_play_flag AND proof.verified
), frames AS (
 SELECT event_key,split,pa,result,'event_batter' AS grain,'Batter' AS role,
  batter_id AS expected_id FROM real
 UNION ALL
 SELECT e.event_key,e.split,e.pa,e.result,'before_runner',p.role,
  CASE p.role WHEN 'First' THEN e.first_id WHEN 'Second' THEN e.second_id
   WHEN 'Third' THEN e.third_id END
 FROM real e CROSS JOIN (VALUES ('First',1),('Second',2),('Third',4)) p(role,bit)
 WHERE (e.base_state & p.bit)<>0
), literal AS (
 SELECT f.grain,f.split,f.pa,f.result,f.role,f.expected_id,
  r.event_key AS child_key,r.runner_id,r.is_out,r.base_end,
  r.attempted_advance_to_base,r.is_advance_attempt
 FROM frames f LEFT JOIN detail.stg_event_baserunners r
  ON r.event_key=f.event_key AND CAST(r.baserunner AS VARCHAR)=f.role
 UNION ALL
 SELECT 'runner_child',e.split,e.pa,e.result,CAST(r.baserunner AS VARCHAR),
  CASE CAST(r.baserunner AS VARCHAR) WHEN 'Batter' THEN e.batter_id
   WHEN 'First' THEN CASE WHEN (e.base_state & 1)<>0 THEN e.first_id END
   WHEN 'Second' THEN CASE WHEN (e.base_state & 2)<>0 THEN e.second_id END
   WHEN 'Third' THEN CASE WHEN (e.base_state & 4)<>0 THEN e.third_id END END,
  r.event_key,r.runner_id,r.is_out,r.base_end,r.attempted_advance_to_base,r.is_advance_attempt
 FROM real e JOIN detail.stg_event_baserunners r USING(event_key)
), cells AS (
 SELECT grain,to_json(struct_pack(split:=split,pa:=pa,result:=result,role:=role,
  expected_identity_present:=expected_id IS NOT NULL,child_present:=child_key IS NOT NULL,
  observed_identity_present:=CASE WHEN child_key IS NOT NULL THEN runner_id IS NOT NULL END,
  identity_matches:=CASE WHEN child_key IS NOT NULL AND expected_id IS NOT NULL
   AND runner_id IS NOT NULL THEN expected_id=runner_id END,
  is_out:=is_out,base_end:=CAST(base_end AS VARCHAR),
  attempted_advance_to_base:=CAST(attempted_advance_to_base AS VARCHAR),
  is_advance_attempt:=is_advance_attempt)) AS facts
 FROM literal
)
SELECT grain,facts,count(*) AS units FROM cells GROUP BY grain,facts
ORDER BY grain,CAST(facts AS VARCHAR);
