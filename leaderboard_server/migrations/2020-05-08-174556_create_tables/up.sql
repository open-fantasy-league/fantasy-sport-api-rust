-- Your SQL goes here
-- CREATE EXTENSION IF NOT EXISTS pgcrypto;
-- CREATE EXTENSION IF NOT EXISTS btree_gist;
-- CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
-- https://computingforgeeks.com/how-to-install-timescaledb-on-ubuntu-18-04-lts/

-- other stats for tiebreakers?
-- think its more flexible to just have timestamps, rather than trying to have "daily/weekly/etc"
-- with timestamp people have flexibility to choose when they want to update. Also easier to show historic rankings
-- also for trying to rank by other stats (i.e. rank by wins n stuff...just have two separate tables. EZ)

-- https://wiki.postgresql.org/images/1/1b/Ranges%2C_Partitioning_and_Limitations.pdf
--useful for future reference maybe

CREATE TABLE leaderboards(
    leaderboard_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    league_id UUID NOT NULL,
    name TEXT NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}',
    timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE stats(
    player_id UUID NOT NULL,
    leaderboard_id UUID NOT NULL REFERENCES leaderboards,
    timestamp timestamptz NOT NULL DEFAULT now(),
    points REAL NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}',
    PRIMARY KEY (player_id, leaderboard_id, timestamp)
);
-- 1 for hashing thingy num-parititioons cos internet man said 1-to-1 with number of logical hard-drives
SELECT create_hypertable('stats', 'timestamp', 'leaderboard_id', 1);
CREATE INDEX leaderboard_league_id_idx on leaderboards(league_id);
CREATE INDEX stats_player_id_idx on stats(player_id);