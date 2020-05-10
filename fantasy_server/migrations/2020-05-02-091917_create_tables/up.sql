-- Your SQL goes here
CREATE TABLE leagues(
    league_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    team_size INT NOT NULL,
    squad_size INT NOT NULL,
    competition_id UUID NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}',
    max_squad_players_same_team INT NOT NULL DEFAULT 256,
    max_squad_players_same_position INT NOT NULL DEFAULT 256,
    max_team_players_same_team INT NOT NULL DEFAULT 256,
    max_team_players_same_position INT NOT NULL DEFAULT 256
);

CREATE TABLE periods(
    period_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    league_id UUID NOT NULL REFERENCES leagues,
    name TEXT NOT NULL,
    timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)'),
    meta JSONB NOT NULL DEFAULT '{}',
    points_multiplier REAL NOT NULL DEFAULT 1.0,
    teams_per_draft INT NOT NULL,
    draft_interval_secs INT NOT NULL,
    draft_start timestamptz NOT NULL
);

CREATE TABLE valid_players(
    period_id UUID NOT NULL REFERENCES periods,
    player_id UUID NOT NULL,
    PRIMARY KEY(period_id, player_id)
);

CREATE TABLE external_users(
    external_user_id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE fantasy_teams(
    fantasy_team_id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    league_id UUID NOT NULL REFERENCES leagues,
    external_user_id UUID NOT NULL REFERENCES external_users,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE fantasy_team_money(
    fantasy_team_id UUID PRIMARY KEY REFERENCES fantasy_teams,
    money_int INT NOT NULL
);

CREATE TABLE commissioners(
    commissioner_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_user_id UUID NOT NULL REFERENCES external_users,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE drafts(
    draft_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    --start timestamptz NOT NULL, Can be inferred from the first draft_choices timespan
    period_id UUID NOT NULL REFERENCES periods,
    meta JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE team_drafts(
    team_draft_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    draft_id UUID NOT NULL REFERENCES drafts,
    fantasy_team_id UUID NOT NULL REFERENCES fantasy_teams,
    UNIQUE(draft_id, fantasy_team_id)
);

CREATE TABLE draft_choices(
    draft_choice_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_draft_id UUID NOT NULL REFERENCES team_drafts,
    timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)')
);

CREATE TABLE picks(
    pick_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fantasy_team_id UUID NOT NULL REFERENCES fantasy_teams,
    draft_choice_id UUID NOT NULL REFERENCES draft_choices,
    player_id UUID NOT NULL,
    timespan TSTZRANGE NOT NULL DEFAULT tstzrange(now(), 'infinity', '[)'),
    active BOOL NOT NULL
);

CREATE TABLE draft_queues(
    fantasy_team_id UUID PRIMARY KEY REFERENCES fantasy_teams,
    player_ids UUID[] NOT NULL DEFAULT ARRAY[]::uuid[]
);

CREATE TABLE stat_multipliers(
    league_id UUID NOT NULL REFERENCES leagues,
    name TEXT UNIQUE NOT NULL,
    multiplier REAL NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}',
    PRIMARY KEY(league_id, name)
);

CREATE INDEX fantasy_team_league_idx on fantasy_teams(league_id);
CREATE INDEX fantasy_team_external_user_idx on fantasy_teams(external_user_id);
CREATE INDEX periods_league_idx on periods(league_id);
--CREATE INDEX stat_multipliers_league_idx on stat_multipliers(league_id); //exist through PKEY
CREATE INDEX drafts_period_idx on drafts(period_id);
CREATE INDEX picks_user_idx on picks(fantasy_team_id);
CREATE INDEX picks_player_idx on picks(player_id);
CREATE INDEX team_drafts_user_idx on team_drafts(fantasy_team_id);
CREATE INDEX team_draft_draft_idx on team_drafts(draft_id);
CREATE INDEX draft_choices_team_draft_id_idx on draft_choices(team_draft_id);
CREATE INDEX picks_draft_choices_idx on picks(draft_choice_id);

CREATE INDEX periods_timespan_idx on periods USING gist (timespan);
CREATE INDEX picks_timespan_idx on picks USING gist (timespan);
CREATE INDEX draft_choices_timespan_idx on draft_choices USING gist (timespan);
ALTER TABLE periods ADD CONSTRAINT non_overlap_period_timespan EXCLUDE USING gist (league_id WITH =, timespan WITH &&);