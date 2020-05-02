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
        draft_user_id -> Uuid,
        timespan -> Tstzrange,
        pick_id -> Nullable<Uuid>,
    }
}

table! {
    draft_queues (user_id) {
        user_id -> Uuid,
        player_ids -> Array<Uuid>,
    }
}

table! {
    drafts (draft_id) {
        draft_id -> Uuid,
        interval_secs -> Int4,
        period_id -> Nullable<Uuid>,
        meta -> Jsonb,
    }
}

table! {
    draft_users (draft_user_id) {
        draft_user_id -> Uuid,
        draft_id -> Uuid,
        user_id -> Uuid,
    }
}

table! {
    external_users (external_user_id) {
        external_user_id -> Uuid,
        username -> Text,
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
        max_players_per_team -> Nullable<Int4>,
        max_players_per_position -> Nullable<Int4>,
    }
}

table! {
    periods (period_id) {
        period_id -> Uuid,
        league_id -> Uuid,
        timespan -> Tstzrange,
        meta -> Jsonb,
        points_multiplier -> Float4,
    }
}

table! {
    picks (pick_id) {
        pick_id -> Uuid,
        user_id -> Uuid,
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
    }
}

table! {
    user_money (user_id) {
        user_id -> Uuid,
        money_int -> Int4,
    }
}

table! {
    users (user_id) {
        user_id -> Uuid,
        username -> Text,
        league_id -> Uuid,
        external_user_id -> Uuid,
        meta -> Jsonb,
    }
}

joinable!(commissioners -> external_users (external_user_id));
joinable!(draft_choices -> draft_users (draft_user_id));
joinable!(draft_choices -> picks (pick_id));
joinable!(draft_queues -> users (user_id));
joinable!(draft_users -> drafts (draft_id));
joinable!(draft_users -> users (user_id));
joinable!(drafts -> periods (period_id));
joinable!(periods -> leagues (league_id));
joinable!(picks -> users (user_id));
joinable!(stat_multipliers -> leagues (league_id));
joinable!(user_money -> users (user_id));
joinable!(users -> external_users (external_user_id));
joinable!(users -> leagues (league_id));

allow_tables_to_appear_in_same_query!(
    commissioners,
    draft_choices,
    draft_queues,
    drafts,
    draft_users,
    external_users,
    leagues,
    periods,
    picks,
    stat_multipliers,
    user_money,
    users,
);
