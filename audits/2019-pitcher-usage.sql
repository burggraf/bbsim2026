-- Read-only context projection; join one-to-one to the accepted outcome export.
SELECT event_key,game_id,event_id,inning,batting_side
FROM events WHERE season=2019 AND no_play_flag=FALSE ORDER BY event_key;
