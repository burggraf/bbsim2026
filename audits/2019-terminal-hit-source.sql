-- Read-only literal terminal context; no crossing chronology or resolver.
-- Caller authenticates complete accepted source/qualification before measuring.
SET enable_external_access=FALSE;
WITH guard AS MATERIALIZED (
 SELECT CASE WHEN
  (SELECT count(*)<>count(DISTINCT event_key) FROM events)
  OR (SELECT count(*)<>count(DISTINCT event_key) FROM states)
  OR EXISTS(SELECT 1 FROM events e LEFT JOIN states s USING(event_key) WHERE s.event_key IS NULL)
  OR EXISTS(SELECT 1 FROM states s LEFT JOIN events e USING(event_key) WHERE e.event_key IS NULL)
  OR EXISTS(SELECT 1 FROM events e JOIN states s USING(event_key) WHERE
   e.inning<>s.inning_start OR e.frame<>s.frame_start OR e.outs<>s.outs_start
   OR e.base_state<>s.base_state_start OR e.outs_on_play<>s.outs_on_play
   OR e.runs_on_play<>s.runs_on_play
   -- Compare only supplied endpoints. BIGINT avoids unsigned subtraction underflow.
   OR CAST(s.score_home_end AS BIGINT)-CAST(s.score_home_start AS BIGINT)<>
      CASE e.frame WHEN 'Bottom' THEN e.runs_on_play WHEN 'Top' THEN 0 END
   OR CAST(s.score_away_end AS BIGINT)-CAST(s.score_away_start AS BIGINT)<>
      CASE e.frame WHEN 'Top' THEN e.runs_on_play WHEN 'Bottom' THEN 0 END)
 THEN error('terminal source: conflicting or incomplete event/state join')
 ELSE TRUE END AS ok
)
SELECT e.event_key,e.game_id,e.event_id,e.plate_appearance_result AS result,
 e.inning,e.frame,e.outs AS outs_before,e.base_state AS base_mask,
 e.outs_on_play AS event_outs,e.runs_on_play AS event_runs,e.runs_batted_in AS event_rbi,
 s.outs_end AS after_outs,s.base_state_end AS after_base_mask,
 s.score_home_start AS home_start,s.score_away_start AS away_start,
 s.score_home_end AS home_end,s.score_away_end AS away_end,
 s.frame_end_flag AS frame_end,s.game_end_flag AS game_end,
 s.truncated_frame_flag AS truncated_frame
FROM events e JOIN states s USING(event_key) CROSS JOIN guard
WHERE guard.ok AND NOT e.no_play_flag AND e.plate_appearance_result IN ('Single','Double')
ORDER BY e.event_key;
