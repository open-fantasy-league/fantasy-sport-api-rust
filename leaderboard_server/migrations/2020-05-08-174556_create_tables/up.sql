-- Your SQL goes here
--CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
-- https://computingforgeeks.com/how-to-install-timescaledb-on-ubuntu-18-04-lts/

-- other stats for tiebreakers?
CREATE TABLE points(
    leaderboard_id UUID NOT NULL REFERENCES leaderboard,
    timestamp timestamptz NOT NULL DEFAULT now(),
    points REAL NOT NULL
);

CREATE TABLE leaderboard(
    leaderboard_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    league_id UUID,
    competition_id UUID,
    name TEXT NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}',
);
SELECT create_hypertable('points', 'timestamp', 'leaderboard_id', 2, create_default_indexes=>FALSE);
CREATE INDEX points_league_id_idx on points(league_id);
CREATE INDEX points_competition_id_idx on points(competition_id);