-- Read-only supplement export. Aggregate children independently before joining.
CREATE TEMP TABLE mechanism_assertions AS
SELECT CASE WHEN
 (SELECT count(*)=count(DISTINCT event_key) AND count(*)=count(DISTINCT (game_id,event_id))
  AND count(*) FILTER(WHERE event_key IS NULL OR game_id IS NULL OR game_id=''
      OR event_id IS NULL OR no_play_flag IS NULL OR outs_on_play IS NULL
      OR runs_on_play IS NULL)=0 FROM selected_events)
 AND (SELECT count(*)=count(DISTINCT event_key)
      AND count(*) FILTER(WHERE raw_play IS NULL OR trim(raw_play)='')=0 FROM stg_event_audit)
 AND NOT EXISTS (SELECT 1 FROM selected_events e WHERE NOT EXISTS
      (SELECT 1 FROM stg_event_audit a WHERE a.event_key=e.event_key))
 AND NOT EXISTS (SELECT 1 FROM stg_event_audit a WHERE NOT EXISTS
      (SELECT 1 FROM selected_events e WHERE e.event_key=a.event_key
       AND e.game_id=a.game_id AND e.event_id=a.event_id))
 AND (SELECT count(*)=count(DISTINCT (event_key,sequence_id))
      AND count(*) FILTER(WHERE event_key IS NULL OR sequence_id IS NULL OR flag IS NULL)=0
      FROM stg_event_flags)
 AND (SELECT count(*)=count(DISTINCT (event_key,sequence_id))
      AND count(*) FILTER(WHERE event_key IS NULL OR sequence_id IS NULL
          OR fielding_position IS NULL OR fielding_play IS NULL)=0
      FROM stg_event_fielding_plays)
 AND (SELECT count(*)=count(DISTINCT (event_key,baserunner))
      AND count(*) FILTER(WHERE event_key IS NULL OR baserunner IS NULL
          OR is_out IS NULL OR run_scored_flag IS NULL)=0 FROM stg_event_baserunners)
 AND NOT EXISTS (SELECT 1 FROM stg_event_flags f WHERE NOT EXISTS
      (SELECT 1 FROM selected_events e WHERE e.event_key=f.event_key))
 AND NOT EXISTS (SELECT 1 FROM stg_event_fielding_plays f WHERE NOT EXISTS
      (SELECT 1 FROM selected_events e WHERE e.event_key=f.event_key
       AND e.game_id=f.game_id AND e.event_id=f.event_id))
 AND NOT EXISTS (SELECT 1 FROM stg_event_baserunners r WHERE NOT EXISTS
      (SELECT 1 FROM selected_events e WHERE e.event_key=r.event_key
       AND e.game_id=r.game_id AND e.event_id=r.event_id))
 THEN TRUE ELSE error('Missing, duplicate, NULL, orphan or mismatched mechanism facts') END verified;
CREATE TEMP VIEW mechanism_flags AS
SELECT event_key, to_json(list(struct_pack(sequence_id:=sequence_id,flag:=flag)
                              ORDER BY sequence_id)) flags
FROM stg_event_flags GROUP BY event_key;
CREATE TEMP VIEW mechanism_credits AS
SELECT event_key, to_json(list(struct_pack(sequence_id:=sequence_id,
       fielding_position:=fielding_position,fielding_play:=fielding_play)
       ORDER BY sequence_id)) credits
FROM stg_event_fielding_plays GROUP BY event_key;
CREATE TEMP VIEW mechanism_runners AS
SELECT event_key, to_json(list(struct_pack(baserunner:=baserunner,runner_id:=runner_id,
       baserunning_play_type:=baserunning_play_type,is_out:=is_out,
       run_scored_flag:=run_scored_flag,explicit_out_flag:=explicit_out_flag,
       advanced_on_error_flag:=advanced_on_error_flag,
       attempted_advance_to_base:=attempted_advance_to_base,base_end:=base_end,
       is_advance_attempt:=is_advance_attempt) ORDER BY baserunner)) runners
FROM stg_event_baserunners GROUP BY event_key;
CREATE TEMP VIEW mechanism_export AS
SELECT e.event_key,e.game_id,e.event_id,e.no_play_flag,
       e.plate_appearance_result result,e.outs_on_play outs,e.runs_on_play runs,
       e.batted_trajectory trajectory,e.batted_location_angle angle,a.raw_play,
       coalesce(f.flags,'[]') flags,coalesce(c.credits,'[]') credits,
       coalesce(r.runners,'[]') runners
FROM selected_events e JOIN stg_event_audit a USING(event_key)
LEFT JOIN mechanism_flags f USING(event_key)
LEFT JOIN mechanism_credits c USING(event_key)
LEFT JOIN mechanism_runners r USING(event_key);
