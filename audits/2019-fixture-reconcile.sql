-- F-02 fixture reconciliation v1. Remote catalog READ_ONLY, only three 2019 game IDs.
-- Output: game_id, team_id, side, metric, event_total, source_summary, independent_box_line_or_NULL.
INSTALL ducklake;
LOAD ducklake;
ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS baseball (TYPE ducklake, READ_ONLY);
CREATE TEMP TABLE fixture_games AS
SELECT game_id, home_team_id, away_team_id
FROM baseball.main_models.game_start_info
WHERE season = 2019 AND is_regular_season = TRUE
  AND home_league IN ('AL','NL') AND away_league IN ('AL','NL')
  AND game_id IN ('ANA201904040','ANA201904050','ANA201904070');
CREATE TEMP TABLE fixture_events AS
SELECT e.game_id,e.event_key,e.batting_team_id,e.fielding_team_id,e.outs_on_play,e.runs_on_play
FROM baseball.main_models.stg_events e JOIN fixture_games g USING (game_id)
WHERE e.season = 2019;
WITH teams AS (
  SELECT game_id, home_team_id AS team_id, 'Home' AS side FROM fixture_games
  UNION ALL SELECT game_id, away_team_id, 'Away' FROM fixture_games
),
pa AS (
  SELECT e.game_id,e.batting_team_id AS team_id,sum(b.plate_appearances) AS pa,sum(b.hits) AS hits
  FROM fixture_events e JOIN baseball.main_models.event_batting_stats b USING (event_key)
  GROUP BY 1,2
),
scoring AS (
  SELECT game_id,batting_team_id AS team_id,sum(runs_on_play) AS runs
  FROM fixture_events GROUP BY 1,2
),
outs AS (
  SELECT game_id,fielding_team_id AS team_id,sum(outs_on_play) AS outs
  FROM fixture_events GROUP BY 1,2
),
running AS (
  SELECT e.game_id,r.batting_team_id AS team_id,
         sum(r.stolen_bases) AS steals,sum(r.caught_stealing) AS cs
  FROM fixture_events e JOIN baseball.main_models.event_baserunning_stats r USING (event_key)
  GROUP BY 1,2
),
fielding AS (
  SELECT e.game_id,f.team_id,sum(f.putouts) AS putouts,
         sum(f.assists) AS assists,sum(f.errors) AS errors
  FROM fixture_events e JOIN baseball.main_models.event_fielding_stats f USING (event_key)
  GROUP BY 1,2
)
SELECT t.game_id,t.team_id,t.side,m.metric,m.event_total,m.source_summary,m.box_total
FROM teams t
LEFT JOIN pa USING (game_id,team_id)
LEFT JOIN scoring USING (game_id,team_id)
LEFT JOIN outs USING (game_id,team_id)
LEFT JOIN running USING (game_id,team_id)
LEFT JOIN fielding USING (game_id,team_id)
LEFT JOIN baseball.main_models.team_game_offense_stats o USING (game_id,team_id)
LEFT JOIN baseball.main_models.team_game_fielding_stats f USING (game_id,team_id)
LEFT JOIN baseball.main_models.team_game_results tr USING (game_id,team_id)
LEFT JOIN baseball.main_models.stg_box_score_team_batting_lines bb ON bb.game_id=t.game_id AND bb.side=t.side
LEFT JOIN baseball.main_models.stg_box_score_team_fielding_lines bf ON bf.game_id=t.game_id AND bf.side=t.side
CROSS JOIN LATERAL (VALUES
 ('pa',pa.pa,o.plate_appearances,NULL::BIGINT),
 ('runs',scoring.runs,tr.runs_scored,bb.runs),
 ('outs',outs.outs,f.outs_played,bf.outs_played),
 ('hits',pa.hits,tr.hits,bb.hits),
 ('steals',running.steals,tr.stolen_bases,bb.stolen_bases),
 ('caught_stealing',running.cs,tr.caught_stealing,bb.caught_stealing),
 ('putouts',fielding.putouts,tr.putouts,bf.putouts),
 ('assists',fielding.assists,tr.assists,bf.assists),
 ('errors',fielding.errors,tr.errors,bf.errors)
) AS m(metric,event_total,source_summary,box_total)
ORDER BY t.game_id,t.side,m.metric;
