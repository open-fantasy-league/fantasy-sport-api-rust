-- Your SQL goes here
CREATE TABLE leagues(
    league_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    team_size INT NOT NULL,
    squad_size INT NOT NULL,
    competition_id UUID NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}',
    max_players_per_team INT NOT NULL DEFAULT 256,
    max_players_per_position INT NOT NULL DEFAULT 256
);

CREATE TABLE periods(
    period_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    league_id UUID NOT NULL REFERENCES leagues,
    timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)'),
    meta JSONB NOT NULL DEFAULT '{}',
    points_multiplier REAL NOT NULL DEFAULT 1.0
);

CREATE TABLE external_users(
    external_user_id UUID PRIMARY KEY,
    username TEXT NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE users(
    user_id UUID PRIMARY KEY,
    username TEXT NOT NULL,
    league_id UUID NOT NULL REFERENCES leagues,
    external_user_id UUID NOT NULL REFERENCES external_users,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE user_money(
    user_id UUID PRIMARY KEY REFERENCES users,
    money_int INT NOT NULL
);

CREATE TABLE commissioners(
    commissioner_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_user_id UUID NOT NULL REFERENCES external_users,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE picks(
    pick_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users,
    player_id UUID NOT NULL,
    timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)'),
    active BOOL NOT NULL
);

CREATE TABLE drafts(
    draft_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    --start timestamptz NOT NULL, Can be inferred from the first draft_choices timespan
    interval_secs INT NOT NULL,
    period_id UUID REFERENCES periods,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE draft_users(
    draft_user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    draft_id UUID NOT NULL REFERENCES drafts,
    user_id UUID NOT NULL REFERENCES users,
    UNIQUE(draft_id, user_id)
);

CREATE TABLE draft_choices(
    draft_choice_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    draft_user_id UUID NOT NULL REFERENCES draft_users,
    timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)'),
    pick_id UUID REFERENCES picks
);

CREATE TABLE draft_queues(
    user_id UUID PRIMARY KEY REFERENCES users,
    player_ids UUID[] NOT NULL DEFAULT ARRAY[]::uuid[]
);

CREATE TABLE stat_multipliers(
    league_id UUID NOT NULL REFERENCES leagues,
    name TEXT UNIQUE NOT NULL,
    multiplier REAL NOT NULL,
    PRIMARY KEY(league_id, name)
);

CREATE INDEX user_league_idx on users(league_id);
CREATE INDEX user_external_user_idx on users(external_user_id);
CREATE INDEX periods_league_idx on periods(league_id);
--CREATE INDEX stat_multipliers_league_idx on stat_multipliers(league_id); //exist through PKEY
CREATE INDEX drafts_period_idx on drafts(period_id);
CREATE INDEX picks_user_idx on picks(user_id);
CREATE INDEX picks_player_idx on picks(player_id);
CREATE INDEX draft_users_user_idx on draft_users(user_id);
CREATE INDEX draft_users_draft_idx on draft_users(draft_id);
CREATE INDEX draft_choices_draft_user_id_idx on draft_choices(draft_user_id);

CREATE INDEX periods_timespan_idx on periods USING gist (timespan);
CREATE INDEX picks_timespan_idx on picks USING gist (timespan);
CREATE INDEX draft_choices_timespan_idx on draft_choices USING gist (timespan);
