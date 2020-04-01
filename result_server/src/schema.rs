table! {
    competitions (competition_id) {
        competition_id -> Uuid,
        code -> Text,
        name -> Text,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

table! {
    matches (match_id) {
        match_id -> Uuid,
        code -> Text,
        name -> Text,
        series_id -> Uuid,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

table! {
    player_results (player_result_id) {
        player_result_id -> Uuid,
        player_id -> Uuid,
        match_id -> Uuid,
        result -> Text,
        meta -> Jsonb,
    }
}

table! {
    players (player_id) {
        player_id -> Uuid,
        code -> Text,
        name -> Text,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

table! {
    series (series_id) {
        series_id -> Uuid,
        code -> Text,
        name -> Text,
        competition_id -> Uuid,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

table! {
    series_teams (series_id, team_id) {
        series_id -> Uuid,
        team_id -> Uuid,
    }
}

table! {
    team_players (player_id, team_id, timespan) {
        player_id -> Uuid,
        team_id -> Uuid,
        timespan -> Tstzrange,
    }
}

table! {
    team_results (team_result_id) {
        team_result_id -> Uuid,
        team_id -> Uuid,
        match_id -> Uuid,
        result -> Text,
        meta -> Jsonb,
    }
}

table! {
    teams (team_id) {
        team_id -> Uuid,
        code -> Text,
        name -> Text,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

joinable!(matches -> series (series_id));
joinable!(player_results -> matches (match_id));
joinable!(player_results -> players (player_id));
joinable!(series -> competitions (competition_id));
joinable!(series_teams -> series (series_id));
joinable!(series_teams -> teams (team_id));
joinable!(team_players -> players (player_id));
joinable!(team_players -> teams (team_id));
joinable!(team_results -> matches (match_id));
joinable!(team_results -> teams (team_id));

allow_tables_to_appear_in_same_query!(
    competitions,
    matches,
    player_results,
    players,
    series,
    series_teams,
    team_players,
    team_results,
    teams,
);
