-- E-03 supplemental published pitcher ER comparison, query v1.
-- Output contains source rows: save outside Git; not engine reconstruction.
INSTALL ducklake;
LOAD ducklake;
SET enable_external_file_cache=false;
ATTACH 'https://data.baseball.computer/baseball/v1/baseball.ducklake' AS source (TYPE ducklake, READ_ONLY);
SELECT er.game_id, er.player_id, er.earned_runs
FROM source.main_models.stg_game_earned_runs er
JOIN source.main_models.game_start_info g USING(game_id)
WHERE g.season=2019 AND g.is_regular_season
 AND g.home_league IN ('AL','NL') AND g.away_league IN ('AL','NL')
 AND er.game_id IN ('ANA201904040','ANA201904050','ANA201904070')
ORDER BY er.game_id, er.player_id;
