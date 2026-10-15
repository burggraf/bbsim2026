-- Independent literal event-grain facts, not a causal transition or fitted rate.
-- Caller opens snapshot READ_ONLY, attaches accepted supplement AS detail READ_ONLY.
-- Aggregate each child table BEFORE joining; stdout is strict event_key,facts CSV.
SET autoinstall_known_extensions=FALSE;
SET autoload_known_extensions=FALSE;
SET enable_external_access=FALSE;
CREATE TEMP TABLE contact_consequence_assertions AS
SELECT CASE WHEN
 (SELECT count(*)=count(DISTINCT event_key) AND count(*)=count(DISTINCT (game_id,event_id))
  AND count(*) FILTER(WHERE event_key IS NULL OR game_id IS NULL OR event_id IS NULL
   OR season IS DISTINCT FROM 2019 OR no_play_flag IS NULL OR outs_on_play IS NULL
   OR runs_on_play IS NULL OR (NOT no_play_flag AND (batter_id IS NULL OR batter_id=''
    OR outs IS NULL OR outs NOT BETWEEN 0 AND 2 OR base_state IS NULL OR base_state NOT BETWEEN 0 AND 7)))=0
  FROM events)
 AND (SELECT count(*)=count(DISTINCT event_key) AND count(*)=count(DISTINCT (game_id,event_id))
  AND count(*) FILTER(WHERE event_key IS NULL OR game_id IS NULL OR event_id IS NULL
   OR no_play_flag IS NULL OR outs_on_play IS NULL OR runs_on_play IS NULL)=0
  FROM detail.selected_events)
 AND NOT EXISTS(SELECT 1 FROM events e LEFT JOIN detail.selected_events d USING(event_key)
  WHERE d.event_key IS NULL OR e.game_id IS DISTINCT FROM d.game_id OR e.event_id IS DISTINCT FROM d.event_id
   OR e.no_play_flag IS DISTINCT FROM d.no_play_flag
   OR CAST(e.plate_appearance_result AS VARCHAR) IS DISTINCT FROM CAST(d.plate_appearance_result AS VARCHAR)
   OR e.outs_on_play IS DISTINCT FROM d.outs_on_play OR e.runs_on_play IS DISTINCT FROM d.runs_on_play
   OR CAST(e.batted_trajectory AS VARCHAR) IS DISTINCT FROM CAST(d.batted_trajectory AS VARCHAR))
 AND NOT EXISTS(SELECT 1 FROM detail.selected_events d LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
 AND (SELECT count(*)=count(DISTINCT event_key) FROM states)
 AND (SELECT count(*)=count(DISTINCT event_key) FROM batting)
 AND (SELECT count(*)=count(DISTINCT event_key) FROM fielding)
 AND NOT EXISTS(SELECT 1 FROM states s LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
 AND NOT EXISTS(SELECT 1 FROM batting b LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
 AND NOT EXISTS(SELECT 1 FROM fielding f LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
 AND NOT EXISTS(SELECT 1 FROM events e LEFT JOIN states s USING(event_key)
  WHERE NOT e.no_play_flag AND (s.event_key IS NULL OR s.frame_end_flag IS NULL))
 AND (SELECT count(*)=count(DISTINCT event_key)
  AND count(*) FILTER(WHERE event_key IS NULL OR raw_play IS NULL OR trim(raw_play)='')=0
  FROM detail.stg_event_audit)
 AND NOT EXISTS(SELECT 1 FROM detail.selected_events d LEFT JOIN detail.stg_event_audit a USING(event_key)
  WHERE a.event_key IS NULL OR a.game_id IS DISTINCT FROM d.game_id OR a.event_id IS DISTINCT FROM d.event_id)
 AND NOT EXISTS(SELECT 1 FROM detail.stg_event_audit a LEFT JOIN detail.selected_events d USING(event_key)
  WHERE d.event_key IS NULL)
 AND (SELECT count(*)=count(DISTINCT (event_key,sequence_id))
  AND count(*) FILTER(WHERE event_key IS NULL OR sequence_id IS NULL OR flag IS NULL)=0
  FROM detail.stg_event_flags)
 AND NOT EXISTS(SELECT 1 FROM detail.stg_event_flags f LEFT JOIN detail.selected_events d USING(event_key)
  WHERE d.event_key IS NULL)
 AND (SELECT count(*)=count(DISTINCT (event_key,baserunner))
  AND count(*) FILTER(WHERE event_key IS NULL OR baserunner IS NULL
   OR baserunner NOT IN ('Batter','First','Second','Third') OR is_out IS NULL OR run_scored_flag IS NULL)=0
  FROM detail.stg_event_baserunners)
 AND NOT EXISTS(SELECT 1 FROM detail.stg_event_baserunners r LEFT JOIN detail.selected_events d USING(event_key)
  WHERE d.event_key IS NULL OR r.game_id IS DISTINCT FROM d.game_id OR r.event_id IS DISTINCT FROM d.event_id)
 THEN TRUE ELSE error('unsupported contact consequence joint facts') END AS verified;
WITH flag_vectors AS (
 SELECT event_key,to_json(list(struct_pack(sequence_id:=sequence_id,flag:=flag) ORDER BY sequence_id)) AS flags
 FROM detail.stg_event_flags GROUP BY event_key
), runner_vectors AS (
 SELECT event_key,to_json(list(struct_pack(baserunner:=baserunner,runner_id:=runner_id,
  baserunning_play_type:=baserunning_play_type,is_out:=is_out,run_scored_flag:=run_scored_flag,
  explicit_out_flag:=explicit_out_flag,advanced_on_error_flag:=advanced_on_error_flag,
  attempted_advance_to_base:=attempted_advance_to_base,base_end:=base_end,
  is_advance_attempt:=is_advance_attempt) ORDER BY baserunner)) AS runners
 FROM detail.stg_event_baserunners GROUP BY event_key
)
SELECT CAST(e.event_key AS VARCHAR) AS event_key,
 to_json(struct_pack(event_key:=CAST(e.event_key AS VARCHAR),game_id:=e.game_id,
  event_id:=CAST(e.event_id AS VARCHAR),result:=CAST(e.plate_appearance_result AS VARCHAR),
  pa:=b.plate_appearances,gidp:=b.grounded_into_double_plays,base_mask:=e.base_state,
  outs_before:=e.outs,event_outs:=e.outs_on_play,event_runs:=e.runs_on_play,
  frame_end:=s.frame_end_flag,after_outs:=s.outs_end,after_base_mask:=s.base_state_end,
  batter_id:=e.batter_id,first:=s.runner_first_id_start,second:=s.runner_second_id_start,
  third:=s.runner_third_id_start,trajectory:=CAST(e.batted_trajectory AS VARCHAR),
  raw_play:=a.raw_play,flags:=coalesce(v.flags,'[]'::JSON),
  runners:=coalesce(r.runners,'[]'::JSON),errors:=f.errors)) AS facts
FROM events e
JOIN detail.stg_event_audit a USING(event_key)
LEFT JOIN states s USING(event_key)
LEFT JOIN batting b USING(event_key)
LEFT JOIN fielding f USING(event_key)
LEFT JOIN flag_vectors v USING(event_key)
LEFT JOIN runner_vectors r USING(event_key)
CROSS JOIN contact_consequence_assertions verified
WHERE NOT e.no_play_flag AND verified.verified
ORDER BY e.event_key;
