-- Your SQL goes here
--CREATE EXTENSION pgcrypto;
--CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE TABLE competitions(
	competition_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	code TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL
);

CREATE TABLE series(
	series_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	code TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	competition_id UUID NOT NULL REFERENCES competitions,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL
);

CREATE TABLE matches(
	match_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	code TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	series_id UUID NOT NULL REFERENCES series,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL
);

CREATE TABLE teams(
	team_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	code TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE players(
	player_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	code TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb,
	timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE series_teams(
	series_id UUID NOT NULL REFERENCES series,
	team_id UUID NOT NULL REFERENCES teams,
	PRIMARY KEY(series_id, team_id)
);

CREATE TABLE team_players(
	player_id UUID NOT NULL REFERENCES players,
	team_id UUID NOT NULL REFERENCES teams,
	timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)'),
	PRIMARY KEY(player_id, team_id, timespan)
);

CREATE TABLE player_results(
	player_result_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	player_id UUID NOT NULL REFERENCES players,
	match_id UUID NOT NULL REFERENCES matches,
	result TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE team_results(
	team_result_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	team_id UUID NOT NULL REFERENCES teams,
	match_id UUID NOT NULL REFERENCES matches,
	result TEXT NOT NULL,
	meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX player_result_idx on player_results(result);
CREATE INDEX team_result_idx on team_results(result);
CREATE INDEX competition_timespan_idx on competitions USING gist (timespan);
CREATE INDEX series_timespan_idx on series USING gist (timespan);
CREATE INDEX matches_timespan_idx on matches USING gist (timespan);
CREATE INDEX team_players_timespan_idx on team_players USING gist (timespan);
