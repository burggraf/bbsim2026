-- Read-only independent native terminal-participant totals, not pitch counts/BF.
WITH source AS (
  SELECT * FROM events WHERE season=2019
), live AS (
  SELECT e.event_key,e.game_id,e.pitcher_id,b.plate_appearances,
         CASE WHEN COALESCE(e.plate_appearance_result,'')<>'' THEN 1 ELSE 0 END AS terminal
  FROM source e LEFT JOIN batting b ON b.event_key=e.event_key
  WHERE e.no_play_flag=false
)
SELECT game_id,pitcher_id,SUM(terminal) AS terminal_pa,SUM(1-terminal) AS non_pa
FROM live GROUP BY game_id,pitcher_id
UNION ALL
SELECT error('unsupported workload identity, PA, no-play metadata or join fanout'),NULL,NULL,NULL
WHERE EXISTS (SELECT 1 FROM source WHERE no_play_flag IS NULL)
   OR EXISTS (SELECT 1 FROM live WHERE game_id IS NULL OR game_id='' OR pitcher_id IS NULL OR pitcher_id=''
              OR (terminal=1 AND plate_appearances IS DISTINCT FROM 1)
              OR (terminal=0 AND plate_appearances IS NOT NULL AND plate_appearances<>0))
   OR (SELECT COUNT(*) FROM live)<>(SELECT COUNT(DISTINCT event_key) FROM live)
ORDER BY game_id,pitcher_id;
