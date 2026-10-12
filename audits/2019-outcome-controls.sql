-- 2019-outcome-controls-v1: independent scalar/grouped paths, same cached upstream.
-- This is not an independent official-record comparison.
SET enable_external_access=FALSE;
SELECT 'metric' AS dimension,'real_events' AS value,count(*) AS n FROM events WHERE NOT no_play_flag
UNION ALL SELECT 'metric','pa',count(*) FROM batting
UNION ALL SELECT 'metric','event_outs',sum(outs_on_play) FROM events WHERE NOT no_play_flag
UNION ALL SELECT 'metric','state_outs',sum(s.outs_on_play) FROM states s JOIN events e USING(event_key) WHERE NOT e.no_play_flag
UNION ALL SELECT 'metric','putouts',sum(putouts) FROM fielding
UNION ALL SELECT 'metric','assists',sum(assists) FROM fielding
UNION ALL SELECT 'metric','errors',sum(errors) FROM fielding
UNION ALL SELECT 'metric','event_runs',sum(runs_on_play) FROM events WHERE NOT no_play_flag
UNION ALL SELECT 'metric','state_runs',sum(s.runs_on_play) FROM states s JOIN events e USING(event_key) WHERE NOT e.no_play_flag
UNION ALL SELECT 'metric','runner_runs',sum(runs) FROM running
UNION ALL SELECT 'metric','runner_rows',count(*) FROM running
UNION ALL SELECT 'metric','force_exposure',sum(times_force_on_runner) FROM running
UNION ALL SELECT 'metric','steal_opportunities',sum(stolen_base_opportunities) FROM running
UNION ALL SELECT 'metric','extra_base_chances',sum(extra_base_chances) FROM running
UNION ALL SELECT 'metric','bip',sum(balls_in_play) FROM batted
UNION ALL SELECT 'metric','batted_rows',count(*) FROM batted
UNION ALL SELECT 'metric','dp_events',count(*) FROM fielding WHERE double_plays=1
UNION ALL SELECT 'metric','tp_events',count(*) FROM fielding WHERE triple_plays=1
UNION ALL SELECT 'metric','gidp',sum(grounded_into_double_plays) FROM batting
UNION ALL SELECT 'result',coalesce(plate_appearance_result,'<non-PA>'),count(*) FROM events WHERE NOT no_play_flag GROUP BY plate_appearance_result
ORDER BY dimension,value;
