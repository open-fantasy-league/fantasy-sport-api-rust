-- Your SQL goes here
--CREATE EXTENSION pgcrypto;
--CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE TABLE competitions(
	competition_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL
);

CREATE TABLE series(
	series_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name TEXT NOT NULL,
	competition_id UUID NOT NULL REFERENCES competitions,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL
);

CREATE TABLE matches(
	match_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name TEXT NOT NULL,
	series_id UUID NOT NULL REFERENCES series,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL
);

CREATE TABLE teams(
	team_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE team_names(
	team_name_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	team_id UUID NOT NULL REFERENCES teams,
	name TEXT NOT NULL,
	timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE players(
	player_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE player_names(
	player_name_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	player_id UUID NOT NULL REFERENCES players,
	name TEXT NOT NULL,
	timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE series_teams(
	series_id UUID NOT NULL REFERENCES series,
	team_id UUID NOT NULL REFERENCES teams,
	PRIMARY KEY(series_id, team_id)
);

CREATE TABLE team_players(
	team_player_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	player_id UUID NOT NULL REFERENCES players,
	team_id UUID NOT NULL REFERENCES teams,
	timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE player_results(
	player_result_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	player_id UUID NOT NULL REFERENCES players,
	match_id UUID NOT NULL REFERENCES matches,
	result JSONB NOT NULL DEFAULT '{}'::jsonb,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE player_positions(
	player_position_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	player_id UUID NOT NULL REFERENCES players,
	position TEXT NOT NULL,
	timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE team_match_results(
	team_match_result_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	team_id UUID NOT NULL REFERENCES teams,
	match_id UUID NOT NULL REFERENCES matches,
	result TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE team_series_results(
	team_series_result_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	team_id UUID NOT NULL REFERENCES teams,
	series_id UUID NOT NULL REFERENCES series,
	result TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX series_competition_idx on series(competition_id);
CREATE INDEX matches_series_idx on matches(series_id);
CREATE INDEX team_results_matches_idx on team_match_results(match_id);
CREATE INDEX team_results_team_idx on team_match_results(team_id);
CREATE INDEX team_results_series_idx on team_series_results(series_id);
CREATE INDEX team_series_results_team_idx on team_series_results(team_id);
CREATE INDEX player_results_matches_idx on player_results(match_id);
CREATE INDEX player_results_player_idx on player_results(player_id);
-- initially not doing composite indexes, because can see wanting to find "all the teams for a player" AND "all the players for a team"
-- composite index wouldnt work with both questions
CREATE INDEX team_players_idx_1 on team_players(team_id);
CREATE INDEX team_players_idx_2 on team_players(player_id);
CREATE INDEX series_teams_idx_1 on series_teams(series_id);
CREATE INDEX series_teams_idx_2 on series_teams(team_id);
CREATE INDEX player_names_idx on player_names(player_id);
CREATE INDEX team_names_idx on team_names(team_id);
CREATE INDEX player_positions_idx on player_positions(player_id);
-- might want an index for query player-name and find most recent player

CREATE INDEX team_match_result_idx on team_match_results(result);
CREATE INDEX team_series_result_idx on team_series_results(result);
CREATE INDEX competition_timespan_idx on competitions USING gist (timespan);
CREATE INDEX series_timespan_idx on series USING gist (timespan);
CREATE INDEX matches_timespan_idx on matches USING gist (timespan);
CREATE INDEX team_players_timespan_idx on team_players USING gist (timespan);
CREATE INDEX player_positions_timespan_idx on player_positions USING gist (timespan);
