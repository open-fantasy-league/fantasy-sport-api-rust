-- Your SQL goes here
ALTER TABLE team_names ADD CONSTRAINT non_overlap_team_name_timespan EXCLUDE USING gist (timespan WITH &&);
ALTER TABLE player_names ADD CONSTRAINT non_overlap_player_name_timespan EXCLUDE USING gist (timespan WITH &&);
ALTER TABLE team_players ADD CONSTRAINT non_overlap_team_players_timespan EXCLUDE USING gist (timespan WITH &&);
