table! {
    commissioners (commissioner_id) {
        commissioner_id -> Uuid,
        external_user_id -> Uuid,
        meta -> Jsonb,
    }
}

table! {
    draft_choices (draft_choice_id) {
        draft_choice_id -> Uuid,
        team_draft_id -> Uuid,
        timespan -> Tstzrange,
    }
}

table! {
    draft_queues (fantasy_team_id) {
        fantasy_team_id -> Uuid,
        player_ids -> Array<Uuid>,
    }
}

table! {
    drafts (draft_id) {
        draft_id -> Uuid,
        period_id -> Uuid,
        meta -> Jsonb,
    }
}

table! {
    external_users (external_user_id) {
        external_user_id -> Uuid,
        name -> Text,
        meta -> Jsonb,
    }
}

table! {
    fantasy_team_money (fantasy_team_id) {
        fantasy_team_id -> Uuid,
        money_int -> Int4,
    }
}

table! {
    fantasy_teams (fantasy_team_id) {
        fantasy_team_id -> Uuid,
        name -> Text,
        league_id -> Uuid,
        external_user_id -> Uuid,
        meta -> Jsonb,
    }
}

table! {
    leagues (league_id) {
        league_id -> Uuid,
        name -> Text,
        team_size -> Int4,
        squad_size -> Int4,
        competition_id -> Uuid,
        meta -> Jsonb,
        max_squad_players_same_team -> Int4,
        max_squad_players_same_position -> Int4,
        max_team_players_same_team -> Int4,
        max_team_players_same_position -> Int4,
    }
}

table! {
    periods (period_id) {
        period_id -> Uuid,
        league_id -> Uuid,
        name -> Text,
        timespan -> Tstzrange,
        meta -> Jsonb,
        points_multiplier -> Float4,
        teams_per_draft -> Int4,
        draft_interval_secs -> Int4,
        draft_start -> Timestamptz,
    }
}

table! {
    picks (pick_id) {
        pick_id -> Uuid,
        fantasy_team_id -> Uuid,
        draft_choice_id -> Uuid,
        player_id -> Uuid,
        timespan -> Tstzrange,
        active -> Bool,
    }
}

table! {
    stat_multipliers (league_id, name) {
        league_id -> Uuid,
        name -> Text,
        multiplier -> Float4,
        meta -> Jsonb,
    }
}

table! {
    team_drafts (team_draft_id) {
        team_draft_id -> Uuid,
        draft_id -> Uuid,
        fantasy_team_id -> Uuid,
    }
}

table! {
    valid_pick_ids (period_id, player_id) {
        period_id -> Uuid,
        player_id -> Uuid,
    }
}

joinable!(commissioners -> external_users (external_user_id));
joinable!(draft_choices -> team_drafts (team_draft_id));
joinable!(draft_queues -> fantasy_teams (fantasy_team_id));
joinable!(drafts -> periods (period_id));
joinable!(fantasy_team_money -> fantasy_teams (fantasy_team_id));
joinable!(fantasy_teams -> external_users (external_user_id));
joinable!(fantasy_teams -> leagues (league_id));
joinable!(periods -> leagues (league_id));
joinable!(picks -> draft_choices (draft_choice_id));
joinable!(picks -> fantasy_teams (fantasy_team_id));
joinable!(stat_multipliers -> leagues (league_id));
joinable!(team_drafts -> drafts (draft_id));
joinable!(team_drafts -> fantasy_teams (fantasy_team_id));
joinable!(valid_pick_ids -> periods (period_id));

allow_tables_to_appear_in_same_query!(
    commissioners,
    draft_choices,
    draft_queues,
    drafts,
    external_users,
    fantasy_team_money,
    fantasy_teams,
    leagues,
    periods,
    picks,
    stat_multipliers,
    team_drafts,
    valid_pick_ids,
);
